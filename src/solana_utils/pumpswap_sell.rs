// PumpSwap 卖出功能模块
// 实现通过 sol-trade-sdk 在 PumpSwap 上卖出代币

use anyhow::Result;
use colored::Colorize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;
use std::sync::Arc;

// Import Language from operations module
use crate::operations::Language;

#[cfg(feature = "sol-trade-sdk")]
use sol_trade_sdk::{
    common::{
        fast_fn::get_associated_token_address_with_program_id_fast_use_seed,
        TradeConfig, GasFeeStrategy,
    },
    swqos::SwqosConfig,
    trading::{
        core::params::{PumpSwapParams, DexParamEnum},
        factory::DexType,
    },
    SolanaTrade, TradeSellParams, TradeTokenType,
};
use solana_commitment_config::CommitmentConfig;

/// 处理 PumpSwap 卖出操作
///
/// # 参数
/// * `keypair` - 用于签名交易的密钥对
/// * `mint` - 要卖出的代币地址
/// * `rpc_url` - Solana RPC 端点
/// * `slippage` - 滑点容忍度(基点,如 100 = 1%)
#[cfg(feature = "sol-trade-sdk")]
pub async fn handle_pumpswap_sell(
    keypair: &Keypair,
    mint: &str,
    rpc_url: &str,
    slippage: u64,
) -> Result<()> {
    println!("\n{}", "🔥 PumpSwap 卖出操作".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════".cyan());

    // 解析 mint 地址
    let mint_pubkey = Pubkey::from_str(mint)
        .map_err(|e| anyhow::anyhow!("无效的代币地址: {}", e))?;

    println!("📍 代币地址: {}", mint.yellow());
    println!("🌐 RPC: {}", rpc_url);
    println!("📊 滑点容忍度: {}%", slippage as f64 / 100.0);

    // 询问是否启用 seed 优化
    let use_seed = ask_use_seed()?;

    println!("\n{}", "🔍 检查代币余额...".cyan());

    // 初始化客户端
    let payer = Arc::new(keypair.insecure_clone());
    let commitment = CommitmentConfig::confirmed();
    let swqos_configs: Vec<SwqosConfig> = vec![SwqosConfig::Default(rpc_url.to_string())];

    // 创建 TradeConfig，根据用户选择设置 use_seed_optimize
    let trade_config = TradeConfig {
        rpc_url: rpc_url.to_string(),
        swqos_configs,
        commitment,
        create_wsol_ata_on_startup: false,
        use_seed_optimize: use_seed,
    };

    let client = SolanaTrade::new(payer.clone(), trade_config).await;

    // 检查代币余额和 ATA 信息
    let (token_balance, decimals, token_program) = check_token_balance(
        &client,
        &mint_pubkey,
        &keypair.pubkey(),
        use_seed,
    ).await?;

    if token_balance == 0 {
        return Err(anyhow::anyhow!("❌ 代币余额为 0，无法卖出"));
    }

    let readable_balance = token_balance as f64 / 10_f64.powi(decimals as i32);
    println!("💰 代币余额: {} (原始数量: {})", readable_balance.to_string().green(), token_balance);
    println!("🔧 Token Program: {}", token_program);

    // 二次确认
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

    println!("\n{}", "📡 从链上获取PumpSwap池子参数...".cyan());
    println!("   正在查询mint: {}", mint_pubkey);
    println!("   Token Program: {}", token_program);

    // 从链上获取PumpSwap参数
    let pump_params = match PumpSwapParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await {
        Ok(params) => {
            println!("✅ 找到PumpSwap池子");
            println!("   Pool: {}", params.pool);
            println!("   Base Mint: {}", params.base_mint);
            println!("   Quote Mint: {}", params.quote_mint);
            println!("   Base Reserves: {}", params.pool_base_token_reserves);
            println!("   Quote Reserves: {}", params.pool_quote_token_reserves);
            params
        }
        Err(e) => {
            println!("{}", "❌ 获取PumpSwap池子失败".red().bold());
            println!("错误详情: {}", e);
            println!();
            println!("{}", "🔍 可能的原因:".yellow());
            println!("   1. RPC节点不支持getProgramAccounts查询");
            println!("   2. 查询超时或被限流");
            println!("   3. Token Program类型不匹配 (Token-2022 vs Token)");
            println!();
            println!("{}", "💡 建议:".bright_cyan());
            println!("   1. 更换RPC节点（推荐使用付费RPC）");
            println!("   2. 在 pump.fun 或 dexscreener.com 上查找pool地址");
            println!("   3. 确认mint地址: {}", mint_pubkey);
            println!();
            println!("{}", "   如果确认代币在PumpSwap，可能是RPC限制导致".yellow());

            return Err(anyhow::anyhow!("无法获取PumpSwap池子参数，请检查RPC或稍后重试"));
        }
    };

    // 获取最新的 blockhash
    let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await?;

    // 配置 Gas 策略
    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_global_fee_strategy(
        150000, 150000, 500000, 500000,
        0.001, 0.001
    );

    println!("\n{}", "🚀 构建卖出交易...".cyan());

    // 构建卖出参数
    let sell_params = TradeSellParams {
        dex_type: DexType::PumpSwap,
        output_token_type: TradeTokenType::WSOL,
        mint: mint_pubkey,
        input_token_amount: token_balance,  // 全部卖出
        slippage_basis_points: Some(slippage),
        recent_blockhash: Some(recent_blockhash),
        with_tip: false,
        extension_params: DexParamEnum::PumpSwap(pump_params),
        address_lookup_table_account: None,
        wait_transaction_confirmed: true,
        create_output_token_ata: true,   // 创建 WSOL ATA
        close_output_token_ata: false,   // 不自动关闭 WSOL ATA
        close_mint_token_ata: false,     // 不关闭代币 ATA（可能还有灰尘）
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy,
        simulate: false,
    };

    println!("{}", "📤 发送交易到链上...".bright_blue());

    // 执行卖出
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
            } else {
                let error_msg = error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
                return Err(anyhow::anyhow!("卖出失败: {}", error_msg));
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("卖出失败: {}", e));
        }
    }

    println!("\n{}", "💡 提示: WSOL 已收到，可以使用 unwrap-sol 命令解包为 SOL".bright_yellow());

    Ok(())
}

