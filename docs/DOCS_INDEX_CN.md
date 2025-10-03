# Sol-SafeKey 文档索引

这是 Sol-SafeKey 的完整文档导航，帮助你快速找到所需的文档。

## 🚀 入门指南

| 文档 | 适用场景 | 内容 |
|------|----------|------|
| [QUICK_START.md](./QUICK_START.md) | 快速上手 | 5 分钟快速开始，常用命令速查表 |
| [README.md](../README.md) | 了解项目 | 完整的项目介绍、功能列表、CLI 使用 |

## 📦 集成文档

| 文档 | 适用场景 | 内容 |
|------|----------|------|
| [INTEGRATION.md](./INTEGRATION.md) | 库集成到项目 | 完整的库集成指南、API 示例 |
| [LIBRARY_VS_CLI.md](./LIBRARY_VS_CLI.md) | 选择使用方式 | 库 vs CLI 对比、依赖分析、选择建议 |

## 🔧 功能文档

| 文档 | 适用场景 | 内容 |
|------|----------|------|
| [SOLANA_OPS.md](./SOLANA_OPS.md) | Solana 操作 | 查询余额、转账、Token 操作等 |

## 📚 按使用场景选择文档

### 场景 1: 我想快速试用 Sol-SafeKey

👉 阅读 [QUICK_START.md](./QUICK_START.md)

**学习路径：**
1. 选择 CLI 或库集成方式
2. 跟随示例生成第一个钱包
3. 尝试加密/解密操作

---

### 场景 2: 我要集成到我的 Rust 项目

👉 阅读 [INTEGRATION.md](./INTEGRATION.md)

**学习路径：**
1. 添加依赖到 `Cargo.toml`
2. 学习核心 API（生成、加密、解密）
3. 查看完整集成示例
4. （可选）启用 Solana 操作功能

---

### 场景 3: 我要使用 CLI 工具管理密钥

👉 阅读 [README.md](../README.md) - CLI 命令部分

**学习路径：**
1. 安装 CLI 工具
2. 学习基础命令（gen-keystore, unlock, address）
3. （可选）学习高级功能（2FA、Solana 操作）

---

### 场景 4: 我需要在项目中进行 Solana 操作

👉 阅读 [SOLANA_OPS.md](./SOLANA_OPS.md)

**学习路径：**
1. 启用 `solana-ops` feature
2. 学习基础操作（查询余额）
3. 学习转账操作
4. 学习 Token 操作和 Wrap/Unwrap

---

### 场景 5: 我不知道该用 CLI 还是库集成

👉 阅读 [LIBRARY_VS_CLI.md](./LIBRARY_VS_CLI.md)

**对比要点：**
- CLI：适合个人使用、快速测试
- 库：适合项目集成、自动化、机器人

---

## 📖 完整文档列表

### 核心文档

- **[README.md](../README.md)**
  - 项目介绍
  - 功能列表
  - CLI 完整命令参考
  - 2FA 功能说明
  - 安全架构

- **[QUICK_START.md](./QUICK_START.md)**
  - 5 分钟快速上手
  - 3 种使用方式对比
  - 常用命令速查
  - 完整机器人示例

### 集成文档

- **[INTEGRATION.md](./INTEGRATION.md)**
  - 库安装方式
  - 核心 API 参考
  - 基础用法示例
  - JSON Keystore 操作
  - 完整集成示例

- **[LIBRARY_VS_CLI.md](./LIBRARY_VS_CLI.md)**
  - 功能对比表
  - 依赖分析
  - 使用场景建议
  - 集成示例对比
  - 混合使用建议

### 功能文档

- **[SOLANA_OPS.md](./SOLANA_OPS.md)**
  - 功能概览
  - CLI 使用方式（所有命令）
  - 库集成方式（完整 API）
  - 转账、Token、Wrap/Unwrap
  - 完整示例代码
  - 常见问题解答

### 代码示例

- **[examples/simple_bot.rs](./examples/simple_bot.rs)**
  - 机器人集成示例
  - 密码管理
  - 钱包加载

- **[examples/solana_bot.rs](./examples/solana_bot.rs)**
  - Solana 操作示例
  - 查询余额
  - 转账示例
  - Wrap/Unwrap 示例

## 🔍 按关键词查找

### 安装
- [安装 CLI](./README.md#as-a-cli-tool)
- [安装库](./INTEGRATION.md#安装)
- [快速安装](./QUICK_START.md)

### API 参考
- [KeyManager API](./INTEGRATION.md#api-文档)
- [Solana 操作 API](./SOLANA_OPS.md#api-参考)
- [完整 API 文档](https://docs.rs/sol-safekey)

### 示例代码
- [基础示例](./INTEGRATION.md#完整示例)
- [机器人示例](./examples/solana_bot.rs)
- [Solana 操作示例](./SOLANA_OPS.md#完整示例)

### CLI 命令
- [基础命令](./README.md#-basic-generation-commands)
- [Solana 操作命令](./SOLANA_OPS.md#cli-使用方式)
- [2FA 命令](./README.md#-triple-factor-2fa-commands-recommended)

### 功能特性
- [加密/解密](./INTEGRATION.md#使用)
- [2FA 功能](./README.md#-triple-factor-security-features)
- [Solana 操作](./SOLANA_OPS.md)
- [机器人集成](./README.md#-bot-integration-recommended-for-bots)

### 配置
- [依赖配置](./INTEGRATION.md#安装)
- [Features 说明](./LIBRARY_VS_CLI.md#依赖对比)
- [环境变量](./QUICK_START.md#环境变量配置)

## 💡 学习建议

### 新手路径
1. 阅读 [QUICK_START.md](./QUICK_START.md) - 了解基础使用
2. 尝试 CLI 工具 - 体验功能
3. 根据需求选择深入阅读：
   - 项目集成 → [INTEGRATION.md](./INTEGRATION.md)
   - Solana 操作 → [SOLANA_OPS.md](./SOLANA_OPS.md)

### 开发者路径
1. 阅读 [LIBRARY_VS_CLI.md](./LIBRARY_VS_CLI.md) - 选择使用方式
2. 阅读 [INTEGRATION.md](./INTEGRATION.md) - 学习 API
3. 查看示例代码 - 理解实际用法
4. （可选）阅读 [SOLANA_OPS.md](./SOLANA_OPS.md) - 增加 Solana 功能

### 机器人开发路径
1. 阅读 [README.md - Bot Integration](./README.md#-bot-integration-recommended-for-bots)
2. 查看 [examples/solana_bot.rs](./examples/solana_bot.rs)
3. 阅读 [SOLANA_OPS.md](./SOLANA_OPS.md) - 学习交易操作
4. 阅读 [QUICK_START.md](./QUICK_START.md) - 环境配置

## 🔗 外部链接

- 📦 [Crates.io](https://crates.io/crates/sol-safekey)
- 📖 [API 文档](https://docs.rs/sol-safekey)
- 🐙 [GitHub 仓库](https://github.com/0xfnzero/sol-safekey)
- 💬 [Telegram 群组](https://t.me/fnzero_group)
- 💬 [Discord](https://discord.gg/ckf5UHxz)

## 📝 文档贡献

如果发现文档有误或需要补充，欢迎：
1. 提交 [Issue](https://github.com/0xfnzero/sol-safekey/issues)
2. 提交 [Pull Request](https://github.com/0xfnzero/sol-safekey/pulls)

---

**最后更新：** 2025-10-03
