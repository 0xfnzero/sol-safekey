//! Bot Helper Module
//!
//! This module provides easy-to-use functions for bot integration.
//! It allows bots to call the CLI tool for key management without implementing their own CLI.

use std::process::{Command, Stdio};
use std::path::Path;

/// Result type for bot helper operations
pub type BotResult<T> = Result<T, String>;

/// Bot Key Manager - High-level wrapper for CLI operations
///
/// This struct provides simple methods to interact with the sol-safekey CLI
/// from your bot application.
pub struct BotKeyManager {
    cli_path: String,
}

impl BotKeyManager {
    /// Create a new BotKeyManager
    ///
    /// # Arguments
    ///
    /// * `cli_path` - Path to the sol-safekey CLI binary (default: "sol-safekey")
    ///
    /// # Example
    ///
    /// ```rust
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// // Use system-installed CLI
    /// let manager = BotKeyManager::new();
    ///
    /// // Or use custom path
    /// let manager = BotKeyManager::with_path("./target/release/sol-safekey");
    /// ```
    pub fn new() -> Self {
        Self {
            cli_path: "sol-safekey".to_string(),
        }
    }

    /// Create a BotKeyManager with custom CLI path
    pub fn with_path(cli_path: &str) -> Self {
        Self {
            cli_path: cli_path.to_string(),
        }
    }

    /// Generate a keystore file (will prompt for password interactively)
    ///
    /// This calls `sol-safekey gen-keystore` and prompts the user for a password.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Where to save the keystore file
    ///
    /// # Returns
    ///
    /// The public key of the generated wallet
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// let manager = BotKeyManager::new();
    /// let public_key = manager.generate_keystore_interactive("wallet.json")?;
    /// println!("Generated wallet with public key: {}", public_key);
    /// # Ok::<(), String>(())
    /// ```
    pub fn generate_keystore_interactive(&self, output_path: &str) -> BotResult<String> {
        println!("🔐 Generating new encrypted keystore...");
        println!("📝 You will be prompted to enter a password.\n");

        let output = Command::new(&self.cli_path)
            .arg("gen-keystore")
            .arg("-o")
            .arg(output_path)
            .stdin(Stdio::inherit())  // Allow interactive password input
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute CLI: {}. Make sure sol-safekey is installed.", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to generate keystore: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract public key from output
        for line in stdout.lines() {
            if line.contains("Public Key:") || line.contains("公钥:") {
                if let Some(pubkey) = line.split(':').nth(1) {
                    return Ok(pubkey.trim().to_string());
                }
            }
        }

        // Fallback: read from generated file
        self.get_public_key_from_file(output_path)
    }

    /// Unlock a keystore file (will prompt for password interactively)
    ///
    /// This calls `sol-safekey unlock` and prompts the user for a password.
    /// Returns the decrypted private key in base58 format.
    ///
    /// # Arguments
    ///
    /// * `keystore_path` - Path to the keystore file
    ///
    /// # Returns
    ///
    /// The decrypted private key as a base58 string
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// let manager = BotKeyManager::new();
    /// let private_key = manager.unlock_keystore_interactive("wallet.json")?;
    /// println!("Wallet unlocked successfully!");
    /// # Ok::<(), String>(())
    /// ```
    pub fn unlock_keystore_interactive(&self, keystore_path: &str) -> BotResult<String> {
        if !Path::new(keystore_path).exists() {
            return Err(format!("Keystore file not found: {}", keystore_path));
        }

        println!("🔓 Unlocking keystore: {}", keystore_path);
        println!("🔑 Please enter your password:\n");

        let output = Command::new(&self.cli_path)
            .arg("unlock")
            .arg("-f")
            .arg(keystore_path)
            .stdin(Stdio::inherit())  // Allow interactive password input
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute CLI: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to unlock keystore: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract private key from output
        // The CLI prints the key on a separate line after "🔑 解密后的私钥:" or "🔑 Private Key:"
        let lines: Vec<&str> = stdout.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("解密后的私钥") || line.contains("Private Key") || line.contains("🔑 私钥:") {
                // The private key is on the next line
                if i + 1 < lines.len() {
                    let key = lines[i + 1].trim();
                    if !key.is_empty() && key.len() > 30 {  // Basic validation
                        return Ok(key.to_string());
                    }
                }
            }
        }

