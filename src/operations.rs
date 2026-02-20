//! Solana Operations Interactive Menu
//!
//! Provides interactive Solana operations using encrypted keystore
//! 提供使用加密 keystore 的交互式 Solana 操作

use std::io::{self, Write};
use colored::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::bs58;
use std::str::FromStr;

#[cfg(feature = "solana-ops")]
use crate::solana_utils::{SolanaClient, SolanaClientSdk, lamports_to_sol};

#[cfg(feature = "solana-ops")]
use solana_client::rpc_client::RpcClient;

const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

/// Language for UI
#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Chinese,
}

/// Read user input with default value
fn read_input(prompt: &str, default: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim();

    if input.is_empty() && !default.is_empty() {
        default.to_string()
    } else {
        input.to_string()
    }
}

/// Show Solana operations menu
#[cfg(feature = "solana-ops")]
pub fn show_operations_menu(keypair: &Keypair, language: Language) -> Result<(), String> {
    loop {
        println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
        if language == Language::English {
            println!("  {} - Solana Operations", "🔧 Sol-SafeKey".bright_yellow().bold());
        } else {
            println!("  {} - Solana 操作", "🔧 Sol-SafeKey".bright_yellow().bold());
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

        println!("\n{}", if language == Language::English {
            "Current Wallet:"
        } else {
            "当前钱包:"
        }.bright_green());
        println!("  📍 {}", keypair.pubkey().to_string().bright_white());

        println!("\n{}", if language == Language::English {
            "Available Operations:"
        } else {
            "可用操作:"
        }.bright_green());

        if language == Language::English {
            println!("  {}  Check SOL Balance", "1.".bright_cyan());
            println!("  {}  Transfer SOL", "2.".bright_cyan());
            println!("  {}  Create WSOL ATA", "3.".bright_cyan());
            println!("  {}  Wrap SOL → WSOL", "4.".bright_cyan());
            println!("  {}  Unwrap WSOL → SOL", "5.".bright_cyan());
            println!("  {}  Transfer SPL Token", "6.".bright_cyan());
            println!("  {}  Create Nonce Account", "7.".bright_cyan());
            println!("  {}  Back to Main Menu", "0.".bright_cyan());
        } else {
            println!("  {}  查询 SOL 余额", "1.".bright_cyan());
            println!("  {}  转账 SOL", "2.".bright_cyan());
            println!("  {}  创建 WSOL ATA 账户", "3.".bright_cyan());
            println!("  {}  包装 SOL → WSOL", "4.".bright_cyan());
            println!("  {}  解包 WSOL → SOL", "5.".bright_cyan());
            println!("  {}  转账 SPL 代币", "6.".bright_cyan());
            println!("  {}  创建 Nonce 账户", "7.".bright_cyan());
            println!("  {}  返回主菜单", "0.".bright_cyan());
        }

        let prompt = if language == Language::English {
            "\nSelect option [0-7]: "
        } else {
            "\n请输入选项 [0-7]: "
        };

        let choice = read_input(prompt, "");

        match choice.as_str() {
            "1" => check_balance(keypair, language)?,
            "2" => transfer_sol(keypair, language)?,
            "3" => create_wsol_ata(keypair, language)?,
            "4" => wrap_sol(keypair, language)?,
            "5" => unwrap_sol(keypair, language)?,
            "6" => transfer_token(keypair, language)?,
            "7" => create_nonce_account(keypair, language)?,
            "0" => {
                if language == Language::English {
                    println!("\n{}", "Returning to main menu...".bright_green());
                } else {
                    println!("\n{}", "返回主菜单...".bright_green());
                }
                return Ok(());
            }
            _ => {
                let msg = if language == Language::English {
                    "❌ Invalid option, please try again"
                } else {
                    "❌ 无效选项，请重试"
                };
                println!("\n{}", msg.red());
            }
        }
    }
}

/// Placeholder for non-solana-ops builds
#[cfg(not(feature = "solana-ops"))]
pub fn show_operations_menu(_keypair: &Keypair, language: Language) -> Result<(), String> {
    let msg = if language == Language::English {
        "❌ Solana operations require the 'solana-ops' feature. Please rebuild with: cargo build --features solana-ops"
    } else {
        "❌ Solana 操作需要 'solana-ops' 功能。请使用以下命令重新编译: cargo build --features solana-ops"
    };
    Err(msg.to_string())
}

#[cfg(feature = "solana-ops")]
pub fn check_balance(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "📊 Check SOL Balance".bright_yellow().bold());
    } else {
        println!("  {}", "📊 查询 SOL 余额".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    if language == Language::English {
        println!("\n🔍 Checking balance on {}...", if network == "2" { "Devnet" } else { "Mainnet" });
    } else {
        println!("\n🔍 正在查询{}余额...", if network == "2" { "测试网" } else { "主网" });
    }

    let client = SolanaClient::new(rpc_url.to_string());
    match client.get_sol_balance(&keypair.pubkey()) {
        Ok(balance) => {
            let sol = lamports_to_sol(balance);
            println!("\n{}", "✅ Balance:".bright_green());
            println!("  💰 {} SOL", sol.to_string().bright_white().bold());
            println!("  📊 {} lamports", balance.to_string().bright_white());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Failed to fetch balance: {}", e)
            } else {
                format!("❌ 查询余额失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
pub fn transfer_sol(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "💸 Transfer SOL".bright_yellow().bold());
    } else {
        println!("  {}", "💸 转账 SOL".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    let recipient_prompt = if language == Language::English {
        "\nRecipient address: "
    } else {
        "\n接收地址: "
    };
    let recipient_str = read_input(recipient_prompt, "");

    let recipient = Pubkey::from_str(&recipient_str)
        .map_err(|_| if language == Language::English {
            "❌ Invalid recipient address".to_string()
        } else {
            "❌ 无效的接收地址".to_string()
        })?;

    let amount_prompt = if language == Language::English {
        "Amount (SOL): "
    } else {
        "金额 (SOL): "
    };
    let amount_str = read_input(amount_prompt, "");
    let amount_sol: f64 = amount_str.parse()
        .map_err(|_| if language == Language::English {
            "❌ Invalid amount".to_string()
        } else {
            "❌ 无效的金额".to_string()
        })?;

    let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

    println!("\n{}", "📋 Transaction Summary:".bright_yellow());
    println!("  From: {}", keypair.pubkey().to_string().bright_white());
    println!("  To: {}", recipient.to_string().bright_white());
    println!("  Amount: {} SOL", amount_sol.to_string().bright_white().bold());

    let confirm_prompt = if language == Language::English {
        "\nConfirm transaction? (yes/no) [no]: "
    } else {
        "\n确认交易? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Transaction cancelled"
        } else {
            "❌ 交易已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Sending transaction...");
    } else {
        println!("\n🚀 正在发送交易...");
    }

    let client = SolanaClient::new(rpc_url.to_string());
    match client.transfer_sol(keypair, &recipient, amount_lamports) {
        Ok(signature) => {
            println!("\n{}", "✅ Transfer successful!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Transfer failed: {}", e)
            } else {
                format!("❌ 转账失败: {}", e)
            };
            Err(msg)
        }
    }
}

/// Helper: Run async code, handling both sync and async contexts
#[cfg(feature = "solana-ops")]
fn run_async<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    // Try to get current runtime handle
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            // We're already in a runtime, use block_in_place
            tokio::task::block_in_place(|| handle.block_on(future))
        }
        Err(_) => {
            // No runtime exists, create a new one
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(future)
        }
    }
}

