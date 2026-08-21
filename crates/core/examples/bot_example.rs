//! Complete Bot Integration Example
//!
//! This example demonstrates how to integrate fnzero-safe into a trading bot.
//! It shows pure code integration without CLI - just straightforward function calls.
//!
//! Run with:
//!   cargo build -p fnzero-safe-core --example bot_example --features solana-ops --release
//!   echo "your_password" | ./build-cache/release/examples/bot_example
//!
//! Or use the startup script:
//!   ./startup-example.sh

use anyhow::{Context, Result};
use colored::Colorize;
use fnzero_safe::{solana_utils::*, KeyManager};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::io::{self, BufRead, IsTerminal};

const DEFAULT_WALLET_PATH: &str = "keystore.json";
const RPC_URL: &str = "https://api.devnet.solana.com";

/// Read password from stdin (supports both interactive and piped input)
fn read_password_from_stdin() -> Result<String> {
    if io::stdin().is_terminal() {
        // Interactive mode - use rpassword for secure input
        print!("🔑 Enter wallet password: ");
        std::io::Write::flush(&mut std::io::stdout())?;
        let password = rpassword::read_password().context("Failed to read password")?;
        return Ok(password);
    }

    // Piped mode - read from stdin (for startup.sh)
    let stdin = io::stdin();
    let password = stdin
        .lock()
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No password provided via stdin"))?
        .context("Failed to read password from stdin")?
        .trim()
        .to_string();

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }

    Ok(password)
}

/// Ensure wallet is ready (create if doesn't exist, unlock if exists)
fn ensure_wallet(wallet_path: &str, password: &str) -> Result<Keypair> {
    if std::path::Path::new(wallet_path).exists() {
        println!("{} Wallet found: {}", "✅".green(), wallet_path);
        println!("{} Unlocking wallet...", "🔓".cyan());

        // Read and decrypt wallet
        let json = std::fs::read_to_string(wallet_path).context("Failed to read wallet file")?;

        let keypair = KeyManager::keypair_from_encrypted_json(&json, password)
            .map_err(|e| anyhow::anyhow!("Failed to unlock wallet: {}", e))?;

        println!("{} Wallet unlocked!", "✅".green());
        println!("📍 Address: {}", keypair.pubkey().to_string().yellow());

        Ok(keypair)
    } else {
        println!("{} Wallet not found: {}", "⚠️".yellow(), wallet_path);
        println!("{} Creating new encrypted wallet...", "📝".cyan());
        println!();

        // Generate new keypair
        let keypair = KeyManager::generate_keypair();
        println!("{} Generated new keypair", "🔑".cyan());
        println!("📍 Address: {}", keypair.pubkey().to_string().yellow());

        // Encrypt and save
        let json = KeyManager::keypair_to_encrypted_json(&keypair, password)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt wallet: {}", e))?;

        std::fs::write(wallet_path, json).context("Failed to save wallet file")?;

        println!("{} Wallet saved to: {}", "💾".green(), wallet_path);
        println!();

        println!(
            "{} IMPORTANT: Backup your wallet file and remember your password!",
            "⚠️".yellow()
        );
        println!("   Wallet file: {}", wallet_path);
        println!();

        Ok(keypair)
    }
}

/// Demo: Check SOL balance
fn check_balance(keypair: &Keypair, rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Checking Balance", "💰".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let client = SolanaClient::new(rpc_url.to_string());
    let balance_lamports = client.get_sol_balance(&keypair.pubkey())?;
    let balance_sol = lamports_to_sol(balance_lamports);

    println!("Address: {}", keypair.pubkey().to_string().yellow());
    println!(
        "Balance: {} SOL ({} lamports)",
        balance_sol.to_string().green().bold(),
        balance_lamports
    );

    if balance_lamports == 0 {
        println!();
        println!("{} Get devnet SOL:", "💡".yellow());
        println!("   solana airdrop 2 {} --url devnet", keypair.pubkey());
    }

    Ok(())
}

