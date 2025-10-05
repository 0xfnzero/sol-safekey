//! Solana Operations Interactive Menu
//!
//! Provides interactive Solana operations using encrypted keystore
//! 提供使用加密 keystore 的交互式 Solana 操作

use std::io::{self, Write};
use colored::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[cfg(feature = "solana-ops")]
use crate::solana_utils::{SolanaClient, lamports_to_sol};

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
            println!("  {}  Wrap SOL → WSOL", "3.".bright_cyan());
            println!("  {}  Unwrap WSOL → SOL", "4.".bright_cyan());
            println!("  {}  Transfer SPL Token", "5.".bright_cyan());
            println!("  {}  Create Nonce Account", "6.".bright_cyan());
            println!("  {}  Back to Main Menu", "0.".bright_cyan());
        } else {
            println!("  {}  查询 SOL 余额", "1.".bright_cyan());
            println!("  {}  转账 SOL", "2.".bright_cyan());
            println!("  {}  包装 SOL → WSOL", "3.".bright_cyan());
            println!("  {}  解包 WSOL → SOL", "4.".bright_cyan());
            println!("  {}  转账 SPL 代币", "5.".bright_cyan());
            println!("  {}  创建 Nonce 账户", "6.".bright_cyan());
            println!("  {}  返回主菜单", "0.".bright_cyan());
        }

        let prompt = if language == Language::English {
            "\nSelect option [0-6]: "
        } else {
            "\n请输入选项 [0-6]: "
        };

        let choice = read_input(prompt, "");

        match choice.as_str() {
            "1" => check_balance(keypair, language)?,
            "2" => transfer_sol(keypair, language)?,
            "3" => wrap_sol(keypair, language)?,
            "4" => unwrap_sol(keypair, language)?,
            "5" => transfer_token(keypair, language)?,
            "6" => create_nonce_account(keypair, language)?,
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

    let client = SolanaClient::new(rpc_url.to_string());
    match client.wrap_sol(keypair, amount_lamports) {
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

    println!("\n{}", if language == Language::English {
        "ℹ️  This will unwrap ALL your WSOL back to SOL"
    } else {
        "ℹ️  这将把您的所有 WSOL 解包回 SOL"
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
        println!("\n🚀 Unwrapping WSOL...");
    } else {
        println!("\n🚀 正在解包 WSOL...");
    }

    let client = SolanaClient::new(rpc_url.to_string());
    match client.unwrap_sol(keypair) {
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
    use rpassword;
    use crate::KeyManager;

    #[cfg(feature = "solana-ops")]
    {
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
