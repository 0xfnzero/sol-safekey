# sol-safekey

🔧 **Solana Security Key Management Tool**

A powerful command-line tool for securely generating, managing, and encrypting Solana private keys. Supports multiple output formats, key segmentation, and encrypted file storage.

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

# Generate encrypted private key
sol-safekey gen-secure -p mypassword -o secure-keys.json

# Encrypt existing private key
sol-safekey encrypt -k YOUR_PRIVATE_KEY -p mypassword

# Decrypt private key string
sol-safekey decrypt -e "ENCRYPTED_DATA" -p mypassword

# Decrypt encrypted file
sol-safekey unlock -f secure-keys.json -p mypassword
```

## 📋 Command Reference

### 🔑 Generation Commands

#### `gen`
Generate Solana private key (compatibility mode)
```bash
sol-safekey gen -o output.json -s 2
```

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

#### `gen-secure`
Generate encrypted private key
```bash
sol-safekey gen-secure -p password123 -s 2 -o secure.json
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