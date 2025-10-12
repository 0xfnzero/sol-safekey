# 使用手册

Sol-safekey 交互命令和操作的完整使用指南。

[English Documentation](USER_GUIDE.md)

## 开始使用

Sol-SafeKey 提供交互式命令行界面，用于安全的 Solana 钱包管理和操作。本指南涵盖所有可用功能和操作。

### 前置要求

- 已安装 Solana CLI 工具（用于空投和验证）
- 可访问 Solana devnet 或 mainnet RPC 端点
- 支持 UTF-8 的终端以正确显示符号

### 访问交互式菜单

如果你使用的 bot 集成了 sol-safekey:

```bash
./你的bot safekey
```

或者使用独立二进制文件:

```bash
./build-cache/release/examples/complete_bot_example safekey
```

## 主菜单

启动后，你会看到语言选择:

```
==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]:
```

选择语言后，主菜单显示所有可用操作。

## 钱包操作

### 1. 创建明文密钥对

**用途**: 生成未加密的密钥对（仅用于测试）

**警告**: 不建议在生产环境使用。密钥以明文存储。

**步骤**:
1. 从主菜单选择选项 `1`
2. 选择生成方式:
   - 选项 1: 生成新的随机密钥对
   - 选项 2: 导入现有私钥
3. 选择输出格式:
   - 选项 1: 保存为 JSON 文件
   - 选项 2: 在终端显示（base58）

**示例**:
```
📝 明文密钥对选项:
  1. 生成新密钥对
  2. 导入现有私钥

请选择 [1/2]: 1

✅ 密钥对生成成功！
📍 公钥: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

输出格式:
  1. 保存为 JSON 文件
  2. 显示 base58 私钥

请选择 [1/2]: 1

文件路径（默认: wallet.json）: test-wallet.json

✅ 已保存到 test-wallet.json
```

### 2. 创建加密密钥对（推荐）

**用途**: 生成或导入带 AES-256 加密的密钥对

**安全性**: 军事级加密配合 PBKDF2 密钥派生

**步骤**:
1. 从主菜单选择选项 `2`
2. 选择生成方式:
   - 选项 1: 生成新密钥对并加密
   - 选项 2: 导入现有私钥并加密
3. 设置加密密码（最少 10 个字符）
4. 确认密码
5. 选择输出格式:
   - 选项 1: 保存为 Keystore 文件（推荐）
   - 选项 2: 显示加密字符串

**示例**:
```
🔐 加密密钥对选项:
  1. 生成新密钥对并加密
  2. 导入现有私钥并加密

请选择 [1/2]: 1

🔒 设置加密密码（至少 10 个字符）:
新密码: ************
确认密码: ************

✅ 密码已接受！

输出格式:
  1. 保存为 Keystore 文件（推荐）
  2. 显示加密字符串

请选择 [1/2]: 1

文件路径（默认: wallet.json）: keystore.json

✅ Keystore 创建成功！
📍 公钥: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
📁 位置: keystore.json

⚠️  重要: 记住你的密码！密码无法恢复。
```

**最佳实践**:
- 使用强密码（建议 16+ 字符）
- 混合大小写字母、数字和符号
- 永不分享你的密码
- 安全存储密码（密码管理器）
- 在安全位置保留 keystore 文件的备份

### 3. 解密加密的密钥对

**用途**: 查看或导出加密的密钥对

**步骤**:
1. 从主菜单选择选项 `3`
2. 选择来源:
   - 选项 1: 从 Keystore 文件加载
   - 选项 2: 粘贴加密字符串
3. 输入解密密码
4. 选择输出格式:
   - 选项 1: 仅显示
   - 选项 2: 保存为明文 JSON

**示例**:
```
🔓 解密加密的密钥对

来源:
  1. Keystore 文件
  2. 加密字符串

请选择 [1/2]: 1

Keystore 文件路径: keystore.json

🔑 输入解密密码: ************

✅ 解密成功！
📍 公钥: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

输出:
  1. 仅显示
  2. 保存为明文 JSON

请选择 [1/2]: 1

私钥（base58）: 5JW8...
```

### 4. 解锁钱包（会话）

**用途**: 为当前会话解锁加密钱包

**优势**: 解锁一次，可用于同一会话中的所有后续操作

**步骤**:
1. 从主菜单选择选项 `U` 或 `u`
2. 输入 keystore 文件路径
3. 输入密码
4. 钱包在会话期间保持解锁状态

**示例**:
```
🔓 解锁钱包

Keystore 文件路径 [keystore.json]: keystore.json

🔑 输入钱包密码: ************

✅ 钱包解锁成功！
📍 地址: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

现在可以使用所有 Solana 操作而无需重新输入密码。
```

## Solana 操作

所有 Solana 操作需要:
- 已解锁的钱包（选项 `U`），或者
- 每次操作时会提示输入 keystore 路径和密码

