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
- [📖 Complete Interactive Menu Guide](#-complete-interactive-menu-guide)
- [For Bot Developers](#for-bot-developers)
- [As a Library](#as-a-library)
- [📚 Documentation](#-documentation)
- [🔐 Security](#-security)
- [📦 Installation](#-installation)
- [🛠️ Available Operations](#️-available-operations)
- [💬 Contact](#-contact)

---

## ✨ Features

- **🔐 AES-256 Encryption**: Military-grade encryption with PBKDF2 key derivation
- **🤖 Bot Integration**: Simple 3-line integration for Solana trading bots
- **💰 Solana Operations**: Built-in support for transfers, wrapping, token operations, PumpSwap DEX sell, and **Pump.fun internal market (bonding curve) sell**
- **🔒 Secure by Default**: Password via stdin pipe (memory only, never environment variables)
- **🎯 Interactive CLI**: Full-featured command-line interface with `safekey` command
- **📖 Complete Menu Guide**: Detailed step-by-step tutorial for interactive menu usage

---

## 🚀 Quick Start

### For Bot Developers

```bash
# Build the bot
cargo build --example bot_example --features solana-ops --release

# Run interactive safekey commands
./build-cache/release/examples/bot_example safekey
```

### As a Library

```rust
use sol_safekey::KeyManager;

// Generate keypair
let keypair = KeyManager::generate_keypair();

// Encrypt and save
let json = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;
std::fs::write("keystore.json", json)?;

// Load and decrypt
let json = std::fs::read_to_string("keystore.json")?;
let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;
```

---

## 📖 Complete Interactive Menu Guide

This guide provides a comprehensive walkthrough of the interactive menu system in **sol-safekey**. Each function is explained with step-by-step instructions and example outputs.

### 📋 Menu Structure

The interactive menu consists of three main sections:

```
==================================================
  Sol-SafeKey - Solana Key Management Tool
==================================================

Core Functions (3 operations):

  1.  Create Plaintext Key
  2.  Create Encrypted Key (bot)
  3.  Decrypt Key

  🔒 Wallet Status: Unlocked
  U.  Unlock Wallet (for Solana Operations)

  Advanced Security Features:
  4.  Setup 2FA Authentication
  5.  Generate Triple-Factor Wallet
  6.  Unlock Triple-Factor Wallet

  Solana On-Chain Operations:
  7.  Check SOL Balance
  8.  Transfer SOL
  9.  Create WSOL ATA
  10.  Wrap SOL → WSOL
  11.  Unwrap WSOL → SOL
  12.  Close WSOL ATA
  13.  Transfer SPL Token
  14.  Create Nonce Account
  15.  Pump.fun Sell Token
  16.  PumpSwap Sell Token
  17.  Pump.fun Cashback (View & Claim)
  18.  PumpSwap Cashback (View & Claim)

  0.  Exit

Select operation (0-18/U): _
```

### 📚 Operation Tutorials Index

Quick access to detailed guides for each interactive menu operation:

#### 🔑 Core Functions (Options 1-3)
- **[1. Create Plaintext Key](#1-create-plaintext-key-option-1)** - Generate and save unencrypted Solana keypair
- **[2. Create Encrypted Key (bot)](#2-create-encrypted-key-option-2)** - Encrypt existing private key and save to keystore
- **[3. Decrypt Key](#3-decrypt-key-option-3)** - Decrypt keystore and reveal private key

#### 🔒 Wallet Management (Option U)
- **[U. Unlock Wallet](#u-unlock-wallet)** - Unlock wallet for Solana operations

#### 🛡️ Advanced Security Features (Options 4-6)
- **[4. Setup 2FA Authentication](#4-setup-2fa-authentication)** - Configure two-factor authentication
- **[5. Generate Triple-Factor Wallet](#5-generate-triple-factor-wallet)** - Create wallet with 3FA (hardware + password + security question + 2FA)
- **[6. Unlock Triple-Factor Wallet](#6-unlock-triple-factor-wallet)** - Decrypt 3FA encrypted wallet

#### 💰 Solana On-Chain Operations (Options 7-18)
- **[7. Check SOL Balance](#7-check-sol-balance)** - Query wallet SOL balance
- **[8. Transfer SOL](#8-transfer-sol)** - Send SOL to another address
- **[9. Create WSOL ATA](#9-create-wsol-ata)** - Create Wrapped SOL Associated Token Account
- **[10. Wrap SOL → WSOL](#10-wrap-sol--wsol)** - Convert SOL to Wrapped SOL
- **[11. Unwrap WSOL → SOL](#11-unwrap-wsol--sol)** - Convert Wrapped SOL back to SOL
- **[12. Close WSOL ATA](#12-close-wsol-ata)** - Close WSOL ATA and convert remaining WSOL to SOL
- **[13. Transfer SPL Token](#13-transfer-spl-token)** - Send SPL tokens to another address
- **[14. Create Nonce Account](#14-create-nonce-account)** - Create durable nonce for transaction replay protection
- **[15. Pump.fun Sell Token](#15-pumpfun-sell-token)** - Sell tokens on Pump.fun DEX (internal market)
- **[16. PumpSwap Sell Token](#16-pumpswap-sell-token)** - Sell tokens on PumpSwap DEX
- **[17. Pump.fun Cashback (View & Claim)](#17-pumpfun-cashback-view--claim)** - View and claim pump.fun cashback (native SOL)
- **[18. PumpSwap Cashback (View & Claim)](INTERACTIVE_TUTORIAL.md#18-pumpswap-cashback)** - View and claim PumpSwap cashback (WSOL)

📖 **[Complete Interactive Menu Guide →](INTERACTIVE_TUTORIAL.md)**

This comprehensive guide provides step-by-step instructions for all 18 interactive menu operations, including detailed steps, use cases, and example outputs for each operation.

**Quick Links to Common Tasks:**
- 🔑 [Wallet Setup](INTERACTIVE_TUTORIAL.md#u-unlock-wallet) - Unlock wallet and get started
- 💰 [Balance Query](INTERACTIVE_TUTORIAL.md#7-check-sol-balance) - Check SOL balance
- 🔑 [Nonce Account](INTERACTIVE_TUTORIAL.md#14-create-nonce-account) - Create durable nonce for trading bots
- 🏪 [Token Operations](INTERACTIVE_TUTORIAL.md#15-pumpfun-sell-token) - Pump.fun and PumpSwap sell operations

---

## For Bot Developers

### As a Library

```rust
use sol_safekey::KeyManager;

// Generate keypair
let keypair = KeyManager::generate_keypair();

// Encrypt and save
let json = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;
std::fs::write("keystore.json", json)?;

// Load and decrypt
let json = std::fs::read_to_string("keystore.json")?;
let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;
```

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

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = "0.1.3"
```

# Or from local path:
```toml
[dependencies]
sol-safekey = { path = "path/to/sol-safekey" }
```

# or from crates.io:
```bash
cargo install sol-safekey --features="full"
```

---

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