/// 询问用户是否启用 seed 优化
fn ask_use_seed() -> Result<bool> {
    println!("\n{}", "🔧 Seed 优化配置".bright_cyan());
    println!("   Seed 优化用于创建优化的 ATA 地址，可以节省交易费用");
    println!("   如果你的代币 ATA 是通过标准方式创建的，请选择 'no'");
    println!("   如果不确定，建议选择 'yes'（默认）");

    print!("\n{} ", "❓ 启用 Seed 优化? (yes/no, 默认 yes):".yellow());
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    // 默认为 yes：空输入或 yes/y 都启用，只有明确输入 no/n 才禁用
    let use_seed = input.is_empty() || input == "yes" || input == "y";

    if use_seed {
        println!("{}", "✅ 已启用 Seed 优化".green());
    } else {
        println!("{}", "✅ 使用标准 ATA".green());
    }

    Ok(use_seed)
}

/// 检查代币余额
///
/// 返回: (余额, decimals, token_program)
#[cfg(feature = "sol-trade-sdk")]
async fn check_token_balance(
    client: &SolanaTrade,
    mint: &Pubkey,
    owner: &Pubkey,
    use_seed: bool,
) -> Result<(u64, u8, Pubkey)> {
    // 获取 mint 账户信息
    let mint_account = client.infrastructure.rpc.get_account(mint).await
        .map_err(|e| anyhow::anyhow!("获取代币账户失败: {}", e))?;

    let token_program = mint_account.owner;

    // 尝试获取代币余额（先尝试标准 ATA）
    let standard_ata = get_associated_token_address_with_program_id_fast_use_seed(
        owner,
        mint,
        &token_program,
        false,  // 不使用 seed
    );

    println!("   检查标准 ATA: {}", standard_ata);

    match client.infrastructure.rpc.get_token_account_balance(&standard_ata).await {
        Ok(balance) => {
            let amount = balance.amount.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("解析余额失败"))?;
            let decimals = balance.decimals;

            println!("   ✅ 找到标准 ATA");
            return Ok((amount, decimals, token_program));
        }
        Err(_) => {
            println!("   ⚠️ 标准 ATA 不存在");
        }
    }

    // 如果启用了 seed，尝试 seed 优化的 ATA
    if use_seed {
        let seed_ata = get_associated_token_address_with_program_id_fast_use_seed(
            owner,
            mint,
            &token_program,
            true,  // 使用 seed
        );

        println!("   检查 Seed ATA: {}", seed_ata);

        match client.infrastructure.rpc.get_token_account_balance(&seed_ata).await {
            Ok(balance) => {
                let amount = balance.amount.parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("解析余额失败"))?;
                let decimals = balance.decimals;

                println!("   ✅ 找到 Seed ATA");
                return Ok((amount, decimals, token_program));
            }
            Err(_) => {
                println!("   ⚠️ Seed ATA 也不存在");
            }
        }
    }

    Err(anyhow::anyhow!("未找到代币账户，余额为 0"))
}