        Err("Could not extract private key from CLI output".to_string())
    }

    /// Unlock keystore with password provided programmatically (non-interactive)
    ///
    /// Use this when you already have the password in your bot code.
    ///
    /// # Arguments
    ///
    /// * `keystore_path` - Path to the keystore file
    /// * `password` - The password to decrypt
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// let manager = BotKeyManager::new();
    /// let private_key = manager.unlock_keystore_with_password(
    ///     "wallet.json",
    ///     "my_password"
    /// )?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn unlock_keystore_with_password(&self, keystore_path: &str, password: &str) -> BotResult<String> {
        if !Path::new(keystore_path).exists() {
            return Err(format!("Keystore file not found: {}", keystore_path));
        }

        let output = Command::new(&self.cli_path)
            .arg("unlock")
            .arg("-f")
            .arg(keystore_path)
            .arg("-p")
            .arg(password)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute CLI: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to unlock keystore: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract private key from output
        // The CLI prints the key on a separate line after "🔑 解密后的私钥:" or "🔑 Private Key:"
        let lines: Vec<&str> = stdout.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("解密后的私钥") || line.contains("Private Key") || line.contains("🔑 私钥:") {
                // The private key is on the next line
                if i + 1 < lines.len() {
                    let key = lines[i + 1].trim();
                    if !key.is_empty() && key.len() > 30 {  // Basic validation
                        return Ok(key.to_string());
                    }
                }
            }
        }

        Err("Could not extract private key from CLI output".to_string())
    }

    /// Get public key from a keystore file without unlocking
    ///
    /// # Arguments
    ///
    /// * `keystore_path` - Path to the keystore file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// let manager = BotKeyManager::new();
    /// let pubkey = manager.get_public_key_from_file("wallet.json")?;
    /// println!("Wallet public key: {}", pubkey);
    /// # Ok::<(), String>(())
    /// ```
    pub fn get_public_key_from_file(&self, keystore_path: &str) -> BotResult<String> {
        use std::fs;
        use serde_json::Value;

        let content = fs::read_to_string(keystore_path)
            .map_err(|e| format!("Failed to read keystore: {}", e))?;

        let data: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid keystore format: {}", e))?;

        data["public_key"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Public key not found in keystore".to_string())
    }

    /// Check if the CLI binary is available
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// let manager = BotKeyManager::new();
    /// if manager.check_cli_available() {
    ///     println!("✅ sol-safekey CLI is available");
    /// } else {
    ///     println!("❌ sol-safekey CLI not found");
    /// }
    /// ```
    pub fn check_cli_available(&self) -> bool {
        Command::new(&self.cli_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Generate a new keystore with password from environment variable
    ///
    /// Reads password from the specified environment variable.
    /// Useful for automated deployment.
    ///
    /// # Arguments
    ///
    /// * `output_path` - Where to save the keystore
    /// * `password_env_var` - Name of environment variable containing password
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sol_safekey::bot_helper::BotKeyManager;
    ///
    /// // Set environment variable first:
    /// // export WALLET_PASSWORD="my_secure_password"
    ///
    /// let manager = BotKeyManager::new();
    /// let pubkey = manager.generate_keystore_from_env(
    ///     "wallet.json",
    ///     "WALLET_PASSWORD"
    /// )?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn generate_keystore_from_env(&self, output_path: &str, password_env_var: &str) -> BotResult<String> {
        let password = std::env::var(password_env_var)
            .map_err(|_| format!("Environment variable {} not found", password_env_var))?;

        let output = Command::new(&self.cli_path)
            .arg("gen-keystore")
            .arg("-o")
            .arg(output_path)
            .arg("-p")
            .arg(&password)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute CLI: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to generate keystore: {}", stderr));
        }

        self.get_public_key_from_file(output_path)
    }
}

impl Default for BotKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick helper function to unlock a keystore interactively
///
/// This is a convenience function for the most common use case.
///
/// # Example
///
/// ```no_run
/// use sol_safekey::bot_helper::unlock_keystore;
///
/// let private_key = unlock_keystore("wallet.json")?;
/// # Ok::<(), String>(())
/// ```
pub fn unlock_keystore(keystore_path: &str) -> BotResult<String> {
    BotKeyManager::new().unlock_keystore_interactive(keystore_path)
}

/// Quick helper function to generate a keystore interactively
///
/// # Example
///
/// ```no_run
/// use sol_safekey::bot_helper::generate_keystore;
///
/// let public_key = generate_keystore("wallet.json")?;
/// println!("Generated wallet: {}", public_key);
/// # Ok::<(), String>(())
/// ```
pub fn generate_keystore(output_path: &str) -> BotResult<String> {
    BotKeyManager::new().generate_keystore_interactive(output_path)
}

/// Quick helper to check if CLI is available
///
/// # Example
///
/// ```no_run
/// use sol_safekey::bot_helper::check_cli;
///
/// if !check_cli() {
///     eprintln!("Please install sol-safekey CLI first!");
///     std::process::exit(1);
/// }
/// ```
pub fn check_cli() -> bool {
    BotKeyManager::new().check_cli_available()
}