/// Demo: Transfer SOL (shows how to do it, but doesn't actually execute)
fn demo_transfer(keypair: &Keypair, rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Transfer SOL Demo", "📤".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let client = SolanaClient::new(rpc_url.to_string());
    let balance = client.get_sol_balance(&keypair.pubkey())?;

    if balance < 10_000_000 {
        // 0.01 SOL
        println!(
            "{} Insufficient balance for transfer demo (need 0.01 SOL)",
            "⚠️".yellow()
        );
        println!("   Current balance: {} SOL", lamports_to_sol(balance));
        println!("   Please fund your wallet on devnet:");
        println!("   → https://faucet.solana.com");
        println!("   → solana airdrop 1 {}", keypair.pubkey());
        return Ok(());
    }

    println!("{} Sufficient balance for demo", "✅".green());
    println!();
    println!("Example code to transfer 0.001 SOL:");
    println!("{}", "─────────────────────────────────────".dimmed());
    println!(
        "{}",
        "let recipient = Pubkey::from_str(\"<address>\")?;".yellow()
    );
    println!("{}", "let lamports = 1_000_000; // 0.001 SOL".yellow());
    println!(
        "{}",
        "let signature = client.transfer_sol(keypair, &recipient, lamports)?;".yellow()
    );
    println!("{}", "println!(\"Signature: {}\", signature);".yellow());
    println!("{}", "─────────────────────────────────────".dimmed());

    Ok(())
}

/// Demo: Wrap SOL to WSOL
fn demo_wrap_sol(keypair: &Keypair, rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Wrap SOL to WSOL Demo", "🎁".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let client = SolanaClient::new(rpc_url.to_string());
    let balance = client.get_sol_balance(&keypair.pubkey())?;

    if balance < 20_000_000 {
        // 0.02 SOL
        println!(
            "{} Insufficient balance for wrap demo (need 0.02 SOL)",
            "⚠️".yellow()
        );
        println!("   Current balance: {} SOL", lamports_to_sol(balance));
        return Ok(());
    }

    println!("{} Sufficient balance for demo", "✅".green());
    println!();
    println!("Example code to wrap 0.01 SOL to WSOL:");
    println!("{}", "─────────────────────────────────────".dimmed());
    println!("{}", "let lamports = 10_000_000; // 0.01 SOL".yellow());
    println!(
        "{}",
        "let signature = client.wrap_sol(keypair, lamports)?;".yellow()
    );
    println!(
        "{}",
        "println!(\"Wrapped SOL to WSOL: {}\", signature);".yellow()
    );
    println!("{}", "─────────────────────────────────────".dimmed());

    Ok(())
}

/// Demo: Unwrap WSOL to SOL
fn demo_unwrap_sol(_keypair: &Keypair, _rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Unwrap WSOL to SOL Demo", "🎁".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("Example code to unwrap all WSOL back to SOL:");
    println!("{}", "─────────────────────────────────────".dimmed());
    println!(
        "{}",
        "let signature = client.unwrap_sol(keypair)?;".yellow()
    );
    println!(
        "{}",
        "println!(\"Unwrapped WSOL to SOL: {}\", signature);".yellow()
    );
    println!("{}", "─────────────────────────────────────".dimmed());

    Ok(())
}

/// Demo: Create durable nonce account
fn demo_create_nonce(keypair: &Keypair, rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Durable Nonce Account Demo", "🔄".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let client = SolanaClient::new(rpc_url.to_string());
    let balance = client.get_sol_balance(&keypair.pubkey())?;

    // Nonce account requires ~0.002 SOL rent + transaction fee
    if balance < 5_000_000 {
        // 0.005 SOL
        println!(
            "{} Insufficient balance for nonce account (need 0.005 SOL)",
            "⚠️".yellow()
        );
        println!("   Current balance: {} SOL", lamports_to_sol(balance));
        return Ok(());
    }

    println!("{} Sufficient balance for demo", "✅".green());
    println!();
    println!("What is a durable nonce account?");
    println!("  - Allows offline transaction signing");
    println!("  - Useful for scheduled or delayed transactions");
    println!("  - Replaces recent_blockhash with durable nonce");
    println!();
    println!("Example code to create nonce account:");
    println!("{}", "─────────────────────────────────────".dimmed());
    println!(
        "{}",
        "let (nonce_pubkey, sig) = client.create_nonce_account(keypair)?;".yellow()
    );
    println!(
        "{}",
        "println!(\"Nonce account: {}\", nonce_pubkey);".yellow()
    );
    println!("{}", "println!(\"Signature: {}\", sig);".yellow());
    println!("{}", "─────────────────────────────────────".dimmed());

    Ok(())
}

