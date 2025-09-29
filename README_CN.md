<div align="center">
    <h1>🔧 Sol SafeKey</h1>
    <h3><em>功能强大的 Solana 安全密钥管理命令行工具 - 支持三因子 2FA</em></h3>
</div>

<p align="center">
    <strong>使用军用级三因子认证安全地生成、管理和加密 Solana 私钥，结合硬件指纹、主密码、安全问题和 2FA 验证。</strong>
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

## 🎯 最新功能：三因子 2FA 安全机制

**最安全的钱包保护方案！** Sol SafeKey 现已推出革命性的**三因子认证系统**，结合：

- 🖥️ **因子 1**: 硬件指纹（绑定到您的设备）
- 🔐 **因子 2**: 主密码（强密码且有复杂度要求）
- 🛡️ **因子 3**: 安全问题答案（额外的知识因子）
- 📱 **2FA 验证**: 基于时间的一次性密码（Google Authenticator/Authy）

这意味着您的钱包需要**全部四个组件**才能解锁 - 即使攻击者窃取了您的加密钱包文件，也几乎不可能访问您的资金！

### 🚀 三因子 2FA 快速上手

```bash
# 步骤 1: 设置 2FA（一次性设置）
sol-safekey setup-2fa

# 步骤 2: 生成您的安全钱包
sol-safekey gen-2fa-wallet -o my-secure-wallet.json

# 步骤 3: 需要时解锁您的钱包
sol-safekey unlock-2fa-wallet -f my-secure-wallet.json
```

**生成过程中会发生什么：**
1. 生成新的 Solana 密钥对
2. 创建三因子加密钱包（设备绑定）
3. 自动创建 keystore 备份（仅需密码即可在任何设备使用）

## ✨ 特性

### 🔑 密钥生成
- **多种格式**: Keypair 格式、字符串格式和加密格式
- **三因子 2FA 钱包**: 最安全的钱包保护方案
- **分段显示**: 将长私钥分段显示，便于记录
- **自定义输出**: 支持指定自定义输出文件路径
- **自动备份**: 自动生成 keystore 备份用于跨设备恢复

### 🔐 三因子安全特性
- **硬件指纹绑定**: 钱包绑定到您的特定设备
  - CPU 信息、系统序列号、MAC 地址、磁盘序列号
  - SHA256 哈希处理以确保一致性识别
- **强密码要求**: 8+ 字符，需包含以下至少 3 种：大写、小写、数字、特殊字符
- **安全问题**: 8 个预定义问题提供额外保护
- **TOTP 2FA**: RFC 6238 标准（兼容 Google Authenticator、Authy 等）
- **确定性密钥派生**: 2FA 密钥从硬件指纹 + 主密码使用 PBKDF2 派生（100,000 次迭代）
- **三因子加密**: 私钥使用所有三个因子组合加密

### 🌐 多语言支持
- **双语界面**: 完整的中英文对照帮助信息
- **双语命令**: 所有命令描述支持中英文

## 🚀 快速开始

### 安装

```bash
# 克隆仓库
git clone https://github.com/0xfnzero/sol-safekey.git
cd sol-safekey

# 编译
cargo build --release

# 或使用构建脚本（macOS/Linux）
./build.sh

# 安装到系统（可选）
cargo install --path .
```

### 基本使用（简单模式）

```bash
# 查看帮助
sol-safekey --help

# 生成 keypair 格式
sol-safekey gen-keypair -o my-wallet.json

# 生成字符串格式，分 3 段显示
sol-safekey gen-key -s 3 -o my-keys.json

# 生成加密 keystore 文件
sol-safekey gen-keystore -p mypassword -o secure-keys.json

# 解锁加密文件
sol-safekey unlock -f secure-keys.json -p mypassword
```

### 高级使用（三因子 2FA 模式）

