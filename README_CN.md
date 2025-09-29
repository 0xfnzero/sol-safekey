# sol-safekey

🔧 **Solana安全密钥管理工具**

一个功能强大的命令行工具，用于安全地生成、管理和加密 Solana 私钥。支持多种格式输出、私钥分段显示和文件加密存储。

## ✨ 特性

### 🔑 密钥生成
- **多种格式**: 支持 keypair 格式、字符串格式和加密格式
- **分段显示**: 可将长私钥分段显示，便于分批记录
- **自定义输出**: 支持指定输出文件路径

### 🔐 加密功能
- **AES-256-GCM**: 使用军用级加密算法保护私钥
- **密码保护**: 支持最多10位密码保护
- **文件加密**: 直接生成加密文件或加密现有私钥

### 🌐 多语言支持
- **中英文界面**: 完整的中英文对照帮助信息
- **双语命令**: 所有命令描述支持中英文

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey

# 编译
cargo build --release

# 安装到系统
cargo install --path .
```

### 基本使用

```bash
# 查看帮助
sol-safekey --help

# 生成 keypair 格式私钥
sol-safekey gen-keypair -o my-wallet.json

# 生成字符串格式私钥，分3段显示
sol-safekey gen-key -s 3 -o my-keys.json

# 生成加密私钥
sol-safekey gen-secure -p mypassword -o secure-keys.json

# 加密现有私钥
sol-safekey encrypt -k YOUR_PRIVATE_KEY -p mypassword

# 解密私钥字符串
sol-safekey decrypt -e "ENCRYPTED_DATA" -p mypassword

# 解锁加密文件
sol-safekey unlock -f secure-keys.json -p mypassword
```

## 📋 命令详解

### 🔑 生成命令

#### `gen`
生成 Solana 私钥（兼容模式）
```bash
sol-safekey gen -o output.json -s 2
```

#### `gen-keypair`
生成 keypair 格式私钥
```bash
sol-safekey gen-keypair -o wallet.json
```

#### `gen-key`
生成字符串格式私钥
```bash
sol-safekey gen-key -s 3 -o keys.json
```

#### `gen-secure`
生成加密私钥
```bash
sol-safekey gen-secure -p password123 -s 2 -o secure.json
```

### 🔐 加密解密命令

#### `encrypt`
加密已有私钥
```bash
sol-safekey encrypt -k "your_private_key_string" -p password123
```

#### `decrypt`
解密私钥字符串
```bash
sol-safekey decrypt -e "encrypted_data" -p password123
```

#### `unlock`
从文件解锁私钥
```bash
sol-safekey unlock -f encrypted-file.json -p password123
```

## 📝 选项说明

| 选项 | 短选项 | 说明 |
|------|-------|------|
| `--output` | `-o` | 输出文件路径 |
| `--segments` | `-s` | 分段数量 |
| `--password` | `-p` | 密码（最多10位） |
| `--private-key` | `-k` | 私钥字符串 |
| `--encrypted-key` | `-e` | 加密数据 |
| `--file-path` | `-f` | 文件路径 |

## 📁 输出格式

### Keypair 格式
```json
[89, 252, 28, 23, ...]  // 64字节数组
```

### 字符串格式
```json
{
  "private_key": "5D1iwg89hSXfoqA28ioE...",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "segments": ["5D1iwg89hS", "XfoqA28io", "E..."],
  "created_at": "2025-09-21T04:03:37.279982+00:00"
}
```

### 加密格式
```json
{
  "encrypted_private_key": "{\"iv\":\"W2Vd3f...\",\"ciphertext\":\"tz+CCE...\"}",
  "public_key": "7o8KDvtxRnJNiv5Bm4NE...",
  "segments": ["segment1", "segment2"],
  "created_at": "2025-09-21T04:03:51.468977+00:00"
}
```

## 🔒 安全特性

- **AES-256-GCM 加密**: 使用业界标准的加密算法
- **随机 IV**: 每次加密使用不同的初始化向量
- **密码强度**: 支持最多10位密码保护
- **本地处理**: 所有加密操作在本地完成，不上传任何数据

## ⚠️ 安全提醒

1. **备份私钥**: 请务必安全备份您的私钥文件
2. **密码管理**: 使用强密码并妥善保管
3. **离线存储**: 建议将加密私钥存储在离线设备中
4. **定期检查**: 定期验证私钥文件的完整性

## 🛠️ 开发

### 构建
```bash
cargo build
```

### 测试
```bash
cargo test
```

### 发布构建
```bash
cargo build --release
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如果您遇到任何问题或有建议，请创建一个 Issue。

---

⭐ 如果这个项目对您有帮助，请给它一个星标！