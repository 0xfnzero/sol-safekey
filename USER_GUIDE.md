# User Guide

Complete guide for using sol-safekey interactive commands and operations.

[中文文档](USER_GUIDE_CN.md)

## Getting Started

Sol-SafeKey provides an interactive command-line interface for secure Solana wallet management and operations. This guide covers all available features and operations.

### Prerequisites

- Solana CLI tools installed (for airdrops and verification)
- Access to Solana devnet or mainnet RPC endpoints
- Terminal with UTF-8 support for proper symbol display

### Accessing Interactive Menu

If you're using a bot that integrates sol-safekey:

```bash
./your-bot safekey
```

Or if using the standalone binary:

```bash
./build-cache/release/examples/complete_bot_example safekey
```

## Main Menu

Upon launching, you'll see the language selection:

```
==================================================
  Language / 语言选择
==================================================

  1.  English
  2.  中文

Select / 选择 [1/2]:
```

After selecting your language, the main menu appears with all available operations.

## Wallet Operations

### 1. Create Plain Text Keypair

**Purpose**: Generate an unencrypted keypair (for testing only)

**Warning**: NOT recommended for production use. Keys are stored in plain text.

**Steps**:
1. Select option `1` from main menu
2. Choose generation method:
   - Option 1: Generate new random keypair
   - Option 2: Import existing private key
3. Choose output format:
   - Option 1: Save as JSON file
   - Option 2: Display in terminal (base58)

**Example**:
```
📝 Plain Text Keypair Options:
  1. Generate new keypair
  2. Import existing private key

Select [1/2]: 1

✅ Keypair generated successfully!
📍 Public Key: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

Output format:
  1. Save as JSON file
  2. Display base58 private key

Select [1/2]: 1

File path (default: wallet.json): test-wallet.json

✅ Saved to test-wallet.json
```

### 2. Create Encrypted Keypair (Recommended)

**Purpose**: Generate or import a keypair with AES-256 encryption

**Security**: Military-grade encryption with PBKDF2 key derivation

**Steps**:
1. Select option `2` from main menu
2. Choose generation method:
   - Option 1: Generate new keypair and encrypt
   - Option 2: Import existing private key and encrypt
3. Set encryption password (minimum 10 characters)
4. Confirm password
5. Choose output format:
   - Option 1: Save as Keystore file (recommended)
   - Option 2: Display encrypted string

**Example**:
```
🔐 Encrypted Keypair Options:
  1. Generate new keypair and encrypt
  2. Import existing private key and encrypt

Select [1/2]: 1

🔒 Set encryption password (min 10 characters):
New password: ************
Confirm password: ************

✅ Password accepted!

Output format:
  1. Save as Keystore file (recommended)
  2. Display encrypted string

Select [1/2]: 1

File path (default: wallet.json): keystore.json

✅ Keystore created successfully!
📍 Public Key: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
📁 Location: keystore.json

⚠️  IMPORTANT: Remember your password! It cannot be recovered.
```

**Best Practices**:
- Use a strong password (16+ characters recommended)
- Mix uppercase, lowercase, numbers, and symbols
- Never share your password
- Store password securely (password manager)
- Keep backup of keystore file in secure location

### 3. Decrypt Encrypted Keypair

**Purpose**: View or export an encrypted keypair

**Steps**:
1. Select option `3` from main menu
2. Choose source:
   - Option 1: Load from Keystore file
   - Option 2: Paste encrypted string
3. Enter decryption password
4. Choose output format:
   - Option 1: Display only
   - Option 2: Save as plain JSON

**Example**:
```
🔓 Decrypt Encrypted Keypair

Source:
  1. Keystore file
  2. Encrypted string

Select [1/2]: 1

Keystore file path: keystore.json

🔑 Enter decryption password: ************

✅ Decrypted successfully!
📍 Public Key: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

Output:
  1. Display only
  2. Save as plain JSON

Select [1/2]: 1

Private Key (base58): 5JW8...
```

### 4. Unlock Wallet (Session)

**Purpose**: Unlock an encrypted wallet for the current session

**Benefit**: Unlocked once, usable for all subsequent operations in same session

**Steps**:
1. Select option `U` or `u` from main menu
2. Enter keystore file path
3. Enter password
4. Wallet remains unlocked for session

**Example**:
```
🔓 Unlock Wallet

Keystore file path [keystore.json]: keystore.json

🔑 Enter wallet password: ************

✅ Wallet unlocked successfully!
📍 Address: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp

You can now use all Solana operations without re-entering password.
```

## Solana Operations

All Solana operations require either:
- An unlocked wallet (option `U`), OR
- You'll be prompted for keystore path and password each time

