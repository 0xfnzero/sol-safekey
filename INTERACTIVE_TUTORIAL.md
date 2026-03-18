<div align="center">
    <h1>📖 Sol-SafeKey 交互式菜单完整教程</h1>
    <h3><em>Step-by-step guide for all interactive menu operations</em></h3>
</div>

---

## 📑 Table of Contents

- [🎯 Getting Started](#-getting-started)
- [📚 Feature-by-Feature Guide](#-feature-by-feature-guide)
  - [1. Create Plaintext Key](#1-create-plaintext-key)
  - [2. Create Encrypted Key](#2-create-encrypted-key)
  - [3. Decrypt Key](#3-decrypt-key)
  - [U. Unlock Wallet](#u-unlock-wallet)
  - [4. Setup 2FA Authentication](#4-setup-2fa-authentication)
  - [5. Generate Triple-Factor Wallet](#5-generate-triple-factor-wallet)
  - [6. Unlock Triple-Factor Wallet](#6-unlock-triple-factor-wallet)
  - [7. Check SOL Balance](#7-check-sol-balance)
  - [8. Transfer SOL](#8-transfer-sol)
  - [9. Create WSOL ATA](#9-create-wsol-ata)
  - [10. Wrap SOL → WSOL](#10-wrap-sol--wsol)
  - [11. Unwrap WSOL → SOL](#11-unwrap-wsol--sol)
  - [12. Close WSOL ATA](#12-close-wsol-ata)
  - [13. Transfer SPL Token](#13-transfer-spl-token)
  - [14. Create Nonce Account](#14-create-nonce-account)
  - [15. Pump.fun Sell Token](#15-pumpfun-sell-token)
  - [16. PumpSwap Sell Token](#16-pumpswap-sell-token)
  - [17. Pump.fun Cashback](#17-pumpfun-cashback)
  - [18. PumpSwap Cashback](#18-pumpswap-cashback)

---

### 📋 Quick Operation Index

Quick access to all 18 interactive menu operations:

#### 🔑 Core Functions (Options 1-3)
- **[1. Create Plaintext Key](#1-create-plaintext-key)** - Generate and save unencrypted Solana keypair
- **[2. Create Encrypted Key (bot)](#2-create-encrypted-key)** - Encrypt existing private key and save to keystore
- **[3. Decrypt Key](#3-decrypt-key)** - Decrypt keystore and reveal private key

#### 🔒 Wallet Management (Option U)
- **[U. Unlock Wallet](#u-unlock-wallet)** - Unlock wallet for Solana operations

#### 🛡️ Advanced Security Features (Options 4-6)
- **[4. Setup 2FA Authentication](#4-setup-2fa-authentication)** - Configure two-factor authentication
- **[5. Generate Triple-Factor Wallet](#5-generate-triple-factor-wallet)** - Create wallet with 3FA (hardware + password + security question + 2FA)
- **[6. Unlock Triple-Factor Wallet](#6-unlock-triple-factor-wallet)** - Decrypt 3FA encrypted wallet

#### 💰 Solana On-Chain Operations (Options 7-18)
- **[7. Check SOL Balance](#7-check-sol-balance)** - Query wallet SOL balance
- **[8. Transfer SOL](#8-transfer-sol)** - Send SOL to another address
- **[9. Create WSOL ATA](#9-create-wsol-ata)** - Create Wrapped SOL Associated Token Account
- **[10. Wrap SOL → WSOL](#10-wrap-sol--wsol)** - Convert SOL to Wrapped SOL
- **[11. Unwrap WSOL → SOL](#11-unwrap-wsol--sol)** - Convert Wrapped SOL back to SOL
- **[12. Close WSOL ATA](#12-close-wsol-ata)** - Close WSOL ATA and convert remaining WSOL to SOL
- **[13. Transfer SPL Token](#13-transfer-spl-token)** - Send SPL tokens to another address
- **[14. Create Nonce Account](#14-create-nonce-account)** - Create durable nonce for transaction replay protection
- **[15. Pump.fun Sell Token](#15-pumpfun-sell-token)** - Sell tokens on Pump.fun DEX (internal market)
- **[16. PumpSwap Sell Token](#16-pumpswap-sell-token)** - Sell tokens on PumpSwap DEX
- **[17. Pump.fun Cashback (View & Claim)](#17-pumpfun-cashback-view--claim)** - View and claim pump.fun cashback (native SOL)
- **[18. PumpSwap Cashback (View & Claim)](#18-pumpswap-cashback-view--claim)** - View and claim PumpSwap cashback (WSOL)

---

## 🎯 Getting Started

#### Step 1: Launch Interactive Menu

```bash
sol-safekey start
```

You will see the language selection screen. Choose your preferred language:

**English**: Enter `2`
**中文**: Enter `1`

#### Step 2: Select an Operation

After selecting language, you'll see the main menu shown above. Enter the number corresponding to your desired operation.

**Important**: If you haven't created a wallet yet, you need to:
- **Unlock an existing wallet** (Option `U`)
- **Create a new wallet** (Options `1` or `2`)

---

### 📚 Feature-by-Feature Guide

#### 1. Create Plaintext Key (Option 1)

**Purpose**: Generate a new Solana keypair and save it to a file (unencrypted)

**Use Cases**:
- Development and testing
- When you need to share the key with team members
- Quick wallet generation for testing

**Steps**:
1. Select `1` from the main menu
2. Choose a filename for saving (default: `keypair.json`)
3. System generates a new Solana keypair
4. Private key is saved to the file (unencrypted)

**Example Output**:
```
🔓 Create Plaintext Key

Enter filename [keypair.json]: my_keypair.json

Generating new Solana keypair...
✅ Key generation successful!

Public Key: 7xKm...9xW3

📝 Important Notes:
   • This file contains your private key in plaintext
   • Only use for development and testing
   • Do NOT share this file with anyone
   • For production, use encrypted options (Option 2)
```

---

#### 2. Create Encrypted Key (Option 2)

**Purpose**: Encrypt an existing private key and save it to a secure keystore file

**Use Cases**:
- Production wallet storage
- Secure backup of existing keys
- Preparing wallet for bot integration

**Steps**:
1. Select `2` from the main menu
2. Paste or enter your private key (base58 format)
3. Enter a password (10-20 characters, must include at least 3 of: uppercase, lowercase, digit, special character)
4. Confirm password by entering it again
5. Choose a filename (default: `keystore.json`)
6. System encrypts and saves to the file

**Password Requirements**:
- Length: 10-20 characters
- Must include at least 3 of: uppercase, lowercase, digit, special character
- Examples: `MySecureP@ssw0rd!`, `StrongKey#2025`, `abc123XYZ!`

**Example Output**:
```
🔐 Create Encrypted Key

Enter or paste your private key (base58): <paste-your-key-here>

Please set a password (10-20 characters, must include at least 3 of: uppercase, lowercase, digit, special character):
Enter password: ********
Confirm password: ********

Enter filename to save [keystore.json]: my_keystore.json

Encrypting and saving...
✅ Keystore encrypted and saved to: my_keystore.json

📝 Important:
   • Password required for decryption - never lose it!
   • keystore.json contains AES-256 encrypted private key
   • Recommended to backup keystore.json to multiple secure locations
```

---

#### 3. Decrypt Key (Option 3)

**Purpose**: Load an encrypted keystore file and decrypt to reveal the private key

**Use Cases**:
- View your private key when needed
- Export wallet to different formats
- Verify wallet contents

**Steps**:
1. Select `3` from the main menu
2. Enter keystore filename (default: `keystore.json`)
3. Enter your password
4. System decrypts and displays the private key

**Example Output**:
```
🔓 Decrypt Key

Enter keystore filename [keystore.json]: my_keystore.json

Enter password: ********

✅ Decryption successful!

Public Key: 7xKm...9xW3
Private Key: <base58-encoded-key>

⚠️  Security Warning:
   • Private key is now displayed in plaintext
   • Keep this screen away from prying eyes
   • Consider using decrypted key only when necessary
```

---

### 🔒 Wallet Management

#### U. Unlock Wallet

**Purpose**: Unlock a wallet for use in Solana operations (Options 7-18)

**Steps**:
1. Select `U` from the main menu
2. Enter keystore filename (default: `keystore.json`)
3. Enter your password
4. Wallet status changes to "Unlocked" and wallet is stored in session

**Wallet Status Indicator**:

The wallet status indicator shows:
- **Unlocked**: Wallet is ready for operations
- **Locked**: No wallet loaded in session

**Example Output**:
```
🔓 Unlock Wallet

Enter keystore filename [keystore.json]:

Enter password: ********

✅ Wallet unlocked successfully!

📍 Current Wallet: 7xKm...9xW3
🔒 Wallet Status: Unlocked

You can now perform Solana operations (Options 7-18)
```

---

### 🛠️ Solana On-Chain Operations

All Solana operations (Options 7-18) require an **unlocked wallet**. Make sure to unlock your wallet first!

#### 7. Check SOL Balance

**Purpose**: Query the SOL balance of your wallet

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `7` from the main menu
3. Enter RPC URL (press Enter for default: mainnet-beta)
4. System queries and displays your balance

**Example Output**:
```
💰 Check SOL Balance

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Querying balance...
✅ Balance query successful!

📍 Wallet Address: 7xKm...9xW3
💰 SOL Balance: 1.234567890 SOL (1,234,567,890 lamports)

Explorer: https://solscan.io/address/7xKm...9xW3
```

---

#### 8. Transfer SOL

**Purpose**: Send SOL to another Solana address

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `8` from the main menu
3. Enter recipient address
4. Enter amount in SOL
5. Enter RPC URL (press Enter for default)
6. Review and confirm transaction
7. System sends the transaction

**Example Output**:
```
💸 Transfer SOL

Enter recipient address: 5xKm...2xW3

Enter amount in SOL: 0.1

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Confirm transaction? (yes/no): yes

Sending transaction...
✅ Transaction sent successfully!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 9. Create WSOL ATA

**Purpose**: Create a Wrapped SOL (WSOL) Associated Token Account

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `9` from the main menu
3. Enter RPC URL (press Enter for default)
4. System creates WSOL ATA account

**Example Output**:
```
📝 Create WSOL ATA

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Creating WSOL ATA...
✅ WSOL ATA created successfully!

📍 ATA Address: 7xKm...9xW3
📊 Token Mint: So11111111111111111111111111111111111111111112

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 10. Wrap SOL → WSOL

**Purpose**: Convert SOL to WSOL (Wrapped SOL)

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `10` from the main menu
3. Enter amount in SOL to wrap
4. Enter RPC URL (press Enter for default)
5. Review and confirm transaction
6. System wraps SOL to WSOL

**Example Output**:
```
📦 Wrap SOL → WSOL

Enter amount in SOL: 0.5

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Confirm transaction? (yes/no): yes

Wrapping SOL to WSOL...
✅ Wrap successful!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3

✅ WSOL Balance updated: 0.5 WSOL
```

---

#### 11. Unwrap WSOL → SOL

**Purpose**: Convert WSOL back to SOL

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `11` from the main menu
3. Enter amount in WSOL to unwrap
4. Enter RPC URL (press Enter for default)
5. Review and confirm transaction
6. System unwraps WSOL to SOL

**Example Output**:
```
📤 Unwrap WSOL → SOL

Enter amount in WSOL: 0.5

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Confirm transaction? (yes/no): yes

Unwrapping WSOL to SOL...
✅ Unwrap successful!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3

✅ SOL Balance updated!
```

---

#### 12. Close WSOL ATA

**Purpose**: Close WSOL ATA account (convert remaining WSOL to SOL)

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `12` from the main menu
3. Enter RPC URL (press Enter for default)
4. System closes ATA account

**Example Output**:
```
🗑️ Close WSOL ATA

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Closing WSOL ATA...
✅ WSOL ATA closed successfully!

Remaining WSOL: 0.5 WSOL → 0.5 SOL
Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 13. Transfer SPL Token

**Purpose**: Send SPL tokens to another Solana address

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `13` from the main menu
3. Enter token mint address
4. Enter recipient address
5. Enter amount
6. Enter RPC URL (press Enter for default)
7. Review and confirm transaction
8. System sends the transaction

**Example Output**:
```
💎 Transfer SPL Token

Enter token mint address: <token-mint-address>

Enter recipient address: 5xKm...2xW3

Enter amount: 100

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Confirm transaction? (yes/no): yes

Sending SPL token...
✅ Token transfer successful!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 14. Create Nonce Account

**Purpose**: Create a durable nonce account for transaction replay protection

**Use Cases**:
- Batch transaction processing
- Preventing transaction replay attacks
- Ensuring transaction ordering

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `14` from the main menu
3. Enter RPC URL (press Enter for default)
4. System creates nonce account

**Example Output**:
```
🔑 Create Nonce Account

Enter RPC URL [https://api.mainnet-beta.solana.com]:

Creating nonce account...
✅ Nonce account created and initialized successfully!

📍 Nonce Address: 5xKm...7xW3

💡 Save this nonce account address for future use!
Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

### 🏪 Token Operations

#### 15. Pump.fun Sell Token

**Purpose**: Sell tokens on Pump.fun DEX

**Use Cases**:
- Trading tokens on Pump.fun platform
- Automated selling strategies
- Quick liquidity exit

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `15` from the main menu
3. Enter token mint address
4. Configure sell options:
   - Slippage (basis points, default: 9900)
   - Seed optimization (default: yes)
   - Use seed-optimized ATA (optional, same as PumpSwap)
5. Confirm transaction
6. System sells all token balance

**Example Output**:
```
🎪 Pump.fun Sell Token

Enter token mint address: <token-mint-address>

Enter slippage (basis points, default 9900): [Enter]

Use seed optimization? (yes/no, default yes) [Enter]

Use seed-optimized ATA? (yes/no, default yes) [Enter]

Selling all tokens...
✅ Sell successful!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 16. PumpSwap Sell Token

**Purpose**: Sell tokens on PumpSwap DEX

**Use Cases**:
- Trading tokens on PumpSwap platform
- Access to multiple liquidity pools
- Advanced trading features

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `16` from the main menu
3. Enter token mint address
4. Configure sell options
5. Confirm transaction
6. System sells tokens

**Example Output**:
```
🔄 PumpSwap Sell Token

Enter token mint address: <token-mint-address>

Enter slippage (basis points, default 9900): [Enter]

Selling tokens...
✅ Sell successful!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 17. Pump.fun Cashback (View & Claim)

**Purpose**: View and claim pump.fun cashback (native SOL)

**Use Cases**:
- Check available cashback balance
- Claim earned cashback

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `17` from the main menu
3. System queries and claims available cashback

**Example Output**:
```
💰 Pump.fun Cashback

Querying cashback status...
✅ Cashback available: 0.123 SOL

Claiming cashback...
✅ Cashback claimed successfully!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

#### 18. PumpSwap Cashback (View & Claim)

**Purpose**: View and claim PumpSwap cashback (WSOL)

**Use Cases**:
- Check available WSOL cashback balance
- Claim earned WSOL cashback

**Steps**:
1. Make sure wallet is unlocked (Status: Unlocked)
2. Select `18` from the main menu
3. System queries and claims available cashback

**Example Output**:
```
💰 PumpSwap Cashback

Querying cashback status...
✅ Cashback available: 0.05 WSOL

Claiming cashback...
✅ Cashback claimed successfully!

Signature: 5xKm...9xW3
Explorer: https://solscan.io/tx/5xKm...9xW3
```

---

### 🔐 Advanced Security Features

#### 4. Setup 2FA Authentication

**Purpose**: Configure two-factor authentication for enhanced security

**Use Cases**:
- Adding an extra security layer to your wallet
- Protecting against unauthorized access
- Required for triple-factor wallets

**Steps**:
1. Select `4` from the main menu
2. System collects hardware fingerprint
3. Set master password
4. Set security question
5. Configure TOTP (Time-based One-Time Password)

**Requirements**:
- Hardware fingerprint collection (device-dependent)
- TOTP authenticator app (Google Authenticator, Authy, etc.)
- Master password (10-20 characters, 3+ character types)
- Security question (choose from predefined options)

**Output**:
- ✅ Hardware fingerprint (device binding)
- ✅ Master password encryption
- ✅ Security question verification
- ✅ TOTP configuration (6-digit codes, 30-second rotation)

---

#### 5. Generate Triple-Factor Wallet

**Purpose**: Generate a wallet with three-factor authentication (hardware + password + security question + 2FA)

**Security Features**:
- ✅ Hardware fingerprint (device binding)
- ✅ Master password encryption
- ✅ Security question verification
- ✅ TOTP dynamic codes (6-digit, 30-second rotation)
- ✅ Durable nonce account support

**Use Cases**:
- Maximum security for production wallets
- Multi-device support with TOTP
- Recovery with multiple authentication factors

**Steps**:
1. Select `5` from the main menu
2. System collects hardware fingerprint
3. Set master password
4. Answer security question
5. Configure TOTP authenticator
6. System generates encrypted wallet file

**Output**:
- Encrypted wallet file (triple-factor)
- Cross-device keystore backup
- Recovery instructions

---

#### 6. Unlock Triple-Factor Wallet

**Purpose**: Decrypt a triple-factor encrypted wallet

**Requirements**:
- Original device (for hardware fingerprint)
- Master password
- Security question answer
- Current TOTP code (6 digits, changes every 30 seconds)

**Steps**:
1. Select `6` from the main menu
2. Enter wallet file path (generated from Option 5)
3. Enter master password
4. Answer security question
5. Enter current TOTP code
6. System decrypts and displays wallet

**Security**: All three factors must be correct to unlock.

---

