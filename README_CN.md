<div align="center">
    <h1>🔐 Sol-SafeKey</h1>
    <h3><em>Solana 私钥管理工具 - 简单、安全、专业</em></h3>
</div>

<p align="center">
    <strong>交互式多语言菜单 | 密码加密 | 三因子 2FA | Solana 操作 | Bot 集成</strong>
</p>

<p align="center">
    <a href="https://crates.io/crates/sol-safekey">
        <img src="https://img.shields.io/crates/v/sol-safekey.svg" alt="Crates.io">
    </a>
    <a href="https://docs.rs/sol-safekey">
        <img src="https://docs.rs/sol-safekey/badge.svg" alt="Documentation">
    </a>
    <a href="./LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
    <img src="https://img.shields.io/badge/Solana-9945FF?style=for-the-badge&logo=solana&logoColor=white" alt="Solana">
    <img src="https://img.shields.io/badge/2FA-4CAF50?style=for-the-badge&logo=google-authenticator&logoColor=white" alt="2FA">
</p>

<p align="center">
    <a href="./README.md">English</a> |
    <a href="./README_CN.md">中文</a>
</p>

---

> ## ⚠️ 安全提醒
>
> **开源加密工具，存在已知限制。** 加密算法是公开可见的。
>
> - ✅ **推荐用于**: 开发、测试、机器人、使用 2FA 的中等钱包（$1k-$10k）
> - ❌ **不推荐用于**: 大额资金（>$10k）- 请使用硬件钱包
> - 🔐 **必须使用**: 20+ 字符密码 + 2FA 用于重要钱包
> - 📖 **请阅读**: 使用前请阅读下方安全部分

---

## 📋 目录