### 5. Query SOL Balance

**Purpose**: Check SOL balance for an address

**Steps**:
1. Select option `4` from main menu
2. Choose address source:
   - Option 1: Current unlocked wallet
   - Option 2: Enter any address manually
3. Select network:
   - Option 1: Devnet
   - Option 2: Mainnet-beta
4. View balance

**Example**:
```
💰 Query SOL Balance

Address source:
  1. Current wallet
  2. Enter address manually

Select [1/2]: 1

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

💰 Querying balance...
Address: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
Balance: 2.5 SOL
```

### 6. Transfer SOL

**Purpose**: Send SOL to another address

**Prerequisites**: Sufficient SOL for amount + fees (~0.000005 SOL)

**Steps**:
1. Select option `5` from main menu
2. If wallet not unlocked, provide keystore and password
3. Enter recipient address
4. Enter amount in SOL
5. Select network
6. Confirm transaction
7. View transaction signature

**Example**:
```
💸 Transfer SOL

[If not unlocked]
Keystore path: keystore.json
Password: ************

Recipient address: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
Transfer amount (SOL): 0.1

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

Summary:
  From: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
  To: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
  Amount: 0.1 SOL
  Network: Devnet

Confirm transaction? (y/n): y

🚀 Sending transaction...
✅ Transfer successful!
Transaction signature: 5J7W8vN2BxC9K4... (view on explorer)
```

### 7. Wrap SOL → WSOL

**Purpose**: Convert native SOL to wrapped SOL (SPL token)

**Use Case**: Required for certain DeFi protocols and DEX trading

**Steps**:
1. Select option `6` from main menu
2. If wallet not unlocked, provide keystore and password
3. Enter amount to wrap
4. Select network
5. Confirm transaction
6. View transaction signature

**Example**:
```
📦 Wrap SOL → WSOL

[If not unlocked]
Keystore path: keystore.json
Password: ************

Amount to wrap (SOL): 1.0

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

Summary:
  Wrapping: 1.0 SOL → 1.0 WSOL
  Network: Devnet

Confirm? (y/n): y

🚀 Creating wrapped SOL account...
✅ Wrap successful!
Transaction signature: 3K9X2nM5DyH8F7...
WSOL Account: 7xF2wD9cN3bV1K...
```

### 8. Unwrap WSOL → SOL

**Purpose**: Convert wrapped SOL back to native SOL

**Steps**:
1. Select option `7` from main menu
2. If wallet not unlocked, provide keystore and password
3. Enter amount to unwrap
4. Select network
5. Confirm transaction
6. View transaction signature

**Example**:
```
📤 Unwrap WSOL → SOL

[If not unlocked]
Keystore path: keystore.json
Password: ************

Amount to unwrap (WSOL): 0.5

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

Summary:
  Unwrapping: 0.5 WSOL → 0.5 SOL
  Network: Devnet

Confirm? (y/n): y

🚀 Unwrapping...
✅ Unwrap successful!
Transaction signature: 2M8Y1oL4CxG9J6...
```

### 9. Transfer SPL Token

**Purpose**: Send any SPL token to another address

**Prerequisites**: Sufficient SOL for transaction fees + token balance