```bash
# 1. 首次设置：配置您的 2FA
sol-safekey setup-2fa

# 这将：
# - 收集您设备的硬件指纹
# - 引导您设置强主密码（8+ 字符，3 种类型）
# - 让您选择并回答安全问题
# - 生成 2FA 密钥并显示二维码
# - 使用您的认证器应用验证设置

# 2. 生成您的安全钱包
sol-safekey gen-2fa-wallet -o my-wallet.json

# 这会创建两个文件：
# - my-wallet.json: 三因子加密（需要设备 + 密码 + 安全问题 + 2FA）
# - XXXXXXXX_keystore.json: 仅密码备份（可在任何设备使用）

# 3. 解锁您的钱包
sol-safekey unlock-2fa-wallet -f my-wallet.json

# 您需要提供：
# - 主密码
# - 安全问题答案
# - 来自认证器应用的当前 2FA 验证码
```

## 📋 命令参考

### 🔐 三因子 2FA 命令（推荐）

#### `setup-2fa`
一次性设置三因子认证
```bash
sol-safekey setup-2fa
```

**过程：**
1. 收集硬件指纹（自动）
2. 设置主密码（需要 8+ 字符，3 种字符类型）
3. 选择安全问题和答案
4. 生成 2FA 密钥（从指纹 + 密码确定性派生）
5. 显示二维码用于 Google Authenticator/Authy
6. 使用测试验证码验证设置

**密码要求：**
- 最少 8 个字符
- 必须包含以下至少 3 种：大写字母、小写字母、数字、特殊字符
- 示例：
  - ✅ `MyPass123!`（有大写、小写、数字、特殊字符）
  - ✅ `secure2024#`（有小写、数字、特殊字符）
  - ❌ `password`（太弱）
  - ❌ `Pass123`（只有 7 个字符）

#### `gen-2fa-wallet`
生成三因子加密钱包并自动创建 keystore 备份
```bash
sol-safekey gen-2fa-wallet -o my-wallet.json
```

**您会得到：**
- `my-wallet.json`: 三因子加密钱包
  - 加密方式：硬件指纹 + 主密码 + 安全问题
  - 只能在此设备上使用所有三个因子 + 2FA 验证码解锁

- `XXXXXXXX_keystore.json`: 跨设备备份
  - XXXXXXXX = 您钱包地址的前 8 个字符
  - 仅使用主密码加密
  - 可在任何设备上用于紧急恢复
  - 解锁命令：`sol-safekey unlock -f XXXXXXXX_keystore.json -p <密码>`

**输入过程：**
1. 输入主密码（一次）
2. 回答安全问题（一次）
3. 输入认证器中的当前 2FA 验证码（一次）

#### `unlock-2fa-wallet`
解锁三因子加密钱包
```bash
sol-safekey unlock-2fa-wallet -f my-wallet.json
```

**要求：**
- 必须在同一设备上（硬件指纹验证）
- 主密码
- 安全问题答案
- 来自认证器应用的当前 2FA 验证码

**安全特性：**
- 硬件指纹自动验证
- 解密需要所有三个因子
- 2FA 验证码必须是当前的（30 秒时间窗口）
- 成功解锁后显示私钥和公钥

### 🔑 基本生成命令

#### `gen-keypair`
生成 keypair 格式私钥
```bash
sol-safekey gen-keypair -o wallet.json
```

#### `gen-key`
生成字符串格式私钥
```bash
sol-safekey gen-key -s 3 -o keys.json
```

#### `gen-keystore`
生成加密 keystore 文件
```bash
sol-safekey gen-keystore -p password123 -o secure.json
```

### 🔐 加密/解密命令

#### `encrypt`
加密现有私钥
```bash
sol-safekey encrypt -k "your_private_key_string" -p password123
```

#### `decrypt`
解密私钥字符串
```bash
sol-safekey decrypt -e "encrypted_data" -p password123
```

#### `unlock`
从文件解密私钥（包括 keystore 备份）
```bash
sol-safekey unlock -f encrypted-file.json -p password123

# 解锁 keystore 备份
sol-safekey unlock -f XXXXXXXX_keystore.json -p your_master_password
```

### 🔍 查询命令

#### `address`
从私钥查看钱包地址
```bash
# 从明文私钥
sol-safekey address -k YOUR_PRIVATE_KEY

# 从加密私钥
sol-safekey address -e ENCRYPTED_KEY -p password123

# 从文件
sol-safekey address -f keys.json

# 从加密文件
sol-safekey address -f encrypted-keys.json -p password123
```

