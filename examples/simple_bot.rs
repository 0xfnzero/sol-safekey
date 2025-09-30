//! Simple Trading Bot Example
//!
//! This example shows a minimal trading bot that:
//! 1. Unlocks wallet at startup using CLI
//! 2. Runs bot logic with the unlocked wallet
//! 3. No need to implement password input - CLI handles it!
//!
//! Run:
//! ```bash
//! cargo run --example simple_bot
//! ```

use sol_safekey::bot_helper::BotKeyManager;
use sol_safekey::{Keypair, Signer};
use std::path::Path;

/// Simple Trading Bot
struct TradingBot {
    keypair: Keypair,
    name: String,
}

impl TradingBot {
    /// Initialize bot by unlocking wallet
    fn new(name: &str, keystore_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🤖 Initializing {} bot...", name);

        // Check if keystore exists
        if !Path::new(keystore_path).exists() {
            return Err(format!(
                "Keystore not found: {}\nRun 'sol-safekey gen-keystore -o {}' first",
                keystore_path, keystore_path
            )
            .into());
        }

        // Create manager
        let manager = BotKeyManager::new();

        // Show wallet info
        let public_key = manager.get_public_key_from_file(keystore_path)?;
        println!("📍 Bot wallet: {}", public_key);

        // Unlock keystore (user will input password via CLI)
        println!("\n🔓 Unlocking wallet...");
        let private_key = manager.unlock_keystore_interactive(keystore_path)?;

        // Create keypair (Solana 3.0 uses from_base58_string directly)
        let keypair = Keypair::from_base58_string(&private_key);

        println!("✅ Wallet unlocked successfully!\n");

        Ok(Self {
            keypair,
            name: name.to_string(),
        })
    }

    /// Start the bot
    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 {} started!", self.name);
        println!("📍 Wallet: {}", self.keypair.pubkey());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Simulate bot running
        self.check_balance()?;
        self.monitor_markets()?;
        self.execute_trades()?;

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ Bot operations completed");

        Ok(())
    }

    /// Check wallet balance (simulated)
    fn check_balance(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n💰 Checking balance...");
        println!("   Balance: 10.5 SOL");
        Ok(())
    }

    /// Monitor markets (simulated)
    fn monitor_markets(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Monitoring markets...");
        println!("   SOL/USDC: $150.23");
        println!("   RAY/USDC: $2.45");
        Ok(())
    }

    /// Execute trades (simulated)
    fn execute_trades(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n💹 Executing trade strategy...");

        // Sign a sample transaction
        let transaction_data = b"TRADE:BUY:SOL:1.0";
        let signature = self.keypair.sign_message(transaction_data);

        println!("   📝 Trade: Buy 1.0 SOL");
        println!("   ✍️  Signed: {}", signature);
        println!("   ✅ Trade executed");

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════════════════════");
    println!("           Simple Trading Bot Example");
    println!("════════════════════════════════════════════════════════\n");

    // Check CLI availability
    if !sol_safekey::bot_helper::check_cli() {
        eprintln!("❌ sol-safekey CLI not found!");
        eprintln!("   Please install: cargo install --path .");
        return Ok(());
    }

    // Setup keystore path
    let keystore_path = "trading_bot_wallet.json";

    // Generate keystore if it doesn't exist
    if !Path::new(keystore_path).exists() {
        println!("📦 First-time setup: Creating bot wallet...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let manager = BotKeyManager::new();
        let public_key = manager.generate_keystore_interactive(keystore_path)?;

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ Bot wallet created!");
        println!("📂 Keystore: {}", keystore_path);
        println!("🔑 Address: {}", public_key);
        println!("⚠️  Remember your password!\n");
    }

    // Initialize and start bot
    let bot = TradingBot::new("TradingBot v1.0", keystore_path)?;
    bot.start()?;

    println!("\n💡 Tips:");
    println!("   • The bot unlocked your wallet at startup");
    println!("   • Keystore file stays encrypted on disk");
    println!("   • You don't need to implement CLI yourself");
    println!("   • Just call BotKeyManager in your bot code!");

    Ok(())
}