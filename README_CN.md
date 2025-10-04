<div align="center">
    <h1>🔧 Sol-SafeKey</h1>
    <h3><em>功能强大的 Solana 密钥管理工具 - 支持交互式多语言界面</em></h3>
</div>

<p align="center">
    <strong>安全地生成、管理和加密 Solana 私钥，提供易用的交互式菜单。无需记忆复杂命令！</strong>
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
    <img src="https://img.shields.io/badge/2FA-4CAF50?style=for-the-badge&logo=google-authenticator&logoColor=white" alt="2FA">
</p>

<p align="center">
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README_CN.md">中文</a> |
    <a href="https://github.com/0xfnzero/sol-safekey/blob/main/README.md">English</a> |
    <a href="https://fnzero.dev/">Website</a> |
    <a href="https://t.me/fnzero_group">Telegram</a> |
    <a href="https://discord.gg/ckf5UHxz">Discord</a>
</p>

---

## 📚 文档导航

| 文档 | 说明 | 语言 |
|------|------|------|
| [README_CN.md](./README_CN.md) | 完整使用指南 | 中文 |
| [README.md](./README.md) | Complete guide and usage | English |

---

## ✨ 功能特性

✅ **交互式菜单** - 选择语言，用数字选择操作，简单直观
🔐 **强大加密** - 基于密码的加密，使用 SHA-256 密钥派生
🌍 **多语言支持** - 完整的中文和英文界面
📦 **Keystore 格式** - 标准 Solana 钱包兼容格式
🛡️ **安全优先** - 隐藏密码输入，从不不必要地暴露敏感数据
⚡ **只需3个操作** - 创建明文私钥、创建加密私钥、解密私钥

---

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone <repository-url>
cd sol-safekey

# 编译
cargo build --release

# 运行（交互式模式 - 推荐）
./target/release/sol-safekey
```

### 交互式模式

直接运行，无需任何参数：

```bash
./sol-safekey
```

**第1步：选择语言**
```
==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]:
```

**第2步：选择操作**
```
==================================================
  Sol-SafeKey - Solana 密钥管理工具
==================================================

核心功能 (只需3个操作):

  1.  创建明文私钥
  2.  创建加密私钥
  3.  解密私钥
  0.  退出

请输入选项 [0-3]:
```

**第3步：跟随提示操作！**

---

## 📖 使用示例

### 示例 1：创建加密 Keystore（推荐）

```bash
./sol-safekey

# 选择：
# 语言: 2 (中文)
# 操作: 2 (创建加密私钥)
# 方式: 1 (生成新的密钥对并加密)
# 密码: [输入密码，至少10个字符]
# 确认密码: [再次输入]
# 输出: 1 (保存为 Keystore 文件)
# 文件路径: wallet.json (或按回车使用默认值)

# ✅ 结果：创建了 wallet.json，包含加密的私钥
```

### 示例 2：解密 Keystore

```bash
./sol-safekey

# 选择：
# 语言: 2 (中文)
# 操作: 3 (解密私钥)
# 输入: 1 (从 Keystore 文件读取)
# 文件路径: wallet.json
# 密码: [输入你的密码]

# ✅ 结果：私钥显示在屏幕上
# 可选：保存到文件或仅查看
```

### 示例 3：导入现有私钥并加密

```bash
./sol-safekey

# 选择：
# 语言: 2 (中文)
# 操作: 2 (创建加密私钥)
# 方式: 2 (导入现有私钥并加密)
# 私钥: [粘贴你的 base58 格式私钥]
# 密码: [输入密码]
# 输出: 1 (保存为 Keystore 文件)

# ✅ 结果：你的现有私钥已加密
```

---

## 💻 命令行模式（高级）

用于脚本和自动化：

```bash
# 生成带密码的加密 keystore
sol-safekey gen-keystore -o wallet.json -p "your_strong_password"

# 解密 keystore
sol-safekey unlock -f wallet.json -p "your_password"

# 加密现有私钥
sol-safekey encrypt -k "YOUR_PRIVATE_KEY" -p "your_password"

# 解密加密字符串
sol-safekey decrypt -e "ENCRYPTED_DATA" -p "your_password"

# 查看钱包地址
sol-safekey address -f wallet.json -p "your_password"

# 显示所有命令
sol-safekey --help
```

---

## 🔧 集成到你的应用程序

### Bot 集成（只需 1-2 行代码！）

**无需 CLI 依赖** - 直接集成库！

将钱包管理添加到你的 Rust bot 最简单的方式：

```rust
// 在你的 Cargo.toml 中
// [dependencies]
// sol-safekey = "0.1"