## 📝 选项参考

| 选项 | 短选项 | 说明 |
|------|-------|------|
| `--output` | `-o` | 输出文件路径 |
| `--segments` | `-s` | 分段数量 |
| `--password` | `-p` | 密码 |
| `--private-key` | `-k` | 私钥字符串 |
| `--encrypted-key` | `-e` | 加密数据 |
| `--file-path` | `-f` | 文件路径 |

## 📁 输出格式

### 三因子钱包格式
```json
{
  "encrypted_private_key": "base64_encrypted_data_with_all_factors",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "version": "triple_factor_v1",
  "question_index": 2,
  "created_at": "2025-09-30T10:15:30Z"
}
```

### Keystore 备份格式
```json
{
  "encrypted_private_key": "base64_encrypted_data_password_only",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "encryption_type": "password_only",
  "created_at": "2025-09-30T10:15:30Z",
  "note": "此文件可在任何设备上使用主密码解锁"
}
```

### Keypair 格式
```json
[89, 252, 28, 23, ...]  // 64 字节数组
```

### 字符串格式
```json
{
  "private_key": "5D1iwg89hSXfoqA28ioE...",
  "public_key": "GfkFnJY5pcPp2xeGYTH...",
  "segments": ["5D1iwg89hS", "XfoqA28io", "E..."],
  "created_at": "2025-09-21T04:03:37+00:00"
}
```

## 🔒 安全架构

### 三因子加密过程

1. **硬件指纹收集**
   - CPU 信息
   - 系统序列号
   - MAC 地址
   - 磁盘序列号
   - 组合后使用 SHA256 哈希

2. **密钥派生（PBKDF2）**
   - 输入：硬件指纹 + 主密码 + 安全问题答案
   - 算法：PBKDF2-HMAC-SHA256
   - 迭代次数：200,000
   - 输出：256 位加密密钥

3. **2FA 密钥生成**
   - 派生自：硬件指纹 + 主密码
   - 算法：PBKDF2-HMAC-SHA256
   - 迭代次数：100,000
   - 输出：BASE32 编码的 TOTP 密钥（160 位）
   - 确定性：相同输入始终产生相同的 2FA 密钥

4. **加密**
   - 私钥 + 2FA 密钥打包在一起
   - 使用三因子派生密钥加密
   - 基于 SHA256 的密钥流 XOR 加密
   - BASE64 编码用于存储

5. **解密 + 验证**
   - 硬件指纹自动验证
   - 用户提供：主密码 + 安全问题答案 + 当前 2FA 验证码
   - 重新派生三因子密钥
   - 数据解密
   - 2FA 验证码验证（30 秒时间窗口）
   - 提取私钥

### 安全问题
可用问题（设置时选择一个）：
1. 您母亲的姓名是？
2. 您出生的城市是？
3. 您小学的名称是？
4. 您最喜欢的电影是？
5. 您的第一个宠物叫什么名字？
6. 您父亲的生日是？（格式：YYYYMMDD）
7. 您配偶的名字是？
8. 您最好朋友的名字是？

**注意：** 答案会被规范化（小写、去空格）以确保一致性。

## 🛡️ 安全最佳实践

### 三因子 2FA 钱包

1. **设备绑定**
   - 三因子钱包绑定到您的设备
   - 无法在不同设备上解锁
   - 保持设备安全并备份

2. **密码管理**
   - 使用强大的、唯一的主密码
   - 切勿重复使用其他服务的密码
   - 将密码存储在安全的密码管理器中

3. **安全问题**
   - 选择您永远记得的问题
   - 回答保持一致（系统会规范化大小写）
   - 不要与他人分享答案

4. **2FA 设置**
   - 立即添加到 Google Authenticator 或 Authy
   - 备份您的认证器应用
   - 完成设置前测试验证

