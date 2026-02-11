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

## 📑 目录

- [✨ 特性](#-特性)
- [🚀 快速开始](#-快速开始)
  - [Bot 开发者](#bot-开发者)
  - [作为库使用](#作为库使用)
- [📚 文档](#-文档)
- [🔐 安全性](#-安全性)
- [📦 安装](#-安装)
- [🛠️ 可用操作](#️-可用操作)
  - [钱包管理](#钱包管理)
  - [SOL 操作](#sol-操作)
  - [代币操作](#代币操作)
  - [高级功能](#高级功能)
- [📖 示例](#-示例)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)

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

## 📦 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
sol-safekey = { path = "path/to/sol-safekey" }

[features]
solana-ops = ["sol-safekey/solana-ops"]
```

## 🛠️ 可用操作

### 钱包管理
- **创建钱包** - 生成新的 AES-256 加密钱包
- **导入钱包** - 从私钥或助记词导入现有钱包
- **导出钱包** - 导出钱包为加密 JSON 格式
- **查看地址** - 显示钱包公钥地址

### SOL 操作
- **查询余额** - 查看钱包中的 SOL 余额
- **转账 SOL** - 向其他地址发送 SOL
- **Wrap SOL** - 将 SOL 转换为 WSOL（包装的 SOL）
- **Unwrap WSOL** - 将 WSOL 转换回 SOL

### 代币操作
- **转账 SPL 代币** - 向其他地址发送 SPL 代币
- **查询代币余额** - 查看代币余额
- **🔥 PumpSwap 卖出** - PumpSwap DEX 上的交互式代币卖出
  - **批量卖出**：一次卖出多个代币（用逗号或空格分割）
  - **智能默认值**：Seed 优化和确认默认为 yes
  - **一键卖出**：自动卖出所有代币余额
  - **Seed 优化 ATA**：更低的交易手续费（默认启用）
  - **99% 滑点**：即使在波动市场中也能快速成交
  - **Token-2022 支持**：自动检测和处理
  - **双语界面**：完整支持中文和英文

### 高级功能
- **Durable Nonce 账户** - 创建和管理 nonce 账户用于离线签名
- **2FA 支持** - 可选的双因素认证增强安全性
- **硬件指纹** - 基于设备的安全层
- **安全密码输入** - 仅通过 stdin 管道（永不使用环境变量）

## 📖 示例

查看 `examples/bot_example.rs` 获取完整的 bot 集成示例。

## 🤝 贡献

欢迎贡献！请确保遵循安全最佳实践。

## 📄 许可证

MIT License - 详见 LICENSE 文件