use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    let wallet_path = "config/wallet.json";

    // 就这样！一行代码获取可用的密钥对：
    let keypair = bot_helper::ensure_wallet_ready(wallet_path).unwrap();

    println!("✅ 钱包就绪！");
    println!("📍 地址: {}", keypair.pubkey());

    // 使用密钥对进行 bot 操作...
}
```

**`ensure_wallet_ready()` 做了什么：**

1. **如果钱包文件不存在：**
   - 启动交互式菜单（含语言选择）
   - 引导用户创建加密钱包
   - 保存到指定路径
   - 提示输入密码解锁
   - 返回可用的密钥对

2. **如果钱包文件已存在：**
   - 提示用户输入密码
   - 解密钱包
   - 返回可用的密钥对

**更简单 - 只需1行：**

```rust
let keypair = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

**完整 bot 示例：**

```rust
use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    // 从配置获取钱包路径
    let wallet_path = std::env::var("WALLET_PATH")
        .unwrap_or_else(|_| "wallet.json".to_string());

    // 确保钱包就绪（自动创建/解锁）
    let keypair = match bot_helper::ensure_wallet_ready(&wallet_path) {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("❌ 钱包设置失败: {}", e);
            std::process::exit(1);
        }
    };

    println!("✅ Bot 钱包就绪: {}", keypair.pubkey());

    // 你的 bot 逻辑...
    // - 使用 keypair 签署交易
    // - 监控钱包余额
    // - 执行交易等
}
```

**功能特性：**
- ✅ 无需 CLI 依赖 - 直接使用库
- ✅ 交互式语言选择（中文/英文）
- ✅ 缺少钱包时自动创建
- ✅ 已有钱包时自动解锁
- ✅ 返回可用的 `Keypair`
- ✅ 所有操作分步指导

### 使用库 API（高级）

如需更多控制，直接使用库 API：

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建加密 keystore
    let keypair = KeyManager::generate_keypair();
    let password = "your_strong_password";

    let keystore_json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("wallet.json", keystore_json)?;

    println!("钱包地址: {}", keypair.pubkey());

    // 稍后，解密它
    let keystore_json = std::fs::read_to_string("wallet.json")?;
    let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, password)?;

    // 使用密钥对签署交易
    Ok(())
}
```

### 从环境变量加载

```bash
# 在你的 .env 或环境中
WALLET_KEYSTORE_PATH=./wallet.json
WALLET_PASSWORD=your_secure_password
```

```rust
// 在你的代码中
let keystore_path = std::env::var("WALLET_KEYSTORE_PATH")?;
let password = std::env::var("WALLET_PASSWORD")?;

let keystore_json = std::fs::read_to_string(keystore_path)?;
let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, &password)?;
```

---

## 📁 文件格式

### 加密 Keystore 文件 (wallet.json)

```json
{
  "encrypted_private_key": "base64_encrypted_data...",
  "public_key": "YourWalletPublicKeyAddress...",
  "encryption_type": "password",
  "created_at": "2025-01-15T10:30:00Z",
  "version": "1.0"
}
```

### 明文 Keypair 文件 (keypair.json)

```json
[1,2,3,4,5,...,64]
```

标准 Solana keypair 格式（64 字节数组）。

---

## 🛡️ 安全最佳实践

1. ✅ **强密码**：使用 10 个字符以上，包含大写、小写、数字和符号
2. ✅ **备份 Keystore**：存储在多个安全位置（U盘、加密云盘）
3. ✅ **永不分享**：不要与任何人分享密码或私钥
4. ✅ **删除明文**：使用后立即删除解密的密钥文件
5. ✅ **先测试**：在给钱包充值前验证你能成功解密

---

## ❓ 常见问题

**问：我忘记密码了，能恢复钱包吗？**
答：不能。解密必须使用密码。这是安全设计。请始终将密码备份在安全的密码管理器中。

**问：可以在多台电脑上使用同一个 keystore 吗？**
答：**取决于 keystore 类型：**
- **标准密码加密 keystore**（交互模式选项 2 创建）：✅ 可以！完全可移植。复制 `wallet.json` 到任何机器，使用相同密码即可。
- **2FA 三因子钱包**（使用 `setup-2fa` 和 `gen-2fa-wallet` 创建）：❌ 不可以！由于硬件指纹绑定设备，无法在其他机器使用。
- **2FA 备份 keystore**（生成 2FA 钱包时同时创建的 `*_keystore.json` 文件）：✅ 可以！这个文件专门用于跨设备恢复。

**问：使用什么加密算法？**
答：XOR 加密 + 从密码派生的 SHA-256 密钥。

**问：将 wallet.json 提交到版本控制安全吗？**
答：加密的 keystore 相对安全，但我们建议将其添加到 `.gitignore` 并使用环境特定的 keystore。

**问：如何更改密码？**
答：解密 keystore 获取私钥，然后使用操作 2 用新密码创建新的 keystore。

**问：这个工具可以离线使用吗？**
答：可以！所有密钥操作都可以完全离线工作。不需要互联网连接。

**问：普通 keystore 和 2FA 钱包有什么区别？**
答：
- **普通 keystore**（交互模式 → 选项 2）：
  - ✅ 可移植（可在任何电脑使用）
  - 🔐 仅密码加密
  - 📦 单个文件（`wallet.json`）
  - 👥 推荐给大多数用户

- **2FA 三因子钱包**（高级功能）：
  - ❌ 设备绑定（硬件指纹）
  - 🔐 密码 + 安全问题 + 2FA 验证码
  - 📦 两个文件（设备绑定 + 可移植备份）
  - 🛡️ 大额资金的最高安全级别

---

## 🔥 高级功能

### 2FA 三因子认证

要获得最高安全性，启用三因子认证：

```bash
# 步骤 1: 设置 2FA（一次性）
sol-safekey setup-2fa

