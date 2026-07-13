use borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};
use solana_loader_v3_interface::{get_program_data_address, instruction as loader_v3_instruction};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::collections::BTreeMap;
use std::str::FromStr;

pub const SQUADS_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf");
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
const TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SEED_PREFIX: &[u8] = b"multisig";
const SEED_PROGRAM_CONFIG: &[u8] = b"program_config";
const SEED_MULTISIG: &[u8] = b"multisig";
const SEED_TRANSACTION: &[u8] = b"transaction";
const SEED_PROPOSAL: &[u8] = b"proposal";
const SEED_VAULT: &[u8] = b"vault";

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct Member {
    pub key: Pubkey,
    pub permissions: Permissions,
}

#[derive(Clone, Copy, Debug, Default, BorshDeserialize, BorshSerialize)]
pub struct Permissions {
    pub mask: u8,
}

impl Permissions {
    pub fn all() -> Self {
        Self { mask: 0b111 }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, BorshDeserialize)]
pub struct ProgramConfig {
    pub authority: Pubkey,
    pub multisig_creation_fee: u64,
    pub treasury: Pubkey,
    pub _reserved: [u8; 64],
}

#[allow(dead_code)]
#[derive(Clone, Debug, BorshDeserialize)]
pub struct Multisig {
    pub create_key: Pubkey,
    pub config_authority: Pubkey,
    pub threshold: u16,
    pub time_lock: u32,
    pub transaction_index: u64,
    pub stale_transaction_index: u64,
    pub rent_collector: Option<Pubkey>,
    pub bump: u8,
    pub members: Vec<Member>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, BorshDeserialize)]
pub struct Proposal {
    pub multisig: Pubkey,
    pub transaction_index: u64,
    pub status: ProposalStatus,
    pub bump: u8,
    pub approved: Vec<Pubkey>,
    pub rejected: Vec<Pubkey>,
    pub cancelled: Vec<Pubkey>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, BorshDeserialize)]
pub struct VaultTransaction {
    pub multisig: Pubkey,
    pub creator: Pubkey,
    pub index: u64,
    pub bump: u8,
    pub vault_index: u8,
    pub vault_bump: u8,
    pub ephemeral_signer_bumps: Vec<u8>,
    pub message: TransactionMessage,
}

#[allow(dead_code)]
#[derive(Clone, Debug, BorshDeserialize)]
pub enum ProposalStatus {
    Draft { timestamp: i64 },
    Active { timestamp: i64 },
    Rejected { timestamp: i64 },
    Approved { timestamp: i64 },
    Executing,
    Executed { timestamp: i64 },
    Cancelled { timestamp: i64 },
}

impl ProposalStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft { .. } => "draft",
            Self::Active { .. } => "active",
            Self::Rejected { .. } => "rejected",
            Self::Approved { .. } => "approved",
            Self::Executing => "executing",
            Self::Executed { .. } => "executed",
            Self::Cancelled { .. } => "cancelled",
        }
    }
}

#[derive(BorshSerialize)]
struct MultisigCreateArgsV2 {
    config_authority: Option<Pubkey>,
    threshold: u16,
    members: Vec<Member>,
    time_lock: u32,
    rent_collector: Option<Pubkey>,
    memo: Option<String>,
}

#[derive(BorshSerialize)]
struct ProposalCreateArgs {
    transaction_index: u64,
    draft: bool,
}

#[derive(BorshSerialize)]
struct ProposalVoteArgs {
    memo: Option<String>,
}

