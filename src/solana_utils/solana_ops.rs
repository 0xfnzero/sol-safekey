use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    sysvar,
    transaction::Transaction,
};
use std::str::FromStr;

// System program ID - hardcoded for solana-sdk 3.0 compatibility
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Create a transfer instruction (replacement for system_instruction::transfer)
fn create_transfer_instruction(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*from, true),
            solana_sdk::instruction::AccountMeta::new(*to, false),
        ],
        data: {
            let mut data = vec![2, 0, 0, 0]; // Transfer instruction discriminator
            data.extend_from_slice(&lamports.to_le_bytes());
            data
        },
    }
}

/// Solana RPC client wrapper
pub struct SolanaClient {
    rpc_url: String,
}

impl SolanaClient {
    /// Create a new Solana client with the given RPC URL
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    /// Get SOL balance for an account
    pub fn get_sol_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let client = RpcClient::new(self.rpc_url.clone());

        let balance = client.get_balance(pubkey)?;
        Ok(balance)
    }

    /// Get SPL token balance for an account
    pub fn get_token_balance(&self, owner: &Pubkey, mint: &Pubkey) -> Result<u64> {
        let client = RpcClient::new(self.rpc_url.clone());

        let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;
        let ata = get_associated_token_address(owner, mint, &token_program);

        // Try to get token account
        match client.get_token_account_balance(&ata) {
            Ok(balance) => {
                let amount = balance.amount.parse::<u64>()
                    .map_err(|_| anyhow!("Failed to parse token balance"))?;
                Ok(amount)
            }
            Err(_) => Ok(0), // Account doesn't exist, return 0
        }
    }

    /// Transfer SOL from one account to another
    pub fn transfer_sol(
        &self,
        from: &Keypair,
        to: &Pubkey,
        amount: u64,
    ) -> Result<Signature> {
        let client = RpcClient::new(self.rpc_url.clone());

        if amount == 0 {
            return Err(anyhow!("Transfer amount cannot be zero"));
        }

        let balance = self.get_sol_balance(&from.pubkey())?;
        if balance < amount {
            return Err(anyhow!(
                "Insufficient balance. Have: {} lamports, Need: {} lamports",
                balance,
                amount
            ));
        }

        let instruction = create_transfer_instruction(&from.pubkey(), to, amount);
        let recent_blockhash = client.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&from.pubkey()),
            &[from],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }

    /// Transfer SPL tokens from one account to another
    pub fn transfer_token(
        &self,
        from: &Keypair,
        to: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> Result<Signature> {
        let client = RpcClient::new(self.rpc_url.clone());

        if amount == 0 {
            return Err(anyhow!("Transfer amount cannot be zero"));
        }

        let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;
        let from_ata = get_associated_token_address(&from.pubkey(), mint, &token_program);
        let to_ata = get_associated_token_address(to, mint, &token_program);

        let mut instructions = vec![];

        // Create recipient's ATA if it doesn't exist
        if client.get_account(&to_ata).is_err() {
            instructions.push(create_associated_token_account(
                &from.pubkey(),
                to,
                mint,
                &token_program,
            )?);
        }

        // Create transfer instruction
        instructions.push(create_transfer_checked_instruction(
            &from_ata,
            mint,
            &to_ata,
            &from.pubkey(),
            amount,
        )?);

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&from.pubkey()),
            &[from],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }

    /// Wrap SOL to WSOL
    pub fn wrap_sol(&self, keypair: &Keypair, amount: u64) -> Result<Signature> {
        let client = RpcClient::new(self.rpc_url.clone());

        if amount == 0 {
            return Err(anyhow!("Wrap amount cannot be zero"));
        }

        let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
        let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;
        let wsol_ata = get_associated_token_address(&keypair.pubkey(), &wsol_mint, &token_program);

        let mut instructions = vec![];

        // Create WSOL ATA if it doesn't exist
        if client.get_account(&wsol_ata).is_err() {
            instructions.push(create_associated_token_account(
                &keypair.pubkey(),
                &keypair.pubkey(),
                &wsol_mint,
                &token_program,
            )?);
        }

        // Transfer SOL to WSOL account
        instructions.push(create_transfer_instruction(
            &keypair.pubkey(),
            &wsol_ata,
            amount,
        ));

        // Sync native (this tells the token program to update the wrapped balance)
        instructions.push(sync_native_instruction(&wsol_ata, &token_program)?);

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&keypair.pubkey()),
            &[keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }

    /// Unwrap WSOL to SOL
    pub fn unwrap_sol(&self, keypair: &Keypair) -> Result<Signature> {
        let client = RpcClient::new(self.rpc_url.clone());

        let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
        let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;
        let wsol_ata = get_associated_token_address(&keypair.pubkey(), &wsol_mint, &token_program);

        // Check if WSOL account exists
        if client.get_account(&wsol_ata).is_err() {
            return Err(anyhow!("WSOL account does not exist"));
        }

        // Close the WSOL account (this returns all SOL to the owner)
        let instruction = close_account_instruction(
            &wsol_ata,
            &keypair.pubkey(),
            &keypair.pubkey(),
            &token_program,
        )?;

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&keypair.pubkey()),
            &[keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature)
    }

    /// Create a durable nonce account
    /// Returns the nonce account pubkey and transaction signature
    pub fn create_nonce_account(&self, payer: &Keypair) -> Result<(Pubkey, Signature)> {
        let client = RpcClient::new(self.rpc_url.clone());

        // Generate a new keypair for the nonce account
        let nonce_account = Keypair::new();
        let nonce_pubkey = nonce_account.pubkey();

        // Calculate rent-exempt balance for nonce account
        // Nonce account size: 80 bytes
        let rent_exemption = client.get_minimum_balance_for_rent_exemption(80)?;

        // 手动构造系统程序指令（solana-sdk 3.0 兼容）
        let mut instructions = vec![];

        println!("📝 构建Nonce账户创建交易...");

        // 1. CreateAccount 指令 (指令ID = 0)
        println!("   ✓ 添加指令1: CreateAccount (分配空间和租金)");
        let create_account_ix = Instruction {
            program_id: SYSTEM_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),  // from (payer, signer)
                AccountMeta::new(nonce_pubkey, true),     // to (new account, signer)
            ],
            data: {
                let mut data = vec![0u32.to_le_bytes()[0], 0, 0, 0]; // CreateAccount 指令ID = 0
                data.extend_from_slice(&rent_exemption.to_le_bytes());
                data.extend_from_slice(&80u64.to_le_bytes());
                data.extend_from_slice(SYSTEM_PROGRAM_ID.as_ref());
                data
            },
        };
        instructions.push(create_account_ix);

        // 2. InitializeNonceAccount 指令 (指令ID = 6)
        println!("   ✓ 添加指令2: InitializeNonceAccount (设置authority和nonce值)");
        let initialize_nonce_ix = Instruction {
            program_id: SYSTEM_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(nonce_pubkey, false),              // nonce account
                AccountMeta::new_readonly(                           // recent_blockhashes sysvar
                    sysvar::recent_blockhashes::id(),
                    false
                ),
                AccountMeta::new_readonly(                           // rent sysvar
                    sysvar::rent::id(),
                    false
                ),
            ],
            data: {
                let mut data = vec![6u32.to_le_bytes()[0], 0, 0, 0]; // InitializeNonceAccount 指令ID = 6
                data.extend_from_slice(payer.pubkey().as_ref());
                data
            },
        };
        instructions.push(initialize_nonce_ix);

        println!("🚀 发送交易（2个指令将在同一交易中原子性执行）...");
        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[payer, &nonce_account],  // Both payer and nonce_account must sign
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        println!("✅ 交易已确认: {}", signature);

        // 验证 nonce 账户已正确创建和初始化
        println!("🔍 验证Nonce账户...");
        match client.get_account(&nonce_pubkey) {
            Ok(account) => {
                if account.data.len() == 80 {
                    // 解析 nonce 数据
                    let version = u32::from_le_bytes([
                        account.data[0],
                        account.data[1],
                        account.data[2],
                        account.data[3],
                    ]);

                    // 提取 nonce 值（blockhash）
                    let nonce_bytes = &account.data[36..68];
                    let nonce_hex: String = nonce_bytes.iter()
                        .map(|b| format!("{:02x}", b))
                        .collect();

                    // 检查是否为默认值（全0）
                    let is_initialized = !nonce_bytes.iter().all(|&b| b == 0);

                    println!("✅ Nonce账户创建和初始化成功！");
                    println!("   📍 地址: {}", nonce_pubkey);
                    println!("   👤 Authority: {}", payer.pubkey());
                    println!("   💰 余额: {:.6} SOL", account.lamports as f64 / 1e9);
                    println!("   📊 版本: {}", version);

                    if is_initialized {
                        println!("   🔐 Nonce值: {}...{}", &nonce_hex[..16], &nonce_hex[nonce_hex.len()-16..]);
                        println!("   ✅ 状态: 已初始化，可以立即使用");
                    } else {
                        println!("   ❌ Nonce值: 全0（未初始化）");
                        return Err(anyhow!("Nonce account was created but NOT initialized properly"));
                    }
                } else {
                    return Err(anyhow!("Nonce account created but data size incorrect: {} bytes (expected 80)", account.data.len()));
                }
            }
            Err(e) => {
                return Err(anyhow!("Failed to verify nonce account after creation: {}", e));
            }
        }

        Ok((nonce_pubkey, signature))
    }
}

