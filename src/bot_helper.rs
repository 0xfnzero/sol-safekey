//! Bot Helper Module
//!
//! Provides easy-to-use functions for bot integration with sol-safekey.
//! Bots can integrate the interactive key management tool with just 1 line of code.
//! **No CLI dependency required** - uses the library directly.
//!
//! # Quick Start
//!
//! ```no_run
//! use sol_safekey::bot_helper;
//!
//! let keypair = bot_helper::ensure_wallet_ready("wallet.json").unwrap();
//! ```

use std::path::Path;
use std::fs;
use std::io::Write;
use serde_json::Value;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use crate::{decrypt_key, generate_encryption_key_simple, interactive};

/// Result type for bot helper operations
pub type Result<T> = std::result::Result<T, String>;

/// Ensure wallet is ready to use - creates or unlocks interactively
///
/// Main function for bot integration:
/// - If wallet exists: prompts for password and unlocks
/// - If wallet doesn't exist: launches interactive creation, then unlocks
///
/// # Arguments
///
/// * `wallet_path` - Path to the encrypted wallet file
///
/// # Example
///
/// ```no_run
/// use sol_safekey::bot_helper;
/// use solana_sdk::signer::Signer;
///
/// let keypair = bot_helper::ensure_wallet_ready("wallet.json")?;
/// println!("Wallet ready: {}", keypair.pubkey());
/// # Ok::<(), String>(())
/// ```
pub fn ensure_wallet_ready(wallet_path: &str) -> Result<Keypair> {
    if wallet_exists(wallet_path) {
        println!("✅ Wallet found at: {}", wallet_path);
        println!("🔓 Starting interactive wallet unlock...\n");
        unlock_wallet(wallet_path)
    } else {
        println!("⚠️  Wallet not found at: {}", wallet_path);
        println!("📝 Starting interactive wallet creation...\n");
        create_wallet(wallet_path)?;

        println!("\nNow unlocking the newly created wallet...\n");
        unlock_wallet(wallet_path)
    }
}

/// Check if a wallet file exists at the given path
pub fn wallet_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Get public key from wallet without unlocking
///
/// # Arguments
///
/// * `wallet_path` - Path to the encrypted wallet file
pub fn get_wallet_pubkey(wallet_path: &str) -> Result<String> {
    let content = fs::read_to_string(wallet_path)
        .map_err(|e| format!("Failed to read wallet: {}", e))?;

    let data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid wallet format: {}", e))?;

    data["public_key"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "Public key not found in wallet file".to_string())
}

/// Create wallet interactively
fn create_wallet(output_path: &str) -> Result<()> {
    println!("🔐 Interactive Wallet Creation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nYou will be guided through the wallet creation process.");
    println!("The wallet will be saved to: {}\n", output_path);

    interactive::show_main_menu()?;

    if !wallet_exists(output_path) {
        println!("\n⚠️  Note: Wallet was not saved to the expected path.");
        println!("   Expected: {}", output_path);
        println!("   Please make sure to save your wallet to this location.");
        return Err(format!("Wallet not created at expected path: {}", output_path));
    }

    println!("\n✅ Wallet created successfully!");
    println!("📁 Location: {}", output_path);
    Ok(())
}

/// Unlock wallet interactively (prompts for password)
fn unlock_wallet(wallet_path: &str) -> Result<Keypair> {
    if !wallet_exists(wallet_path) {
        return Err(format!("Wallet file not found: {}", wallet_path));
    }

    println!("🔓 Unlocking wallet: {}", wallet_path);

    let content = fs::read_to_string(wallet_path)
        .map_err(|e| format!("Failed to read wallet: {}", e))?;

    let data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid wallet format: {}", e))?;

    let encrypted_key = data["encrypted_private_key"]
        .as_str()
        .ok_or_else(|| "Encrypted private key not found in wallet file".to_string())?;

    print!("🔑 Enter wallet password: ");
    std::io::stdout().flush().unwrap();

    let password = rpassword::read_password()
        .map_err(|e| format!("Failed to read password: {}", e))?;

    let encryption_key = generate_encryption_key_simple(&password);
    let private_key = decrypt_key(encrypted_key, &encryption_key)?;
    let keypair = Keypair::from_base58_string(&private_key);

    println!("✅ Wallet unlocked successfully!");
    println!("📍 Address: {}", keypair.pubkey());

    Ok(keypair)
}
