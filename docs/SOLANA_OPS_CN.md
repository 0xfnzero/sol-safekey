# Solana 操作功能使用指南

本指南介绍如何使用 Sol-SafeKey 的 Solana 操作功能，包括查询余额、转账、Token 操作等。

## 📋 目录

- [功能概览](#功能概览)
- [CLI 使用方式](#cli-使用方式)
- [库集成方式](#库集成方式)
- [完整示例](#完整示例)
- [常见问题](#常见问题)

---

## 功能概览

Sol-SafeKey 提供以下 Solana 操作功能：

| 功能 | 说明 | CLI 命令 | 库方法 |
|------|------|----------|--------|
| 查询 SOL 余额 | 查询钱包的 SOL 余额 | `balance` | `get_sol_balance()` |
| 查询 Token 余额 | 查询 SPL Token 余额 | `token-balance` | `get_token_balance()` |
| 转账 SOL | 转账 SOL 到其他地址 | `transfer` | `transfer_sol()` |
| 转账 Token | 转账 SPL Token | `transfer-token` | `transfer_token()` |
| SOL → WSOL | 将 SOL 包装为 WSOL | `wrap-sol` | `wrap_sol()` |
| WSOL → SOL | 将 WSOL 解包为 SOL | `unwrap-sol` | `unwrap_sol()` |

---

## CLI 使用方式

### 前提条件

安装完整版 CLI：

```bash
cargo install sol-safekey --features full
```

或从源码构建：

```bash
cargo build --release --features full
```

### 基础用法

所有 Solana 操作命令的格式：

```bash
sol-safekey sol-ops -f <加密钱包文件> <子命令> [选项]
```

### 1. 查询 SOL 余额

```bash
# 查询加密钱包的余额
sol-safekey sol-ops -f wallet.json balance

# 查询指定地址的余额
sol-safekey sol-ops -f wallet.json balance -a <钱包地址>

# 使用自定义 RPC
sol-safekey sol-ops -f wallet.json balance -r https://api.devnet.solana.com
```

**示例输出：**
```
🔐 Loading encrypted keypair...
Enter password: ********
✅ Keypair loaded successfully!
Public key: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu

📊 Checking SOL balance...

Balance Information:
Address: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu
Balance: 1.5 SOL (1500000000 lamports)
```

### 2. 查询 Token 余额

```bash
# 查询 USDC 余额
sol-safekey sol-ops -f wallet.json token-balance \
  -m EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# 查询其他地址的 Token 余额
sol-safekey sol-ops -f wallet.json token-balance \
  -m <TOKEN_MINT> \
  -a <钱包地址>
```

**示例输出：**
```
📊 Checking token balance...

Token Balance Information:
Address: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu
Token Mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
Balance: 1000000 (smallest units)
Balance (9 decimals): 0.001
```

### 3. 转账 SOL

```bash
# 转账 0.1 SOL
sol-safekey sol-ops -f wallet.json transfer \
  -t <接收地址> \
  -a 0.1

# 使用 devnet
sol-safekey sol-ops -f wallet.json transfer \
  -t <接收地址> \
  -a 0.1 \
  -r https://api.devnet.solana.com
```

**交互流程：**
```
🔐 Loading encrypted keypair...
Enter password: ********
✅ Keypair loaded successfully!

💸 Preparing SOL transfer...
From: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu
To: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
Amount: 0.1 SOL (100000000 lamports)

Confirm transfer? (yes/no): yes

🚀 Sending transaction...

✅ Transfer successful!
Signature: 5jK3mN...xyz123
Explorer: https://solscan.io/tx/5jK3mN...xyz123
```

### 4. 转账 Token

```bash
# 转账 1000 个 Token (最小单位)
sol-safekey sol-ops -f wallet.json transfer-token \
  -m <TOKEN_MINT> \
  -t <接收地址> \
  -a 1000

# 转账 1 个完整 Token (9 位小数)
# 1 Token = 1,000,000,000 最小单位
sol-safekey sol-ops -f wallet.json transfer-token \
  -m <TOKEN_MINT> \
  -t <接收地址> \
  -a 1000000000
```

### 5. SOL → WSOL (包装)

```bash
# 将 0.5 SOL 包装为 WSOL
sol-safekey sol-ops -f wallet.json wrap-sol -a 0.5
```

**用途：**
- WSOL 是 SOL 的 SPL Token 版本
- 可用于 DEX 交易（如 Raydium, Orca）
- 某些程序需要 WSOL 而不是原生 SOL

### 6. WSOL → SOL (解包)

```bash
# 将所有 WSOL 解包回 SOL
sol-safekey sol-ops -f wallet.json unwrap-sol
```

---

## 库集成方式

### 安装

在 `Cargo.toml` 中启用 `solana-ops` feature：

```toml
[dependencies]
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }
solana-sdk = "3.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### 基础用法

```rust
use sol_safekey::solana_utils::{SolanaClient, lamports_to_sol};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建客户端
    let client = SolanaClient::new(
        "https://api.mainnet-beta.solana.com".to_string()
    );

    // 从加密文件加载密钥对（这里简化处理）
    let keypair = Keypair::new(); // 实际使用 KeyManager 解密

    // 查询余额
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    println!("Balance: {} SOL", lamports_to_sol(balance));

    Ok(())
}
```

### 完整集成示例

创建文件 `examples/solana_bot.rs`：

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, pubkey::Pubkey, signer::Signer};
use std::{str::FromStr, fs};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 加载加密钱包
    println!("🔐 Loading encrypted wallet...");
    let json = fs::read_to_string("wallet.json")?;
    let password = "your_password"; // 实际使用中从环境变量或用户输入获取
    let keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;
    println!("✅ Wallet loaded: {}", keypair.pubkey());

    // 2. 创建 Solana 客户端
    let client = SolanaClient::new(
        "https://api.mainnet-beta.solana.com".to_string()
    );

    // 3. 查询 SOL 余额
    println!("\n📊 Checking SOL balance...");
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    println!("Balance: {} SOL", lamports_to_sol(balance));

    // 4. 查询 Token 余额 (USDC 示例)
    println!("\n📊 Checking USDC balance...");
    let usdc_mint = Pubkey::from_str(
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    )?;
    let token_balance = client.get_token_balance(
        &keypair.pubkey(),
        &usdc_mint
    ).await?;
    println!("USDC: {} (smallest units)", token_balance);
    println!("USDC: {} (6 decimals)", format_token_amount(token_balance, 6));

    // 5. 转账示例（注释掉以防误操作）
    /*
    println!("\n💸 Transferring SOL...");
    let recipient = Pubkey::from_str("...")?;
    let lamports = 100_000_000; // 0.1 SOL

    let signature = client.transfer_sol(
        &keypair,
        &recipient,
        lamports
    ).await?;
    println!("✅ Transfer successful!");
    println!("Signature: {}", signature);
    */

    // 6. Wrap SOL 示例
    /*
    println!("\n🔄 Wrapping SOL...");
    let wrap_amount = 500_000_000; // 0.5 SOL
    let signature = client.wrap_sol(&keypair, wrap_amount).await?;
    println!("✅ Wrap successful!");
    println!("Signature: {}", signature);
    */

    Ok(())
}
```

运行示例：

```bash
cargo run --example solana_bot --features solana-ops
```

### API 参考

#### `SolanaClient::new(rpc_url: String) -> Self`

创建新的 Solana 客户端。

**参数：**
- `rpc_url` - RPC 节点 URL
  - Mainnet: `https://api.mainnet-beta.solana.com`
  - Devnet: `https://api.devnet.solana.com`
  - Testnet: `https://api.testnet.solana.com`
  - 自定义 RPC (如 QuickNode, Alchemy 等)

#### `async fn get_sol_balance(&self, pubkey: &Pubkey) -> Result<u64>`

查询 SOL 余额（lamports）。

**返回值：** 余额（单位：lamports，1 SOL = 1,000,000,000 lamports）

**示例：**
```rust
let balance = client.get_sol_balance(&keypair.pubkey()).await?;
println!("Balance: {} SOL", lamports_to_sol(balance));
```

#### `async fn get_token_balance(&self, owner: &Pubkey, mint: &Pubkey) -> Result<u64>`

查询 SPL Token 余额。

**参数：**
- `owner` - 钱包地址
- `mint` - Token Mint 地址

**返回值：** Token 余额（最小单位）

**示例：**
```rust
let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
let balance = client.get_token_balance(&keypair.pubkey(), &usdc_mint).await?;
println!("Balance: {}", format_token_amount(balance, 6)); // USDC 有 6 位小数
```

#### `async fn transfer_sol(&self, from: &Keypair, to: &Pubkey, amount: u64) -> Result<Signature>`

转账 SOL。

**参数：**
- `from` - 发送方密钥对
- `to` - 接收方地址
- `amount` - 金额（lamports）

**返回值：** 交易签名

**示例：**
```rust
use solana_sdk::native_token::LAMPORTS_PER_SOL;

let recipient = Pubkey::from_str("...")?;
let amount = (0.1 * LAMPORTS_PER_SOL as f64) as u64; // 0.1 SOL

let signature = client.transfer_sol(&keypair, &recipient, amount).await?;
println!("Transaction: https://solscan.io/tx/{}", signature);
```

#### `async fn transfer_token(&self, from: &Keypair, to: &Pubkey, mint: &Pubkey, amount: u64) -> Result<Signature>`

转账 SPL Token。

**参数：**
- `from` - 发送方密钥对
- `to` - 接收方地址
- `mint` - Token Mint 地址
- `amount` - 金额（最小单位）

**示例：**
```rust
let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?;
let recipient = Pubkey::from_str("...")?;
let amount = 1_000_000; // 1 USDC (6 decimals)

let signature = client.transfer_token(
    &keypair,
    &recipient,
    &usdc_mint,
    amount
).await?;
```

#### `async fn wrap_sol(&self, keypair: &Keypair, amount: u64) -> Result<Signature>`

将 SOL 包装为 WSOL。

**参数：**
- `keypair` - 钱包密钥对
- `amount` - 包装金额（lamports）

#### `async fn unwrap_sol(&self, keypair: &Keypair) -> Result<Signature>`

将 WSOL 解包为 SOL。

**参数：**
- `keypair` - 钱包密钥对

### 工具函数

#### `lamports_to_sol(lamports: u64) -> f64`

将 lamports 转换为 SOL。

```rust
let lamports = 1_500_000_000;
let sol = lamports_to_sol(lamports); // 1.5
```

#### `format_token_amount(amount: u64, decimals: u8) -> f64`

将 Token 最小单位转换为人类可读格式。

```rust
let usdc_amount = 1_000_000; // USDC 最小单位
let readable = format_token_amount(usdc_amount, 6); // 1.0 USDC
```

---

## 完整示例

### 交易机器人示例

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, pubkey::Pubkey, signer::Signer};
use std::{str::FromStr, fs, env};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量加载配置
    dotenv::dotenv().ok();
    let wallet_path = env::var("WALLET_PATH")?;
    let password = env::var("WALLET_PASSWORD")?;
    let rpc_url = env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    // 加载钱包
    let json = fs::read_to_string(&wallet_path)?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, &password)?;
    println!("🤖 Bot started with wallet: {}", keypair.pubkey());

    // 创建客户端
    let client = SolanaClient::new(rpc_url);

    // 主循环
    loop {
        // 检查余额
        let balance = client.get_sol_balance(&keypair.pubkey()).await?;
        println!("Current balance: {} SOL", lamports_to_sol(balance));

        // 你的交易逻辑...
        // if should_trade() {
        //     execute_trade(&client, &keypair).await?;
        // }

        // 等待下一次检查
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
```

### 批量转账示例

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, pubkey::Pubkey, signer::Signer};
use std::{str::FromStr, fs};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载钱包
    let json = fs::read_to_string("wallet.json")?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;

    // 创建客户端
    let client = SolanaClient::new("https://api.mainnet-beta.solana.com".to_string());

    // 接收者列表
    let recipients = vec![
        ("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", 0.1),
        ("7xVdF5G926cUy8EPjHmaT24yvAM3ZWbrrpZd8QvsVGjm", 0.2),
        // ... 更多接收者
    ];

    println!("🚀 Starting batch transfer...");

    for (addr, amount) in recipients {
        let recipient = Pubkey::from_str(addr)?;
        let lamports = (amount * 1_000_000_000.0) as u64;

        println!("\nTransferring {} SOL to {}...", amount, addr);

        match client.transfer_sol(&keypair, &recipient, lamports).await {
            Ok(signature) => {
                println!("✅ Success! Signature: {}", signature);
            }
            Err(e) => {
                eprintln!("❌ Failed: {}", e);
            }
        }

        // 避免速率限制
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("\n🎉 Batch transfer complete!");
    Ok(())
}
```

---

## 常见问题

### 1. 如何选择 RPC 节点？

**公共 RPC（免费，有限制）：**
- Mainnet: `https://api.mainnet-beta.solana.com`
- Devnet: `https://api.devnet.solana.com`

**私有 RPC（付费，性能更好）：**
- [QuickNode](https://www.quicknode.com/)
- [Alchemy](https://www.alchemy.com/)
- [Helius](https://www.helius.dev/)

### 2. lamports 和 SOL 的换算

```
1 SOL = 1,000,000,000 lamports
0.1 SOL = 100,000,000 lamports
0.01 SOL = 10,000,000 lamports
```

转换示例：
```rust
use solana_sdk::native_token::LAMPORTS_PER_SOL;

// SOL 转 lamports
let sol = 1.5;
let lamports = (sol * LAMPORTS_PER_SOL as f64) as u64;

// lamports 转 SOL
let lamports = 1_500_000_000;
let sol = lamports_to_sol(lamports); // 1.5
```

### 3. Token 小数位数

不同 Token 有不同的小数位数：

| Token | 小数位数 | 示例 |
|-------|----------|------|
| USDC | 6 | 1,000,000 = 1 USDC |
| SOL/WSOL | 9 | 1,000,000,000 = 1 SOL |
| 大多数 SPL Token | 9 | - |

查询 Token 信息：
```bash
# 使用 Solana CLI
spl-token display <MINT_ADDRESS>
```

### 4. 交易失败怎么办？

常见原因：
- ❌ 余额不足（包括手续费）
- ❌ RPC 节点超时
- ❌ 网络拥堵
- ❌ Token 账户不存在

解决方案：
```rust
// 检查余额
let balance = client.get_sol_balance(&keypair.pubkey()).await?;
let min_balance = amount + 5_000; // 预留手续费
if balance < min_balance {
    return Err(anyhow!("Insufficient balance"));
}

// 重试机制
let mut retries = 3;
while retries > 0 {
    match client.transfer_sol(&keypair, &to, amount).await {
        Ok(sig) => return Ok(sig),
        Err(e) => {
            retries -= 1;
            if retries == 0 {
                return Err(e);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
```

### 5. 如何在测试环境使用？

```bash
# CLI - 使用 devnet
sol-safekey sol-ops -f wallet.json balance \
  -r https://api.devnet.solana.com

# 代码 - 使用 devnet
let client = SolanaClient::new(
    "https://api.devnet.solana.com".to_string()
);
```

获取测试 SOL：
```bash
solana airdrop 2 <YOUR_ADDRESS> --url devnet
```

### 6. 如何安全地存储密码？

**推荐方式：**

1. **环境变量**（推荐用于开发）
```bash
export WALLET_PASSWORD="your_password"
```

```rust
let password = env::var("WALLET_PASSWORD")?;
```

2. **`.env` 文件**（不要提交到 Git）
```
# .env
WALLET_PASSWORD=your_password
RPC_URL=https://api.mainnet-beta.solana.com
```

```rust
dotenv::dotenv().ok();
let password = env::var("WALLET_PASSWORD")?;
```

3. **密钥管理服务**（推荐用于生产）
- AWS Secrets Manager
- HashiCorp Vault
- Google Secret Manager

---

## 相关文档

- [库集成指南](./INTEGRATION.md)
- [完整 API 文档](https://docs.rs/sol-safekey)
- [CLI 使用指南](../README.md)
- [Solana 官方文档](https://docs.solana.com/)
