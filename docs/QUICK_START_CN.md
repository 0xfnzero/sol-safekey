# Sol-SafeKey 快速开始

## 5 分钟快速上手

### 方式 1: 作为库使用（推荐用于项目集成）

**1. 添加依赖**

```toml
[dependencies]
sol-safekey = "0.1.0"
```

**2. 生成并加密钱包**

```rust
use sol_safekey::KeyManager;

fn main() {
    // 生成密钥对
    let keypair = KeyManager::generate_keypair();

    // 加密并保存
    let json = KeyManager::keypair_to_encrypted_json(&keypair, "password").unwrap();
    std::fs::write("wallet.json", json).unwrap();

    println!("✅ Wallet created: {}", keypair.pubkey());
}
```

**3. 解密并使用**

```rust
use sol_safekey::KeyManager;
use std::fs;

fn main() {
    // 读取并解密
    let json = fs::read_to_string("wallet.json").unwrap();
    let keypair = KeyManager::keypair_from_encrypted_json(&json, "password").unwrap();

    println!("✅ Wallet loaded: {}", keypair.pubkey());
}
```

### 方式 2: 作为 CLI 工具使用

**1. 安装**

```bash
cargo install sol-safekey --features full
```

**2. 生成钱包**

```bash
sol-safekey gen-keystore -o wallet.json
# 提示输入密码
```

**3. 解锁钱包**

```bash
sol-safekey unlock -f wallet.json
# 提示输入密码
```

### 方式 3: Solana 操作（查询余额、转账等）

**CLI 方式**

```bash
# 查询余额
sol-safekey sol-ops -f wallet.json balance

# 转账
sol-safekey sol-ops -f wallet.json transfer -t <地址> -a 0.1
```

**代码方式**

```toml
[dependencies]
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }
tokio = { version = "1.0", features = ["full"] }
```

```rust
use sol_safekey::{KeyManager, solana_utils::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载钱包
    let json = std::fs::read_to_string("wallet.json")?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;

    // 创建客户端
    let client = SolanaClient::new("https://api.mainnet-beta.solana.com".to_string());

    // 查询余额
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    println!("Balance: {} SOL", lamports_to_sol(balance));

    Ok(())
}
```

## 完整文档

- 📖 [库集成指南](./INTEGRATION.md) - 完整的 API 集成文档
- 📖 [Solana 操作指南](./SOLANA_OPS.md) - 查询、转账、Token 操作
- 📖 [CLI vs 库对比](./LIBRARY_VS_CLI.md) - 选择合适的使用方式
- 📖 [完整 README](../README.md) - 详细功能说明

## 常用命令速查

### CLI 命令

| 命令 | 说明 |
|------|------|
| `gen-keystore -o <文件>` | 生成加密钱包 |
| `unlock -f <文件>` | 解锁钱包查看私钥 |
| `address -f <文件> -p <密码>` | 查看钱包地址 |
| `sol-ops -f <文件> balance` | 查询 SOL 余额 |
| `sol-ops -f <文件> transfer -t <地址> -a <金额>` | 转账 SOL |

### 库 API

| API | 说明 |
|-----|------|
| `KeyManager::generate_keypair()` | 生成密钥对 |
| `KeyManager::encrypt_with_password(key, pwd)` | 加密私钥 |
| `KeyManager::decrypt_with_password(enc, pwd)` | 解密私钥 |
| `KeyManager::keypair_to_encrypted_json(kp, pwd)` | 导出加密 JSON |
| `KeyManager::keypair_from_encrypted_json(json, pwd)` | 从 JSON 导入 |

## 快速示例

### 完整的机器人示例

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::signer::Signer;
use std::{fs, env};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 配置
    let wallet_path = env::var("WALLET_PATH").unwrap_or("wallet.json".to_string());
    let password = env::var("WALLET_PASSWORD")?;
    let rpc_url = "https://api.mainnet-beta.solana.com";

    // 2. 加载钱包
    let json = fs::read_to_string(&wallet_path)?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, &password)?;
    println!("🤖 Bot started: {}", keypair.pubkey());

    // 3. 创建客户端
    let client = SolanaClient::new(rpc_url.to_string());

    // 4. 主循环
    loop {
        // 查询余额
        let balance = client.get_sol_balance(&keypair.pubkey()).await?;
        println!("Balance: {} SOL", lamports_to_sol(balance));

        // 你的交易逻辑...

        // 等待 60 秒
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
```

### 环境变量配置

创建 `.env` 文件：

```env
WALLET_PATH=wallet.json
WALLET_PASSWORD=your_password_here
RPC_URL=https://api.mainnet-beta.solana.com
```

使用方式：

```toml
[dependencies]
dotenv = "0.15"
```

```rust
fn main() {
    dotenv::dotenv().ok();
    let password = std::env::var("WALLET_PASSWORD").unwrap();
    // ...
}
```

## 安全提醒

⚠️ **重要：**
- 永远不要将密码硬编码在代码中
- 永远不要将 `.env` 文件提交到 Git
- 使用强密码（至少 8 位，包含大小写、数字、特殊字符）
- 定期备份加密的钱包文件
- 在生产环境使用密钥管理服务（AWS Secrets Manager, HashiCorp Vault 等）

## 获取帮助

- 💬 [Telegram 群组](https://t.me/fnzero_group)
- 💬 [Discord](https://discord.gg/ckf5UHxz)
- 🐛 [报告问题](https://github.com/0xfnzero/sol-safekey/issues)
- 📖 [完整文档](https://docs.rs/sol-safekey)
