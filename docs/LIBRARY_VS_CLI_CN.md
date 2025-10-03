# Sol-SafeKey: 库集成 vs CLI 工具

## 快速对比

| 特性 | 库集成 | CLI 工具 |
|------|--------|----------|
| **使用场景** | 集成到 Rust 项目 | 命令行使用 |
| **安装方式** | `Cargo.toml` 依赖 | `cargo install` |
| **依赖大小** | 最小化（仅核心库） | 完整（包含 CLI 依赖） |
| **编译速度** | 更快 | 较慢 |
| **适合人群** | 开发者集成 | 个人使用 |
| **2FA 功能** | 可选 (`features = ["2fa"]`) | 完整支持 |
| **Solana 操作** | 可选 (`features = ["solana-ops"]`) | 完整支持 |

## 库集成（推荐用于项目集成）

### 安装

```toml
[dependencies]
sol-safekey = "0.1.0"  # 最小化安装
```

### 使用

```rust
use sol_safekey::KeyManager;

fn main() {
    let keypair = KeyManager::generate_keypair();
    let encrypted = KeyManager::encrypt_with_password(
        &keypair.to_base58_string(),
        "password"
    ).unwrap();
}
```

### 优势

✅ **无需安装 CLI** - 仅添加库依赖即可
✅ **体积小** - 不包含 CLI 工具的依赖（如 `clap`, `colored`, `qrcode` 等）
✅ **编译快** - 更少的依赖意味着更快的编译速度
✅ **灵活** - 可以按需启用功能（`2fa`, `solana-ops`）
✅ **易集成** - 简洁的 API，3 行代码完成加密/解密

### 适用场景

- 🤖 交易机器人
- 🌐 Web 服务
- 📱 钱包应用
- 🔧 自动化脚本
- 📦 其他 Rust 项目

## CLI 工具（推荐用于个人使用）

### 安装

```bash
cargo install sol-safekey --features full
```

### 使用

```bash
# 生成加密钱包
sol-safekey gen-keystore -o wallet.json -p mypassword

# 解锁钱包
sol-safekey unlock -f wallet.json -p mypassword
```

### 优势

✅ **开箱即用** - 无需编写代码
✅ **交互式** - 友好的命令行界面
✅ **功能完整** - 支持所有功能（2FA、Solana 操作等）
✅ **可视化** - 彩色输出、QR 码显示

### 适用场景

- 👨‍💻 开发者个人使用
- 🔑 快速生成测试密钥
- 💼 管理个人钱包
- 🧪 功能测试

## 集成示例对比

### 场景：生成并加密钱包

#### 使用库集成

```rust
use sol_safekey::KeyManager;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成密钥对
    let keypair = KeyManager::generate_keypair();

    // 导出为加密 JSON
    let json = KeyManager::keypair_to_encrypted_json(&keypair, "password")?;

    // 保存到文件
    fs::write("wallet.json", json)?;

    println!("Wallet created: {}", keypair.pubkey());
    Ok(())
}
```

**优势：**
- 可以集成到你的程序流程中
- 可以自定义密码来源（环境变量、配置文件等）
- 可以添加自己的错误处理逻辑

#### 使用 CLI 工具

```bash
sol-safekey gen-keystore -o wallet.json -p password
```

**优势：**
- 一行命令完成
- 无需编写代码
- 即时可用

## 依赖对比

### 库集成（默认）

仅包含核心依赖：
- `base64` - Base64 编解码
- `ring` - 加密算法
- `solana-sdk` - Solana SDK
- `serde` / `serde_json` - JSON 序列化
- `chrono` - 时间处理
- `data-encoding` - 数据编码
- `anyhow` - 错误处理

**总依赖数：** ~100 个 crate

### CLI 工具（features = "full"）

包含所有依赖：
- 核心库依赖
- `clap` - 命令行解析
- `colored` - 彩色输出
- `rpassword` - 密码输入
- `qrcode` - QR 码生成
- `totp-rs` - 2FA 支持
- `solana-client` - Solana 客户端
- `tokio` - 异步运行时

**总依赖数：** ~200+ 个 crate

## 选择建议

### 选择库集成，如果你：

- ✅ 需要集成到自己的 Rust 项目
- ✅ 想要控制加密/解密流程
- ✅ 需要自定义用户界面
- ✅ 想要最小化依赖
- ✅ 构建 Web 服务或机器人

### 选择 CLI 工具，如果你：

- ✅ 需要快速生成/管理密钥
- ✅ 不想编写代码
- ✅ 需要交互式操作
- ✅ 使用 2FA 功能
- ✅ 个人使用或测试

## 混合使用

你也可以同时使用两种方式：

1. **开发阶段** - 使用 CLI 工具快速测试和生成密钥
2. **生产环境** - 使用库集成到你的项目中

示例：

```toml
[dependencies]
# 生产环境：仅使用库
sol-safekey = "0.1.0"

[dev-dependencies]
# 开发环境：完整功能（用于测试）
sol-safekey = { version = "0.1.0", features = ["full"] }
```

## 完整文档

- 📖 [库集成指南](./INTEGRATION.md)
- 📖 [CLI 使用指南](../README.md)
- 📖 [API 文档](https://docs.rs/sol-safekey)