# 步骤 2: 使用 2FA 生成钱包
sol-safekey gen-2fa-wallet -o wallet.json
```

结合了：
- 🖥️ **硬件指纹**（设备绑定，不可移植）
- 🔑 **主密码**（用户自定义的强密码）
- ❓ **安全问题**（额外验证层）
- 📱 **2FA 验证码**（Google Authenticator/Authy）

**重要说明**：创建 2FA 钱包时，你会得到 **两个文件**：
1. `wallet.json` - 三因子加密（⚠️ **设备绑定，无法在其他电脑使用**）
2. `<地址前缀>_keystore.json` - 仅密码备份（✅ **可移植，可在任何电脑使用**）

备份 keystore 是你的安全网，用于：
- 设备损坏/丢失时恢复
- 需要在其他电脑访问钱包
- 重装操作系统后恢复

### Solana 操作

使用加密 keystore 执行 Solana 操作：

```bash
# 查询 SOL 余额
sol-safekey sol-ops -f wallet.json balance

# 转账 SOL
sol-safekey sol-ops -f wallet.json transfer -t <接收地址> -a 0.1

# 查询代币余额
sol-safekey sol-ops -f wallet.json token-balance -m <代币铸造地址>

# 将 SOL 包装为 WSOL
sol-safekey sol-ops -f wallet.json wrap-sol -a 1.0

# 将 WSOL 解包为 SOL
sol-safekey sol-ops -f wallet.json unwrap-sol
```

工具会在执行操作前提示输入密码来解密 keystore。

---

## 🌟 为什么选择 Sol-SafeKey？

| 功能 | Sol-SafeKey | 其他工具 |
|------|-------------|----------|
| 交互式菜单 | ✅ 有 | ❌ 仅命令行 |
| 多语言支持 | ✅ 中文 + 英文 | ❌ 仅英文 |
| 加密存储 | ✅ 有 | ⚠️ 通常是明文 |
| 无依赖 | ✅ 单一二进制文件 | ❌ 需要 Node.js/Python |
| 离线支持 | ✅ 完整 | ⚠️ 有限 |
| 2FA 支持 | ✅ 可选 | ❌ 无 |
| 开源 | ✅ MIT 许可证 | ✅ 各不相同 |

---

## 📄 许可证

MIT 许可证 - 可免费用于个人和商业用途。

详见 [LICENSE](./LICENSE) 文件。

---

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

---

## 💬 社区与支持

- 📧 **问题反馈**：[GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- 💬 **Telegram**：[加入我们的群组](https://t.me/fnzero_group)
- 🎮 **Discord**：[加入我们的服务器](https://discord.gg/ckf5UHxz)
- 🌐 **网站**：[fnzero.dev](https://fnzero.dev/)

---

<div align="center">
    <p>用 ❤️ 为 Solana 社区打造</p>
    <p>
        <a href="https://github.com/0xfnzero/sol-safekey">⭐ 在 GitHub 上给我们 Star</a>
    </p>
</div>
