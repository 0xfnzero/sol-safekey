use agave_syscalls::create_program_runtime_environment_v1;
use sha2::{Digest, Sha256};
use solana_loader_v3_interface::state::UpgradeableLoaderState;
use solana_program_runtime::{
    execution_budget::SVMTransactionExecutionBudget, invoke_context::InvokeContext,
};
use solana_sbpf::{elf::Executable, verifier::RequisiteVerifier};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use solana_svm_feature_set::SVMFeatureSet;
use std::{str::FromStr, sync::Arc};
use zeroize::Zeroize;

pub const MAX_PROGRAM_KEYPAIR_JSON_BYTES: usize = 4 * 1024;
pub const UPGRADEABLE_LOADER_ID: &str = "BPFLoaderUpgradeab1e11111111111111111111111";
pub const OFFLINE_SBF_VERIFIER_AGAVE_VERSION: &str = "3.1.12";
pub const DEPLOY_FEE_RATE_RESERVE_BPS: u64 = 2_000;
pub const DEPLOY_RECOVERY_WRITE_PERCENT: usize = 5;
pub const DEPLOY_MIN_RECOVERY_WRITES: usize = 8;

const ANCHOR_PROGRAM_MARKERS: &[&[u8]] = &[
    b"DeclaredProgramIdMismatch",
    b"The declared program id does not match the actual program id",
    b"AnchorError occurred",
];

