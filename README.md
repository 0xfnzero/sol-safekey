<div align="center">
    <h1>🔧 Sol SafeKey</h1>
    <h3><em>A powerful command-line tool for secure Solana key management</em></h3>
</div>

<p align="center">
    <strong>Securely generate, manage, and encrypt Solana private keys with multiple output formats, key segmentation, and encrypted file storage.</strong>
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
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

## ✨ Features

### 🔑 Key Generation
- **Multiple Formats**: Supports keypair format, string format, and encrypted format
- **Segmentation**: Split long private keys into segments for easier recording
- **Custom Output**: Specify custom output file paths

### 🔐 Encryption Features
- **AES-256-GCM**: Military-grade encryption algorithm to protect private keys
- **Password Protection**: Supports up to 10-character password protection
- **File Encryption**: Generate encrypted files directly or encrypt existing private keys

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

# Install to system
cargo install --path .
```

### Basic Usage

```bash
# View help
sol-safekey --help

# Generate keypair format
sol-safekey gen-keypair -o my-wallet.json

# Generate string format, split into 3 segments
sol-safekey gen-key -s 3 -o my-keys.json

# Generate encrypted private key with password
sol-safekey gen-key -s 3 -p mypassword -o my-encrypted-keys.json

# Generate encrypted keystore file
sol-safekey gen-keystore -p mypassword -o secure-keys.json

# Encrypt existing private key
sol-safekey encrypt -k YOUR_PRIVATE_KEY -p mypassword

# Decrypt private key string
sol-safekey decrypt -e "ENCRYPTED_DATA" -p mypassword

# Decrypt encrypted file
sol-safekey unlock -f secure-keys.json -p mypassword
```

## 📋 Command Reference

### 🔑 Generation Commands

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
Decrypt private key from file
```bash
sol-safekey unlock -f encrypted-file.json -p password123
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

### ⚙️ Configuration Commands

#### `init`
Initialize tool and generate random encryption key
```bash
# Initialize (creates .env file)
sol-safekey init

# Force regenerate master key
sol-safekey init --force
```

## 📝 Options Reference

| Option | Short | Description |
|--------|-------|-------------|
| `--output` | `-o` | Output file path |
| `--segments` | `-s` | Number of segments |
| `--password` | `-p` | Password (max 10 chars) |
| `--private-key` | `-k` | Private key string |
| `--encrypted-key` | `-e` | Encrypted data |
| `--file-path` | `-f` | File path |

## 📁 Output Formats

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
  "created_at": "2025-09-21T04:03:37.279982+00:00"
}
```

### Encrypted Format
```json
{
  "encrypted_private_key": "{\"iv\":\"W2Vd3f...\",\"ciphertext\":\"tz+CCE...\"}",
  "public_key": "7o8KDvtxRnJNiv5Bm4NE...",
  "segments": ["segment1", "segment2"],
  "created_at": "2025-09-21T04:03:51.468977+00:00"
}
```

## 🔒 Security Features

- **AES-256-GCM Encryption**: Uses industry-standard encryption algorithm
- **Random IV**: Uses different initialization vector for each encryption
- **Password Strength**: Supports up to 10-character password protection
- **Local Processing**: All encryption operations are performed locally, no data upload

## ⚠️ Security Reminders

1. **Backup Private Keys**: Always securely backup your private key files
2. **Password Management**: Use strong passwords and keep them safe
3. **Offline Storage**: Recommend storing encrypted private keys on offline devices
4. **Regular Checks**: Regularly verify the integrity of private key files

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
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📞 Support

If you encounter any problems or have suggestions, please create an Issue.

---

⭐ If this project helps you, please give it a star!