<div align="center">
    <h1>🔐 Sol-SafeKey</h1>
    <h3><em>Solana Private Key Management Tool - Simple, Secure, Professional</em></h3>
</div>

<p align="center">
    <strong>Interactive multi-language menu | Password encryption | Triple-factor 2FA | Solana operations | Bot integration</strong>
</p>

<p align="center">
    <a href="https://crates.io/crates/sol-safekey">
        <img src="https://img.shields.io/crates/v/sol-safekey.svg" alt="Crates.io">
    </a>
    <a href="https://docs.rs/sol-safekey">
        <img src="https://docs.rs/sol-safekey/badge.svg" alt="Documentation">
    </a>
    <a href="./LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/2FA-4CAF50?style=for-the-badge&logo=google-authenticator&logoColor=white" alt="2FA">
</p>

<p align="center">
    <a href="./README.md">English</a> |
    <a href="./README_CN.md">中文</a>
</p>

---

> ## ⚠️ SECURITY NOTICE
>
> **Open-source encryption tool with known limitations.** The encryption algorithm is publicly visible.
>
> - ✅ **Recommended**: Development, testing, bots, medium wallets with 2FA ($1k-$10k)
> - ❌ **Not recommended**: Large holdings (>$10k) - use hardware wallets instead
> - 🔐 **Must use**: 20+ character passwords + 2FA for important wallets
> - 📖 **Read**: Security section below before use

---

## 📋 Table of Contents

