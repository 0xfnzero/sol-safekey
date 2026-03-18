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
- [📦 安装](#-安装)
- [📋 功能指南](#-功能指南)
- [📚 文档](#-文档)
- [🔐 安全性](#-安全性)
- [📖 示例](#-示例)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)
- [📖 示例](#-示例)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)

### 📋 功能指南

#### 🎯 开始使用

#### 步骤 1：启动交互式菜单
```bash
sol-safekey start
```

你将看到语言选择界面。选择你偏好的语言：

**英文**：输入 `2`
**中文**：输入 `1`

#### 步骤 2：选择一个操作
选择语言后，你将看到主菜单。输入与你想执行的操作对应的数字。

**重要提示**：如果你还没有创建钱包，你需要：
- **解锁现有钱包**（选项 `U`）
- **创建新钱包**（选项 `1` 或 `2`）

---

#### 🔑 核心功能（选项 1-3）
- **[1. 创建明文私钥](INTERACTIVE_TUTORIAL_CN.md#1-创建明文私钥（选项-1）)** - 生成未加密密钥对（仅测试）
- **[2. 创建加密私钥](INTERACTIVE_TUTORIAL_CN.md#2-创建加密私钥bot)（选项-2）)** - 加密并保存到 keystore
- **[3. 解密私钥](INTERACTIVE_TUTORIAL_CN.md#3-解密私钥（选项-3）)** - 解密 keystore 显示私钥

#### 🔒 钱包管理（选项 U）
- **[U. 解锁钱包](INTERACTIVE_TUTORIAL_CN.md#u-解锁钱包)** - 解锁钱包进行 Solana 操作

#### 🛡️ 高级安全（选项 4-6）
- **[4. 设置 2FA](INTERACTIVE_TUTORIAL_CN.md#4-设置-2fa-认证)** - 配置双因素认证
- **[5. 生成三因子钱包](INTERACTIVE_TUTORIAL_CN.md#5-生成三因子钱包)** - 创建 3FA 钱包
- **[6. 解锁三因子钱包](INTERACTIVE_TUTORIAL_CN.md#6-解锁三因子钱包)** - 解密 3FA 加密钱包

#### 💰 Solana 操作（选项 7-18）
##### 余额与转账
- **[7. 查询余额](INTERACTIVE_TUTORIAL_CN.md#7-查询-sol-余额)** - 查询 SOL 余额
- **[8. 转账 SOL](INTERACTIVE_TUTORIAL_CN.md#8-转账-sol)** - 发送 SOL

##### WSOL 操作
- **[9. 创建 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#9-创建-wsol-ata)** - 创建 WSOL 关联代币账户
- **[10. 包装 SOL](INTERACTIVE_TUTORIAL_CN.md#10-包装-sol--wsol)** - SOL → WSOL
- **[11. 解包 WSOL](INTERACTIVE_TUTORIAL_CN.md#11-解包-wsol--sol)** - WSOL → SOL
- **[12. 关闭 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#12-关闭-wsol-ata)** - 关闭 WSOL ATA

##### 代币操作
- **[13. 转账 SPL 代币](INTERACTIVE_TUTORIAL_CN.md#13-转账-spl-代币)** - 发送 SPL 代币
- **[14. 创建 Nonce 账户](INTERACTIVE_TUTORIAL_CN.md#14-创建-nonce-账户)** - 创建持久化 nonce

##### DEX 操作
- **[15. Pump.fun 卖出](INTERACTIVE_TUTORIAL_CN.md#15-pumpfun-卖出代币)** - Pump.fun DEX 卖出
- **[16. PumpSwap 卖出](INTERACTIVE_TUTORIAL_CN.md#16-pumpswap-卖出代币)** - PumpSwap DEX 卖出

##### 返现操作
- **[17. Pump.fun 返现](INTERACTIVE_TUTORIAL_CN.md#17-pumpfun-返现-查看与领取)** - pump.fun 返现（SOL）
- **[18. PumpSwap 返现](INTERACTIVE_TUTORIAL_CN.md#18-pumpswap-返现-查看与领取)** - PumpSwap 返现（WSOL）

---
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


## 📖 示例

参见 `examples/bot_example.rs` 获取完整的 bot 集成示例。

---

## 🤝 贡献

欢迎贡献！请确保遵循安全最佳实践。**提交与 PR 描述请使用英文。**

---

## 📄 许可证

MIT License - 详见 LICENSE 文件
