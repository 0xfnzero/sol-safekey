<div align="center">
    <h1>🔧 Sol SafeKey</h1>
    <h3><em>A powerful command-line tool for secure Solana key management with Triple-Factor 2FA</em></h3>
</div>

<p align="center">
    <strong>Securely generate, manage, and encrypt Solana private keys with military-grade triple-factor authentication combining hardware fingerprint, master password, security question, and 2FA verification.</strong>
</p>

<p align="center">
    <a href="https://crates.io/crates/sol-safekey">
        <img src="https://img.shields.io/crates/v/sol-safekey.svg" alt="Crates.io">
    </a>
    <a href="https://docs.rs/sol-safekey">
        <img src="https://docs.rs/sol-safekey/badge.svg" alt="Documentation">
    </a>
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/0xfnzero/sol-safekey">
        <img src="https://img.shields.io/github/stars/0xfnzero/sol-safekey?style=social" alt="GitHub stars">
    </a>
    <a href="https://github.com/0xfnzero/sol-safekey/network">
        <img src="https://img.shields.io/github/forks/0xfnzero/sol-safekey?style=social" alt="GitHub forks">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/Security-FF6B6B?style=for-the-badge&logo=shield&logoColor=white" alt="Security">
    <img src="https://img.shields.io/badge/2FA-4CAF50?style=for-the-badge&logo=google-authenticator&logoColor=white" alt="2FA">
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

---

## 📚 Documentation

| Document | Description | Language |
|----------|-------------|----------|
| [README.md](./README.md) | Complete project overview, CLI usage | English |
| [README_CN.md](./README_CN.md) | 完整项目概述、CLI 使用 | 中文 |
| [INTEGRATION.md](./docs/INTEGRATION.md) | Library integration guide | English |
| [INTEGRATION_CN.md](./docs/INTEGRATION_CN.md) | 库集成指南 | 中文 |
| [SOLANA_OPS.md](./docs/SOLANA_OPS.md) | Solana operations (transfer, balance) | English |
| [SOLANA_OPS_CN.md](./docs/SOLANA_OPS_CN.md) | Solana 操作（转账、余额） | 中文 |
| [LIBRARY_VS_CLI.md](./docs/LIBRARY_VS_CLI.md) | Library vs CLI comparison | English |
| [LIBRARY_VS_CLI_CN.md](./docs/LIBRARY_VS_CLI_CN.md) | 库 vs CLI 对比 | 中文 |

**Quick Navigation:**
- 🚀 New to Sol-SafeKey? → Start with [README.md](./README.md)
- 📦 Want to integrate into your project? → See [INTEGRATION.md](./docs/INTEGRATION.md)
- 💰 Need Solana operations? → Check [SOLANA_OPS.md](./docs/SOLANA_OPS.md)
- 🤔 Library or CLI? → Read [LIBRARY_VS_CLI.md](./docs/LIBRARY_VS_CLI.md)

---

## 📋 Table of Contents

