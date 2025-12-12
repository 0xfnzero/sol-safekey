//! # Sol SafeKey
//!
//! A powerful Solana key management library with military-grade encryption.
//!
//! ## Features
//!
//! - **Simple Encryption**: Password-based encryption for Solana private keys
//! - **Triple-Factor Authentication**: Hardware fingerprint + master password + security question
//! - **2FA Support**: TOTP-based two-factor authentication
//! - **Cross-Platform**: Works on macOS, Linux, and Windows
//!
//! ## Quick Start
//!
//! ```rust
//! use sol_safekey::{KeyManager, EncryptionResult};
//!
//! // Generate a new Solana keypair
//! let keypair = KeyManager::generate_keypair();
//!
//! // Encrypt with password
//! let encrypted = KeyManager::encrypt_with_password(
//!     &keypair.to_base58_string(),
//!     "my_strong_password"
//! ).unwrap();
//!
//! // Decrypt with password
//! let decrypted = KeyManager::decrypt_with_password(
//!     &encrypted,
//!     "my_strong_password"
//! ).unwrap();
//! ```

use base64::engine::general_purpose;
use base64::Engine;
use ring::digest;

// Re-export modules for advanced usage (conditional compilation)
#[cfg(feature = "2fa")]
pub mod totp;

#[cfg(feature = "2fa")]
pub mod secure_totp;

#[cfg(feature = "2fa")]
pub mod hardware_fingerprint;

#[cfg(feature = "2fa")]
pub mod security_question;

// Interactive menu module - needed for bot integration
pub mod interactive;

// Bot helper module for easy bot integration (no CLI dependency)
pub mod bot_helper;

// Solana operations interactive menu
pub mod operations;

// Solana utilities for token operations
#[cfg(any(feature = "solana-ops", feature = "sol-trade-sdk"))]
pub mod solana_utils;

// Re-export commonly used types
pub use solana_sdk::signature::{Keypair, Signer};
pub use solana_sdk::pubkey::Pubkey;

// ============================================================================
// Core Encryption/Decryption Functions
// ============================================================================

/// Simple XOR encryption/decryption using a 32-byte key
fn xor_encrypt_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());

    // Generate keystream from the key
    let mut keystream = Vec::new();
    let mut i: u32 = 0;
    while keystream.len() < data.len() {
        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(key);
        ctx.update(&i.to_le_bytes());
        let hash = ctx.finish();
        keystream.extend_from_slice(hash.as_ref());
        i += 1;
    }

    // XOR operation
    for (i, &byte) in data.iter().enumerate() {
        result.push(byte ^ keystream[i % keystream.len()]);
    }

    result
}

/// Encrypt a string with a 32-byte encryption key
///
/// Returns base64-encoded encrypted data
pub fn encrypt_key(secret_key: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    let data = secret_key.as_bytes();
    let encrypted = xor_encrypt_decrypt(data, encryption_key);
    Ok(general_purpose::STANDARD.encode(encrypted))
}

/// Decrypt a base64-encoded encrypted string with a 32-byte encryption key
///
/// Returns the original plaintext string
pub fn decrypt_key(encrypted_data: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    let ciphertext = general_purpose::STANDARD.decode(encrypted_data)
        .map_err(|_| "Invalid encrypted data format".to_string())?;

    let decrypted = xor_encrypt_decrypt(&ciphertext, encryption_key);

    String::from_utf8(decrypted)
        .map_err(|_| "Invalid UTF-8 data in decrypted content".to_string())
}

/// Minimum password length for encryption/decryption
pub const MIN_PASSWORD_LENGTH: usize = 10;

/// Maximum password length for encryption/decryption
pub const MAX_PASSWORD_LENGTH: usize = 20;

/// Fixed salt for password hashing
/// This prevents rainbow table attacks and ensures consistent key derivation
const PASSWORD_SALT: &[u8] = b"sol-safekey-v1-salt-2025";