### 5. 查询 SOL 余额

**用途**: 查询地址的 SOL 余额

**步骤**:
1. 从主菜单选择选项 `4`
2. 选择地址来源:
   - 选项 1: 当前解锁的钱包
   - 选项 2: 手动输入任何地址
3. 选择网络:
   - 选项 1: Devnet
   - 选项 2: Mainnet-beta
4. 查看余额

**示例**:
```
💰 查询 SOL 余额

地址来源:
  1. 当前钱包
  2. 手动输入地址

请选择 [1/2]: 1

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

💰 正在查询余额...
地址: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
余额: 2.5 SOL
```

### 6. 转账 SOL

**用途**: 向另一个地址发送 SOL

**前置条件**: 足够的 SOL 用于金额 + 手续费（约 0.000005 SOL）

**步骤**:
1. 从主菜单选择选项 `5`
2. 如果钱包未解锁，提供 keystore 和密码
3. 输入接收地址
4. 输入 SOL 金额
5. 选择网络
6. 确认交易
7. 查看交易签名

**示例**:
```
💸 转账 SOL

[如果未解锁]
Keystore 路径: keystore.json
密码: ************

接收地址: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
转账金额（SOL）: 0.1

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

摘要:
  从: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
  到: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
  金额: 0.1 SOL
  网络: Devnet

确认交易？(y/n): y

🚀 发送交易中...
✅ 转账成功！
交易签名: 5J7W8vN2BxC9K4... (在浏览器中查看)
```

### 7. 包装 SOL → WSOL

**用途**: 将原生 SOL 转换为包装的 SOL（SPL 代币）

**使用场景**: 某些 DeFi 协议和 DEX 交易需要

**步骤**:
1. 从主菜单选择选项 `6`
2. 如果钱包未解锁，提供 keystore 和密码
3. 输入要包装的金额
4. 选择网络
5. 确认交易
6. 查看交易签名

**示例**:
```
📦 包装 SOL → WSOL

[如果未解锁]
Keystore 路径: keystore.json
密码: ************

包装金额（SOL）: 1.0

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

摘要:
  包装: 1.0 SOL → 1.0 WSOL
  网络: Devnet

确认？(y/n): y

🚀 正在创建包装 SOL 账户...
✅ 包装成功！
交易签名: 3K9X2nM5DyH8F7...
WSOL 账户: 7xF2wD9cN3bV1K...
```

### 8. 解包 WSOL → SOL

**用途**: 将包装的 SOL 转换回原生 SOL

**步骤**:
1. 从主菜单选择选项 `7`
2. 如果钱包未解锁，提供 keystore 和密码
3. 输入要解包的金额
4. 选择网络
5. 确认交易
6. 查看交易签名

**示例**:
```
📤 解包 WSOL → SOL

[如果未解锁]
Keystore 路径: keystore.json
密码: ************

解包金额（WSOL）: 0.5

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

摘要:
  解包: 0.5 WSOL → 0.5 SOL
  网络: Devnet

确认？(y/n): y

🚀 解包中...
✅ 解包成功！
交易签名: 2M8Y1oL4CxG9J6...
```

### 9. 转账 SPL 代币

**用途**: 向另一个地址发送任何 SPL 代币

**前置条件**: 足够的 SOL 用于交易手续费 + 代币余额

**步骤**:
1. 从主菜单选择选项 `8`
2. 如果钱包未解锁，提供 keystore 和密码
3. 输入代币 mint 地址
4. 输入接收地址
5. 输入金额（以代币的小数单位）
6. 选择网络
7. 确认交易
8. 查看交易签名

**示例**:
```
🪙 转账 SPL 代币

[如果未解锁]
Keystore 路径: keystore.json
密码: ************

代币 mint 地址: So11111111111111111111111111111111111111112
接收地址: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
金额: 100

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

摘要:
  代币: So11111111111111111111111111111111111111112
  到: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
  金额: 100
  网络: Devnet

确认？(y/n): y

🚀 发送代币中...
✅ 转账成功！
交易签名: 4L7X3pN6EzH0K8...
```

### 10. 创建 Durable Nonce 账户

**用途**: 创建用于离线交易签名的 nonce 账户

**使用场景**: 高级交易模式和离线签名所需

**前置条件**: 约 0.00144288 SOL 用于免租金 nonce 账户

**步骤**:
1. 从主菜单选择选项 `9`
2. 如果钱包未解锁，提供 keystore 和密码
3. 选择网络
4. 确认交易
5. 查看 nonce 账户地址和交易签名

