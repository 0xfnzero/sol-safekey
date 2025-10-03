# Sol-SafeKey Integration Guide

A complete guide for integrating Sol-SafeKey into your Rust projects.

> 📖 **中文文档**: [INTEGRATION_CN.md](./INTEGRATION_CN.md)

## Quick Start

### 1. Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = "0.1.0"
```

This installs the **minimal library** without CLI dependencies - perfect for integration into your Rust projects.

### 2. Basic Usage

```rust
use sol_safekey::KeyManager;

fn main() {
    // Generate new Solana keypair
    let keypair = KeyManager::generate_keypair();
    println!("Public key: {}", keypair.pubkey());

    // Get private key string
    let private_key = keypair.to_base58_string();

    // Encrypt with password
    let encrypted = KeyManager::encrypt_with_password(
        &private_key,
        "your_strong_password"
    ).unwrap();
    println!("Encrypted: {}", encrypted);

    // Decrypt
    let decrypted = KeyManager::decrypt_with_password(
        &encrypted,
        "your_strong_password"
    ).unwrap();
    println!("Decrypted: {}", decrypted);

    // Verify
    assert_eq!(private_key, decrypted);
}
```

### 3. Export/Import JSON Keystore

```rust
use sol_safekey::KeyManager;

fn main() {
    let keypair = KeyManager::generate_keypair();
    let password = "secure_password";

    // Export encrypted JSON keystore
    let json = KeyManager::keypair_to_encrypted_json(&keypair, password).unwrap();
    std::fs::write("wallet.json", &json).unwrap();

    // Import from JSON
    let json = std::fs::read_to_string("wallet.json").unwrap();
    let restored_keypair = KeyManager::keypair_from_encrypted_json(&json, password).unwrap();

    assert_eq!(keypair.pubkey(), restored.pubkey());
}
```

### 4. Get Public Key Address

```rust
use sol_safekey::KeyManager;

fn main() {
    let private_key = "your_private_key_base58";
    let public_key = KeyManager::get_public_key(private_key).unwrap();
    println!("Wallet address: {}", public_key);
}
```

## Optional Features

If you need additional features, you can enable them:

```toml
[dependencies]
# Enable 2FA features (hardware fingerprint, TOTP, etc.)
sol-safekey = { version = "0.1.0", features = ["2fa"] }

# Enable Solana operations (transfer, balance query, etc.)
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }

# Enable all features (including CLI)
sol-safekey = { version = "0.1.0", features = ["full"] }
```

## API Documentation

### `KeyManager::generate_keypair() -> Keypair`

Generate a new Solana keypair.

### `KeyManager::encrypt_with_password(private_key: &str, password: &str) -> Result<String, String>`

Encrypt a private key with a password, returns base64 encoded encrypted data.

### `KeyManager::decrypt_with_password(encrypted_data: &str, password: &str) -> Result<String, String>`

Decrypt a private key with a password, returns the original base58 private key.

### `KeyManager::get_public_key(private_key: &str) -> Result<String, String>`

Get the public key address from a private key.

### `KeyManager::keypair_to_encrypted_json(keypair: &Keypair, password: &str) -> Result<String, String>`

Export keypair as encrypted JSON format.

### `KeyManager::keypair_from_encrypted_json(json_data: &str, password: &str) -> Result<Keypair, String>`

Restore keypair from encrypted JSON.

## CLI Tool vs Library Integration

### CLI Tool

Suitable for:
- Personal use by developers
- Command-line scripts
- Quick testing

Installation:
```bash
cargo install sol-safekey --features full
```

### Library Integration

Suitable for:
- Integration into your Rust projects
- Web services
- Automation programs

Integration:
```toml
[dependencies]
sol-safekey = "0.1.0"  # No need to install CLI
```

## Complete Example

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Scenario 1: Create new wallet and encrypt it
    println!("=== Creating New Wallet ===");
    let keypair = KeyManager::generate_keypair();
    let password = "MySecurePassword123!";

    let json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("my_wallet.json", &json)?;

    println!("✅ Wallet created");
    println!("📍 Address: {}", keypair.pubkey());

    // Scenario 2: Load wallet from file
    println!("\n=== Loading Wallet ===");
    let json = std::fs::read_to_string("my_wallet.json")?;
    let loaded_keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;

    println!("✅ Wallet loaded");
    println!("📍 Address: {}", loaded_keypair.pubkey());

    // Scenario 3: Encrypt/decrypt private key string only
    println!("\n=== Encrypting Private Key ===");
    let private_key = keypair.to_base58_string();
    let encrypted = KeyManager::encrypt_with_password(&private_key, password)?;
    println!("🔒 Encrypted: {}...", &encrypted[..50]);

    let decrypted = KeyManager::decrypt_with_password(&encrypted, password)?;
    println!("🔓 Decrypted: {}...", &decrypted[..50]);

    assert_eq!(private_key, decrypted);
    println!("✅ Verification successful");

    Ok(())
}
```

## Security Recommendations

1. **Password Management**: Use strong passwords (at least 8 characters, including uppercase, lowercase, numbers, special characters)
2. **Private Key Protection**: Never store private keys in plain text in code
3. **Environment Variables**: You can use environment variables to pass passwords, but don't commit them to Git
4. **Keystore Backup**: Regularly backup encrypted keystore files to secure locations

## More Information

- [GitHub Repository](https://github.com/your-repo/sol-safekey)
- [Complete CLI Documentation](../README.md)
- [API Documentation](https://docs.rs/sol-safekey)
- [Solana Operations Guide](./SOLANA_OPS.md)
