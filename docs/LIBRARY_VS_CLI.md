# Sol-SafeKey: Library vs CLI Tool

> 📖 **中文文档**: [LIBRARY_VS_CLI_CN.md](./LIBRARY_VS_CLI_CN.md)

## Quick Comparison

| Feature | Library Integration | CLI Tool |
|---------|-------------------|----------|
| **Use Case** | Integrate into Rust projects | Command-line usage |
| **Installation** | `Cargo.toml` dependency | `cargo install` |
| **Dependency Size** | Minimal (core only) | Full (includes CLI deps) |
| **Compile Speed** | Faster | Slower |
| **Target Users** | Developers (integration) | Personal use |
| **2FA Features** | Optional (`features = ["2fa"]`) | Full support |
| **Solana Ops** | Optional (`features = ["solana-ops"]`) | Full support |

## Library Integration (Recommended for Projects)

### Installation

```toml
[dependencies]
sol-safekey = "0.1.0"  # Minimal installation
```

### Usage

```rust
use sol_safekey::KeyManager;

fn main() {
    let keypair = KeyManager::generate_keypair();
    let encrypted = KeyManager::encrypt_with_password(
        &keypair.to_base58_string(),
        "password"
    ).unwrap();
}
```

### Advantages

✅ **No CLI Installation** - Just add library dependency
✅ **Small Size** - Doesn't include CLI tool dependencies (`clap`, `colored`, `qrcode`, etc.)
✅ **Fast Compilation** - Fewer dependencies mean faster builds
✅ **Flexible** - Enable features on demand (`2fa`, `solana-ops`)
✅ **Easy Integration** - Simple API, 3 lines of code for encryption/decryption

### Use Cases

- 🤖 Trading bots
- 🌐 Web services
- 📱 Wallet applications
- 🔧 Automation scripts
- 📦 Other Rust projects

## CLI Tool (Recommended for Personal Use)

### Installation

```bash
cargo install sol-safekey --features full
```

### Usage

```bash
# Generate encrypted wallet
sol-safekey gen-keystore -o wallet.json -p mypassword

# Unlock wallet
sol-safekey unlock -f wallet.json -p mypassword
```

### Advantages

✅ **Ready to Use** - No code writing required
✅ **Interactive** - Friendly command-line interface
✅ **Full Features** - Supports all features (2FA, Solana ops, etc.)
✅ **Visual** - Colored output, QR code display

### Use Cases

- 👨‍💻 Personal developer use
- 🔑 Quick test key generation
- 💼 Personal wallet management
- 🧪 Feature testing

## Choosing Recommendation

### Choose Library Integration if you:

- ✅ Need to integrate into your Rust project
- ✅ Want to control encryption/decryption flow
- ✅ Need custom user interface
- ✅ Want to minimize dependencies
- ✅ Building web services or bots

### Choose CLI Tool if you:

- ✅ Need to quickly generate/manage keys
- ✅ Don't want to write code
- ✅ Need interactive operations
- ✅ Using 2FA features
- ✅ Personal use or testing

## Complete Documentation

- 📖 [Integration Guide](./INTEGRATION.md) - Complete API integration docs
- 📖 [Solana Operations Guide](./SOLANA_OPS.md) - Query, transfer, token operations
- 📖 [CLI Usage Guide](../README.md) - Detailed feature description
- 📖 [API Documentation](https://docs.rs/sol-safekey)