#[derive(Debug, PartialEq, Eq)]
pub struct DeploymentReadback {
    pub deployed_slot: u64,
    pub programdata_len: usize,
    pub program_sha256: String,
    pub upgrade_authority: Pubkey,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BufferWritePlan {
    pub completed_chunks: usize,
    pub pending_chunk_indexes: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeploymentFeeBudget {
    pub estimated_fees_lamports: u64,
    pub fee_rate_reserve_lamports: u64,
    pub recovery_write_reserve_lamports: u64,
    pub total_fee_budget_lamports: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeploymentRentBudget {
    pub final_rent_lamports: u64,
    pub required_rent_balance_lamports: u64,
}

pub fn deployment_rent_budget(
    buffer_lamports: u64,
    program_lamports: u64,
    programdata_lamports: u64,
    resume_buffer_lamports: Option<u64>,
) -> Result<DeploymentRentBudget, String> {
    let final_rent_lamports = program_lamports
        .checked_add(programdata_lamports)
        .ok_or_else(|| "Final Program and ProgramData rent total overflow".to_string())?;

    // The deploy transaction creates Program before the loader drains Buffer to fund ProgramData.
    let additional_programdata_lamports = match resume_buffer_lamports {
        Some(available_buffer_lamports) => {
            programdata_lamports.saturating_sub(available_buffer_lamports)
        }
        None => buffer_lamports.max(programdata_lamports),
    };
    let required_rent_balance_lamports = program_lamports
        .checked_add(additional_programdata_lamports)
        .ok_or_else(|| "Deployment peak rent balance overflow".to_string())?;

    Ok(DeploymentRentBudget {
        final_rent_lamports,
        required_rent_balance_lamports,
    })
}

pub fn deployment_fee_budget(
    estimated_fees_lamports: u64,
    estimated_write_fee_lamports: u64,
    pending_write_count: usize,
) -> Result<DeploymentFeeBudget, String> {
    let fee_rate_reserve_lamports = estimated_fees_lamports
        .checked_mul(DEPLOY_FEE_RATE_RESERVE_BPS)
        .and_then(|value| value.checked_add(9_999))
        .map(|value| value / 10_000)
        .ok_or_else(|| "Deployment fee-rate reserve overflow".to_string())?;
    let recovery_write_count = if pending_write_count == 0 {
        0
    } else {
        pending_write_count
            .checked_mul(DEPLOY_RECOVERY_WRITE_PERCENT)
            .and_then(|value| value.checked_add(99))
            .map(|value| value / 100)
            .ok_or_else(|| "Deployment recovery-write count overflow".to_string())?
            .max(DEPLOY_MIN_RECOVERY_WRITES)
    };
    let recovery_write_count = u64::try_from(recovery_write_count)
        .map_err(|_| "Deployment recovery-write count is too large".to_string())?;
    let recovery_write_reserve_lamports = estimated_write_fee_lamports
        .checked_mul(recovery_write_count)
        .ok_or_else(|| "Deployment recovery-write reserve overflow".to_string())?;
    let total_fee_budget_lamports = estimated_fees_lamports
        .checked_add(fee_rate_reserve_lamports)
        .and_then(|value| value.checked_add(recovery_write_reserve_lamports))
        .ok_or_else(|| "Deployment fee budget overflow".to_string())?;

    Ok(DeploymentFeeBudget {
        estimated_fees_lamports,
        fee_rate_reserve_lamports,
        recovery_write_reserve_lamports,
        total_fee_budget_lamports,
    })
}

pub fn parse_program_keypair_json(json: &str) -> Result<Keypair, String> {
    if json.is_empty() {
        return Err("Program keypair JSON cannot be empty".to_string());
    }
    if json.len() > MAX_PROGRAM_KEYPAIR_JSON_BYTES {
        return Err("Program keypair JSON is too large".to_string());
    }

    let mut bytes: Vec<u8> = serde_json::from_str(json)
        .map_err(|_| "Program keypair must be a JSON array of 64 bytes".to_string())?;
    if bytes.len() != 64 {
        bytes.zeroize();
        return Err("Program keypair must contain exactly 64 bytes".to_string());
    }

    let keypair = Keypair::try_from(bytes.as_slice())
        .map_err(|_| "Program keypair bytes are invalid or inconsistent".to_string());
    bytes.zeroize();
    keypair
}

pub fn require_program_id(keypair: &Keypair, expected_program_id: &str) -> Result<Pubkey, String> {
    let expected = Pubkey::from_str(expected_program_id.trim())
        .map_err(|_| "Expected Program ID is invalid".to_string())?;
    if keypair.pubkey() != expected {
        return Err(format!(
            "Program keypair resolves to {}, expected {}",
            keypair.pubkey(),
            expected
        ));
    }
    Ok(expected)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn require_sha256(bytes: &[u8], expected_sha256: &str) -> Result<String, String> {
    let expected = expected_sha256.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Expected Program SHA-256 must contain exactly 64 hex characters".to_string());
    }

    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(format!("Program SHA-256 is {actual}, expected {expected}"));
    }
    Ok(actual)
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

pub fn looks_like_anchor_program(program: &[u8]) -> bool {
    ANCHOR_PROGRAM_MARKERS
        .iter()
        .any(|marker| contains_bytes(program, marker))
}

pub fn require_anchor_declared_program_id(
    program: &[u8],
    expected_program_id: &Pubkey,
) -> Result<(), String> {
    if !looks_like_anchor_program(program) {
        return Ok(());
    }
    let expected_program_id_bytes = expected_program_id.to_bytes();
    if contains_bytes(program, &expected_program_id_bytes) {
        return Ok(());
    }
    Err(format!(
        "Anchor .so 编译产物中的 declare_id! 与目标 Program ID 不匹配：当前目标 Program ID 为 {expected_program_id}，但 .so 内未包含该地址。请执行 clean 后重新编译，或选择与该 .so 匹配的 Program keypair。"
    ))
}

/// Applies the strictest offline SBF loading and bytecode verification available
/// to the Agave 3.1.12 runtime linked into this binary. `all_enabled()` is a
/// capability ceiling for that release, not a claim about current cluster feature
/// activation or compatibility with features introduced after 3.1.12.
pub fn verify_sbf_elf(program: &[u8]) -> Result<(), String> {
    let feature_set = SVMFeatureSet::all_enabled();
    let compute_budget =
        SVMTransactionExecutionBudget::new_with_defaults(feature_set.raise_cpi_nesting_limit_to_8);
    let runtime_environment = create_program_runtime_environment_v1(
        &feature_set,
        &compute_budget,
        true,  // Reject ELF forms which the deployment path rejects.
        false, // Do not enable debugger-only runtime behavior.
    )
    .map_err(|error| {
        format!(
            "Failed to construct Agave {OFFLINE_SBF_VERIFIER_AGAVE_VERSION} SBF verifier: {error}"
        )
    })?;

    let executable =
        Executable::<InvokeContext>::load(program, Arc::new(runtime_environment)).map_err(
            |error| {
                format!(
                    "Agave {OFFLINE_SBF_VERIFIER_AGAVE_VERSION} SBF runtime loader rejected the ELF: {error}"
                )
            },
        )?;
    executable.verify::<RequisiteVerifier>().map_err(|error| {
        format!(
            "Agave {OFFLINE_SBF_VERIFIER_AGAVE_VERSION} RequisiteVerifier rejected the program: {error}"
        )
    })
}

pub fn verify_resume_buffer(
    expected_authority: &Pubkey,
    expected_program: &[u8],
    chunk_size: usize,
    owner: &Pubkey,
    executable: bool,
    account_data: &[u8],
) -> Result<BufferWritePlan, String> {
    if chunk_size == 0 {
        return Err("Buffer write chunk size cannot be zero".to_string());
    }
    let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID)
        .map_err(|_| "Upgradeable loader ID is invalid".to_string())?;
    if owner != &loader_id {
        return Err("Resume buffer is not owned by the upgradeable loader".to_string());
    }
    if executable {
        return Err("Resume buffer must not be executable".to_string());
    }

    let metadata_len = UpgradeableLoaderState::size_of_buffer_metadata();
    let expected_len = UpgradeableLoaderState::size_of_buffer(expected_program.len());
    if account_data.len() != expected_len || account_data.len() < metadata_len {
        return Err(format!(
            "Resume buffer length is {}; expected {}",
            account_data.len(),
            expected_len
        ));
    }
    let state: UpgradeableLoaderState = bincode::deserialize(&account_data[..metadata_len])
        .map_err(|_| "Resume buffer has invalid loader state".to_string())?;
    match state {
        UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        } if authority == *expected_authority => {}
        UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        } => {
            return Err(format!(
                "Resume buffer authority is {authority}; expected {expected_authority}"
            ));
        }
        UpgradeableLoaderState::Buffer {
            authority_address: None,
        } => return Err("Resume buffer has no authority".to_string()),
        _ => return Err("Resume account is not an upgradeable-loader Buffer".to_string()),
    }