/// Helper: Calculate and print WSOL ATA address
#[cfg(feature = "solana-ops")]
fn print_wsol_ata_address(owner: &Pubkey, language: Language, _use_seed: bool) {
    use sol_trade_sdk::common::fast_fn::{
        get_associated_token_address_with_program_id_fast,
    };

    let wsol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    // SDK 强制对 WSOL 使用标准 ATA（不支持 seed 优化）
    let ata = get_associated_token_address_with_program_id_fast(owner, &wsol_mint, &token_program);

    println!("\n{}", if language == Language::English {
        "📍 WSOL ATA Address:"
    } else {
        "📍 WSOL ATA 地址:"
    }.bright_yellow());
    println!("  {}", ata.to_string().bright_white().bold());
}

#[cfg(feature = "solana-ops")]
pub fn create_wsol_ata(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "🏦 Create WSOL ATA Account".bright_yellow().bold());
    } else {
        println!("  {}", "🏦 创建 WSOL ATA 账户".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    // WSOL 强制使用标准 ATA（SDK 设计）
    let use_seed_optimize = false;

    // 打印WSOL ATA地址
    print_wsol_ata_address(&keypair.pubkey(), language, use_seed_optimize);

    // 检查账号是否已存在
    if language == Language::English {
        println!("\n🔍 Checking if account already exists...");
    } else {
        println!("\n🔍 检查账号是否已存在...");
    }

    let client_sdk = SolanaClientSdk::new(rpc_url.to_string(), use_seed_optimize);
    let wsol_balance = client_sdk.get_wsol_balance(&keypair.pubkey())
        .unwrap_or(0);

    // 如果能查到余额（即使是0），说明账号已存在
    let rpc_client = RpcClient::new(rpc_url.to_string());
    let wsol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let ata = {
        use sol_trade_sdk::common::fast_fn::get_associated_token_address_with_program_id_fast;
        get_associated_token_address_with_program_id_fast(&keypair.pubkey(), &wsol_mint, &token_program)
    };

    let account_exists = rpc_client.get_account(&ata).is_ok();

    if account_exists {
        println!("\n{}", "ℹ️  WSOL ATA account already exists!".bright_yellow().bold());
        println!("  💰 Balance: {} lamports ({} SOL)",
            wsol_balance.to_string().bright_white(),
            lamports_to_sol(wsol_balance).to_string().bright_white());

        if language == Language::English {
            println!("\n✅ Account is ready to use. No need to create.");
        } else {
            println!("\n✅ 账号已准备就绪，无需创建。");
        }
        return Ok(());
    }

    println!("\n{}", if language == Language::English {
        "ℹ️  Account does not exist. Creating new WSOL ATA..."
    } else {
        "ℹ️  账号不存在，将创建新的 WSOL ATA..."
    }.bright_yellow());

    let confirm_prompt = if language == Language::English {
        "\nConfirm operation? (yes/no) [no]: "
    } else {
        "\n确认操作? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Operation cancelled"
        } else {
            "❌ 操作已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Creating WSOL ATA...");
    } else {
        println!("\n🚀 正在创建 WSOL ATA...");
    }

    // 使用run_async执行异步操作
    match run_async(client_sdk.create_wsol_ata(keypair)) {
        Ok(signature) => {
            println!("\n{}", "✅ WSOL ATA created successfully!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Creation failed: {}", e)
            } else {
                format!("❌ 创建失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
pub fn wrap_sol(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "🔄 Wrap SOL → WSOL".bright_yellow().bold());
    } else {
        println!("  {}", "🔄 包装 SOL → WSOL".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    // WSOL 强制使用标准 ATA（SDK 设计）
    let use_seed_optimize = false;

    // 打印WSOL ATA地址
    print_wsol_ata_address(&keypair.pubkey(), language, use_seed_optimize);

    let amount_prompt = if language == Language::English {
        "\nAmount to wrap (SOL): "
    } else {
        "\n包装金额 (SOL): "
    };
    let amount_str = read_input(amount_prompt, "");
    let amount_sol: f64 = amount_str.parse()
        .map_err(|_| if language == Language::English {
            "❌ Invalid amount".to_string()
        } else {
            "❌ 无效的金额".to_string()
        })?;

    let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;

    println!("\n{}", "📋 Operation Summary:".bright_yellow());
    println!("  Wrap: {} SOL → WSOL", amount_sol.to_string().bright_white().bold());

    let confirm_prompt = if language == Language::English {
        "\nConfirm operation? (yes/no) [no]: "
    } else {
        "\n确认操作? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Operation cancelled"
        } else {
            "❌ 操作已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Wrapping SOL...");
    } else {
        println!("\n🚀 正在包装 SOL...");
    }

    // 使用SolanaClientSdk调用sol-trade-sdk
    let client = SolanaClientSdk::new(rpc_url.to_string(), use_seed_optimize);

    // 使用run_async执行异步操作
    match run_async(client.wrap_sol(keypair, amount_lamports)) {
        Ok(signature) => {
            println!("\n{}", "✅ Wrap successful!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Wrap failed: {}", e)
            } else {
                format!("❌ 包装失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
pub fn unwrap_sol(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "🔄 Unwrap WSOL → SOL".bright_yellow().bold());
    } else {
        println!("  {}", "🔄 解包 WSOL → SOL".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    // WSOL 强制使用标准 ATA（SDK 设计）
    let use_seed_optimize = false;

    // 打印WSOL ATA地址
    print_wsol_ata_address(&keypair.pubkey(), language, use_seed_optimize);

    // 询问是否指定金额
    let amount_prompt = if language == Language::English {
        "\nUnwrap amount (in SOL, leave empty for ALL): "
    } else {
        "\n解包金额（单位: SOL，留空则解包全部）: "
    };
    let amount_input = read_input(amount_prompt, "");
    let amount_input = amount_input.trim();

    let (is_partial, amount_lamports) = if amount_input.is_empty() {
        // 解包全部
        println!("\n{}", if language == Language::English {
            "ℹ️  Will unwrap ALL WSOL back to SOL"
        } else {
            "ℹ️  将解包所有 WSOL 回 SOL"
        }.bright_yellow());
        (false, 0)
    } else {
        // 解包指定金额
        let amount_sol: f64 = amount_input.parse()
            .map_err(|_| if language == Language::English {
                "❌ Invalid amount".to_string()
            } else {
                "❌ 无效的金额".to_string()
            })?;
        
        if amount_sol <= 0.0 {
            return Err(if language == Language::English {
                "❌ Amount must be greater than 0".to_string()
            } else {
                "❌ 金额必须大于 0".to_string()
            });
        }

        let lamports = (amount_sol * 1_000_000_000.0) as u64;
        println!("\n{}", if language == Language::English {
            format!("ℹ️  Will unwrap {} SOL from WSOL", amount_sol)
        } else {
            format!("ℹ️  将从 WSOL 解包 {} SOL", amount_sol)
        }.bright_yellow());
        (true, lamports)
    };

    let confirm_prompt = if language == Language::English {
        "\nConfirm operation? (yes/no) [no]: "
    } else {
        "\n确认操作? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Operation cancelled"
        } else {
            "❌ 操作已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Unwrapping WSOL...");
    } else {
        println!("\n🚀 正在解包 WSOL...");
    }

    // 使用SolanaClientSdk调用sol-trade-sdk
    let client = SolanaClientSdk::new(rpc_url.to_string(), use_seed_optimize);

    // 使用run_async执行异步操作
    let result = if is_partial {
        run_async(client.unwrap_sol_partial(keypair, amount_lamports))
    } else {
        run_async(client.unwrap_sol(keypair))
    };
    
    match result {
        Ok(signature) => {
            println!("\n{}", "✅ Unwrap successful!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Unwrap failed: {}", e)
            } else {
                format!("❌ 解包失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
/// Close WSOL ATA account and reclaim rent
pub fn close_wsol_ata(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "🗑️  Close WSOL ATA Account".bright_yellow().bold());
    } else {
        println!("  {}", "🗑️  关闭 WSOL ATA 账号".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    // WSOL 强制使用标准 ATA（SDK 设计）
    let use_seed_optimize = false;

    // 打印WSOL ATA地址
    print_wsol_ata_address(&keypair.pubkey(), language, use_seed_optimize);

    println!("\n{}", if language == Language::English {
        "ℹ️  This operation will:"
    } else {
        "ℹ️  此操作将:"
    }.bright_yellow().bold());
    
    if language == Language::English {
        println!("     • Unwrap ALL WSOL back to SOL automatically");
        println!("     • Close the WSOL ATA account");
        println!("     • Reclaim rent (~0.00203928 SOL) to your wallet");
    } else {
        println!("     • 自动将所有 WSOL 解包回 SOL");
        println!("     • 关闭 WSOL ATA 账号");
        println!("     • 回收租金 (~0.00203928 SOL) 到您的钱包");
    }

    println!("\n{}", if language == Language::English {
        "💰 All SOL (unwrapped WSOL + rent) will be returned to your wallet!"
    } else {
        "�� 所有 SOL（解包的 WSOL + 租金）将返回到您的钱包!"
    }.green().bold());

    let confirm_prompt = if language == Language::English {
        "\nConfirm operation? (yes/no) [no]: "
    } else {
        "\n确认操作? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Operation cancelled"
        } else {
            "❌ 操作已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Closing WSOL ATA account...");
    } else {
        println!("\n🚀 正在关闭 WSOL ATA 账号...");
    }

    // 使用SolanaClientSdk调用sol-trade-sdk
    let client = SolanaClientSdk::new(rpc_url.to_string(), use_seed_optimize);

    // 使用run_async执行异步操作
    match run_async(client.unwrap_sol(keypair)) {
        Ok(signature) => {
            println!("\n{}", "✅ WSOL ATA closed successfully!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            println!("\n{}", if language == Language::English {
                "💰 Rent reclaimed to your wallet!"
            } else {
                "💰 租金已返还到您的钱包!"
            }.green());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Close failed: {}", e)
            } else {
                format!("❌ 关闭失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
pub fn transfer_token(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "💎 Transfer SPL Token".bright_yellow().bold());
    } else {
        println!("  {}", "💎 转账 SPL 代币".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    let mint_prompt = if language == Language::English {
        "\nToken Mint address: "
    } else {
        "\n代币 Mint 地址: "
    };
    let mint_str = read_input(mint_prompt, "");

    let mint = Pubkey::from_str(&mint_str)
        .map_err(|_| if language == Language::English {
            "❌ Invalid mint address".to_string()
        } else {
            "❌ 无效的 Mint 地址".to_string()
        })?;

    let recipient_prompt = if language == Language::English {
        "Recipient address: "
    } else {
        "接收地址: "
    };
    let recipient_str = read_input(recipient_prompt, "");

    let recipient = Pubkey::from_str(&recipient_str)
        .map_err(|_| if language == Language::English {
            "❌ Invalid recipient address".to_string()
        } else {
            "❌ 无效的接收地址".to_string()
        })?;

    let amount_prompt = if language == Language::English {
        "Amount (smallest units): "
    } else {
        "金额 (最小单位): "
    };
    let amount_str = read_input(amount_prompt, "");
    let amount: u64 = amount_str.parse()
        .map_err(|_| if language == Language::English {
            "❌ Invalid amount".to_string()
        } else {
            "❌ 无效的金额".to_string()
        })?;

    println!("\n{}", "📋 Transaction Summary:".bright_yellow());
    println!("  From: {}", keypair.pubkey().to_string().bright_white());
    println!("  To: {}", recipient.to_string().bright_white());
    println!("  Token: {}", mint.to_string().bright_white());
    println!("  Amount: {} (smallest units)", amount.to_string().bright_white().bold());

    let confirm_prompt = if language == Language::English {
        "\nConfirm transaction? (yes/no) [no]: "
    } else {
        "\n确认交易? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Transaction cancelled"
        } else {
            "❌ 交易已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Sending transaction...");
    } else {
        println!("\n🚀 正在发送交易...");
    }

    let client = SolanaClient::new(rpc_url.to_string());
    match client.transfer_token(keypair, &recipient, &mint, amount) {
        Ok(signature) => {
            println!("\n{}", "✅ Transfer successful!".bright_green().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/tx/{}?cluster=devnet", signature)
            } else {
                format!("https://explorer.solana.com/tx/{}", signature)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Transfer failed: {}", e)
            } else {
                format!("❌ 转账失败: {}", e)
            };
            Err(msg)
        }
    }
}

#[cfg(feature = "solana-ops")]
pub fn create_nonce_account(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    if language == Language::English {
        println!("  {}", "🔑 Create Nonce Account".bright_yellow().bold());
    } else {
        println!("  {}", "🔑 创建 Nonce 账户".bright_yellow().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());

    let network_prompt = if language == Language::English {
        "\nSelect network:\n  1. Mainnet\n  2. Devnet\nChoice [1]: "
    } else {
        "\n选择网络:\n  1. 主网 (Mainnet)\n  2. 测试网 (Devnet)\n选择 [1]: "
    };

    let network = read_input(network_prompt, "1");
    let rpc_url = if network == "2" { DEVNET_RPC_URL } else { DEFAULT_RPC_URL };

    println!("\n{}", if language == Language::English {
        "ℹ️  A nonce account will be created for durable transactions"
    } else {
        "ℹ️  将创建一个用于持久交易的 Nonce 账户"
    }.bright_yellow());

    println!("{}", if language == Language::English {
        "ℹ️  This requires ~0.00144 SOL for rent exemption"
    } else {
        "ℹ️  这需要约 0.00144 SOL 用于租金豁免"
    }.bright_yellow());

    let confirm_prompt = if language == Language::English {
        "\nConfirm creation? (yes/no) [no]: "
    } else {
        "\n确认创建? (yes/no) [no]: "
    };
    let confirm = read_input(confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Operation cancelled"
        } else {
            "❌ 操作已取消"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    if language == Language::English {
        println!("\n🚀 Creating nonce account...");
    } else {
        println!("\n🚀 正在创建 Nonce 账户...");
    }

    let client = SolanaClient::new(rpc_url.to_string());
    match client.create_nonce_account(keypair) {
        Ok((nonce_pubkey, signature)) => {
            println!("\n{}", "✅ Nonce account created successfully!".bright_green().bold());
            println!("  🔑 Nonce Account: {}", nonce_pubkey.to_string().bright_white().bold());
            println!("  📝 Signature: {}", signature.to_string().bright_white());
            let explorer_url = if network == "2" {
                format!("https://explorer.solana.com/address/{}?cluster=devnet", nonce_pubkey)
            } else {
                format!("https://explorer.solana.com/address/{}", nonce_pubkey)
            };
            println!("  🔗 Explorer: {}", explorer_url.bright_blue());
            println!("\n{}", if language == Language::English {
                "💡 Save this nonce account address for future use!"
            } else {
                "💡 请保存此 Nonce 账户地址以供将来使用！"
            }.bright_yellow());
            Ok(())
        }
        Err(e) => {
            let msg = if language == Language::English {
                format!("❌ Creation failed: {}", e)
            } else {
                format!("❌ 创建失败: {}", e)
            };
            Err(msg)
        }
    }
}

/// Entry point for Solana operations from interactive menu
/// Prompts for keystore file and password, then shows operations menu
pub fn show_solana_operations_menu(language: crate::interactive::Language) -> Result<(), String> {
    #[cfg(feature = "solana-ops")]
    {
        use crate::KeyManager;

        // Convert language from interactive module to operations module
        let ops_language = match language {
            crate::interactive::Language::English => Language::English,
            crate::interactive::Language::Chinese => Language::Chinese,
        };

        // Prompt for keystore file
        let file_prompt = if ops_language == Language::English {
            "Keystore file path (default: wallet.json): "
        } else {
            "Keystore 文件路径 (默认: wallet.json): "
        };

        let file_path = read_input(file_prompt, "wallet.json");

        // Check if file exists
        if !std::path::Path::new(&file_path).exists() {
            let err_msg = if ops_language == Language::English {
                format!("❌ File not found: {}", file_path)
            } else {
                format!("❌ 文件不存在: {}", file_path)
            };
            return Err(err_msg);
        }

        // Prompt for password
        let password_prompt = if ops_language == Language::English {
            "Enter password: "
        } else {
            "请输入密码: "
        };

        print!("{}", password_prompt);
        io::stdout().flush().map_err(|e| e.to_string())?;
        let password = rpassword::read_password()
            .map_err(|e| format!("Failed to read password: {}", e))?;

        // Load keystore
        let keystore_json = std::fs::read_to_string(&file_path)
            .map_err(|e| {
                if ops_language == Language::English {
                    format!("❌ Failed to read file: {}", e)
                } else {
                    format!("❌ 读取文件失败: {}", e)
                }
            })?;

        let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, &password)
            .map_err(|e| {
                if ops_language == Language::English {
                    format!("❌ Failed to decrypt keystore: {}", e)
                } else {
                    format!("❌ 解密 keystore 失败: {}", e)
                }
            })?;

        println!("\n{}", if ops_language == Language::English {
            "✅ Wallet unlocked successfully!"
        } else {
            "✅ 钱包解锁成功！"
        }.bright_green());

        // Run synchronous operations menu
        show_operations_menu(&keypair, ops_language)
    }

    #[cfg(not(feature = "solana-ops"))]
    {
        let _ = language; // Suppress unused variable warning
        Err("Solana operations require the 'solana-ops' feature".to_string())
    }
}

/// PumpSwap 交互式卖出
#[cfg(feature = "sol-trade-sdk")]
pub fn pumpswap_sell_interactive(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("  {}", "🔥 PumpSwap Sell Tokens".bright_magenta().bold());
    } else {
        println!("  {}", "🔥 PumpSwap 卖出代币".bright_magenta().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    println!("\n{}", if language == Language::English {
        "Current Wallet:"
    } else {
        "当前钱包:"
    }.bright_green());
    println!("  📍 {}", keypair.pubkey().to_string().bright_white());

    // Step 1: 输入 RPC URL（可选）
    println!();
    let rpc_prompt = if language == Language::English {
        format!("RPC URL (default: {}): ", DEFAULT_RPC_URL)
    } else {
        format!("RPC URL (默认: {}): ", DEFAULT_RPC_URL)
    };
    let rpc_url = read_input(&rpc_prompt, DEFAULT_RPC_URL);

    // Step 2: 询问是否使用 seed 优化
    println!();
    if language == Language::English {
        println!("{}", "🔧 Seed Optimization Configuration".bright_cyan());
        println!("   Seed optimization is used to create optimized ATA addresses");
        println!("   If your token ATA was created using the standard method, choose 'no'");
        println!("   If unsure, it's recommended to choose 'no'");
    } else {
        println!("{}", "🔧 Seed 优化配置".bright_cyan());
        println!("   Seed 优化用于创建优化的 ATA 地址，可以节省交易费用");
        println!("   如果你的代币 ATA 是通过标准方式创建的，请选择 'no'");
        println!("   如果不确定，建议选择 'yes'（默认）");
    }

    print!("\n{} ", if language == Language::English {
        "❓ Enable Seed Optimization? (yes/no, default: yes):"
    } else {
        "❓ 启用 Seed 优化? (yes/no, 默认 yes):"
    }.yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut seed_input = String::new();
    io::stdin().read_line(&mut seed_input).map_err(|e| e.to_string())?;
    let seed_input_trimmed = seed_input.trim().to_lowercase();
    // 默认为 yes：空输入或 yes/y 都启用，只有明确输入 no/n 才禁用
    let use_seed = seed_input_trimmed.is_empty() || seed_input_trimmed == "yes" || seed_input_trimmed == "y";

    if use_seed {
        println!("{}", if language == Language::English {
            "✅ Seed optimization enabled"
        } else {
            "✅ 已启用 Seed 优化"
        }.green());
    } else {
        println!("{}", if language == Language::English {
            "✅ Using standard ATA"
        } else {
            "✅ 使用标准 ATA"
        }.green());
    }

    // Step 3: 输入 token mint 地址（支持多个，用逗号或空格分割）
    println!();
    if language == Language::English {
        println!("{}", "💡 You can enter multiple mint addresses separated by commas or spaces".bright_cyan());
        println!("   Tokens will be sold in the order entered");
    } else {
        println!("{}", "💡 可以输入多个 Mint 地址，用逗号或空格分割".bright_cyan());
        println!("   将按输入顺序依次卖出");
    }

    let mint_prompt = if language == Language::English {
        "\nToken Mint Address(es): "
    } else {
        "\n代币 Mint 地址: "
    };
    print!("{}", mint_prompt.yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut mint_input = String::new();
    io::stdin().read_line(&mut mint_input).map_err(|e| e.to_string())?;
    let mint_input = mint_input.trim();

    if mint_input.is_empty() {
        return Err(if language == Language::English {
            "Token mint address cannot be empty".to_string()
        } else {
            "代币 Mint 地址不能为空".to_string()
        });
    }

    // 解析多个 mint 地址（支持逗号和空格分割）
    let mint_addresses: Vec<String> = mint_input
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if mint_addresses.is_empty() {
        return Err(if language == Language::English {
            "No valid mint addresses found".to_string()
        } else {
            "未找到有效的 Mint 地址".to_string()
        });
    }

    // 验证所有 mint 地址格式
    for (idx, mint) in mint_addresses.iter().enumerate() {
        if let Err(e) = Pubkey::from_str(mint) {
            return Err(if language == Language::English {
                format!("Invalid mint address #{}: {} (error: {})", idx + 1, mint, e)
            } else {
                format!("无效的 Mint 地址 #{}: {} (错误: {})", idx + 1, mint, e)
            });
        }
    }

    // 显示将要处理的 mint 地址
    println!();
    if language == Language::English {
        println!("{}", format!("📋 Found {} token(s) to sell:", mint_addresses.len()).bright_green());
    } else {
        println!("{}", format!("📋 找到 {} 个代币待卖出:", mint_addresses.len()).bright_green());
    }
    for (idx, mint) in mint_addresses.iter().enumerate() {
        println!("   {}. {}", idx + 1, mint.bright_white());
    }

    // Step 4: 使用默认滑点 99%
    let slippage = 9900u64;
    println!();
    if language == Language::English {
        println!("📊 Slippage tolerance: {}%", slippage as f64 / 100.0);
    } else {
        println!("📊 滑点容忍度: {}%", slippage as f64 / 100.0);
    }

    // Step 4.5: 批量卖出前统一确认
    let total_mints = mint_addresses.len();
    if total_mints > 1 {
        println!();
        if language == Language::English {
            println!("{}", format!("⚠️  You are about to sell {} tokens", total_mints).yellow().bold());
            println!("   All tokens will be sold automatically without individual confirmation");
        } else {
            println!("{}", format!("⚠️  您即将卖出 {} 个代币", total_mints).yellow().bold());
            println!("   所有代币将自动卖出，不会逐个确认");
        }

        print!("\n{}", if language == Language::English {
            "❓ Confirm batch sell? (yes/no, default: yes): "
        } else {
            "❓ 确认批量卖出? (yes/no, 默认 yes): "
        }.yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).map_err(|e| e.to_string())?;
        let confirm_trimmed = confirm.trim().to_lowercase();

        if confirm_trimmed == "no" || confirm_trimmed == "n" {
            return Err(if language == Language::English {
                "❌ Batch sell cancelled".to_string()
            } else {
                "❌ 批量卖出已取消".to_string()
            });
        }
    }

    // Step 5: 循环处理每个 mint 地址
    // 批量卖出时跳过单个确认（skip_confirmation=true）
    let skip_confirmation = total_mints > 1;
    for (idx, mint) in mint_addresses.iter().enumerate() {
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
        if language == Language::English {
            println!("{}", format!("🚀 Processing token {}/{}", idx + 1, total_mints).bright_blue());
            println!("   Mint: {}", mint.bright_white());
        } else {
            println!("{}", format!("🚀 处理第 {}/{} 个代币", idx + 1, total_mints).bright_blue());
            println!("   Mint: {}", mint.bright_white());
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

        // 使用当前 tokio runtime 执行异步操作
        let result = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // 在当前运行时中执行
                let keypair_b58 = bs58::encode(keypair.to_bytes()).into_string();
                let mint_clone = mint.to_string();
                let rpc_url_clone = rpc_url.clone();

                std::thread::spawn(move || {
                    let keypair_clone = Keypair::from_base58_string(&keypair_b58);
                    handle.block_on(async move {
                        crate::solana_utils::pumpswap_sell::handle_pumpswap_sell_no_prompt(
                            &keypair_clone,
                            &mint_clone,
                            &rpc_url_clone,
                            slippage,
                            use_seed,
                            language,
                            skip_confirmation,  // 传入 skip_confirmation 参数
                        ).await
                    })
                })
                .join()
                .map_err(|_| "Thread panicked".to_string())?
            }
            Err(_) => {
                // 没有运行时，创建新的
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                rt.block_on(async {
                    crate::solana_utils::pumpswap_sell::handle_pumpswap_sell_no_prompt(
                        keypair,
                        mint,
                        &rpc_url,
                        slippage,
                        use_seed,
                        language,
                        skip_confirmation,  // 传入 skip_confirmation 参数
                    ).await
                })
            }
        };

        // 处理结果
        match result {
            Ok(_) => {
                if language == Language::English {
                    println!("\n{}", format!("✅ Token {}/{} sold successfully", idx + 1, total_mints).bright_green());
                } else {
                    println!("\n{}", format!("✅ 第 {}/{} 个代币卖出成功", idx + 1, total_mints).bright_green());
                }
            }
            Err(e) => {
                if language == Language::English {
                    println!("\n{}", format!("❌ Token {}/{} failed: {}", idx + 1, total_mints, e).bright_red());
                    println!("   Continuing with next token...");
                } else {
                    println!("\n{}", format!("❌ 第 {}/{} 个代币卖出失败: {}", idx + 1, total_mints, e).bright_red());
                    println!("   继续处理下一个代币...");
                }
            }
        }

        // 如果不是最后一个，添加延迟
        if idx < total_mints - 1 {
            println!();
            if language == Language::English {
                println!("⏳ Waiting 2 seconds before next transaction...");
            } else {
                println!("⏳ 等待 2 秒后处理下一个交易...");
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    // 所有交易完成
    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("{}", "🎉 All transactions completed!".bright_green().bold());
    } else {
        println!("{}", "🎉 所有交易已完成！".bright_green().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    Ok(())
}

#[cfg(not(feature = "sol-trade-sdk"))]
pub fn pumpswap_sell_interactive(_keypair: &Keypair, language: Language) -> Result<(), String> {
    Err(if language == Language::English {
        "PumpSwap sell requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk".to_string()
    } else {
        "PumpSwap 卖出需要 'sol-trade-sdk' 功能。请使用以下命令重新编译:\ncargo build --release --features sol-trade-sdk".to_string()
    })
}

/// Pump.fun 内盘（bonding curve）交互式卖出
#[cfg(feature = "sol-trade-sdk")]
pub fn pumpfun_sell_interactive(keypair: &Keypair, language: Language) -> Result<(), String> {
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("  {}", "🔥 Pump.fun Bonding Curve Sell".bright_magenta().bold());
    } else {
        println!("  {}", "🔥 Pump.fun 内盘卖出代币".bright_magenta().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    println!("\n{}", if language == Language::English {
        "Current Wallet:"
    } else {
        "当前钱包:"
    }.bright_green());
    println!("  📍 {}", keypair.pubkey().to_string().bright_white());

    let rpc_prompt = if language == Language::English {
        format!("RPC URL (default: {}): ", DEFAULT_RPC_URL)
    } else {
        format!("RPC URL (默认: {}): ", DEFAULT_RPC_URL)
    };
    let rpc_url = read_input(&rpc_prompt, DEFAULT_RPC_URL);

    println!();
    if language == Language::English {
        println!("{}", "🔧 Seed Optimization Configuration".bright_cyan());
        println!("   If your token ATA was created using the standard method, choose 'no'");
        println!("   If unsure, it's recommended to choose 'no'");
    } else {
        println!("{}", "🔧 Seed 优化配置".bright_cyan());
        println!("   如果你的代币 ATA 是通过标准方式创建的，请选择 'no'");
        println!("   如果不确定，建议选择 'yes'（默认）");
    }

    print!("\n{} ", if language == Language::English {
        "❓ Enable Seed Optimization? (yes/no, default: yes):"
    } else {
        "❓ 启用 Seed 优化? (yes/no, 默认 yes):"
    }.yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut seed_input = String::new();
    io::stdin().read_line(&mut seed_input).map_err(|e| e.to_string())?;
    let seed_input_trimmed = seed_input.trim().to_lowercase();
    let use_seed = seed_input_trimmed.is_empty() || seed_input_trimmed == "yes" || seed_input_trimmed == "y";

    if language == Language::English {
        println!("{}", "💡 You can enter multiple mint addresses separated by commas or spaces".bright_cyan());
    } else {
        println!("{}", "💡 可以输入多个 Mint 地址，用逗号或空格分割".bright_cyan());
    }

    let mint_prompt = if language == Language::English {
        "\nToken Mint Address(es): "
    } else {
        "\n代币 Mint 地址: "
    };
    print!("{}", mint_prompt.yellow());
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut mint_input = String::new();
    io::stdin().read_line(&mut mint_input).map_err(|e| e.to_string())?;
    let mint_input = mint_input.trim();

    if mint_input.is_empty() {
        return Err(if language == Language::English {
            "Token mint address cannot be empty".to_string()
        } else {
            "代币 Mint 地址不能为空".to_string()
        });
    }

    let mint_addresses: Vec<String> = mint_input
        .split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if mint_addresses.is_empty() {
        return Err(if language == Language::English {
            "No valid mint addresses found".to_string()
        } else {
            "未找到有效的 Mint 地址".to_string()
        });
    }

    for (idx, mint) in mint_addresses.iter().enumerate() {
        if let Err(e) = Pubkey::from_str(mint) {
            return Err(if language == Language::English {
                format!("Invalid mint address #{}: {} (error: {})", idx + 1, mint, e)
            } else {
                format!("无效的 Mint 地址 #{}: {} (错误: {})", idx + 1, mint, e)
            });
        }
    }

    println!();
    if language == Language::English {
        println!("{}", format!("📋 Found {} token(s) to sell:", mint_addresses.len()).bright_green());
    } else {
        println!("{}", format!("📋 找到 {} 个代币待卖出:", mint_addresses.len()).bright_green());
    }
    for (idx, mint) in mint_addresses.iter().enumerate() {
        println!("   {}. {}", idx + 1, mint.bright_white());
    }

    let slippage = 9900u64;
    println!();
    if language == Language::English {
        println!("📊 Slippage tolerance: {}%", slippage as f64 / 100.0);
    } else {
        println!("📊 滑点容忍度: {}%", slippage as f64 / 100.0);
    }

    let total_mints = mint_addresses.len();
    if total_mints > 1 {
        println!();
        if language == Language::English {
            println!("{}", format!("⚠️  You are about to sell {} tokens", total_mints).yellow().bold());
        } else {
            println!("{}", format!("⚠️  您即将卖出 {} 个代币", total_mints).yellow().bold());
        }

        print!("\n{}", if language == Language::English {
            "❓ Confirm batch sell? (yes/no, default: yes): "
        } else {
            "❓ 确认批量卖出? (yes/no, 默认 yes): "
        }.yellow());
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).map_err(|e| e.to_string())?;
        let confirm_trimmed = confirm.trim().to_lowercase();

        if confirm_trimmed == "no" || confirm_trimmed == "n" {
            return Err(if language == Language::English {
                "❌ Batch sell cancelled".to_string()
            } else {
                "❌ 批量卖出已取消".to_string()
            });
        }
    }

    let skip_confirmation = total_mints > 1;
    for (idx, mint) in mint_addresses.iter().enumerate() {
        println!();
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
        if language == Language::English {
            println!("{}", format!("🚀 Processing token {}/{}", idx + 1, total_mints).bright_blue());
            println!("   Mint: {}", mint.bright_white());
        } else {
            println!("{}", format!("🚀 处理第 {}/{} 个代币", idx + 1, total_mints).bright_blue());
            println!("   Mint: {}", mint.bright_white());
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

        let result = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let keypair_b58 = bs58::encode(keypair.to_bytes()).into_string();
                let mint_clone = mint.to_string();
                let rpc_url_clone = rpc_url.clone();

                std::thread::spawn(move || {
                    let keypair_clone = Keypair::from_base58_string(&keypair_b58);
                    handle.block_on(async move {
                        crate::solana_utils::pumpfun_sell::handle_pumpfun_sell_no_prompt(
                            &keypair_clone,
                            &mint_clone,
                            &rpc_url_clone,
                            slippage,
                            use_seed,
                            language,
                            skip_confirmation,
                        ).await
                    })
                })
                .join()
                .map_err(|_| "Thread panicked".to_string())?
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                rt.block_on(async {
                    crate::solana_utils::pumpfun_sell::handle_pumpfun_sell_no_prompt(
                        keypair,
                        mint,
                        &rpc_url,
                        slippage,
                        use_seed,
                        language,
                        skip_confirmation,
                    ).await
                })
            }
        };

        match result {
            Ok(_) => {
                if language == Language::English {
                    println!("\n{}", format!("✅ Token {}/{} sold successfully", idx + 1, total_mints).bright_green());
                } else {
                    println!("\n{}", format!("✅ 第 {}/{} 个代币卖出成功", idx + 1, total_mints).bright_green());
                }
            }
            Err(e) => {
                if language == Language::English {
                    println!("\n{}", format!("❌ Token {}/{} failed: {}", idx + 1, total_mints, e).bright_red());
                } else {
                    println!("\n{}", format!("❌ 第 {}/{} 个代币卖出失败: {}", idx + 1, total_mints, e).bright_red());
                }
            }
        }

        if idx < total_mints - 1 {
            println!();
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    println!();
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("{}", "🎉 All transactions completed!".bright_green().bold());
    } else {
        println!("{}", "🎉 所有交易已完成！".bright_green().bold());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    Ok(())
}

/// Pump (Pump.fun) 返现：查询余额 → 显示 → 确认 → 领取（原生 SOL）
#[cfg(feature = "sol-trade-sdk")]
pub fn pumpfun_cashback_interactive(keypair: &Keypair, language: Language) -> Result<(), String> {
    use std::sync::Arc;
    use sol_trade_sdk::{common::TradeConfig, SolanaTrade};
    use solana_commitment_config::CommitmentConfig;

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("  {}", "💰 Pump (Pump.fun) Cashback – View & Claim".bright_magenta().bold());
        println!("  {}", "   Cashback is native SOL from trading on Pump.fun.".bright_white());
    } else {
        println!("  {}", "💰 Pump (Pump.fun) 返现 – 查看与领取".bright_magenta().bold());
        println!("  {}", "   返现为在 Pump.fun 交易累积的原生 SOL。".bright_white());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    println!("\n{}", if language == Language::English {
        "Current Wallet:"
    } else {
        "当前钱包:"
    }.bright_green());
    println!("  📍 {}", keypair.pubkey().to_string().bright_white());

    let rpc_prompt = if language == Language::English {
        format!("RPC URL (default: {}): ", DEFAULT_RPC_URL)
    } else {
        format!("RPC URL (默认: {}): ", DEFAULT_RPC_URL)
    };
    let rpc_url = read_input(&rpc_prompt, DEFAULT_RPC_URL);

    // Step 1: 查询 UserVolumeAccumulator PDA 余额
    if language == Language::English {
        println!("\n{}", "🔍 Querying cashback balance...".bright_cyan());
    } else {
        println!("\n{}", "🔍 正在查询返现余额...".bright_cyan());
    }

    let pda = sol_trade_sdk::instruction::utils::pumpfun::get_user_volume_accumulator_pda(&keypair.pubkey())
        .ok_or_else(|| if language == Language::English {
            "Failed to derive UserVolumeAccumulator PDA".to_string()
        } else {
            "无法派生 UserVolumeAccumulator PDA 地址".to_string()
        })?;

    let rpc_client = RpcClient::new(rpc_url.clone());
    let pda_balance_lamports = rpc_client.get_balance(&pda).unwrap_or(0);
    let rent_exempt_min: u64 = 890_880;
    let claimable_lamports = pda_balance_lamports.saturating_sub(rent_exempt_min);
    let claimable_sol = claimable_lamports as f64 / 1_000_000_000.0;

    // Step 2: 显示余额
    println!("\n{}", if language == Language::English {
        "📊 Cashback Info:"
    } else {
        "📊 返现信息:"
    }.bright_yellow());
    println!("  PDA: {}", pda.to_string().bright_white());
    if language == Language::English {
        println!("  PDA Balance: {} lamports ({:.9} SOL)", pda_balance_lamports, pda_balance_lamports as f64 / 1e9);
        println!("  Claimable:   {} lamports ({:.9} SOL)", claimable_lamports, claimable_sol);
    } else {
        println!("  PDA 余额:    {} lamports ({:.9} SOL)", pda_balance_lamports, pda_balance_lamports as f64 / 1e9);
        println!("  可领取:      {} lamports ({:.9} SOL)", claimable_lamports, claimable_sol);
    }

    if claimable_lamports == 0 {
        if language == Language::English {
            println!("\n{}", "ℹ️  No cashback available to claim.".bright_yellow());
        } else {
            println!("\n{}", "ℹ️  暂无可领取的返现。".bright_yellow());
        }
        return Ok(());
    }

    // Step 3: 确认是否领取
    let confirm_prompt = if language == Language::English {
        format!("\nClaim {:.9} SOL? (yes/no) [no]: ", claimable_sol)
    } else {
        format!("\n领取 {:.9} SOL? (yes/no) [no]: ", claimable_sol)
    };
    let confirm = read_input(&confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Claim cancelled"
        } else {
            "❌ 已取消领取"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    // Step 4: 执行领取
    if language == Language::English {
        println!("\n{}", "🚀 Claiming...".bright_cyan());
    } else {
        println!("\n{}", "🚀 正在领取...".bright_cyan());
    }

    let sig = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            let keypair_b58 = bs58::encode(keypair.to_bytes()).into_string();
            let rpc_url_clone = rpc_url.clone();
            std::thread::spawn(move || {
                let kp = Keypair::from_base58_string(&keypair_b58);
                let payer = Arc::new(kp);
                let config = TradeConfig {
                    rpc_url: rpc_url_clone.clone(),
                    swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url_clone)],
                    commitment: CommitmentConfig::confirmed(),
                    create_wsol_ata_on_startup: false,
                    use_seed_optimize: false,
                };
                handle.block_on(async move {
                    let client = SolanaTrade::new(payer, config).await;
                    client.claim_cashback_pumpfun().await.map_err(|e| e.to_string())
                })
            })
            .join()
            .map_err(|_| "Thread panicked".to_string())??
        }
        Err(_) => {
            let payer = Arc::new(keypair.insecure_clone());
            let config = TradeConfig {
                rpc_url: rpc_url.clone(),
                swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url)],
                commitment: CommitmentConfig::confirmed(),
                create_wsol_ata_on_startup: false,
                use_seed_optimize: false,
            };
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            let client = rt.block_on(SolanaTrade::new(payer, config));
            rt.block_on(client.claim_cashback_pumpfun()).map_err(|e| e.to_string())?
        }
    };

    if language == Language::English {
        println!("{}", "✅ Claim successful!".bright_green().bold());
        println!("Signature: {}", sig.yellow());
        println!("Explorer: https://solscan.io/tx/{}", sig);
    } else {
        println!("{}", "✅ 领取成功！".bright_green().bold());
        println!("签名: {}", sig.yellow());
        println!("浏览器: https://solscan.io/tx/{}", sig);
    }
    Ok(())
}

/// PumpSwap 返现：查询余额 → 显示 → 确认 → 领取（WSOL）
#[cfg(feature = "sol-trade-sdk")]
pub fn pumpswap_cashback_interactive(keypair: &Keypair, language: Language) -> Result<(), String> {
    use std::sync::Arc;
    use sol_trade_sdk::{common::TradeConfig, SolanaTrade};
    use solana_commitment_config::CommitmentConfig;

    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());
    if language == Language::English {
        println!("  {}", "💰 PumpSwap Cashback – View & Claim".bright_magenta().bold());
        println!("  {}", "   Cashback is WSOL from trading on PumpSwap.".bright_white());
    } else {
        println!("  {}", "💰 PumpSwap 返现 – 查看与领取".bright_magenta().bold());
        println!("  {}", "   返现为在 PumpSwap 交易累积的 WSOL。".bright_white());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_magenta());

    println!("\n{}", if language == Language::English {
        "Current Wallet:"
    } else {
        "当前钱包:"
    }.bright_green());
    println!("  📍 {}", keypair.pubkey().to_string().bright_white());

    let rpc_prompt = if language == Language::English {
        format!("RPC URL (default: {}): ", DEFAULT_RPC_URL)
    } else {
        format!("RPC URL (默认: {}): ", DEFAULT_RPC_URL)
    };
    let rpc_url = read_input(&rpc_prompt, DEFAULT_RPC_URL);

    // Step 1: 查询 PumpSwap UserVolumeAccumulator 的 WSOL ATA 余额
    if language == Language::English {
        println!("\n{}", "🔍 Querying cashback balance...".bright_cyan());
    } else {
        println!("\n{}", "🔍 正在查询返现余额...".bright_cyan());
    }

    let wsol_ata = sol_trade_sdk::instruction::utils::pumpswap::get_user_volume_accumulator_wsol_ata(&keypair.pubkey())
        .ok_or_else(|| if language == Language::English {
            "Failed to derive PumpSwap UserVolumeAccumulator WSOL ATA".to_string()
        } else {
            "无法派生 PumpSwap UserVolumeAccumulator WSOL ATA 地址".to_string()
        })?;

    let rpc_client = RpcClient::new(rpc_url.clone());
    let claimable_lamports = match rpc_client.get_token_account_balance(&wsol_ata) {
        Ok(balance) => balance.amount.parse::<u64>().unwrap_or(0),
        Err(_) => 0,
    };
    let claimable_sol = claimable_lamports as f64 / 1_000_000_000.0;

    // Step 2: 显示余额
    println!("\n{}", if language == Language::English {
        "📊 Cashback Info:"
    } else {
        "📊 返现信息:"
    }.bright_yellow());
    println!("  WSOL ATA: {}", wsol_ata.to_string().bright_white());
    if language == Language::English {
        println!("  Claimable: {} lamports ({:.9} SOL)", claimable_lamports, claimable_sol);
    } else {
        println!("  可领取:   {} lamports ({:.9} SOL)", claimable_lamports, claimable_sol);
    }

    if claimable_lamports == 0 {
        if language == Language::English {
            println!("\n{}", "ℹ️  No cashback available to claim.".bright_yellow());
        } else {
            println!("\n{}", "ℹ️  暂无可领取的返现。".bright_yellow());
        }
        return Ok(());
    }

    // Step 3: 确认是否领取
    let confirm_prompt = if language == Language::English {
        format!("\nClaim {:.9} SOL (WSOL)? (yes/no) [no]: ", claimable_sol)
    } else {
        format!("\n领取 {:.9} SOL (WSOL)? (yes/no) [no]: ", claimable_sol)
    };
    let confirm = read_input(&confirm_prompt, "no");

    if confirm.to_lowercase() != "yes" {
        let msg = if language == Language::English {
            "❌ Claim cancelled"
        } else {
            "❌ 已取消领取"
        };
        println!("\n{}", msg.red());
        return Ok(());
    }

    // Step 4: 执行领取
    if language == Language::English {
        println!("\n{}", "🚀 Claiming...".bright_cyan());
    } else {
        println!("\n{}", "🚀 正在领取...".bright_cyan());
    }

    let sig = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            let keypair_b58 = bs58::encode(keypair.to_bytes()).into_string();
            let rpc_url_clone = rpc_url.clone();
            std::thread::spawn(move || {
                let kp = Keypair::from_base58_string(&keypair_b58);
                let payer = Arc::new(kp);
                let config = TradeConfig {
                    rpc_url: rpc_url_clone.clone(),
                    swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url_clone)],
                    commitment: CommitmentConfig::confirmed(),
                    create_wsol_ata_on_startup: false,
                    use_seed_optimize: false,
                };
                handle.block_on(async move {
                    let client = SolanaTrade::new(payer, config).await;
                    client.claim_cashback_pumpswap().await.map_err(|e| e.to_string())
                })
            })
            .join()
            .map_err(|_| "Thread panicked".to_string())??
        }
        Err(_) => {
            let payer = Arc::new(keypair.insecure_clone());
            let config = TradeConfig {
                rpc_url: rpc_url.clone(),
                swqos_configs: vec![sol_trade_sdk::swqos::SwqosConfig::Default(rpc_url)],
                commitment: CommitmentConfig::confirmed(),
                create_wsol_ata_on_startup: false,
                use_seed_optimize: false,
            };
            let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
            let client = rt.block_on(SolanaTrade::new(payer, config));
            rt.block_on(client.claim_cashback_pumpswap()).map_err(|e| e.to_string())?
        }
    };

    if language == Language::English {
        println!("{}", "✅ Claim successful!".bright_green().bold());
        println!("Signature: {}", sig.yellow());
        println!("Explorer: https://solscan.io/tx/{}", sig);
    } else {
        println!("{}", "✅ 领取成功！".bright_green().bold());
        println!("签名: {}", sig.yellow());
        println!("浏览器: https://solscan.io/tx/{}", sig);
    }
    Ok(())
}

#[cfg(not(feature = "sol-trade-sdk"))]
pub fn pumpfun_sell_interactive(_keypair: &Keypair, language: Language) -> Result<(), String> {
    Err(if language == Language::English {
        "Pump.fun sell requires 'sol-trade-sdk' feature. Please rebuild with:\ncargo build --release --features sol-trade-sdk".to_string()
    } else {
        "Pump.fun 内盘卖出需要 'sol-trade-sdk' 功能。请使用以下命令重新编译:\ncargo build --release --features sol-trade-sdk".to_string()
    })
}

#[cfg(not(feature = "sol-trade-sdk"))]
pub fn pumpfun_cashback_interactive(_keypair: &Keypair, language: Language) -> Result<(), String> {
    Err(if language == Language::English {
        "Pump cashback requires 'sol-trade-sdk' feature.".to_string()
    } else {
        "Pump 返现需要 'sol-trade-sdk' 功能。".to_string()
    })
}

#[cfg(not(feature = "sol-trade-sdk"))]
pub fn pumpswap_cashback_interactive(_keypair: &Keypair, language: Language) -> Result<(), String> {
    Err(if language == Language::English {
        "PumpSwap cashback requires 'sol-trade-sdk' feature.".to_string()
    } else {
        "PumpSwap 返现需要 'sol-trade-sdk' 功能。".to_string()
    })
}