**Steps**:
1. Select option `8` from main menu
2. If wallet not unlocked, provide keystore and password
3. Enter token mint address
4. Enter recipient address
5. Enter amount (in token's decimal units)
6. Select network
7. Confirm transaction
8. View transaction signature

**Example**:
```
🪙 Transfer SPL Token

[If not unlocked]
Keystore path: keystore.json
Password: ************

Token mint address: So11111111111111111111111111111111111111112
Recipient address: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
Amount: 100

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

Summary:
  Token: So11111111111111111111111111111111111111112
  To: 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin
  Amount: 100
  Network: Devnet

Confirm? (y/n): y

🚀 Sending token...
✅ Transfer successful!
Transaction signature: 4L7X3pN6EzH0K8...
```

### 10. Create Durable Nonce Account

**Purpose**: Create a nonce account for offline transaction signing

**Use Case**: Required for advanced transaction patterns and offline signing

**Prerequisites**: ~0.00144288 SOL for rent-exempt nonce account

**Steps**:
1. Select option `9` from main menu
2. If wallet not unlocked, provide keystore and password
3. Select network
4. Confirm transaction
5. View nonce account address and transaction signature

**Example**:
```
🔢 Create Durable Nonce Account

[If not unlocked]
Keystore path: keystore.json
Password: ************

Select network:
  1. Devnet
  2. Mainnet-beta

Select [1/2]: 1

Summary:
  Creating nonce account...
  Rent: ~0.00144288 SOL
  Authority: E7Rmd6piasPNs9jqRBUfS8nvNqDx6j5qPDE6Le7us5bp
  Network: Devnet

Confirm? (y/n): y

🚀 Creating nonce account...
✅ Nonce account created successfully!
Account: 8yG3wE5dK2mH9L...
Transaction signature: 6N9Z4qO7FyJ1M3...
```

## Tips and Best Practices

### Wallet Security

1. **Strong Passwords**:
   - Minimum 16 characters
   - Mix uppercase, lowercase, numbers, symbols
   - Use a password manager
   - Never reuse passwords

2. **Backup Strategy**:
   - Keep encrypted keystore in multiple secure locations
   - Store password separately from keystore
   - Consider hardware wallet for large amounts
   - Test recovery procedure on devnet

3. **Network Safety**:
   - Always test on devnet first
   - Double-check recipient addresses
   - Start with small amounts
   - Verify transaction signatures on explorer

### Session Management

1. **Unlock Wallet** once per session for convenience
2. Operations become faster without repeated password entry
3. Wallet unlocked only for current process
4. Re-lock by exiting and restarting

### Testing Workflow

1. Create test wallet on devnet
2. Get devnet SOL: `solana airdrop 2 <address> --url devnet`
3. Test all operations on devnet
4. Verify on devnet explorer: https://explorer.solana.com/?cluster=devnet
5. Only use mainnet after thorough testing

### Transaction Fees

- SOL transfer: ~0.000005 SOL
- Token transfer: ~0.00001 SOL
- Wrap/Unwrap: ~0.00001 SOL
- Create nonce: ~0.00144288 SOL (rent-exempt)

### Common Workflows

**Daily Trading Bot**:
1. Start bot with `./bot safekey`
2. Select `U` to unlock wallet
3. Enter password once
4. Exit menu to let bot run
5. Bot can sign transactions without password

**One-Time Transfer**:
1. Launch safekey
2. Select transfer operation
3. Enter keystore path and password when prompted
4. Complete transaction
5. Exit

**Portfolio Management**:
1. Unlock wallet (`U`)
2. Check balance (`4`)
3. Perform multiple operations as needed
4. No password re-entry required

## Troubleshooting

### "Failed to decrypt keystore"
**Cause**: Incorrect password or corrupted file
**Solution**: Verify password, try backup keystore file

### "Connection refused" / "RPC error"
**Cause**: Network issues or RPC node down
**Solution**: Check internet, try different network, use different RPC endpoint

### "Insufficient funds"
**Cause**: Not enough SOL for transaction + fees
**Solution**:
- Check balance
- Get devnet SOL: `solana airdrop 2 <address> --url devnet`
- For mainnet: transfer SOL to account

### "Transaction failed"
**Cause**: Network congestion, invalid transaction, or account issues
**Solution**:
- Wait and retry
- Check recipient address is valid
- Ensure account has token account for SPL transfers
- Verify sufficient balance for amount + fees

### "Account not found"
**Cause**: Address has never received SOL (not initialized)
**Solution**: Send a small amount of SOL first to initialize

### Terminal display issues
**Cause**: Terminal doesn't support UTF-8 or emojis
**Solution**: Use modern terminal emulator (iTerm2, Windows Terminal, etc.)

## Advanced Usage

### Scripting with Stdin

You can automate password input via stdin for bot deployment:

```bash
# Create startup script
echo "your-password" | ./your-bot
```

**Security Note**: Only use this in secure environments. Never hardcode passwords.

### Multiple Wallets

Create separate keystores for different purposes:

```bash
# Trading wallet
./bot safekey
# Create encrypted → keystore-trading.json

# Holding wallet
./bot safekey
# Create encrypted → keystore-holding.json

# Use specific wallet
./bot safekey
# Unlock → keystore-trading.json
```

### Batch Operations

After unlocking wallet once, perform multiple operations without re-entering password:

1. Unlock wallet (`U`)
2. Check balance (`4`)
3. Transfer SOL (`5`)
4. Wrap SOL (`6`)
5. Transfer tokens (`8`)
6. All without password prompts

## Getting Help

- **Documentation**: Check [Bot Integration Guide](BOT_INTEGRATION.md)
- **Examples**: Review `examples/complete_bot_example.rs`
- **Explorer**: Verify transactions on https://explorer.solana.com
- **Solana Docs**: https://docs.solana.com

---

**Remember**:
- Always test on devnet first
- Keep keystores and passwords secure
- Backup everything before mainnet use
- Start with small amounts
- Verify all addresses carefully