/// 处理 PumpSwap 卖出操作（无交互式提示版本，用于主菜单调用）
///
/// # 参数
/// * `keypair` - 用于签名交易的密钥对
/// * `mint` - 要卖出的代币地址
/// * `rpc_url` - Solana RPC 端点
/// * `slippage` - 滑点容忍度(基点,如 100 = 1%)
/// * `use_seed` - 是否使用 seed 优化
/// * `language` - 界面语言
#[cfg(feature = "sol-trade-sdk")]
pub async fn handle_pumpswap_sell_no_prompt(
    keypair: &Keypair,
    mint: &str,
    rpc_url: &str,
    slippage: u64,
    use_seed: bool,
    language: Language,
    skip_confirmation: bool,  // 新增参数：是否跳过确认
) -> Result<(), String> {
    // 解析 mint 地址
    let mint_pubkey = Pubkey::from_str(mint)
        .map_err(|e| format!("无效的代币地址: {}", e))?;

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

    // 初始化客户端
    let payer = Arc::new(keypair.insecure_clone());
    let commitment = CommitmentConfig::confirmed();
    let swqos_configs: Vec<SwqosConfig> = vec![SwqosConfig::Default(rpc_url.to_string())];

    // 创建 TradeConfig
    let trade_config = TradeConfig {
        rpc_url: rpc_url.to_string(),
        swqos_configs,
        commitment,
        create_wsol_ata_on_startup: false,
        use_seed_optimize: use_seed,
    };

    let client = SolanaTrade::new(payer.clone(), trade_config).await;

    // 检查代币余额和 ATA 信息
    let (token_balance, decimals, token_program) = check_token_balance(
        &client,
        &mint_pubkey,
        &keypair.pubkey(),
        use_seed,
    ).await
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
        println!("💰 代币余额: {} (原始数量: {})", readable_balance.to_string().green(), token_balance);
        println!("🔧 Token Program: {}", token_program);
    } else {
        println!("💰 Token Balance: {} (raw: {})", readable_balance.to_string().green(), token_balance);
        println!("🔧 Token Program: {}", token_program);
    }

    // 二次确认（如果需要）
    if !skip_confirmation {
        if language == Language::Chinese {
            println!("\n{}", "❓ 确认全部卖出? (yes/no, 默认 yes): ".yellow());
        } else {
            println!("\n{}", "❓ Confirm sell all? (yes/no, default: yes): ".yellow());
        }

        use std::io::{self, Write};
        print!("{}", if language == Language::Chinese {
            "请输入 (yes/no, 默认 yes): "
        } else {
            "Enter (yes/no, default: yes): "
        });
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
        println!("\n{}", "📡 从链上获取池子参数...".cyan());
    } else {
        println!("\n{}", "📡 Fetching pool parameters...".cyan());
    }

    // 从链上获取 PumpSwap 参数
    let pump_params = PumpSwapParams::from_mint_by_rpc(&client.infrastructure.rpc, &mint_pubkey).await
        .map_err(|e| format!("获取池子参数失败: {}", e))?;

    if language == Language::Chinese {
        println!("✅ 池子参数已获取");
        println!("   Pool: {}", pump_params.pool);
        println!("   Base Reserves: {}", pump_params.pool_base_token_reserves);
        println!("   Quote Reserves: {}", pump_params.pool_quote_token_reserves);
    } else {
        println!("✅ Pool parameters fetched");
        println!("   Pool: {}", pump_params.pool);
        println!("   Base Reserves: {}", pump_params.pool_base_token_reserves);
        println!("   Quote Reserves: {}", pump_params.pool_quote_token_reserves);
    }

    // 获取最新的 blockhash
    let recent_blockhash = client.infrastructure.rpc.get_latest_blockhash().await
        .map_err(|e| format!("获取blockhash失败: {}", e))?;

    // 配置 Gas 策略
    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_global_fee_strategy(
        150000, 150000, 500000, 500000,
        0.001, 0.001
    );

    if language == Language::Chinese {
        println!("\n{}", "🚀 构建卖出交易...".cyan());
    } else {
        println!("\n{}", "🚀 Building sell transaction...".cyan());
    }

    // 构建卖出参数
    let sell_params = TradeSellParams {
        dex_type: DexType::PumpSwap,
        output_token_type: TradeTokenType::WSOL,
        mint: mint_pubkey,
        input_token_amount: token_balance,  // 全部卖出
        slippage_basis_points: Some(slippage),
        recent_blockhash: Some(recent_blockhash),
        with_tip: false,
        extension_params: DexParamEnum::PumpSwap(pump_params),
        address_lookup_table_account: None,
        wait_transaction_confirmed: true,
        create_output_token_ata: true,   // 创建 WSOL ATA
        close_output_token_ata: false,   // 不自动关闭 WSOL ATA
        close_mint_token_ata: false,     // 不关闭代币 ATA（可能还有灰尘）
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

    // 执行卖出
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
                    println!("\n{}", "💡 提示: WSOL 已收到，可以使用 'Unwrap WSOL → SOL' 功能解包为 SOL".bright_yellow());
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
                    println!("\n{}", "💡 Tip: WSOL received, use 'Unwrap WSOL → SOL' to unwrap to SOL".bright_yellow());
                }
                Ok(())
            } else {
                let error_msg = error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string());
                Err(format!("卖出失败: {}", error_msg))
            }
        }
        Err(e) => {
            Err(format!("卖出失败: {}", e))
        }
    }
}
