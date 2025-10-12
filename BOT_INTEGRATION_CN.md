# Bot 集成指南

将 sol-safekey 集成到你的 Solana 交易机器人或应用程序的完整指南。

[English Documentation](BOT_INTEGRATION.md)

## 为什么选择 Sol-SafeKey？

Sol-SafeKey 提供军事级钱包安全性和简单集成 - 只需 3 行代码即可为你的 bot 添加完整的交互式钱包管理系统。

### 核心优势

- **🔐 军事级安全**: AES-256 加密配合 PBKDF2 密钥派生
- **🚀 简单集成**: 3 行代码实现完整钱包管理
- **🎯 交互式 CLI**: 内置所有钱包操作命令
- **💰 Solana 就绪**: 原生支持 SOL、WSOL、SPL 代币和 durable nonce
- **🔒 默认安全**: 密码通过 stdin 管道（仅内存，永不使用环境变量）

## 集成步骤

### 步骤 1: 添加依赖

在你的 `Cargo.toml` 中添加:

```toml
[dependencies]
sol-safekey = { path = "../sol-safekey" }

[features]
default = ["solana-ops"]
solana-ops = ["sol-safekey/solana-ops"]
```

### 步骤 2: 添加 Safekey 命令

在你的 bot 的 `main()` 函数中，在 bot 逻辑**之前**添加此代码:

```rust
use anyhow::Result;

fn main() -> Result<()> {
    // 检查是否运行在 safekey 交互模式
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("safekey") {
        // 启动 sol-safekey 交互式菜单
        if let Err(e) = sol_safekey::interactive::show_main_menu() {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // 你的 bot 逻辑从这里开始...
    println!("🤖 正在启动 bot...");

    Ok(())
}
```

就是这样！只需 3 行实际集成代码。

### 步骤 3: 构建你的 Bot

```bash
cargo build --features solana-ops --release
```

## 使用 Safekey 命令

集成后，用户可以运行:

```bash
./你的bot safekey
```

这将启动完整的交互式菜单，包含所有钱包操作:

### 可用操作

**钱包管理:**
- 创建明文密钥对
- 创建加密密钥对（推荐）
- 解密加密的密钥对
- 解锁会话钱包

**Solana 操作:**
- 查询 SOL 余额
- 转账 SOL
- 包装 SOL → WSOL
- 解包 WSOL → SOL
- 转账 SPL 代币
- 创建 durable nonce 账户

## 安全实现

### 密码处理

Sol-SafeKey 遵循与 wick-catching-bot 相同的安全模型:

**✅ 安全方式:**
- 密码通过 stdin 管道传递
- 仅存在于内存中
- 永不存储在文件或环境变量中
- 使用后立即清除

**❌ 不安全（永远不要这样做）:**
```bash
# 不要: 环境变量可能被泄露
export WALLET_PASSWORD="mysecret"
./你的bot
```

**✅ 安全（总是这样做）:**
```bash
# 通过 stdin 管道传递密码 - 仅内存
echo "你的密码" | ./你的bot
```

### 启动脚本示例

为你的 bot 创建一个安全的启动脚本:

```bash
#!/bin/bash

# 构建 bot
echo "🔧 正在构建 bot..."
cargo build --features solana-ops --release

# 安全获取密码（不回显）
echo -n "🔐 输入钱包密码: "
read -s WALLET_PASSWORD
echo ""

# 通过 stdin 管道启动 bot
echo "$WALLET_PASSWORD" | ./build-cache/release/你的bot > bot.log 2>&1
EXIT_CODE=$?

# 立即从内存清除密码
WALLET_PASSWORD=""
unset WALLET_PASSWORD

# 检查执行结果
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Bot 执行成功"
else
    echo "❌ Bot 失败，退出码: $EXIT_CODE"
    echo "📝 查看 bot.log 了解详情"
fi
```

## Bot 逻辑集成

### 加载加密钱包

```rust
use sol_safekey::KeyManager;
use std::io::{self, Read};

fn load_wallet() -> Result<solana_sdk::signature::Keypair> {
    let wallet_path = "keystore.json";

    // 读取加密的 keystore
    let json = std::fs::read_to_string(wallet_path)?;

    // 从 stdin 读取密码
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;
    let password = password.trim();

    // 解密并加载密钥对
    let keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;

    Ok(keypair)
}
```

### 创建新钱包

```rust
use sol_safekey::KeyManager;

fn create_wallet(password: &str) -> Result<()> {
    // 生成新密钥对
    let keypair = KeyManager::generate_keypair();

    println!("📍 钱包地址: {}", keypair.pubkey());

    // 加密并保存
    let json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("keystore.json", json)?;

    println!("✅ 加密钱包已保存到 keystore.json");

    Ok(())
}
```

### 使用 Solana 操作

```rust
use sol_safekey::solana_ops::SolanaClient;

fn bot_logic(keypair: &solana_sdk::signature::Keypair) -> Result<()> {
    // 初始化 Solana 客户端
    let client = SolanaClient::new("https://api.devnet.solana.com")?;

    // 查询余额
    let balance = client.get_sol_balance(&keypair.pubkey())?;
    println!("💰 余额: {} SOL", balance);

    // 转账 SOL
    if balance > 0.01 {
        let recipient = "接收地址".parse()?;
        let signature = client.transfer_sol(keypair, &recipient, 0.01)?;
        println!("✅ 转账成功: {}", signature);
    }

    // 将 SOL 包装为 WSOL
    let signature = client.wrap_sol(keypair, 0.1)?;
    println!("✅ 已包装 0.1 SOL: {}", signature);

    Ok(())
}
```

