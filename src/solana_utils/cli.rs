use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;

use crate::solana_utils::solana_ops::{lamports_to_sol, format_token_amount, SolanaClient};
#[cfg(feature = "solana-ops")]
use crate::solana_utils::solana_ops::SolanaClientSdk;
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
        /// Optional: Unwrap specific amount in SOL (partial unwrap; keep WSOL ATA open)
        #[arg(short, long)]
        amount: Option<f64>,
    },

    /// PumpSwap sell tokens (sell all balance)
    #[command(name = "pumpswap-sell")]
    PumpSwapSell {
        /// Token mint address to sell
        #[arg(short, long)]
        mint: String,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,

        /// Slippage tolerance in basis points (e.g., 100 = 1%, 9900 = 99%)
        #[arg(short, long, default_value = "9900")]
        slippage: u64,
    },

    /// Pump.fun bonding curve sell (sell all balance, receive native SOL)
    #[command(name = "pumpfun-sell")]
    PumpFunSell {
        /// Token mint address to sell
        #[arg(short, long)]
        mint: String,

        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,

        /// Slippage tolerance in basis points (e.g., 100 = 1%, 9900 = 99%)
        #[arg(short, long, default_value = "9900")]
        slippage: u64,
    },

    /// View and claim Pump (Pump.fun) cashback (native SOL)
    #[command(name = "pumpfun-cashback")]
    PumpFunCashback {
        /// RPC URL (defaults to mainnet)
        #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },

    /// View and claim PumpSwap cashback (WSOL)
    #[command(name = "pumpswap-cashback")]
    PumpSwapCashback {
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
pub fn execute_solana_ops(args: SolanaOpsArgs, encrypted_file: &str) -> Result<()> {
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
            let balance = client.get_sol_balance(&pubkey)?;
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
            let balance = client.get_token_balance(&owner, &mint_pubkey)?;

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
            let signature = client.transfer_sol(&keypair, &to_pubkey, lamports)?;

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
            let signature = client.transfer_token(&keypair, &to_pubkey, &mint_pubkey, amount)?;

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
            let signature = client.wrap_sol(&keypair, lamports)?;

            println!("\n{}", "✅ Wrap successful!".green().bold());
            println!("Signature: {}", signature.to_string().yellow());
            println!("Explorer: https://solscan.io/tx/{}", signature);
        }

        SolanaOpsCommand::UnwrapSol { rpc_url, amount } => {
            let keypair = load_encrypted_keypair(encrypted_file)?;

            if let Some(unwrap_amount) = amount {
                let lamports = (unwrap_amount * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;

                println!("\n{}", "🔄 Unwrapping partial WSOL to SOL...".cyan());
                println!("Amount: {} SOL ({} lamports)", unwrap_amount, lamports);
                println!("{}", "💡 WSOL 主账户将保持开启（仅解包指定金额）".yellow());

                // 二次确认
                print!("\n{}", "Confirm partial unwrap? (yes/no): ".yellow());
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;
                if confirm.trim().to_lowercase() != "yes" {
                    println!("{}", "Operation cancelled.".red());
                    return Ok(());
                }

                // 余额校验（避免因余额不足导致交易失败）
                #[cfg(feature = "solana-ops")]
                {
                    let client = SolanaClientSdk::new(rpc_url.clone(), false);
                    let wsol_balance = client.get_wsol_balance(&keypair.pubkey())?;
                    if wsol_balance < lamports {
                        return Err(anyhow::anyhow!(
                            "WSOL余额不足，当前: {} lamports，需要: {} lamports",
                            wsol_balance, lamports
                        ));
                    }

                    println!("\n{}", "🚀 Sending transaction...".cyan());
                    let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                    let signature = rt.block_on(client.unwrap_sol_partial(&keypair, lamports))?;

                    println!("\n{}", "✅ Partial unwrap successful!".green().bold());
                    println!("Signature: {}", signature.to_string().yellow());
                    println!("Explorer: https://solscan.io/tx/{}", signature);
                }

                #[cfg(not(feature = "solana-ops"))]
                {
                    return Err(anyhow::anyhow!(
                        "该功能需要启用 feature 'solana-ops' 才可使用指定金额解包"
                    ));
                }
            } else {
                println!("\n{}", "🔄 Unwrapping WSOL to SOL (close WSOL ATA)...".cyan());
                println!("{}", "⚠️ 注意：该操作会关闭 WSOL ATA，并将所有 SOL 余额返还至钱包，且不可逆。".bright_red().bold());

                print!("\n{}", "Confirm close WSOL ATA? (yes/no): ".yellow());
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;
                if confirm.trim().to_lowercase() != "yes" {
                    println!("{}", "Operation cancelled.".red());
                    return Ok(());
                }

                #[cfg(feature = "solana-ops")]
                {
                    let client = SolanaClientSdk::new(rpc_url.clone(), false);
                    println!("\n{}", "🚀 Sending transaction...".cyan());
                    let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                    let signature = rt.block_on(client.unwrap_sol(&keypair))?;

                    println!("\n{}", "✅ Unwrap successful!".green().bold());
                    println!("Signature: {}", signature.to_string().yellow());
                    println!("Explorer: https://solscan.io/tx/{}", signature);
                }

                #[cfg(not(feature = "solana-ops"))]
                {
                    let client = SolanaClient::new(rpc_url);
                    println!("\n{}", "🚀 Sending transaction...".cyan());
                    let signature = client.unwrap_sol(&keypair)?;

                    println!("\n{}", "✅ Unwrap successful!".green().bold());
                    println!("Signature: {}", signature.to_string().yellow());
                    println!("Explorer: https://solscan.io/tx/{}", signature);
                }
            }
        }

        SolanaOpsCommand::PumpSwapSell { mint, rpc_url, slippage } => {
            #[cfg(not(feature = "sol-trade-sdk"))]
            {
                return Err(anyhow::anyhow!(
                    "PumpSwap sell requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk"
                ));
            }

            #[cfg(feature = "sol-trade-sdk")]
            {
                use crate::solana_utils::pumpswap_sell::handle_pumpswap_sell;

                let keypair = load_encrypted_keypair(encrypted_file)?;
                let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                rt.block_on(handle_pumpswap_sell(&keypair, &mint, &rpc_url, slippage))?;
            }
        }

        SolanaOpsCommand::PumpFunSell { mint, rpc_url, slippage } => {
            #[cfg(not(feature = "sol-trade-sdk"))]
            {
                return Err(anyhow::anyhow!(
                    "Pump.fun sell requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk"
                ));
            }

            #[cfg(feature = "sol-trade-sdk")]
            {
                use crate::solana_utils::pumpfun_sell::handle_pumpfun_sell;

                let keypair = load_encrypted_keypair(encrypted_file)?;
                let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                rt.block_on(handle_pumpfun_sell(&keypair, &mint, &rpc_url, slippage))?;
            }
        }

        SolanaOpsCommand::PumpFunCashback { rpc_url } => {
            #[cfg(not(feature = "sol-trade-sdk"))]
            {
                return Err(anyhow::anyhow!(
                    "Pump cashback requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk"
                ));
            }

            #[cfg(feature = "sol-trade-sdk")]
            {
                use sol_trade_sdk::{common::TradeConfig, SolanaTrade};
                use solana_commitment_config::CommitmentConfig;

                let keypair = load_encrypted_keypair(encrypted_file)?;
                let payer = std::sync::Arc::new(keypair.insecure_clone());
                let config = TradeConfig {
                    rpc_url: rpc_url.clone(),
                    swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url.clone())],
                    commitment: CommitmentConfig::confirmed(),
                    create_wsol_ata_on_startup: false,
                    use_seed_optimize: false,
                    check_min_tip: false,
                    log_enabled: false,
                    swqos_cores_from_end: false,
                };
                let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                let client = rt.block_on(SolanaTrade::new(payer, config));
                println!("\n{}", "💰 Claiming Pump (Pump.fun) cashback (native SOL)...".cyan());
                let sig = rt.block_on(client.claim_cashback_pumpfun())?;
                println!("\n{}", "✅ Claim successful!".green().bold());
                println!("Signature: {}", sig.yellow());
                println!("Explorer: https://solscan.io/tx/{}", sig);
            }
        }

        SolanaOpsCommand::PumpSwapCashback { rpc_url } => {
            #[cfg(not(feature = "sol-trade-sdk"))]
            {
                return Err(anyhow::anyhow!(
                    "PumpSwap cashback requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk"
                ));
            }

            #[cfg(feature = "sol-trade-sdk")]
            {
                use sol_trade_sdk::{common::TradeConfig, SolanaTrade};
                use solana_commitment_config::CommitmentConfig;

                let keypair = load_encrypted_keypair(encrypted_file)?;
                let payer = std::sync::Arc::new(keypair.insecure_clone());
                let config = TradeConfig {
                    rpc_url: rpc_url.clone(),
                    swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url.clone())],
                    commitment: CommitmentConfig::confirmed(),
                    create_wsol_ata_on_startup: false,
                    use_seed_optimize: false,
                    check_min_tip: false,
                    log_enabled: false,
                    swqos_cores_from_end: false,
                };
                let rt = tokio::runtime::Runtime::new().map_err(|e| anyhow::anyhow!(e))?;
                let client = rt.block_on(SolanaTrade::new(payer, config));
                println!("\n{}", "💰 Claiming PumpSwap cashback (WSOL)...".cyan());
                let sig = rt.block_on(client.claim_cashback_pumpswap())?;
                println!("\n{}", "✅ Claim successful!".green().bold());
                println!("Signature: {}", sig.yellow());
                println!("Explorer: https://solscan.io/tx/{}", sig);
            }
        }
    }

    Ok(())
}