/// Demo: Transfer SPL tokens
fn demo_transfer_token(_keypair: &Keypair, _rpc_url: &str) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Transfer SPL Token Demo", "🪙".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("Example code to transfer SPL tokens:");
    println!("{}", "─────────────────────────────────────".dimmed());
    println!(
        "{}",
        "let mint = Pubkey::from_str(\"<token_mint_address>\")?;".yellow()
    );
    println!(
        "{}",
        "let recipient = Pubkey::from_str(\"<recipient_address>\")?;".yellow()
    );
    println!(
        "{}",
        "let amount = 1000; // token amount (smallest units)".yellow()
    );
    println!(
        "{}",
        "let signature = client.transfer_token(keypair, &recipient, &mint, amount)?;".yellow()
    );
    println!(
        "{}",
        "println!(\"Token transfer: {}\", signature);".yellow()
    );
    println!("{}", "─────────────────────────────────────".dimmed());

    Ok(())
}

fn main() -> Result<()> {
    // 检查是否是 safekey 交互模式
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("safekey") {
        // 直接调用 fnzero-safe 的交互式菜单
        if let Err(e) = fnzero_safe::interactive::show_main_menu() {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║   Complete Bot Integration Example - fnzero-safe  ║");
    println!("╚════════════════════════════════════════════════════╝\n");

    // Configuration
    let wallet_path = DEFAULT_WALLET_PATH;
    let rpc_url = RPC_URL;

    println!("⚙️  Configuration:");
    println!("   Wallet: {}", wallet_path);
    println!("   RPC: {}", rpc_url);
    println!("   Network: devnet");
    println!();

    // Step 1: Read password from stdin
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Password Input", "🔐".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let password = read_password_from_stdin().context("Failed to read password")?;

    println!("{} Password received", "✅".green());
    println!();

    // Step 2: Ensure wallet is ready (create or unlock)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Wallet Setup", "🔑".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let keypair = ensure_wallet(wallet_path, &password)?;

    // Clear password from memory immediately
    drop(password);

    // Step 3: Run bot operations
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Bot Operations", "🤖".cyan());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check balance
    check_balance(&keypair, rpc_url)?;

    // Demo other operations
    demo_transfer(&keypair, rpc_url)?;
    demo_wrap_sol(&keypair, rpc_url)?;
    demo_unwrap_sol(&keypair, rpc_url)?;
    demo_transfer_token(&keypair, rpc_url)?;
    demo_create_nonce(&keypair, rpc_url)?;

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} Bot Integration Complete!", "✅".green());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("{} Your bot can now:", "🎉".cyan());
    println!("   ✓ Create and manage encrypted wallets");
    println!("   ✓ Check SOL and token balances");
    println!("   ✓ Transfer SOL and SPL tokens");
    println!("   ✓ Wrap/unwrap SOL to WSOL");
    println!("   ✓ Create durable nonce accounts");
    println!("   ✓ Sign and submit transactions");
    println!();
    println!("{} Integration Pattern:", "💡".cyan());
    println!("   • Password via stdin pipe (never environment variables)");
    println!("   • Wallet file encrypted with AES-256");
    println!("   • Use startup-example.sh for secure password handling");
    println!("   • All operations use synchronous Solana RPC client");
    println!();
    println!("{} Next Steps:", "📚".cyan());
    println!("   1. Customize the operations for your trading strategy");
    println!("   2. Add your bot logic (monitoring, decision making, execution)");
    println!("   3. Integrate with your price feeds and trading signals");
    println!("   4. Test on devnet before moving to mainnet");
    println!();
    println!("{} For production use:", "⚠️".yellow());
    println!("   • Always test on devnet first");
    println!("   • Keep your wallet file secure");
    println!("   • Never share your password");
    println!("   • Backup your keystore.json file");
    println!();

    Ok(())
}
