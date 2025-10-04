# Bot Integration Guide | Bot 集成指南

## Quick Start | 快速开始

### 1-Line Integration | 一行代码集成

**English:**
```rust
let keypair = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

**中文:**
```rust
let keypair = sol_safekey::bot_helper::ensure_wallet_ready("wallet.json").unwrap();
```

---

## Complete Example | 完整示例

### Cargo.toml

```toml
[dependencies]
sol-safekey = "0.1"
solana-sdk = "3.0"
```

### main.rs

```rust
use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    // Get wallet path from environment or use default
    // 从环境变量获取钱包路径或使用默认值
    let wallet_path = std::env::var("WALLET_PATH")
        .unwrap_or_else(|_| "wallet.json".to_string());

    // Ensure wallet is ready (creates if missing, unlocks if exists)
    // 确保钱包就绪（不存在则创建，存在则解锁）
    let keypair = match bot_helper::ensure_wallet_ready(&wallet_path) {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("❌ Wallet setup failed | 钱包设置失败: {}", e);
            std::process::exit(1);
        }
    };

    println!("✅ Bot wallet ready | Bot 钱包就绪: {}", keypair.pubkey());

    // Your bot logic here...
    // 你的 bot 逻辑...
    // - Sign transactions | 签署交易
    // - Check balance | 查询余额
    // - Execute trades | 执行交易
}
```

---

## How It Works | 工作原理

### First Run (Wallet Doesn't Exist) | 首次运行（钱包不存在）

**English:**
```
⚠️  Wallet not found at: wallet.json
📝 Starting interactive wallet creation...

==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]: 2

[User selects language and creates encrypted wallet through interactive prompts]

✅ Wallet created successfully!
📁 Location: wallet.json

Now unlocking the newly created wallet...
🔓 Unlocking wallet: wallet.json
🔑 Enter wallet password: ********

✅ Wallet unlocked successfully!
📍 Address: [Your wallet address]
```

**中文:**
```
⚠️  钱包未找到: wallet.json
📝 启动交互式钱包创建...

==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]: 2

[用户通过交互式提示选择语言并创建加密钱包]

✅ 钱包创建成功！
📁 位置: wallet.json

现在解锁新创建的钱包...
🔓 解锁钱包: wallet.json
🔑 输入钱包密码: ********

✅ 钱包解锁成功！
📍 地址: [你的钱包地址]
```

### Subsequent Runs (Wallet Exists) | 后续运行（钱包已存在）

**English:**
```
✅ Wallet found at: wallet.json
🔓 Starting interactive wallet unlock...

🔓 Unlocking wallet: wallet.json
🔑 Enter wallet password: ********

✅ Wallet unlocked successfully!
📍 Address: [Your wallet address]
```

**中文:**
```
✅ 找到钱包: wallet.json
🔓 启动交互式钱包解锁...

🔓 解锁钱包: wallet.json
🔑 输入钱包密码: ********

✅ 钱包解锁成功！
📍 地址: [你的钱包地址]
```

---

## Features | 功能特性

**English:**
- ✅ **No CLI Dependency** - Uses library API directly
- ✅ **Auto-Create** - Creates wallet if missing
- ✅ **Auto-Unlock** - Unlocks wallet if exists
- ✅ **Interactive** - User-friendly prompts
- ✅ **Multi-Language** - English/Chinese support
- ✅ **Secure** - Password-protected encryption

**中文:**
- ✅ **无需 CLI 依赖** - 直接使用库 API
- ✅ **自动创建** - 钱包不存在时自动创建
- ✅ **自动解锁** - 钱包存在时自动解锁
- ✅ **交互式** - 用户友好的提示
- ✅ **多语言** - 支持中文/英文
- ✅ **安全** - 密码保护的加密

---

## API Reference | API 参考

### `ensure_wallet_ready(path: &str) -> Result<Keypair, String>`

**English:**
Main function for bot integration. Ensures wallet is ready to use.

**Parameters:**
- `path` - Path to wallet file

**Returns:**
- `Ok(Keypair)` - Ready-to-use keypair
- `Err(String)` - Error message

**Behavior:**
- If file doesn't exist: launches interactive creation
- If file exists: prompts for password to unlock

**中文:**
主要的 bot 集成函数。确保钱包可用。

**参数:**
- `path` - 钱包文件路径

**返回:**
- `Ok(Keypair)` - 可用的密钥对
- `Err(String)` - 错误信息

**行为:**
- 如果文件不存在：启动交互式创建
- 如果文件存在：提示输入密码解锁

---

### `wallet_exists(path: &str) -> bool`

**English:**
Check if wallet file exists.

**中文:**
检查钱包文件是否存在。

---

### `get_wallet_pubkey(path: &str) -> Result<String, String>`

**English:**
Get public key from wallet without unlocking.

**中文:**
无需解锁获取钱包公钥。

---

### `load_keypair_interactive(path: &str) -> Result<Keypair, String>`

**English:**
Load and unlock an existing wallet interactively.

**中文:**
交互式加载并解锁现有钱包。

---

## Run Example | 运行示例

**English:**
```bash
# Run the example
cargo run --example bot_integration

# With custom wallet path
WALLET_PATH=my_bot_wallet.json cargo run --example bot_integration
```

**中文:**
```bash
# 运行示例
cargo run --example bot_integration

# 使用自定义钱包路径
WALLET_PATH=my_bot_wallet.json cargo run --example bot_integration
```

---

## Troubleshooting | 故障排除

### "Wallet not created at expected path" | "钱包未在预期路径创建"

**English:**
Make sure to save the wallet to the correct path when using interactive menu.

**中文:**
使用交互式菜单时确保将钱包保存到正确的路径。

---

### "Failed to read password" | "读取密码失败"

**English:**
Terminal must support password input. Use a proper terminal emulator.

**中文:**
终端必须支持密码输入。使用合适的终端模拟器。

---

### "Wallet unlocked successfully but bot crashes" | "钱包解锁成功但 bot 崩溃"

**English:**
Check that you're using the returned `keypair` correctly with `use solana_sdk::signer::Signer;`

**中文:**
检查是否正确使用返回的 `keypair`，需要 `use solana_sdk::signer::Signer;`

---

## Support | 支持

- **GitHub Issues**: https://github.com/0xfnzero/sol-safekey/issues
- **Telegram**: https://t.me/fnzero_group
- **Discord**: https://discord.gg/ckf5UHxz
- **Website | 网站**: https://fnzero.dev/