**示例**:
```
🔢 创建 Durable Nonce 账户

[如果未解锁]
Keystore 路径: keystore.json
密码: ************

选择网络:
  1. Devnet
  2. Mainnet-beta

请选择 [1/2]: 1

摘要:
  正在创建 nonce 账户...
  租金: 约 0.00144288 SOL
  权限: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
  网络: Devnet

确认？(y/n): y

🚀 创建 nonce 账户中...
✅ Nonce 账户创建成功！
账户: 8yG3wE5dK2mH9L...
交易签名: 6N9Z4qO7FyJ1M3...
```

## 提示和最佳实践

### 钱包安全

1. **强密码**:
   - 最少 16 个字符
   - 混合大小写字母、数字、符号
   - 使用密码管理器
   - 永不重复使用密码

2. **备份策略**:
   - 在多个安全位置保存加密的 keystore
   - 密码与 keystore 分开存储
   - 对于大额资产考虑使用硬件钱包
   - 在 devnet 上测试恢复流程

3. **网络安全**:
   - 始终先在 devnet 上测试
   - 仔细检查接收地址
   - 从小额开始
   - 在浏览器上验证交易签名

### 会话管理

1. 每个会话**解锁钱包一次**以方便使用
2. 无需重复输入密码，操作更快
3. 钱包仅为当前进程解锁
4. 退出并重启以重新锁定

### 测试工作流

1. 在 devnet 上创建测试钱包
2. 获取 devnet SOL: `solana airdrop 2 <地址> --url devnet`
3. 在 devnet 上测试所有操作
4. 在 devnet 浏览器上验证: https://explorer.solana.com/?cluster=devnet
5. 彻底测试后才使用 mainnet

### 交易手续费

- SOL 转账: 约 0.000005 SOL
- 代币转账: 约 0.00001 SOL
- 包装/解包: 约 0.00001 SOL
- 创建 nonce: 约 0.00144288 SOL（免租金）

### 常见工作流程

**日常交易 Bot**:
1. 使用 `./bot safekey` 启动 bot
2. 选择 `U` 解锁钱包
3. 输入密码一次
4. 退出菜单让 bot 运行
5. Bot 可以签署交易而无需密码

**一次性转账**:
1. 启动 safekey
2. 选择转账操作
3. 提示时输入 keystore 路径和密码
4. 完成交易
5. 退出

**投资组合管理**:
1. 解锁钱包（`U`）
2. 查询余额（`4`）
3. 根据需要执行多个操作
4. 无需重新输入密码

## 故障排除

### "解密 keystore 失败"
**原因**: 密码错误或文件损坏
**解决方案**: 验证密码，尝试备份的 keystore 文件

### "连接被拒绝" / "RPC 错误"
**原因**: 网络问题或 RPC 节点宕机
**解决方案**: 检查网络，尝试不同网络，使用不同 RPC 端点

### "余额不足"
**原因**: SOL 不足以支付交易 + 手续费
**解决方案**:
- 检查余额
- 获取 devnet SOL: `solana airdrop 2 <地址> --url devnet`
- 对于 mainnet: 向账户转入 SOL

### "交易失败"
**原因**: 网络拥堵、无效交易或账户问题
**解决方案**:
- 等待后重试
- 检查接收地址是否有效
- 确保账户有 SPL 转账所需的代币账户
- 验证余额足够支付金额 + 手续费

### "未找到账户"
**原因**: 地址从未收到过 SOL（未初始化）
**解决方案**: 先发送少量 SOL 以初始化

### 终端显示问题
**原因**: 终端不支持 UTF-8 或表情符号
**解决方案**: 使用现代终端模拟器（iTerm2、Windows Terminal 等）

## 高级用法

### 通过 Stdin 脚本化

你可以通过 stdin 自动输入密码用于 bot 部署:

```bash
# 创建启动脚本
echo "你的密码" | ./你的bot
```

**安全提示**: 仅在安全环境中使用。永不硬编码密码。

### 多个钱包

为不同目的创建单独的 keystore:

```bash
# 交易钱包
./bot safekey
# 创建加密 → keystore-trading.json

# 持有钱包
./bot safekey
# 创建加密 → keystore-holding.json

# 使用特定钱包
./bot safekey
# 解锁 → keystore-trading.json
```

### 批量操作

解锁钱包一次后，执行多个操作而无需重新输入密码:

1. 解锁钱包（`U`）
2. 查询余额（`4`）
3. 转账 SOL（`5`）
4. 包装 SOL（`6`）
5. 转账代币（`8`）
6. 所有操作都无需密码提示

## 获取帮助

- **文档**: 查看 [Bot 集成指南](BOT_INTEGRATION_CN.md)
- **示例**: 查看 `examples/complete_bot_example.rs`
- **浏览器**: 在 https://explorer.solana.com 验证交易
- **Solana 文档**: https://docs.solana.com

---

**记住**:
- 始终先在 devnet 上测试
- 保持 keystore 和密码安全
- 在使用 mainnet 前备份所有内容
- 从小额开始
- 仔细验证所有地址