/// Get associated token address
fn get_associated_token_address(
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let associated_token_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap();

    Pubkey::find_program_address(
        &[
            wallet.as_ref(),
            token_program.as_ref(),
            mint.as_ref(),
        ],
        &associated_token_program,
    )
    .0
}

/// Create associated token account instruction
fn create_associated_token_account(
    payer: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction> {
    let associated_token_program = Pubkey::from_str(ASSOCIATED_TOKEN_PROGRAM_ID)?;
    let ata = get_associated_token_address(wallet, mint, token_program);

    Ok(Instruction {
        program_id: associated_token_program,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*payer, true),
            solana_sdk::instruction::AccountMeta::new(ata, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*wallet, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*mint, false),
            solana_sdk::instruction::AccountMeta::new_readonly(
                SYSTEM_PROGRAM_ID,
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(*token_program, false),
        ],
        data: vec![],
    })
}

/// Create transfer checked instruction
fn create_transfer_checked_instruction(
    from: &Pubkey,
    mint: &Pubkey,
    to: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<Instruction> {
    let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;

    // TransferChecked instruction (instruction index: 12)
    let mut data = vec![12];
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(9); // decimals (standard for most tokens, will be ignored for actual execution)

    Ok(Instruction {
        program_id: token_program,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*from, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*mint, false),
            solana_sdk::instruction::AccountMeta::new(*to, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}

/// Create sync native instruction
fn sync_native_instruction(account: &Pubkey, token_program: &Pubkey) -> Result<Instruction> {
    // SyncNative instruction (instruction index: 17)
    Ok(Instruction {
        program_id: *token_program,
        accounts: vec![solana_sdk::instruction::AccountMeta::new(*account, false)],
        data: vec![17],
    })
}

/// Create close account instruction
fn close_account_instruction(
    account: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction> {
    // CloseAccount instruction (instruction index: 9)
    Ok(Instruction {
        program_id: *token_program,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*account, false),
            solana_sdk::instruction::AccountMeta::new(*destination, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*owner, true),
        ],
        data: vec![9],
    })
}

/// Format lamports to SOL with proper decimals
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

/// Format smallest token units to human readable format
pub fn format_token_amount(amount: u64, decimals: u8) -> f64 {
    amount as f64 / 10_f64.powi(decimals as i32)
}

// ==================== Sol-Trade-SDK Integration ====================

#[cfg(feature = "solana-ops")]
use std::sync::Arc;

/// Enhanced Solana client using sol-trade-sdk for optimized operations
#[cfg(feature = "solana-ops")]
pub struct SolanaClientSdk {
    rpc_url: String,
    use_seed_optimize: bool,
}

#[cfg(feature = "solana-ops")]
impl SolanaClientSdk {
    /// Create a new SDK-backed Solana client
    pub fn new(rpc_url: String, use_seed_optimize: bool) -> Self {
        Self {
            rpc_url,
            use_seed_optimize,
        }
    }

    /// Create WSOL ATA using sol-trade-sdk (with seed optimization support)
    pub async fn create_wsol_ata(&self, keypair: &Keypair) -> Result<Signature> {
        use sol_trade_sdk::{SolanaTrade, common::TradeConfig, swqos::SwqosConfig};
        use solana_commitment_config::CommitmentConfig;

        let trade_config = TradeConfig {
            rpc_url: self.rpc_url.clone(),
            swqos_configs: vec![SwqosConfig::Default(self.rpc_url.clone())],
            commitment: CommitmentConfig::confirmed(),
            create_wsol_ata_on_startup: false,  // 不在启动时创建，由手动调用
            use_seed_optimize: self.use_seed_optimize,
        };

        let trade_client = SolanaTrade::new(Arc::new(keypair.insecure_clone()), trade_config).await;
        let signature_str = trade_client.create_wsol_ata().await?;
        Ok(Signature::from_str(&signature_str)?)
    }

    /// Wrap SOL to WSOL using sol-trade-sdk (with seed optimization support)
    pub async fn wrap_sol(&self, keypair: &Keypair, amount: u64) -> Result<Signature> {
        use sol_trade_sdk::{SolanaTrade, common::TradeConfig, swqos::SwqosConfig};
        use solana_commitment_config::CommitmentConfig;

        if amount == 0 {
            return Err(anyhow!("Wrap amount cannot be zero"));
        }

        let trade_config = TradeConfig {
            rpc_url: self.rpc_url.clone(),
            swqos_configs: vec![SwqosConfig::Default(self.rpc_url.clone())],
            commitment: CommitmentConfig::confirmed(),
            create_wsol_ata_on_startup: false,  // 不在启动时创建
            use_seed_optimize: self.use_seed_optimize,
        };

        let trade_client = SolanaTrade::new(Arc::new(keypair.insecure_clone()), trade_config).await;
        let signature_str = trade_client.wrap_sol_to_wsol(amount).await?;
        Ok(Signature::from_str(&signature_str)?)
    }

    /// Unwrap WSOL to SOL using sol-trade-sdk (with seed optimization support)
    pub async fn unwrap_sol(&self, keypair: &Keypair) -> Result<Signature> {
        use sol_trade_sdk::{SolanaTrade, common::TradeConfig, swqos::SwqosConfig};
        use solana_commitment_config::CommitmentConfig;

        let trade_config = TradeConfig {
            rpc_url: self.rpc_url.clone(),
            swqos_configs: vec![SwqosConfig::Default(self.rpc_url.clone())],
            commitment: CommitmentConfig::confirmed(),
            create_wsol_ata_on_startup: false,
            use_seed_optimize: self.use_seed_optimize,
        };

        let trade_client = SolanaTrade::new(Arc::new(keypair.insecure_clone()), trade_config).await;
        let signature_str = trade_client.close_wsol().await?;
        Ok(Signature::from_str(&signature_str)?)
    }

    /// Get SOL balance (reuses existing RPC client)
    pub fn get_sol_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let client = RpcClient::new(self.rpc_url.clone());
        let balance = client.get_balance(pubkey)?;
        Ok(balance)
    }

    /// Get WSOL balance
    pub fn get_wsol_balance(&self, owner: &Pubkey) -> Result<u64> {
        let wsol_mint = Pubkey::from_str(WSOL_MINT)?;
        let token_program = Pubkey::from_str(TOKEN_PROGRAM_ID)?;

        // 根据是否使用 seed 优化计算不同的地址
        let ata = get_associated_token_address(owner, &wsol_mint, &token_program);

        let client = RpcClient::new(self.rpc_url.clone());
        match client.get_token_account_balance(&ata) {
            Ok(balance) => {
                let amount = balance.amount.parse::<u64>()
                    .map_err(|_| anyhow!("Failed to parse token balance"))?;
                Ok(amount)
            }
            Err(_) => Ok(0),
        }
    }
}