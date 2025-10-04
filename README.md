<div align="center">
    <h1>🔧 Sol-SafeKey</h1>
    <h3><em>Secure Solana Key Management Tool with Interactive Multi-language Interface</em></h3>
</div>

<p align="center">
    <strong>Securely generate, manage, and encrypt Solana private keys with an easy-to-use interactive menu. No commands to remember!</strong>
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
| [README.md](./README.md) | Complete guide and usage | English |
| [README_CN.md](./README_CN.md) | 完整使用指南 | 中文 |

---

## ✨ Features

✅ **Interactive Menu** - Choose your language, select operations with simple numbers
🔐 **Strong Encryption** - Password-based encryption with SHA-256 key derivation
🌍 **Multi-language** - Full English and Chinese support
📦 **Keystore Format** - Standard Solana wallet-compatible format
🛡️ **Security First** - Hidden password input, never exposes sensitive data unnecessarily
⚡ **3 Simple Operations** - Create plain key, create encrypted key, decrypt key

---

## 🚀 Quick Start

### Installation

```bash
# Clone repository
git clone <repository-url>
cd sol-safekey

# Build
cargo build --release

# Run (Interactive Mode - Recommended)
./target/release/sol-safekey
```

### Interactive Mode

Simply run without any arguments:

```bash
./sol-safekey
```

**Step 1: Choose Language**
```
==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]:
```

**Step 2: Select Operation**
```
==================================================
  Sol-SafeKey - Solana Key Management Tool
==================================================

Core Functions (3 operations):

  1.  Create Plain Private Key
  2.  Create Encrypted Private Key
  3.  Decrypt Private Key
  0.  Exit

Select option [0-3]:
```

**Step 3: Follow the prompts!**

---

## 📖 Usage Examples

### Example 1: Create Encrypted Keystore (Recommended)

```bash
./sol-safekey

# Select:
# Language: 1 (English)
# Operation: 2 (Create Encrypted Private Key)
# Method: 1 (Generate new keypair)
# Password: [enter password, minimum 10 characters]
# Confirm password: [enter again]
# Output: 1 (Save as Keystore file)
# File path: wallet.json (or press Enter for default)

# ✅ Result: wallet.json created with encrypted private key
```

### Example 2: Decrypt Keystore

```bash
./sol-safekey

# Select:
# Language: 1 (English)
# Operation: 3 (Decrypt Private Key)
# Input: 1 (From Keystore file)
# File path: wallet.json
# Password: [enter your password]

# ✅ Result: Private key displayed on screen
# Optional: Save to file or just view
```

### Example 3: Import Existing Key and Encrypt

```bash
./sol-safekey

# Select:
# Language: 1 (English)
# Operation: 2 (Create Encrypted Private Key)
# Method: 2 (Import existing private key)
# Private key: [paste your base58 private key]
# Password: [enter password]
# Output: 1 (Save as Keystore file)

# ✅ Result: Your existing key is now encrypted
```

---

## 💻 Command Line Mode (Advanced)

For scripts and automation:

```bash
# Generate encrypted keystore with password
sol-safekey gen-keystore -o wallet.json -p "your_strong_password"

# Decrypt keystore
sol-safekey unlock -f wallet.json -p "your_password"

# Encrypt existing private key
sol-safekey encrypt -k "YOUR_PRIVATE_KEY" -p "your_password"

# Decrypt encrypted string
sol-safekey decrypt -e "ENCRYPTED_DATA" -p "your_password"

# View wallet address
sol-safekey address -f wallet.json -p "your_password"

# Show all commands
sol-safekey --help
```

---

## 🔧 Integration with Your Application

### Bot Integration (Just 1-2 Lines of Code!)

**No CLI dependency required** - integrates directly with the library!

The easiest way to add wallet management to your Rust bot:

```rust
// In your Cargo.toml
// [dependencies]
// sol-safekey = "0.1"

use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    let wallet_path = "config/wallet.json";

    // That's it! One line to get a ready-to-use keypair:
    let keypair = bot_helper::ensure_wallet_ready(wallet_path).unwrap();

    println!("✅ Wallet ready!");
    println!("📍 Address: {}", keypair.pubkey());

    // Use keypair for your bot operations...
}
```