    let buffer_program = &account_data[metadata_len..];
    let mut completed_chunks = 0usize;
    let mut pending_chunk_indexes = Vec::new();
    for (index, expected_chunk) in expected_program.chunks(chunk_size).enumerate() {
        let start = index
            .checked_mul(chunk_size)
            .ok_or_else(|| "Buffer chunk offset overflow".to_string())?;
        let end = start
            .checked_add(expected_chunk.len())
            .ok_or_else(|| "Buffer chunk length overflow".to_string())?;
        let actual_chunk = &buffer_program[start..end];
        if actual_chunk == expected_chunk {
            completed_chunks = completed_chunks.saturating_add(1);
        } else if actual_chunk.iter().all(|byte| *byte == 0) {
            pending_chunk_indexes.push(index);
        } else {
            return Err(format!(
                "Resume buffer contains unexpected non-zero data at chunk {index} (offset {start})"
            ));
        }
    }

    Ok(BufferWritePlan {
        completed_chunks,
        pending_chunk_indexes,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn verify_deployment_readback(
    program_id: &Pubkey,
    expected_upgrade_authority: &Pubkey,
    expected_program: &[u8],
    max_data_len: usize,
    program_owner: &Pubkey,
    program_executable: bool,
    program_account_data: &[u8],
    programdata_owner: &Pubkey,
    programdata_account_data: &[u8],
) -> Result<DeploymentReadback, String> {
    let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID)
        .map_err(|_| "Upgradeable loader ID is invalid".to_string())?;
    if program_owner != &loader_id || programdata_owner != &loader_id {
        return Err("Program or ProgramData is not owned by the upgradeable loader".to_string());
    }
    if !program_executable {
        return Err("Program account is not executable".to_string());
    }

    let program_state: UpgradeableLoaderState = bincode::deserialize(program_account_data)
        .map_err(|_| "Program account has invalid loader state".to_string())?;
    let expected_programdata = solana_loader_v3_interface::get_program_data_address(program_id);
    match program_state {
        UpgradeableLoaderState::Program {
            programdata_address,
        } if programdata_address == expected_programdata => {}
        _ => {
            return Err("Program account references an unexpected ProgramData address".to_string());
        }
    }

    let metadata_len = UpgradeableLoaderState::size_of_programdata_metadata();
    let expected_programdata_len = UpgradeableLoaderState::size_of_programdata(max_data_len);
    if programdata_account_data.len() != expected_programdata_len
        || programdata_account_data.len() < metadata_len
    {
        return Err(format!(
            "ProgramData length is {}; expected {}",
            programdata_account_data.len(),
            expected_programdata_len
        ));
    }

    let programdata_state: UpgradeableLoaderState =
        bincode::deserialize(&programdata_account_data[..metadata_len])
            .map_err(|_| "ProgramData account has invalid loader state".to_string())?;
    let (deployed_slot, upgrade_authority) = match programdata_state {
        UpgradeableLoaderState::ProgramData {
            slot,
            upgrade_authority_address: Some(authority),
        } => (slot, authority),
        UpgradeableLoaderState::ProgramData {
            upgrade_authority_address: None,
            ..
        } => return Err("ProgramData has no upgrade authority".to_string()),
        _ => return Err("ProgramData account has an unexpected loader state".to_string()),
    };
    if &upgrade_authority != expected_upgrade_authority {
        return Err(format!(
            "ProgramData upgrade authority is {}; expected {}",
            upgrade_authority, expected_upgrade_authority
        ));
    }

    let code_end = metadata_len
        .checked_add(expected_program.len())
        .ok_or_else(|| "Program code length overflow".to_string())?;
    if code_end > programdata_account_data.len() {
        return Err("ProgramData is too short for the deployed ELF".to_string());
    }
    let deployed_program = &programdata_account_data[metadata_len..code_end];
    if deployed_program != expected_program {
        return Err("Finalized on-chain ELF does not match the uploaded program".to_string());
    }
    if programdata_account_data[code_end..]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err("ProgramData trailing bytes are not zeroed".to_string());
    }

    Ok(DeploymentReadback {
        deployed_slot,
        programdata_len: programdata_account_data.len(),
        program_sha256: sha256_hex(deployed_program),
        upgrade_authority,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn native_x86_64_elf_header() -> Vec<u8> {
        let mut elf = vec![0u8; 128];
        elf[..4].copy_from_slice(b"\x7fELF");
        elf[4] = 2; // ELFCLASS64
        elf[5] = 1; // ELFDATA2LSB
        elf[6] = 1; // EV_CURRENT
        elf[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN
        elf[18..20].copy_from_slice(&62u16.to_le_bytes()); // EM_X86_64, not SBF
        elf[20..24].copy_from_slice(&1u32.to_le_bytes()); // EV_CURRENT
        elf[40..48].copy_from_slice(&64u64.to_le_bytes()); // section headers
        elf[52..54].copy_from_slice(&64u16.to_le_bytes()); // ELF64 header size
        elf[54..56].copy_from_slice(&56u16.to_le_bytes()); // program header size
        elf[58..60].copy_from_slice(&64u16.to_le_bytes()); // section header size
        elf[60..62].copy_from_slice(&1u16.to_le_bytes()); // one null section
        elf
    }

    #[test]
    fn rejects_elf_magic_without_a_header() {
        assert!(verify_sbf_elf(b"\x7fELF").is_err());
    }

    #[test]
    fn rejects_truncated_elf_header() {
        let elf = native_x86_64_elf_header();
        assert!(verify_sbf_elf(&elf[..52]).is_err());
    }

    #[test]
    fn rejects_native_elf() {
        let error = verify_sbf_elf(&native_x86_64_elf_header()).unwrap_err();
        assert!(error.contains("runtime loader rejected"));
    }

    #[test]
    fn parses_standard_solana_keypair_json() {
        let keypair = Keypair::new();
        let json = serde_json::to_string(&keypair.to_bytes().to_vec()).unwrap();
        let parsed = parse_program_keypair_json(&json).unwrap();
        assert_eq!(parsed.pubkey(), keypair.pubkey());
        assert_eq!(
            require_program_id(&parsed, &keypair.pubkey().to_string()).unwrap(),
            keypair.pubkey()
        );
    }

    #[test]
    fn rejects_malformed_program_keypairs() {
        assert!(parse_program_keypair_json("").is_err());
        assert!(parse_program_keypair_json("[1,2,3]").is_err());
        assert!(parse_program_keypair_json(&format!("[{}]", "0,".repeat(64))).is_err());
        assert!(
            parse_program_keypair_json(&"x".repeat(MAX_PROGRAM_KEYPAIR_JSON_BYTES + 1)).is_err()
        );
    }

    #[test]
    fn rejects_program_id_mismatch() {
        let keypair = Keypair::new();
        let other = Keypair::new();
        assert!(require_program_id(&keypair, &other.pubkey().to_string()).is_err());
    }

    #[test]
    fn verifies_expected_program_hash() {
        let program = b"\x7fELFtest-program";
        let expected = sha256_hex(program);
        assert_eq!(
            require_sha256(program, &expected.to_uppercase()).unwrap(),
            expected
        );
        assert!(require_sha256(program, "not-a-hash").is_err());
        assert!(require_sha256(b"\x7fELFother", &expected).is_err());
    }

    #[test]
    fn anchor_program_id_validation_requires_embedded_target_id() {
        let expected = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let mut matching_anchor = b"\x7fELFAnchorError occurred".to_vec();
        matching_anchor.extend_from_slice(&expected.to_bytes());
        assert!(require_anchor_declared_program_id(&matching_anchor, &expected).is_ok());
        assert!(require_anchor_declared_program_id(&matching_anchor, &other).is_err());

        let non_anchor = b"\x7fELFplain-sbf-program";
        assert!(require_anchor_declared_program_id(non_anchor, &expected).is_ok());
    }

    #[test]
    fn deployment_fee_budget_includes_rate_and_recovery_reserves() {
        let budget = deployment_fee_budget(100_000, 5_000, 100).unwrap();
        assert_eq!(budget.estimated_fees_lamports, 100_000);
        assert_eq!(budget.fee_rate_reserve_lamports, 20_000);
        assert_eq!(budget.recovery_write_reserve_lamports, 40_000);
        assert_eq!(budget.total_fee_budget_lamports, 160_000);

        let large = deployment_fee_budget(1_000_001, 5_000, 1_000).unwrap();
        assert_eq!(large.fee_rate_reserve_lamports, 200_001);
        assert_eq!(large.recovery_write_reserve_lamports, 250_000);
        assert_eq!(large.total_fee_budget_lamports, 1_450_002);
    }

    #[test]
    fn deployment_fee_budget_has_no_recovery_reserve_without_pending_writes() {
        let budget = deployment_fee_budget(10_001, 5_000, 0).unwrap();
        assert_eq!(budget.fee_rate_reserve_lamports, 2_001);
        assert_eq!(budget.recovery_write_reserve_lamports, 0);
        assert_eq!(budget.total_fee_budget_lamports, 12_002);
    }

    #[test]
    fn deployment_fee_budget_rejects_overflow() {
        assert!(deployment_fee_budget(u64::MAX, 1, 1).is_err());
        assert!(deployment_fee_budget(1, u64::MAX, 1).is_err());
    }

    #[test]
    fn new_buffer_rent_budget_covers_both_peak_balance_shapes() {
        let buffer_dominates = deployment_rent_budget(100, 10, 80, None).unwrap();
        assert_eq!(buffer_dominates.final_rent_lamports, 90);
        assert_eq!(buffer_dominates.required_rent_balance_lamports, 110);

        let programdata_dominates = deployment_rent_budget(80, 10, 100, None).unwrap();
        assert_eq!(programdata_dominates.final_rent_lamports, 110);
        assert_eq!(programdata_dominates.required_rent_balance_lamports, 110);
    }

    #[test]
    fn resume_buffer_rent_budget_credits_only_available_buffer_lamports() {
        let partially_funded = deployment_rent_budget(80, 10, 100, Some(80)).unwrap();
        assert_eq!(partially_funded.final_rent_lamports, 110);
        assert_eq!(partially_funded.required_rent_balance_lamports, 30);

        let fully_funded = deployment_rent_budget(120, 10, 100, Some(120)).unwrap();
        assert_eq!(fully_funded.final_rent_lamports, 110);
        assert_eq!(fully_funded.required_rent_balance_lamports, 10);
    }

    #[test]
    fn deployment_rent_budget_rejects_overflow() {
        assert!(deployment_rent_budget(0, u64::MAX, 1, None).is_err());
        assert!(deployment_rent_budget(u64::MAX, 1, 0, None).is_err());
    }

    #[test]
    fn verifies_program_and_programdata_readback() {
        let program_id = Keypair::new().pubkey();
        let authority = Keypair::new().pubkey();
        let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID).unwrap();
        let elf = b"\x7fELFtest-program";
        let max_data_len = elf.len() + 16;
        let programdata_address = solana_loader_v3_interface::get_program_data_address(&program_id);
        let program_account_data = bincode::serialize(&UpgradeableLoaderState::Program {
            programdata_address,
        })
        .unwrap();
        let mut programdata_account_data =
            bincode::serialize(&UpgradeableLoaderState::ProgramData {
                slot: 42,
                upgrade_authority_address: Some(authority),
            })
            .unwrap();
        programdata_account_data.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
        programdata_account_data.extend_from_slice(elf);
        programdata_account_data
            .resize(UpgradeableLoaderState::size_of_programdata(max_data_len), 0);

        let readback = verify_deployment_readback(
            &program_id,
            &authority,
            elf,
            max_data_len,
            &loader_id,
            true,
            &program_account_data,
            &loader_id,
            &programdata_account_data,
        )
        .unwrap();
        assert_eq!(readback.deployed_slot, 42);
        assert_eq!(readback.program_sha256, sha256_hex(elf));
        assert_eq!(readback.upgrade_authority, authority);
    }

    #[test]
    fn plans_only_missing_resume_buffer_chunks() {
        let authority = Keypair::new().pubkey();
        let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID).unwrap();
        let program = b"\x7fELF-resumable-program-data";
        let chunk_size = 8;
        let metadata_len = UpgradeableLoaderState::size_of_buffer_metadata();
        let mut data = bincode::serialize(&UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        })
        .unwrap();
        data.resize(metadata_len + program.len(), 0);
        data[metadata_len..metadata_len + chunk_size].copy_from_slice(&program[..chunk_size]);

        let plan = verify_resume_buffer(&authority, program, chunk_size, &loader_id, false, &data)
            .unwrap();
        assert_eq!(plan.completed_chunks, 1);
        assert_eq!(plan.pending_chunk_indexes, vec![1, 2, 3]);
    }

    #[test]
    fn rejects_unsafe_resume_buffers() {
        let authority = Keypair::new().pubkey();
        let wrong_authority = Keypair::new().pubkey();
        let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID).unwrap();
        let program = b"\x7fELF-resume";
        let metadata_len = UpgradeableLoaderState::size_of_buffer_metadata();
        let mut data = bincode::serialize(&UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        })
        .unwrap();
        data.resize(metadata_len + program.len(), 0);