5. **Keystore 备份**
   - 始终保留 `XXXXXXXX_keystore.json` 备份文件
   - 存储在多个安全位置（USB 驱动器、加密云存储）
   - 这是您的紧急恢复选项
   - 解锁命令：`sol-safekey unlock -f XXXXXXXX_keystore.json -p <密码>`

6. **恢复规划**
   - **如果设备丢失/损坏**：使用 keystore 备份文件和主密码
   - **如果忘记密码**：无法恢复（设计如此）
   - **如果失去 2FA 访问**：使用 keystore 备份恢复，然后重新运行 setup-2fa
   - **如果忘记安全问题**：无法从三因子钱包恢复（使用 keystore 备份）

### 一般安全

1. **离线存储**：将加密私钥存储在离线设备上
2. **定期备份**：在安全位置保留 keystore 备份的多个副本
3. **测试恢复**：定期测试解密以确保备份有效
4. **物理安全**：保护包含钱包和认证器应用的设备
5. **切勿分享**：切勿分享密码、2FA 验证码或安全问题答案

## ⚠️ 安全警告

1. **密码丢失 = 资金丢失**：如果忘记主密码，加密钱包无法恢复
2. **设备绑定安全**：三因子钱包只能在原始设备上解锁
3. **Keystore 备份至关重要**：`XXXXXXXX_keystore.json` 文件是您唯一的跨设备恢复选项
4. **2FA 应用备份**：失去认证器应用访问需要使用 keystore 备份恢复
5. **无密码重置**：没有"忘记密码"功能 - 这是有意为之的安全设计
6. **安全问题重要性**：将安全问题答案视为与密码一样敏感

## 🔄 迁移和恢复场景

### 场景 1：新设备（计划迁移）
1. 在旧设备上解锁钱包并记录私钥
2. 在新设备上运行 `setup-2fa` 配置新的三因子设置
3. 导入私钥或使用 keystore 备份

### 场景 2：设备丢失/损坏（紧急恢复）
1. 在新设备上安装 sol-safekey
2. 使用您的 `XXXXXXXX_keystore.json` 备份文件
3. 运行：`sol-safekey unlock -f XXXXXXXX_keystore.json -p <主密码>`
4. 提取私钥
5. 在新设备上运行 `setup-2fa` 用于未来安全
6. 使用恢复的密钥生成新的三因子钱包

### 场景 3：失去 2FA 访问
1. 使用 keystore 备份恢复私钥
2. 再次运行 `setup-2fa` 配置新的 2FA
3. 生成新的三因子钱包

### 场景 4：忘记安全问题答案
1. 无法解锁三因子钱包
2. 使用 keystore 备份恢复
3. 使用新的安全问题再次运行 `setup-2fa`

## 🛠️ 开发

### 构建

```bash
cargo build
```

### 测试

```bash
cargo test
```

### 发布构建

```bash
cargo build --release

# 或使用构建脚本
./build.sh
```

### 项目结构

```
sol-safekey/
├── src/
│   ├── lib.rs                    # 核心加密/解密逻辑
│   ├── main.rs                   # CLI 接口
│   ├── totp.rs                   # TOTP 实现
│   ├── secure_totp.rs            # 安全 TOTP 管理器
│   ├── hardware_fingerprint.rs   # 硬件指纹收集
│   └── security_question.rs      # 安全问题处理
├── Cargo.toml                    # 依赖项
├── build.sh                      # 构建脚本
└── README.md                     # 本文件
```

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 贡献领域
- 额外的硬件指纹来源
- 更多安全问题
- 多语言翻译
- 跨平台测试
- 安全审计

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 支持

- **Issues**: [GitHub Issues](https://github.com/0xfnzero/sol-safekey/issues)
- **Telegram**: [加入我们的群组](https://t.me/fnzero_group)
- **Discord**: [加入我们的服务器](https://discord.gg/ckf5UHxz)

## 🙏 致谢

- Solana Foundation 提供的优秀 SDK
- Ring 加密库提供的安全加密操作
- TOTP-RS 提供的 RFC 6238 实现
- QRCode 库提供的 2FA 设置可视化

---

⭐ 如果这个项目帮助您保护 Solana 资产，请给它一个星标！

**用 ❤️ 为 Solana 社区制作**