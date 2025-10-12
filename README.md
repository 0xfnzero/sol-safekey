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
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

---

## ✨ Features

- **🔐 AES-256 Encryption**: Military-grade encryption with PBKDF2 key derivation
- **🤖 Bot Integration**: Simple 3-line integration for Solana trading bots
- **💰 Solana Operations**: Built-in support for transfers, wrapping, token operations
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
- ✅ **Production Ready**: Same security model as wick-catching-bot

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = { path = "path/to/sol-safekey" }

[features]
solana-ops = ["sol-safekey/solana-ops"]
```

## 🛠️ Available Operations

Via `safekey` command:
- Create encrypted wallet
- Query SOL balance
- Transfer SOL
- Wrap/Unwrap SOL ↔ WSOL
- Transfer SPL tokens
- Create durable nonce accounts

## 📖 Examples

See `examples/bot_example.rs` for a complete bot integration example.

## 🤝 Contributing

Contributions welcome! Please ensure security best practices are followed.

## 📄 License

MIT License - See LICENSE file for details