**What `ensure_wallet_ready()` does:**

1. **If wallet file doesn't exist:**
   - Launches interactive menu (with language selection)
   - Guides user to create encrypted wallet
   - Saves to specified path
   - Prompts for password to unlock
   - Returns ready-to-use keypair

2. **If wallet file exists:**
   - Prompts user to enter password
   - Decrypts the wallet
   - Returns ready-to-use keypair

**Even simpler - just 1 line:**

```rust
let keypair = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

**Complete bot example:**

```rust
use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    // Get wallet from config
    let wallet_path = std::env::var("WALLET_PATH")
        .unwrap_or_else(|_| "wallet.json".to_string());

    // Ensure wallet is ready (creates/unlocks as needed)
    let keypair = match bot_helper::ensure_wallet_ready(&wallet_path) {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("❌ Wallet setup failed: {}", e);
            std::process::exit(1);
        }
    };

    println!("✅ Bot wallet ready: {}", keypair.pubkey());

    // Your bot logic here...
    // - Sign transactions with keypair
    // - Monitor wallet balance
    // - Execute trades, etc.
}
```

**Features:**
- ✅ No CLI dependency - uses library directly
- ✅ Interactive language selection (English/Chinese)
- ✅ Auto-creates wallet if missing
- ✅ Auto-unlocks wallet if exists
- ✅ Returns ready-to-use `Keypair`
- ✅ All operations guided step-by-step

### Using Library API (Advanced)

For more control, use the library API directly:

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create encrypted keystore
    let keypair = KeyManager::generate_keypair();
    let password = "your_strong_password";

    let keystore_json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("wallet.json", keystore_json)?;

    println!("Wallet address: {}", keypair.pubkey());

    // Later, decrypt it
    let keystore_json = std::fs::read_to_string("wallet.json")?;
    let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, password)?;

    // Use keypair for signing transactions
    Ok(())
}
```

### Load from Environment Variables

```bash
# In your .env or environment
WALLET_KEYSTORE_PATH=./wallet.json
WALLET_PASSWORD=your_secure_password
```

```rust
// In your code
let keystore_path = std::env::var("WALLET_KEYSTORE_PATH")?;
let password = std::env::var("WALLET_PASSWORD")?;

let keystore_json = std::fs::read_to_string(keystore_path)?;
let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, &password)?;
```

---

## 📁 File Formats

### Encrypted Keystore File (wallet.json)

```json
{
  "encrypted_private_key": "base64_encrypted_data...",
  "public_key": "YourWalletPublicKeyAddress...",
  "encryption_type": "password",
  "created_at": "2025-01-15T10:30:00Z",
  "version": "1.0"
}
```

### Plain Keypair File (keypair.json)

```json
[1,2,3,4,5,...,64]
```

Standard Solana keypair format (64-byte array).

---

## 🛡️ Security Best Practices

1. ✅ **Strong Passwords**: Use 10+ characters with uppercase, lowercase, numbers, and symbols
2. ✅ **Backup Keystores**: Store in multiple secure locations (USB drives, encrypted cloud)
3. ✅ **Never Share**: Don't share passwords or private keys with anyone
4. ✅ **Delete Plaintext**: Remove decrypted key files immediately after use
5. ✅ **Test First**: Verify you can decrypt before funding the wallet

---

## ❓ FAQ

**Q: I forgot my password, can I recover my wallet?**
A: No. The password is required to decrypt the keystore. This is by design for security. Always keep password backups in a secure password manager.

**Q: Can I use the same keystore on multiple computers?**
A: **It depends on the keystore type:**
- **Standard password-encrypted keystore** (created with interactive mode option 2): ✅ Yes! Fully portable. Copy `wallet.json` to any machine and use the same password.
- **2FA triple-factor wallet** (created with `setup-2fa` and `gen-2fa-wallet`): ❌ No! Device-bound due to hardware fingerprint. Cannot be used on other machines.
- **2FA backup keystore** (the `*_keystore.json` file generated alongside 2FA wallet): ✅ Yes! This is specifically for cross-device recovery.

