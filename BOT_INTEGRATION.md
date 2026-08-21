# Bot Integration Guide

Complete guide for integrating fnzero-safe into your Solana trading bot or application.

[中文文档](BOT_INTEGRATION_CN.md)

## Why FnzeroSafe?

FnzeroSafe provides military-grade wallet security with simple integration - just 3 lines of code to add a complete interactive wallet management system to your bot.

### Key Benefits

- **🔐 Military-Grade Security**: AES-256 encryption with PBKDF2 key derivation
- **🚀 Simple Integration**: 3 lines of code for complete wallet management
- **🎯 Interactive CLI**: Built-in commands for all wallet operations
- **💰 Solana Ready**: Native support for SOL, WSOL, SPL tokens, and durable nonce
- **🔒 Secure by Default**: Password via stdin pipe (memory only, never environment variables)

## Integration Steps

### Step 1: Add Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
fnzero-safe = { path = "../fnzero-safe" }

[features]
default = ["solana-ops"]
solana-ops = ["fnzero-safe/solana-ops"]
```

### Step 2: Add Safekey Command

Add this code to your bot's `main()` function **before** your bot logic:

```rust
use anyhow::Result;

fn main() -> Result<()> {
    // Check if running in safekey interactive mode
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("safekey") {
        // Launch fnzero-safe interactive menu
        if let Err(e) = fnzero_safe::interactive::show_main_menu() {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Your bot logic starts here...
    println!("🤖 Starting bot...");

    Ok(())
}
```

That's it! Just 3 lines of actual integration code.

### Step 3: Build Your Bot

```bash
cargo build --features solana-ops --release
```

## Using the Safekey Command

After integration, users can run:

```bash
./your-bot safekey
```

This launches the full interactive menu with all wallet operations:

### Available Operations

**Wallet Management:**
- Create plain text keypair
- Create encrypted keypair (recommended)
- Decrypt encrypted keypair
- Unlock wallet for session

**Solana Operations:**
- Query SOL balance
- Transfer SOL
- Wrap SOL → WSOL
- Unwrap WSOL → SOL
- Transfer SPL tokens
- Create durable nonce accounts
- **PumpSwap Sell** - One-click token selling on PumpSwap DEX
- **Pump.fun Sell** - One-click token selling on Pump.fun bonding curve (internal market)

## Security Implementation

### Password Handling

FnzeroSafe uses a secure password handling model:

**✅ Secure Approach:**
- Password passed via stdin pipe
- Exists only in memory
- Never stored in files or environment variables
- Immediately cleared after use

**❌ Insecure (Never Do This):**
```bash
# DON'T: Environment variables can be leaked
export WALLET_PASSWORD="mysecret"
./your-bot
```

**✅ Secure (Always Do This):**
```bash
# Password through stdin pipe - memory only
echo "your-password" | ./your-bot
```

### Startup Script Example

Create a secure startup script for your bot:

```bash
#!/bin/bash

# Build the bot
echo "🔧 Building bot..."
cargo build --features solana-ops --release

# Get password securely (no echo)
echo -n "🔐 Enter wallet password: "
read -s WALLET_PASSWORD
echo ""

# Start bot with password piped through stdin
echo "$WALLET_PASSWORD" | ./build-cache/release/your-bot > bot.log 2>&1
EXIT_CODE=$?

# Immediately clear password from memory
WALLET_PASSWORD=""
unset WALLET_PASSWORD

# Check execution result
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Bot completed successfully"
else
    echo "❌ Bot failed with exit code: $EXIT_CODE"
    echo "📝 Check bot.log for details"
fi
```

## Bot Logic Integration

### Loading Encrypted Wallet

```rust
use fnzero_safe::KeyManager;
use std::io::{self, Read};

fn load_wallet() -> Result<solana_sdk::signature::Keypair> {
    let wallet_path = "keystore.json";

    // Read encrypted keystore
    let json = std::fs::read_to_string(wallet_path)?;

    // Read password from stdin
    let mut password = String::new();
    io::stdin().read_to_string(&mut password)?;
    let password = password.trim();

    // Decrypt and load keypair
    let keypair = KeyManager::keypair_from_encrypted_json(&json, password)?;

    Ok(keypair)
}
```

### Creating New Wallet

```rust
use fnzero_safe::KeyManager;

fn create_wallet(password: &str) -> Result<()> {
    // Generate new keypair
    let keypair = KeyManager::generate_keypair();

    println!("📍 Wallet Address: {}", keypair.pubkey());

    // Encrypt and save
    let json = KeyManager::keypair_to_encrypted_json(&keypair, password)?;
    std::fs::write("keystore.json", json)?;

    println!("✅ Encrypted wallet saved to keystore.json");

    Ok(())
}
```

### Using Solana Operations

```rust
use fnzero_safe::solana_ops::SolanaClient;

fn bot_logic(keypair: &solana_sdk::signature::Keypair) -> Result<()> {
    // Initialize Solana client
    let client = SolanaClient::new("https://api.devnet.solana.com")?;

    // Check balance
    let balance = client.get_sol_balance(&keypair.pubkey())?;
    println!("💰 Balance: {} SOL", balance);

    // Transfer SOL
    if balance > 0.01 {
        let recipient = "RECIPIENT_ADDRESS_HERE".parse()?;
        let signature = client.transfer_sol(keypair, &recipient, 0.01)?;
        println!("✅ Transfer successful: {}", signature);
    }

    // Wrap SOL to WSOL
    let signature = client.wrap_sol(keypair, 0.1)?;
    println!("✅ Wrapped 0.1 SOL: {}", signature);

    Ok(())
}
```

## Complete Bot Example

See `examples/bot_example.rs` for a full working example that demonstrates:

- Safekey command integration
- Secure password handling via stdin
- Encrypted wallet loading
- All Solana operations
- Proper error handling
- Production-ready patterns

Build and run:

```bash
# Build the example
cargo build --example bot_example --features solana-ops --release

# Launch interactive safekey commands
./build-cache/release/examples/bot_example safekey

# Run bot with password from stdin
echo "your-password" | ./build-cache/release/examples/bot_example
```

## Key Features Summary

FnzeroSafe provides comprehensive wallet management:

| Feature | Support |
|---------|---------|
| Safekey command | ✅ `./your-bot safekey` |
| Interactive menu | ✅ Full featured |
| Wallet creation | ✅ AES-256 encryption |
| Password security | ✅ stdin pipe (memory only) |
| SOL operations | ✅ Transfer, balance, wrap/unwrap |
| Token support | ✅ SPL tokens, Token-2022 |
| PumpSwap DEX | ✅ One-click sell with seed-optimized ATA |
| Pump.fun (bonding curve) | ✅ One-click sell on internal market |
| Durable nonce | ✅ Offline transaction support |
| Integration effort | 🎯 3 lines of code |

## Testing Your Integration

### On Devnet

1. Create test wallet:
```bash
./your-bot safekey
# Select: Create encrypted keypair → Save to keystore.json
```

2. Get devnet SOL:
```bash
solana airdrop 2 YOUR_WALLET_ADDRESS --url devnet
```

3. Test operations:
```bash
./your-bot safekey
# Select: Unlock wallet → Query balance → Transfer SOL
```

### Integration Checklist

- [ ] Added fnzero-safe dependency to Cargo.toml
- [ ] Added 3-line safekey command check to main()
- [ ] Created secure startup script with stdin password
- [ ] Tested wallet creation with safekey command
- [ ] Tested wallet loading in bot logic
- [ ] Verified password never in environment variables
- [ ] Tested on devnet before production
- [ ] Backed up keystore.json securely

## Best Practices

### Security

1. **Never** store passwords in:
   - Environment variables
   - Configuration files
   - Source code
   - Log files

2. **Always** use:
   - Stdin pipe for password input
   - Encrypted keystore files (AES-256)
   - Strong passwords (16+ characters)
   - Secure backup locations

3. **Production Checklist**:
   - [ ] Test thoroughly on devnet
   - [ ] Backup keystore.json securely
   - [ ] Use hardware security module (HSM) for high-value accounts
   - [ ] Implement rate limiting for operations
   - [ ] Monitor for unusual activity
   - [ ] Keep dependencies updated

### Error Handling

```rust
use anyhow::{Context, Result};

fn robust_bot_logic() -> Result<()> {
    // Load wallet with context
    let keypair = load_wallet()
        .context("Failed to load wallet from keystore.json")?;

    // Initialize client with retry logic
    let client = SolanaClient::new_with_retry("https://api.mainnet-beta.solana.com")
        .context("Failed to connect to Solana network")?;

    // Perform operations with error handling
    match client.get_sol_balance(&keypair.pubkey()) {
        Ok(balance) => println!("Balance: {}", balance),
        Err(e) => eprintln!("Failed to get balance: {}", e),
    }

    Ok(())
}
```

### Performance Tips

1. **Connection Pooling**: Reuse SolanaClient instances
2. **Batch Operations**: Group multiple transactions
3. **Async Processing**: Use tokio for concurrent operations
4. **Caching**: Cache balance checks and account info
5. **Rate Limiting**: Respect RPC node limits

## Troubleshooting

### Common Issues

**Issue**: "Failed to decrypt keystore"
- **Cause**: Wrong password
- **Solution**: Verify password or create new wallet

**Issue**: "Connection refused"
- **Cause**: RPC node unreachable
- **Solution**: Check network, try different RPC endpoint

**Issue**: "Insufficient funds"
- **Cause**: Not enough SOL for transaction + fees
- **Solution**: Ensure balance covers amount + ~0.00001 SOL fee

**Issue**: "Transaction failed"
- **Cause**: Network congestion or invalid transaction
- **Solution**: Retry with higher priority fee or check transaction details

### Getting Help

- Check the [User Guide](USER_GUIDE.md) for detailed operation instructions
- Review `examples/bot_example.rs` for working code
- Check logs in `bot.log` for error details
- Review the integration steps above to ensure correct implementation

## Next Steps

1. ✅ Complete integration (3 lines of code)
2. ✅ Create test wallet on devnet
3. ✅ Test all operations via safekey command
4. ✅ Implement your bot logic
5. ✅ Test thoroughly on devnet
6. 🚀 Deploy to production

---

**Remember**: Security is paramount. Never compromise on password handling, always test on devnet first, and keep backups of your keystore files.
