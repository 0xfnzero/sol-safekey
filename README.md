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
    <img src="https://img.shields.io/badge/Security-FF0000?style=for-the-badge&logo=security&logoColor=white" alt="Security">
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/vuazbGkqQE">Discord</a>
</p>

---

## 📑 Table of Contents

- [✨ Features](#-features)
- [🚀 Quick Start](#-quick-start)
  - [For Bot Developers](#for-bot-developers)
  - [As a Library](#as-a-library)
- [📚 Documentation](#-documentation)
- [🔐 Security](#-security)
- [📦 Installation](#-installation)
- [🛠️ Available Operations](#️-available-operations)
  - [Wallet Management](#wallet-management)
  - [SOL Operations](#sol-operations)
  - [Token Operations](#token-operations)
  - [Advanced Features](#advanced-features)
- [📖 Examples](#-examples)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

---

## ✨ Features

- **🔐 AES-256 Encryption**: Military-grade encryption with PBKDF2 key derivation
- **🤖 Bot Integration**: Simple 3-line integration for Solana trading bots
- **💰 Solana Operations**: Built-in support for transfers, wrapping, token operations, PumpSwap DEX sell, and **Pump.fun internal market (bonding curve) sell**
- **🔒 Secure by Default**: Password via stdin pipe (memory only, never environment variables)
- **🎯 Interactive CLI**: Full-featured command-line interface with `safekey` command

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

## 📚 Documentation

- **[Bot Integration Guide](BOT_INTEGRATION.md)** - How to integrate sol-safekey into your bot
- **[User Guide](USER_GUIDE.md)** - Complete usage instructions and examples

## 🔐 Security

- ✅ **Password Security**: stdin pipe only (never in environment variables or files)
- ✅ **Encryption**: AES-256 with PBKDF2 key derivation
- ✅ **Memory Safety**: Immediate password cleanup after use

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = "0.1.2"
# or from local path: sol-safekey = { path = "path/to/sol-safekey" }

[features]
solana-ops = ["sol-safekey/solana-ops"]
```

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

### Token Operations
- **Transfer SPL Tokens** - Send SPL tokens to other addresses
- **Query Token Balance** - Check token balances
- **🔥 PumpSwap Sell** - Interactive token selling on PumpSwap DEX
  - **Batch Selling**: Sell multiple tokens at once (comma or space separated)
  - **Smart Defaults**: Seed optimization and confirmations default to yes
  - **One-click Sell**: Automatically sell all token balance
  - **Seed-optimized ATA**: Lower transaction fees (enabled by default)
  - **99% Slippage**: Fast execution even in volatile markets
  - **Token-2022 Support**: Automatic detection and handling
  - **Bilingual UI**: Full support for English and Chinese
- **🔥 Pump.fun Internal Market Sell** - One-click sell on Pump.fun bonding curve (internal market only; for migrated tokens use PumpSwap)
  - **Internal market (bonding curve)**: Sell tokens that are still on Pump.fun’s bonding curve for native SOL
  - **Seed-optimized ATA**: Lower fees (optional, same as PumpSwap)
  - **Token-2022 Support**: Automatic detection
  - **Bilingual UI**: English and Chinese

### Advanced Features
- **Durable Nonce Accounts** - Create and manage nonce accounts for offline signing
- **2FA Support** - Optional two-factor authentication for enhanced security
- **Hardware Fingerprint** - Device-based security layer
- **Secure Password Input** - stdin pipe only (never in environment variables)

## 📖 Examples

See `examples/bot_example.rs` for a complete bot integration example.

## 🤝 Contributing

Contributions welcome! Please ensure security best practices are followed. **Use English for commit and PR descriptions.**

## 📄 License

MIT License - See LICENSE file for details