1. [Quick Start](#-quick-start)
2. [Module 1: Simple Encryption/Decryption](#-module-1-simple-encryptiondecryption)
3. [Module 2: Triple-Factor 2FA Encryption](#-module-2-triple-factor-2fa-encryption)
4. [Module 3: Solana Wallet Operations](#-module-3-solana-wallet-operations)
5. [Module 4: Bot Integration](#-module-4-bot-integration)
6. [Security & Best Practices](#-security--best-practices)

---

## 🚀 Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey

# Build with all features
cargo build --release --features full

# Run interactive menu
./target/release/sol-safekey start
```

### Interactive Menu

```
==================================================
  Sol-SafeKey - Solana Key Management Tool
==================================================

Core Functions:
  1.  Create Plain Private Key
  2.  Create Encrypted Private Key
  3.  Decrypt Private Key

Advanced Security:
  4.  Setup 2FA Authentication
  5.  Generate Triple-Factor Wallet
  6.  Unlock Triple-Factor Wallet

Solana Operations:
  7.  Check SOL Balance
  8.  Transfer SOL
  9.  Wrap SOL → WSOL
  10. Unwrap WSOL → SOL
  11. Transfer SPL Token
  12. Create Nonce Account

  0.  Exit
```

---

## 🔑 Module 1: Simple Encryption/Decryption

**For**: Quick setup, development, testing, personal wallets

### 1.1 Create Encrypted Keystore

**Interactive Mode:**

```bash
./sol-safekey start
# Select option 2: Create Encrypted Private Key
```

**CLI Mode:**

```bash
# Not available via CLI - use interactive mode
```

**Process:**

1. Choose method:
   - Generate new keypair
   - Import existing private key

2. Set password (10+ characters, 3 types):
   - Uppercase, lowercase, numbers, special chars

3. Save as keystore file (JSON format)

**Output File (`keystore.json`):**

```json
{
  "encrypted_private_key": "base64_encrypted_data",
  "public_key": "7nWq3...",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### 1.2 Decrypt Keystore

**Interactive Mode:**

```bash
./sol-safekey start
# Select option 3: Decrypt Private Key
```

**Process:**

1. Choose input method:
   - From keystore file
   - Enter encrypted string

2. Enter password

3. View decrypted private key

### 1.3 Use in Code

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load encrypted keystore
    let keypair = KeyManager::keypair_from_keystore_file(
        "keystore.json",
        "your-password"
    )?;

    println!("Wallet: {}", keypair.pubkey());
    Ok(())
}
```

### 1.4 Security Level

- **Encryption**: AES-256 + PBKDF2
- **Security**: ⭐⭐⭐ (Medium)
- **Portable**: ✅ Works on any device
- **Best for**: Development, testing, small wallets (<$1k)

---

## 🛡️ Module 2: Triple-Factor 2FA Encryption

**For**: Production bots, medium-value wallets, maximum security

### 2.1 What is Triple-Factor?

```
┌─────────────────────────────────────────┐
│  Factor 1: Hardware Fingerprint         │
│  - CPU ID, MAC address, hostname        │
│  - Device-bound (not portable)          │
├─────────────────────────────────────────┤
│  Factor 2: Master Password              │
│  - Strong password (10+ chars)          │
│  - Never stored, only hashed            │
├─────────────────────────────────────────┤
│  Factor 3: Security Question            │
│  - Pre-defined questions                │
│  - Extra protection layer               │
├─────────────────────────────────────────┤
│  Factor 4: TOTP 2FA Code                │
│  - 6-digit code (30s refresh)           │
│  - Google Authenticator compatible      │
└─────────────────────────────────────────┘
```

### 2.2 Setup 2FA (One-time)

**Interactive Mode:**

```bash
./sol-safekey start
# Select option 4: Setup 2FA Authentication
```

**CLI Mode:**

```bash
./sol-safekey setup-2fa
```

**Process:**

```
Step 1/4: Collect hardware fingerprint
✅ Fingerprint collected: a3f7b2...

Step 2/4: Set master password
Enter password: ************
Confirm: ************
✅ Password set

Step 3/4: Security question
Select question from list
Enter answer
✅ Question set

Step 4/4: Setup 2FA
📱 Scan QR code with Google Authenticator
Enter 6-digit code: 123456
✅ 2FA verified!

🎉 Setup complete!
```

### 2.3 Generate Triple-Factor Wallet

**CLI Mode:**

```bash
./sol-safekey gen-2fa-wallet -o secure-wallet.json
```

**Process:**

1. Collects hardware fingerprint (automatic)
2. Enter master password
3. Answer security question
4. Enter 2FA code from authenticator
5. Generates 2 files:
   - `secure-wallet.json` (triple-factor, device-bound)
   - `XXXXXX_keystore.json` (password-only backup)

**Output Files:**

```
📁 secure-wallet.json
   Security: ⭐⭐⭐⭐⭐ (Maximum)
   Requires: Hardware + Password + Security + 2FA
   Device: Current device only

📁 XXXXXX_keystore.json
   Security: ⭐⭐⭐ (Medium)
   Requires: Password only
   Device: Any device (emergency recovery)
```

### 2.4 Unlock Triple-Factor Wallet

**CLI Mode:**

```bash
./sol-safekey unlock-2fa-wallet -f secure-wallet.json
```

**Process:**

1. Verify hardware fingerprint (automatic)
2. Enter master password
3. Answer security question
4. Enter 2FA code (from authenticator)
5. ✅ Wallet unlocked!

### 2.5 Use in Code

```rust
use sol_safekey::{decrypt_with_triple_factor_and_2fa, hardware_fingerprint::HardwareFingerprint};

fn unlock_triple_wallet(
    encrypted_data: &str,
    master_password: &str,
    security_answer: &str,
    twofa_code: &str
) -> Result<String, String> {
    let hw_fp = HardwareFingerprint::collect()?;

    let (private_key, _, _) = decrypt_with_triple_factor_and_2fa(
        encrypted_data,
        &hw_fp.fingerprint,
        master_password,
        security_answer,
        twofa_code
    )?;

    Ok(private_key)
}
```

### 2.6 Security Level

- **Encryption**: AES-256 + PBKDF2 + Hardware binding + 2FA
- **Security**: ⭐⭐⭐⭐⭐ (Maximum)
- **Portable**: ❌ Device-bound (has backup file)
- **Best for**: Production bots, medium wallets ($1k-$10k)

---

## ⚡ Module 3: Solana Wallet Operations

**For**: Managing SOL, tokens, and blockchain interactions

All operations work with both simple keystores and 2FA wallets!

### 3.1 Check SOL Balance

**Interactive Mode:**

```bash
./sol-safekey start
# Select option 7: Check SOL Balance
```

**CLI Mode:**

```bash
./sol-safekey sol-ops -f keystore.json balance --rpc-url https://api.mainnet-beta.solana.com
```

**Process:**

1. Enter keystore path
2. Enter password
3. Select network (Mainnet/Devnet)
4. View balance

**Output:**

```
✅ Balance:
  💰 1.5 SOL
  📊 1500000000 lamports
```

### 3.2 Transfer SOL

**Interactive Mode:**

```bash
./sol-safekey start
# Select option 8: Transfer SOL
```

**CLI Mode:**

```bash
./sol-safekey sol-ops -f keystore.json transfer \
  --to 7nWq3... \
  --amount 0.1
```

**Process:**

1. Enter keystore & password
2. Select network
3. Enter recipient address
4. Enter amount (SOL)
5. Confirm transaction
6. ✅ Transaction sent!

**Output:**

```
✅ Transfer successful!
  📝 Signature: 5Kq7...
  🔗 Explorer: https://solscan.io/tx/5Kq7...
```

### 3.3 Wrap SOL → WSOL

**Use case**: Convert native SOL to wrapped SOL for DeFi

```bash
# Interactive
./sol-safekey start → Option 9

# CLI
./sol-safekey sol-ops -f keystore.json wrap-sol --amount 1.0
```

### 3.4 Unwrap WSOL → SOL

**Use case**: Convert wrapped SOL back to native SOL

```bash
# Interactive
./sol-safekey start → Option 10

# CLI
./sol-safekey sol-ops -f keystore.json unwrap-sol
```

### 3.5 Transfer SPL Tokens

```bash
# Interactive
./sol-safekey start → Option 11

# CLI
./sol-safekey sol-ops -f keystore.json transfer-token \
  --mint EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v \
  --to 7nWq3... \
  --amount 100
```

### 3.6 Create Nonce Account

**Use case**: Durable transactions that don't expire

```bash
# Interactive
./sol-safekey start → Option 12

# CLI - use interactive mode
```

### 3.7 Use in Code

```rust
use sol_safekey::operations::*;
use solana_sdk::signature::Keypair;

fn example_check_balance(keypair: &Keypair) -> Result<(), String> {
    // Check balance
    check_balance(keypair, Language::English)?;

    Ok(())
}

fn example_transfer(keypair: &Keypair) -> Result<(), String> {
    // Transfer SOL
    transfer_sol(keypair, Language::English)?;

    Ok(())
}
```

---

## 🤖 Module 4: Bot Integration

**For**: Integrating Sol-SafeKey into your Rust trading bots

### 4.1 Add Dependency

```toml
# Cargo.toml
[dependencies]
sol-safekey = { path = "./sol-safekey", features = ["full"] }
solana-sdk = "1.18"
```

### 4.2 Basic Integration

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load wallet
    let keypair = KeyManager::keypair_from_keystore_file(
        "keystore.json",
        "password"
    )?;

    println!("Bot wallet ready: {}", keypair.pubkey());

    // Your bot logic here...

    Ok(())
}
```

### 4.3 Bot Helper (Auto Password Prompt)

```rust
use sol_safekey::bot_helper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Automatically prompts for password if needed
    let keypair = bot_helper::ensure_wallet_ready("keystore.json")?;

    println!("✅ Wallet ready: {}", keypair.pubkey());

    // Start bot
    run_trading_bot(&keypair).await?;

    Ok(())
}

async fn run_trading_bot(keypair: &Keypair) -> Result<(), Box<dyn std::error::Error>> {
    // Your bot logic
    Ok(())
}
```

### 4.4 Use with Trading Operations

```rust
use sol_safekey::KeyManager;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

struct TradingBot {
    keypair: Keypair,
    rpc_client: RpcClient,
}

impl TradingBot {
    pub fn new(keystore_path: &str, password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let keypair = KeyManager::keypair_from_keystore_file(keystore_path, password)?;
        let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com");

        Ok(Self { keypair, rpc_client })
    }

    pub fn get_balance(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let balance = self.rpc_client.get_balance(&self.keypair.pubkey())?;
        Ok(balance)
    }

    pub fn execute_trade(&self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        // Sign and send transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&tx)?;
        println!("Trade executed: {}", signature);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = TradingBot::new("keystore.json", "your-password")?;

    let balance = bot.get_balance()?;
    println!("Bot balance: {} lamports", balance);

    // Run bot logic...

    Ok(())
}
```

### 4.5 Environment Variables

```rust
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load password from environment
    let password = env::var("WALLET_PASSWORD")
        .expect("WALLET_PASSWORD not set");

    let keypair = KeyManager::keypair_from_keystore_file(
        "keystore.json",
        &password
    )?;

    // Bot logic...

    Ok(())
}
```

**.env file:**

```bash
WALLET_PASSWORD=your-strong-password-here
```

**Run bot:**

```bash
# Load .env and run
source .env && cargo run --release
```

### 4.6 Bot Integration Checklist

- ✅ Use strong passwords (20+ chars)
- ✅ Store keystore outside of git repo
- ✅ Use environment variables for passwords
- ✅ Enable 2FA for production bots
- ✅ Test on devnet first
- ✅ Monitor wallet balance
- ✅ Implement error handling
- ✅ Log all transactions

---

## 🔒 Security & Best Practices

### Security Limitations

**Known Issues:**

1. **Open-Source Encryption**
   - Algorithm is public
   - Vulnerable to offline brute-force if keystore stolen
   - Mitigation: Use 20+ character passwords

2. **Static Salt (Current)**
   - Future versions will use random salts
   - Mitigation: Use very strong passwords

3. **No Rate Limiting**
   - Unlimited decryption attempts possible
   - Mitigation: Secure your keystore files

### Best Practices

✅ **Password Requirements:**

```
Minimum: 10 characters
Recommended: 20+ characters
Example: Xk9#mP2$vL8@qR5&wN3!tY7*uH4

Must contain 3 of:
- Uppercase (A-Z)
- Lowercase (a-z)
- Numbers (0-9)
- Special (!@#$%^&*)
```

✅ **File Security:**

```bash
# Set proper permissions
chmod 600 keystore.json

# Never commit to git
echo "*.json" >> .gitignore
echo "keystore*" >> .gitignore

# Backup securely
gpg -c keystore.json  # Encrypt backup
```

✅ **2FA Setup:**

- Use for wallets > $1,000
- Keep backup keystore file safe
- Store 2FA secret in password manager
- Test recovery process

✅ **When to Use What:**

| Wallet Value | Recommended Method |
|--------------|-------------------|
| < $100 | Simple encryption |
| $100 - $1k | Simple encryption + strong password |
| $1k - $10k | Triple-factor 2FA |
| > $10k | Hardware wallet (Ledger/Trezor) |

---

## 📦 Build Features

```bash
# Basic encryption only
cargo build --release

# With Solana operations
cargo build --release --features solana-ops

# With 2FA security
cargo build --release --features 2fa

# All features
cargo build --release --features full
```

**Feature Flags:**
- `solana-ops` - Wallet operations (balance, transfer, etc.)
- `2fa` - Triple-factor authentication + TOTP
- `cli` - Interactive menu (default)
- `full` - All features

---

## 📞 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- 💬 **Telegram**: [Join our group](https://t.me/fnzero_group)
- 🎮 **Discord**: [Join our server](https://discord.gg/ckf5UHxz)
- 🌐 **Website**: [fnzero.dev](https://fnzero.dev/)

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file

---

<div align="center">
    <p><strong>⭐ Star this repo if you find it helpful!</strong></p>
    <p>Made with ❤️ for the Solana community</p>
</div>
