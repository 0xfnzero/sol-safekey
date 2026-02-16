// Pump.fun 内盘（bonding curve）卖出功能模块
// 实现通过 sol-trade-sdk 在 Pump.fun bonding curve 上卖出代币，输出为原生 SOL

use anyhow::Result;
use colored::Colorize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;
use std::sync::Arc;

use crate::operations::Language;

#[cfg(feature = "sol-trade-sdk")]
use sol_trade_sdk::{
    common::{
        fast_fn::get_associated_token_address_with_program_id_fast_use_seed,
        TradeConfig, GasFeeStrategy,
    },
    swqos::SwqosConfig,
    trading::{
        core::params::{PumpFunParams, DexParamEnum},
        factory::DexType,
    },
    SolanaTrade, TradeSellParams, TradeTokenType,
};
use solana_commitment_config::CommitmentConfig;

/// 处理 Pump.fun 内盘卖出操作（交互式，单次调用）
#[cfg(feature = "sol-trade-sdk")]
pub async fn handle_pumpfun_sell(
    keypair: &Keypair,
    mint: &str,
    rpc_url: &str,
    slippage: u64,
) -> Result<()> {
    println!("\n{}", "🔥 Pump.fun 内盘卖出操作".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════".cyan());

    let mint_pubkey = Pubkey::from_str(mint)
        .map_err(|e| anyhow::anyhow!("无效的代币地址: {}", e))?;

    println!("📍 代币地址: {}", mint.yellow());
    println!("🌐 RPC: {}", rpc_url);
    println!("📊 滑点容忍度: {}%", slippage as f64 / 100.0);

    let use_seed = crate::solana_utils::pumpswap_sell::ask_use_seed()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    println!("\n{}", "🔍 检查代币余额...".cyan());

    let payer = Arc::new(keypair.insecure_clone());
    let commitment = CommitmentConfig::confirmed();
    let swqos_configs: Vec<SwqosConfig> = vec![SwqosConfig::Default(rpc_url.to_string())];

    let trade_config = TradeConfig {
        rpc_url: rpc_url.to_string(),
        swqos_configs,
        commitment,
        create_wsol_ata_on_startup: false,
        use_seed_optimize: use_seed,
    };

    let client = SolanaTrade::new(payer.clone(), trade_config).await;

    let (token_balance, decimals, token_program) = check_token_balance(
        &client,
        &mint_pubkey,
        &keypair.pubkey(),
        use_seed,
    )
    .await?;

    if token_balance == 0 {
        return Err(anyhow::anyhow!("❌ 代币余额为 0，无法卖出"));
    }

    let readable_balance = token_balance as f64 / 10_f64.powi(decimals as i32);
    println!(
        "💰 代币余额: {} (原始数量: {})",
        readable_balance.to_string().green(),
        token_balance
    );
    println!("🔧 Token Program: {}", token_program);

    print!("\n{}", "❓ 确认全部卖出? (yes/no, 默认 yes): ".yellow());
    use std::io::{self, Write};
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    let confirm_trimmed = confirm.trim().to_lowercase();
    if confirm_trimmed == "no" || confirm_trimmed == "n" {
        println!("{}", "❌ 操作已取消".red());
        return Ok(());
    }

    println!("\n{}", "📡 从链上获取 Pump.fun bonding curve 参数...".cyan());

    let pump_params = match PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await {
        Ok(params) => {
            println!("✅ 找到 Pump.fun 池子");
            println!("   Bonding Curve: {}", params.bonding_curve.account);
            params
        }
        Err(e) => {
            println!("{}", "❌ 获取 Pump.fun 参数失败".red().bold());
            println!("错误详情: {}", e);
            println!();
            println!("{}", "🔍 可能的原因:".yellow());
            println!("   1. 代币未在 Pump.fun 上架或已迁移到 Raydium");
            println!("   2. RPC 限流或超时");
            println!("   3. Mint 地址错误: {}", mint_pubkey);
            return Err(anyhow::anyhow!("无法获取 Pump.fun 池子参数，请检查 RPC 或代币是否在 bonding curve"));
        }
    };

    let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;

    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_global_fee_strategy(150000, 150000, 500000, 500000, 0.001, 0.001);

    println!("\n{}", "🚀 构建卖出交易...".cyan());

    let sell_params = TradeSellParams {
        dex_type: DexType::PumpFun,
        output_token_type: TradeTokenType::SOL, // 内盘卖出得到原生 SOL
        mint: mint_pubkey,
        input_token_amount: token_balance,
        slippage_basis_points: Some(slippage),
        recent_blockhash: Some(recent_blockhash),
        with_tip: false,
        extension_params: DexParamEnum::PumpFun(pump_params),
        address_lookup_table_account: None,
        wait_transaction_confirmed: true,
        create_output_token_ata: false, // 输出为 SOL，无需 ATA
        close_output_token_ata: false,
        close_mint_token_ata: false,
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy,
        simulate: false,
    };

    println!("{}", "📤 发送交易到链上...".bright_blue());

    match client.sell(sell_params).await {
        Ok((success, signatures, error)) => {
            if success {
                println!("\n{}", "✅ 卖出成功！".green().bold());
                println!("   卖出数量: {} tokens", token_balance);
                for (i, signature) in signatures.iter().enumerate() {
                    if i == 0 || signatures.len() == 1 {
                        println!("   交易签名: {}", signature.to_string().yellow());
                        println!("   🔗 https://solscan.io/tx/{}", signature);
                    } else {
                        println!("   交易签名 {}: {}", i + 1, signature.to_string().yellow());
                        println!("   🔗 https://solscan.io/tx/{}", signature);
                    }
                }
                println!("\n{}", "💡 提示: 已收到原生 SOL，无需解包".bright_yellow());
            } else {
                let error_msg = error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
                return Err(anyhow::anyhow!("卖出失败: {}", error_msg));
            }
        }
        Err(e) => return Err(anyhow::anyhow!("卖出失败: {}", e)),
    }

    Ok(())
}

/// 检查代币余额（与 pumpswap 共用逻辑，返回 余额, decimals, token_program）
#[cfg(feature = "sol-trade-sdk")]
async fn check_token_balance(
    client: &SolanaTrade,
    mint: &Pubkey,
    owner: &Pubkey,
    use_seed: bool,
) -> Result<(u64, u8, Pubkey)> {
    let mint_account = client
        .infrastructure
        .rpc
        .get_account(mint)
        .await
        .map_err(|e| anyhow::anyhow!("获取代币账户失败: {}", e))?;

    let token_program = mint_account.owner;

    let standard_ata = get_associated_token_address_with_program_id_fast_use_seed(
        owner,
        mint,
        &token_program,
        false,
    );

    println!("   检查标准 ATA: {}", standard_ata);

    if let Ok(balance) = client.infrastructure.rpc.get_token_account_balance(&standard_ata).await {
        let amount = balance
            .amount
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("解析余额失败"))?;
        let decimals = balance.decimals;
        println!("   ✅ 找到标准 ATA");
        return Ok((amount, decimals, token_program));
    }

    if use_seed {
        let seed_ata = get_associated_token_address_with_program_id_fast_use_seed(
            owner,
            mint,
            &token_program,
            true,
        );
        println!("   检查 Seed ATA: {}", seed_ata);

        if let Ok(balance) = client.infrastructure.rpc.get_token_account_balance(&seed_ata).await {
            let amount = balance
                .amount
                .parse::<u64>()
                .map_err(|_| anyhow::anyhow!("解析余额失败"))?;
            let decimals = balance.decimals;
            println!("   ✅ 找到 Seed ATA");
            return Ok((amount, decimals, token_program));
        }
    }

    Err(anyhow::anyhow!("未找到代币账户，余额为 0"))
}

