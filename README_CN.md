<div align="center">
    <h1>🔐 Sol-SafeKey</h1>
    <h3><em>安全的 Solana 密钥管理库，支持 AES-256 加密</em></h3>
</div>

<p align="center">
    <strong>军事级钱包安全，简单的 Bot 集成 - 安全的密码处理、加密的密钥存储和完整的 Solana 操作支持。</strong>
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

## ✨ 特性

- **🔐 AES-256 加密**: 军事级加密，使用 PBKDF2 密钥派生
- **🤖 Bot 集成**: 3 行代码即可集成到 Solana 交易机器人
- **💰 Solana 操作**: 内置转账、wrap、token 等链上操作
- **🔒 默认安全**: 密码通过 stdin 管道传递（仅内存，永不使用环境变量）
- **🎯 交互式 CLI**: 完整的命令行界面，通过 `safekey` 命令使用

## 🚀 快速开始

### Bot 开发者

```bash
# 编译 bot
cargo build --example bot_example --features solana-ops --release

# 运行交互式 safekey 命令
./build-cache/release/examples/bot_example safekey
```

### 作为库使用

```rust
use sol_safekey::KeyManager;

// 生成密钥对
let keypair = KeyManager::generate_keypair();

// 加密并保存
let json = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;
std::fs::write("keystore.json", json)?;

// 加载并解密
let json = std::fs::read_to_string("keystore.json")?;
let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;
```

## 📚 文档

- **[Bot 集成指南](BOT_INTEGRATION_CN.md)** - 如何将 sol-safekey 集成到你的 bot
- **[使用手册](USER_GUIDE_CN.md)** - 完整的使用说明和示例

## 🔐 安全性

- ✅ **密码安全**: 仅通过 stdin 管道（永不使用环境变量或文件）
- ✅ **加密方式**: AES-256 配合 PBKDF2 密钥派生
- ✅ **内存安全**: 使用后立即清除密码
- ✅ **生产就绪**: 与 wick-catching-bot 相同的安全模型

## 📦 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
sol-safekey = { path = "path/to/sol-safekey" }

[features]
solana-ops = ["sol-safekey/solana-ops"]
```

## 🛠️ 可用操作

通过 `safekey` 命令：
- 创建加密钱包
- 查询 SOL 余额
- 转账 SOL
- Wrap/Unwrap SOL ↔ WSOL
- 转账 SPL 代币
- 创建 durable nonce 账户

## 📖 示例

查看 `examples/bot_example.rs` 获取完整的 bot 集成示例。

## 🤝 贡献

欢迎贡献！请确保遵循安全最佳实践。

## 📄 许可证

MIT License - 详见 LICENSE 文件