#[derive(BorshSerialize)]
struct VaultTransactionCreateArgs {
    vault_index: u8,
    ephemeral_signers: u8,
    transaction_message: Vec<u8>,
    memo: Option<String>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct TransactionMessage {
    pub num_signers: u8,
    pub num_writable_signers: u8,
    pub num_writable_non_signers: u8,
    pub account_keys: Vec<Pubkey>,
    pub instructions: Vec<CompiledInstruction>,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct CompiledInstruction {
    pub program_id_index: u8,
    pub account_indexes: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct MessageAddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

#[derive(Default, Debug, Clone)]
struct CompiledKeyMeta {
    is_signer: bool,
    is_writable: bool,
}

pub fn parse_pubkey(value: &str, field: &str) -> Result<Pubkey, String> {
    Pubkey::from_str(value.trim()).map_err(|_| format!("无效的 {}", field))
}

pub fn program_config_pda() -> Pubkey {
    Pubkey::find_program_address(&[SEED_PREFIX, SEED_PROGRAM_CONFIG], &SQUADS_PROGRAM_ID).0
}

pub fn multisig_pda(create_key: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[SEED_PREFIX, SEED_MULTISIG, create_key.as_ref()],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn vault_pda(multisig: &Pubkey, vault_index: u8) -> Pubkey {
    Pubkey::find_program_address(
        &[SEED_PREFIX, multisig.as_ref(), SEED_VAULT, &[vault_index]],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn transaction_pda(multisig: &Pubkey, transaction_index: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig.as_ref(),
            SEED_TRANSACTION,
            transaction_index.to_le_bytes().as_ref(),
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn proposal_pda(multisig: &Pubkey, transaction_index: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[
            SEED_PREFIX,
            multisig.as_ref(),
            SEED_TRANSACTION,
            transaction_index.to_le_bytes().as_ref(),
            SEED_PROPOSAL,
        ],
        &SQUADS_PROGRAM_ID,
    )
    .0
}

pub fn account_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("account:{name}").as_bytes());
    let hash = hasher.finalize();
    let mut out = [0_u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn instruction_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{name}").as_bytes());
    let hash = hasher.finalize();
    let mut out = [0_u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn anchor_data<T: BorshSerialize>(name: &str, args: &T) -> Result<Vec<u8>, String> {
    let mut data = instruction_discriminator(name).to_vec();
    let mut args_data = borsh::to_vec(args).map_err(|e| format!("序列化 Squads 指令失败: {e}"))?;
    data.append(&mut args_data);
    Ok(data)
}

fn anchor_data_empty(name: &str) -> Vec<u8> {
    instruction_discriminator(name).to_vec()
}

pub fn decode_account<T: BorshDeserialize>(data: &[u8], name: &str) -> Result<T, String> {
    let discriminator = account_discriminator(name);
    if data.len() < 8 || data[..8] != discriminator {
        return Err(format!("不是有效的 Squads {name} 账户"));
    }
    T::try_from_slice(&data[8..]).map_err(|e| format!("解析 Squads {name} 账户失败: {e}"))
}

pub fn multisig_create_ix(
    create_key: &Pubkey,
    creator: &Pubkey,
    treasury: &Pubkey,
    threshold: u16,
    members: Vec<Member>,
    time_lock: u32,
    memo: Option<String>,
) -> Result<(Instruction, Pubkey), String> {
    let multisig = multisig_pda(create_key);
    let accounts = vec![
        AccountMeta::new_readonly(program_config_pda(), false),
        AccountMeta::new(*treasury, false),
        AccountMeta::new(multisig, false),
        AccountMeta::new_readonly(*create_key, true),
        AccountMeta::new(*creator, true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];
    let data = anchor_data(
        "multisig_create_v2",
        &MultisigCreateArgsV2 {
            config_authority: None,
            threshold,
            members,
            time_lock,
            rent_collector: Some(*creator),
            memo,
        },
    )?;
    Ok((
        Instruction {
            program_id: SQUADS_PROGRAM_ID,
            accounts,
            data,
        },
        multisig,
    ))
}

pub fn vault_transaction_create_ix(
    multisig: &Pubkey,
    creator: &Pubkey,
    transaction_index: u64,
    vault_index: u8,
    inner_instructions: &[Instruction],
    memo: Option<String>,
) -> Result<(Instruction, Pubkey, Pubkey, TransactionMessage), String> {
    let vault = vault_pda(multisig, vault_index);
    let transaction = transaction_pda(multisig, transaction_index);
    let message = compile_transaction_message(&vault, inner_instructions)?;
    let message_data = serialize_transaction_message_arg(&message)?;
    let data = anchor_data(
        "vault_transaction_create",
        &VaultTransactionCreateArgs {
            vault_index,
            ephemeral_signers: 0,
            transaction_message: message_data,
            memo,
        },
    )?;
    let accounts = vec![
        AccountMeta::new(*multisig, false),
        AccountMeta::new(transaction, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(*creator, true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];
    Ok((
        Instruction {
            program_id: SQUADS_PROGRAM_ID,
            accounts,
            data,
        },
        transaction,
        vault,
        message,
    ))
}

pub fn proposal_create_ix(
    multisig: &Pubkey,
    creator: &Pubkey,
    transaction_index: u64,
    draft: bool,
) -> Result<(Instruction, Pubkey), String> {
    let proposal = proposal_pda(multisig, transaction_index);
    let data = anchor_data(
        "proposal_create",
        &ProposalCreateArgs {
            transaction_index,
            draft,
        },
    )?;
    let accounts = vec![
        AccountMeta::new(*multisig, false),
        AccountMeta::new(proposal, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(*creator, true),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];
    Ok((
        Instruction {
            program_id: SQUADS_PROGRAM_ID,
            accounts,
            data,
        },
        proposal,
    ))
}

pub fn proposal_approve_ix(
    multisig: &Pubkey,
    proposal: &Pubkey,
    member: &Pubkey,
    memo: Option<String>,
) -> Result<Instruction, String> {
    let accounts = vec![
        AccountMeta::new(*multisig, false),
        AccountMeta::new(*member, true),
        AccountMeta::new(*proposal, false),
    ];
    Ok(Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts,
        data: anchor_data("proposal_approve", &ProposalVoteArgs { memo })?,
    })
}

pub fn proposal_reject_ix(
    multisig: &Pubkey,
    proposal: &Pubkey,
    member: &Pubkey,
    memo: Option<String>,
) -> Result<Instruction, String> {
    let accounts = vec![
        AccountMeta::new(*multisig, false),
        AccountMeta::new(*member, true),
        AccountMeta::new(*proposal, false),
    ];
    Ok(Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts,
        data: anchor_data("proposal_reject", &ProposalVoteArgs { memo })?,
    })
}

pub fn vault_transaction_execute_ix(
    multisig: &Pubkey,
    transaction: &Pubkey,
    proposal: &Pubkey,
    member: &Pubkey,
    message: &TransactionMessage,
    vault_index: u8,
    ephemeral_signers: u8,
) -> Result<Instruction, String> {
    if !message.address_table_lookups.is_empty() {
        return Err("暂不支持执行包含 Address Lookup Table 的外部 Squads 提案".to_string());
    }
    let mut accounts = vec![
        AccountMeta::new(*multisig, false),
        AccountMeta::new(*proposal, false),
        AccountMeta::new_readonly(*transaction, false),
        AccountMeta::new_readonly(*member, true),
    ];
    accounts.extend(message_accounts_for_execute(
        message,
        multisig,
        transaction,
        vault_index,
        ephemeral_signers,
    ));
    Ok(Instruction {
        program_id: SQUADS_PROGRAM_ID,
        accounts,
        data: anchor_data_empty("vault_transaction_execute"),
    })
}

pub fn sol_transfer_ix(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut data = vec![2, 0, 0, 0];
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)],
        data,
    }
}

pub fn token_transfer_checked_ix(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    let mut data = vec![12];
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);
    Instruction {
        program_id: TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*source, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(*authority, true),
        ],
        data,
    }
}

pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}

pub fn create_associated_token_account_idempotent_ix(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(associated_token_address(owner, mint), false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![1],
    }
}

pub fn upgrade_program_ix(
    program_id: &Pubkey,
    buffer: &Pubkey,
    authority: &Pubkey,
    spill: &Pubkey,
) -> Instruction {
    loader_v3_instruction::upgrade(program_id, buffer, authority, spill)
}

pub fn set_program_upgrade_authority_ix(
    program_id: &Pubkey,
    current_authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    loader_v3_instruction::set_upgrade_authority(program_id, current_authority, Some(new_authority))
}

pub fn set_buffer_authority_ix(
    buffer: &Pubkey,
    current_authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    loader_v3_instruction::set_buffer_authority(buffer, current_authority, new_authority)
}

pub fn programdata_address(program_id: &Pubkey) -> Pubkey {
    get_program_data_address(program_id)
}

fn compile_transaction_message(
    vault_key: &Pubkey,
    instructions: &[Instruction],
) -> Result<TransactionMessage, String> {
    let mut key_meta_map = BTreeMap::<Pubkey, CompiledKeyMeta>::new();

    for ix in instructions {
        key_meta_map.entry(ix.program_id).or_default();
        for account in &ix.accounts {
            let meta = key_meta_map.entry(account.pubkey).or_default();
            meta.is_signer |= account.is_signer;
            meta.is_writable |= account.is_writable;
        }
    }

    {
        let meta = key_meta_map.entry(*vault_key).or_default();
        meta.is_signer = true;
        meta.is_writable = true;
    }

    key_meta_map.remove(vault_key);

    let writable_signers = std::iter::once(*vault_key)
        .chain(
            key_meta_map
                .iter()
                .filter_map(|(key, meta)| (meta.is_signer && meta.is_writable).then_some(*key)),
        )
        .collect::<Vec<_>>();
    let readonly_signers = key_meta_map
        .iter()
        .filter_map(|(key, meta)| (meta.is_signer && !meta.is_writable).then_some(*key))
        .collect::<Vec<_>>();
    let writable_non_signers = key_meta_map
        .iter()
        .filter_map(|(key, meta)| (!meta.is_signer && meta.is_writable).then_some(*key))
        .collect::<Vec<_>>();
    let readonly_non_signers = key_meta_map
        .iter()
        .filter_map(|(key, meta)| (!meta.is_signer && !meta.is_writable).then_some(*key))
        .collect::<Vec<_>>();

    let num_signers = checked_u8(writable_signers.len() + readonly_signers.len())?;
    let num_writable_signers = checked_u8(writable_signers.len())?;
    let num_writable_non_signers = checked_u8(writable_non_signers.len())?;

    let account_keys = writable_signers
        .into_iter()
        .chain(readonly_signers)
        .chain(writable_non_signers)
        .chain(readonly_non_signers)
        .collect::<Vec<_>>();

    let mut compiled_instructions = Vec::with_capacity(instructions.len());
    for ix in instructions {
        let program_id_index = account_index(&account_keys, &ix.program_id)?;
        let account_indexes = ix
            .accounts
            .iter()
            .map(|account| account_index(&account_keys, &account.pubkey))
            .collect::<Result<Vec<_>, _>>()?;
        compiled_instructions.push(CompiledInstruction {
            program_id_index,
            account_indexes,
            data: ix.data.clone(),
        });
    }

    Ok(TransactionMessage {
        num_signers,
        num_writable_signers,
        num_writable_non_signers,
        account_keys,
        instructions: compiled_instructions,
        address_table_lookups: Vec::new(),
    })
}

fn checked_u8(value: usize) -> Result<u8, String> {
    u8::try_from(value).map_err(|_| "Squads 交易账户数量过多".to_string())
}

fn checked_u16(value: usize, field: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("Squads {field} 数据过大"))
}

fn write_small_vec_u8<T, F>(
    out: &mut Vec<u8>,
    values: &[T],
    field: &str,
    mut write_item: F,
) -> Result<(), String>
where
    F: FnMut(&mut Vec<u8>, &T) -> Result<(), String>,
{
    out.push(checked_u8(values.len()).map_err(|_| format!("Squads {field} 数量过多"))?);
    for value in values {
        write_item(out, value)?;
    }
    Ok(())
}

fn write_small_bytes_u8(out: &mut Vec<u8>, values: &[u8], field: &str) -> Result<(), String> {
    out.push(checked_u8(values.len()).map_err(|_| format!("Squads {field} 数量过多"))?);
    out.extend_from_slice(values);
    Ok(())
}

fn write_small_bytes_u16(out: &mut Vec<u8>, values: &[u8], field: &str) -> Result<(), String> {
    out.extend_from_slice(&checked_u16(values.len(), field)?.to_le_bytes());
    out.extend_from_slice(values);
    Ok(())
}

fn serialize_transaction_message_arg(message: &TransactionMessage) -> Result<Vec<u8>, String> {
    let mut out = vec![
        message.num_signers,
        message.num_writable_signers,
        message.num_writable_non_signers,
    ];

    write_small_vec_u8(
        &mut out,
        &message.account_keys,
        "account_keys",
        |out, key| {
            out.extend_from_slice(key.as_ref());
            Ok(())
        },
    )?;
    write_small_vec_u8(
        &mut out,
        &message.instructions,
        "instructions",
        |out, ix| {
            out.push(ix.program_id_index);
            write_small_bytes_u8(out, &ix.account_indexes, "instruction account_indexes")?;
            write_small_bytes_u16(out, &ix.data, "instruction data")
        },
    )?;
    write_small_vec_u8(
        &mut out,
        &message.address_table_lookups,
        "address_table_lookups",
        |out, lookup| {
            out.extend_from_slice(lookup.account_key.as_ref());
            write_small_bytes_u8(out, &lookup.writable_indexes, "lookup writable_indexes")?;
            write_small_bytes_u8(out, &lookup.readonly_indexes, "lookup readonly_indexes")
        },
    )?;

    Ok(out)
}

fn account_index(account_keys: &[Pubkey], pubkey: &Pubkey) -> Result<u8, String> {
    account_keys
        .iter()
        .position(|key| key == pubkey)
        .ok_or_else(|| "Squads 交易消息缺少账户".to_string())
        .and_then(checked_u8)
}

fn message_accounts_for_execute(
    message: &TransactionMessage,
    multisig: &Pubkey,
    transaction: &Pubkey,
    vault_index: u8,
    ephemeral_signers: u8,
) -> Vec<AccountMeta> {
    let vault = vault_pda(multisig, vault_index);
    let ephemeral_signer_pdas = (0..ephemeral_signers)
        .map(|index| {
            Pubkey::find_program_address(
                &[
                    SEED_PREFIX,
                    transaction.as_ref(),
                    b"ephemeral_signer",
                    index.to_le_bytes().as_ref(),
                ],
                &SQUADS_PROGRAM_ID,
            )
            .0
        })
        .collect::<Vec<_>>();

    message
        .account_keys
        .iter()
        .enumerate()
        .map(|(index, pubkey)| {
            let is_writable = is_static_writable_index(message, index);
            let is_signer = is_signer_index(message, index)
                && pubkey != &vault
                && !ephemeral_signer_pdas.contains(pubkey);
            AccountMeta {
                pubkey: *pubkey,
                is_writable,
                is_signer,
            }
        })
        .collect()
}

fn is_static_writable_index(message: &TransactionMessage, index: usize) -> bool {
    let num_signers = usize::from(message.num_signers);
    let num_writable_signers = usize::from(message.num_writable_signers);
    let num_writable_non_signers = usize::from(message.num_writable_non_signers);

    if index < num_writable_signers {
        return true;
    }
    if index >= num_signers {
        let non_signer_index = index.saturating_sub(num_signers);
        return non_signer_index < num_writable_non_signers;
    }
    false
}

fn is_signer_index(message: &TransactionMessage, index: usize) -> bool {
    index < usize::from(message.num_signers)
}
