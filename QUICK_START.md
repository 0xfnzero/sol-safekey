# Sol-SafeKey Quick Start | 快速开始

## For Bot Developers | Bot 开发者

### 1 Line to Add Wallet Management | 一行代码添加钱包管理

```rust
let keypair = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

**English:**
That's it! Your bot now has:
- ✅ Interactive wallet creation
- ✅ Automatic encryption
- ✅ Password-protected security
- ✅ Multi-language support

**中文:**
就这样！你的 bot 现在拥有：
- ✅ 交互式钱包创建
- ✅ 自动加密
- ✅ 密码保护
- ✅ 多语言支持

---

## For CLI Users | 命令行用户

### Interactive Mode (Recommended) | 交互模式（推荐）

```bash
sol-safekey start
```

**English:**
Gives you a menu with:
- Create wallet
- Encrypt/decrypt keys
- Language selection (English/Chinese)

**中文:**
菜单包含：
- 创建钱包
- 加密/解密密钥
- 语言选择（中文/英文）

---

### Advanced: 2FA Triple-Factor | 高级：2FA 三因子

```bash
# Setup 2FA | 设置 2FA
sol-safekey setup-2fa

# Generate 2FA wallet | 生成 2FA 钱包
sol-safekey gen-2fa-wallet -o wallet.json

# Unlock 2FA wallet | 解锁 2FA 钱包
sol-safekey unlock-2fa-wallet -f wallet.json
```

---

## Installation | 安装

**English:**
```bash
cargo install sol-safekey
```

Or build from source:
```bash
git clone <repo-url>
cd sol-safekey
cargo build --release
```

**中文:**
```bash
cargo install sol-safekey
```

或从源码构建：
```bash
git clone <repo-url>
cd sol-safekey
cargo build --release
```

---

## Example: Bot Integration | 示例：Bot 集成

```rust
use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    let wallet_path = "config/wallet.json";

    let keypair = bot_helper::ensure_wallet_ready(wallet_path)
        .expect("Failed to setup wallet | 钱包设置失败");

    println!("Bot ready! | Bot 就绪！Address | 地址: {}", keypair.pubkey());

    // Your bot logic... | 你的 bot 逻辑...
}
```

**English:**
- **First run:** Creates wallet interactively
- **Next runs:** Just enters password

**中文:**
- **首次运行：** 交互式创建钱包
- **后续运行：** 只需输入密码

---

## Documentation | 文档

**English:**
- [Complete README](./README.md)
- [Chinese Documentation](./README_CN.md)
- [Bot Integration Guide](./BOT_INTEGRATION.md)
- [Examples](./examples/)

**中文:**
- [完整 README](./README.md)
- [中文文档](./README_CN.md)
- [Bot 集成指南](./BOT_INTEGRATION.md)
- [示例](./examples/)

---

## CLI Commands | 命令行命令

### Core Commands | 核心命令

**English:**

| Command | Description |
|---------|-------------|
| `start` | Launch interactive menu (main command) |
| `setup-2fa` | Setup 2FA authentication |
| `gen-2fa-wallet` | Generate 2FA wallet |
| `unlock-2fa-wallet` | Unlock 2FA wallet |
| `sol-ops` | Solana operations |

**中文:**

| 命令 | 说明 |
|------|------|
| `start` | 启动交互式菜单（主命令） |
| `setup-2fa` | 设置 2FA 认证 |
| `gen-2fa-wallet` | 生成 2FA 钱包 |
| `unlock-2fa-wallet` | 解锁 2FA 钱包 |
| `sol-ops` | Solana 操作 |

---

## Features | 功能特性

**English:**
- ✅ **Simple CLI** - Only 5 core commands
- ✅ **Interactive Menu** - No commands to remember
- ✅ **Bot Integration** - Just 1 line of code
- ✅ **Multi-Language** - English/Chinese support
- ✅ **Strong Encryption** - Password-based encryption
- ✅ **2FA Support** - Triple-factor authentication
- ✅ **No CLI Dependency** - Bot uses library directly

**中文:**
- ✅ **简单 CLI** - 只有 5 个核心命令
- ✅ **交互式菜单** - 无需记住命令
- ✅ **Bot 集成** - 只需 1 行代码
- ✅ **多语言** - 支持中文/英文
- ✅ **强加密** - 基于密码的加密
- ✅ **2FA 支持** - 三因子认证
- ✅ **无 CLI 依赖** - Bot 直接使用库

---

## Quick Examples | 快速示例

### Create Encrypted Wallet | 创建加密钱包

**English:**
```bash
sol-safekey start
# Select: 2 (Create Encrypted Private Key)
# Follow prompts...
```

**中文:**
```bash
sol-safekey start
# 选择: 2 (创建加密私钥)
# 按照提示操作...
```

---

### Bot Integration | Bot 集成

**Minimal Example | 最简示例:**
```rust
let kp = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

**Full Example | 完整示例:**
```rust
use sol_safekey::bot_helper;

fn main() {
    match bot_helper::ensure_wallet_ready("wallet.json") {
        Ok(keypair) => {
            println!("✅ Ready | 就绪: {}", keypair.pubkey());
            // Use keypair... | 使用密钥对...
        }
        Err(e) => {
            eprintln!("❌ Error | 错误: {}", e);
            std::process::exit(1);
        }
    }
}
```

---

## Need Help? | 需要帮助？

**English:**
- GitHub Issues: https://github.com/0xfnzero/sol-safekey/issues
- Telegram: https://t.me/fnzero_group
- Discord: https://discord.gg/ckf5UHxz
- Website: https://fnzero.dev/

**中文:**
- GitHub Issues: https://github.com/0xfnzero/sol-safekey/issues
- Telegram: https://t.me/fnzero_group
- Discord: https://discord.gg/ckf5UHxz
- 网站: https://fnzero.dev/

---

## What's Next? | 下一步？

**English:**
1. Read [BOT_INTEGRATION.md](./BOT_INTEGRATION.md) for detailed bot integration guide
2. Check [examples/](./examples/) for working examples
3. Read [README.md](./README.md) for complete documentation

**中文:**
1. 阅读 [BOT_INTEGRATION.md](./BOT_INTEGRATION.md) 了解详细的 bot 集成指南
2. 查看 [examples/](./examples/) 查看可运行示例
3. 阅读 [README_CN.md](./README_CN.md) 了解完整文档
