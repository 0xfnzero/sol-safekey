# sol-safekey

🔧 **Solana安全密钥管理工具 | Solana Security Key Management Tool**

一个功能强大的命令行工具，用于安全地生成、管理和加密 Solana 私钥。支持多种格式输出、私钥分段显示和文件加密存储。

A powerful command-line tool for securely generating, managing, and encrypting Solana private keys. Supports multiple output formats, key segmentation, and encrypted file storage.

## ✨ 特性 | Features

### 🔑 密钥生成 | Key Generation
- **多种格式**: 支持 keypair 格式、字符串格式和加密格式
- **分段显示**: 可将长私钥分段显示，便于分批记录
- **自定义输出**: 支持指定输出文件路径

- **Multiple Formats**: Supports keypair format, string format, and encrypted format
- **Segmentation**: Split long private keys into segments for easier recording
- **Custom Output**: Specify custom output file paths

### 🔐 加密功能 | Encryption Features
- **AES-256-GCM**: 使用军用级加密算法保护私钥
- **密码保护**: 支持最多10位密码保护
- **文件加密**: 直接生成加密文件或加密现有私钥

- **AES-256-GCM**: Military-grade encryption algorithm to protect private keys
- **Password Protection**: Supports up to 10-character password protection
- **File Encryption**: Generate encrypted files directly or encrypt existing private keys

### 🌐 多语言支持 | Multi-language Support
- **中英文界面**: 完整的中英文对照帮助信息
- **双语命令**: 所有命令描述支持中英文

- **Bilingual Interface**: Complete Chinese-English help information
- **Bilingual Commands**: All command descriptions support Chinese and English

## 🚀 快速开始 | Quick Start

### 安装 | Installation

```bash
# 克隆仓库 | Clone repository
git clone https://github.com/your-username/sol-safekey.git
cd sol-safekey

# 编译 | Build
cargo build --release

# 安装到系统 | Install to system
cargo install --path .
```

### 基本使用 | Basic Usage

```bash
# 查看帮助 | View help
sol-safekey --help

# 生成 keypair 格式私钥 | Generate keypair format
sol-safekey gen-keypair -o my-wallet.json

# 生成字符串格式私钥，分3段显示 | Generate string format, split into 3 segments
sol-safekey gen-key -s 3 -o my-keys.json

# 生成加密私钥 | Generate encrypted private key
sol-safekey gen-secure -p mypassword -o secure-keys.json

# 加密现有私钥 | Encrypt existing private key
sol-safekey encrypt -k YOUR_PRIVATE_KEY -p mypassword

# 解锁加密文件 | Decrypt encrypted file
sol-safekey unlock -f secure-keys.json -p mypassword
```

## 📋 命令详解 | Command Reference

### 🔑 生成命令 | Generation Commands

#### `gen`
生成 Solana 私钥（兼容模式）| Generate Solana private key (compatibility mode)
```bash
sol-safekey gen -o output.json -s 2
```

#### `gen-keypair`
生成 keypair 格式私钥 | Generate keypair format private key
```bash
sol-safekey gen-keypair -o wallet.json
```

#### `gen-key`
生成字符串格式私钥 | Generate string format private key
```bash
sol-safekey gen-key -s 3 -o keys.json
```

#### `gen-secure`
生成加密私钥 | Generate encrypted private key
```bash
sol-safekey gen-secure -p password123 -s 2 -o secure.json
```

### 🔐 加密解密命令 | Encryption/Decryption Commands

#### `encrypt`
加密已有私钥 | Encrypt existing private key
```bash
sol-safekey encrypt -k "your_private_key_string" -p password123
```

#### `decrypt`
解密私钥字符串 | Decrypt private key string
```bash
sol-safekey decrypt -e "encrypted_data" -p password123
```

#### `unlock`
从文件解锁私钥 | Decrypt private key from file
```bash
sol-safekey unlock -f encrypted-file.json -p password123
```

## 📝 选项说明 | Options Reference

| 选项 Option | 短选项 Short | 说明 Description |
|------------|-------------|------------------|
| `--output` | `-o` | 输出文件路径 \| Output file path |
| `--segments` | `-s` | 分段数量 \| Number of segments |
| `--password` | `-p` | 密码（最多10位）\| Password (max 10 chars) |
| `--private-key` | `-k` | 私钥字符串 \| Private key string |
| `--encrypted-key` | `-e` | 加密数据 \| Encrypted data |
| `--file-path` | `-f` | 文件路径 \| File path |

## 📁 输出格式 | Output Formats

### Keypair 格式 | Keypair Format
```json
[89, 252, 28, 23, ...]  // 64字节数组 | 64-byte array
```

### 字符串格式 | String Format
```json
{
  "private_key": "5D1iwg89hSXfoqA28ioE...",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "segments": ["5D1iwg89hS", "XfoqA28io", "E..."],
  "created_at": "2025-09-21T04:03:37.279982+00:00"
}
```

### 加密格式 | Encrypted Format
```json
{
  "encrypted_private_key": "{\"iv\":\"W2Vd3f...\",\"ciphertext\":\"tz+CCE...\"}",
  "public_key": "7o8KDvtxRnJNiv5Bm4NE...",
  "segments": ["segment1", "segment2"],
  "created_at": "2025-09-21T04:03:51.468977+00:00"
}
```

## 🔒 安全特性 | Security Features

- **AES-256-GCM 加密**: 使用业界标准的加密算法
- **随机 IV**: 每次加密使用不同的初始化向量
- **密码强度**: 支持最多10位密码保护
- **本地处理**: 所有加密操作在本地完成，不上传任何数据

- **AES-256-GCM Encryption**: Uses industry-standard encryption algorithm
- **Random IV**: Uses different initialization vector for each encryption
- **Password Strength**: Supports up to 10-character password protection
- **Local Processing**: All encryption operations are performed locally, no data upload

## ⚠️ 安全提醒 | Security Reminders

1. **备份私钥**: 请务必安全备份您的私钥文件
2. **密码管理**: 使用强密码并妥善保管
3. **离线存储**: 建议将加密私钥存储在离线设备中
4. **定期检查**: 定期验证私钥文件的完整性

1. **Backup Private Keys**: Always securely backup your private key files
2. **Password Management**: Use strong passwords and keep them safe
3. **Offline Storage**: Recommend storing encrypted private keys on offline devices
4. **Regular Checks**: Regularly verify the integrity of private key files

## 🛠️ 开发 | Development

### 构建 | Build
```bash
cargo build
```

### 测试 | Test
```bash
cargo test
```

### 发布构建 | Release Build
```bash
cargo build --release
```

## 📄 许可证 | License

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 贡献 | Contributing

欢迎提交 Issue 和 Pull Request！

Issues and Pull Requests are welcome!

## 📞 支持 | Support

如果您遇到任何问题或有建议，请创建一个 Issue。

If you encounter any problems or have suggestions, please create an Issue.

---

⭐ 如果这个项目对您有帮助，请给它一个星标！

⭐ If this project helps you, please give it a star!