        assert!(
            verify_resume_buffer(&wrong_authority, program, 8, &loader_id, false, &data,).is_err()
        );
        assert!(
            verify_resume_buffer(&authority, program, 8, &Pubkey::new_unique(), false, &data,)
                .is_err()
        );
        data[metadata_len + 1] = 42;
        assert!(verify_resume_buffer(&authority, program, 8, &loader_id, false, &data,).is_err());
    }

    #[test]
    fn rejects_modified_finalized_program() {
        let program_id = Keypair::new().pubkey();
        let authority = Keypair::new().pubkey();
        let loader_id = Pubkey::from_str(UPGRADEABLE_LOADER_ID).unwrap();
        let elf = b"\x7fELFexpected";
        let programdata_address = solana_loader_v3_interface::get_program_data_address(&program_id);
        let program_account_data = bincode::serialize(&UpgradeableLoaderState::Program {
            programdata_address,
        })
        .unwrap();
        let mut programdata_account_data =
            bincode::serialize(&UpgradeableLoaderState::ProgramData {
                slot: 1,
                upgrade_authority_address: Some(authority),
            })
            .unwrap();
        programdata_account_data.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
        programdata_account_data.extend_from_slice(b"\x7fELFtampered");

        assert!(verify_deployment_readback(
            &program_id,
            &authority,
            elf,
            b"\x7fELFtampered".len(),
            &loader_id,
            true,
            &program_account_data,
            &loader_id,
            &programdata_account_data,
        )
        .is_err());
    }
}
