use fnzero_safe_app_services as svc;
use serde::{Deserialize, Serialize};
use svc::{
    biometric_policy_stub, create_wallet, empty_asset_snapshot, export_private_key,
    import_keystore, import_mnemonic, import_private_key, load_asset_snapshot, mobile_capabilities,
    preview_dapp_signing, preview_payment, preview_pump_trade, preview_squads_action, setup_totp,
    squads_approve_submit, squads_create_submit, squads_execute_submit, squads_info,
    squads_proposals, squads_reject_submit, squads_transfer_proposal_submit, submit_dapp_signing,
    submit_payment, unlock_wallet, unsupported_mobile_program_workflow, verify_totp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppNetwork {
    Mainnet,
    Devnet,
    Testnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobileErrorCode {
    InvalidInput,
    WrongPassword,
    RpcUnavailable,
    InsufficientFunds,
    UserRejected,
    BiometricCancelled,
    TotpInvalid,
    Unsupported,
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileError {
    pub code: MobileErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileCapabilitySummary {
    pub enabled: Vec<String>,
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileBridgeHealth {
    pub ok: bool,
    pub service: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletSummary {
    pub id: String,
    pub name: String,
    pub public_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportKeystoreRequest {
    pub name: String,
    pub keystore_json: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPrivateKeyRequest {
    pub name: String,
    pub private_key_base58: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMnemonicRequest {
    pub name: String,
    pub mnemonic: String,
    pub derivation_path: Option<String>,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlockWalletRequest {
    pub keystore_json: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPrivateKeyRequest {
    pub keystore_json: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPrivateKeyResponse {
    pub public_key: String,
    pub private_key_base58: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletKeystore {
    pub wallet: WalletSummary,
    pub keystore_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnlockWalletResponse {
    pub wallet: WalletSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetSummary {
    pub token_account: String,
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub amount: String,
    pub raw_amount: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetSnapshot {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub sol_balance_lamports: u64,
    pub tokens: Vec<AssetSummary>,
    pub recent_transactions: Vec<TransactionHistoryEntry>,
    pub refreshed_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetQueryRequest {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub rpc_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentPreviewRequest {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub recipient: String,
    pub mint: Option<String>,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningPreview {
    pub id: String,
    pub title: String,
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub summary: String,
    pub warnings: Vec<String>,
    pub requires_user_confirmation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentOperation {
    SolTransfer,
    SplTokenTransfer,
    WsolWrap,
    WsolUnwrap,
    WsolCloseAta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentSubmitRequest {
    pub preview_id: String,
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub wallet_public_key: String,
    pub keystore_json: String,
    pub password: String,
    pub operation: PaymentOperation,
    pub recipient: Option<String>,
    pub mint: Option<String>,
    pub amount_base_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionSubmitResult {
    pub signature: String,
    pub slot: Option<u64>,
    pub network: AppNetwork,
    pub submitted_at: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PumpPreviewRequest {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub mint: String,
    pub sell_percent_bps: u32,
    pub slippage_bps: u32,
    pub venue: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DappSignPreviewRequest {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub app_name: String,
    pub app_url: String,
    pub method: String,
    pub payload_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DappSignSubmitRequest {
    pub preview_id: String,
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub wallet_public_key: String,
    pub keystore_json: String,
    pub password: String,
    pub method: String,
    pub payload_base64: String,
    pub transaction_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DappSignSubmitResult {
    pub signature: Option<String>,
    pub signature_base64: Option<String>,
    pub signed_payload_base64: Option<String>,
    pub signed_payloads_base64: Vec<String>,
    pub transaction: Option<TransactionSubmitResult>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsPreviewRequest {
    pub network: AppNetwork,
    pub wallet_public_key: String,
    pub multisig: String,
    pub proposal: Option<String>,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsMemberSummary {
    pub key: String,
    pub permissions: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsProposalSummary {
    pub address: String,
    pub transaction_index: u64,
    pub status: String,
    pub approved: Vec<String>,
    pub rejected: Vec<String>,
    pub cancelled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsInfoRequest {
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub multisig: String,
    pub proposal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsInfoResponse {
    pub multisig: String,
    pub vault: String,
    pub create_key: String,
    pub threshold: u16,
    pub time_lock: u32,
    pub transaction_index: u64,
    pub stale_transaction_index: u64,
    pub members: Vec<SquadsMemberSummary>,
    pub proposal: Option<SquadsProposalSummary>,
    pub network: AppNetwork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsProposalsRequest {
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub multisig: String,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsProposalsResponse {
    pub multisig: String,
    pub vault: String,
    pub proposals: Vec<SquadsProposalSummary>,
    pub latest_transaction_index: u64,
    pub network: AppNetwork,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsCreateSubmitRequest {
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub keystore_json: String,
    pub password: String,
    pub members: Vec<String>,
    pub threshold: u16,
    pub time_lock: Option<u32>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsCreateSubmitResult {
    pub multisig: String,
    pub vault: String,
    pub create_key: String,
    pub signature: String,
    pub threshold: u16,
    pub members: Vec<SquadsMemberSummary>,
    pub creation_fee_lamports: u64,
    pub network: AppNetwork,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SquadsTransferKind {
    Sol,
    SplToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsTransferProposalSubmitRequest {
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub keystore_json: String,
    pub password: String,
    pub multisig: String,
    pub kind: SquadsTransferKind,
    pub recipient: Option<String>,
    pub destination_token_account: Option<String>,
    pub source_token_account: Option<String>,
    pub mint: Option<String>,
    pub amount_base_units: u64,
    pub decimals: Option<u8>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsProposalCreateSubmitResult {
    pub multisig: String,
    pub vault: String,
    pub transaction: String,
    pub proposal: String,
    pub transaction_index: u64,
    pub signature: String,
    pub network: AppNetwork,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsVoteSubmitRequest {
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub keystore_json: String,
    pub password: String,
    pub multisig: String,
    pub proposal: String,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadsExecuteSubmitRequest {
    pub approved: bool,
    pub network: AppNetwork,
    pub rpc_url: Option<String>,
    pub keystore_json: String,
    pub password: String,
    pub multisig: String,
    pub proposal: String,
    pub transaction_index: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TotpSetup {
    pub secret: String,
    pub issuer: String,
    pub account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TotpVerifyRequest {
    pub secret: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiometricPolicy {
    pub supported: bool,
    pub configured: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningDecision {
    pub preview_id: String,
    pub approved: bool,
    pub status: String,
}

impl From<AppNetwork> for svc::AppNetwork {
    fn from(value: AppNetwork) -> Self {
        match value {
            AppNetwork::Mainnet => Self::Mainnet,
            AppNetwork::Devnet => Self::Devnet,
            AppNetwork::Testnet => Self::Testnet,
        }
    }
}

impl From<svc::AppNetwork> for AppNetwork {
    fn from(value: svc::AppNetwork) -> Self {
        match value {
            svc::AppNetwork::Mainnet => Self::Mainnet,
            svc::AppNetwork::Devnet => Self::Devnet,
            svc::AppNetwork::Testnet => Self::Testnet,
        }
    }
}

impl From<svc::MobileErrorCode> for MobileErrorCode {
    fn from(value: svc::MobileErrorCode) -> Self {
        match value {
            svc::MobileErrorCode::InvalidInput => Self::InvalidInput,
            svc::MobileErrorCode::WrongPassword => Self::WrongPassword,
            svc::MobileErrorCode::RpcUnavailable => Self::RpcUnavailable,
            svc::MobileErrorCode::InsufficientFunds => Self::InsufficientFunds,
            svc::MobileErrorCode::UserRejected => Self::UserRejected,
            svc::MobileErrorCode::BiometricCancelled => Self::BiometricCancelled,
            svc::MobileErrorCode::TotpInvalid => Self::TotpInvalid,
            svc::MobileErrorCode::Unsupported => Self::Unsupported,
            svc::MobileErrorCode::NotImplemented => Self::NotImplemented,
        }
    }
}

impl From<svc::MobileError> for MobileError {
    fn from(value: svc::MobileError) -> Self {
        Self {
            code: value.code.into(),
            message: value.message,
        }
    }
}

impl From<svc::WalletSummary> for WalletSummary {
    fn from(value: svc::WalletSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            public_key: value.public_key,
        }
    }
}

impl From<WalletSummary> for svc::WalletSummary {
    fn from(value: WalletSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            public_key: value.public_key,
        }
    }
}

impl From<CreateWalletRequest> for svc::CreateWalletRequest {
    fn from(value: CreateWalletRequest) -> Self {
        Self {
            name: value.name,
            password: value.password,
        }
    }
}

impl From<ImportKeystoreRequest> for svc::ImportKeystoreRequest {
    fn from(value: ImportKeystoreRequest) -> Self {
        Self {
            name: value.name,
            keystore_json: value.keystore_json,
            password: value.password,
        }
    }
}

impl From<ImportPrivateKeyRequest> for svc::ImportPrivateKeyRequest {
    fn from(value: ImportPrivateKeyRequest) -> Self {
        Self {
            name: value.name,
            private_key_base58: value.private_key_base58,
            password: value.password,
        }
    }
}

impl From<ImportMnemonicRequest> for svc::ImportMnemonicRequest {
    fn from(value: ImportMnemonicRequest) -> Self {
        Self {
            name: value.name,
            mnemonic: value.mnemonic,
            derivation_path: value.derivation_path,
            password: value.password,
        }
    }
}

impl From<UnlockWalletRequest> for svc::UnlockWalletRequest {
    fn from(value: UnlockWalletRequest) -> Self {
        Self {
            keystore_json: value.keystore_json,
            password: value.password,
        }
    }
}

impl From<ExportPrivateKeyRequest> for svc::ExportPrivateKeyRequest {
    fn from(value: ExportPrivateKeyRequest) -> Self {
        Self {
            keystore_json: value.keystore_json,
            password: value.password,
        }
    }
}

impl From<svc::ExportPrivateKeyResponse> for ExportPrivateKeyResponse {
    fn from(value: svc::ExportPrivateKeyResponse) -> Self {
        Self {
            public_key: value.public_key,
            private_key_base58: value.private_key_base58,
        }
    }
}

impl From<svc::WalletKeystore> for WalletKeystore {
    fn from(value: svc::WalletKeystore) -> Self {
        Self {
            wallet: value.wallet.into(),
            keystore_json: value.keystore_json,
        }
    }
}

impl From<svc::UnlockWalletResponse> for UnlockWalletResponse {
    fn from(value: svc::UnlockWalletResponse) -> Self {
        Self {
            wallet: value.wallet.into(),
        }
    }
}

impl From<svc::AssetSummary> for AssetSummary {
    fn from(value: svc::AssetSummary) -> Self {
        Self {
            token_account: value.token_account,
            mint: value.mint,
            symbol: value.symbol,
            name: value.name,
            amount: value.amount,
            raw_amount: value.raw_amount,
            decimals: value.decimals,
            logo_uri: value.logo_uri,
        }
    }
}

impl From<svc::TransactionHistoryEntry> for TransactionHistoryEntry {
    fn from(value: svc::TransactionHistoryEntry) -> Self {
        Self {
            signature: value.signature,
            slot: value.slot,
            block_time: value.block_time,
            status: value.status,
        }
    }
}

impl From<svc::AssetSnapshot> for AssetSnapshot {
    fn from(value: svc::AssetSnapshot) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            sol_balance_lamports: value.sol_balance_lamports,
            tokens: value.tokens.into_iter().map(Into::into).collect(),
            recent_transactions: value
                .recent_transactions
                .into_iter()
                .map(Into::into)
                .collect(),
            refreshed_at_ms: value.refreshed_at_ms,
        }
    }
}

impl From<AssetQueryRequest> for svc::AssetQueryRequest {
    fn from(value: AssetQueryRequest) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            rpc_url: value.rpc_url,
        }
    }
}

impl From<PaymentPreviewRequest> for svc::PaymentPreviewRequest {
    fn from(value: PaymentPreviewRequest) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            recipient: value.recipient,
            mint: value.mint,
            amount: value.amount,
            memo: value.memo,
        }
    }
}

impl From<svc::SigningPreview> for SigningPreview {
    fn from(value: svc::SigningPreview) -> Self {
        Self {
            id: value.id,
            title: value.title,
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            summary: value.summary,
            warnings: value.warnings,
            requires_user_confirmation: value.requires_user_confirmation,
        }
    }
}

impl From<PaymentOperation> for svc::PaymentOperation {
    fn from(value: PaymentOperation) -> Self {
        match value {
            PaymentOperation::SolTransfer => Self::SolTransfer,
            PaymentOperation::SplTokenTransfer => Self::SplTokenTransfer,
            PaymentOperation::WsolWrap => Self::WsolWrap,
            PaymentOperation::WsolUnwrap => Self::WsolUnwrap,
            PaymentOperation::WsolCloseAta => Self::WsolCloseAta,
        }
    }
}

impl From<PaymentSubmitRequest> for svc::PaymentSubmitRequest {
    fn from(value: PaymentSubmitRequest) -> Self {
        Self {
            preview_id: value.preview_id,
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            wallet_public_key: value.wallet_public_key,
            keystore_json: value.keystore_json,
            password: value.password,
            operation: value.operation.into(),
            recipient: value.recipient,
            mint: value.mint,
            amount_base_units: value.amount_base_units,
        }
    }
}

impl From<svc::TransactionSubmitResult> for TransactionSubmitResult {
    fn from(value: svc::TransactionSubmitResult) -> Self {
        Self {
            signature: value.signature,
            slot: value.slot,
            network: value.network.into(),
            submitted_at: value.submitted_at,
            status: value.status,
        }
    }
}

impl From<PumpPreviewRequest> for svc::PumpPreviewRequest {
    fn from(value: PumpPreviewRequest) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            mint: value.mint,
            sell_percent_bps: value.sell_percent_bps,
            slippage_bps: value.slippage_bps,
            venue: value.venue,
        }
    }
}

impl From<DappSignPreviewRequest> for svc::DappSignPreviewRequest {
    fn from(value: DappSignPreviewRequest) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            app_name: value.app_name,
            app_url: value.app_url,
            method: value.method,
            payload_base64: value.payload_base64,
        }
    }
}

impl From<DappSignSubmitRequest> for svc::DappSignSubmitRequest {
    fn from(value: DappSignSubmitRequest) -> Self {
        Self {
            preview_id: value.preview_id,
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            wallet_public_key: value.wallet_public_key,
            keystore_json: value.keystore_json,
            password: value.password,
            method: value.method,
            payload_base64: value.payload_base64,
            transaction_format: value.transaction_format,
        }
    }
}

impl From<svc::DappSignSubmitResult> for DappSignSubmitResult {
    fn from(value: svc::DappSignSubmitResult) -> Self {
        Self {
            signature: value.signature,
            signature_base64: value.signature_base64,
            signed_payload_base64: value.signed_payload_base64,
            signed_payloads_base64: value.signed_payloads_base64,
            transaction: value.transaction.map(Into::into),
            status: value.status,
        }
    }
}

impl From<SquadsPreviewRequest> for svc::SquadsPreviewRequest {
    fn from(value: SquadsPreviewRequest) -> Self {
        Self {
            network: value.network.into(),
            wallet_public_key: value.wallet_public_key,
            multisig: value.multisig,
            proposal: value.proposal,
            action: value.action,
        }
    }
}

impl From<svc::SquadsMemberSummary> for SquadsMemberSummary {
    fn from(value: svc::SquadsMemberSummary) -> Self {
        Self {
            key: value.key,
            permissions: value.permissions,
        }
    }
}

impl From<svc::SquadsProposalSummary> for SquadsProposalSummary {
    fn from(value: svc::SquadsProposalSummary) -> Self {
        Self {
            address: value.address,
            transaction_index: value.transaction_index,
            status: value.status,
            approved: value.approved,
            rejected: value.rejected,
            cancelled: value.cancelled,
        }
    }
}

impl From<SquadsInfoRequest> for svc::SquadsInfoRequest {
    fn from(value: SquadsInfoRequest) -> Self {
        Self {
            network: value.network.into(),
            rpc_url: value.rpc_url,
            multisig: value.multisig,
            proposal: value.proposal,
        }
    }
}

impl From<svc::SquadsInfoResponse> for SquadsInfoResponse {
    fn from(value: svc::SquadsInfoResponse) -> Self {
        Self {
            multisig: value.multisig,
            vault: value.vault,
            create_key: value.create_key,
            threshold: value.threshold,
            time_lock: value.time_lock,
            transaction_index: value.transaction_index,
            stale_transaction_index: value.stale_transaction_index,
            members: value.members.into_iter().map(Into::into).collect(),
            proposal: value.proposal.map(Into::into),
            network: value.network.into(),
        }
    }
}

impl From<SquadsProposalsRequest> for svc::SquadsProposalsRequest {
    fn from(value: SquadsProposalsRequest) -> Self {
        Self {
            network: value.network.into(),
            rpc_url: value.rpc_url,
            multisig: value.multisig,
            limit: value.limit,
        }
    }
}

impl From<svc::SquadsProposalsResponse> for SquadsProposalsResponse {
    fn from(value: svc::SquadsProposalsResponse) -> Self {
        Self {
            multisig: value.multisig,
            vault: value.vault,
            proposals: value.proposals.into_iter().map(Into::into).collect(),
            latest_transaction_index: value.latest_transaction_index,
            network: value.network.into(),
        }
    }
}

impl From<SquadsCreateSubmitRequest> for svc::SquadsCreateSubmitRequest {
    fn from(value: SquadsCreateSubmitRequest) -> Self {
        Self {
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            keystore_json: value.keystore_json,
            password: value.password,
            members: value.members,
            threshold: value.threshold,
            time_lock: value.time_lock,
            memo: value.memo,
        }
    }
}

impl From<svc::SquadsCreateSubmitResult> for SquadsCreateSubmitResult {
    fn from(value: svc::SquadsCreateSubmitResult) -> Self {
        Self {
            multisig: value.multisig,
            vault: value.vault,
            create_key: value.create_key,
            signature: value.signature,
            threshold: value.threshold,
            members: value.members.into_iter().map(Into::into).collect(),
            creation_fee_lamports: value.creation_fee_lamports,
            network: value.network.into(),
            status: value.status,
        }
    }
}

impl From<SquadsTransferKind> for svc::SquadsTransferKind {
    fn from(value: SquadsTransferKind) -> Self {
        match value {
            SquadsTransferKind::Sol => Self::Sol,
            SquadsTransferKind::SplToken => Self::SplToken,
        }
    }
}

impl From<SquadsTransferProposalSubmitRequest> for svc::SquadsTransferProposalSubmitRequest {
    fn from(value: SquadsTransferProposalSubmitRequest) -> Self {
        Self {
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            keystore_json: value.keystore_json,
            password: value.password,
            multisig: value.multisig,
            kind: value.kind.into(),
            recipient: value.recipient,
            destination_token_account: value.destination_token_account,
            source_token_account: value.source_token_account,
            mint: value.mint,
            amount_base_units: value.amount_base_units,
            decimals: value.decimals,
            memo: value.memo,
        }
    }
}

impl From<svc::SquadsProposalCreateSubmitResult> for SquadsProposalCreateSubmitResult {
    fn from(value: svc::SquadsProposalCreateSubmitResult) -> Self {
        Self {
            multisig: value.multisig,
            vault: value.vault,
            transaction: value.transaction,
            proposal: value.proposal,
            transaction_index: value.transaction_index,
            signature: value.signature,
            network: value.network.into(),
            status: value.status,
        }
    }
}

impl From<SquadsVoteSubmitRequest> for svc::SquadsVoteSubmitRequest {
    fn from(value: SquadsVoteSubmitRequest) -> Self {
        Self {
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            keystore_json: value.keystore_json,
            password: value.password,
            multisig: value.multisig,
            proposal: value.proposal,
            memo: value.memo,
        }
    }
}

impl From<SquadsExecuteSubmitRequest> for svc::SquadsExecuteSubmitRequest {
    fn from(value: SquadsExecuteSubmitRequest) -> Self {
        Self {
            approved: value.approved,
            network: value.network.into(),
            rpc_url: value.rpc_url,
            keystore_json: value.keystore_json,
            password: value.password,
            multisig: value.multisig,
            proposal: value.proposal,
            transaction_index: value.transaction_index,
        }
    }
}

impl From<svc::TotpSetup> for TotpSetup {
    fn from(value: svc::TotpSetup) -> Self {
        Self {
            secret: value.secret,
            issuer: value.issuer,
            account: value.account,
        }
    }
}

impl From<TotpVerifyRequest> for svc::TotpVerifyRequest {
    fn from(value: TotpVerifyRequest) -> Self {
        Self {
            secret: value.secret,
            code: value.code,
        }
    }
}

impl From<svc::BiometricPolicy> for BiometricPolicy {
    fn from(value: svc::BiometricPolicy) -> Self {
        Self {
            supported: value.supported,
            configured: value.configured,
            reason: value.reason,
        }
    }
}

fn bridge_error(error: svc::AppServiceError) -> MobileError {
    error.to_mobile_error().into()
}

pub fn health() -> MobileBridgeHealth {
    MobileBridgeHealth {
        ok: true,
        service: "fnzero-safe-mobile-bridge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

pub fn get_mobile_capabilities() -> MobileCapabilitySummary {
    let capabilities = mobile_capabilities();

    MobileCapabilitySummary {
        enabled: capabilities
            .enabled
            .into_iter()
            .map(str::to_owned)
            .collect(),
        excluded: capabilities
            .excluded
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

pub fn wallet_create(req: CreateWalletRequest) -> Result<WalletKeystore, MobileError> {
    create_wallet(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_import_keystore(req: ImportKeystoreRequest) -> Result<WalletKeystore, MobileError> {
    import_keystore(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_import_private_key(
    req: ImportPrivateKeyRequest,
) -> Result<WalletKeystore, MobileError> {
    import_private_key(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_import_mnemonic(req: ImportMnemonicRequest) -> Result<WalletKeystore, MobileError> {
    import_mnemonic(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_unlock(req: UnlockWalletRequest) -> Result<UnlockWalletResponse, MobileError> {
    unlock_wallet(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_export_private_key(
    req: ExportPrivateKeyRequest,
) -> Result<ExportPrivateKeyResponse, MobileError> {
    export_private_key(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn wallet_delete_preview(wallet_public_key: String) -> SigningDecision {
    SigningDecision {
        preview_id: wallet_public_key,
        approved: false,
        status: "local_delete_requires_flutter_storage_confirmation".to_string(),
    }
}

pub fn assets_empty_snapshot(network: AppNetwork, wallet_public_key: String) -> AssetSnapshot {
    empty_asset_snapshot(network.into(), wallet_public_key).into()
}

pub fn assets_snapshot(req: AssetQueryRequest) -> Result<AssetSnapshot, MobileError> {
    load_asset_snapshot(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn payment_preview(req: PaymentPreviewRequest) -> Result<SigningPreview, MobileError> {
    preview_payment(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn payment_confirm(req: PaymentSubmitRequest) -> Result<TransactionSubmitResult, MobileError> {
    submit_payment(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn security_setup_totp(account: String) -> Result<TotpSetup, MobileError> {
    setup_totp(account).map(Into::into).map_err(bridge_error)
}

pub fn security_verify_totp(req: TotpVerifyRequest) -> Result<bool, MobileError> {
    verify_totp(req.into()).map_err(bridge_error)
}

pub fn security_biometric_policy() -> BiometricPolicy {
    biometric_policy_stub().into()
}

pub fn pump_preview(req: PumpPreviewRequest) -> Result<SigningPreview, MobileError> {
    preview_pump_trade(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn dapp_sign_preview(req: DappSignPreviewRequest) -> Result<SigningPreview, MobileError> {
    preview_dapp_signing(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn dapp_sign_confirm(req: DappSignSubmitRequest) -> Result<DappSignSubmitResult, MobileError> {
    submit_dapp_signing(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_preview(req: SquadsPreviewRequest) -> Result<SigningPreview, MobileError> {
    preview_squads_action(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_info_query(req: SquadsInfoRequest) -> Result<SquadsInfoResponse, MobileError> {
    squads_info(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_proposals_query(
    req: SquadsProposalsRequest,
) -> Result<SquadsProposalsResponse, MobileError> {
    squads_proposals(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_create_confirm(
    req: SquadsCreateSubmitRequest,
) -> Result<SquadsCreateSubmitResult, MobileError> {
    squads_create_submit(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_transfer_proposal_confirm(
    req: SquadsTransferProposalSubmitRequest,
) -> Result<SquadsProposalCreateSubmitResult, MobileError> {
    squads_transfer_proposal_submit(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_approve_confirm(
    req: SquadsVoteSubmitRequest,
) -> Result<TransactionSubmitResult, MobileError> {
    squads_approve_submit(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_reject_confirm(
    req: SquadsVoteSubmitRequest,
) -> Result<TransactionSubmitResult, MobileError> {
    squads_reject_submit(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn squads_execute_confirm(
    req: SquadsExecuteSubmitRequest,
) -> Result<TransactionSubmitResult, MobileError> {
    squads_execute_submit(req.into())
        .map(Into::into)
        .map_err(bridge_error)
}

pub fn mobile_program_deploy() -> Result<(), MobileError> {
    Err(bridge_error(unsupported_mobile_program_workflow(
        "program_deploy",
    )))
}

pub fn mobile_program_upgrade() -> Result<(), MobileError> {
    Err(bridge_error(unsupported_mobile_program_workflow(
        "program_upgrade",
    )))
}

pub fn mobile_program_invoke() -> Result<(), MobileError> {
    Err(bridge_error(unsupported_mobile_program_workflow(
        "program_invoke",
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_reports_mobile_scope() {
        let capabilities = get_mobile_capabilities();

        assert!(capabilities
            .enabled
            .contains(&"squads_multisig".to_string()));
        assert!(capabilities
            .excluded
            .contains(&"program_deploy".to_string()));
    }

    #[test]
    fn program_deploy_is_not_available_on_mobile() {
        let error = mobile_program_deploy().unwrap_err();

        assert_eq!(error.code, MobileErrorCode::Unsupported);
        assert!(error.message.contains("program_deploy"));
    }
}
