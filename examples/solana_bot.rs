/// Solana 操作示例 - 展示如何使用 solana-ops 功能
///
/// 运行方式：
/// ```bash
/// cargo run --example solana_bot --features solana-ops
/// ```

use sol_safekey::{KeyManager, solana_utils::*};
use solana_sdk::{signature::Keypair, pubkey::Pubkey, signer::Signer};
use std::{str::FromStr, fs, env};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤖 Solana Bot Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 配置
    let use_devnet = true; // 使用 devnet 进行测试
    let rpc_url = if use_devnet {
        "https://api.devnet.solana.com"
    } else {
        "https://api.mainnet-beta.solana.com"
    };

    println!("📡 Network: {}", if use_devnet { "Devnet" } else { "Mainnet" });
    println!("🔗 RPC: {}\n", rpc_url);

    // 步骤 1: 生成或加载钱包
    let keypair = setup_wallet()?;
    println!("✅ Wallet loaded!");
    println!("📍 Address: {}\n", keypair.pubkey());

    // 步骤 2: 创建 Solana 客户端
    let client = SolanaClient::new(rpc_url.to_string());

    // 步骤 3: 查询 SOL 余额
    demo_check_balance(&client, &keypair).await?;

    // 步骤 4: 查询 Token 余额 (USDC 示例 - devnet)
    if use_devnet {
        demo_check_token_balance(&client, &keypair).await?;
    }

    // 步骤 5: 转账示例 (注释掉以防误操作)
    // demo_transfer(&client, &keypair).await?;

    // 步骤 6: Wrap/Unwrap 示例
    // demo_wrap_unwrap(&client, &keypair).await?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Example completed successfully!");
    println!("\n💡 Tip: Edit this file to uncomment transfer examples");
    println!("⚠️  Warning: Make sure to use devnet for testing!");

    Ok(())
}

/// 设置钱包 - 从文件加载或生成新的
fn setup_wallet() -> Result<Keypair> {
    println!("🔑 Setting up wallet...");

    // 尝试从环境变量加载
    if let Ok(wallet_path) = env::var("WALLET_PATH") {
        println!("📂 Loading from: {}", wallet_path);
        let json = fs::read_to_string(&wallet_path)?;

        // 尝试从环境变量获取密码，否则使用默认密码
        let password = env::var("WALLET_PASSWORD")
            .unwrap_or_else(|_| "example_password".to_string());

        return Ok(KeyManager::keypair_from_encrypted_json(&json, &password)?);
    }

    // 否则生成新钱包（仅用于演示）
    println!("🆕 Generating new wallet for demo...");
    let keypair = KeyManager::generate_keypair();

    // 可选：保存到文件
    // let json = KeyManager::keypair_to_encrypted_json(&keypair, "example_password")?;
    // fs::write("demo_wallet.json", json)?;

    Ok(keypair)
}

/// 演示：查询 SOL 余额
async fn demo_check_balance(client: &SolanaClient, keypair: &Keypair) -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Demo 1: Check SOL Balance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    let sol_amount = lamports_to_sol(balance);

    println!("Address: {}", keypair.pubkey());
    println!("Balance: {} SOL", sol_amount);
    println!("Balance: {} lamports", balance);

    if balance == 0 {
        println!("\n💡 Tip: Get devnet SOL with:");
        println!("   solana airdrop 2 {} --url devnet", keypair.pubkey());
    }

    Ok(())
}

/// 演示：查询 Token 余额
async fn demo_check_token_balance(client: &SolanaClient, keypair: &Keypair) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Demo 2: Check Token Balance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 示例：查询 devnet USDC (你需要替换为实际的 devnet token mint)
    // 这里使用一个示例 mint 地址
    let example_mint = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"; // devnet 示例

    println!("Token Mint: {}", example_mint);

    match Pubkey::from_str(example_mint) {
        Ok(mint_pubkey) => {
            match client.get_token_balance(&keypair.pubkey(), &mint_pubkey).await {
                Ok(balance) => {
                    println!("Balance: {} (smallest units)", balance);
                    println!("Balance (9 decimals): {}", format_token_amount(balance, 9));
                }
                Err(e) => {
                    println!("⚠️  Could not fetch balance: {}", e);
                    println!("💡 Note: Token account may not exist");
                }
            }
        }
        Err(e) => {
            println!("⚠️  Invalid mint address: {}", e);
        }
    }

    Ok(())
}

/// 演示：转账（默认注释）
#[allow(dead_code)]
async fn demo_transfer(client: &SolanaClient, keypair: &Keypair) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💸 Demo 3: Transfer SOL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 接收地址（替换为你的测试地址）
    let recipient_str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
    let recipient = Pubkey::from_str(recipient_str)?;

    // 转账金额：0.01 SOL
    let amount = (0.01 * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;

    println!("From: {}", keypair.pubkey());
    println!("To: {}", recipient);
    println!("Amount: 0.01 SOL ({} lamports)", amount);

    // 检查余额
    let balance = client.get_sol_balance(&keypair.pubkey()).await?;
    if balance < amount + 5000 { // 预留手续费
        return Err(anyhow::anyhow!("Insufficient balance for transfer + fees"));
    }

    println!("\n🚀 Sending transaction...");
    let signature = client.transfer_sol(keypair, &recipient, amount).await?;

    println!("✅ Transfer successful!");
    println!("Signature: {}", signature);
    println!("Explorer: https://solscan.io/tx/{}?cluster=devnet", signature);

    Ok(())
}

/// 演示：Wrap/Unwrap SOL
#[allow(dead_code)]
async fn demo_wrap_unwrap(client: &SolanaClient, keypair: &Keypair) -> Result<()> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 Demo 4: Wrap/Unwrap SOL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Wrap 0.1 SOL
    let wrap_amount = (0.1 * solana_sdk::native_token::LAMPORTS_PER_SOL as f64) as u64;

    println!("🔄 Wrapping {} lamports to WSOL...", wrap_amount);
    let wrap_sig = client.wrap_sol(keypair, wrap_amount).await?;
    println!("✅ Wrap successful!");
    println!("Signature: {}", wrap_sig);

    // 等待确认
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Unwrap all WSOL
    println!("\n🔄 Unwrapping WSOL to SOL...");
    let unwrap_sig = client.unwrap_sol(keypair).await?;
    println!("✅ Unwrap successful!");
    println!("Signature: {}", unwrap_sig);

    Ok(())
}