1. [快速开始](#-快速开始)
2. [模块 1: 简单加密/解密](#-模块-1-简单加密解密)
3. [模块 2: 三因子 2FA 加密](#️-模块-2-三因子-2fa-加密)
4. [模块 3: Solana 钱包操作](#-模块-3-solana-钱包操作)
5. [模块 4: Bot 集成](#-模块-4-bot-集成)
6. [安全与最佳实践](#-安全与最佳实践)

---

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey

# 构建全功能版本
cargo build --release --features full

# 运行交互式菜单
./target/release/sol-safekey start
```

### 交互式菜单

```
==================================================
  Sol-SafeKey - Solana 密钥管理工具
==================================================

核心功能:
  1.  创建明文私钥
  2.  创建加密私钥
  3.  解密私钥

高级安全:
  4.  设置 2FA 认证
  5.  生成三因子钱包
  6.  解锁三因子钱包

Solana 操作:
  7.  查询 SOL 余额
  8.  转账 SOL
  9.  包装 SOL → WSOL
  10. 解包 WSOL → SOL
  11. 转账 SPL 代币
  12. 创建 Nonce 账户

  0.  退出
```

---

## 🔑 模块 1: 简单加密/解密

**适用于**: 快速设置、开发、测试、个人钱包

### 1.1 创建加密密钥库

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 2: 创建加密私钥
```

**CLI 模式:**

```bash
# CLI 模式不可用 - 请使用交互式模式
```

**流程:**

1. 选择方式:
   - 生成新密钥对
   - 导入现有私钥

2. 设置密码（10+ 字符，3 种类型）:
   - 大写、小写、数字、特殊字符

3. 保存为密钥库文件（JSON 格式）

**输出文件 (`keystore.json`):**

```json
{
  "encrypted_private_key": "base64_加密数据",
  "public_key": "7nWq3...",
  "created_at": "2024-01-01T00:00:00Z"
}
```

### 1.2 解密密钥库

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 3: 解密私钥
```

**流程:**

1. 选择输入方式:
   - 从密钥库文件
   - 输入加密字符串

2. 输入密码

3. 查看解密后的私钥

### 1.3 在代码中使用

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载加密密钥库
    let keypair = KeyManager::keypair_from_keystore_file(
        "keystore.json",
        "your-password"
    )?;

    println!("钱包: {}", keypair.pubkey());
    Ok(())
}
```

### 1.4 安全级别

- **加密**: AES-256 + PBKDF2
- **安全性**: ⭐⭐⭐ (中等)
- **便携性**: ✅ 可在任何设备上使用
- **最适合**: 开发、测试、小额钱包（<$1k）

---

## 🛡️ 模块 2: 三因子 2FA 加密

**适用于**: 生产 bot、中等价值钱包、最大安全性

### 2.1 什么是三因子？

```
┌─────────────────────────────────────────┐
│  因子 1: 硬件指纹                        │
│  - CPU ID、MAC 地址、主机名             │
│  - 设备绑定（不可移植）                 │
├─────────────────────────────────────────┤
│  因子 2: 主密码                          │
│  - 强密码（10+ 字符）                   │
│  - 永不存储，仅哈希                     │
├─────────────────────────────────────────┤
│  因子 3: 安全问题                        │
│  - 预定义问题                           │
│  - 额外保护层                           │
├─────────────────────────────────────────┤
│  因子 4: TOTP 2FA 代码                   │
│  - 6 位数字代码（30 秒刷新）            │
│  - 兼容 Google Authenticator            │
└─────────────────────────────────────────┘
```

### 2.2 设置 2FA（一次性）

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 4: 设置 2FA 认证
```

**CLI 模式:**

```bash
./sol-safekey setup-2fa
```

**流程:**

```
步骤 1/4: 收集硬件指纹
✅ 指纹已收集: a3f7b2...

步骤 2/4: 设置主密码
输入密码: ************
确认: ************
✅ 密码已设置

步骤 3/4: 安全问题
从列表中选择问题
输入答案
✅ 问题已设置

步骤 4/4: 设置 2FA
📱 使用 Google Authenticator 扫描二维码
输入 6 位数字代码: 123456
✅ 2FA 已验证！

配置已保存到: ~/.sol-safekey/2fa-config.json
备份已保存到: ~/.sol-safekey/backup/2fa-backup-<timestamp>.json
```

**重要文件:**

- `~/.sol-safekey/2fa-config.json` - 2FA 配置（设备绑定）
- `~/.sol-safekey/backup/2fa-backup-*.json` - 备份文件（离线安全存储）

### 2.3 生成三因子钱包

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 5: 生成三因子钱包
```

**CLI 模式:**

```bash
./sol-safekey gen-2fa-wallet -o wallet-2fa.json
```

**流程:**

```
输入主密码: ************
回答安全问题: ********
输入 2FA 代码: 123456

✅ 三因子钱包已创建！
   公钥: 7nWq3JkPQVx...
   已保存到: wallet-2fa.json
```

### 2.4 解锁三因子钱包

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 6: 解锁三因子钱包
```

**CLI 模式:**

```bash
./sol-safekey unlock-2fa-wallet -f wallet-2fa.json
```

**流程:**

```
选择密钥库: wallet-2fa.json
输入主密码: ************
回答安全问题: ********
输入 2FA 代码: 123456

✅ 钱包已解锁！
   私钥: 5Kk8h... (base58)
```

### 2.5 在代码中使用

```rust
use sol_safekey::{TwoFAConfig, KeyManager};
use solana_sdk::signature::Keypair;

fn unlock_2fa_wallet() -> Result<Keypair, Box<dyn std::error::Error>> {
    // 加载 2FA 配置
    let config = TwoFAConfig::load()?;

    // 从环境变量获取凭证
    let master_password = std::env::var("MASTER_PASSWORD")?;
    let security_answer = std::env::var("SECURITY_ANSWER")?;
    let totp_code = std::env::var("TOTP_CODE")?;

    // 解锁钱包
    let keypair = KeyManager::unlock_2fa_wallet(
        "wallet-2fa.json",
        &config,
        &master_password,
        &security_answer,
        &totp_code
    )?;

    Ok(keypair)
}
```

### 2.6 安全级别

- **加密**: AES-256 + 硬件绑定 + TOTP
- **安全性**: ⭐⭐⭐⭐⭐ (最高)
- **便携性**: ❌ 设备绑定（有备份文件可在其他设备恢复）
- **最适合**: 生产 bot、中等价值钱包（$1k-$10k）

### 2.7 设备迁移

如果需要在新设备上使用 2FA 钱包:

```bash
# 1. 复制备份文件到新设备
cp ~/.sol-safekey/backup/2fa-backup-*.json /path/to/new/device/

# 2. 在新设备上恢复
./sol-safekey restore-2fa-backup --file 2fa-backup-*.json

# 3. 重新扫描二维码到 Google Authenticator
```

---

## 💼 模块 3: Solana 钱包操作

**适用于**: 链上操作、余额查询、代币转账

所有操作都支持简单密钥库和 2FA 钱包。

### 3.1 查询 SOL 余额

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 7: 查询 SOL 余额
```

**CLI 模式:**

```bash
./sol-safekey sol-ops balance \
  --keystore wallet.json \
  --password "your-password"
```

**输出:**

```
钱包地址: 7nWq3JkPQVx...
SOL 余额: 1.23456789 SOL
```

### 3.2 转账 SOL

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 8: 转账 SOL
```

**CLI 模式:**

```bash
./sol-safekey sol-ops transfer \
  --keystore wallet.json \
  --password "your-password" \
  --recipient "接收地址" \
  --amount 0.5
```

**流程:**

```
输入接收地址: 7nWq3JkPQVx...
输入金额: 0.5

确认转账:
  从: 你的地址
  到: 接收地址
  金额: 0.5 SOL

继续？(y/n): y

✅ 转账成功！
   交易签名: 2ZE7xK...
```

### 3.3 包装 SOL → WSOL

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 9: 包装 SOL → WSOL
```

**CLI 模式:**

```bash
./sol-safekey sol-ops wrap \
  --keystore wallet.json \
  --password "your-password" \
  --amount 1.0
```

**说明:**

WSOL 是 SOL 的 SPL Token 包装版本，用于与某些 DeFi 协议交互。

### 3.4 解包 WSOL → SOL

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 10: 解包 WSOL → SOL
```

**CLI 模式:**

```bash
./sol-safekey sol-ops unwrap \
  --keystore wallet.json \
  --password "your-password"
```

**说明:**

关闭 WSOL 账户并将所有 WSOL 转换回 SOL。

### 3.5 转账 SPL 代币

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 11: 转账 SPL 代币
```

**CLI 模式:**

```bash
./sol-safekey sol-ops transfer-token \
  --keystore wallet.json \
  --password "your-password" \
  --mint "代币地址" \
  --recipient "接收地址" \
  --amount 100.0
```

**流程:**

```
输入代币 Mint 地址: EPjFWdd5...
输入接收地址: 7nWq3JkPQVx...
输入金额: 100.0

✅ 代币转账成功！
   交易签名: 3XK9pL...
```

### 3.6 创建 Nonce 账户

**交互式模式:**

```bash
./sol-safekey start
# 选择选项 12: 创建 Nonce 账户
```

**CLI 模式:**

```bash
./sol-safekey sol-ops create-nonce \
  --keystore wallet.json \
  --password "your-password"
```

**说明:**

Nonce 账户用于创建可离线签名的持久交易。

---

## 🤖 模块 4: Bot 集成

**适用于**: 交易机器人、自动化脚本、后台服务

### 4.1 添加依赖

**Cargo.toml:**

```toml
[dependencies]
sol-safekey = { path = "../sol-safekey", features = ["full"] }
solana-sdk = "1.18"
solana-client = "1.18"
```

### 4.2 基础集成（简单密钥库）

```rust
use sol_safekey::KeyManager;
use solana_sdk::signature::Keypair;

fn load_wallet() -> Result<Keypair, Box<dyn std::error::Error>> {
    // 从环境变量获取密码
    let password = std::env::var("WALLET_PASSWORD")
        .expect("WALLET_PASSWORD 未设置");

    // 加载密钥库
    let keypair = KeyManager::keypair_from_keystore_file(
        "config/wallet.json",
        &password
    )?;

    println!("钱包已加载: {}", keypair.pubkey());
    Ok(keypair)
}

fn main() {
    let wallet = load_wallet().unwrap();

    // 你的 bot 逻辑...
}
```

### 4.3 高级集成（2FA 钱包）

```rust
use sol_safekey::{TwoFAConfig, KeyManager, TOTPManager};
use solana_sdk::signature::Keypair;

fn load_2fa_wallet() -> Result<Keypair, Box<dyn std::error::Error>> {
    // 加载 2FA 配置
    let config = TwoFAConfig::load()?;

    // 从环境变量获取凭证
    let master_password = std::env::var("MASTER_PASSWORD")?;
    let security_answer = std::env::var("SECURITY_ANSWER")?;

    // 生成当前 TOTP 代码
    let totp_manager = TOTPManager::from_config(&config)?;
    let totp_code = totp_manager.generate_current_code()?;

    // 解锁钱包
    let keypair = KeyManager::unlock_2fa_wallet(
        "config/wallet-2fa.json",
        &config,
        &master_password,
        &security_answer,
        &totp_code
    )?;

    println!("2FA 钱包已加载: {}", keypair.pubkey());
    Ok(keypair)
}
```

### 4.4 完整交易 Bot 示例

```rust
use sol_safekey::KeyManager;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    pubkey::Pubkey,
};
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载钱包
    let password = std::env::var("WALLET_PASSWORD")?;
    let wallet = KeyManager::keypair_from_keystore_file(
        "config/wallet.json",
        &password
    )?;

    // 2. 连接到 Solana
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    // 3. 检查余额
    let balance = client.get_balance(&wallet.pubkey())?;
    println!("当前余额: {} SOL", balance as f64 / 1e9);

    // 4. 执行交易
    let recipient = Pubkey::from_str("目标地址")?;
    let amount_lamports = 1_000_000; // 0.001 SOL

    let recent_blockhash = client.get_latest_blockhash()?;
    let instruction = system_instruction::transfer(
        &wallet.pubkey(),
        &recipient,
        amount_lamports
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash
    );

    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("✅ 交易成功: {}", signature);

    Ok(())
}
```

### 4.5 环境变量配置

**.env 文件:**

```bash
# 简单密钥库
WALLET_PASSWORD=your-secure-password

# 2FA 钱包
MASTER_PASSWORD=your-master-password
SECURITY_ANSWER=your-security-answer

# RPC 配置
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
```

**加载 .env:**

```rust
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    // ... 你的代码
}
```

### 4.6 生产部署检查清单

- [ ] 使用 2FA 钱包用于生产环境
- [ ] 在 `.gitignore` 中排除密钥库文件
- [ ] 在环境变量中存储密码，永不硬编码
- [ ] 备份 2FA 配置文件到安全位置
- [ ] 使用强密码（20+ 字符）
- [ ] 在主网部署前在 devnet 测试
- [ ] 实现适当的错误处理和日志记录
- [ ] 定期轮换密码和安全问题

---

## 🔒 安全与最佳实践

### 密码强度要求

**最低要求:**
- 10+ 字符
- 大写字母
- 小写字母
- 数字

**推荐用于生产:**
- 20+ 字符
- 大写 + 小写 + 数字 + 特殊字符
- 避免常见单词或模式
- 使用密码管理器生成

**示例强密码:**
```
弱: MyPassword123
中: MyP@ssw0rd!2024
强: Kx9$mP2#vL8@nQ5!zR3&wT7^
```

### Git 安全

**永远不要提交:**

```bash
# .gitignore
*.json
.sol-safekey/
.env
config/wallet*.json
keystore*.json
*-2fa.json
```

**检查提交历史:**

```bash
# 检查是否意外提交了密钥
git log --all -- '*.json'

# 如果发现，使用 git-filter-branch 或 BFG 清除历史
```

### 环境变量最佳实践

**在脚本中:**

```rust
// ❌ 不要这样做
let password = "my-password-123";

// ✅ 这样做
let password = std::env::var("WALLET_PASSWORD")
    .expect("WALLET_PASSWORD 必须设置");
```

**在 shell 中:**

```bash
# ✅ 从安全存储读取
export WALLET_PASSWORD=$(security find-generic-password -s wallet -w)

# ✅ 从加密文件读取
export WALLET_PASSWORD=$(gpg -d password.gpg)

# ❌ 不要在 shell 历史中硬编码
export WALLET_PASSWORD="my-password"
```

### 备份策略

**对于 2FA 配置:**

1. **主备份**:
   ```bash
   cp ~/.sol-safekey/backup/2fa-backup-*.json /secure/location/
   ```

2. **离线备份**:
   - 打印二维码
   - 存储在防火保险箱中
   - 保存备份短语

3. **测试恢复**:
   ```bash
   # 定期测试备份恢复流程
   ./sol-safekey restore-2fa-backup --file backup.json
   ```

**对于密钥库:**

```bash
# 加密备份
gpg -c wallet.json -o wallet.json.gpg

# 多位置存储
cp wallet.json.gpg /backup1/
cp wallet.json.gpg /backup2/
```

### 2FA 设备丢失恢复

如果丢失 2FA 设备（手机）:

```bash
# 1. 使用备份文件恢复
./sol-safekey restore-2fa-backup --file ~/.sol-safekey/backup/2fa-backup-*.json

# 2. 重新扫描二维码到新设备

# 3. 验证可以生成正确的代码
./sol-safekey verify-2fa
```

### 生产环境安全检查清单

**部署前:**

- [ ] 所有密钥库都已加密
- [ ] 使用 2FA 用于关键钱包
- [ ] 密码强度 ≥ 20 字符
- [ ] 备份存储在 3 个独立位置
- [ ] `.gitignore` 正确配置
- [ ] 环境变量已设置
- [ ] 已在 devnet 测试

**运行时:**

- [ ] 使用专用服务器/容器
- [ ] 限制文件系统权限（600 用于密钥库）
- [ ] 启用日志记录但不记录密码
- [ ] 定期轮换凭证
- [ ] 监控异常活动

**审计:**

- [ ] 每月审查访问日志
- [ ] 每季度更新密码
- [ ] 每年测试灾难恢复
- [ ] 保持软件更新

---

## 📞 支持

- 🐛 **问题反馈**: [GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- 💬 **Telegram**: [加入我们的群组](https://t.me/fnzero_group)
- 🎮 **Discord**: [加入我们的服务器](https://discord.gg/ckf5UHxz)
- 🌐 **网站**: [fnzero.dev](https://fnzero.dev/)

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## ⚖️ 免责声明

本软件按"原样"提供，不提供任何形式的保证。作者不对使用本软件导致的任何损失或损害负责。请自行承担使用风险。

**重要提醒:**
- 始终在主网使用前在 devnet 测试
- 从小额开始测试
- 保持私钥安全，永不分享
- 定期备份所有配置

---

<div align="center">
    <p><strong>使用 ❤️ 由 fnzero 团队制作</strong></p>
    <p>
        <a href="https://github.com/0xfnzero/sol-safekey">⭐ 在 GitHub 上给我们星标</a>
    </p>
</div>
