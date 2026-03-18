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
        <img src="https://img.shields.io/docs.rs/sol-safekey/badge.svg" alt="Documentation">
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
    <a href="https://fnzero.dev/">官网</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

---

## 📑 目录

- [✨ 特性](#-特性)
- [🚀 快速开始](#-快速开始)
- [📋 完整菜单索引](#-完整菜单索引)
- [📖 交互式菜单完整使用指南](#-交互式菜单完整使用指南)
- [📚 文档](#-文档)
- [🔐 安全性](#-安全性)
- [📦 安装](#-安装)
- [🛠️ 可用操作](#️-可用操作)
- [📖 示例](#-示例)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)

### 📋 完整菜单索引

所有 18 个交互式菜单操作的快速概览，按类别组织：

#### 🔑 核心功能（选项 1-3）
- **[1. 创建明文私钥](INTERACTIVE_TUTORIAL_CN.md#1-创建明文私钥（选项-1）)** - 生成并保存未加密的 Solana 密钥对（仅用于测试）
- **[2. 创建加密私钥(bot)](INTERACTIVE_TUTORIAL_CN.md#2-创建加密私钥bot)（选项-2）)** - 加密现有私钥并保存到 keystore
- **[3. 解密私钥](INTERACTIVE_TUTORIAL_CN.md#3-解密私钥（选项-3）)** - 解密 keystore 并显示私钥

#### 🔒 钱包管理（选项 U）
- **[U. 解锁钱包](INTERACTIVE_TUTORIAL_CN.md#u-解锁钱包)** - 解锁钱包以进行 Solana 操作

#### 🛡️ 高级安全功能（选项 4-6）
- **[4. 设置 2FA 认证](INTERACTIVE_TUTORIAL_CN.md#4-设置-2fa-认证)** - 配置双因素认证
- **[5. 生成三因子钱包](INTERACTIVE_TUTORIAL_CN.md#5-生成三因子钱包)** - 创建 3FA 钱包（硬件 + 密码 + 安全问题 + 2FA）
- **[6. 解锁三因子钱包](INTERACTIVE_TUTORIAL_CN.md#6-解锁三因子钱包)** - 解密 3FA 加密钱包

#### 💰 Solana 链上操作（选项 7-18）
##### 余额与转账
- **[7. 查询 SOL 余额](INTERACTIVE_TUTORIAL_CN.md#7-查询-sol-余额)** - 查询钱包 SOL 余额
- **[8. 转账 SOL](INTERACTIVE_TUTORIAL_CN.md#8-转账-sol)** - 向其他地址发送 SOL

##### WSOL 操作
- **[9. 创建 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#9-创建-wsol-ata)** - 创建包装 SOL 关联代币账户
- **[10. 包装 SOL → WSOL](INTERACTIVE_TUTORIAL_CN.md#10-包装-sol--wsol)** - 将 SOL 转换为包装 SOL
- **[11. 解包 WSOL → SOL](INTERACTIVE_TUTORIAL_CN.md#11-解包-wsol--sol)** - 将包装 SOL 转换回 SOL
- **[12. 关闭 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#12-关闭-wsol-ata)** - 关闭 WSOL ATA 并将剩余 WSOL 转换为 SOL

##### 代币操作
- **[13. 转账 SPL 代币](INTERACTIVE_TUTORIAL_CN.md#13-转账-spl-代币)** - 向其他地址发送 SPL 代币

##### 交易工具
- **[14. 创建 Nonce 账户](INTERACTIVE_TUTORIAL_CN.md#14-创建-nonce-账户)** - 创建持久化 nonce 以防止交易重放

##### DEX 操作
- **[15. Pump.fun 卖出代币](INTERACTIVE_TUTORIAL_CN.md#15-pumpfun-卖出代币)** - 在 Pump.fun DEX 上卖出代币（内盘）
- **[16. PumpSwap 卖出代币](INTERACTIVE_TUTORIAL_CN.md#16-pumpswap-卖出代币)** - 在 PumpSwap DEX 上卖出代币

##### 返现操作
- **[17. Pump.fun 返现](INTERACTIVE_TUTORIAL_CN.md#17-pumpfun-返现-查看与领取)** - 查看并领取 pump.fun 返现（原生 SOL）
- **[18. PumpSwap 返现](INTERACTIVE_TUTORIAL_CN.md#18-pumpswap-返现-查看与领取)** - 查看并领取 PumpSwap 返现（WSOL）

#### 🎯 快速访问
- 🔑 **[钱包设置](INTERACTIVE_TUTORIAL_CN.md#u-解锁钱包)** - 解锁钱包并开始使用
- 💰 **[查询余额](INTERACTIVE_TUTORIAL_CN.md#7-查询-sol-余额)** - 快速查询 SOL 余额
- 🔑 **[Nonce 账户](INTERACTIVE_TUTORIAL_CN.md#14-创建-nonce-账户)** - 创建持久化 nonce 用于交易机器人
- 🏪 **[DEX 操作](INTERACTIVE_TUTORIAL_CN.md#15-pumpfun-卖出代币)** - 访问 Pump.fun 和 PumpSwap 卖出功能

---

## ✨ 特性

- **🔐 AES-256 加密**：军事级加密，使用 PBKDF2 密钥派生
- **🤖 Bot 集成**：3 行代码即可集成到 Solana 交易机器人
- **💰 Solana 操作**：内置转账、wrap、token、PumpSwap DEX 卖出及 **Pump.fun 内盘（bonding curve）卖出**
- **🔒 默认安全**：密码通过 stdin 管道传递（仅内存，永不使用环境变量）
- **🎯 交互式 CLI**：完整的命令行界面，通过 `safekey` 命令使用
- **📖 完整菜单指南**：详细的交互式菜单逐步使用教程

---

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

---

## 📦 安装

添加到你的 `Cargo.toml`：

```toml
[dependencies]
sol-safekey = "0.1.3"
```

# 或从本地路径：
```toml
[dependencies]
sol-safekey = { path = "path/to/sol-safekey" }
```

# 或从 crates.io：
```bash
cargo install sol-safekey --features="full"
```

---

## 📖 交互式菜单完整使用指南

📖 **[查看交互式菜单完整教程 → INTERACTIVE_TUTORIAL_CN.md]**

所有交互式菜单操作的完整逐步指南，包括每个操作的详细步骤、使用场景和示例输出。

---

## 📚 文档

- **[Bot 集成指南](BOT_INTEGRATION_CN.md)** - 如何将 sol-safekey 集成到你的 bot
- **[使用手册](USER_GUIDE_CN.md)** - 完整的使用说明和示例

---

## 🔐 安全性

- ✅ **密码安全**：仅通过 stdin 管道（永不使用环境变量）
- ✅ **加密方式**：AES-256 配合 PBKDF2 密钥派生
- ✅ **内存安全**：使用后立即清除密码
- ✅ **硬件指纹**：基于设备的安全层
- ✅ **2FA 支持**：可选的双重因素认证以增强安全性


## 🛠️ 可用操作

### 钱包管理
- **创建钱包** - 使用 AES-256 生成新的加密钱包
- **导入钱包** - 从私钥或助记词导入现有钱包
- **导出钱包** - 导出钱包为加密 JSON 格式
- **查看地址** - 显示钱包公钥地址

### SOL 操作
- **查询余额** - 查看钱包中的 SOL 余额
- **转账 SOL** - 向其他地址发送 SOL
- **包装 SOL** - 将 SOL 转换为 WSOL（包装的 SOL）
- **解包 WSOL** - 将 WSOL 转换回 SOL
- **关闭 WSOL ATA** - 关闭 WSOL ATA 账户

### 代币操作
- **转账 SPL 代币** - 向其他地址发送 SPL 代币
- **查询代币余额** - 查看代币余额

### DEX 操作
- **🔥 Pump.fun 卖出** - Pump.fun DEX 上的交互式代币卖出（仅限内盘）
- **🔄 PumpSwap 卖出** - 在 PumpSwap DEX 上卖出代币

### 返现操作
- **Pump.fun 返现** - 查看和领取 pump.fun 返现（原生 SOL）
- **PumpSwap 返现** - 查看和领取 PumpSwap 返现（WSOL）

### 高级功能
- **Durable Nonce 账户** - 创建和管理用于离线签名的 nonce 账户
- **2FA 支持** - 可选的双重因素认证以增强安全性

### 交易管理
- **查询交易状态** - 在 Solana 上查询交易状态

---

## 📖 示例

参见 `examples/bot_example.rs` 获取完整的 bot 集成示例。

---

## 🤝 贡献

欢迎贡献！请确保遵循安全最佳实践。**提交与 PR 描述请使用英文。**

---

## 📄 许可证

MIT License - 详见 LICENSE 文件
