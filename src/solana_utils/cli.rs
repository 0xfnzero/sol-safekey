use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;

use crate::solana_utils::solana_ops::{lamports_to_sol, format_token_amount, SolanaClient};
use crate::KeyManager;

#[derive(Parser)]
#[command(name = "sol-ops")]
#[command(about = "Solana operations with encrypted private key", long_about = None)]
pub struct SolanaOpsArgs {
    #[command(subcommand)]
    pub command: SolanaOpsCommand,
}

#[derive(Subcommand, Clone)]
pub enum SolanaOpsCommand {
    /// Get SOL balance
    Balance {
        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,

        /// Optional wallet address (if not provided, will use encrypted keypair)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Get SPL token balance
    TokenBalance {
        /// Token mint address
        #[arg(short, long)]
        mint: String,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,

        /// Optional wallet address (if not provided, will use encrypted keypair)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Transfer SOL
    Transfer {
        /// Recipient address
        #[arg(short, long)]
        to: String,

        /// Amount in SOL
        #[arg(short, long)]
        amount: f64,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },

    /// Transfer SPL tokens
    TransferToken {
        /// Token mint address
        #[arg(short, long)]
        mint: String,

        /// Recipient address
        #[arg(short, long)]
        to: String,

        /// Amount (in token's smallest unit)
        #[arg(short, long)]
        amount: u64,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },

    /// Wrap SOL to WSOL
    WrapSol {
        /// Amount in SOL
        #[arg(short, long)]
        amount: f64,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },

    /// Unwrap WSOL to SOL
    UnwrapSol {
        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },
}

/// Load keypair from encrypted file
pub fn load_encrypted_keypair(file_path: &str) -> Result<Keypair> {
    use std::fs;

    println!("{}", "🔐 Loading encrypted keypair...".cyan());

    // Read encrypted file
    let encrypted_data = fs::read_to_string(file_path)?;

    // Parse JSON to get encryption type
    let json: serde_json::Value = serde_json::from_str(&encrypted_data)?;

    let encryption_type = json["encryption_type"]
        .as_str()
        .unwrap_or("password_only");

    let keypair = match encryption_type {
        "password_only" => {
            // Simple password-based decryption
            let password = rpassword::prompt_password("Enter password: ")?;
            KeyManager::keypair_from_encrypted_json(&encrypted_data, &password)
                .map_err(|e| anyhow::anyhow!(e))?
        }
        "triple_factor_v1" => {
            // Triple-factor authentication
            load_triple_factor_keypair(&encrypted_data)?
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown encryption type: {}", encryption_type));
        }
    };

    println!("{}", "✅ Keypair loaded successfully!".green());
    println!("Public key: {}", keypair.pubkey().to_string().yellow());

    Ok(keypair)
}

/// Load keypair with triple-factor authentication
fn load_triple_factor_keypair(encrypted_data: &str) -> Result<Keypair> {
    use crate::hardware_fingerprint::HardwareFingerprint;

    println!("{}", "\n🔐 Triple-Factor Authentication Required".cyan().bold());

    // Get hardware fingerprint
    let hw_fp = HardwareFingerprint::collect()
        .map_err(|e| anyhow::anyhow!("Failed to get hardware fingerprint: {}", e))?;

    // Get master password
    let master_password = rpassword::prompt_password("Enter master password: ")?;

    // Get security question answer
    let security_answer = rpassword::prompt_password("Enter security question answer: ")?;

    // Get 2FA code
    let twofa_code = rpassword::prompt_password("Enter 2FA code: ")?;

    // Decrypt
    let (private_key, _, _) = crate::decrypt_with_triple_factor_and_2fa(
        encrypted_data,
        &hw_fp.fingerprint,
        &master_password,
        &security_answer,
        &twofa_code,
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    Ok(Keypair::from_base58_string(&private_key))
}

/// Execute Solana operations CLI
pub async fn execute_solana_ops(args: SolanaOpsArgs, encrypted_file: &str) -> Result<()> {
    match args.command {
        SolanaOpsCommand::Balance { rpc_url, address } => {
            let client = SolanaClient::new(rpc_url);

            let pubkey = if let Some(addr) = address {
                Pubkey::from_str(&addr)?
            } else {
                let keypair = load_encrypted_keypair(encrypted_file)?;
                keypair.pubkey()
            };

            println!("\n{}", "📊 Checking SOL balance...".cyan());
            let balance = client.get_sol_balance(&pubkey).await?;
            let sol_amount = lamports_to_sol(balance);

            println!("\n{}", "Balance Information:".green().bold());
            println!("Address: {}", pubkey.to_string().yellow());
            println!("Balance: {} SOL ({} lamports)", sol_amount.to_string().green(), balance);
        }

        SolanaOpsCommand::TokenBalance { mint, rpc_url, address } => {
            let client = SolanaClient::new(rpc_url);
            let mint_pubkey = Pubkey::from_str(&mint)?;

            let owner = if let Some(addr) = address {
                Pubkey::from_str(&addr)?
            } else {
                let keypair = load_encrypted_keypair(encrypted_file)?;
                keypair.pubkey()
            };

            println!("\n{}", "📊 Checking token balance...".cyan());
            let balance = client.get_token_balance(&owner, &mint_pubkey).await?;

            println!("\n{}", "Token Balance Information:".green().bold());
            println!("Address: {}", owner.to_string().yellow());
            println!("Token Mint: {}", mint.yellow());
            println!("Balance: {} (smallest units)", balance.to_string().green());
            println!("Balance (9 decimals): {}", format_token_amount(balance, 9));
        }

        SolanaOpsCommand::Transfer { to, amount, rpc_url } => {
            let keypair = load_encrypted_keypair(encrypted_file)?;
            let client = SolanaClient::new(rpc_url);
            let to_pubkey = Pubkey::from_str(&to)?;

            let lamports = (amount * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;

            println!("\n{}", "💸 Preparing SOL transfer...".cyan());
            println!("From: {}", keypair.pubkey().to_string().yellow());
            println!("To: {}", to.yellow());
            println!("Amount: {} SOL ({} lamports)", amount, lamports);

            // Confirm transfer
            print!("\n{}", "Confirm transfer? (yes/no): ".yellow());
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase() != "yes" {
                println!("{}", "Transfer cancelled.".red());
                return Ok(());
            }

            println!("\n{}", "🚀 Sending transaction...".cyan());
            let signature = client.transfer_sol(&keypair, &to_pubkey, lamports).await?;

            println!("\n{}", "✅ Transfer successful!".green().bold());
            println!("Signature: {}", signature.to_string().yellow());
            println!("Explorer: https://solscan.io/tx/{}", signature);
        }

        SolanaOpsCommand::TransferToken { mint, to, amount, rpc_url } => {
            let keypair = load_encrypted_keypair(encrypted_file)?;
            let client = SolanaClient::new(rpc_url);
            let to_pubkey = Pubkey::from_str(&to)?;
            let mint_pubkey = Pubkey::from_str(&mint)?;

            println!("\n{}", "💸 Preparing token transfer...".cyan());
            println!("From: {}", keypair.pubkey().to_string().yellow());
            println!("To: {}", to.yellow());
            println!("Token Mint: {}", mint.yellow());
            println!("Amount: {} (smallest units)", amount);

            // Confirm transfer
            print!("\n{}", "Confirm transfer? (yes/no): ".yellow());
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase() != "yes" {
                println!("{}", "Transfer cancelled.".red());
                return Ok(());
            }

            println!("\n{}", "🚀 Sending transaction...".cyan());
            let signature = client.transfer_token(&keypair, &to_pubkey, &mint_pubkey, amount).await?;

            println!("\n{}", "✅ Transfer successful!".green().bold());
            println!("Signature: {}", signature.to_string().yellow());
            println!("Explorer: https://solscan.io/tx/{}", signature);
        }

        SolanaOpsCommand::WrapSol { amount, rpc_url } => {
            let keypair = load_encrypted_keypair(encrypted_file)?;
            let client = SolanaClient::new(rpc_url);

            let lamports = (amount * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;

            println!("\n{}", "🔄 Wrapping SOL to WSOL...".cyan());
            println!("Amount: {} SOL ({} lamports)", amount, lamports);

            // Confirm
            print!("\n{}", "Confirm wrap? (yes/no): ".yellow());
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase() != "yes" {
                println!("{}", "Operation cancelled.".red());
                return Ok(());
            }

            println!("\n{}", "🚀 Sending transaction...".cyan());
            let signature = client.wrap_sol(&keypair, lamports).await?;

            println!("\n{}", "✅ Wrap successful!".green().bold());
            println!("Signature: {}", signature.to_string().yellow());
            println!("Explorer: https://solscan.io/tx/{}", signature);
        }

        SolanaOpsCommand::UnwrapSol { rpc_url } => {
            let keypair = load_encrypted_keypair(encrypted_file)?;
            let client = SolanaClient::new(rpc_url);

            println!("\n{}", "🔄 Unwrapping WSOL to SOL...".cyan());

            // Confirm
            print!("\n{}", "Confirm unwrap? (yes/no): ".yellow());
            use std::io::{self, Write};
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if confirm.trim().to_lowercase() != "yes" {
                println!("{}", "Operation cancelled.".red());
                return Ok(());
            }

            println!("\n{}", "🚀 Sending transaction...".cyan());
            let signature = client.unwrap_sol(&keypair).await?;

            println!("\n{}", "✅ Unwrap successful!".green().bold());
            println!("Signature: {}", signature.to_string().yellow());
            println!("Explorer: https://solscan.io/tx/{}", signature);
        }
    }

    Ok(())
}