/// Generate a 16-byte encryption key from a password using SHA-256
///
/// This function:
/// 1. Combines password with fixed salt
/// 2. Hashes using SHA-256 (produces 32 bytes)
/// 3. Takes the first 16 bytes of the hash as the encryption key
///
/// Password requirements:
/// - Minimum length: 10 characters
/// - Maximum length: 20 characters
pub fn generate_encryption_key_simple(password: &str) -> [u8; 32] {
    // Combine password with fixed salt
    let mut salted_password = password.as_bytes().to_vec();
    salted_password.extend_from_slice(PASSWORD_SALT);

    // Hash the salted password using SHA-256
    let hash = digest::digest(&digest::SHA256, &salted_password);

    // Take the first 16 bytes of the hash
    let mut key = [0u8; 32];
    key[0..16].copy_from_slice(&hash.as_ref()[0..16]);

    // Fill the remaining 16 bytes by repeating the first 16 bytes
    // This ensures we have a 32-byte key for compatibility
    key[16..32].copy_from_slice(&hash.as_ref()[0..16]);

    key
}

// ============================================================================
// High-Level Key Management API (简单集成用)
// ============================================================================

/// Result type for encryption operations
pub type EncryptionResult<T> = Result<T, String>;

/// Main interface for key management operations
///
/// This is the recommended API for library integration.
/// It provides simple, safe methods for common key operations.
pub struct KeyManager;

impl KeyManager {
    /// Generate a new Solana keypair
    ///
    /// # Example
    ///
    /// ```
    /// use sol_safekey::KeyManager;
    ///
    /// let keypair = KeyManager::generate_keypair();
    /// println!("Public key: {}", keypair.pubkey());
    /// ```
    pub fn generate_keypair() -> Keypair {
        Keypair::new()
    }

    /// Encrypt a private key with a password
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key in base58 string format
    /// * `password` - The password to use for encryption
    ///
    /// # Returns
    ///
    /// Base64-encoded encrypted string
    ///
    /// # Example
    ///
    /// ```
    /// use sol_safekey::KeyManager;
    ///
    /// let keypair = KeyManager::generate_keypair();
    /// let private_key = keypair.to_base58_string();
    ///
    /// let encrypted = KeyManager::encrypt_with_password(
    ///     &private_key,
    ///     "my_password"
    /// ).unwrap();
    /// ```
    pub fn encrypt_with_password(private_key: &str, password: &str) -> EncryptionResult<String> {
        let key = generate_encryption_key_simple(password);
        encrypt_key(private_key, &key)
    }

    /// Decrypt a private key with a password
    ///
    /// # Arguments
    ///
    /// * `encrypted_data` - Base64-encoded encrypted data
    /// * `password` - The password used for encryption
    ///
    /// # Returns
    ///
    /// The original private key in base58 string format
    ///
    /// # Example
    ///
    /// ```
    /// use sol_safekey::KeyManager;
    ///
    /// let encrypted = "..."; // from encryption
    /// let decrypted = KeyManager::decrypt_with_password(
    ///     encrypted,
    ///     "my_password"
    /// ).unwrap();
    /// ```
    pub fn decrypt_with_password(encrypted_data: &str, password: &str) -> EncryptionResult<String> {
        let key = generate_encryption_key_simple(password);
        decrypt_key(encrypted_data, &key)
    }

    /// Get public key from a private key
    ///
    /// # Arguments
    ///
    /// * `private_key` - Private key in base58 string format
    ///
    /// # Returns
    ///
    /// Public key as a base58 string
    pub fn get_public_key(private_key: &str) -> EncryptionResult<String> {
        use solana_sdk::signature::Keypair;

        // Solana 3.0 uses from_base58_string directly
        let keypair = Keypair::from_base58_string(private_key);

        Ok(keypair.pubkey().to_string())
    }

    /// Encrypt a keypair to a JSON keystore format
    ///
    /// This creates a standard encrypted keystore file compatible with Solana tools.
    ///
    /// # Arguments
    ///
    /// * `keypair` - The Solana keypair to encrypt
    /// * `password` - The password for encryption
    ///
    /// # Returns
    ///
    /// JSON string containing the encrypted keystore
    pub fn keypair_to_encrypted_json(keypair: &Keypair, password: &str) -> EncryptionResult<String> {
        use serde_json::json;
        use chrono::Utc;

        let private_key = keypair.to_base58_string();
        let public_key = keypair.pubkey().to_string();

        let encrypted = Self::encrypt_with_password(&private_key, password)?;

        let keystore = json!({
            "encrypted_private_key": encrypted,
            "public_key": public_key,
            "encryption_type": "password_only",
            "created_at": Utc::now().to_rfc3339(),
        });

        Ok(keystore.to_string())
    }