- [What's New: Triple-Factor 2FA Security](#-whats-new-triple-factor-2fa-security)
- [Features](#-features)
- [Installation](#-installation)
  - [As a Library (Recommended for Integration)](#as-a-library-recommended-for-integration)
  - [As a CLI Tool](#as-a-cli-tool)
- [Quick Start](#-quick-start)
  - [🤖 Bot Integration (Recommended for Bots)](#-bot-integration-recommended-for-bots)
  - [📦 Library Integration (For Developers)](#-library-integration-for-developers)
  - [🔧 CLI Tool Usage](#-cli-tool-usage)
- [Library API Reference](#-library-api-reference)
- [CLI Command Reference](#-cli-command-reference)
  - [🔐 Triple-Factor 2FA Commands (Recommended)](#-triple-factor-2fa-commands-recommended)
  - [🔑 Basic Generation Commands](#-basic-generation-commands)
  - [🔐 Encryption/Decryption Commands](#-encryptiondecryption-commands)
  - [🔍 Query Commands](#-query-commands)
- [Options Reference](#-options-reference)
- [Output Formats](#-output-formats)
- [Security Architecture](#-security-architecture)
- [Security Best Practices](#️-security-best-practices)
- [Security Warnings](#️-security-warnings)
- [Migration & Recovery Scenarios](#-migration--recovery-scenarios)
- [Development](#️-development)
- [Contributing](#-contributing)
- [License](#-license)
- [Support](#-support)
- [Acknowledgments](#-acknowledgments)

---

## 🎯 What's New: Triple-Factor 2FA Security

**The Most Secure Wallet Protection Available!** Sol SafeKey now features a revolutionary **Triple-Factor Authentication System** that combines:

- 🖥️ **Factor 1**: Hardware Fingerprint (binds to your device)
- 🔐 **Factor 2**: Master Password (strong password with complexity requirements)
- 🛡️ **Factor 3**: Security Question Answer (additional knowledge factor)
- 📱 **2FA Verification**: Time-based One-Time Password (Google Authenticator/Authy)

This means your wallet requires **all four components** to unlock - making it virtually impossible for attackers to access your funds even if they steal your encrypted wallet file!

### 🚀 Quick Start with Triple-Factor 2FA

```bash
# Step 1: Setup 2FA (one-time setup)
sol-safekey setup-2fa

# Step 2: Generate your secure wallet
sol-safekey gen-2fa-wallet -o my-secure-wallet.json

# Step 3: Unlock your wallet when needed
sol-safekey unlock-2fa-wallet -f my-secure-wallet.json
```

**What happens during generation:**
1. Generates a new Solana keypair
2. Creates triple-factor encrypted wallet (device-bound)
3. Automatically creates a keystore backup (recover private key cross-device using master password)

## 📦 Installation

### As a Library (Recommended for Integration)

Add to your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = "0.1.0"
```

This installs the **minimal library** without CLI dependencies - perfect for integration into your Rust projects.

**Optional features:**
```toml
# Enable 2FA features (hardware fingerprint, TOTP, etc.)
sol-safekey = { version = "0.1.0", features = ["2fa"] }

# Enable Solana operations (balance, transfer, etc.)
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }

# Enable all features
sol-safekey = { version = "0.1.0", features = ["full"] }
```

**Quick Example:**
```rust
use sol_safekey::KeyManager;

fn main() {
    // Generate new keypair
    let keypair = KeyManager::generate_keypair();

    // Encrypt with password
    let encrypted = KeyManager::encrypt_with_password(
        &keypair.to_base58_string(),
        "your_password"
    ).unwrap();

    // Decrypt
    let decrypted = KeyManager::decrypt_with_password(
        &encrypted,
        "your_password"
    ).unwrap();
}
```

👉 See [INTEGRATION.md](./INTEGRATION.md) for complete library integration guide.

### As a CLI Tool

For command-line usage, install with full features:

```bash
cargo install sol-safekey --features full
```

Or build from source:

```bash
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey
cargo build --release --features full
```

The binary will be available at `target/release/sol-safekey`.

## ✨ Features

### 🔑 Key Generation
- **Multiple Formats**: Keypair format, string format, and encrypted format
- **Triple-Factor 2FA Wallet**: Most secure wallet protection available
- **Segmentation**: Split long private keys into segments for easier recording
- **Custom Output**: Specify custom output file paths
- **Automatic Backup**: Keystore backup generation for cross-device recovery

### 🔐 Triple-Factor Security Features
- **Hardware Fingerprint Binding**: Wallet is bound to your specific device
  - CPU info, system serial, MAC address, disk serial
  - SHA256 hashed for consistent identification
- **Strong Password Requirements**: 8+ characters with 3 of: uppercase, lowercase, digits, special chars
- **Security Questions**: 8 predefined questions for additional protection
- **TOTP 2FA**: RFC 6238 standard (compatible with Google Authenticator, Authy, etc.)
- **Deterministic Key Derivation**: 2FA secret derived from hardware fingerprint + master password using PBKDF2 (100,000 iterations)
- **Triple-Factor Encryption**: Private key encrypted with all three factors combined

### 🌐 Multi-language Support
- **Bilingual Interface**: Complete Chinese-English help information
- **Bilingual Commands**: All command descriptions support Chinese and English

## 🚀 Quick Start

Sol SafeKey can be used in three ways:
1. **As a CLI Tool** - Command-line interface for managing Solana keys
2. **As a Rust Library** - Integrate encryption functionality into your own projects
3. **For Bot Integration** - Easy bot integration with CLI-based key management (🔥 **Recommended for Bots**)

### 🤖 Bot Integration (Recommended for Bots)

Perfect for trading bots, automated tools, and applications that need secure key management.

#### Why Use This for Bots?
- ✅ **No CLI Implementation Needed** - Just call `BotKeyManager`
- ✅ **Interactive Password Input** - Secure password prompt at startup
- ✅ **Encrypted Storage** - Keystore files remain encrypted on disk
- ✅ **Simple API** - Only 3 lines of code to unlock wallet

#### Quick Bot Example

```rust
use sol_safekey::bot_helper::BotKeyManager;
use solana_sdk::signature::Keypair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = BotKeyManager::new();

    // Interactive unlock (prompts user for password)
    let private_key = manager.unlock_keystore_interactive("bot_wallet.json")?;
    let keypair = Keypair::from_base58_string(&private_key);

    println!("🚀 Bot started with wallet: {}", keypair.pubkey());
    // Your bot logic here...

    Ok(())
}
```

#### Complete Bot Example

See [`examples/simple_bot.rs`](./examples/simple_bot.rs) for a complete working example:

```bash
# Run the bot example
cargo run --example simple_bot
```

The example includes:
- First-time wallet generation
- Interactive password input
- Secure wallet unlocking
- Bot operations (balance check, trading simulation)

#### Using in Your Bot

Add to your `Cargo.toml`:
```toml
[dependencies]
sol-safekey = "0.1.0"
solana-sdk = "3.0"

# Optional: For Solana operations (balance, transfer, etc.)
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }
tokio = { version = "1.0", features = ["full"] }
```

Then use in your bot code:
```rust
use sol_safekey::bot_helper::BotKeyManager;

let manager = BotKeyManager::new();

// First run: Generate wallet
let pubkey = manager.generate_keystore_interactive("wallet.json")?;

// Every run: Unlock wallet
let private_key = manager.unlock_keystore_interactive("wallet.json")?;
```

#### Solana Operations (Optional)

If you need to perform Solana operations (check balance, transfer, etc.), enable the `solana-ops` feature:

```toml
[dependencies]
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }
```

**Example - Check Balance:**
```rust
use sol_safekey::{KeyManager, solana_utils::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load wallet
    let keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;

    // Create Solana client
    let client = SolanaClient::new("https://api.mainnet-beta.solana.com".to_string());

    // Check balance
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    println!("Balance: {} SOL", lamports_to_sol(balance));

    Ok(())
}
```

📖 **Complete Solana Operations Guide:** See [SOLANA_OPS.md](./SOLANA_OPS.md) for detailed documentation including:
- CLI usage for all operations
- Library integration examples
- Transfer SOL and tokens
- Wrap/Unwrap SOL
- API reference

### 📦 Library Integration (For Developers)

Integrate encryption functionality directly into your projects.

Add to your `Cargo.toml`:
```toml
[dependencies]
sol-safekey = "0.1.0"
```

Or without CLI features:
```toml
[dependencies]
sol-safekey = { version = "0.1.0", default-features = false }
```

#### Basic Usage Example

```rust
use sol_safekey::KeyManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate keypair
    let keypair = KeyManager::generate_keypair();
    println!("Public key: {}", keypair.pubkey());

    // Encrypt with password
    let private_key = keypair.to_base58_string();
    let encrypted = KeyManager::encrypt_with_password(&private_key, "password")?;

    // Decrypt
    let decrypted = KeyManager::decrypt_with_password(&encrypted, "password")?;

    // Create encrypted JSON keystore
    let keystore = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;

    // Restore from keystore
    let restored = KeyManager::keypair_from_encrypted_json(&keystore, "password")?;

    Ok(())
}
```

### 🔧 CLI Tool Installation

```bash
# Clone repository
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey

# Build
cargo build --release

# Or use the build script (macOS/Linux)
./build.sh

# Install to system (optional)
cargo install --path .
```

### Basic CLI Usage

```bash
# View help
sol-safekey --help

# Generate keypair format
sol-safekey gen-keypair -o my-wallet.json

# Generate encrypted keystore (interactive password input)
sol-safekey gen-keystore -o secure-wallet.json

# Unlock keystore (interactive password input)
sol-safekey unlock -f secure-wallet.json

# Or provide password as argument for non-interactive use
sol-safekey gen-keystore -o secure-wallet.json -p mypassword
sol-safekey unlock -f secure-wallet.json -p mypassword
```

### Advanced Usage (Triple-Factor 2FA Mode)

```bash
# 1. First-time setup: Configure your 2FA
sol-safekey setup-2fa

# This will:
# - Collect your device's hardware fingerprint
# - Guide you to set a strong master password (8+ chars, 3 types)
# - Let you choose and answer a security question
# - Generate a 2FA secret and show QR code
# - Verify setup with your authenticator app

# 2. Generate your secure wallet
sol-safekey gen-2fa-wallet -o my-wallet.json

# This creates TWO files:
# - my-wallet.json: Triple-factor encrypted (requires device + password + security question + 2FA)
# - XXXXXXXX_keystore.json: Password-only backup (recover private key cross-device using master password)

# 3. Unlock your wallet
sol-safekey unlock-2fa-wallet -f my-wallet.json

# You'll need to provide:
# - Master password
# - Security question answer
# - Current 2FA code from your authenticator app
```

## 📚 Library API Reference

When using sol-safekey as a library, the main interface is the `KeyManager` struct:

### Core Functions

#### `KeyManager::generate_keypair()`
Generate a new Solana keypair.

```rust
let keypair = KeyManager::generate_keypair();
```

#### `KeyManager::encrypt_with_password(private_key, password)`
Encrypt a private key with a password.

```rust
let encrypted = KeyManager::encrypt_with_password(&private_key, "password")?;
```

#### `KeyManager::decrypt_with_password(encrypted_data, password)`
Decrypt an encrypted private key.

```rust
let decrypted = KeyManager::decrypt_with_password(&encrypted, "password")?;
```

#### `KeyManager::get_public_key(private_key)`
Derive public key from a private key.

```rust
let public_key = KeyManager::get_public_key(&private_key)?;
```

#### `KeyManager::keypair_to_encrypted_json(keypair, password)`
Create an encrypted keystore JSON from a keypair.

```rust
let json = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;
```

#### `KeyManager::keypair_from_encrypted_json(json_data, password)`
Restore a keypair from encrypted JSON.

```rust
let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;
```

### Usage Patterns

#### Pattern 1: Simple Encryption
```rust
use sol_safekey::KeyManager;

let keypair = KeyManager::generate_keypair();
let encrypted = KeyManager::encrypt_with_password(
    &keypair.to_base58_string(),
    "password"
)?;
```

#### Pattern 2: Keystore Management
```rust
use sol_safekey::KeyManager;

// Save to keystore
let keypair = KeyManager::generate_keypair();
let keystore = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;
std::fs::write("wallet.json", keystore)?;

// Load from keystore
let keystore = std::fs::read_to_string("wallet.json")?;
let keypair = KeyManager::keypair_from_encrypted_json(&keystore, "password")?;
```

#### Pattern 3: Multiple Wallet Management
```rust
use sol_safekey::KeyManager;
use std::collections::HashMap;

let mut wallets: HashMap<String, String> = HashMap::new();
let password = "master_password";

// Create multiple wallets
for i in 0..3 {
    let keypair = KeyManager::generate_keypair();
    let encrypted = KeyManager::encrypt_with_password(
        &keypair.to_base58_string(),
        password
    )?;
    wallets.insert(format!("wallet_{}", i), encrypted);
}
```

## 📋 CLI Command Reference

### 🔐 Triple-Factor 2FA Commands (Recommended)

#### `setup-2fa`
One-time setup for triple-factor authentication
```bash
sol-safekey setup-2fa
```

**Process:**
1. Collects hardware fingerprint (automatic)
2. Sets master password (requires 8+ chars, 3 character types)
3. Chooses security question and answer
4. Generates 2FA secret (deterministic from fingerprint + password)
5. Displays QR code for Google Authenticator/Authy
6. Verifies setup with test code

**Password Requirements:**
- Minimum 8 characters
- Must include at least 3 of: uppercase, lowercase, digits, special characters
- Examples:
  - ✅ `MyPass123!` (has uppercase, lowercase, digits, special)
  - ✅ `secure2024#` (has lowercase, digits, special)
  - ❌ `password` (too weak)
  - ❌ `Pass123` (only 7 characters)

#### `gen-2fa-wallet`
Generate triple-factor encrypted wallet with automatic keystore backup
```bash
sol-safekey gen-2fa-wallet -o my-wallet.json
```

**What you get:**
- `my-wallet.json`: Triple-factor encrypted wallet
  - Encrypted with: hardware fingerprint + master password + security question
  - Can only be unlocked on this device with all three factors + 2FA code

- `XXXXXXXX_keystore.json`: Cross-device backup
  - XXXXXXXX = first 8 characters of your wallet address
  - Encrypted with master password only
  - Recover private key cross-device using master password for emergency recovery
  - Unlock with: `sol-safekey unlock -f XXXXXXXX_keystore.json -p <password>`

**Input Process:**
1. Enter master password (once)
2. Answer security question (once)
3. Enter current 2FA code from authenticator (once)

#### `unlock-2fa-wallet`
Unlock triple-factor encrypted wallet
```bash
sol-safekey unlock-2fa-wallet -f my-wallet.json
```

**Requirements:**
- Must be on the same device (hardware fingerprint verification)
- Master password
- Security question answer
- Current 2FA code from authenticator app

**Security Features:**
- Hardware fingerprint automatically verified
- All three factors required for decryption
- 2FA code must be current (30-second window)
- Displays private key and public key after successful unlock

### 🔑 Basic Generation Commands

#### `gen-keypair`
Generate keypair format private key
```bash
sol-safekey gen-keypair -o wallet.json
```

#### `gen-key`
Generate string format private key
```bash
sol-safekey gen-key -s 3 -o keys.json
```

#### `gen-keystore`
Generate encrypted keystore file
```bash
sol-safekey gen-keystore -p password123 -o secure.json
```

### 🔐 Encryption/Decryption Commands

#### `encrypt`
Encrypt existing private key
```bash
sol-safekey encrypt -k "your_private_key_string" -p password123
```

#### `decrypt`
Decrypt private key string
```bash
sol-safekey decrypt -e "encrypted_data" -p password123
```

#### `unlock`
Decrypt private key from file (including keystore backups)
```bash
sol-safekey unlock -f encrypted-file.json -p password123

# Unlock keystore backup
sol-safekey unlock -f XXXXXXXX_keystore.json -p your_master_password
```

### 🔍 Query Commands

#### `address`
View wallet address from private key
```bash
# From plain private key
sol-safekey address -k YOUR_PRIVATE_KEY

# From encrypted private key
sol-safekey address -e ENCRYPTED_KEY -p password123

# From file
sol-safekey address -f keys.json

# From encrypted file
sol-safekey address -f encrypted-keys.json -p password123
```

## 📝 Options Reference

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Output file path |
| `--segments` | `-s` | Number of segments |
| `--password` | `-p` | Password |
| `--private-key` | `-k` | Private key string |
| `--encrypted-key` | `-e` | Encrypted data |
| `--file-path` | `-f` | File path |

## 📁 Output Formats

### Triple-Factor Wallet Format
```json
{
  "encrypted_private_key": "base64_encrypted_data_with_all_factors",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "version": "triple_factor_v1",
  "question_index": 2,
  "created_at": "2025-09-30T10:15:30Z"
}
```

### Keystore Backup Format
```json
{
  "encrypted_private_key": "base64_encrypted_data_password_only",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "encryption_type": "password_only",
  "created_at": "2025-09-30T10:15:30Z",
  "note": "Recover private key cross-device using master password"
}
```

### Keypair Format
```json
[89, 252, 28, 23, ...]  // 64-byte array
```

### String Format
```json
{
  "private_key": "5D1iwg89hSXfoqA28ioE...",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "segments": ["5D1iwg89hS", "XfoqA28io", "E..."],
  "created_at": "2025-09-21T04:03:37+00:00"
}
```

## 🔒 Security Architecture

### Triple-Factor Encryption Process

1. **Hardware Fingerprint Collection**
   - CPU information
   - System serial number
   - MAC address
   - Disk serial number
   - Combined and hashed with SHA256

2. **Key Derivation (PBKDF2)**
   - Input: Hardware fingerprint + Master password + Security answer
   - Algorithm: PBKDF2-HMAC-SHA256
   - Iterations: 200,000
   - Output: 256-bit encryption key

3. **2FA Secret Generation**
   - Derived from: Hardware fingerprint + Master password
   - Algorithm: PBKDF2-HMAC-SHA256
   - Iterations: 100,000
   - Output: BASE32-encoded TOTP secret (160-bit)
   - Deterministic: Same inputs always produce same 2FA secret

4. **Encryption**
   - Private key + 2FA secret packaged together
   - Encrypted with triple-factor derived key
   - XOR encryption with SHA256-based keystream
   - BASE64 encoded for storage

5. **Decryption + Verification**
   - Hardware fingerprint verified automatically
   - User provides: master password + security answer + current 2FA code
   - Triple-factor key re-derived
   - Data decrypted
   - 2FA code verified (30-second time window)
   - Private key extracted

### Security Questions
Available questions (select one during setup):
1. Your mother's maiden name?
2. City where you were born?
3. Name of your elementary school?
4. Your favorite movie?
5. Name of your first pet?
6. Your father's birthday? (Format: YYYYMMDD)
7. Your spouse's name?
8. Your best friend's name?

**Note:** Answers are normalized (lowercase, trimmed) for consistency.

## 🛡️ Security Best Practices

### For Triple-Factor 2FA Wallets

1. **Device Binding**
   - Triple-factor wallet is bound to your device
   - Cannot be unlocked on different devices
   - Keep your device secure and backed up

2. **Password Management**
   - Use a strong, unique master password
   - Never reuse passwords from other services
   - Store password in a secure password manager

3. **Security Question**
   - Choose a question you can always remember
   - Answer consistently (system normalizes case)
   - Don't share answers with others

4. **2FA Setup**
   - Add to Google Authenticator or Authy immediately
   - Backup your authenticator app
   - Test verification before completing setup

5. **Keystore Backup**
   - Always keep the `XXXXXXXX_keystore.json` backup file
   - Store in multiple secure locations (USB drive, encrypted cloud storage)
   - This is your emergency recovery option
   - Unlock command: `sol-safekey unlock -f XXXXXXXX_keystore.json -p <password>`

6. **Recovery Planning**
   - **If device is lost/damaged**: Use keystore backup file with master password
   - **If you forget password**: No recovery possible (by design)
   - **If you lose 2FA access**: Use keystore backup to recover, then run setup-2fa again
   - **If you forget security question**: No recovery possible from triple-factor wallet (use keystore backup)

### General Security

1. **Offline Storage**: Store encrypted private keys on offline devices
2. **Regular Backups**: Keep multiple copies of keystore backups in secure locations
3. **Test Recovery**: Periodically test decryption to ensure backups work
4. **Physical Security**: Protect devices containing wallets and authenticator apps
5. **Never Share**: Never share passwords, 2FA codes, or security answers

## ⚠️ Security Warnings

1. **Password Loss = Fund Loss**: If you forget your master password, encrypted wallets cannot be recovered
2. **Device-Bound Security**: Triple-factor wallets can only be unlocked on the original device
3. **Keystore Backup Critical**: The `XXXXXXXX_keystore.json` file is your only cross-device recovery option
4. **2FA App Backup**: Losing access to your authenticator app requires using keystore backup for recovery
5. **No Password Reset**: There is no "forgot password" feature - this is intentional for security
6. **Security Question Important**: Treat security question answer as sensitive as your password

## 🔄 Migration & Recovery Scenarios

### Scenario 1: New Device (Planned Migration)
1. On old device, unlock wallet and note private key
2. On new device, run `setup-2fa` to configure new triple-factor setup
3. Import private key or use keystore backup

### Scenario 2: Device Lost/Damaged (Emergency Recovery)
1. On new device, install sol-safekey
2. Use your `XXXXXXXX_keystore.json` backup file
3. Run: `sol-safekey unlock -f XXXXXXXX_keystore.json -p <master_password>`
4. Extract private key
5. Run `setup-2fa` on new device for future security
6. Generate new triple-factor wallet with recovered key

### Scenario 3: Lost 2FA Access
1. Use keystore backup to recover private key
2. Run `setup-2fa` again to configure new 2FA
3. Generate new triple-factor wallet

### Scenario 4: Forgot Security Question Answer
1. Cannot unlock triple-factor wallet
2. Use keystore backup for recovery
3. Run `setup-2fa` again with new security question

## 🛠️ Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

### Release Build

```bash
cargo build --release

# Or use build script
./build.sh
```

### Project Structure

```
sol-safekey/
├── src/
│   ├── lib.rs                    # Core encryption/decryption logic
│   ├── main.rs                   # CLI interface
│   ├── totp.rs                   # TOTP implementation
│   ├── secure_totp.rs            # Secure TOTP manager
│   ├── hardware_fingerprint.rs   # Hardware fingerprint collection
│   └── security_question.rs      # Security question handling
├── Cargo.toml                    # Dependencies
├── build.sh                      # Build script
└── README.md                     # This file
```

## 🤝 Contributing

Issues and Pull Requests are welcome!

### Areas for Contribution
- Additional hardware fingerprint sources
- More security questions
- Multi-language translations
- Cross-platform testing
- Security audits

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- **Telegram**: [Join our group](https://t.me/fnzero_group)
- **Discord**: [Join our server](https://discord.gg/ckf5UHxz)

## 🙏 Acknowledgments

- Solana Foundation for the excellent SDK
- Ring crypto library for secure cryptographic operations
- TOTP-RS for RFC 6238 implementation
- QRCode library for 2FA setup visualization

---

⭐ If this project helps you secure your Solana assets, please give it a star!

**Made with ❤️ for the Solana community**