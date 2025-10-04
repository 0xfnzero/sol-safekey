//! Bot Integration Example
//!
//! This example shows how to integrate sol-safekey into your bot with just 1-2 lines of code.
//!
//! Run with: cargo run --example bot_integration

use sol_safekey::bot_helper;
use solana_sdk::signer::Signer;

fn main() {
    println!("🤖 Bot Integration Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Get wallet path from config (or use default)
    let wallet_path = std::env::var("WALLET_PATH")
        .unwrap_or_else(|_| "bot_wallet.json".to_string());

    println!("📂 Wallet path: {}", wallet_path);
    println!();

    // ============================================================================
    // THIS IS THE ONLY LINE YOU NEED IN YOUR BOT!
    // ============================================================================
    let keypair = match bot_helper::ensure_wallet_ready(&wallet_path) {
        Ok(kp) => kp,
        Err(e) => {
            eprintln!("❌ Wallet setup failed: {}", e);
            std::process::exit(1);
        }
    };
    // ============================================================================

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Bot wallet ready!");
    println!("📍 Address: {}", keypair.pubkey());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("🎉 You can now use the keypair for:");
    println!("   - Signing transactions");
    println!("   - Checking balance");
    println!("   - Executing trades");
    println!("   - Any other bot operations");
    println!();

    println!("💡 The wallet file is saved at: {}", wallet_path);
    println!("   You only need to enter the password once at startup.");
}
