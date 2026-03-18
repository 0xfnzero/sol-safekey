<div align="center">
    <h1>🔐 Sol-SafeKey</h1>
    <h3><em>Secure Solana key management library with AES-256 encryption</em></h3>
</div>

<p align="center">
    <strong>Military-grade wallet security with simple bot integration - secure password handling, encrypted keystores, and full Solana operations support.</strong>
</p>

<p align="center">
    <a href="https://crates.io/crates/sol-safekey">
        <img src="https://img.shields.io/crates/v/sol-safekey.svg" alt="Crates.io">
    </a>
    <a href="https://docs.rs/sol-safekey">
        <img src="https://img.shields.io/docs.rs/sol-safekey/badge.svg" alt="Documentation">
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
    <img src="https://img.shields.io/badge/Security-FF0000?style=for-the-badge&logo=security&logoColor=white" alt="Security">
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

---

## 📑 Table of Contents

- [✨ Features](#-features)
- [🚀 Quick Start](#-quick-start)
- [📦 Installation](#-installation)
- [📋 Feature Guide](#-feature-guide)
- [📚 Documentation](#-documentation)
- [🔐 Security](#-security)
- [💬 Contact](#-contact)

### 📋 Feature Guide

#### 🚀 Quick Start
**[Installation →](#-installation)** → **[Unlock Wallet](INTERACTIVE_TUTORIAL.md#u-unlock-wallet)** → **[Check Balance](INTERACTIVE_TUTORIAL.md#7-check-sol-balance)**

#### 🔑 Core Functions
- **[1. Create Plaintext Key](INTERACTIVE_TUTORIAL.md#1-create-plaintext-key)** - Generate unencrypted keypair (testing only)
- **[2. Create Encrypted Key](INTERACTIVE_TUTORIAL.md#2-create-encrypted-key)** - Encrypt and save to keystore
- **[3. Decrypt Key](INTERACTIVE_TUTORIAL.md#3-decrypt-key)** - Decrypt keystore and reveal private key

#### 🔒 Wallet Management
- **[U. Unlock Wallet](INTERACTIVE_TUTORIAL.md#u-unlock-wallet)** - Unlock wallet for Solana operations

#### 🛡️ Advanced Security
- **[4. Setup 2FA](INTERACTIVE_TUTORIAL.md#4-setup-2fa-authentication)** - Configure two-factor authentication
- **[5. Generate Triple-Factor Wallet](INTERACTIVE_TUTORIAL.md#5-generate-triple-factor-wallet)** - Create 3FA wallet
- **[6. Unlock Triple-Factor Wallet](INTERACTIVE_TUTORIAL.md#6-unlock-triple-factor-wallet)** - Decrypt 3FA encrypted wallet

#### 💰 Solana Operations
##### Balance & Transfer
- **[7. Check Balance](INTERACTIVE_TUTORIAL.md#7-check-sol-balance)** - Query SOL balance
- **[8. Transfer SOL](INTERACTIVE_TUTORIAL.md#8-transfer-sol)** - Send SOL

##### WSOL Operations
- **[9. Create WSOL ATA](INTERACTIVE_TUTORIAL.md#9-create-wsol-ata)** - Create WSOL ATA
- **[10. Wrap SOL](INTERACTIVE_TUTORIAL.md#10-wrap-sol--wsol)** - SOL → WSOL
- **[11. Unwrap WSOL](INTERACTIVE_TUTORIAL.md#11-unwrap-wsol--sol)** - WSOL → SOL
- **[12. Close WSOL ATA](INTERACTIVE_TUTORIAL.md#12-close-wsol-ata)** - Close WSOL ATA

##### Token Operations
- **[13. Transfer SPL Token](INTERACTIVE_TUTORIAL.md#13-transfer-spl-token)** - Send SPL tokens
- **[14. Create Nonce Account](INTERACTIVE_TUTORIAL.md#14-create-nonce-account)** - Create durable nonce

##### DEX Operations
- **[15. Pump.fun Sell](INTERACTIVE_TUTORIAL.md#15-pumpfun-sell-token)** - Sell on Pump.fun DEX
- **[16. PumpSwap Sell](INTERACTIVE_TUTORIAL.md#16-pumpswap-sell-token)** - Sell on PumpSwap DEX

##### Cashback Operations
- **[17. Pump.fun Cashback](INTERACTIVE_TUTORIAL.md#17-pumpfun-cashback-view--claim)** - View/claim pump.fun cashback (SOL)
- **[18. PumpSwap Cashback](INTERACTIVE_TUTORIAL.md#18-pumpswap-cashback-view--claim)** - View/claim PumpSwap cashback (WSOL)

---


---

## 📖 Complete Interactive Menu Guide

📖 **[View Complete Interactive Menu Guide → INTERACTIVE_TUTORIAL.md]**

Comprehensive step-by-step guide for all interactive menu operations, including detailed instructions, use cases, and example outputs for each of the 18 menu options.

---

## 📚 Documentation

- **[Bot Integration Guide](BOT_INTEGRATION.md)** - How to integrate sol-safekey into your bot
- **[User Guide](USER_GUIDE.md)** - Complete usage instructions and examples

---

## 🔐 Security

- ✅ **Password Security**: stdin pipe only (never in environment variables)
- ✅ **Encryption**: AES-256 with PBKDF2 key derivation
- ✅ **Memory Safety**: Immediate password cleanup after use
- ✅ **Hardware Fingerprint**: Device-based security layer
- ✅ **2FA Support**: Optional two-factor authentication for enhanced security


## 🛠️ Available Operations

### Wallet Management
- **Create Wallet** - Generate new encrypted wallet with AES-256
- **Import Wallet** - Import existing wallet from private key or seed phrase
- **Export Wallet** - Export wallet to encrypted JSON format
- **View Address** - Display wallet public address

### SOL Operations
- **Query Balance** - Check SOL balance in your wallet
- **Transfer SOL** - Send SOL to other addresses
- **Wrap SOL** - Convert SOL to WSOL (Wrapped SOL)
- **Unwrap WSOL** - Convert WSOL back to SOL
- **Close WSOL ATA** - Close WSOL ATA account

### Token Operations
- **Transfer SPL Token** - Send SPL tokens to other addresses
- **Query Token Balance** - Check token balances

### DEX Operations
- **🔥 Pump.fun Sell** - Interactive token selling on Pump.fun DEX (internal market only)
- **🔄 PumpSwap Sell** - Sell tokens on PumpSwap DEX

### Cashback Operations
- **Pump.fun Cashback** - View and claim pump.fun cashback (native SOL)
- **PumpSwap Cashback** - View and claim PumpSwap cashback (WSOL)

### Advanced Features
- **Durable Nonce Accounts** - Create and manage nonce accounts for offline signing
- **2FA Support** - Optional two-factor authentication for enhanced security

### Transaction Management
- **Check Transaction Status** - Query transaction status on Solana

---

## 📖 Examples

See `examples/bot_example.rs` for a complete bot integration example.

---

## 🤝 Contributing

Contributions welcome! Please ensure security best practices are followed. **Use English for commit and PR descriptions.**

---

## 📄 License

MIT License - See LICENSE file for details