## 完整 Bot 示例

查看 `examples/complete_bot_example.rs` 获取完整工作示例，演示:

- Safekey 命令集成
- 通过 stdin 安全处理密码
- 加密钱包加载
- 所有 Solana 操作
- 适当的错误处理
- 生产就绪模式

构建和运行:

```bash
# 构建示例
cargo build --example complete_bot_example --features solana-ops --release

# 启动交互式 safekey 命令
./build-cache/release/examples/complete_bot_example safekey

# 通过 stdin 运行 bot
echo "你的密码" | ./build-cache/release/examples/complete_bot_example
```

## 与 wick-catching-bot 对比

Sol-SafeKey 使用与 wick-catching-bot **完全相同的集成模式**:

| 功能 | wick-catching-bot | 你的 Bot + sol-safekey |
|------|-------------------|------------------------|
| Safekey 命令 | ✅ `./bot safekey` | ✅ `./你的bot safekey` |
| 交互式菜单 | ✅ 全功能 | ✅ 全功能 |
| 钱包创建 | ✅ AES-256 | ✅ AES-256 |
| 密码安全 | ✅ stdin 管道 | ✅ stdin 管道 |
| SOL 操作 | ✅ 内置 | ✅ 内置 |
| 代币支持 | ✅ SPL 代币 | ✅ SPL 代币 |
| Durable nonce | ✅ 支持 | ✅ 支持 |
| 集成工作量 | N/A | 🎯 3 行代码 |

## 测试你的集成

### 在 Devnet 上

1. 创建测试钱包:
```bash
./你的bot safekey
# 选择: 创建加密密钥对 → 保存到 keystore.json
```

2. 获取 devnet SOL:
```bash
solana airdrop 2 你的钱包地址 --url devnet
```

3. 测试操作:
```bash
./你的bot safekey
# 选择: 解锁钱包 → 查询余额 → 转账 SOL
```

### 集成检查清单

- [ ] 在 Cargo.toml 中添加了 sol-safekey 依赖
- [ ] 在 main() 中添加了 3 行 safekey 命令检查
- [ ] 创建了带 stdin 密码的安全启动脚本
- [ ] 通过 safekey 命令测试了钱包创建
- [ ] 在 bot 逻辑中测试了钱包加载
- [ ] 验证密码永不在环境变量中
- [ ] 在生产环境前在 devnet 上测试
- [ ] 安全备份 keystore.json

## 最佳实践

### 安全性

1. **永远不要**将密码存储在:
   - 环境变量
   - 配置文件
   - 源代码
   - 日志文件

2. **总是**使用:
   - Stdin 管道输入密码
   - 加密的 keystore 文件（AES-256）
   - 强密码（16+ 字符）
   - 安全的备份位置

3. **生产检查清单**:
   - [ ] 在 devnet 上彻底测试
   - [ ] 安全备份 keystore.json
   - [ ] 对高价值账户使用硬件安全模块（HSM）
   - [ ] 实现操作速率限制
   - [ ] 监控异常活动
   - [ ] 保持依赖更新

### 错误处理

```rust
use anyhow::{Context, Result};

fn robust_bot_logic() -> Result<()> {
    // 加载钱包并添加上下文
    let keypair = load_wallet()
        .context("无法从 keystore.json 加载钱包")?;

    // 使用重试逻辑初始化客户端
    let client = SolanaClient::new_with_retry("https://api.mainnet-beta.solana.com")
        .context("无法连接到 Solana 网络")?;

    // 执行操作并处理错误
    match client.get_sol_balance(&keypair.pubkey()) {
        Ok(balance) => println!("余额: {}", balance),
        Err(e) => eprintln!("获取余额失败: {}", e),
    }

    Ok(())
}
```

### 性能提示

1. **连接池**: 重用 SolanaClient 实例
2. **批量操作**: 将多个交易分组
3. **异步处理**: 使用 tokio 进行并发操作
4. **缓存**: 缓存余额查询和账户信息
5. **速率限制**: 遵守 RPC 节点限制

## 故障排除

### 常见问题

**问题**: "无法解密 keystore"
- **原因**: 密码错误
- **解决方案**: 验证密码或创建新钱包

**问题**: "连接被拒绝"
- **原因**: RPC 节点无法访问
- **解决方案**: 检查网络，尝试不同的 RPC 端点

**问题**: "余额不足"
- **原因**: SOL 不足以支付交易 + 手续费
- **解决方案**: 确保余额覆盖金额 + 约 0.00001 SOL 手续费

**问题**: "交易失败"
- **原因**: 网络拥堵或无效交易
- **解决方案**: 使用更高优先级费用重试或检查交易详情

### 获取帮助

- 查看[使用手册](USER_GUIDE_CN.md)了解详细操作说明
- 查看 `examples/complete_bot_example.rs` 获取工作代码
- 检查 `bot.log` 了解错误详情
- 验证你的集成是否匹配 wick-catching-bot 模式

## 下一步

1. ✅ 完成集成（3 行代码）
2. ✅ 在 devnet 上创建测试钱包
3. ✅ 通过 safekey 命令测试所有操作
4. ✅ 实现你的 bot 逻辑
5. ✅ 在 devnet 上彻底测试
6. 🚀 部署到生产环境

---

**记住**: 安全至上。永不妥协密码处理，始终先在 devnet 测试，并保持 keystore 文件的备份。