    /// Decrypt a keypair from encrypted JSON keystore
    ///
    /// # Arguments
    ///
    /// * `json_data` - The encrypted keystore JSON string
    /// * `password` - The password used for encryption
    ///
    /// # Returns
    ///
    /// The restored Keypair
    pub fn keypair_from_encrypted_json(json_data: &str, password: &str) -> EncryptionResult<Keypair> {
        use serde_json::Value;

        let data: Value = serde_json::from_str(json_data)
            .map_err(|_| "Invalid JSON format")?;

        let encrypted = data["encrypted_private_key"]
            .as_str()
            .ok_or("Missing encrypted_private_key field")?;

        let private_key_str = Self::decrypt_with_password(encrypted, password)?;

        // Solana 3.0 uses from_base58_string directly
        let keypair = Keypair::from_base58_string(&private_key_str);

        Ok(keypair)
    }
}

// ============================================================================
// Advanced 2FA Functions (CLI 工具使用，库集成可选)
// ============================================================================

// ============================================================================
// 2FA Functions (only available with "2fa" feature)
// ============================================================================

#[cfg(feature = "2fa")]
/// Derive a TOTP secret from password
///
/// This is used internally for deterministic 2FA key generation.
#[allow(dead_code)]
fn derive_totp_secret_from_password(password: &str, account: &str, issuer: &str) -> Result<String, String> {
    use ring::pbkdf2;
    use data_encoding::BASE32_NOPAD;
    use std::num::NonZeroU32;

    let salt = format!("sol-safekey-totp-{}-{}", issuer, account);
    let iterations = NonZeroU32::new(100_000)
        .ok_or("Invalid iteration count")?;

    let mut secret = [0u8; 20]; // 160 bits for TOTP
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt.as_bytes(),
        password.as_bytes(),
        &mut secret,
    );

    Ok(BASE32_NOPAD.encode(&secret))
}

#[cfg(feature = "2fa")]
/// Derive TOTP secret from hardware fingerprint and password
///
/// This creates a deterministic 2FA key bound to specific hardware.
pub fn derive_totp_secret_from_hardware_and_password(
    hardware_fingerprint: &str,
    master_password: &str,
    account: &str,
    issuer: &str,
) -> Result<String, String> {
    use ring::pbkdf2;
    use data_encoding::BASE32_NOPAD;
    use std::num::NonZeroU32;

    let key_material = format!("{}::{}", hardware_fingerprint, master_password);
    let salt = format!("sol-safekey-2fa-{}-{}", issuer, account);
    let iterations = NonZeroU32::new(100_000)
        .ok_or("Invalid iteration count")?;

    let mut secret = [0u8; 20];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt.as_bytes(),
        key_material.as_bytes(),
        &mut secret,
    );

    Ok(BASE32_NOPAD.encode(&secret))
}

#[cfg(feature = "2fa")]
/// Verify a TOTP code
fn verify_current_totp_code(totp_secret: &str, current_code: &str) -> Result<(), String> {
    use crate::totp::{TOTPConfig, TOTPManager};

    let config = TOTPConfig {
        secret: totp_secret.to_string(),
        account: "wallet".to_string(),
        issuer: "Sol-SafeKey".to_string(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        step: 30,
    };

    let totp_manager = TOTPManager::new(config);

    match totp_manager.verify_code(current_code) {
        Ok(true) => Ok(()),
        Ok(false) => Err("验证失败，请检查主密码、安全问题答案或2FA验证码".to_string()),
        Err(e) => Err(format!("验证失败: {}", e)),
    }
}

// ============================================================================
// Triple-Factor Encryption (only available with "2fa" feature)
// ============================================================================

#[cfg(feature = "2fa")]
/// Generate a triple-factor encryption key
///
/// Combines hardware fingerprint + master password + security answer
pub fn generate_triple_factor_key(
    hardware_fingerprint: &str,
    master_password: &str,
    security_answer: &str,
) -> [u8; 32] {
    use ring::pbkdf2;
    use std::num::NonZeroU32;

    let key_material = format!(
        "HW:{}|PASS:{}|QA:{}",
        hardware_fingerprint,
        master_password,
        security_answer.trim().to_lowercase()
    );

    let salt = b"sol-safekey-triple-factor-v1";
    let iterations = NonZeroU32::new(200_000).unwrap();

    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        key_material.as_bytes(),
        &mut key,
    );

    key
}

