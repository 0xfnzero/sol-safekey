# Sol-SafeKey 集成指南

## 快速开始

### 1. 添加依赖

在你的项目 `Cargo.toml` 中添加：

```toml
[dependencies]
sol-safekey = "0.1.0"
```

这会安装最小化的库版本，**不包含 CLI 工具依赖**，体积更小，编译更快。

### 2. 基础用法

```rust
use sol_safekey::KeyManager;

fn main() {
    // 生成新的 Solana 密钥对
    let keypair = KeyManager::generate_keypair();
    println!("公钥: {}", keypair.pubkey());

    // 获取私钥字符串
    let private_key = keypair.to_base58_string();

    // 使用密码加密私钥
    let encrypted = KeyManager::encrypt_with_password(
        &private_key,
        "your_strong_password"
    ).unwrap();
    println!("加密后: {}", encrypted);

    // 解密私钥
    let decrypted = KeyManager::decrypt_with_password(
        &encrypted,
        "your_strong_password"
    ).unwrap();
    println!("解密后: {}", decrypted);

    // 验证
    assert_eq!(private_key, decrypted);
}
```

### 3. 导出/导入 JSON Keystore

```rust
use sol_safekey::KeyManager;

fn main() {
    let keypair = KeyManager::generate_keypair();
    let password = "secure_password";

    // 导出加密的 JSON keystore
    let json = KeyManager::keypair_to_encrypted_json(&keypair, password).unwrap();
    std::fs::write("wallet.json", &json).unwrap();

    // 从 JSON 导入
    let json = std::fs::read_to_string("wallet.json").unwrap();
    let restored_keypair = KeyManager::keypair_from_encrypted_json(&json, password).unwrap();

    assert_eq!(keypair.pubkey(), restored_keypair.pubkey());
}
```

### 4. 获取公钥地址

```rust
use sol_safekey::KeyManager;

fn main() {
    let private_key = "your_private_key_base58";
    let public_key = KeyManager::get_public_key(private_key).unwrap();
    println!("钱包地址: {}", public_key);
}
```

## 可选功能

如果你需要更多功能，可以启用 features：

```toml
[dependencies]
# 启用 2FA 功能（硬件指纹、TOTP等）
sol-safekey = { version = "0.1.0", features = ["2fa"] }

# 启用 Solana 操作功能（转账、查询余额等）
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }

# 启用所有功能（包括 CLI）
sol-safekey = { version = "0.1.0", features = ["full"] }
```

## API 文档

### `KeyManager::generate_keypair() -> Keypair`
生成新的 Solana 密钥对

### `KeyManager::encrypt_with_password(private_key: &str, password: &str) -> Result<String, String>`
使用密码加密私钥，返回 base64 编码的加密数据

### `KeyManager::decrypt_with_password(encrypted_data: &str, password: &str) -> Result<String, String>`
使用密码解密私钥，返回原始 base58 私钥

### `KeyManager::get_public_key(private_key: &str) -> Result<String, String>`
从私钥获取公钥地址

### `KeyManager::keypair_to_encrypted_json(keypair: &Keypair, password: &str) -> Result<String, String>`
将密钥对导出为加密的 JSON 格式

### `KeyManager::keypair_from_encrypted_json(json_data: &str, password: &str) -> Result<Keypair, String>`
从加密的 JSON 恢复密钥对

## CLI 工具 vs 库集成

### CLI 工具
适用于：
- 开发者个人使用
- 命令行脚本
- 快速测试

安装方式：
```bash
cargo install sol-safekey --features full
```

### 库集成
适用于：
- 集成到你的 Rust 项目
- Web 服务
- 自动化程序

集成方式：
```toml
[dependencies]
sol-safekey = "0.1.0"  # 不需要安装 CLI
```

## 完整示例

```rust
use sol_safekey::KeyManager;
use solana_sdk::signer::Signer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 场景1: 创建新钱包并加密保存
    println!("=== 创建新钱包 ===");
    let keypair = KeyManager::generate_keypair();
    let password = "MySecurePassword123!";

    let json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("my_wallet.json", &json)?;

    println!("✅ 钱包已创建");
    println!("📍 地址: {}", keypair.pubkey());

    // 场景2: 从文件加载钱包
    println!("\n=== 加载钱包 ===");
    let json = std::fs::read_to_string("my_wallet.json")?;
    let loaded_keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;

    println!("✅ 钱包已加载");
    println!("📍 地址: {}", loaded_keypair.pubkey());

    // 场景3: 仅加密/解密私钥字符串
    println!("\n=== 加密私钥 ===");
    let private_key = keypair.to_base58_string();
    let encrypted = KeyManager::encrypt_with_password(&private_key, password)?;
    println!("🔒 加密后: {}...", &encrypted[..50]);

    let decrypted = KeyManager::decrypt_with_password(&encrypted, password)?;
    println!("🔓 解密后: {}...", &decrypted[..50]);

    assert_eq!(private_key, decrypted);
    println!("✅ 验证成功");

    Ok(())
}
```

## 安全建议

1. **密码管理**：使用强密码（至少 8 位，包含大小写、数字、特殊字符）
2. **私钥保护**：永远不要将私钥明文存储在代码中
3. **环境变量**：可以使用环境变量传递密码，但不要提交到 Git
4. **Keystore 备份**：定期备份加密的 keystore 文件到安全位置

## 更多信息

- [GitHub 仓库](https://github.com/your-repo/sol-safekey)
- [完整 CLI 文档](../README.md)
- [API 文档](https://docs.rs/sol-safekey)
