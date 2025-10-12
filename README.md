# Sol-SafeKey

A secure Solana key management library with AES-256 encryption and bot integration support.

[中文文档](README_CN.md)

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
cargo build --example complete_bot_example --features solana-ops --release

# Run interactive safekey commands
./build-cache/release/examples/complete_bot_example safekey
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

See `examples/complete_bot_example.rs` for a complete bot integration example.

## 🤝 Contributing

Contributions welcome! Please ensure security best practices are followed.

## 📄 License

MIT License - See LICENSE file for details
