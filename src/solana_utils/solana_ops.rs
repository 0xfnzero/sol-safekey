use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

// System program ID constant
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Create a transfer instruction (replacement for system_instruction::transfer)
fn create_transfer_instruction(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let system_program_id = Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap();

    Instruction {
        program_id: system_program_id,
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

        let system_program = Pubkey::from_str(SYSTEM_PROGRAM_ID)?;

        // Create nonce account instructions
        let mut instructions = vec![];

        // 1. Create account
        let create_account_ix = Instruction {
            program_id: system_program,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(payer.pubkey(), true),
                solana_sdk::instruction::AccountMeta::new(nonce_pubkey, true),
            ],
            data: {
                let mut data = vec![0, 0, 0, 0]; // CreateAccount instruction
                data.extend_from_slice(&rent_exemption.to_le_bytes());
                data.extend_from_slice(&80u64.to_le_bytes()); // space
                data.extend_from_slice(system_program.as_ref());
                data
            },
        };
        instructions.push(create_account_ix);

        // 2. Initialize nonce account
        let initialize_nonce_ix = Instruction {
            program_id: system_program,
            accounts: vec![
                solana_sdk::instruction::AccountMeta::new(nonce_pubkey, false),
                solana_sdk::instruction::AccountMeta::new_readonly(
                    solana_sdk::sysvar::recent_blockhashes::ID,
                    false,
                ),
                solana_sdk::instruction::AccountMeta::new_readonly(
                    solana_sdk::sysvar::rent::ID,
                    false,
                ),
            ],
            data: {
                let mut data = vec![6, 0, 0, 0]; // InitializeNonceAccount instruction
                data.extend_from_slice(payer.pubkey().as_ref());
                data
            },
        };
        instructions.push(initialize_nonce_ix);

        let recent_blockhash = client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[payer, &nonce_account],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
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
                Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
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