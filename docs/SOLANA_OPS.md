# Solana Operations Guide

This guide explains how to use Sol-SafeKey's Solana operations features, including balance queries, transfers, and token operations.

> 📖 **中文文档**: [SOLANA_OPS_CN.md](./SOLANA_OPS_CN.md)

## 📋 Table of Contents

- [Feature Overview](#feature-overview)
- [CLI Usage](#cli-usage)
- [Library Integration](#library-integration)
- [Complete Examples](#complete-examples)
- [FAQ](#faq)

---

## Feature Overview

Sol-SafeKey provides the following Solana operations:

| Feature | Description | CLI Command | Library Method |
|---------|-------------|-------------|----------------|
| Check SOL Balance | Query wallet SOL balance | `balance` | `get_sol_balance()` |
| Check Token Balance | Query SPL Token balance | `token-balance` | `get_token_balance()` |
| Transfer SOL | Transfer SOL to another address | `transfer` | `transfer_sol()` |
| Transfer Token | Transfer SPL Tokens | `transfer-token` | `transfer_token()` |
| SOL → WSOL | Wrap SOL as WSOL | `wrap-sol` | `wrap_sol()` |
| WSOL → SOL | Unwrap WSOL to SOL | `unwrap-sol` | `unwrap_sol()` |

---

## CLI Usage

### Prerequisites

Install the full CLI:

```bash
cargo install sol-safekey --features full
```

Or build from source:

```bash
cargo build --release --features full
```

### Basic Usage

All Solana operations commands follow this format:

```bash
sol-safekey sol-ops -f <encrypted-wallet-file> <subcommand> [options]
```

### 1. Check SOL Balance

```bash
# Check encrypted wallet balance
sol-safekey sol-ops -f wallet.json balance

# Check specific address balance
sol-safekey sol-ops -f wallet.json balance -a <WALLET_ADDRESS>

# Use custom RPC
sol-safekey sol-ops -f wallet.json balance -r https://api.devnet.solana.com
```

**Example Output:**
```
🔐 Loading encrypted keypair...
Enter password: ********
✅ Keypair loaded successfully!
Public key: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu

📊 Checking SOL balance...

Balance Information:
Address: HUZjZSuyw2cPdqgGz7nY6hVbmhVL6SMHNv78TUktKogu
Balance: 1.5 SOL (1500000000 lamports)
```

### 2. Check Token Balance

```bash
# Check USDC balance
sol-safekey sol-ops -f wallet.json token-balance \
  -m EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v

# Check token balance for another address
sol-safekey sol-ops -f wallet.json token-balance \
  -m <TOKEN_MINT> \
  -a <WALLET_ADDRESS>
```

### 3. Transfer SOL

```bash
# Transfer 0.1 SOL
sol-safekey sol-ops -f wallet.json transfer \
  -t <RECIPIENT_ADDRESS> \
  -a 0.1

# Use devnet
sol-safekey sol-ops -f wallet.json transfer \
  -t <RECIPIENT_ADDRESS> \
  -a 0.1 \
  -r https://api.devnet.solana.com
```

### 4. Transfer Token

```bash
# Transfer 1000 tokens (smallest unit)
sol-safekey sol-ops -f wallet.json transfer-token \
  -m <TOKEN_MINT> \
  -t <RECIPIENT_ADDRESS> \
  -a 1000
```

### 5. Wrap SOL (SOL → WSOL)

```bash
# Wrap 0.5 SOL to WSOL
sol-safekey sol-ops -f wallet.json wrap-sol -a 0.5
```

### 6. Unwrap SOL (WSOL → SOL)

```bash
# Unwrap all WSOL to SOL
sol-safekey sol-ops -f wallet.json unwrap-sol
```

---

## Library Integration

### Installation

Enable the `solana-ops` feature in your `Cargo.toml`:

```toml
[dependencies]
sol-safekey = { version = "0.1.0", features = ["solana-ops"] }
solana-sdk = "3.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Basic Usage

```rust
use sol_safekey::solana_utils::{SolanaClient, lamports_to_sol};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create client
    let client = SolanaClient::new(
        "https://api.mainnet-beta.solana.com".to_string()
    );

    // Load keypair (simplified here)
    let keypair = Keypair::new();

    // Check balance
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    println!("Balance: {} SOL", lamports_to_sol(balance));

    Ok(())
}
```

### API Reference

#### `SolanaClient::new(rpc_url: String) -> Self`

Create a new Solana client.

**Parameters:**
- `rpc_url` - RPC node URL
  - Mainnet: `https://api.mainnet-beta.solana.com`
  - Devnet: `https://api.devnet.solana.com`
  - Testnet: `https://api.testnet.solana.com`

#### `async fn get_sol_balance(&self, pubkey: &Pubkey) -> Result<u64>`

Query SOL balance (in lamports).

**Returns:** Balance in lamports (1 SOL = 1,000,000,000 lamports)

**Example:**
```rust
let balance = client.get_sol_balance(&keypair.pubkey()).await?;
println!("Balance: {} SOL", lamports_to_sol(balance));
```

#### `async fn get_token_balance(&self, owner: &Pubkey, mint: &Pubkey) -> Result<u64>`

Query SPL Token balance.

**Parameters:**
- `owner` - Wallet address
- `mint` - Token mint address

**Returns:** Token balance (smallest unit)

#### `async fn transfer_sol(&self, from: &Keypair, to: &Pubkey, amount: u64) -> Result<Signature>`

Transfer SOL.

**Parameters:**
- `from` - Sender keypair
- `to` - Recipient address
- `amount` - Amount in lamports

**Returns:** Transaction signature

**Example:**
```rust
use solana_sdk::native_token::LAMPORTS_PER_SOL;

let recipient = Pubkey::from_str("...")?;
let amount = (0.1 * LAMPORTS_PER_SOL as f64) as u64; // 0.1 SOL

let signature = client.transfer_sol(&keypair, &recipient, amount).await?;
println!("Transaction: https://solscan.io/tx/{}", signature);
```

#### `async fn transfer_token(&self, from: &Keypair, to: &Pubkey, mint: &Pubkey, amount: u64) -> Result<Signature>`

Transfer SPL Tokens.

#### `async fn wrap_sol(&self, keypair: &Keypair, amount: u64) -> Result<Signature>`

Wrap SOL to WSOL.

#### `async fn unwrap_sol(&self, keypair: &Keypair) -> Result<Signature>`

Unwrap WSOL to SOL.

### Utility Functions

#### `lamports_to_sol(lamports: u64) -> f64`

Convert lamports to SOL.

```rust
let lamports = 1_500_000_000;
let sol = lamports_to_sol(lamports); // 1.5
```

#### `format_token_amount(amount: u64, decimals: u8) -> f64`

Convert token smallest units to human-readable format.

```rust
let usdc_amount = 1_000_000; // USDC smallest unit
let readable = format_token_amount(usdc_amount, 6); // 1.0 USDC
```

---

## Complete Examples

### Trading Bot Example

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::{fs, env};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config from environment
    dotenv::dotenv().ok();
    let wallet_path = env::var("WALLET_PATH")?;
    let password = env::var("WALLET_PASSWORD")?;
    let rpc_url = env::var("RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    // Load wallet
    let json = fs::read_to_string(&wallet_path)?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, &password)?;
    println!("🤖 Bot started with wallet: {}", keypair.pubkey());

    // Create client
    let client = SolanaClient::new(rpc_url);

    // Main loop
    loop {
        // Check balance
        let balance = client.get_sol_balance(&keypair.pubkey()).await?;
        println!("Current balance: {} SOL", lamports_to_sol(balance));

        // Your trading logic...

        // Wait for next check
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
```

### Batch Transfer Example

```rust
use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, pubkey::Pubkey, signer::Signer};
use std::{str::FromStr, fs};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load wallet
    let json = fs::read_to_string("wallet.json")?;
    let keypair = KeyManager::keypair_from_encrypted_json(&json, "password")?;

    // Create client
    let client = SolanaClient::new("https://api.mainnet-beta.solana.com".to_string());

    // Recipients list
    let recipients = vec![
        ("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", 0.1),
        ("7xVdF5G926cUy8EPjHmaT24yvAM3ZWbrrpZd8QvsVGjm", 0.2),
    ];

    println!("🚀 Starting batch transfer...");

    for (addr, amount) in recipients {
        let recipient = Pubkey::from_str(addr)?;
        let lamports = (amount * 1_000_000_000.0) as u64;

        println!("\nTransferring {} SOL to {}...", amount, addr);

        match client.transfer_sol(&keypair, &recipient, lamports).await {
            Ok(signature) => {
                println!("✅ Success! Signature: {}", signature);
            }
            Err(e) => {
                eprintln!("❌ Failed: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("\n🎉 Batch transfer complete!");
    Ok(())
}
```

---

## FAQ

### 1. How to choose RPC node?

**Public RPC (Free, Limited):**
- Mainnet: `https://api.mainnet-beta.solana.com`
- Devnet: `https://api.devnet.solana.com`

**Private RPC (Paid, Better Performance):**
- [QuickNode](https://www.quicknode.com/)
- [Alchemy](https://www.alchemy.com/)
- [Helius](https://www.helius.dev/)

### 2. lamports and SOL conversion

```
1 SOL = 1,000,000,000 lamports
0.1 SOL = 100,000,000 lamports
```

### 3. Token decimals

Different tokens have different decimal places:

| Token | Decimals | Example |
|-------|----------|---------|
| USDC | 6 | 1,000,000 = 1 USDC |
| SOL/WSOL | 9 | 1,000,000,000 = 1 SOL |
| Most SPL Tokens | 9 | - |

### 4. Transaction failures

Common reasons:
- ❌ Insufficient balance (including fees)
- ❌ RPC timeout
- ❌ Network congestion
- ❌ Token account doesn't exist

Solution with retry:
```rust
let mut retries = 3;
while retries > 0 {
    match client.transfer_sol(&keypair, &to, amount).await {
        Ok(sig) => return Ok(sig),
        Err(e) => {
            retries -= 1;
            if retries == 0 {
                return Err(e);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
}
```

### 5. Using testnet

```bash
# CLI - use devnet
sol-safekey sol-ops -f wallet.json balance \
  -r https://api.devnet.solana.com

# Code - use devnet
let client = SolanaClient::new(
    "https://api.devnet.solana.com".to_string()
);
```

Get test SOL:
```bash
solana airdrop 2 <YOUR_ADDRESS> --url devnet
```

### 6. Secure password storage

**Recommended approaches:**

1. **Environment variables** (for development)
```bash
export WALLET_PASSWORD="your_password"
```

2. **`.env` file** (don't commit to Git)
```
WALLET_PASSWORD=your_password
RPC_URL=https://api.mainnet-beta.solana.com
```

3. **Secret management services** (for production)
- AWS Secrets Manager
- HashiCorp Vault
- Google Secret Manager

---

## Related Documentation

- [Integration Guide](./INTEGRATION.md)
- [Complete API Documentation](https://docs.rs/sol-safekey)
- [CLI Usage Guide](../README.md)
- [Solana Official Docs](https://docs.solana.com/)
