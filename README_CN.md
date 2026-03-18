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
- [📖 交互式菜单完整使用指南](#-交互式菜单完整使用指南)
- [Bot 开发者](#bot-开发者)
- [作为库使用](#-作为库使用)
- [📚 文档](#-文档)
- [🔐 安全性](#-安全性)
- [📦 安装](#-安装)
- [🛠️ 可用操作](#️-可用操作)
- [📖 示例](#-示例)
- [🤝 贡献](#-贡献)
- [📄 许可证](#-许可证)

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

## 📋 完整菜单索引

所有 18 个交互式菜单操作的快速概览，按类别组织：

### 🔑 核心功能（选项 1-3）

- **[1. 创建明文私钥](INTERACTIVE_TUTORIAL_CN.md#1-创建明文私钥（选项-1）)** - 生成并保存未加密的 Solana 密钥对（仅用于测试）
- **[2. 创建加密私钥(bot)](INTERACTIVE_TUTORIAL_CN.md#2-创建加密私钥bot)（选项-2）)** - 加密现有私钥并保存到 keystore
- **[3. 解密私钥](INTERACTIVE_TUTORIAL_CN.md#3-解密私钥（选项-3）)** - 解密 keystore 并显示私钥

### 🔒 钱包管理（选项 U）

- **[U. 解锁钱包](INTERACTIVE_TUTORIAL_CN.md#u-解锁钱包)** - 解锁钱包以进行 Solana 操作

### 🛡️ 高级安全功能（选项 4-6）

- **[4. 设置 2FA 认证](INTERACTIVE_TUTORIAL_CN.md#4-设置-2fa-认证)** - 配置双因素认证
- **[5. 生成三因子钱包](INTERACTIVE_TUTORIAL_CN.md#5-生成三因子钱包)** - 创建 3FA 钱包（硬件 + 密码 + 安全问题 + 2FA）
- **[6. 解锁三因子钱包](INTERACTIVE_TUTORIAL_CN.md#6-解锁三因子钱包)** - 解密 3FA 加密钱包

### 💰 Solana 链上操作（选项 7-18）

#### 余额与转账
- **[7. 查询 SOL 余额](INTERACTIVE_TUTORIAL_CN.md#7-查询-sol-余额)** - 查询钱包 SOL 余额
- **[8. 转账 SOL](INTERACTIVE_TUTORIAL_CN.md#8-转账-sol)** - 向其他地址发送 SOL

#### WSOL 操作
- **[9. 创建 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#9-创建-wsol-ata)** - 创建包装 SOL 关联代币账户
- **[10. 包装 SOL → WSOL](INTERACTIVE_TUTORIAL_CN.md#10-包装-sol--wsol)** - 将 SOL 转换为包装 SOL
- **[11. 解包 WSOL → SOL](INTERACTIVE_TUTORIAL_CN.md#11-解包-wsol--sol)** - 将包装 SOL 转换回 SOL
- **[12. 关闭 WSOL ATA](INTERACTIVE_TUTORIAL_CN.md#12-关闭-wsol-ata)** - 关闭 WSOL ATA 并将剩余 WSOL 转换为 SOL

#### 代币操作
- **[13. 转账 SPL 代币](INTERACTIVE_TUTORIAL_CN.md#13-转账-spl-代币)** - 向其他地址发送 SPL 代币

#### 交易工具
- **[14. 创建 Nonce 账户](INTERACTIVE_TUTORIAL_CN.md#14-创建-nonce-账户)** - 创建持久化 nonce 以防止交易重放

#### DEX 操作
- **[15. Pump.fun 卖出代币](INTERACTIVE_TUTORIAL_CN.md#15-pumpfun-卖出代币)** - 在 Pump.fun DEX 上卖出代币（内盘）
- **[16. PumpSwap 卖出代币](INTERACTIVE_TUTORIAL_CN.md#16-pumpswap-卖出代币)** - 在 PumpSwap DEX 上卖出代币

#### 返现操作
- **[17. Pump.fun 返现](INTERACTIVE_TUTORIAL_CN.md#17-pumpfun-返现（查看与领取））** - 查看并领取 pump.fun 返现（原生 SOL）
- **[18. PumpSwap 返现](INTERACTIVE_TUTORIAL_CN.md#18-pumpswap-返现（查看与领取））** - 查看并领取 PumpSwap 返现（WSOL）

### 🎯 快速访问

常用任务及其直接链接：

- 🔑 **[钱包设置](INTERACTIVE_TUTORIAL_CN.md#u-解锁钱包)** - 解锁钱包并开始使用
- 💰 **[查询余额](INTERACTIVE_TUTORIAL_CN.md#7-查询-sol-余额)** - 快速查询 SOL 余额
- 🔑 **[Nonce 账户](INTERACTIVE_TUTORIAL_CN.md#14-创建-nonce-账户)** - 创建持久化 nonce 用于交易机器人
- 🏪 **[DEX 操作](INTERACTIVE_TUTORIAL_CN.md#15-pumpfun-卖出代币)** - 访问 Pump.fun 和 PumpSwap 卖出功能

---

## 📖 交互式菜单完整使用指南

📖 **[查看交互式菜单完整教程 → INTERACTIVE_TUTORIAL_CN.md]**

所有交互式菜单操作的完整逐步指南，包括每个操作的详细步骤、使用场景和示例输出。

---

## Bot 开发者

#### 步骤 1：启动交互式菜单

```bash
sol-safekey start
```

你将看到语言选择界面。选择你偏好的语言：

**英文**：输入 `2`
**中文**：输入 `1`

#### 步骤 2：选择一个操作

选择语言后，你将看到上面显示的主菜单。输入与你想执行的操作对应的数字。

**重要提示**：如果你还没有创建钱包，你需要：
- **解锁现有钱包**（选项 `U`）
- **创建新钱包**（选项 `1` 或 `2`）

---

### 📚 逐功能指南

#### 1. 创建明文私钥（选项 1）

**用途**：生成新的 Solana 密钥对并保存到文件（未加密）

**使用场景**：
- 开发和测试
- 与团队成员共享密钥
- 快速生成测试钱包

**步骤**：
1. 从主菜单选择 `1`
2. 选择保存文件名（默认：`keypair.json`）
3. 系统生成新的 Solana 密钥对
4. 私钥被保存到文件（未加密）

**示例输出**：
```
🔓 创建明文私钥

输入文件名 [keypair.json]: my_keypair.json

正在生成新的 Solana 密钥对...
✅ 密钥对生成成功！

公钥: 7xKm...9xW3

📝 重要提示：
   • 此文件包含你的私钥（明文）
   • 仅用于开发和测试
   • 请勿与任何人共享此文件
   • 生产环境请使用加密选项（选项 2）
```

---

#### 2. 创建加密私钥(bot)（选项 2）

**用途**：加密现有私钥并保存到安全 keystore 文件

**使用场景**：
- 生产钱包存储
- 安全备份现有密钥
- 为 bot 集成准备钱包

**步骤**：
1. 从主菜单选择 `2`
2. 粘贴或输入你的私钥（base58 格式）
3. 输入密码（10-20 字符，必须包含大写、小写、数字、特殊字符中的至少 3 种）
4. 再次输入密码确认
5. 选择文件名（默认：`keystore.json`）
6. 系统加密并保存到文件

**密码要求**：
- 长度：10-20 字符
- 必须包含至少 3 种：大写、小写、数字、特殊字符
- 示例：`MySecureP@ssw0rd!`、`StrongKey#2025`、`abc123XYZ!`

**示例输出**：
```
🔐 创建加密私钥(bot)

输入或粘贴你的私钥（base58）: <在此处粘贴你的密钥>

请设置密码（10-20 字符，必须包含大写、小写、数字、特殊字符中的至少 3 种）:
输入密码: ********
再次输入密码: ********

输入保存文件名 [keystore.json]: my_keystore.json

正在加密并保存...
✅ 密钥已加密并保存到: my_keystore.json

📝 重要提示：
   • 解密时需要密码 - 请勿丢失！
   • keystore.json 包含 AES-256 加密的私钥
   • 建议将 keystore.json 备份到多个安全位置
```

---

#### 3. 解密私钥（选项 3）

**用途**：加载加密的 keystore 文件并解密以显示私钥

**使用场景**：
- 查看私钥（需要时）
- 导出钱包到不同格式
- 验证钱包内容

**步骤**：
1. 从主菜单选择 `3`
2. 输入 keystore 文件名（默认：`keystore.json`）
3. 输入你的密码
4. 系统解密并显示私钥

**示例输出**：
```
🔓 解密私钥

输入 keystore 文件名 [keystore.json]: my_keystore.json

输入密码: ********

✅ 解密成功！

公钥: 7xKm...9xW3
私钥: <base58-编码的密钥>

⚠️  安全警告:
   • 私钥现在以明文显示
   • 请让此屏幕远离窥视
   • 仅在必要时使用解密后的密钥
```

---

### 🔒 钱包管理

#### U. 解锁钱包

**用途**：解锁钱包以用于 Solana 操作（选项 7-18）

**步骤**：
1. 从主菜单选择 `U`
2. 输入 keystore 文件名（默认：`keystore.json`）
3. 输入你的密码
4. 钱包状态变为"已解锁"并保存在会话中

**钱包状态指示器**：

钱包状态指示器显示：
- **已解锁**：钱包已准备好进行操作
- **已锁定**：会话中未加载钱包

**示例输出**：
```
🔓 解锁钱包

输入 keystore 文件名 [keystore.json]:

输入密码: ********

✅ 钱包解锁成功！

📍 当前钱包: 7xKm...9xW3
🔒 钱包状态: 已解锁

你现在可以执行 Solana 操作（选项 7-18）
```

---

### 🛠️ Solana 链上操作

所有 Solana 操作（选项 7-18）都需要**已解锁的钱包**。请先解锁你的钱包！

#### 7. 查询 SOL 余额

**用途**：查询你钱包的 SOL 余额

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `7`
3. 输入 RPC 地址（按回车使用默认：mainnet-beta）
4. 系统查询并显示你的余额

**示例输出**：
```
💰 查询 SOL 余额

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

正在查询余额...
✅ 余额查询成功！

📍 钱包地址: 7xKm...9xW3
💰 SOL 余额: 1.234567890 SOL (1,234,567,890 lamports)

浏览器: https://solscan.io/address/7xKm...9xW3
```

---

#### 8. 转账 SOL

**用途**：向其他 Solana 地址发送 SOL

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `8`
3. 输入接收地址
4. 输入 SOL 数量
5. 输入 RPC 地址（按回车使用默认）
6. 审查并确认交易
7. 系统发送交易

**示例输出**：
```
💸 转账 SOL

输入接收地址: 5xKm...2xW3

输入 SOL 数量: 0.1

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

确认交易？(yes/no): yes

正在发送交易...
✅ 交易发送成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 9. 创建 WSOL ATA

**用途**：创建包装 SOL（WSOL）关联代币账户

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `9`
3. 输入 RPC 地址（按回车使用默认）
4. 系统创建 WSOL ATA 账户

**示例输出**：
```
📝 创建 WSOL ATA

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

正在创建 WSOL ATA...
✅ WSOL ATA 创建成功！

📍 ATA 地址: 7xKm...9xW3
📊 代币铸造地址: So11111111111111111111111111111111111111111112

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 10. 包装 SOL → WSOL

**用途**：将 SOL 转换为 WSOL（包装的 SOL）

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `10`
3. 输入要包装的 SOL 数量
4. 输入 RPC 地址（按回车使用默认）
5. 审查并确认交易
6. 系统将 SOL 包装为 WSOL

**示例输出**：
```
📦 包装 SOL → WSOL

输入 SOL 数量: 0.5

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

确认交易？(yes/no): yes

正在包装 SOL 为 WSOL...
✅ 包装成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3

✅ WSOL 余额更新: 0.5 WSOL
```

---

#### 11. 解包 WSOL → SOL

**用途**：将 WSOL 转换回 SOL

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `11`
3. 输入要解包的 WSOL 数量
4. 输入 RPC 地址（按回车使用默认）
5. 审查并确认交易
6. 系统将 WSOL 解包为 SOL

**示例输出**：
```
📤 解包 WSOL → SOL

输入 WSOL 数量: 0.5

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

确认交易？(yes/no): yes

正在解包 WSOL 为 SOL...
✅ 解包成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3

✅ SOL 余额更新!
```

---

#### 12. 关闭 WSOL ATA

**用途**：关闭 WSOL ATA 账户（将剩余 WSOL 转换为 SOL）

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `12`
3. 输入 RPC 地址（按回车使用默认）
4. 系统关闭 ATA 账户

**示例输出**：
```
🗑️ 关闭 WSOL ATA

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

正在关闭 WSOL ATA...
✅ WSOL ATA 关闭成功!

剩余 WSOL: 0.5 WSOL → 0.5 SOL
签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 13. 转账 SPL 代币

**用途**：向其他 Solana 地址发送 SPL 代币

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `13`
3. 输入代币铸造地址
4. 输入接收地址
5. 输入数量
6. 输入 RPC 地址（按回车使用默认）
7. 审查并确认交易
8. 系统发送代币

**示例输出**：
```
💎 转账 SPL 代币

输入代币铸造地址: <代币铸造地址>

输入接收地址: 5xKm...2xW3

输入数量: 100

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

确认交易？(yes/no): yes

正在发送 SPL 代币...
✅ 代币转账成功!

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 14. 创建 Nonce 账户

**用途**：创建持久化 nonce 账户以防止交易重放攻击

**使用场景**：
- 批量交易处理
- 防止交易重放攻击
- 确保交易排序

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `14`
3. 输入 RPC 地址（按回车使用默认）
4. 系统创建 nonce 账户

**示例输出**：
```
🔑 创建 Nonce 账户

输入 RPC 地址 [https://api.mainnet-beta.solana.com]:

正在创建 nonce 账户...
✅ Nonce 账户创建并初始化成功！

📍 Nonce 地址: 5xKm...7xW3

💡 保存此 nonce 账户地址以供将来使用！
签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

### 🏪 代币操作

#### 15. Pump.fun 卖出代币

**用途**：在 Pump.fun DEX 上卖出代币

**使用场景**：
- 在 Pump.fun 平台上交易代币
- 自动化卖出策略
- 快速流动性退出

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `15`
3. 输入代币铸造地址
4. 配置卖出选项：
   - 滑点（基点，默认：9900）
   - Seed 优化（默认：是）
5. 确认交易
6. 系统卖出所有代币余额

**示例输出**：
```
🎪 Pump.fun 卖出代币

输入代币铸造地址: <代币铸造地址>

输入滑点（基点，默认 9900）: [Enter]

使用 seed 优化？(yes/no，默认 yes) [Enter]

卖出所有代币...
✅ 卖出成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 16. PumpSwap 卖出代币

**用途**：在 PumpSwap DEX 上卖出代币

**使用场景**：
- 在 PumpSwap 平台上交易代币
- 访问多个流动性池
- 高级交易功能

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `16`
3. 输入代币铸造地址
4. 配置卖出选项
5. 确认交易
6. 系统卖出代币

**示例输出**：
```
🔄 PumpSwap 卖出代币

输入代币铸造地址: <代币铸造地址>

输入滑点（基点，默认 9900）: [Enter]

卖出代币...
✅ 卖出成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 17. Pump.fun 返现（查看与领取）

**用途**：查看和领取 pump.fun 返现（原生 SOL）

**使用场景**：
- 查看可用的返现余额
- 领取已赚取的返现

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `17`
3. 系统查询并领取可用返现

**示例输出**：
```
💰 Pump.fun 返现

正在查询返现状态...
✅ 可用返现: 0.123 SOL

领取返现...
✅ 返现领取成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

#### 18. PumpSwap 返现（查看与领取）

**用途**：查看和领取 PumpSwap 返现（WSOL）

**使用场景**：
- 查看可用的 WSOL 返现余额
- 领取已赚取的 WSOL 返现

**步骤**：
1. 确保钱包已解锁（状态：已解锁）
2. 从主菜单选择 `18`
3. 系统查询并领取可用返现

**示例输出**：
```
💰 PumpSwap 返现

正在查询返现状态...
✅ 可用返现: 0.05 WSOL

领取返现...
✅ 返现领取成功！

签名: 5xKm...9xW3
浏览器: https://solscan.io/tx/5xKm...9xW3
```

---

### 🔐 高级安全功能

#### 4. 设置 2FA 认证

**用途**：为钱包配置双重因素认证以增强安全性

**使用场景**：
- 为钱包添加额外的安全层
- 防止未授权访问
- 三因子钱包的必需功能

**步骤**：
1. 从主菜单选择 `4`
2. 系统收集硬件指纹
3. 设置主密码
4. 设置安全问题
5. 配置 TOTP（基于时间的一次性密码）

**要求**：
- 硬件指纹收集（设备相关）
- TOTP 认证器应用（Google Authenticator、Authy 等）
- 主密码（10-20 字符，3+ 字符类型）
- 安全问题（从预定义选项中选择）

**输出**：
- ✅ 硬件指纹（设备绑定）
- ✅ 主密码加密
- ✅ 安全问题验证
- ✅ TOTP 配置（6 位代码，30 秒轮换）

---

#### 5. 生成三因子钱包

**用途**：生成三因素认证钱包（硬件 + 密码 + 安全问题 + 2FA）

**安全特性**：
- ✅ 硬件指纹（设备绑定）
- ✅ 主密码加密
- ✅ 安全问题验证
- ✅ TOTP 动态代码（6 位，30 秒轮换）
- ✅ Durable nonce 账户支持

**使用场景**：
- 生产环境的最高安全性
- 多设备支持（使用 TOTP）
- 多种身份验证因素恢复

**步骤**：
1. 从主菜单选择 `5`
2. 系统收集硬件指纹
3. 设置主密码
4. 回答安全问题
5. 配置 TOTP 认证器
6. 系统生成加密钱包文件

**输出**：
- 加密钱包文件（三因素）
- 跨设备 keystore 备份
- 恢复说明

---

#### 6. 解锁三因子钱包

**用途**：解密三因素加密钱包

**要求**：
- 原始设备（用于硬件指纹）
- 主密码
- 安全问题答案
- 当前 TOTP 代码（6 位，每 30 秒变化）

**步骤**：
1. 从主菜单选择 `6`
2. 输入钱包文件路径（从选项 5 生成）
3. 输入主密码
4. 回答安全问题
5. 输入当前 TOTP 代码（6 位，30 秒内有效）
6. 系统解密并显示钱包

**安全性**：所有三种因素必须正确才能解锁。

---

## Bot 开发者

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