**Q: What encryption algorithm is used?**
A: XOR encryption with SHA-256 key derivation from your password.

**Q: Is it safe to commit wallet.json to version control?**
A: The encrypted keystore is relatively safe, but we recommend adding it to `.gitignore` and using environment-specific keystores.

**Q: How do I change my password?**
A: Decrypt the keystore to get the private key, then create a new keystore with the new password using operation 2.

**Q: Does this work offline?**
A: Yes! All key operations work completely offline. No internet connection required.

**Q: What's the difference between regular keystore and 2FA wallet?**
A:
- **Regular keystore** (Interactive mode → Option 2):
  - ✅ Portable (works on any computer)
  - 🔐 Password-only encryption
  - 📦 Single file (`wallet.json`)
  - 👥 Recommended for most users

- **2FA triple-factor wallet** (Advanced):
  - ❌ Device-bound (hardware fingerprint)
  - 🔐 Password + Security question + 2FA codes
  - 📦 Two files (device-bound + portable backup)
  - 🛡️ Maximum security for large holdings

---

## 🔥 Advanced Features

### 2FA Triple-Factor Authentication

For maximum security, enable triple-factor authentication:

```bash
# Step 1: Setup 2FA (one-time)
sol-safekey setup-2fa

# Step 2: Generate wallet with 2FA
sol-safekey gen-2fa-wallet -o wallet.json
```

This combines:
- 🖥️ **Hardware fingerprint** (device-bound, not portable)
- 🔑 **Master password** (user-defined strong password)
- ❓ **Security question** (additional verification layer)
- 📱 **2FA verification codes** (Google Authenticator/Authy)

**Important**: When you create a 2FA wallet, you get **TWO files**:
1. `wallet.json` - Triple-factor encrypted (⚠️ **device-bound, cannot be used on other computers**)
2. `<address_prefix>_keystore.json` - Password-only backup (✅ **portable, works on any computer**)

The backup keystore is your safety net if:
- Your device is damaged/lost
- You need to access wallet from another computer
- You reinstall your operating system

### Solana Operations

Execute Solana operations with encrypted keystores:

```bash
# Check SOL balance
sol-safekey sol-ops -f wallet.json balance

# Transfer SOL
sol-safekey sol-ops -f wallet.json transfer -t <recipient_address> -a 0.1

# Check token balance
sol-safekey sol-ops -f wallet.json token-balance -m <token_mint_address>

# Wrap SOL to WSOL
sol-safekey sol-ops -f wallet.json wrap-sol -a 1.0

# Unwrap WSOL to SOL
sol-safekey sol-ops -f wallet.json unwrap-sol
```

The tool will prompt for your password to decrypt the keystore before executing operations.

---

## 🌟 Why Choose Sol-SafeKey?

| Feature | Sol-SafeKey | Other Tools |
|---------|-------------|-------------|
| Interactive Menu | ✅ Yes | ❌ Command-line only |
| Multi-language | ✅ English + Chinese | ❌ English only |
| Encrypted Storage | ✅ Yes | ⚠️ Often plaintext |
| No Dependencies | ✅ Single binary | ❌ Requires Node.js/Python |
| Offline Support | ✅ Complete | ⚠️ Limited |
| 2FA Support | ✅ Optional | ❌ No |
| Open Source | ✅ MIT License | ✅ Varies |

---

## 📄 License

MIT License - Free for personal and commercial use.

See [LICENSE](./LICENSE) file for details.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## 💬 Community & Support

- 📧 **Issues**: [GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- 💬 **Telegram**: [Join our group](https://t.me/fnzero_group)
- 🎮 **Discord**: [Join our server](https://discord.gg/ckf5UHxz)
- 🌐 **Website**: [fnzero.dev](https://fnzero.dev/)

---

<div align="center">
    <p>Made with ❤️ for the Solana community</p>
    <p>
        <a href="https://github.com/0xfnzero/sol-safekey">⭐ Star us on GitHub</a>
    </p>
</div>