/// 处理 Pump.fun 内盘卖出（无交互式提示版本，用于主菜单与批量）
#[cfg(feature = "sol-trade-sdk")]
pub async fn handle_pumpfun_sell_no_prompt(
    keypair: &Keypair,
    mint: &str,
    rpc_url: &str,
    slippage: u64,
    use_seed: bool,
    language: Language,
    skip_confirmation: bool,
) -> Result<(), String> {
    let mint_pubkey = Pubkey::from_str(mint).map_err(|e| format!("无效的代币地址: {}", e))?;

    if language == Language::Chinese {
        println!("📍 代币地址: {}", mint.yellow());
        println!("🌐 RPC: {}", rpc_url);
        println!("📊 滑点容忍度: {}%", slippage as f64 / 100.0);
        println!("🔧 Seed优化: {}", if use_seed { "启用" } else { "禁用" });
        println!("\n{}", "🔍 检查代币余额...".cyan());
    } else {
        println!("📍 Token Address: {}", mint.yellow());
        println!("🌐 RPC: {}", rpc_url);
        println!("📊 Slippage: {}%", slippage as f64 / 100.0);
        println!("🔧 Seed Opt: {}", if use_seed { "Enabled" } else { "Disabled" });
        println!("\n{}", "🔍 Checking token balance...".cyan());
    }

    let payer = Arc::new(keypair.insecure_clone());
    let commitment = CommitmentConfig::confirmed();
    let swqos_configs: Vec<SwqosConfig> = vec![SwqosConfig::Default(rpc_url.to_string())];

    let trade_config = TradeConfig {
        rpc_url: rpc_url.to_string(),
        swqos_configs,
        commitment,
        create_wsol_ata_on_startup: false,
        use_seed_optimize: use_seed,
    };

    let client = SolanaTrade::new(payer.clone(), trade_config).await;

    let (token_balance, decimals, token_program) = check_token_balance(
        &client,
        &mint_pubkey,
        &keypair.pubkey(),
        use_seed,
    )
    .await
    .map_err(|e| e.to_string())?;

    if token_balance == 0 {
        return Err(if language == Language::Chinese {
            "❌ 代币余额为 0，无法卖出".to_string()
        } else {
            "❌ Token balance is 0, cannot sell".to_string()
        });
    }

    let readable_balance = token_balance as f64 / 10_f64.powi(decimals as i32);

    if language == Language::Chinese {
        println!(
            "💰 代币余额: {} (原始数量: {})",
            readable_balance.to_string().green(),
            token_balance
        );
        println!("🔧 Token Program: {}", token_program);
    } else {
        println!(
            "💰 Token Balance: {} (raw: {})",
            readable_balance.to_string().green(),
            token_balance
        );
        println!("🔧 Token Program: {}", token_program);
    }

    if !skip_confirmation {
        if language == Language::Chinese {
            println!("\n{}", "❓ 确认全部卖出? (yes/no, 默认 yes): ".yellow());
        } else {
            println!("\n{}", "❓ Confirm sell all? (yes/no, default: yes): ".yellow());
        }

        use std::io::{self, Write};
        print!(
            "{}",
            if language == Language::Chinese {
                "请输入 (yes/no, 默认 yes): "
            } else {
                "Enter (yes/no, default: yes): "
            }
        );
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).map_err(|e| e.to_string())?;

        let confirm_trimmed = confirm.trim().to_lowercase();
        if confirm_trimmed == "no" || confirm_trimmed == "n" {
            return Err(if language == Language::Chinese {
                "❌ 操作已取消".to_string()
            } else {
                "❌ Operation cancelled".to_string()
            });
        }
    }

    if language == Language::Chinese {
        println!("\n{}", "📡 从链上获取 Pump.fun 参数...".cyan());
    } else {
        println!("\n{}", "📡 Fetching Pump.fun bonding curve...".cyan());
    }

    let pump_params = PumpFunParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey)
        .await
        .map_err(|e| format!("获取 Pump.fun 参数失败: {}", e))?;

    if language == Language::Chinese {
        println!("✅ Pump.fun 参数已获取");
        println!("   Bonding Curve: {}", pump_params.bonding_curve.account);
    } else {
        println!("✅ Pump.fun parameters fetched");
        println!("   Bonding Curve: {}", pump_params.bonding_curve.account);
    }

    let recent_blockhash = client
        .infrastructure
        .rpc
        .get_latest_blockhash()
        .await
        .map_err(|e| format!("获取 blockhash 失败: {}", e))?;

    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_global_fee_strategy(150000, 150000, 500000, 500000, 0.001, 0.001);

    if language == Language::Chinese {
        println!("\n{}", "🚀 构建卖出交易...".cyan());
    } else {
        println!("\n{}", "🚀 Building sell transaction...".cyan());
    }

    let sell_params = TradeSellParams {
        dex_type: DexType::PumpFun,
        output_token_type: TradeTokenType::SOL,
        mint: mint_pubkey,
        input_token_amount: token_balance,
        slippage_basis_points: Some(slippage),
        recent_blockhash: Some(recent_blockhash),
        with_tip: false,
        extension_params: DexParamEnum::PumpFun(pump_params),
        address_lookup_table_account: None,
        wait_transaction_confirmed: true,
        create_output_token_ata: false,
        close_output_token_ata: false,
        close_mint_token_ata: false,
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy,
        simulate: false,
    };

    if language == Language::Chinese {
        println!("{}", "📤 发送交易到链上...".bright_blue());
    } else {
        println!("{}", "📤 Sending transaction...".bright_blue());
    }

    match client.sell(sell_params).await {
        Ok((success, signatures, error)) => {
            if success {
                if language == Language::Chinese {
                    println!("\n{}", "✅ 卖出成功！".green().bold());
                    println!("   卖出数量: {} tokens", token_balance);
                    for (i, signature) in signatures.iter().enumerate() {
                        if i == 0 || signatures.len() == 1 {
                            println!("   交易签名: {}", signature.to_string().yellow());
                            println!("   🔗 https://solscan.io/tx/{}", signature);
                        } else {
                            println!("   交易签名 {}: {}", i + 1, signature.to_string().yellow());
                            println!("   🔗 https://solscan.io/tx/{}", signature);
                        }
                    }
                    println!("\n{}", "💡 提示: 已收到原生 SOL".bright_yellow());
                } else {
                    println!("\n{}", "✅ Sell successful!".green().bold());
                    println!("   Sold: {} tokens", token_balance);
                    for (i, signature) in signatures.iter().enumerate() {
                        if i == 0 || signatures.len() == 1 {
                            println!("   Signature: {}", signature.to_string().yellow());
                            println!("   🔗 https://solscan.io/tx/{}", signature);
                        } else {
                            println!("   Signature {}: {}", i + 1, signature.to_string().yellow());
                            println!("   🔗 https://solscan.io/tx/{}", signature);
                        }
                    }
                    println!("\n{}", "💡 Received native SOL".bright_yellow());
                }
                Ok(())
            } else {
                let error_msg = error
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                Err(format!("卖出失败: {}", error_msg))
            }
        }
        Err(e) => Err(format!("卖出失败: {}", e)),
    }
}
