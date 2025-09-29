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
3. Automatically creates a keystore backup (works on any device with password only)

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

### Installation

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

### Basic Usage (Simple Mode)

```bash
# View help
sol-safekey --help

# Generate keypair format
sol-safekey gen-keypair -o my-wallet.json

# Generate string format, split into 3 segments
sol-safekey gen-key -s 3 -o my-keys.json

# Generate encrypted keystore file
sol-safekey gen-keystore -p mypassword -o secure-keys.json

# Unlock encrypted file
sol-safekey unlock -f secure-keys.json -p mypassword
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
# - XXXXXXXX_keystore.json: Password-only backup (works on any device)

# 3. Unlock your wallet
sol-safekey unlock-2fa-wallet -f my-wallet.json

# You'll need to provide:
# - Master password
# - Security question answer
# - Current 2FA code from your authenticator app
```

## 📋 Command Reference

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
  - Can be used on any device for emergency recovery
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
  "note": "This file can be unlocked on any device with master password"
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