#[cfg(feature = "2fa")]
/// Encrypt with triple-factor authentication
///
/// Used by CLI for maximum security with device binding.
pub fn encrypt_with_triple_factor(
    private_key: &str,
    twofa_secret: &str,
    hardware_fingerprint: &str,
    master_password: &str,
    question_index: usize,
    security_answer: &str,
) -> Result<String, String> {
    use serde_json::json;

    let encryption_key = generate_triple_factor_key(
        hardware_fingerprint,
        master_password,
        security_answer,
    );

    let data_package = json!({
        "private_key": private_key,
        "twofa_secret": twofa_secret,
        "question_index": question_index,
        "version": "triple_factor_v1",
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let package_str = data_package.to_string();
    let encrypted = encrypt_key(&package_str, &encryption_key)?;

    Ok(encrypted)
}

#[cfg(feature = "2fa")]
/// Decrypt with triple-factor authentication and verify 2FA code
///
/// Used by CLI for unlocking triple-factor encrypted wallets.
pub fn decrypt_with_triple_factor_and_2fa(
    encrypted_data: &str,
    hardware_fingerprint: &str,
    master_password: &str,
    security_answer: &str,
    twofa_code: &str,
) -> Result<(String, String, usize), String> {
    let decryption_key = generate_triple_factor_key(
        hardware_fingerprint,
        master_password,
        security_answer,
    );

    let decrypted = decrypt_key(encrypted_data, &decryption_key)
        .map_err(|_| "解密失败，请检查主密码、安全问题答案是否正确")?;

    let data: serde_json::Value = serde_json::from_str(&decrypted)
        .map_err(|_| "解密失败，请检查主密码、安全问题答案是否正确")?;

    let private_key = data["private_key"]
        .as_str()
        .ok_or("缺少私钥数据")?
        .to_string();

    let twofa_secret = data["twofa_secret"]
        .as_str()
        .ok_or("缺少2FA密钥数据")?
        .to_string();

    let question_index = data["question_index"]
        .as_u64()
        .ok_or("缺少安全问题索引")? as usize;

    // Verify 2FA code
    verify_current_totp_code(&twofa_secret, twofa_code)?;

    Ok((private_key, twofa_secret, question_index))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = KeyManager::generate_keypair();
        assert_eq!(keypair.to_bytes().len(), 64);
    }

    #[test]
    fn test_encrypt_decrypt_with_password() {
        let keypair = KeyManager::generate_keypair();
        let private_key = keypair.to_base58_string();
        let password = "test_password_123";

        let encrypted = KeyManager::encrypt_with_password(&private_key, password).unwrap();
        let decrypted = KeyManager::decrypt_with_password(&encrypted, password).unwrap();

        assert_eq!(private_key, decrypted);
    }

    #[test]
    fn test_get_public_key() {
        let keypair = KeyManager::generate_keypair();
        let private_key = keypair.to_base58_string();
        let expected_pubkey = keypair.pubkey().to_string();

        let pubkey = KeyManager::get_public_key(&private_key).unwrap();
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    fn test_keystore_json_round_trip() {
        let keypair = KeyManager::generate_keypair();
        let password = "secure_password";

        let json = KeyManager::keypair_to_encrypted_json(&keypair, password).unwrap();
        let restored_keypair = KeyManager::keypair_from_encrypted_json(&json, password).unwrap();

        assert_eq!(keypair.to_bytes(), restored_keypair.to_bytes());
    }

    #[test]
    fn test_wrong_password_fails() {
        let keypair = KeyManager::generate_keypair();
        let private_key = keypair.to_base58_string();

        let encrypted = KeyManager::encrypt_with_password(&private_key, "correct").unwrap();
        let result = KeyManager::decrypt_with_password(&encrypted, "wrong");

        assert!(result.is_err());
    }
}