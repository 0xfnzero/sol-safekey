use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use aws_lc_rs::{
    encoding::{AsDer, PublicKeyX509Der},
    rsa::{KeySize, OaepPrivateDecryptingKey, PrivateDecryptingKey, OAEP_SHA256_MGF1SHA256},
};
use axum::extract::DefaultBodyLimit;
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Path, Request},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use futures::{
    future::join_all,
    stream::{self, StreamExt},
    FutureExt,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use sol_safekey::solana_utils::{lamports_to_sol, SolanaClient};
use sol_safekey::KeyManager;
use sol_trade_sdk::{
    common::{
        fast_fn::get_associated_token_address_with_program_id_fast_use_seed, GasFeeStrategy,
        SolanaRpcClient, TradeConfig,
    },
    constants::WSOL_TOKEN_ACCOUNT,
    instruction::utils::pumpswap,
    swqos::{SwqosConfig, SwqosRegion, SwqosType},
    trading::{
        core::params::{DexParamEnum, PumpFunParams, PumpSwapParams},
        factory::DexType,
    },
    SolanaTrade, TradeSellParams, TradeTokenType, TradingInfrastructure,
};
use solana_account_decoder_client_types::{
    token::{TokenAccountType, UiExtension},
    UiAccount, UiAccountData, UiAccountEncoding,
};
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_commitment_config::CommitmentConfig;
use solana_loader_v3_interface::{
    get_program_data_address, instruction as loader_v3_instruction, state::UpgradeableLoaderState,
};
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcSimulateTransactionConfig, RpcTransactionConfig},
    request::{Address as RpcAddress, TokenAccountsFilter},
    response::RpcKeyedAccount,
};
use solana_sdk::account::Account;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sanitize::Sanitize;
use solana_sdk::signature::Signer;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::transaction::Transaction;
use solana_transaction_status_client_types::{
    option_serializer::OptionSerializer, EncodedTransaction, TransactionStatus, UiInstruction,
    UiMessage, UiParsedInstruction, UiTransactionEncoding,
};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::panic::AssertUnwindSafe;
use std::path::{Path as FsPath, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tokio::time::timeout;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zeroize::{Zeroize, Zeroizing};

mod program_deploy;
mod squads_v4;
mod wallet_store;
use squads_v4::{
    Member as SquadsMember, Multisig as SquadsMultisig, Permissions as SquadsPermissions,
    ProgramConfig as SquadsProgramConfig, Proposal as SquadsProposal,
    VaultTransaction as SquadsVaultTransaction,
};
use wallet_store::TokenMetadataRecord;
use wallet_store::WalletAssetsRecord;
use wallet_store::WalletSummary;
use wallet_store::WalletTokenAssetRecord;

const API_TOKEN_HEADER: &str = "x-sol-safekey-token";
const MAX_JSON_BODY_BYTES: usize = 6 * 1024 * 1024;
const MAX_SECURE_ENVELOPE_BYTES: usize = MAX_JSON_BODY_BYTES * 2;
const MAX_KEYSTORE_JSON_BYTES: usize = 128 * 1024;
const MAX_PROGRAM_SO_BYTES: usize = 3 * 1024 * 1024;
const MAX_PROGRAM_SO_BASE64_BYTES: usize = 4 * 1024 * 1024;
const PROGRAM_WRITE_CHUNK_BYTES: usize = 800;
const UPGRADEABLE_LOADER_ID: Pubkey =
    Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");
const SBF_VERIFY_BUSY_MESSAGE: &str = "SBF 验证容量已满，请等待当前验证结束后重试";
const PROGRAM_DEPLOY_BUSY_MESSAGE: &str =
    "已有 Program 部署正在进行；为避免重复部署，请等待其完成或进入恢复流程";
const PROGRAM_SOURCE_BUILD_TIMEOUT_SECS: u64 = 30 * 60;
const PROGRAM_SOURCE_BUILD_LOG_BYTES: usize = 24 * 1024;
const PROGRAM_DEPLOY_RPC_TIMEOUT_SECS: u64 = 20;
const MAX_LABEL_CHARS: usize = 80;
const MAX_TEXT_FIELD_CHARS: usize = 512;
const MAX_TOKEN_DECIMALS: u8 = 18;
const DEFAULT_SLIPPAGE_BPS: u64 = 100;
const MAX_SLIPPAGE_BPS: u64 = 10_000;
const DEFAULT_SELL_PERCENT_BPS: u64 = 10_000;
const PUMPFUN_PARAM_RPC_ATTEMPTS: usize = 2;
const PUMPFUN_PARAM_RPC_TIMEOUT_MS: u64 = 1_500;
const PUMPFUN_PARAM_RETRY_DELAY_MS: u64 = 75;
const PUMP_SELL_SUBMIT_TIMEOUT_SECS: u64 = 8;
const MAX_NONCE_BATCH_COUNT: u8 = 20;
const MAX_WALLET_TRANSACTION_HISTORY: usize = 100;
const PUMPFUN_UVA_DISCRIMINATOR: [u8; 8] = [86, 255, 112, 14, 102, 53, 154, 250];
const PUMPFUN_PROGRAM_ID: &str = "6EF8rrecthR5DkP5hnbZQGmVfRGhPUgAaoeS8QJmR5j";
const PUMPSWAP_PROGRAM_ID: &str = "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA";
const WRAPPED_SOL_MINT: &str = "So11111111111111111111111111111111111111112";
const SOLANA_TOKEN_LOGO_URI: &str = "/token-icons/solana.png";
const LOCAL_TOKEN_METADATA: &[LocalTokenMetadata] = &[
    LocalTokenMetadata {
        mint: WRAPPED_SOL_MINT,
        name: "Wrapped SOL",
        symbol: "WSOL",
        logo_uri: SOLANA_TOKEN_LOGO_URI,
    },
    LocalTokenMetadata {
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        name: "USD Coin",
        symbol: "USDC",
        logo_uri: "/token-icons/usdc.png",
    },
    LocalTokenMetadata {
        mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
        name: "USDT",
        symbol: "USDT",
        logo_uri: "/token-icons/usdt.png",
    },
    LocalTokenMetadata {
        mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
        name: "Jupiter",
        symbol: "JUP",
        logo_uri: "/token-icons/jup.png",
    },
    LocalTokenMetadata {
        mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
        name: "Jito Staked SOL",
        symbol: "JitoSOL",
        logo_uri: "/token-icons/jitosol.png",
    },
    LocalTokenMetadata {
        mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        name: "Marinade staked SOL",
        symbol: "mSOL",
        logo_uri: "/token-icons/msol.png",
    },
    LocalTokenMetadata {
        mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
        name: "Bonk",
        symbol: "BONK",
        logo_uri: "/token-icons/bonk.jpg",
    },
    LocalTokenMetadata {
        mint: "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
        name: "Pyth Network",
        symbol: "PYTH",
        logo_uri: "/token-icons/pyth.png",
    },
    LocalTokenMetadata {
        mint: "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
        name: "dogwifhat",
        symbol: "WIF",
        logo_uri: "/token-icons/wif.jpg",
    },
    LocalTokenMetadata {
        mint: "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
        name: "Jito",
        symbol: "JTO",
        logo_uri: "/token-icons/jto.webp",
    },
];
const METAPLEX_TOKEN_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const SPL_TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const MAX_TOKEN_METADATA_URI_BYTES: usize = 256 * 1024;
const TOKEN_METADATA_BATCH_SIZE: usize = 100;
const TOKEN_METADATA_JSON_CONCURRENCY: usize = 4;
const TOKEN_METADATA_JSON_TIMEOUT_MS: u64 = 1_500;
const TOKEN_METADATA_CACHE_TTL_SECS: u64 = 24 * 60 * 60;
const RPC_QUERY_TIMEOUT_SECS: u64 = 8;
const RPC_TRANSACTION_DETAIL_TIMEOUT_SECS: u64 = 2;
const ALLOW_SECRET_EXPORT_ENV: &str = "SOL_SAFEKEY_ALLOW_SECRET_EXPORT";
const ALLOW_DIRECT_SECRET_INPUT_ENV: &str = "SOL_SAFEKEY_ALLOW_DIRECT_SECRET_INPUT";
const ALLOWED_ORIGINS_ENV: &str = "SOL_SAFEKEY_ALLOWED_ORIGINS";
const MAX_SECURITY_QUESTION_INDEX: usize = 7;
const SECURE_BODY_HEADER: &str = "x-sol-safekey-secure-body";
const SECURE_BODY_VERSION: &str = "1";
const SECURE_MAX_ENCRYPTED_KEY_BYTES: usize = 1024;
const SECURE_OPTIONAL_API_PATHS: &[&str] = &["/api/health", "/api/secure/session"];
const FLASHBLOCK_SWQOS_API_TOKEN_ENV: &str = "SOL_SAFEKEY_FLASHBLOCK_SWQOS_API_TOKEN";
const BLOCKRAZOR_SWQOS_API_TOKEN_ENV: &str = "SOL_SAFEKEY_BLOCKRAZOR_SWQOS_API_TOKEN";
const ASTRALANE_SWQOS_API_TOKEN_ENV: &str = "SOL_SAFEKEY_ASTRALANE_SWQOS_API_TOKEN";
const SPEEDLANDING_SWQOS_API_TOKEN_ENV: &str = "SOL_SAFEKEY_SPEEDLANDING_SWQOS_API_TOKEN";
const DEFAULT_SWQOS_TIP_SOL: f64 = 0.0001;

static SECURE_BODY_KEYPAIR: OnceLock<SecureBodyKeyPair> = OnceLock::new();
static PROGRAM_DEPLOY_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
static PROGRAM_SOURCE_BUILD_JOBS: OnceLock<
    Arc<tokio::sync::Mutex<HashMap<String, ProgramSourceBuildJob>>>,
> = OnceLock::new();
static SBF_VERIFY_LIMIT: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
static KEYSTORE_TASK_LIMIT: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();

fn official_swqos_configs() -> Vec<SwqosConfig> {
    let region = SwqosRegion::Frankfurt;
    let configured_token = |name: &str| {
        std::env::var(name)
            .ok()
            .map(|token| token.trim().to_string())
            .filter(|token| !token.is_empty())
    };
    let mut configs = Vec::new();
    if let Some(token) = configured_token(FLASHBLOCK_SWQOS_API_TOKEN_ENV) {
        configs.push(SwqosConfig::FlashBlock(token, region.clone(), None));
    }
    if let Some(token) = configured_token(BLOCKRAZOR_SWQOS_API_TOKEN_ENV) {
        configs.push(SwqosConfig::BlockRazor(token, region.clone(), None, None));
    }
    if let Some(token) = configured_token(ASTRALANE_SWQOS_API_TOKEN_ENV) {
        configs.push(SwqosConfig::Astralane(token, region.clone(), None, None));
    }
    if let Some(token) = configured_token(SPEEDLANDING_SWQOS_API_TOKEN_ENV) {
        configs.push(SwqosConfig::Speedlanding(token, region, None));
    }
    configs
}

fn default_swqos_configs(rpc_url: &str) -> Vec<SwqosConfig> {
    let mut configs = official_swqos_configs();
    configs.push(SwqosConfig::Default(rpc_url.to_string()));
    configs
}

fn swqos_only_sell_client(
    client: &SolanaTrade,
    payer: Arc<Keypair>,
    use_seed: bool,
) -> Result<SolanaTrade, ApiError> {
    let swqos_clients = client
        .infrastructure
        .swqos_clients
        .iter()
        .filter(|swqos| !matches!(swqos.get_swqos_type(), SwqosType::Default))
        .cloned()
        .collect::<Vec<_>>();
    if swqos_clients.is_empty() {
        return Err(ApiError {
            message: "未初始化可用 SWQOS 通道，已按配置禁止 RPC 卖出".to_string(),
        });
    }

    let labels = swqos_clients
        .iter()
        .map(|swqos| swqos.get_swqos_type().as_str())
        .collect::<Vec<_>>();
    tracing::info!("Pump sell SWQOS-only channels: {}", labels.join(", "));

    let infrastructure = TradingInfrastructure {
        rpc: client.infrastructure.rpc.clone(),
        swqos_clients: Arc::new(swqos_clients),
        config: client.infrastructure.config.clone(),
        max_sender_concurrency: client.infrastructure.max_sender_concurrency,
        effective_core_ids: client.infrastructure.effective_core_ids.clone(),
    };
    let mut sell_client =
        SolanaTrade::from_infrastructure(payer, Arc::new(infrastructure), use_seed);
    sell_client.log_enabled = false;
    sell_client.check_min_tip = false;
    Ok(sell_client)
}

struct SecureBodyKeyPair {
    private_key: OaepPrivateDecryptingKey,
    public_key_pem: String,
}

fn secure_body_keypair() -> &'static SecureBodyKeyPair {
    SECURE_BODY_KEYPAIR.get_or_init(|| {
        let private_key = PrivateDecryptingKey::generate(KeySize::Rsa2048)
            .expect("failed to generate request encryption key");
        let public_key = private_key.public_key();
        let public_key_der = AsDer::<PublicKeyX509Der<'static>>::as_der(&public_key)
            .expect("failed to encode request encryption public key");
        let public_key_pem = pem_rfc7468::encode_string(
            "PUBLIC KEY",
            pem_rfc7468::LineEnding::LF,
            public_key_der.as_ref(),
        )
        .expect("failed to encode request encryption public key PEM");
        let private_key = OaepPrivateDecryptingKey::new(private_key)
            .expect("failed to initialize request decryption key");

        SecureBodyKeyPair {
            private_key,
            public_key_pem,
        }
    })
}

fn secure_body_required(method: &Method, path: &str) -> bool {
    matches!(method, &Method::POST | &Method::PUT | &Method::PATCH)
        && path.starts_with("/api/")
        && !SECURE_OPTIONAL_API_PATHS
            .iter()
            .any(|optional| path == *optional || path.strip_suffix('/') == Some(*optional))
}

#[derive(Serialize)]
struct SecureSessionResponse {
    version: &'static str,
    algorithm: &'static str,
    public_key_pem: String,
}

#[derive(Deserialize)]
struct SecureBodyEnvelope {
    version: u8,
    encrypted_key: String,
    iv: String,
    ciphertext: String,
}

struct SensitiveBodyBytes(Vec<u8>);

impl SensitiveBodyBytes {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<[u8]> for SensitiveBodyBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SensitiveBodyBytes {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

// Helper function to safely create keypair from base58 string
fn keypair_from_base58_safe(private_key: &str) -> Result<Keypair, String> {
    let mut bytes = bs58::decode(private_key)
        .into_vec()
        .map_err(|e| format!("无效的私钥格式: {}", e))?;
    let result = Keypair::try_from(bytes.as_slice()).map_err(|e| format!("无效的私钥格式: {}", e));
    bytes.zeroize();
    result
}

#[derive(Deserialize, Default)]
struct WalletAuthRequest {
    #[serde(default)]
    wallet_id: Option<String>,
    #[serde(default)]
    private_key: Option<String>,
    #[serde(default)]
    secret_key: Option<String>,
    #[serde(default)]
    keystore_json: Option<String>,
    #[serde(default)]
    encrypted_key: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

impl WalletAuthRequest {
    fn keypair(&self) -> Result<Keypair, ApiError> {
        if let Some(id) = self
            .wallet_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            validate_wallet_id(id)?;
            let password = self.required_password("使用已保存钱包时需要提供密码")?;
            let wallet = wallet_store::find(id).map_err(|message| ApiError { message })?;
            return KeyManager::keypair_from_encrypted_json_v2(&wallet.keystore_json, password)
                .map_err(|e| ApiError {
                    message: format!(
                        "钱包解锁失败: {}。签名操作只接受 authenticated v2 keystore",
                        e
                    ),
                });
        }

        if let Some(encrypted_key) = self
            .encrypted_key
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            require_direct_secret_input_enabled()?;
            let password = self.required_password("使用加密私钥时需要提供密码")?;
            let secret =
                KeyManager::decrypt_with_password(encrypted_key, password).map_err(|e| {
                    ApiError {
                        message: format!("解密失败: {}", e),
                    }
                })?;
            return keypair_from_base58_safe(&secret).map_err(|message| ApiError { message });
        }

        if let Some(keystore_json) = self
            .keystore_json
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let password = self.required_password("使用 keystore 时需要提供密码")?;
            return KeyManager::keypair_from_encrypted_json_v2(keystore_json, password).map_err(
                |e| ApiError {
                    message: format!(
                        "keystore 解密失败: {}。签名操作只接受 authenticated v2 keystore",
                        e
                    ),
                },
            );
        }

        let private_key = self
            .private_key
            .as_deref()
            .or(self.secret_key.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError {
                message: "需要提供私钥、加密私钥、keystore 或已保存钱包".to_string(),
            })?;
        require_direct_secret_input_enabled()?;
        keypair_from_base58_safe(private_key).map_err(|message| ApiError { message })
    }

    fn public_key(&self) -> Result<String, ApiError> {
        if let Some(id) = self
            .wallet_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            validate_wallet_id(id)?;
            let password = self.required_password("使用已保存钱包时需要提供密码")?;
            let wallet = wallet_store::find(id).map_err(|message| ApiError { message })?;
            KeyManager::keypair_from_encrypted_json(&wallet.keystore_json, password).map_err(
                |e| ApiError {
                    message: format!("钱包解锁失败: {}", e),
                },
            )?;
            return Ok(wallet.public_key);
        }

        Ok(self.keypair()?.pubkey().to_string())
    }

    fn required_password(&self, message: &str) -> Result<&str, ApiError> {
        self.password
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ApiError {
                message: message.to_string(),
            })
    }

    fn clear_secrets(&mut self) {
        for value in [
            &mut self.private_key,
            &mut self.secret_key,
            &mut self.keystore_json,
            &mut self.encrypted_key,
            &mut self.password,
        ] {
            if let Some(secret) = value.as_mut() {
                secret.zeroize();
            }
            *value = None;
        }
    }
}

impl Drop for WalletAuthRequest {
    fn drop(&mut self) {
        self.clear_secrets();
    }
}

#[derive(Clone, Debug)]
struct DecimalAmount(String);

impl DecimalAmount {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for DecimalAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(value) => Ok(Self(value)),
            Value::Number(value) => Ok(Self(value.to_string())),
            _ => Err(serde::de::Error::custom("amount must be a decimal value")),
        }
    }
}

fn parse_decimal_to_units(amount: &str, decimals: u8, field: &str) -> Result<u64, ApiError> {
    let amount = amount.trim();
    if amount.is_empty() || amount.starts_with(['+', '-']) {
        return Err(ApiError {
            message: format!("{} 必须是大于 0 的有效十进制数字", field),
        });
    }

    let mut parts = amount.split('.');
    let whole_part = parts.next().unwrap_or_default();
    let fractional_part = parts.next();
    if parts.next().is_some() || (whole_part.is_empty() && fractional_part.unwrap_or("").is_empty())
    {
        return Err(ApiError {
            message: format!("{} 必须是大于 0 的有效十进制数字", field),
        });
    }
    if !whole_part.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(ApiError {
            message: format!("{} 必须是大于 0 的有效十进制数字", field),
        });
    }

    let scale = 10_u64
        .checked_pow(decimals as u32)
        .ok_or_else(|| ApiError {
            message: format!("{}精度超出有效范围", field),
        })?;
    let mut whole_units = 0_u64;
    for ch in whole_part.bytes() {
        whole_units = whole_units
            .checked_mul(10)
            .and_then(|value| value.checked_add((ch - b'0') as u64))
            .ok_or_else(|| ApiError {
                message: format!("{}超出有效范围", field),
            })?;
    }
    let mut units = whole_units.checked_mul(scale).ok_or_else(|| ApiError {
        message: format!("{}超出有效范围", field),
    })?;

    if let Some(fractional_part) = fractional_part {
        if !fractional_part.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(ApiError {
                message: format!("{} 必须是大于 0 的有效十进制数字", field),
            });
        }
        let decimals = decimals as usize;
        let used_fractional = if fractional_part.len() > decimals {
            let (used, extra) = fractional_part.split_at(decimals);
            if extra.bytes().any(|ch| ch != b'0') {
                return Err(ApiError {
                    message: format!("{}最多支持 {} 位小数", field, decimals),
                });
            }
            used
        } else {
            fractional_part
        };

        let mut fractional_units = 0_u64;
        for ch in used_fractional.bytes() {
            fractional_units = fractional_units
                .checked_mul(10)
                .and_then(|value| value.checked_add((ch - b'0') as u64))
                .ok_or_else(|| ApiError {
                    message: format!("{}超出有效范围", field),
                })?;
        }
        for _ in used_fractional.len()..decimals {
            fractional_units = fractional_units.checked_mul(10).ok_or_else(|| ApiError {
                message: format!("{}超出有效范围", field),
            })?;
        }
        units = units
            .checked_add(fractional_units)
            .ok_or_else(|| ApiError {
                message: format!("{}超出有效范围", field),
            })?;
    }

    if units == 0 {
        return Err(ApiError {
            message: format!("{}必须至少为最小单位", field),
        });
    }

    Ok(units)
}

fn sol_to_lamports(amount: &DecimalAmount) -> Result<u64, ApiError> {
    parse_decimal_to_units(amount.as_str(), 9, "金额")
}

fn token_amount_to_raw(amount: &DecimalAmount, decimals: u8) -> Result<u64, ApiError> {
    if decimals > MAX_TOKEN_DECIMALS {
        return Err(ApiError {
            message: format!("代币精度不能超过 {}", MAX_TOKEN_DECIMALS),
        });
    }
    parse_decimal_to_units(amount.as_str(), decimals, "代币数量")
}

fn require_nonempty(value: &str, field: &str) -> Result<(), ApiError> {
    if value.is_empty() {
        Err(ApiError {
            message: format!("{}不能为空", field),
        })
    } else {
        Ok(())
    }
}

fn validate_text_len(value: &str, field: &str, max_chars: usize) -> Result<(), ApiError> {
    if value.chars().count() <= max_chars {
        Ok(())
    } else {
        Err(ApiError {
            message: format!("{}长度不能超过 {} 个字符", field, max_chars),
        })
    }
}

fn validate_optional_label(value: Option<String>, field: &str) -> Result<Option<String>, ApiError> {
    match value {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                validate_text_len(trimmed, field, MAX_LABEL_CHARS)?;
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn keystore_metadata_value(keystore_json: &str, key: &str) -> Option<String> {
    let data: Value = serde_json::from_str(keystore_json).ok()?;
    data.get("metadata")
        .and_then(|metadata| metadata.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

fn keystore_metadata_name(keystore_json: &str) -> Option<String> {
    keystore_metadata_value(keystore_json, "name")
}

fn with_keystore_metadata(keystore_json: &str, name: Option<&str>) -> Result<String, ApiError> {
    let mut data: Value = serde_json::from_str(keystore_json).map_err(|_| ApiError {
        message: "Invalid JSON format".to_string(),
    })?;
    let Some(object) = data.as_object_mut() else {
        return Err(ApiError {
            message: "Invalid JSON format".to_string(),
        });
    };

    let mut metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    metadata.insert(
        "app".to_string(),
        Value::String("sol-safekey-ui".to_string()),
    );
    metadata.insert("version".to_string(), Value::Number(1.into()));

    if let Some(name) = name.map(str::trim).filter(|value| !value.is_empty()) {
        metadata.insert("name".to_string(), Value::String(name.to_string()));
    } else {
        metadata.remove("name");
    }

    metadata.remove("tag");

    object.insert("metadata".to_string(), Value::Object(metadata));
    serde_json::to_string(&data).map_err(|e| ApiError {
        message: format!("序列化 Keystore 失败: {}", e),
    })
}

fn validate_wallet_id(wallet_id: &str) -> Result<(), ApiError> {
    if wallet_id.len() == 32 && wallet_id.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ApiError {
            message: "无效的钱包 ID".to_string(),
        })
    }
}

fn normalize_slippage_bps(slippage: Option<u64>) -> Result<u64, ApiError> {
    let slippage = slippage.unwrap_or(DEFAULT_SLIPPAGE_BPS);
    if slippage <= MAX_SLIPPAGE_BPS {
        Ok(slippage)
    } else {
        Err(ApiError {
            message: "滑点不能超过 100%".to_string(),
        })
    }
}

fn normalize_sell_percent_bps(sell_percent: Option<u64>) -> Result<Option<u64>, ApiError> {
    match sell_percent {
        Some(value) if (1..=10_000).contains(&value) => Ok(Some(value)),
        Some(_) => Err(ApiError {
            message: "卖出比例必须大于 0 且不能超过 100%".to_string(),
        }),
        None => Ok(None),
    }
}

fn token_amount_from_sell_percent(
    token_balance: u64,
    sell_percent_bps: u64,
) -> Result<u64, ApiError> {
    let raw_amount = if sell_percent_bps == DEFAULT_SELL_PERCENT_BPS {
        token_balance
    } else {
        ((token_balance as u128)
            .checked_mul(sell_percent_bps as u128)
            .ok_or_else(|| ApiError {
                message: "卖出数量超出有效范围".to_string(),
            })?
            / DEFAULT_SELL_PERCENT_BPS as u128) as u64
    };
    if raw_amount == 0 {
        Err(ApiError {
            message: "按当前比例计算后的卖出数量小于最小单位".to_string(),
        })
    } else {
        Ok(raw_amount)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PumpSellTokenAccountKind {
    StandardAta,
    SeedAccount,
}

impl PumpSellTokenAccountKind {
    fn label(self) -> &'static str {
        match self {
            Self::StandardAta => "standard_ata",
            Self::SeedAccount => "seed_account",
        }
    }

    fn use_seed(self) -> bool {
        matches!(self, Self::SeedAccount)
    }
}

#[derive(Clone, Debug)]
struct PumpSellTokenAccountBalance {
    kind: PumpSellTokenAccountKind,
    account: Pubkey,
    token_program: Pubkey,
    raw_amount: u64,
    decimals: u8,
}

fn mint_token_program_for_sell(rpc_client: &RpcClient, mint: &Pubkey) -> Result<Pubkey, ApiError> {
    let token_2022_program = Pubkey::from_str(SPL_TOKEN_2022_PROGRAM_ID).map_err(|e| ApiError {
        message: format!("Token-2022 程序地址解析失败: {}", e),
    })?;
    if mint.to_string().ends_with("pump") {
        return Ok(token_2022_program);
    }

    let account = rpc_client.get_account(mint).map_err(|e| ApiError {
        message: format!("查询 Token Mint 账户失败: {}", e),
    })?;
    let spl_token_program = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).map_err(|e| ApiError {
        message: format!("Token 程序地址解析失败: {}", e),
    })?;
    if account.owner == spl_token_program || account.owner == token_2022_program {
        Ok(account.owner)
    } else {
        Err(ApiError {
            message: format!(
                "Mint {} 的 owner 不是 SPL Token 或 Token-2022 程序: {}",
                mint, account.owner
            ),
        })
    }
}

fn is_missing_token_account_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("accountnotfound")
        || lower.contains("could not find account")
        || lower.contains("account not found")
        || lower.contains("invalid param: could not find")
}

fn token_account_balance_for_sell(
    rpc_client: &RpcClient,
    account: &Pubkey,
) -> Result<Option<(u64, u8)>, ApiError> {
    match rpc_client.get_token_account_balance(account) {
        Ok(balance) => {
            let raw_amount = balance.amount.parse::<u64>().map_err(|_| ApiError {
                message: format!("解析卖出账户余额失败: {}", account),
            })?;
            Ok(Some((raw_amount, balance.decimals)))
        }
        Err(e) => {
            let message = e.to_string();
            if is_missing_token_account_error(&message) {
                Ok(None)
            } else {
                Err(ApiError {
                    message: format!("查询卖出账户余额失败 ({}): {}", account, message),
                })
            }
        }
    }
}

fn candidate_sell_token_account(
    rpc_client: &RpcClient,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
    kind: PumpSellTokenAccountKind,
) -> Result<Option<PumpSellTokenAccountBalance>, ApiError> {
    let account = get_associated_token_address_with_program_id_fast_use_seed(
        owner,
        mint,
        token_program,
        kind.use_seed(),
    );
    let Some((raw_amount, decimals)) = token_account_balance_for_sell(rpc_client, &account)? else {
        return Ok(None);
    };
    Ok(Some(PumpSellTokenAccountBalance {
        kind,
        account,
        token_program: *token_program,
        raw_amount,
        decimals,
    }))
}

fn sell_balance_overflow_error() -> ApiError {
    ApiError {
        message: "代币余额超出有效范围".to_string(),
    }
}

fn select_pump_sell_token_source(
    rpc_client: &RpcClient,
    mint: &Pubkey,
    owner: &Pubkey,
    human_amount: Option<&DecimalAmount>,
    sell_percent_bps: Option<u64>,
) -> Result<(PumpSellTokenAccountBalance, u64), ApiError> {
    let token_program = mint_token_program_for_sell(rpc_client, mint)?;
    let standard = candidate_sell_token_account(
        rpc_client,
        owner,
        mint,
        &token_program,
        PumpSellTokenAccountKind::StandardAta,
    )?;
    let seed = candidate_sell_token_account(
        rpc_client,
        owner,
        mint,
        &token_program,
        PumpSellTokenAccountKind::SeedAccount,
    )?;
    let mut candidates = [standard, seed]
        .into_iter()
        .flatten()
        .filter(|candidate| candidate.raw_amount > 0)
        .collect::<Vec<_>>();

    candidates.sort_by_key(|candidate| match candidate.kind {
        PumpSellTokenAccountKind::StandardAta => 0_u8,
        PumpSellTokenAccountKind::SeedAccount => 1_u8,
    });

    if candidates.is_empty() {
        let standard_account = get_associated_token_address_with_program_id_fast_use_seed(
            owner,
            mint,
            &token_program,
            false,
        );
        let seed_account = get_associated_token_address_with_program_id_fast_use_seed(
            owner,
            mint,
            &token_program,
            true,
        );
        return Err(ApiError {
            message: format!(
                "未找到可卖出的标准 ATA 或 seed token account 余额。standard ATA: {}; seed account: {}; token program: {}",
                standard_account, seed_account, token_program
            ),
        });
    }

    let total_balance = candidates.iter().try_fold(0_u64, |total, candidate| {
        total
            .checked_add(candidate.raw_amount)
            .ok_or_else(sell_balance_overflow_error)
    })?;

    if let Some(human_amount) = human_amount {
        let decimals = candidates[0].decimals;
        let raw_amount = token_amount_to_raw(human_amount, decimals)?;
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| {
                candidate.kind == PumpSellTokenAccountKind::StandardAta
                    && candidate.raw_amount >= raw_amount
            })
            .cloned()
        {
            return Ok((candidate, raw_amount));
        }
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| {
                candidate.kind == PumpSellTokenAccountKind::SeedAccount
                    && candidate.raw_amount >= raw_amount
            })
            .cloned()
        {
            return Ok((candidate, raw_amount));
        }
        if raw_amount <= total_balance {
            return Err(ApiError {
                message: "卖出数量分散在标准 ATA 和 seed token account 中，当前单笔卖出不能跨账户扣款；请按单个账户余额分次卖出或先合并账户".to_string(),
            });
        }
        return Err(ApiError {
            message: format!(
                "卖出数量超过余额，可用余额: {}",
                total_balance as f64 / 10_f64.powi(decimals as i32)
            ),
        });
    }

    let sell_percent_bps = sell_percent_bps.unwrap_or(DEFAULT_SELL_PERCENT_BPS);
    let selected = if candidates.len() == 1 {
        candidates.remove(0)
    } else {
        let selected = candidates
            .iter()
            .max_by_key(|candidate| candidate.raw_amount)
            .cloned()
            .expect("non-empty candidates");
        tracing::warn!(
            "Sell balance for mint {} is split across standard/seed accounts; selected {} {} balance={} for percent sell",
            mint,
            selected.kind.label(),
            selected.account,
            selected.raw_amount
        );
        selected
    };
    let raw_amount = token_amount_from_sell_percent(selected.raw_amount, sell_percent_bps)?;
    Ok((selected, raw_amount))
}

fn pda_from_seed(program_id: &str, seed: &[u8], owner: &Pubkey) -> Result<Pubkey, ApiError> {
    let program = Pubkey::from_str(program_id).map_err(|e| ApiError {
        message: format!("解析程序地址失败: {}", e),
    })?;
    Pubkey::try_find_program_address(&[seed, owner.as_ref()], &program)
        .map(|(pubkey, _)| pubkey)
        .ok_or_else(|| ApiError {
            message: "推导返现账户失败".to_string(),
        })
}

fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
    data.get(offset..offset + 8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
}

fn cashback_amount_from_uva_data(data: &[u8], stable: bool) -> Option<u64> {
    if data.len() < 8 || data[0..8] != PUMPFUN_UVA_DISCRIMINATOR {
        return None;
    }
    let earned_offset = if stable { 73 } else { 57 };
    let claimed_offset = if stable { 81 } else { 65 };
    let earned = read_u64_le(data, earned_offset)?;
    let claimed = read_u64_le(data, claimed_offset)?;
    Some(earned.saturating_sub(claimed))
}

fn cashback_info(
    rpc_client: &RpcClient,
    owner: &Pubkey,
    dex_type: DexType,
) -> Result<(Pubkey, u64, String), ApiError> {
    let (program_id, stable, asset) = match dex_type {
        DexType::PumpFun => (PUMPFUN_PROGRAM_ID, false, "SOL"),
        DexType::PumpSwap => (PUMPSWAP_PROGRAM_ID, false, "WSOL"),
        _ => {
            return Err(ApiError {
                message: "不支持的返现类型".to_string(),
            });
        }
    };
    let uva = pda_from_seed(program_id, b"user_volume_accumulator", owner)?;
    let amount = match rpc_client.get_account(&uva) {
        Ok(account) => cashback_amount_from_uva_data(&account.data, stable).unwrap_or(0),
        Err(_) => 0,
    };
    Ok((uva, amount, asset.to_string()))
}

async fn execute_cashback_claim(
    keypair: Keypair,
    rpc_url: &str,
    dex_type: DexType,
) -> Result<String, ApiError> {
    let payer = Arc::new(keypair);
    let commitment = CommitmentConfig::confirmed();
    let swqos_configs = default_swqos_configs(rpc_url);
    let trade_config = TradeConfig::builder(rpc_url.to_string(), swqos_configs, commitment)
        .create_wsol_ata_on_startup(false)
        .use_seed_optimize(false)
        .check_min_tip(false)
        .log_enabled(false)
        .swqos_cores_from_end(false)
        .mev_protection(false)
        .build();
    let client = SolanaTrade::new(payer, trade_config).await;
    match dex_type {
        DexType::PumpFun => client.claim_cashback_pumpfun().await,
        DexType::PumpSwap => client.claim_cashback_pumpswap().await,
        _ => Err(anyhow::anyhow!("unsupported cashback type")),
    }
    .map_err(|e| ApiError {
        message: format!("领取返现失败: {}", e),
    })
}

fn require_secret_export_enabled() -> Result<(), ApiError> {
    if std::env::var(ALLOW_SECRET_EXPORT_ENV).ok().as_deref() == Some("true") {
        Ok(())
    } else {
        Err(ApiError {
            message: format!(
                "Web 明文私钥导出已禁用；如确需本机调试，请显式设置 {}=true",
                ALLOW_SECRET_EXPORT_ENV
            ),
        })
    }
}

fn require_direct_secret_input_enabled() -> Result<(), ApiError> {
    if std::env::var(ALLOW_DIRECT_SECRET_INPUT_ENV).ok().as_deref() == Some("true") {
        Ok(())
    } else {
        Err(ApiError {
            message: format!(
                "Web 直接提交明文私钥或加密私钥已禁用；请导入 keystore 后选择钱包，或显式设置 {}=true 进行本机调试",
                ALLOW_DIRECT_SECRET_INPUT_ENV
            ),
        })
    }
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0_u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "unknown panic".to_string()
}

async fn catch_panics(request: Request, next: Next) -> Response {
    match AssertUnwindSafe(next.run(request)).catch_unwind().await {
        Ok(response) => response,
        Err(panic_payload) => {
            let message = panic_message(panic_payload.as_ref());
            tracing::error!("HTTP handler panic: {}", message);
            let body = Json(json!({ "error": format!("服务执行异常: {}", message) }));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}

fn normalized_origin(value: &str) -> Option<String> {
    let Ok(uri) = value.parse::<Uri>() else {
        return None;
    };
    let scheme = uri.scheme_str()?;
    let host = uri.host()?;
    let default_port = match scheme {
        "http" => Some(80),
        "https" => Some(443),
        "tauri" => None,
        _ => None,
    };
    let port = uri.port_u16();
    let port_part = match (port, default_port) {
        (Some(port), Some(default_port)) if port == default_port => String::new(),
        (Some(port), _) => format!(":{}", port),
        _ => String::new(),
    };
    Some(format!("{}://{}{}", scheme, host, port_part))
}

fn configured_allowed_origins() -> Vec<String> {
    std::env::var(ALLOWED_ORIGINS_ENV)
        .ok()
        .map(|value| {
            value
                .split(',')
                .filter_map(|item| normalized_origin(item.trim()))
                .collect()
        })
        .unwrap_or_default()
}

fn is_allowed_local_origin(value: &str) -> bool {
    let Some(origin) = normalized_origin(value) else {
        return false;
    };
    let host = origin
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split(':').next())
        .unwrap_or("");
    if host == "127.0.0.1" || host == "localhost" || host == "tauri.localhost" {
        return true;
    }
    if origin == "tauri://localhost" {
        return true;
    }
    configured_allowed_origins()
        .iter()
        .any(|allowed| constant_time_eq(allowed, &origin))
}

async fn require_local_origin(headers: HeaderMap, request: Request, next: Next) -> Response {
    if request.uri().path().starts_with("/api/")
        && !matches!(
            request.method(),
            &Method::GET | &Method::HEAD | &Method::OPTIONS
        )
    {
        let allowed = headers
            .get("origin")
            .and_then(|value| value.to_str().ok())
            .map(is_allowed_local_origin)
            .unwrap_or_else(|| {
                headers
                    .get("referer")
                    .and_then(|value| value.to_str().ok())
                    .map(is_allowed_local_origin)
                    .unwrap_or(true)
            });
        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": format!(
                        "拒绝非本机页面发起的本地 API 请求；如需允许自己的 HTTPS 页面访问本机 API，请设置 {}=https://你的域名",
                        ALLOWED_ORIGINS_ENV
                    )
                })),
            )
                .into_response();
        }
    }

    next.run(request).await
}

async fn require_api_token(headers: HeaderMap, request: Request, next: Next) -> Response {
    let expected = match std::env::var("SOL_SAFEKEY_API_TOKEN") {
        Ok(token) if !token.trim().is_empty() => token,
        _ => return next.run(request).await,
    };

    let provided = headers
        .get(API_TOKEN_HEADER)
        .and_then(|value| value.to_str().ok());
    if !provided
        .map(|provided| constant_time_eq(provided, expected.as_str()))
        .unwrap_or(false)
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "缺少或无效的本地 API token" })),
        )
            .into_response();
    }

    next.run(request).await
}

async fn decrypt_secure_body(headers: HeaderMap, request: Request, next: Next) -> Response {
    let secure_required = secure_body_required(request.method(), request.uri().path());
    let is_secure = headers
        .get(SECURE_BODY_HEADER)
        .and_then(|value| value.to_str().ok())
        == Some(SECURE_BODY_VERSION);

    if !is_secure {
        if secure_required {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "变更类 API 请求必须使用加密请求体" })),
            )
                .into_response();
        }
        return next.run(request).await;
    }

    if !matches!(
        request.method(),
        &Method::POST | &Method::PUT | &Method::PATCH
    ) {
        return next.run(request).await;
    }

    match decrypt_secure_request(request).await {
        Ok(request) => next.run(request).await,
        Err(err) => err.into_response(),
    }
}

async fn decrypt_secure_request(request: Request) -> Result<Request, ApiError> {
    let (mut parts, body) = request.into_parts();
    let encrypted_body = to_bytes(body, MAX_SECURE_ENVELOPE_BYTES)
        .await
        .map_err(|e| ApiError {
            message: format!("读取加密请求失败: {}", e),
        })?;
    let envelope: SecureBodyEnvelope =
        serde_json::from_slice(&encrypted_body).map_err(|_| ApiError {
            message: "无效的加密请求格式".to_string(),
        })?;

    if envelope.version != 1 {
        return Err(ApiError {
            message: "不支持的加密请求版本".to_string(),
        });
    }

    let encrypted_key = BASE64
        .decode(envelope.encrypted_key.as_bytes())
        .map_err(|_| ApiError {
            message: "无效的加密请求密钥".to_string(),
        })?;
    if encrypted_key.len() > SECURE_MAX_ENCRYPTED_KEY_BYTES {
        return Err(ApiError {
            message: "加密请求密钥过大".to_string(),
        });
    }

    let iv = BASE64
        .decode(envelope.iv.as_bytes())
        .map_err(|_| ApiError {
            message: "无效的加密请求 IV".to_string(),
        })?;
    if iv.len() != 12 {
        return Err(ApiError {
            message: "无效的加密请求 IV 长度".to_string(),
        });
    }

    let ciphertext = BASE64
        .decode(envelope.ciphertext.as_bytes())
        .map_err(|_| ApiError {
            message: "无效的加密请求正文".to_string(),
        })?;

    let secure_keypair = secure_body_keypair();
    let mut decrypted_key =
        Zeroizing::new(vec![0_u8; secure_keypair.private_key.min_output_size()]);
    let aes_key_len = secure_keypair
        .private_key
        .decrypt(
            &OAEP_SHA256_MGF1SHA256,
            &encrypted_key,
            decrypted_key.as_mut_slice(),
            None,
        )
        .map_err(|_| ApiError {
            message: "解密请求密钥失败".to_string(),
        })?
        .len();
    if aes_key_len != 32 {
        return Err(ApiError {
            message: "无效的请求密钥长度".to_string(),
        });
    }

    let cipher =
        Aes256Gcm::new_from_slice(&decrypted_key[..aes_key_len]).map_err(|_| ApiError {
            message: "初始化请求解密失败".to_string(),
        })?;
    let plaintext = SensitiveBodyBytes(
        cipher
            .decrypt(Nonce::from_slice(&iv), ciphertext.as_ref())
            .map_err(|_| ApiError {
                message: "解密请求正文失败".to_string(),
            })?,
    );
    if plaintext.len() > MAX_JSON_BODY_BYTES {
        return Err(ApiError {
            message: "请求正文过大".to_string(),
        });
    }

    parts.headers.remove(SECURE_BODY_HEADER);
    parts
        .headers
        .insert("content-type", HeaderValue::from_static("application/json"));
    parts.headers.insert(
        "content-length",
        HeaderValue::from_str(&plaintext.len().to_string()).map_err(|e| ApiError {
            message: format!("更新请求长度失败: {}", e),
        })?,
    );

    Ok(Request::from_parts(
        parts,
        Body::from(Bytes::from_owner(plaintext)),
    ))
}

async fn add_security_headers(request: Request, next: Next) -> Response {
    let is_api = request.uri().path().starts_with("/api/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static("frame-ancestors 'none'; base-uri 'none'; form-action 'self'"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    if is_api {
        headers.insert(
            "cache-control",
            HeaderValue::from_static("no-store, max-age=0"),
        );
    }
    response
}

fn validate_keystore_size(keystore_json: &str) -> Result<(), ApiError> {
    if keystore_json.len() <= MAX_KEYSTORE_JSON_BYTES {
        Ok(())
    } else {
        Err(ApiError {
            message: "Keystore 文件过大".to_string(),
        })
    }
}

fn validate_program_binary(program: &[u8]) -> Result<(), ApiError> {
    if program.is_empty() {
        return Err(ApiError {
            message: "Program .so 文件不能为空".to_string(),
        });
    }
    if program.len() > MAX_PROGRAM_SO_BYTES {
        return Err(ApiError {
            message: format!(
                "Program .so 文件过大，最大支持 {} MB",
                MAX_PROGRAM_SO_BYTES / 1024 / 1024
            ),
        });
    }
    if program.len() < 4 || &program[0..4] != b"\x7fELF" {
        return Err(ApiError {
            message: "无效的 Program .so 文件".to_string(),
        });
    }
    Ok(())
}

fn decode_program_binary_base64(encoded: &str) -> Result<Vec<u8>, ApiError> {
    let encoded = encoded.trim();
    if encoded.len() > MAX_PROGRAM_SO_BASE64_BYTES {
        return Err(ApiError {
            message: format!(
                "Program .so base64 过大，编码后不能超过 {} bytes",
                MAX_PROGRAM_SO_BASE64_BYTES
            ),
        });
    }
    let program = BASE64.decode(encoded.as_bytes()).map_err(|_| ApiError {
        message: "无效的 Program .so base64".to_string(),
    })?;
    validate_program_binary(&program)?;
    Ok(program)
}

async fn verify_program_binary_offline(program: Vec<u8>) -> Result<Vec<u8>, ApiError> {
    let permit = SBF_VERIFY_LIMIT
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError {
            message: SBF_VERIFY_BUSY_MESSAGE.to_string(),
        })?;
    tokio::task::spawn_blocking(move || {
        let result = program_deploy::verify_sbf_elf(&program);
        drop(permit);
        result.map(|()| program)
    })
    .await
    .map_err(|error| ApiError {
        message: format!("SBF 验证任务异常终止: {error}"),
    })?
    .map_err(|message| ApiError { message })
}

async fn run_keystore_task<T, F>(task: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ApiError> + Send + 'static,
{
    let permit = KEYSTORE_TASK_LIMIT
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2)))
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError {
            message: "Keystore 密钥派生并发控制已关闭".to_string(),
        })?;
    tokio::task::spawn_blocking(move || {
        let result = task();
        drop(permit);
        result
    })
    .await
    .map_err(|error| ApiError {
        message: format!("Keystore 密钥派生任务异常终止: {error}"),
    })?
}

fn sign_and_send(
    client: &RpcClient,
    instructions: Vec<solana_sdk::instruction::Instruction>,
    signers: &[&Keypair],
    payer: &Pubkey,
) -> Result<String, ApiError> {
    let blockhash = client.get_latest_blockhash().map_err(|e| ApiError {
        message: format!("获取 blockhash 失败: {}", e),
    })?;
    let tx = Transaction::new_signed_with_payer(&instructions, Some(payer), signers, blockhash);
    client
        .send_and_confirm_transaction(&tx)
        .map(|signature| signature.to_string())
        .map_err(|e| ApiError {
            message: format!("提交交易失败: {}", e),
        })
}

async fn wait_for_signature_commitment(
    client: &RpcClient,
    signature: &Signature,
    timeout: Duration,
    action: &str,
    commitment: CommitmentConfig,
    commitment_label: &str,
) -> Result<u64, ApiError> {
    let deadline = Instant::now() + timeout;
    let mut last_rpc_error = None;

    loop {
        match client.get_signature_statuses_with_history(&[*signature]) {
            Ok(response) => {
                if let Some(status) = response.value.into_iter().next().flatten() {
                    if status.satisfies_commitment(commitment) {
                        if let Some(error) = status.err.as_ref() {
                            return Err(ApiError {
                                message: format!(
                                    "{}交易 {} 已在目标确认级别执行失败，不能盲目重试: {}",
                                    action, signature, error
                                ),
                            });
                        }
                        return Ok(status.slot);
                    }
                }
            }
            Err(error) => last_rpc_error = Some(error.to_string()),
        }

        if Instant::now() >= deadline {
            let detail = last_rpc_error
                .map(|error| format!("；最后一次 RPC 错误: {error}"))
                .unwrap_or_default();
            return Err(ApiError {
                message: format!(
                    "{}交易 {} 已提交，但未能在超时前确认 {}{}。请先按该签名核对链上状态，不能盲目重试",
                    action, signature, commitment_label, detail
                ),
            });
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn submit_signed_transaction_once(
    client: &RpcClient,
    transaction: &Transaction,
    action: &str,
    context: &str,
    commitment: CommitmentConfig,
    commitment_label: &str,
    timeout: Duration,
) -> Result<(Signature, u64), ApiError> {
    let local_signature = *transaction.signatures.first().ok_or_else(|| ApiError {
        message: format!("{action}交易缺少本地签名；{context}"),
    })?;
    if local_signature == Signature::default() {
        return Err(ApiError {
            message: format!("{action}交易尚未签名；{context}"),
        });
    }

    let send_error = match client.send_transaction(transaction) {
        Ok(rpc_signature) => {
            if rpc_signature != local_signature {
                return Err(ApiError {
                    message: format!(
                        "{action} RPC 返回签名 {rpc_signature}，但本地确定签名为 {local_signature}；{context}"
                    ),
                });
            }
            None
        }
        Err(error) => Some(error.to_string()),
    };

    let wait_timeout = if send_error.is_some() {
        timeout.min(Duration::from_secs(10))
    } else {
        timeout
    };
    match wait_for_signature_commitment(
        client,
        &local_signature,
        wait_timeout,
        action,
        commitment,
        commitment_label,
    )
    .await
    {
        Ok(slot) => Ok((local_signature, slot)),
        Err(wait_error) => {
            let send_detail = send_error
                .map(|error| format!("；首次提交 RPC 错误: {error}"))
                .unwrap_or_default();
            Err(ApiError {
                message: format!(
                    "{}{}；{}；本地确定签名: {}。不得自动重签或盲目重试",
                    wait_error.message, send_detail, context, local_signature
                ),
            })
        }
    }
}

fn estimate_instruction_fee(
    client: &RpcClient,
    instructions: &[solana_sdk::instruction::Instruction],
    payer: &Pubkey,
    recent_blockhash: &solana_sdk::hash::Hash,
    stage: &str,
) -> Result<u64, ApiError> {
    let message = Message::new_with_blockhash(instructions, Some(payer), recent_blockhash);
    client
        .get_fee_for_message(&message)
        .map_err(|error| ApiError {
            message: format!("估算 {stage} 交易费失败: {error}"),
        })
}

fn simulate_program_deployment_transaction(
    client: &RpcClient,
    transaction: &Transaction,
    stage: &str,
) -> Result<(), ApiError> {
    transaction.sanitize().map_err(|error| ApiError {
        message: format!("{stage} 交易结构校验失败: {error:?}"),
    })?;
    let response = client
        .simulate_transaction_with_config(
            transaction,
            RpcSimulateTransactionConfig {
                sig_verify: true,
                replace_recent_blockhash: false,
                commitment: Some(CommitmentConfig::confirmed()),
                encoding: Some(UiTransactionEncoding::Base64),
                accounts: None,
                min_context_slot: None,
                inner_instructions: false,
            },
        )
        .map_err(|error| ApiError {
            message: format!("{stage} 交易模拟 RPC 失败: {error}"),
        })?;
    if let Some(error) = response.value.err {
        return Err(ApiError {
            message: format!("{stage} 交易模拟失败: {error}"),
        });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn wait_for_finalized_deployment_readback(
    client: &RpcClient,
    program_id: &Pubkey,
    programdata_address: &Pubkey,
    expected_upgrade_authority: &Pubkey,
    expected_program: &[u8],
    max_data_len: usize,
    minimum_slot: u64,
) -> Result<program_deploy::DeploymentReadback, ApiError> {
    let deadline = Instant::now() + Duration::from_secs(20);

    loop {
        let pending_reason = match client.get_multiple_accounts_with_commitment(
            &[*program_id, *programdata_address],
            CommitmentConfig::finalized(),
        ) {
            Ok(response) if response.context.slot >= minimum_slot => {
                let mut accounts = response.value.into_iter();
                match (accounts.next().flatten(), accounts.next().flatten()) {
                    (Some(program), Some(programdata)) => {
                        return program_deploy::verify_deployment_readback(
                            program_id,
                            expected_upgrade_authority,
                            expected_program,
                            max_data_len,
                            &program.owner,
                            program.executable,
                            &program.data,
                            &programdata.owner,
                            &programdata.data,
                        )
                        .map_err(|message| ApiError { message });
                    }
                    _ => "finalized Program 或 ProgramData 尚不可见".to_string(),
                }
            }
            Ok(response) => format!(
                "finalized 账户读取 slot {} 早于部署 slot {}",
                response.context.slot, minimum_slot
            ),
            Err(error) => format!("finalized 回读 RPC 失败: {error}"),
        };

        if Instant::now() >= deadline {
            return Err(ApiError {
                message: format!("部署已 finalized，但链上回读校验超时: {pending_reason}"),
            });
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

fn network_name(network: Option<&str>) -> String {
    rpc_selector(network)
        .map(|selector| selector.network)
        .unwrap_or_else(|_| "mainnet".to_string())
}

fn rpc_client_for(network: Option<&str>) -> Result<(RpcClient, String), ApiError> {
    let selector = rpc_selector(network)?;
    let rpc_url = selector.url;
    let network = selector.network;
    Ok((
        RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed()),
        network,
    ))
}

fn rpc_query_client_for_timeout(
    network: Option<&str>,
    timeout_secs: u64,
) -> Result<(RpcClient, String), ApiError> {
    let selector = rpc_selector(network)?;
    let rpc_url = selector.url.clone();
    let network = selector.network.clone();
    Ok((
        RpcClient::new_with_timeout_and_commitment(
            rpc_url,
            Duration::from_secs(timeout_secs),
            CommitmentConfig::confirmed(),
        ),
        network,
    ))
}

fn rpc_query_client_for_url(url: String, timeout_secs: u64) -> RpcClient {
    RpcClient::new_with_timeout_and_commitment(
        url,
        Duration::from_secs(timeout_secs),
        CommitmentConfig::confirmed(),
    )
}

fn is_retryable_rpc_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("error sending request")
        || message.contains("timed out")
        || message.contains("timeout")
        || message.contains("超时")
        || message.contains("connection rate limits exceeded")
        || message.contains("too many requests")
        || message.contains("429")
        || message.contains("econnreset")
        || message.contains("connection reset")
        || message.contains("connection refused")
        || message.contains("temporarily unavailable")
        || message.contains("service unavailable")
        || message.contains("502")
        || message.contains("503")
        || message.contains("504")
}

fn is_retryable_api_error(error: &ApiError) -> bool {
    is_retryable_rpc_message(&error.message)
}

fn dedupe_rpc_urls(urls: Vec<String>) -> Vec<String> {
    let mut seen = HashMap::new();
    let mut deduped = Vec::new();
    for url in urls {
        let url = url.trim().trim_end_matches('/').to_string();
        if url.is_empty() || seen.contains_key(&url) {
            continue;
        }
        seen.insert(url.clone(), true);
        deduped.push(url);
    }
    deduped
}

fn fallback_rpc_urls(selector: &RpcSelector) -> Vec<String> {
    let mut urls = vec![selector.url.clone()];
    match selector.network.as_str() {
        "devnet" => urls.push(DEVNET_RPC_URL.to_string()),
        "testnet" => {
            urls.push(PUBLICNODE_TESTNET_RPC_URL.to_string());
            urls.push(TESTNET_RPC_URL.to_string());
        }
        _ => {
            urls.push(PUBLICNODE_MAINNET_RPC_URL.to_string());
            urls.push(DEFAULT_RPC_URL.to_string());
        }
    }
    dedupe_rpc_urls(urls)
}

fn rpc_failure_summary(errors: &[(String, String)]) -> String {
    if errors.is_empty() {
        return "没有可用的 RPC 节点".to_string();
    }
    errors
        .iter()
        .map(|(url, error)| format!("{}: {}", url, error))
        .collect::<Vec<_>>()
        .join("; ")
}

fn token_mint_info_from_rpc(
    client: &RpcClient,
    mint: &Pubkey,
) -> Result<solana_account_decoder_client_types::token::UiTokenAmount, ApiError> {
    let supply = client.get_token_supply(mint).map_err(|e| ApiError {
        message: format!("查询 Token Mint 信息失败: {}", e),
    })?;
    if supply.decimals > MAX_TOKEN_DECIMALS {
        return Err(ApiError {
            message: format!("Token 精度不能超过 {}", MAX_TOKEN_DECIMALS),
        });
    }
    Ok(supply)
}

fn parse_token_account(
    keyed_account: RpcKeyedAccount,
) -> Option<(
    String,
    solana_account_decoder_client_types::token::UiTokenAccount,
)> {
    let UiAccountData::Json(parsed_account) = keyed_account.account.data else {
        return None;
    };
    let Ok(TokenAccountType::Account(token_account)) =
        serde_json::from_value::<TokenAccountType>(parsed_account.parsed)
    else {
        return None;
    };
    Some((keyed_account.pubkey, token_account))
}

fn token_accounts_by_program(
    client: &RpcClient,
    owner: &Pubkey,
    program_id: Pubkey,
) -> Result<
    Vec<(
        String,
        solana_account_decoder_client_types::token::UiTokenAccount,
    )>,
    ApiError,
> {
    let filter = TokenAccountsFilter::ProgramId(
        RpcAddress::from_str(&program_id.to_string()).map_err(|e| ApiError {
            message: format!("Token 程序地址解析失败: {}", e),
        })?,
    );
    let accounts = client
        .get_token_accounts_by_owner(owner, filter)
        .map_err(|e| ApiError {
            message: format!("查询 Token 账户失败: {}", e),
        })?;
    Ok(accounts
        .into_iter()
        .filter_map(parse_token_account)
        .collect())
}

fn all_owner_token_accounts(
    client: &RpcClient,
    owner: &Pubkey,
) -> Result<
    Vec<(
        String,
        solana_account_decoder_client_types::token::UiTokenAccount,
    )>,
    ApiError,
> {
    let token_program = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).map_err(|e| ApiError {
        message: format!("Token 程序地址解析失败: {}", e),
    })?;
    let token_2022_program = Pubkey::from_str(SPL_TOKEN_2022_PROGRAM_ID).map_err(|e| ApiError {
        message: format!("Token-2022 程序地址解析失败: {}", e),
    })?;
    let token_accounts = token_accounts_by_program(client, owner, token_program);
    let token_2022_accounts = token_accounts_by_program(client, owner, token_2022_program);

    match (token_accounts, token_2022_accounts) {
        (Ok(mut accounts), Ok(mut token_2022_accounts)) => {
            accounts.append(&mut token_2022_accounts);
            Ok(accounts)
        }
        (Ok(_), Err(err)) if is_retryable_api_error(&err) => Err(ApiError {
            message: format!("查询 Token 账户失败: Token-2022: {}", err.message),
        }),
        (Err(err), Ok(_)) if is_retryable_api_error(&err) => Err(ApiError {
            message: format!("查询 Token 账户失败: SPL Token: {}", err.message),
        }),
        (Ok(accounts), Err(err)) => {
            tracing::warn!(
                "Token-2022 account query failed; returning SPL Token accounts only: {}",
                err.message
            );
            Ok(accounts)
        }
        (Err(err), Ok(accounts)) => {
            tracing::warn!(
                "SPL Token account query failed; returning Token-2022 accounts only: {}",
                err.message
            );
            Ok(accounts)
        }
        (Err(token_err), Err(token_2022_err)) => Err(ApiError {
            message: format!(
                "查询 Token 账户失败: SPL Token: {}; Token-2022: {}",
                token_err.message, token_2022_err.message
            ),
        }),
    }
}

#[derive(Clone, Default)]
struct TokenMetadata {
    name: Option<String>,
    symbol: Option<String>,
    logo_uri: Option<String>,
    metadata_uri: Option<String>,
}

#[derive(Clone, Copy)]
struct LocalTokenMetadata {
    mint: &'static str,
    name: &'static str,
    symbol: &'static str,
    logo_uri: &'static str,
}

impl LocalTokenMetadata {
    fn into_token_metadata(self) -> TokenMetadata {
        TokenMetadata {
            name: Some(self.name.to_string()),
            symbol: Some(self.symbol.to_string()),
            logo_uri: Some(self.logo_uri.to_string()),
            metadata_uri: None,
        }
    }
}

impl TokenMetadata {
    fn merge_missing(&mut self, other: TokenMetadata) {
        if self.name.is_none() {
            self.name = other.name;
        }
        if self.symbol.is_none() {
            self.symbol = other.symbol;
        }
        if self.logo_uri.is_none() {
            self.logo_uri = other.logo_uri;
        }
        if self.metadata_uri.is_none() {
            self.metadata_uri = other.metadata_uri;
        }
    }
}

fn now_unix_secs_lossy() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn normalize_token_metadata(mint: &str, mut metadata: TokenMetadata) -> TokenMetadata {
    if let Some(local_metadata) = local_token_metadata(mint) {
        metadata.name = Some(local_metadata.name.to_string());
        metadata.symbol = Some(local_metadata.symbol.to_string());
        metadata.logo_uri = Some(local_metadata.logo_uri.to_string());
    }
    metadata
}

fn local_token_metadata(mint: &str) -> Option<LocalTokenMetadata> {
    LOCAL_TOKEN_METADATA
        .iter()
        .copied()
        .find(|metadata| metadata.mint == mint)
}

fn clean_metadata_string(value: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(value)
        .trim_matches(char::from(0))
        .trim()
        .to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn clean_json_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn has_token_identity(metadata: &TokenMetadata) -> bool {
    metadata.name.is_some() && metadata.symbol.is_some()
}

fn read_metadata_string(data: &[u8], offset: &mut usize) -> Option<String> {
    let len_bytes = data.get(*offset..(*offset + 4))?;
    let len = u32::from_le_bytes(len_bytes.try_into().ok()?) as usize;
    *offset += 4;
    let value = data.get(*offset..(*offset + len))?;
    *offset += len;
    clean_metadata_string(value)
}

fn normalize_metadata_uri(value: &str) -> Option<String> {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") {
        Some(value.to_string())
    } else {
        value
            .strip_prefix("ipfs://")
            .map(|path| format!("https://ipfs.io/ipfs/{}", path.trim_start_matches("ipfs/")))
    }
}

fn logo_uri_from_value(value: &Value) -> Option<String> {
    ["image", "image_uri", "logo_uri"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .and_then(normalize_metadata_uri)
}

async fn fetch_token_json_metadata(uri: &str) -> TokenMetadata {
    let Some(uri) = normalize_metadata_uri(uri) else {
        return TokenMetadata::default();
    };
    let Some(response) = reqwest::Client::new()
        .get(uri.as_str())
        .timeout(std::time::Duration::from_millis(
            TOKEN_METADATA_JSON_TIMEOUT_MS,
        ))
        .send()
        .await
        .ok()
    else {
        return TokenMetadata::default();
    };
    if !response.status().is_success() {
        return TokenMetadata::default();
    }
    let Some(bytes) = response.bytes().await.ok() else {
        return TokenMetadata::default();
    };
    if bytes.len() > MAX_TOKEN_METADATA_URI_BYTES {
        return TokenMetadata::default();
    }
    let Some(value) = serde_json::from_slice::<Value>(&bytes).ok() else {
        return TokenMetadata::default();
    };

    TokenMetadata {
        name: clean_json_string(value.get("name")),
        symbol: clean_json_string(value.get("symbol")),
        logo_uri: logo_uri_from_value(&value),
        metadata_uri: None,
    }
}

async fn enrich_token_metadata_uri(metadata: &mut TokenMetadata, uri: Option<&str>) {
    if let Some(uri) = uri {
        metadata.merge_missing(fetch_token_json_metadata(uri.trim()).await);
    }
}

async fn token_2022_metadata_from_account(
    account: solana_account_decoder_client_types::UiAccount,
) -> TokenMetadata {
    let UiAccountData::Json(parsed_account) = account.data else {
        return TokenMetadata::default();
    };
    let Ok(TokenAccountType::Mint(mint)) =
        serde_json::from_value::<TokenAccountType>(parsed_account.parsed)
    else {
        return TokenMetadata::default();
    };

    for extension in mint.extensions {
        if let UiExtension::TokenMetadata(token_metadata) = extension {
            let mut metadata = TokenMetadata {
                name: clean_metadata_string(token_metadata.name.as_bytes()),
                symbol: clean_metadata_string(token_metadata.symbol.as_bytes()),
                logo_uri: None,
                metadata_uri: normalize_metadata_uri(token_metadata.uri.as_str()),
            };
            if !has_token_identity(&metadata) && metadata.metadata_uri.is_none() {
                enrich_token_metadata_uri(&mut metadata, Some(token_metadata.uri.as_str())).await;
            }
            return metadata;
        }
    }

    TokenMetadata::default()
}

fn should_try_metaplex_metadata(
    account: Option<&solana_account_decoder_client_types::UiAccount>,
) -> bool {
    match account {
        Some(account) => account.owner == SPL_TOKEN_PROGRAM_ID,
        None => true,
    }
}

async fn fetch_metaplex_token_metadata(client: &RpcClient, mint_pubkey: &Pubkey) -> TokenMetadata {
    let Ok(program_id) = Pubkey::from_str(METAPLEX_TOKEN_METADATA_PROGRAM_ID) else {
        return TokenMetadata::default();
    };
    let Some((metadata_pubkey, _)) = Pubkey::try_find_program_address(
        &[b"metadata", program_id.as_ref(), mint_pubkey.as_ref()],
        &program_id,
    ) else {
        return TokenMetadata::default();
    };
    let Ok(account) = client.get_account(&metadata_pubkey) else {
        return TokenMetadata::default();
    };

    // Metaplex Metadata V1 layout: key(1), update_authority(32), mint(32), then Data(name, symbol, uri).
    let mut offset = 1 + 32 + 32;
    let name = read_metadata_string(&account.data, &mut offset);
    let symbol = read_metadata_string(&account.data, &mut offset);
    let metadata_uri = read_metadata_string(&account.data, &mut offset);

    let mut metadata = TokenMetadata {
        name,
        symbol,
        logo_uri: None,
        metadata_uri: metadata_uri.as_deref().and_then(normalize_metadata_uri),
    };
    if !has_token_identity(&metadata) && metadata.metadata_uri.is_none() {
        enrich_token_metadata_uri(&mut metadata, metadata_uri.as_deref()).await;
    }
    metadata
}

async fn enrich_token_json_metadata_map(metadata_by_mint: &mut HashMap<String, TokenMetadata>) {
    let pending = metadata_by_mint
        .iter()
        .filter_map(|(mint, metadata)| {
            if metadata.logo_uri.is_none() || !has_token_identity(metadata) {
                metadata
                    .metadata_uri
                    .as_ref()
                    .map(|uri| (mint.clone(), uri.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let logos = stream::iter(pending)
        .map(|(mint, uri)| async move {
            let metadata = fetch_token_json_metadata(&uri).await;
            (mint, metadata)
        })
        .buffer_unordered(TOKEN_METADATA_JSON_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    for (mint, json_metadata) in logos {
        if let Some(metadata) = metadata_by_mint.get_mut(&mint) {
            metadata.merge_missing(json_metadata);
        }
    }
}

async fn load_token_metadata_map(
    client: &RpcClient,
    mints: &[String],
) -> HashMap<String, TokenMetadata> {
    let mut metadata_by_mint: HashMap<String, TokenMetadata> = HashMap::new();
    let mut parsed_mints = Vec::new();
    for mint in mints {
        if let Ok(pubkey) = Pubkey::from_str(mint) {
            parsed_mints.push((mint.clone(), pubkey));
        }
    }

    for chunk in parsed_mints.chunks(TOKEN_METADATA_BATCH_SIZE) {
        let pubkeys: Vec<Pubkey> = chunk.iter().map(|(_, pubkey)| *pubkey).collect();
        let accounts = client
            .get_multiple_ui_accounts_with_config(
                &pubkeys,
                RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..RpcAccountInfoConfig::default()
                },
            )
            .map(|response| response.value)
            .unwrap_or_default();

        for ((mint, pubkey), account) in chunk.iter().zip(accounts.into_iter()) {
            let metadata = match account.clone() {
                Some(account) => token_2022_metadata_from_account(account).await,
                None => TokenMetadata::default(),
            };
            if metadata.name.is_some() || metadata.symbol.is_some() || metadata.logo_uri.is_some() {
                metadata_by_mint.insert(mint.clone(), metadata);
            } else if should_try_metaplex_metadata(account.as_ref()) {
                metadata_by_mint.insert(
                    mint.clone(),
                    fetch_metaplex_token_metadata(client, pubkey).await,
                );
            } else {
                metadata_by_mint.insert(mint.clone(), metadata);
            }
        }
    }

    enrich_token_json_metadata_map(&mut metadata_by_mint).await;
    metadata_by_mint
}

fn token_metadata_from_cache_record(record: &TokenMetadataRecord) -> TokenMetadata {
    normalize_token_metadata(
        &record.mint,
        TokenMetadata {
            name: clean_cached_metadata_string(record.name.clone()),
            symbol: clean_cached_metadata_string(record.symbol.clone()),
            logo_uri: clean_cached_metadata_string(record.logo_uri.clone()),
            metadata_uri: None,
        },
    )
}

fn token_metadata_is_complete(metadata: &TokenMetadata) -> bool {
    has_token_identity(metadata) && metadata.logo_uri.is_some()
}

fn token_metadata_records(
    network: &str,
    metadata_by_mint: &HashMap<String, TokenMetadata>,
) -> Vec<TokenMetadataRecord> {
    metadata_by_mint
        .iter()
        .filter_map(|(mint, metadata)| {
            let metadata = normalize_token_metadata(mint, metadata.clone());
            if metadata.name.is_none() && metadata.symbol.is_none() && metadata.logo_uri.is_none() {
                return None;
            }
            Some(TokenMetadataRecord {
                network: network.to_string(),
                mint: mint.clone(),
                name: metadata.name,
                symbol: metadata.symbol,
                logo_uri: metadata.logo_uri,
                updated_at: 0,
            })
        })
        .collect()
}

fn merge_cached_token_metadata(
    network: &str,
    metadata_by_mint: &mut HashMap<String, TokenMetadata>,
    mints: &[String],
) -> Vec<String> {
    let mut missing_mints = Vec::new();
    let now = now_unix_secs_lossy();
    let cached_records = wallet_store::get_token_metadata(network, mints).unwrap_or_default();
    let mut cached_by_mint: HashMap<String, TokenMetadataRecord> = cached_records
        .into_iter()
        .map(|record| (record.mint.clone(), record))
        .collect();

    for mint in mints {
        if let Some(local_metadata) = local_token_metadata(mint) {
            metadata_by_mint.insert(mint.clone(), local_metadata.into_token_metadata());
            continue;
        }

        let Some(cached) = cached_by_mint.remove(mint) else {
            if metadata_by_mint
                .get(mint)
                .map(token_metadata_is_complete)
                .unwrap_or(false)
            {
                continue;
            }
            missing_mints.push(mint.clone());
            continue;
        };
        let cached_metadata = token_metadata_from_cache_record(&cached);
        metadata_by_mint
            .entry(mint.clone())
            .or_default()
            .merge_missing(cached_metadata.clone());
        let stale = now.saturating_sub(cached.updated_at) > TOKEN_METADATA_CACHE_TTL_SECS;
        if stale || !token_metadata_is_complete(&cached_metadata) {
            missing_mints.push(mint.clone());
        }
    }

    missing_mints
}

async fn load_missing_token_metadata(
    client: &RpcClient,
    network: &str,
    metadata_by_mint: &mut HashMap<String, TokenMetadata>,
    missing_mints: &[String],
) {
    if missing_mints.is_empty() {
        return;
    }

    let fresh_metadata_by_mint = load_token_metadata_map(client, missing_mints).await;
    if fresh_metadata_by_mint.is_empty() {
        return;
    }

    for (mint, metadata) in fresh_metadata_by_mint.iter() {
        metadata_by_mint
            .entry(mint.clone())
            .or_default()
            .merge_missing(normalize_token_metadata(mint, metadata.clone()));
    }

    let records = token_metadata_records(network, &fresh_metadata_by_mint);
    let _ = wallet_store::save_token_metadata(network, &records);
}

fn load_squads_multisig(client: &RpcClient, multisig: &Pubkey) -> Result<SquadsMultisig, ApiError> {
    let account = client.get_account(multisig).map_err(|e| ApiError {
        message: format!("读取 Squads 多签账户失败: {}", e),
    })?;
    if account.owner != squads_v4::SQUADS_PROGRAM_ID || account.executable {
        return Err(ApiError {
            message: format!(
                "{} 不是由 Squads v4 Program 持有的非 executable 多签账户",
                multisig
            ),
        });
    }
    let state = squads_v4::decode_account::<SquadsMultisig>(&account.data, "Multisig")
        .map_err(|message| ApiError { message })?;
    let (expected_multisig, expected_bump) = squads_v4::multisig_pda_with_bump(&state.create_key);
    if expected_multisig != *multisig || expected_bump != state.bump {
        return Err(ApiError {
            message: format!(
                "Squads 多签地址或 bump 与 create key 派生结果不一致；请求 {}，派生 {}",
                multisig, expected_multisig
            ),
        });
    }
    Ok(state)
}

fn load_squads_proposal(client: &RpcClient, proposal: &Pubkey) -> Result<SquadsProposal, ApiError> {
    let account = client.get_account(proposal).map_err(|e| ApiError {
        message: format!("读取 Squads 提案失败: {}", e),
    })?;
    squads_v4::decode_account::<SquadsProposal>(&account.data, "Proposal")
        .map_err(|message| ApiError { message })
}

fn load_squads_vault_transaction(
    client: &RpcClient,
    transaction: &Pubkey,
) -> Result<SquadsVaultTransaction, ApiError> {
    let account = client.get_account(transaction).map_err(|e| ApiError {
        message: format!("读取 Squads 交易账户失败: {}", e),
    })?;
    squads_v4::decode_account::<SquadsVaultTransaction>(&account.data, "VaultTransaction")
        .map_err(|message| ApiError { message })
}

fn load_squads_program_config(client: &RpcClient) -> Result<SquadsProgramConfig, ApiError> {
    let program_config = squads_v4::program_config_pda();
    let account = client.get_account(&program_config).map_err(|e| ApiError {
        message: format!("读取 Squads program config 失败: {}", e),
    })?;
    squads_v4::decode_account::<SquadsProgramConfig>(&account.data, "ProgramConfig")
        .map_err(|message| ApiError { message })
}

fn next_squads_transaction_index(multisig: &SquadsMultisig) -> Result<u64, ApiError> {
    multisig
        .transaction_index
        .checked_add(1)
        .ok_or_else(|| ApiError {
            message: "Squads transaction index 超出范围".to_string(),
        })
}

fn sign_and_send_single(
    client: &RpcClient,
    instruction: solana_sdk::instruction::Instruction,
    signer: &Keypair,
) -> Result<String, ApiError> {
    sign_and_send(client, vec![instruction], &[signer], &signer.pubkey())
}

fn require_squads_member(multisig: &SquadsMultisig, signer: &Pubkey) -> Result<(), ApiError> {
    if multisig.members.iter().any(|member| member.key == *signer) {
        Ok(())
    } else {
        Err(ApiError {
            message: format!("签名钱包 {} 不是该 Squads 多签成员", signer),
        })
    }
}

fn loader_upgrade_authority_from_programdata(data: &[u8]) -> Result<Pubkey, ApiError> {
    if data.len() < UpgradeableLoaderState::size_of_programdata_metadata() {
        return Err(ApiError {
            message: "ProgramData 账户长度不足".to_string(),
        });
    }
    let state: UpgradeableLoaderState =
        bincode::deserialize(&data[..UpgradeableLoaderState::size_of_programdata_metadata()])
            .map_err(|_| ApiError {
                message: "ProgramData loader state 解析失败".to_string(),
            })?;
    match state {
        UpgradeableLoaderState::ProgramData {
            upgrade_authority_address: Some(authority),
            ..
        } => Ok(authority),
        UpgradeableLoaderState::ProgramData {
            upgrade_authority_address: None,
            ..
        } => Err(ApiError {
            message: "ProgramData 没有 upgrade authority，无法升级".to_string(),
        }),
        _ => Err(ApiError {
            message: "账户不是 upgradeable-loader ProgramData".to_string(),
        }),
    }
}

fn require_program_upgrade_authority(
    client: &RpcClient,
    program_id: &Pubkey,
    expected_authority: &Pubkey,
) -> Result<Pubkey, ApiError> {
    let programdata_address = get_program_data_address(program_id);
    let response = client
        .get_multiple_accounts_with_commitment(
            &[*program_id, programdata_address],
            CommitmentConfig::confirmed(),
        )
        .map_err(|error| ApiError {
            message: format!("读取 Program upgrade authority 失败: {error}"),
        })?;
    let mut accounts = response.value.into_iter();
    let program_account = accounts.next().flatten().ok_or_else(|| ApiError {
        message: format!("Program {} 不存在", program_id),
    })?;
    let programdata_account = accounts.next().flatten().ok_or_else(|| ApiError {
        message: format!("ProgramData {} 不存在", programdata_address),
    })?;
    if program_account.owner != UPGRADEABLE_LOADER_ID
        || programdata_account.owner != UPGRADEABLE_LOADER_ID
        || !program_account.executable
    {
        return Err(ApiError {
            message: "目标 Program 不是 upgradeable loader 可升级程序".to_string(),
        });
    }
    let state: UpgradeableLoaderState =
        bincode::deserialize(&program_account.data).map_err(|_| ApiError {
            message: "Program loader state 解析失败".to_string(),
        })?;
    match state {
        UpgradeableLoaderState::Program {
            programdata_address: actual,
        } if actual == programdata_address => {}
        _ => {
            return Err(ApiError {
                message: "Program 指向的 ProgramData 与派生地址不一致".to_string(),
            })
        }
    }
    let authority = loader_upgrade_authority_from_programdata(&programdata_account.data)?;
    if authority != *expected_authority {
        return Err(ApiError {
            message: format!(
                "Program upgrade authority 为 {}，预期为 {}",
                authority, expected_authority
            ),
        });
    }
    Ok(programdata_address)
}

fn require_upgrade_buffer_authority(
    client: &RpcClient,
    buffer: &Pubkey,
    expected_authority: &Pubkey,
) -> Result<(), ApiError> {
    let account = client.get_account(buffer).map_err(|error| ApiError {
        message: format!("读取升级 Buffer {} 失败: {error}", buffer),
    })?;
    if account.owner != UPGRADEABLE_LOADER_ID || account.executable {
        return Err(ApiError {
            message: format!("{} 不是 upgradeable loader Buffer", buffer),
        });
    }
    let metadata_len = UpgradeableLoaderState::size_of_buffer_metadata();
    if account.data.len() < metadata_len {
        return Err(ApiError {
            message: "升级 Buffer 账户长度不足".to_string(),
        });
    }
    let state: UpgradeableLoaderState = bincode::deserialize(&account.data[..metadata_len])
        .map_err(|_| ApiError {
            message: "升级 Buffer loader state 解析失败".to_string(),
        })?;
    match state {
        UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        } if authority == *expected_authority => Ok(()),
        UpgradeableLoaderState::Buffer {
            authority_address: Some(authority),
        } => Err(ApiError {
            message: format!(
                "升级 Buffer authority 为 {}，预期为 Squads vault {}",
                authority, expected_authority
            ),
        }),
        UpgradeableLoaderState::Buffer {
            authority_address: None,
        } => Err(ApiError {
            message: "升级 Buffer 没有 authority".to_string(),
        }),
        _ => Err(ApiError {
            message: "账户不是 upgradeable-loader Buffer".to_string(),
        }),
    }
}

struct PumpSellRoute {
    dex_type: DexType,
    extension_params: DexParamEnum,
    output_token_type: TradeTokenType,
    create_output_token_ata: bool,
}

impl PumpSellRoute {
    fn pumpfun(params: PumpFunParams) -> Self {
        Self {
            dex_type: DexType::PumpFun,
            extension_params: DexParamEnum::PumpFun(params.with_quote_mint(WSOL_TOKEN_ACCOUNT)),
            output_token_type: TradeTokenType::SOL,
            create_output_token_ata: false,
        }
    }

    fn pumpswap(params: PumpSwapParams) -> Self {
        Self {
            dex_type: DexType::PumpSwap,
            extension_params: DexParamEnum::PumpSwap(params),
            output_token_type: TradeTokenType::WSOL,
            create_output_token_ata: true,
        }
    }
}

struct PumpSellExecution {
    signature: String,
    dex_type: DexType,
    sold_raw_amount: u64,
    decimals: u8,
    source_account: String,
}

fn pump_sell_dex_value(dex_type: DexType) -> &'static str {
    match dex_type {
        DexType::PumpFun => "pumpfun",
        DexType::PumpSwap => "pumpswap",
        _ => "unknown",
    }
}

fn pump_sell_market_value(dex_type: DexType) -> &'static str {
    match dex_type {
        DexType::PumpFun => "inner",
        DexType::PumpSwap => "outer",
        _ => "unknown",
    }
}

fn is_retryable_rpc_read_error(error: &anyhow::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("error sending request")
        || message.contains("timed out")
        || message.contains("timeout")
        || message.contains("超时")
        || message.contains("connection rate limits exceeded")
        || message.contains("too many requests")
        || message.contains("429")
        || message.contains("econnreset")
        || message.contains("connection reset")
        || message.contains("connection refused")
        || message.contains("temporarily unavailable")
}

async fn pumpfun_params_from_mint_by_rpc(
    rpc: &SolanaRpcClient,
    mint: &Pubkey,
) -> Result<PumpFunParams, anyhow::Error> {
    let mut last_error = None;
    for attempt in 1..=PUMPFUN_PARAM_RPC_ATTEMPTS {
        let retry_error = match tokio::time::timeout(
            Duration::from_millis(PUMPFUN_PARAM_RPC_TIMEOUT_MS),
            PumpFunParams::from_mint_by_rpc(rpc, mint),
        )
        .await
        {
            Ok(Ok(params)) => return Ok(params),
            Ok(Err(error)) if is_retryable_rpc_read_error(&error) => error,
            Ok(Err(error)) => return Err(error),
            Err(_) => anyhow::anyhow!(
                "Pump.fun 参数读取超时 (timeout after {} ms)",
                PUMPFUN_PARAM_RPC_TIMEOUT_MS
            ),
        };

        tracing::warn!(
            "Pump.fun param read failed for mint {} on attempt {}/{}: {}",
            mint,
            attempt,
            PUMPFUN_PARAM_RPC_ATTEMPTS,
            retry_error
        );
        last_error = Some(retry_error);
        if attempt < PUMPFUN_PARAM_RPC_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(PUMPFUN_PARAM_RETRY_DELAY_MS)).await;
            continue;
        }
        break;
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Pump.fun 参数读取失败")))
}

async fn pumpswap_params_from_mint_by_rpc(
    rpc: &SolanaRpcClient,
    mint: &Pubkey,
) -> Result<PumpSwapParams, anyhow::Error> {
    let (pool_address, pool_data) = pumpswap::find_by_mint(rpc, mint).await?;
    PumpSwapParams::from_pool_data(rpc, &pool_address, &pool_data).await
}

async fn resolve_pump_sell_route(
    rpc: &SolanaRpcClient,
    mint: &Pubkey,
    requested_dex_type: DexType,
) -> Result<PumpSellRoute, ApiError> {
    match requested_dex_type {
        DexType::PumpFun => match pumpfun_params_from_mint_by_rpc(rpc, mint).await {
            Ok(params) => {
                if !params.bonding_curve.complete {
                    return Ok(PumpSellRoute::pumpfun(params));
                }

                match pumpswap_params_from_mint_by_rpc(rpc, mint).await {
                    Ok(pumpswap_params) => {
                        tracing::info!(
                            "Pump.fun bonding curve is complete; routing sell to PumpSwap for mint {}",
                            mint
                        );
                        Ok(PumpSellRoute::pumpswap(pumpswap_params))
                    }
                    Err(pumpswap_err) => Err(ApiError {
                        message: format!(
                            "该代币 Pump.fun bonding curve 已完成，但未找到可用 PumpSwap 池，无法自动外盘卖出: {}",
                            pumpswap_err
                        ),
                    }),
                }
            }
            Err(pumpfun_err) => match pumpswap_params_from_mint_by_rpc(rpc, mint).await {
                Ok(pumpswap_params) => {
                    tracing::info!(
                        "Pump.fun params failed for mint {}; routing sell to PumpSwap: {}",
                        mint,
                        pumpfun_err
                    );
                    Ok(PumpSellRoute::pumpswap(pumpswap_params))
                }
                Err(pumpswap_err) => {
                    let message = if is_retryable_rpc_read_error(&pumpfun_err) {
                        format!(
                            "RPC 节点暂时无法读取 Pump.fun 参数: {}；同时未找到 PumpSwap 池: {}。该代币可能仍在 Pump.fun 内盘，请稍后重试，或在网络设置中换用稳定的主网 RPC。",
                            pumpfun_err, pumpswap_err
                        )
                    } else {
                        format!(
                            "获取 Pump.fun 参数失败: {}；同时获取 PumpSwap 参数失败: {}。请确认代币还在 Pump.fun 内盘或已经迁移到 PumpSwap 外盘。",
                            pumpfun_err, pumpswap_err
                        )
                    };
                    Err(ApiError { message })
                }
            },
        },
        DexType::PumpSwap => pumpswap_params_from_mint_by_rpc(rpc, mint)
            .await
            .map(PumpSellRoute::pumpswap)
            .map_err(|e| ApiError {
                message: format!("获取 PumpSwap 参数失败: {}", e),
            }),
        _ => Err(ApiError {
            message: "不支持的交易类型".to_string(),
        }),
    }
}

async fn execute_pump_sell(
    keypair: Keypair,
    mint: String,
    rpc_url: &str,
    human_amount: Option<DecimalAmount>,
    sell_percent_bps: Option<u64>,
    slippage: u64,
    dex_type: DexType,
) -> Result<PumpSellExecution, ApiError> {
    let mint_pubkey = Pubkey::from_str(&mint).map_err(|_| ApiError {
        message: "无效的代币地址".to_string(),
    })?;
    let payer = Arc::new(keypair);
    let commitment = CommitmentConfig::confirmed();
    let rpc_client = RpcClient::new_with_timeout_and_commitment(
        rpc_url.to_string(),
        Duration::from_secs(RPC_QUERY_TIMEOUT_SECS),
        commitment,
    );
    let (sell_source, raw_amount) = select_pump_sell_token_source(
        &rpc_client,
        &mint_pubkey,
        &payer.pubkey(),
        human_amount.as_ref(),
        sell_percent_bps,
    )?;
    let use_seed = sell_source.kind.use_seed();
    let swqos_configs = default_swqos_configs(rpc_url);
    let init_trade_config = TradeConfig::builder(rpc_url.to_string(), swqos_configs, commitment)
        .create_wsol_ata_on_startup(false)
        .use_seed_optimize(use_seed)
        .check_min_tip(false)
        .log_enabled(false)
        .swqos_cores_from_end(false)
        .mev_protection(false)
        .build();
    let init_client = SolanaTrade::new(payer.clone(), init_trade_config).await;
    let client = swqos_only_sell_client(&init_client, payer.clone(), use_seed)?;

    let recent_blockhash = client
        .infrastructure
        .rpc
        .get_latest_blockhash()
        .await
        .map_err(|e| ApiError {
            message: format!("获取 blockhash 失败: {}", e),
        })?;
    let gas_fee_strategy = GasFeeStrategy::new();
    gas_fee_strategy.set_global_fee_strategy(
        150000,
        150000,
        500000,
        500000,
        DEFAULT_SWQOS_TIP_SOL,
        DEFAULT_SWQOS_TIP_SOL,
    );

    let route = resolve_pump_sell_route(&client.infrastructure.rpc, &mint_pubkey, dex_type).await?;
    let actual_dex_type = route.dex_type;
    tracing::info!(
        "Submitting pump sell mint={} requested_dex={} actual_dex={} raw_amount={} source_kind={} source_account={} source_balance={} decimals={} token_program={} use_seed={} slippage_bps={}",
        mint_pubkey,
        pump_sell_dex_value(dex_type),
        pump_sell_dex_value(actual_dex_type),
        raw_amount,
        sell_source.kind.label(),
        sell_source.account,
        sell_source.raw_amount,
        sell_source.decimals,
        sell_source.token_program,
        use_seed,
        slippage
    );

    let sell_params = TradeSellParams {
        dex_type: actual_dex_type,
        output_token_type: route.output_token_type,
        mint: mint_pubkey,
        input_token_amount: raw_amount,
        slippage_basis_points: Some(slippage),
        recent_blockhash: Some(recent_blockhash),
        with_tip: true,
        extension_params: route.extension_params,
        address_lookup_table_account: None,
        // Return once the transaction is submitted and a signature is available.
        // Waiting for confirmation inside this HTTP request can outlive the browser/proxy
        // connection and surfaces as ECONNRESET even though the sell was already submitted.
        wait_tx_confirmed: false,
        create_output_token_ata: route.create_output_token_ata,
        close_output_token_ata: false,
        close_mint_token_ata: false,
        durable_nonce: None,
        fixed_output_token_amount: None,
        gas_fee_strategy,
        simulate: false,
        wait_for_all_submits: false,
        grpc_recv_us: None,
    };

    let sell_result = tokio::time::timeout(
        Duration::from_secs(PUMP_SELL_SUBMIT_TIMEOUT_SECS),
        AssertUnwindSafe(client.sell(sell_params)).catch_unwind(),
    )
    .await
    .map_err(|_| {
        let message = format!(
            "卖出提交超时，{} 秒内没有拿到交易签名；请先刷新交易记录或链上浏览器确认是否已提交成功",
            PUMP_SELL_SUBMIT_TIMEOUT_SECS
        );
        tracing::warn!(
            "Pump sell submit timeout mint={} dex={} raw_amount={} after {}s",
            mint_pubkey,
            pump_sell_dex_value(actual_dex_type),
            raw_amount,
            PUMP_SELL_SUBMIT_TIMEOUT_SECS
        );
        ApiError { message }
    })?
    .map_err(|panic_payload| {
        let message = format!("卖出执行异常: {}", panic_message(panic_payload.as_ref()));
        tracing::error!(
            "Pump sell panic mint={} dex={} raw_amount={}: {}",
            mint_pubkey,
            pump_sell_dex_value(actual_dex_type),
            raw_amount,
            message
        );
        ApiError { message }
    })?;

    let (success, signatures, error, _latency_info) = sell_result.map_err(|e| {
        let message = format!("卖出失败: {}", e);
        tracing::warn!(
            "Pump sell failed mint={} dex={} raw_amount={}: {}",
            mint_pubkey,
            pump_sell_dex_value(actual_dex_type),
            raw_amount,
            message
        );
        ApiError { message }
    })?;
    if !success {
        let error_msg = error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Unknown error".to_string());
        tracing::warn!(
            "Pump sell returned unsuccessful mint={} dex={} raw_amount={}: {}",
            mint_pubkey,
            pump_sell_dex_value(actual_dex_type),
            raw_amount,
            error_msg
        );
        return Err(ApiError {
            message: format!("卖出失败: {}", error_msg),
        });
    }

    let signature = signatures
        .first()
        .map(|sig| sig.to_string())
        .ok_or_else(|| ApiError {
            message: "卖出成功但未返回交易签名".to_string(),
        })?;

    Ok(PumpSellExecution {
        signature,
        dex_type: actual_dex_type,
        sold_raw_amount: raw_amount,
        decimals: sell_source.decimals,
        source_account: sell_source.account.to_string(),
    })
}

#[derive(RustEmbed)]
#[folder = "out"]
struct Assets;

const DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const TESTNET_RPC_URL: &str = "https://api.testnet.solana.com";
const PUBLICNODE_MAINNET_RPC_URL: &str = "https://solana.publicnode.com";
const PUBLICNODE_TESTNET_RPC_URL: &str = "https://solana-testnet-rpc.publicnode.com";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sol_safekey_ui=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let _ = secure_body_keypair();

    let app = Router::new()
        // Health
        .route("/api/health", get(health))
        .route("/api/health/", get(health))
        .route("/api/secure/session", get(secure_session))
        .route("/api/secure/session/", get(secure_session))
        // Core Functions (1-3)
        .route("/api/keys/create", post(create_key))
        .route("/api/keys/create/", post(create_key))
        .route("/api/keys/encrypt", post(encrypt_key))
        .route("/api/keys/encrypt/", post(encrypt_key))
        .route("/api/keys/create-encrypted", post(create_encrypted_key))
        .route("/api/keys/create-encrypted/", post(create_encrypted_key))
        .route("/api/keys/decrypt", post(decrypt_key))
        .route("/api/keys/decrypt/", post(decrypt_key))
        .route("/api/keys/create-keystore", post(create_keystore))
        .route("/api/keys/create-keystore/", post(create_keystore))
        .route("/api/keys/import-keystore", post(import_keystore))
        .route("/api/keys/import-keystore/", post(import_keystore))
        .route("/api/wallets", get(list_wallets).post(save_keystore_wallet))
        .route(
            "/api/wallets/",
            get(list_wallets).post(save_keystore_wallet),
        )
        .route(
            "/api/wallets/{wallet_id}",
            patch(rename_wallet).delete(delete_wallet),
        )
        .route(
            "/api/wallets/{wallet_id}/",
            patch(rename_wallet).delete(delete_wallet),
        )
        .route("/api/wallets/{wallet_id}/delete", post(delete_wallet_post))
        .route("/api/wallets/{wallet_id}/delete/", post(delete_wallet_post))
        .route("/api/wallets/{wallet_id}/export", post(export_wallet))
        .route("/api/wallets/{wallet_id}/export/", post(export_wallet))
        .route(
            "/api/wallets/{wallet_id}/migrate-keystore",
            post(migrate_wallet_keystore),
        )
        .route(
            "/api/wallets/{wallet_id}/migrate-keystore/",
            post(migrate_wallet_keystore),
        )
        .route(
            "/api/wallets/{wallet_id}/export-private-key",
            post(export_wallet_private_key),
        )
        .route(
            "/api/wallets/{wallet_id}/export-private-key/",
            post(export_wallet_private_key),
        )
        // Wallet Management (U, 7)
        .route("/api/wallet/balance", post(get_balance))
        .route("/api/wallet/balance/", post(get_balance))
        .route("/api/wallet/assets", post(get_assets))
        .route("/api/wallet/assets/", post(get_assets))
        .route("/api/wallet/transactions", post(get_wallet_transactions))
        .route("/api/wallet/transactions/", post(get_wallet_transactions))
        .route("/api/wallet/unlock", post(unlock_wallet))
        .route("/api/wallet/unlock/", post(unlock_wallet))
        .route("/api/wallet/get-pubkey", post(get_pubkey))
        .route("/api/wallet/get-pubkey/", post(get_pubkey))
        // SOL Operations (8)
        .route("/api/transfer/sol", post(transfer_sol))
        .route("/api/transfer/sol/", post(transfer_sol))
        // WSOL Operations (9-12)
        .route("/api/wsol/create-ata", post(create_wsol_ata))
        .route("/api/wsol/create-ata/", post(create_wsol_ata))
        .route("/api/wsol/wrap", post(wrap_sol))
        .route("/api/wsol/wrap/", post(wrap_sol))
        .route("/api/wsol/unwrap", post(unwrap_sol))
        .route("/api/wsol/unwrap/", post(unwrap_sol))
        .route("/api/wsol/close-ata", post(close_wsol_ata))
        .route("/api/wsol/close-ata/", post(close_wsol_ata))
        // 2FA Operations (4-6)
        .route("/api/2fa/setup", post(setup_2fa))
        .route("/api/2fa/setup/", post(setup_2fa))
        .route("/api/2fa/create-tfa", post(create_triple_factor_wallet))
        .route("/api/2fa/create-tfa/", post(create_triple_factor_wallet))
        .route("/api/2fa/unlock-tfa", post(unlock_triple_factor_wallet))
        .route("/api/2fa/unlock-tfa/", post(unlock_triple_factor_wallet))
        // Pump.fun Operations (15-18)
        .route("/api/pumpfun/sell", post(pumpfun_sell))
        .route("/api/pumpfun/sell/", post(pumpfun_sell))
        .route("/api/pumpfun/cashback-info", post(pumpfun_cashback_info))
        .route("/api/pumpfun/cashback-info/", post(pumpfun_cashback_info))
        .route("/api/pumpfun/cashback", post(pumpfun_cashback))
        .route("/api/pumpfun/cashback/", post(pumpfun_cashback))
        .route("/api/pumpswap/sell", post(pumpswap_sell))
        .route("/api/pumpswap/sell/", post(pumpswap_sell))
        .route("/api/pumpswap/cashback-info", post(pumpswap_cashback_info))
        .route("/api/pumpswap/cashback-info/", post(pumpswap_cashback_info))
        .route("/api/pumpswap/cashback", post(pumpswap_cashback))
        .route("/api/pumpswap/cashback/", post(pumpswap_cashback))
        // Token Operations (13)
        .route("/api/token/mint-info", post(token_mint_info))
        .route("/api/token/mint-info/", post(token_mint_info))
        .route("/api/transfer/token", post(transfer_token))
        .route("/api/transfer/token/", post(transfer_token))
        // Nonce Operations (14)
        .route("/api/nonce/create", post(create_nonce_account))
        .route("/api/nonce/create/", post(create_nonce_account))
        .route("/api/nonce/list", post(list_nonce_accounts))
        .route("/api/nonce/list/", post(list_nonce_accounts))
        // Program Deployment
        .route("/api/program/deploy", post(deploy_generic_program))
        .route("/api/program/deploy/", post(deploy_generic_program))
        .route("/api/program/deploy-source", post(program_deploy_source))
        .route("/api/program/deploy-source/", post(program_deploy_source))
        .route(
            "/api/program/deploy-source/build/{job_id}",
            get(program_deploy_source_build_status),
        )
        .route(
            "/api/program/deploy-source/build/{job_id}/",
            get(program_deploy_source_build_status),
        )
        .route(
            "/api/program/deployment-journal",
            post(program_deployment_journal),
        )
        .route(
            "/api/program/deployment-journal/",
            post(program_deployment_journal),
        )
        .route("/api/program/info", post(program_info))
        .route("/api/program/info/", post(program_info))
        // Squads v4 multisig
        .route("/api/squads/create", post(squads_create))
        .route("/api/squads/create/", post(squads_create))
        .route("/api/squads/info", post(squads_info))
        .route("/api/squads/info/", post(squads_info))
        .route("/api/squads/proposals", post(squads_proposals))
        .route("/api/squads/proposals/", post(squads_proposals))
        .route(
            "/api/squads/proposal/sol-transfer",
            post(squads_sol_transfer_proposal),
        )
        .route(
            "/api/squads/proposal/sol-transfer/",
            post(squads_sol_transfer_proposal),
        )
        .route(
            "/api/squads/proposal/token-transfer",
            post(squads_token_transfer_proposal),
        )
        .route(
            "/api/squads/proposal/token-transfer/",
            post(squads_token_transfer_proposal),
        )
        .route(
            "/api/squads/program/prepare-upgrade-buffer",
            post(squads_prepare_upgrade_buffer),
        )
        .route(
            "/api/squads/program/prepare-upgrade-buffer/",
            post(squads_prepare_upgrade_buffer),
        )
        .route(
            "/api/squads/proposal/program-upgrade",
            post(squads_program_upgrade_proposal),
        )
        .route(
            "/api/squads/proposal/program-upgrade/",
            post(squads_program_upgrade_proposal),
        )
        .route(
            "/api/squads/proposal/approve",
            post(squads_proposal_approve),
        )
        .route(
            "/api/squads/proposal/approve/",
            post(squads_proposal_approve),
        )
        .route("/api/squads/proposal/reject", post(squads_proposal_reject))
        .route("/api/squads/proposal/reject/", post(squads_proposal_reject))
        .route(
            "/api/squads/proposal/execute",
            post(squads_proposal_execute),
        )
        .route(
            "/api/squads/proposal/execute/",
            post(squads_proposal_execute),
        )
        .route(
            "/api/squads/program/set-authority",
            post(squads_set_program_authority),
        )
        .route(
            "/api/squads/program/set-authority/",
            post(squads_set_program_authority),
        )
        .route_layer(middleware::from_fn(decrypt_secure_body))
        .route_layer(middleware::from_fn(require_api_token))
        .route_layer(middleware::from_fn(require_local_origin))
        .layer(DefaultBodyLimit::max(MAX_JSON_BODY_BYTES))
        .layer(middleware::from_fn(add_security_headers))
        .layer(middleware::from_fn(catch_panics))
        .fallback(serve_assets);

    const DEFAULT_PORT: u16 = 3841;
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let host: IpAddr = host.parse()?;
    if !host.is_loopback() {
        anyhow::bail!(
            "Sol SafeKey API contains local-only wallet operations and must bind to a loopback address"
        );
    }
    let addr = SocketAddr::new(host, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "sol-safekey-ui",
        "version": env!("CARGO_PKG_VERSION"),
        "features": ["keys", "wallet", "faucet", "transfer", "wsol", "token", "nonce", "program", "squads-v4"]
    }))
}

async fn secure_session() -> Result<Json<SecureSessionResponse>, ApiError> {
    // This key prevents sensitive JSON from appearing as plaintext in DevTools/request logs.
    // It is not an authentication boundary for JavaScript already running in this origin.
    Ok(Json(SecureSessionResponse {
        version: SECURE_BODY_VERSION,
        algorithm: "RSA-OAEP-256+A256GCM",
        public_key_pem: secure_body_keypair().public_key_pem.clone(),
    }))
}

// ============= Helper Functions =============

struct RpcSelector {
    network: String,
    url: String,
}

fn normalize_network_name(network: &str) -> Result<&'static str, ApiError> {
    match network {
        "mainnet" => Ok("mainnet"),
        "devnet" => Ok("devnet"),
        "testnet" => Ok("testnet"),
        _ => Err(ApiError {
            message: format!("不支持的 Solana 网络标签: {network}"),
        }),
    }
}

fn default_rpc_url_for(network: &str) -> &'static str {
    match network {
        "devnet" => DEVNET_RPC_URL,
        "testnet" => TESTNET_RPC_URL,
        _ => DEFAULT_RPC_URL,
    }
}

fn is_valid_rpc_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://")) && url.len() <= 512
}

fn parse_rpc_selector(value: &str) -> Result<RpcSelector, ApiError> {
    if let Some(rest) = value.strip_prefix("rpc:") {
        let (network, encoded_url) = rest.split_once(':').ok_or_else(|| ApiError {
            message: "无效的 RPC 配置".to_string(),
        })?;
        let network = normalize_network_name(network)?.to_string();
        let decoded_url = urlencoding::decode(encoded_url)
            .map_err(|_| ApiError {
                message: "无效的 RPC URL".to_string(),
            })?
            .trim()
            .trim_end_matches('/')
            .to_string();
        if !is_valid_rpc_url(&decoded_url) {
            return Err(ApiError {
                message: "RPC URL 必须使用 http 或 https".to_string(),
            });
        }
        return Ok(RpcSelector {
            network,
            url: decoded_url,
        });
    }

    let network = normalize_network_name(value)?.to_string();
    Ok(RpcSelector {
        url: default_rpc_url_for(&network).to_string(),
        network,
    })
}

fn rpc_selector(network: Option<&str>) -> Result<RpcSelector, ApiError> {
    parse_rpc_selector(network.unwrap_or("mainnet"))
}

fn get_rpc_url(network: Option<&str>) -> Result<String, ApiError> {
    rpc_selector(network).map(|selector| selector.url)
}

// ============= Core Functions (1-3) =============

// 1. Create Plaintext Key
#[derive(Deserialize)]
struct CreateKeyRequest {
    #[serde(default)]
    name: Option<String>,
}
#[derive(Serialize)]
struct CreateKeyResponse {
    public_key: String,
    secret_key: String,
    name: String,
}

async fn create_key(
    Json(req): Json<CreateKeyRequest>,
) -> Result<Json<CreateKeyResponse>, ApiError> {
    require_secret_export_enabled()?;
    let name = validate_optional_label(req.name, "名称")?.unwrap_or_else(|| "default".to_string());
    let keypair = KeyManager::generate_keypair();
    Ok(Json(CreateKeyResponse {
        public_key: keypair.pubkey().to_string(),
        secret_key: keypair.to_base58_string(),
        name,
    }))
}

// 2. Create Encrypted Key / Encrypt Key
#[derive(Deserialize)]
struct EncryptKeyRequest {
    secret_key: String,
    password: String,
}
#[derive(Serialize)]
struct EncryptKeyResponse {
    encrypted_key: String,
}

#[derive(Deserialize)]
struct CreateEncryptedKeyRequest {
    password: String,
}
#[derive(Serialize)]
struct CreateEncryptedKeyResponse {
    public_key: String,
    encrypted_key: String,
}

async fn encrypt_key(
    Json(req): Json<EncryptKeyRequest>,
) -> Result<Json<EncryptKeyResponse>, ApiError> {
    require_direct_secret_input_enabled()?;
    require_nonempty(req.secret_key.trim(), "私钥")?;
    require_nonempty(req.password.as_str(), "密码")?;
    let encrypted = KeyManager::encrypt_with_password(&req.secret_key, &req.password)
        .map_err(|e| ApiError { message: e })?;
    Ok(Json(EncryptKeyResponse {
        encrypted_key: encrypted,
    }))
}

async fn create_encrypted_key(
    Json(req): Json<CreateEncryptedKeyRequest>,
) -> Result<Json<CreateEncryptedKeyResponse>, ApiError> {
    require_nonempty(req.password.as_str(), "密码")?;
    let keypair = KeyManager::generate_keypair();
    let private_key = keypair.to_base58_string();
    let encrypted = KeyManager::encrypt_with_password(&private_key, &req.password)
        .map_err(|e| ApiError { message: e })?;
    Ok(Json(CreateEncryptedKeyResponse {
        public_key: keypair.pubkey().to_string(),
        encrypted_key: encrypted,
    }))
}

// 3. Decrypt Key
#[derive(Deserialize)]
struct DecryptKeyRequest {
    encrypted_key: String,
    password: String,
}
#[derive(Serialize)]
struct DecryptKeyResponse {
    secret_key: String,
}

async fn decrypt_key(
    Json(req): Json<DecryptKeyRequest>,
) -> Result<Json<DecryptKeyResponse>, ApiError> {
    require_secret_export_enabled()?;
    require_nonempty(req.encrypted_key.trim(), "加密私钥")?;
    require_nonempty(req.password.as_str(), "密码")?;
    let decrypted = KeyManager::decrypt_with_password(&req.encrypted_key, &req.password)
        .map_err(|e| ApiError { message: e })?;
    Ok(Json(DecryptKeyResponse {
        secret_key: decrypted,
    }))
}

// Create Keystore
#[derive(Deserialize)]
struct CreateKeystoreRequest {
    password: String,
    #[serde(default)]
    name: Option<String>,
}

impl Drop for CreateKeystoreRequest {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}
#[derive(Serialize)]
struct CreateKeystoreResponse {
    keystore_json: String,
    public_key: String,
}

async fn create_keystore(
    Json(mut req): Json<CreateKeystoreRequest>,
) -> Result<Json<CreateKeystoreResponse>, ApiError> {
    require_nonempty(req.password.as_str(), "密码")?;
    let name = validate_optional_label(req.name.take(), "钱包名称")?;
    let response = run_keystore_task(move || {
        let keypair = KeyManager::generate_keypair();
        let keystore_json = KeyManager::keypair_to_encrypted_json(&keypair, &req.password)
            .map_err(|message| ApiError { message })?;
        let keystore_json = with_keystore_metadata(&keystore_json, name.as_deref())?;
        Ok(CreateKeystoreResponse {
            keystore_json,
            public_key: keypair.pubkey().to_string(),
        })
    })
    .await?;
    Ok(Json(response))
}

// Import Keystore
#[derive(Deserialize)]
struct ImportKeystoreRequest {
    keystore_json: String,
    password: String,
}

impl Drop for ImportKeystoreRequest {
    fn drop(&mut self) {
        self.keystore_json.zeroize();
        self.password.zeroize();
    }
}
#[derive(Serialize)]
struct ImportKeystoreResponse {
    public_key: String,
    unlocked: bool,
}

async fn import_keystore(
    Json(req): Json<ImportKeystoreRequest>,
) -> Result<Json<ImportKeystoreResponse>, ApiError> {
    validate_keystore_size(&req.keystore_json)?;
    require_nonempty(req.password.as_str(), "密码")?;
    let response = run_keystore_task(move || {
        let keypair = KeyManager::keypair_from_encrypted_json(&req.keystore_json, &req.password)
            .map_err(|message| ApiError { message })?;
        Ok(ImportKeystoreResponse {
            public_key: keypair.pubkey().to_string(),
            unlocked: true,
        })
    })
    .await?;
    Ok(Json(response))
}

// Saved Wallets
#[derive(Deserialize)]
struct SaveKeystoreWalletRequest {
    keystore_json: String,
    password: String,
    #[serde(default)]
    name: Option<String>,
}

impl Drop for SaveKeystoreWalletRequest {
    fn drop(&mut self) {
        self.keystore_json.zeroize();
        self.password.zeroize();
    }
}
#[derive(Serialize)]
struct SaveKeystoreWalletResponse {
    wallet: WalletSummary,
}
#[derive(Serialize)]
struct ListWalletsResponse {
    wallets: Vec<WalletSummary>,
}
#[derive(Deserialize)]
struct RenameWalletRequest {
    name: String,
}
#[derive(Deserialize)]
struct ExportWalletRequest {
    password: String,
}

#[derive(Deserialize)]
struct MigrateWalletKeystoreRequest {
    current_password: String,
    new_password: String,
}

impl Drop for MigrateWalletKeystoreRequest {
    fn drop(&mut self) {
        self.current_password.zeroize();
        self.new_password.zeroize();
    }
}
#[derive(Serialize)]
struct ExportWalletResponse {
    keystore_json: String,
}
#[derive(Serialize)]
struct ExportWalletPrivateKeyResponse {
    private_key: String,
}

async fn list_wallets() -> Result<Json<ListWalletsResponse>, ApiError> {
    let wallets = wallet_store::list_summaries().map_err(|message| ApiError { message })?;
    Ok(Json(ListWalletsResponse { wallets }))
}

async fn save_keystore_wallet(
    Json(mut req): Json<SaveKeystoreWalletRequest>,
) -> Result<Json<SaveKeystoreWalletResponse>, ApiError> {
    validate_keystore_size(&req.keystore_json)?;
    require_nonempty(req.password.as_str(), "密码")?;
    let request_name = validate_optional_label(req.name.take(), "钱包名称")?;
    let name = if request_name.is_some() {
        request_name
    } else {
        validate_optional_label(keystore_metadata_name(&req.keystore_json), "钱包名称")?
    };
    let response = run_keystore_task(move || {
        let keypair = KeyManager::keypair_from_encrypted_json(&req.keystore_json, &req.password)
            .map_err(|error| ApiError {
                message: format!("keystore 校验失败: {error}"),
            })?;
        let keystore_json = with_keystore_metadata(&req.keystore_json, name.as_deref())?;
        let wallet = wallet_store::upsert(keystore_json, keypair.pubkey().to_string(), name)
            .map_err(|message| ApiError { message })?;
        Ok(SaveKeystoreWalletResponse {
            wallet: wallet.into(),
        })
    })
    .await?;
    Ok(Json(response))
}

async fn rename_wallet(
    Path(wallet_id): Path<String>,
    Json(req): Json<RenameWalletRequest>,
) -> Result<Json<SaveKeystoreWalletResponse>, ApiError> {
    validate_wallet_id(&wallet_id)?;
    validate_text_len(req.name.trim(), "钱包名称", MAX_LABEL_CHARS)?;
    let name = req.name.trim();
    if name.is_empty() {
        return Err(ApiError {
            message: "钱包名称不能为空".to_string(),
        });
    }
    let stored = wallet_store::find(&wallet_id).map_err(|message| ApiError { message })?;
    let keystore_json = with_keystore_metadata(&stored.keystore_json, Some(name))?;
    let wallet = wallet_store::update_metadata(&wallet_id, name, keystore_json)
        .map_err(|message| ApiError { message })?;
    Ok(Json(SaveKeystoreWalletResponse {
        wallet: wallet.into(),
    }))
}

async fn delete_wallet(
    Path(_wallet_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Err(ApiError {
        message: "DELETE 请求不再用于钱包删除；请使用 POST /api/wallets/{wallet_id}/delete"
            .to_string(),
    })
}

async fn delete_wallet_post(
    Path(wallet_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !req.as_object().is_some_and(|body| body.is_empty()) {
        return Err(ApiError {
            message: "删除钱包请求体必须为空对象".to_string(),
        });
    }

    validate_wallet_id(&wallet_id)?;
    wallet_store::delete(&wallet_id).map_err(|message| ApiError { message })?;
    Ok(Json(json!({ "status": "success" })))
}

async fn export_wallet(
    Path(wallet_id): Path<String>,
    Json(req): Json<ExportWalletRequest>,
) -> Result<Json<ExportWalletResponse>, ApiError> {
    validate_wallet_id(&wallet_id)?;
    let wallet = wallet_store::find(&wallet_id).map_err(|message| ApiError { message })?;
    if req.password.is_empty() {
        return Err(ApiError {
            message: "导出钱包需要提供密码".to_string(),
        });
    }
    KeyManager::keypair_from_encrypted_json(&wallet.keystore_json, &req.password).map_err(|e| {
        ApiError {
            message: format!("钱包密码校验失败: {}", e),
        }
    })?;
    let keystore_json = with_keystore_metadata(&wallet.keystore_json, Some(&wallet.name))?;
    Ok(Json(ExportWalletResponse { keystore_json }))
}

async fn migrate_wallet_keystore(
    Path(wallet_id): Path<String>,
    Json(req): Json<MigrateWalletKeystoreRequest>,
) -> Result<Json<SaveKeystoreWalletResponse>, ApiError> {
    validate_wallet_id(&wallet_id)?;
    require_nonempty(&req.current_password, "当前钱包密码")?;
    require_nonempty(&req.new_password, "新钱包密码")?;
    let wallet = wallet_store::find(&wallet_id).map_err(|message| ApiError { message })?;
    let updated = run_keystore_task(move || {
        let (migrated, verified_public_key) =
            KeyManager::migrate_encrypted_json_to_v2_with_public_key(
                &wallet.keystore_json,
                &req.current_password,
                &req.new_password,
            )
            .map_err(|error| ApiError {
                message: format!("Legacy keystore 迁移失败: {error}"),
            })?;
        if verified_public_key != wallet.public_key {
            return Err(ApiError {
                message: "Legacy keystore 实际解出的钱包地址与已保存记录不一致".to_string(),
            });
        }
        let migrated = with_keystore_metadata(&migrated, Some(&wallet.name))?;
        let updated =
            wallet_store::upsert(migrated, verified_public_key, Some(wallet.name.clone()))
                .map_err(|message| ApiError { message })?;
        if updated.id != wallet.id || updated.public_key != wallet.public_key {
            return Err(ApiError {
                message: "Keystore 迁移后的钱包身份不一致".to_string(),
            });
        }
        wallet_store::checkpoint_sensitive_rewrite().map_err(|message| ApiError { message })?;
        Ok(updated)
    })
    .await?;
    Ok(Json(SaveKeystoreWalletResponse {
        wallet: updated.into(),
    }))
}

async fn export_wallet_private_key(
    Path(wallet_id): Path<String>,
    Json(req): Json<ExportWalletRequest>,
) -> Result<Json<ExportWalletPrivateKeyResponse>, ApiError> {
    validate_wallet_id(&wallet_id)?;
    let wallet = wallet_store::find(&wallet_id).map_err(|message| ApiError { message })?;
    if req.password.is_empty() {
        return Err(ApiError {
            message: "导出私钥需要提供密码".to_string(),
        });
    }
    let keypair = KeyManager::keypair_from_encrypted_json(&wallet.keystore_json, &req.password)
        .map_err(|e| ApiError {
            message: format!("钱包密码校验失败: {}", e),
        })?;
    Ok(Json(ExportWalletPrivateKeyResponse {
        private_key: keypair.to_base58_string(),
    }))
}

// ============= Wallet Management (U, 7) =============

// U. Unlock Wallet (same as import_keystore)
#[derive(Deserialize)]
struct UnlockWalletRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
}
#[derive(Serialize)]
struct UnlockWalletResponse {
    public_key: String,
    unlocked: bool,
}

async fn unlock_wallet(
    Json(req): Json<UnlockWalletRequest>,
) -> Result<Json<UnlockWalletResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;
    Ok(Json(UnlockWalletResponse {
        public_key: keypair.pubkey().to_string(),
        unlocked: true,
    }))
}

// 7. Check SOL Balance
#[derive(Deserialize)]
struct GetBalanceRequest {
    address: String,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct GetBalanceResponse {
    balance: f64,
    address: String,
    network: String,
}

async fn get_balance(
    Json(req): Json<GetBalanceRequest>,
) -> Result<Json<GetBalanceResponse>, ApiError> {
    let pubkey = Pubkey::from_str(&req.address).map_err(|_| ApiError {
        message: "无效的地址".to_string(),
    })?;
    let selector = rpc_selector(req.network.as_deref())?;
    let mut errors = Vec::new();
    for rpc_url in fallback_rpc_urls(&selector) {
        let client = rpc_query_client_for_url(rpc_url.clone(), RPC_QUERY_TIMEOUT_SECS);
        match client.get_balance(&pubkey) {
            Ok(balance) => {
                return Ok(Json(GetBalanceResponse {
                    balance: lamports_to_sol(balance),
                    address: req.address,
                    network: selector.network.clone(),
                }));
            }
            Err(error) => {
                let message = format!("查询失败: {}", error);
                if !is_retryable_rpc_message(&message) {
                    return Err(ApiError { message });
                }
                tracing::warn!("SOL balance RPC read failed at {}: {}", rpc_url, message);
                errors.push((rpc_url, message));
            }
        }
    }

    Err(ApiError {
        message: format!(
            "所有 RPC 节点均无法查询余额: {}",
            rpc_failure_summary(&errors)
        ),
    })
}

#[derive(Deserialize)]
struct GetAssetsRequest {
    address: String,
    #[serde(default)]
    network: Option<String>,
    #[serde(default = "default_refresh_assets")]
    refresh: bool,
}

#[derive(Serialize)]
struct WalletTokenAsset {
    account: String,
    mint: String,
    amount: String,
    ui_amount_string: String,
    decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logo_uri: Option<String>,
}

#[derive(Serialize)]
struct GetAssetsResponse {
    address: String,
    network: String,
    sol_balance: f64,
    tokens: Vec<WalletTokenAsset>,
    cached: bool,
    updated_at: Option<u64>,
}

fn default_refresh_assets() -> bool {
    true
}

struct FreshWalletAssets {
    sol_balance: f64,
    tokens: Vec<WalletTokenAsset>,
    metadata_by_mint: HashMap<String, TokenMetadata>,
}

fn wallet_assets_response_from_cache(
    address: String,
    network: String,
    cached: WalletAssetsRecord,
) -> GetAssetsResponse {
    GetAssetsResponse {
        address,
        network,
        sol_balance: cached.sol_balance,
        tokens: cached
            .tokens
            .into_iter()
            .map(wallet_token_asset_from_record)
            .collect(),
        cached: true,
        updated_at: Some(cached.updated_at),
    }
}

fn wallet_token_asset_from_record(record: WalletTokenAssetRecord) -> WalletTokenAsset {
    let WalletTokenAssetRecord {
        account,
        mint,
        amount,
        ui_amount_string,
        decimals,
        name,
        symbol,
        logo_uri,
        ..
    } = record;
    let metadata = normalize_token_metadata(
        &mint,
        TokenMetadata {
            name,
            symbol,
            logo_uri,
            metadata_uri: None,
        },
    );
    WalletTokenAsset {
        account,
        mint,
        amount,
        ui_amount_string,
        decimals,
        name: metadata.name,
        symbol: metadata.symbol,
        logo_uri: metadata.logo_uri,
    }
}

fn clean_cached_metadata_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn cached_token_metadata_by_mint(
    tokens: &[WalletTokenAssetRecord],
) -> HashMap<String, TokenMetadata> {
    let mut metadata_by_mint = HashMap::new();
    for token in tokens {
        let metadata = TokenMetadata {
            name: clean_cached_metadata_string(token.name.clone()),
            symbol: clean_cached_metadata_string(token.symbol.clone()),
            logo_uri: clean_cached_metadata_string(token.logo_uri.clone()),
            metadata_uri: None,
        };
        if metadata.name.is_none() && metadata.symbol.is_none() && metadata.logo_uri.is_none() {
            continue;
        }
        let metadata = normalize_token_metadata(&token.mint, metadata);
        metadata_by_mint
            .entry(token.mint.clone())
            .or_insert_with(TokenMetadata::default)
            .merge_missing(metadata);
    }
    metadata_by_mint
}

async fn read_fresh_wallet_assets(
    client: &RpcClient,
    pubkey: &Pubkey,
    network: &str,
    cached_metadata_by_mint: &HashMap<String, TokenMetadata>,
) -> Result<FreshWalletAssets, ApiError> {
    let balance = client.get_balance(pubkey).map_err(|e| ApiError {
        message: format!("查询余额失败: {}", e),
    })?;

    let mut raw_tokens = Vec::new();
    let mut mints = Vec::new();
    let mut seen_mints = HashMap::new();
    for (account, token_account) in all_owner_token_accounts(client, pubkey)? {
        let token_amount = token_account.token_amount;
        if token_amount.amount == "0" {
            continue;
        }
        let ui_amount_string = token_amount.real_number_string_trimmed();
        if !seen_mints.contains_key(&token_account.mint) {
            seen_mints.insert(token_account.mint.clone(), true);
            mints.push(token_account.mint.clone());
        }
        raw_tokens.push((
            account,
            token_account.mint,
            token_amount.amount,
            ui_amount_string,
            token_amount.decimals,
        ));
    }
    let mut metadata_by_mint = cached_metadata_by_mint.clone();
    let missing_mints = merge_cached_token_metadata(network, &mut metadata_by_mint, &mints);
    load_missing_token_metadata(client, network, &mut metadata_by_mint, &missing_mints).await;
    let tokens: Vec<WalletTokenAsset> = raw_tokens
        .into_iter()
        .map(|(account, mint, amount, ui_amount_string, decimals)| {
            let mut metadata = metadata_by_mint.get(&mint).cloned().unwrap_or_default();
            if let Some(cached_metadata) = cached_metadata_by_mint.get(&mint) {
                metadata.merge_missing(cached_metadata.clone());
            }
            let metadata = normalize_token_metadata(&mint, metadata);
            WalletTokenAsset {
                account,
                mint,
                amount,
                ui_amount_string,
                decimals,
                name: metadata.name,
                symbol: metadata.symbol,
                logo_uri: metadata.logo_uri,
            }
        })
        .collect();

    Ok(FreshWalletAssets {
        sol_balance: lamports_to_sol(balance),
        tokens,
        metadata_by_mint,
    })
}

async fn get_assets(
    Json(req): Json<GetAssetsRequest>,
) -> Result<Json<GetAssetsResponse>, ApiError> {
    let pubkey = Pubkey::from_str(&req.address).map_err(|_| ApiError {
        message: "无效的地址".to_string(),
    })?;
    let selector = rpc_selector(req.network.as_deref())?;
    let network = selector.network.clone();
    let cached_assets = wallet_store::get_wallet_assets(&req.address, &network)
        .map_err(|message| ApiError { message })?;

    if !req.refresh {
        if let Some(cached) = cached_assets {
            return Ok(Json(wallet_assets_response_from_cache(
                req.address,
                network,
                cached,
            )));
        }

        return Ok(Json(GetAssetsResponse {
            address: req.address,
            network,
            sol_balance: 0.0,
            tokens: Vec::new(),
            cached: true,
            updated_at: None,
        }));
    }
    let cached_metadata_by_mint = cached_assets
        .as_ref()
        .map(|cached| cached_token_metadata_by_mint(&cached.tokens))
        .unwrap_or_default();

    let mut errors = Vec::new();
    let mut fresh_assets = None;
    for rpc_url in fallback_rpc_urls(&selector) {
        let client = rpc_query_client_for_url(rpc_url.clone(), RPC_QUERY_TIMEOUT_SECS);
        match read_fresh_wallet_assets(&client, &pubkey, &network, &cached_metadata_by_mint).await {
            Ok(assets) => {
                fresh_assets = Some(assets);
                break;
            }
            Err(error) if is_retryable_api_error(&error) => {
                tracing::warn!(
                    "Wallet assets RPC read failed at {}: {}",
                    rpc_url,
                    error.message
                );
                errors.push((rpc_url, error.message));
            }
            Err(error) => return Err(error),
        }
    }
    let Some(FreshWalletAssets {
        sol_balance,
        tokens,
        metadata_by_mint,
    }) = fresh_assets
    else {
        if let Some(cached) = cached_assets {
            tracing::warn!(
                "All wallet asset RPC reads failed for {} on {}; returning cached assets: {}",
                req.address,
                network,
                rpc_failure_summary(&errors)
            );
            return Ok(Json(wallet_assets_response_from_cache(
                req.address,
                network,
                cached,
            )));
        }
        return Err(ApiError {
            message: format!(
                "所有 RPC 节点均无法查询资产: {}",
                rpc_failure_summary(&errors)
            ),
        });
    };
    let cache_tokens: Vec<WalletTokenAssetRecord> = tokens
        .iter()
        .map(|token| WalletTokenAssetRecord {
            owner: req.address.clone(),
            network: network.clone(),
            account: token.account.clone(),
            mint: token.mint.clone(),
            amount: token.amount.clone(),
            ui_amount_string: token.ui_amount_string.clone(),
            decimals: token.decimals,
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            logo_uri: token.logo_uri.clone(),
            updated_at: 0,
        })
        .collect();
    let metadata_records = token_metadata_records(&network, &metadata_by_mint);
    let _ = wallet_store::save_token_metadata(&network, &metadata_records);
    let updated_at =
        wallet_store::save_wallet_assets(&req.address, &network, sol_balance, &cache_tokens)
            .map_err(|message| ApiError { message })?;

    Ok(Json(GetAssetsResponse {
        address: req.address,
        network,
        sol_balance,
        tokens,
        cached: false,
        updated_at: Some(updated_at),
    }))
}

#[derive(Deserialize)]
struct WalletTransactionsRequest {
    address: String,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    before: Option<String>,
}

#[derive(Serialize)]
struct WalletTransactionChange {
    asset: String,
    mint: Option<String>,
    amount: String,
    ui_amount: String,
    direction: String,
    decimals: u8,
}

#[derive(Serialize)]
struct WalletTransactionRecord {
    signature: String,
    slot: u64,
    block_time: Option<i64>,
    confirmation_status: Option<String>,
    err: Option<Value>,
    memo: Option<String>,
    action: String,
    summary: String,
    counterparty: Option<String>,
    programs: Vec<String>,
    changes: Vec<WalletTransactionChange>,
}

fn ui_amount_from_raw_amount(amount: u128, decimals: u8) -> String {
    let raw = amount.to_string();
    if decimals == 0 {
        return raw;
    }
    let decimals = decimals as usize;
    let padded = if raw.len() <= decimals {
        format!("{}{}", "0".repeat(decimals + 1 - raw.len()), raw)
    } else {
        raw
    };
    let whole = &padded[..padded.len() - decimals];
    let fractional = padded[padded.len() - decimals..].trim_end_matches('0');
    if fractional.is_empty() {
        whole.to_string()
    } else {
        format!("{}.{}", whole, fractional)
    }
}

fn signed_ui_amount_from_raw(delta: i128, decimals: u8) -> String {
    let sign = if delta < 0 { "-" } else { "+" };
    format!(
        "{}{}",
        sign,
        ui_amount_from_raw_amount(delta.unsigned_abs(), decimals)
    )
}

fn add_change(
    changes: &mut Vec<WalletTransactionChange>,
    asset: &str,
    mint: Option<String>,
    delta: i128,
    decimals: u8,
) {
    if delta == 0 {
        return;
    }
    changes.push(WalletTransactionChange {
        asset: asset.to_string(),
        mint,
        amount: delta.to_string(),
        ui_amount: signed_ui_amount_from_raw(delta, decimals),
        direction: if delta > 0 { "in" } else { "out" }.to_string(),
        decimals,
    });
}

fn token_symbol_from_metadata(mint: &str, metadata: Option<&TokenMetadata>) -> String {
    let normalized = normalize_token_metadata(mint, metadata.cloned().unwrap_or_default());
    normalized
        .symbol
        .filter(|symbol| !symbol.trim().is_empty())
        .unwrap_or_else(|| mint.to_string())
}

fn option_serializer_vec<T>(value: &OptionSerializer<Vec<T>>) -> &[T] {
    match value.as_ref() {
        OptionSerializer::Some(items) => items.as_slice(),
        _ => &[],
    }
}

fn parsed_message_accounts_and_programs(
    transaction: &EncodedTransaction,
) -> (Vec<String>, Vec<String>) {
    let EncodedTransaction::Json(ui_transaction) = transaction else {
        return (Vec::new(), Vec::new());
    };
    match &ui_transaction.message {
        UiMessage::Parsed(message) => {
            let accounts = message
                .account_keys
                .iter()
                .map(|account| account.pubkey.clone())
                .collect::<Vec<_>>();
            let mut programs = Vec::new();
            for instruction in &message.instructions {
                match instruction {
                    UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed)) => {
                        if !programs.contains(&parsed.program) {
                            programs.push(parsed.program.clone());
                        }
                    }
                    UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(parsed)) => {
                        if !programs.contains(&parsed.program_id) {
                            programs.push(parsed.program_id.clone());
                        }
                    }
                    UiInstruction::Compiled(_) => {}
                }
            }
            (accounts, programs)
        }
        UiMessage::Raw(message) => {
            let accounts = message.account_keys.clone();
            let mut programs = Vec::new();
            for instruction in &message.instructions {
                if let Some(program_id) = accounts.get(instruction.program_id_index as usize) {
                    if !programs.contains(program_id) {
                        programs.push(program_id.clone());
                    }
                }
            }
            (accounts, programs)
        }
    }
}

fn transaction_counterparty(accounts: &[String], owner: &str) -> Option<String> {
    accounts
        .iter()
        .find(|account| {
            account.as_str() != owner && account.as_str() != "11111111111111111111111111111111"
        })
        .cloned()
}

fn fallback_program_label(programs: &[String]) -> String {
    if programs
        .iter()
        .any(|program| program.eq_ignore_ascii_case("jupiter"))
    {
        "Jupiter".to_string()
    } else if programs
        .iter()
        .any(|program| program.eq_ignore_ascii_case("spl-token"))
    {
        "SPL Token".to_string()
    } else if programs
        .iter()
        .any(|program| program.eq_ignore_ascii_case("system"))
    {
        "System Program".to_string()
    } else if let Some(program) = programs.first() {
        program.clone()
    } else {
        "Solana".to_string()
    }
}

fn summarize_transaction(
    changes: &[WalletTransactionChange],
    programs: &[String],
) -> (String, String) {
    let has_in = changes.iter().any(|change| change.direction == "in");
    let has_out = changes.iter().any(|change| change.direction == "out");
    if has_in && has_out {
        ("swap".to_string(), fallback_program_label(programs))
    } else if has_in {
        ("receive".to_string(), "Received".to_string())
    } else if has_out {
        ("send".to_string(), "Sent".to_string())
    } else if !programs.is_empty() {
        ("contract".to_string(), fallback_program_label(programs))
    } else {
        ("transaction".to_string(), "Transaction".to_string())
    }
}

async fn enrich_transaction_change_assets(
    client: &RpcClient,
    network: &str,
    transactions: &mut [WalletTransactionRecord],
) {
    let mut mints = Vec::new();
    let mut seen_mints = HashMap::new();
    for transaction in transactions.iter() {
        for change in transaction.changes.iter() {
            let Some(mint) = change.mint.as_deref() else {
                continue;
            };
            if mint == WRAPPED_SOL_MINT || !seen_mints.contains_key(mint) {
                seen_mints.insert(mint.to_string(), true);
                mints.push(mint.to_string());
            }
        }
    }
    if mints.is_empty() {
        return;
    }

    let mut metadata_by_mint = HashMap::new();
    let missing_mints = merge_cached_token_metadata(network, &mut metadata_by_mint, &mints);
    load_missing_token_metadata(client, network, &mut metadata_by_mint, &missing_mints).await;
    if metadata_by_mint.is_empty() {
        return;
    }

    for transaction in transactions.iter_mut() {
        for change in transaction.changes.iter_mut() {
            let Some(mint) = change.mint.as_ref() else {
                continue;
            };
            change.asset = token_symbol_from_metadata(mint, metadata_by_mint.get(mint));
        }
    }
}

fn parse_wallet_transaction_details(
    client: &RpcClient,
    signature: &Signature,
    owner: &Pubkey,
    token_metadata: &HashMap<String, TokenMetadata>,
) -> (Vec<WalletTransactionChange>, Option<String>, Vec<String>) {
    let tx = match client.get_transaction_with_config(
        signature,
        RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        },
    ) {
        Ok(tx) => tx,
        Err(_) => return (Vec::new(), None, Vec::new()),
    };
    let (accounts, programs) = parsed_message_accounts_and_programs(&tx.transaction.transaction);
    let counterparty = transaction_counterparty(&accounts, &owner.to_string());
    let Some(meta) = tx.transaction.meta else {
        return (Vec::new(), counterparty, programs);
    };

    let mut changes = Vec::new();
    if let Some(owner_index) = accounts
        .iter()
        .position(|account| account == &owner.to_string())
    {
        if let (Some(pre), Some(post)) = (
            meta.pre_balances.get(owner_index),
            meta.post_balances.get(owner_index),
        ) {
            add_change(&mut changes, "SOL", None, *post as i128 - *pre as i128, 9);
        }
    }

    let mut token_deltas: HashMap<String, (i128, u8)> = HashMap::new();
    for token_balance in option_serializer_vec(&meta.pre_token_balances) {
        let is_owner = match token_balance.owner.as_ref() {
            OptionSerializer::Some(token_owner) => token_owner.as_str() == owner.to_string(),
            _ => false,
        };
        if !is_owner {
            continue;
        }
        let amount = token_balance
            .ui_token_amount
            .amount
            .parse::<i128>()
            .unwrap_or_default();
        let entry = token_deltas
            .entry(token_balance.mint.clone())
            .or_insert((0, token_balance.ui_token_amount.decimals));
        entry.0 -= amount;
        entry.1 = token_balance.ui_token_amount.decimals;
    }
    for token_balance in option_serializer_vec(&meta.post_token_balances) {
        let is_owner = match token_balance.owner.as_ref() {
            OptionSerializer::Some(token_owner) => token_owner.as_str() == owner.to_string(),
            _ => false,
        };
        if !is_owner {
            continue;
        }
        let amount = token_balance
            .ui_token_amount
            .amount
            .parse::<i128>()
            .unwrap_or_default();
        let entry = token_deltas
            .entry(token_balance.mint.clone())
            .or_insert((0, token_balance.ui_token_amount.decimals));
        entry.0 += amount;
        entry.1 = token_balance.ui_token_amount.decimals;
    }
    for (mint, (delta, decimals)) in token_deltas {
        let asset = token_symbol_from_metadata(&mint, token_metadata.get(&mint));
        add_change(&mut changes, &asset, Some(mint.clone()), delta, decimals);
    }

    (changes, counterparty, programs)
}

fn parse_wallet_transaction_details_with_timeout(
    network: Option<&str>,
    signature: Signature,
    owner: Pubkey,
    token_metadata: HashMap<String, TokenMetadata>,
) -> (Vec<WalletTransactionChange>, Option<String>, Vec<String>) {
    let Ok((client, _)) =
        rpc_query_client_for_timeout(network, RPC_TRANSACTION_DETAIL_TIMEOUT_SECS)
    else {
        return (Vec::new(), None, Vec::new());
    };
    parse_wallet_transaction_details(&client, &signature, &owner, &token_metadata)
}

#[derive(Serialize)]
struct WalletTransactionsResponse {
    address: String,
    network: String,
    limit: usize,
    before: Option<String>,
    next_before: Option<String>,
    has_more: bool,
    transactions: Vec<WalletTransactionRecord>,
}

async fn get_wallet_transactions(
    Json(req): Json<WalletTransactionsRequest>,
) -> Result<Json<WalletTransactionsResponse>, ApiError> {
    let pubkey = Pubkey::from_str(&req.address).map_err(|_| ApiError {
        message: "无效的地址".to_string(),
    })?;
    let limit = req
        .limit
        .unwrap_or(20)
        .clamp(1, MAX_WALLET_TRANSACTION_HISTORY);
    let before = req
        .before
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let before_signature = before
        .as_deref()
        .map(Signature::from_str)
        .transpose()
        .map_err(|_| ApiError {
            message: "无效的分页交易签名".to_string(),
        })?;
    let selector = rpc_selector(req.network.as_deref())?;
    let network = selector.network.clone();
    let mut errors = Vec::new();
    let mut signatures_with_client = None;
    for rpc_url in fallback_rpc_urls(&selector) {
        let client = rpc_query_client_for_url(rpc_url.clone(), RPC_QUERY_TIMEOUT_SECS);
        match client.get_signatures_for_address_with_config(
            &pubkey,
            GetConfirmedSignaturesForAddress2Config {
                before: before_signature,
                until: None,
                limit: Some(limit),
                commitment: Some(CommitmentConfig::confirmed()),
            },
        ) {
            Ok(signatures) => {
                signatures_with_client = Some((signatures, client));
                break;
            }
            Err(error) => {
                let message = format!("查询交易记录失败: {}", error);
                if !is_retryable_rpc_message(&message) {
                    return Err(ApiError { message });
                }
                tracing::warn!(
                    "Wallet transaction list RPC read failed at {}: {}",
                    rpc_url,
                    message
                );
                errors.push((rpc_url, message));
            }
        }
    }
    let Some((signatures, client)) = signatures_with_client else {
        return Err(ApiError {
            message: format!(
                "所有 RPC 节点均无法查询交易记录: {}",
                rpc_failure_summary(&errors)
            ),
        });
    };

    let has_more = signatures.len() >= limit;
    let request_network = req.network.clone();
    let cached_token_metadata = wallet_store::get_wallet_assets(&req.address, &network)
        .ok()
        .flatten()
        .map(|cached| cached_token_metadata_by_mint(&cached.tokens))
        .unwrap_or_default();
    let pending_transactions = signatures
        .into_iter()
        .take(MAX_WALLET_TRANSACTION_HISTORY)
        .map(|item| {
            let details = Signature::from_str(&item.signature).ok().map(|sig| {
                let owner = pubkey;
                let network = request_network.clone();
                let token_metadata = cached_token_metadata.clone();
                tokio::task::spawn_blocking(move || {
                    parse_wallet_transaction_details_with_timeout(
                        network.as_deref(),
                        sig,
                        owner,
                        token_metadata,
                    )
                })
            });
            (item, details)
        })
        .collect::<Vec<_>>();

    let mut transactions = join_all(pending_transactions.into_iter().map(
        |(item, details)| async move {
            let signature = item.signature.clone();
            let (changes, counterparty, programs) = match details {
                Some(details) => tokio::time::timeout(
                    Duration::from_secs(RPC_TRANSACTION_DETAIL_TIMEOUT_SECS + 1),
                    details,
                )
                .await
                .ok()
                .and_then(Result::ok)
                .unwrap_or_else(|| (Vec::new(), None, Vec::new())),
                None => (Vec::new(), None, Vec::new()),
            };
            let (action, summary) = summarize_transaction(&changes, &programs);
            WalletTransactionRecord {
                signature,
                slot: item.slot,
                block_time: item.block_time,
                confirmation_status: item
                    .confirmation_status
                    .map(|status| format!("{:?}", status)),
                err: item.err.map(|err| json!(err)),
                memo: item.memo,
                action,
                summary,
                counterparty,
                programs,
                changes,
            }
        },
    ))
    .await;
    enrich_transaction_change_assets(&client, &network, &mut transactions).await;
    let next_before = transactions.last().map(|item| item.signature.clone());

    Ok(Json(WalletTransactionsResponse {
        address: req.address,
        network,
        limit,
        before,
        next_before,
        has_more,
        transactions,
    }))
}

// Get Public Key from Secret Key
#[derive(Deserialize)]
struct GetPubkeyRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
}
#[derive(Serialize)]
struct GetPubkeyResponse {
    public_key: String,
}

async fn get_pubkey(
    Json(req): Json<GetPubkeyRequest>,
) -> Result<Json<GetPubkeyResponse>, ApiError> {
    Ok(Json(GetPubkeyResponse {
        public_key: req.wallet.public_key()?,
    }))
}

// ============= SOL Operations (8) =============

// 8. Transfer SOL
#[derive(Deserialize)]
struct TransferSolRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    to_address: String,
    amount: DecimalAmount,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct TransferSolResponse {
    signature: String,
    status: String,
}

async fn transfer_sol(
    Json(req): Json<TransferSolRequest>,
) -> Result<Json<TransferSolResponse>, ApiError> {
    let to_pubkey = Pubkey::from_str(&req.to_address).map_err(|_| ApiError {
        message: "无效的接收地址".to_string(),
    })?;

    let keypair = req.wallet.keypair()?;

    let amount_lamports = sol_to_lamports(&req.amount)?;
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let client = SolanaClient::new(rpc_url.to_string());

    let signature = client
        .transfer_sol(&keypair, &to_pubkey, amount_lamports)
        .map_err(|e| ApiError {
            message: format!("转账失败: {}", e),
        })?;

    Ok(Json(TransferSolResponse {
        signature: signature.to_string(),
        status: "success".to_string(),
    }))
}

// ============= WSOL Operations (9-11) =============

// 9. Create WSOL ATA
#[derive(Deserialize)]
struct CreateWsolAtaRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct CreateWsolAtaResponse {
    signature: String,
    status: String,
}

async fn create_wsol_ata(
    Json(req): Json<CreateWsolAtaRequest>,
) -> Result<Json<CreateWsolAtaResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;
    let mint = Pubkey::from_str(WRAPPED_SOL_MINT).map_err(|_| ApiError {
        message: "无效的 WSOL mint 地址".to_string(),
    })?;
    let (client, _) = rpc_client_for(req.network.as_deref())?;

    let instruction = squads_v4::create_associated_token_account_idempotent_ix(
        &keypair.pubkey(),
        &keypair.pubkey(),
        &mint,
    );
    let signature = sign_and_send_single(&client, instruction, &keypair)?;

    Ok(Json(CreateWsolAtaResponse {
        signature,
        status: "success".to_string(),
    }))
}

// 10. Wrap SOL
#[derive(Deserialize)]
struct WrapSolRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    amount: DecimalAmount,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct WrapSolResponse {
    signature: String,
    status: String,
}

async fn wrap_sol(Json(req): Json<WrapSolRequest>) -> Result<Json<WrapSolResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;

    let amount_lamports = sol_to_lamports(&req.amount)?;
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let client = SolanaClient::new(rpc_url.to_string());

    let signature = client
        .wrap_sol(&keypair, amount_lamports)
        .map_err(|e| ApiError {
            message: format!("封装失败: {}", e),
        })?;

    Ok(Json(WrapSolResponse {
        signature: signature.to_string(),
        status: "success".to_string(),
    }))
}

// 11. Unwrap SOL
#[derive(Deserialize)]
struct UnwrapSolRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct UnwrapSolResponse {
    signature: String,
    status: String,
}

async fn unwrap_sol(
    Json(req): Json<UnwrapSolRequest>,
) -> Result<Json<UnwrapSolResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;

    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let client = SolanaClient::new(rpc_url.to_string());

    let signature = client.unwrap_sol(&keypair).map_err(|e| ApiError {
        message: format!("解封失败: {}", e),
    })?;

    Ok(Json(UnwrapSolResponse {
        signature: signature.to_string(),
        status: "success".to_string(),
    }))
}

// 12. Close WSOL ATA
#[derive(Deserialize)]
struct CloseWsolAtaRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct CloseWsolAtaResponse {
    signature: String,
    status: String,
}

async fn close_wsol_ata(
    Json(req): Json<CloseWsolAtaRequest>,
) -> Result<Json<CloseWsolAtaResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;

    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let client = SolanaClient::new(rpc_url.to_string());

    // Close WSOL ATA by unwrapping all WSOL
    let signature = client.unwrap_sol(&keypair).map_err(|e| ApiError {
        message: format!("关闭 ATA 失败: {}", e),
    })?;

    Ok(Json(CloseWsolAtaResponse {
        signature: signature.to_string(),
        status: "success".to_string(),
    }))
}

// ============= Token Operations (13) =============

#[derive(Deserialize)]
struct TokenMintInfoRequest {
    mint: String,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct TokenMintInfoResponse {
    mint: String,
    network: String,
    decimals: u8,
    supply: String,
    ui_amount_string: String,
}

async fn token_mint_info(
    Json(req): Json<TokenMintInfoRequest>,
) -> Result<Json<TokenMintInfoResponse>, ApiError> {
    let mint = Pubkey::from_str(req.mint.trim()).map_err(|_| ApiError {
        message: "无效的 Token Mint 地址".to_string(),
    })?;
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let supply = token_mint_info_from_rpc(&client, &mint)?;
    let ui_amount_string = supply.real_number_string_trimmed();

    Ok(Json(TokenMintInfoResponse {
        mint: mint.to_string(),
        network,
        decimals: supply.decimals,
        supply: supply.amount,
        ui_amount_string,
    }))
}

// 13. Transfer SPL Token
#[derive(Deserialize)]
struct TransferTokenRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    to_address: String,
    mint: String,
    amount: DecimalAmount,
    #[allow(dead_code)]
    #[serde(default)]
    decimals: Option<u8>,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct TransferTokenResponse {
    signature: String,
    status: String,
}

async fn transfer_token(
    Json(req): Json<TransferTokenRequest>,
) -> Result<Json<TransferTokenResponse>, ApiError> {
    let to_pubkey = Pubkey::from_str(&req.to_address).map_err(|_| ApiError {
        message: "无效的接收地址".to_string(),
    })?;
    let mint = Pubkey::from_str(&req.mint).map_err(|_| ApiError {
        message: "无效的mint地址".to_string(),
    })?;

    let keypair = req.wallet.keypair()?;

    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let client = SolanaClient::new(rpc_url.to_string());

    // Always trust on-chain mint decimals instead of user-submitted decimals.
    let mint_info = token_mint_info_from_rpc(&rpc_client, &mint)?;
    let token_amount = token_amount_to_raw(&req.amount, mint_info.decimals)?;

    let signature = client
        .transfer_token(&keypair, &to_pubkey, &mint, token_amount)
        .map_err(|e| ApiError {
            message: format!("转账失败: {}", e),
        })?;

    Ok(Json(TransferTokenResponse {
        signature: signature.to_string(),
        status: "success".to_string(),
    }))
}

// ============= Nonce Operations (14) =============

// 14. Create Nonce Account
#[derive(Deserialize)]
struct CreateNonceAccountRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    count: Option<u8>,
}
#[derive(Serialize)]
struct CreatedNonceAccount {
    nonce_account: String,
    signature: String,
    created_at: u64,
}
#[derive(Serialize)]
struct CreateNonceAccountResponse {
    nonce_accounts: Vec<CreatedNonceAccount>,
    nonce_account: String,
    signature: String,
    count: usize,
    status: String,
}

async fn create_nonce_account(
    Json(req): Json<CreateNonceAccountRequest>,
) -> Result<Json<CreateNonceAccountResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;
    let owner = keypair.pubkey().to_string();
    let wallet_id = req
        .wallet
        .wallet_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let count = req.count.unwrap_or(1);
    if count == 0 || count > MAX_NONCE_BATCH_COUNT {
        return Err(ApiError {
            message: format!("Nonce 创建数量必须在 1-{} 之间", MAX_NONCE_BATCH_COUNT),
        });
    }

    let selector = rpc_selector(req.network.as_deref())?;
    let rpc_url = selector.url;
    let network = selector.network;
    let client = SolanaClient::new(rpc_url.to_string());

    let mut nonce_accounts = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (nonce_account, signature) =
            client
                .create_nonce_account(&keypair)
                .map_err(|e| ApiError {
                    message: format!("创建失败: {}", e),
                })?;
        let record = wallet_store::add_nonce_account(
            wallet_id,
            &owner,
            &network,
            &nonce_account.to_string(),
            &signature.to_string(),
        )
        .map_err(|message| ApiError { message })?;
        nonce_accounts.push(CreatedNonceAccount {
            nonce_account: record.nonce_account,
            signature: record.signature,
            created_at: record.created_at,
        });
    }
    let first = nonce_accounts.first().ok_or_else(|| ApiError {
        message: "未创建 Nonce 账户".to_string(),
    })?;

    Ok(Json(CreateNonceAccountResponse {
        nonce_account: first.nonce_account.clone(),
        signature: first.signature.clone(),
        count: nonce_accounts.len(),
        nonce_accounts,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct ListNonceAccountsRequest {
    owner: String,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct ListNonceAccountsResponse {
    owner: String,
    network: String,
    nonce_accounts: Vec<wallet_store::NonceAccountRecord>,
}

async fn list_nonce_accounts(
    Json(req): Json<ListNonceAccountsRequest>,
) -> Result<Json<ListNonceAccountsResponse>, ApiError> {
    let owner = Pubkey::from_str(&req.owner).map_err(|_| ApiError {
        message: "无效的钱包地址".to_string(),
    })?;
    let network = network_name(req.network.as_deref()).to_string();
    let nonce_accounts = wallet_store::list_nonce_accounts(&owner.to_string(), &network)
        .map_err(|message| ApiError { message })?;
    Ok(Json(ListNonceAccountsResponse {
        owner: owner.to_string(),
        network,
        nonce_accounts,
    }))
}

// ============= Program Deployment =============

const DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED: &str = "create_buffer_signed";
const DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE: &str = "create_buffer_requires_reconciliation";
const DEPLOYMENT_STATUS_BUFFER_READY: &str = "buffer_ready";
const DEPLOYMENT_STATUS_WRITE_SIGNED: &str = "write_signed";
const DEPLOYMENT_STATUS_WRITE_CONFIRMED: &str = "write_confirmed";
const DEPLOYMENT_STATUS_WRITE_RECONCILE: &str = "write_requires_reconciliation";
const DEPLOYMENT_STATUS_BUFFER_FINALIZED: &str = "buffer_finalized";
const DEPLOYMENT_STATUS_DEPLOY_SIGNED: &str = "deploy_signed";
const DEPLOYMENT_STATUS_DEPLOY_RECONCILE: &str = "deploy_requires_reconciliation";
const DEPLOYMENT_STATUS_DEPLOY_FINALIZED: &str = "deploy_finalized_pending_readback";
const DEPLOYMENT_STATUS_FINALIZED: &str = "finalized";

#[derive(Debug, PartialEq, Eq)]
enum DeploymentAttemptExpiryDecision {
    NotExpired,
    ExpiredAbsent {
        min_context_slot: u64,
    },
    HistoryContextTooOld {
        expiry_observation_slot: u64,
        history_context_slot: u64,
    },
    SeenButNotFinalized,
    FinalizedSucceeded,
    FinalizedFailed(String),
}

fn classify_deployment_attempt_expiry(
    finalized_block_height: u64,
    last_valid_block_height: u64,
    historical_status: Option<&TransactionStatus>,
    history_context_slot: u64,
    expiry_observation_slot: u64,
) -> DeploymentAttemptExpiryDecision {
    if finalized_block_height <= last_valid_block_height {
        return DeploymentAttemptExpiryDecision::NotExpired;
    }
    if history_context_slot < expiry_observation_slot {
        return DeploymentAttemptExpiryDecision::HistoryContextTooOld {
            expiry_observation_slot,
            history_context_slot,
        };
    }
    let Some(status) = historical_status else {
        return DeploymentAttemptExpiryDecision::ExpiredAbsent {
            min_context_slot: history_context_slot,
        };
    };
    if !status.satisfies_commitment(CommitmentConfig::finalized()) {
        return DeploymentAttemptExpiryDecision::SeenButNotFinalized;
    }
    match status.err.as_ref() {
        Some(error) => DeploymentAttemptExpiryDecision::FinalizedFailed(format!("{error:?}")),
        None => DeploymentAttemptExpiryDecision::FinalizedSucceeded,
    }
}

fn inspect_deployment_attempt_expiry(
    client: &RpcClient,
    attempt: &wallet_store::ProgramDeploymentAttemptRecord,
) -> Result<DeploymentAttemptExpiryDecision, ApiError> {
    let finalized_epoch_info = client
        .get_epoch_info_with_commitment(CommitmentConfig::finalized())
        .map_err(|error| {
            deployment_journal_error(format!(
                "读取 finalized epoch info 失败，无法判定签名 {} 是否过期: {error}",
                attempt.signature
            ))
        })?;
    let signature = Signature::from_str(attempt.signature.trim()).map_err(|_| {
        deployment_journal_error(format!(
            "attempt {} 包含无效签名，拒绝自动恢复",
            attempt.signature
        ))
    })?;
    if finalized_epoch_info.block_height <= attempt.last_valid_block_height {
        return Ok(DeploymentAttemptExpiryDecision::NotExpired);
    }
    let response = client
        .get_signature_statuses_with_history(&[signature])
        .map_err(|error| {
            deployment_journal_error(format!(
                "查询签名 {} 的完整历史失败，拒绝自动重试: {error}",
                attempt.signature
            ))
        })?;
    if response.value.len() != 1 {
        return Err(deployment_journal_error(format!(
            "签名 {} 的历史响应数量异常，拒绝自动重试",
            attempt.signature
        )));
    }
    Ok(classify_deployment_attempt_expiry(
        finalized_epoch_info.block_height,
        attempt.last_valid_block_height,
        response.value.first().and_then(Option::as_ref),
        response.context.slot,
        finalized_epoch_info.absolute_slot,
    ))
}

fn gate_expired_absent_with<F>(
    decision: DeploymentAttemptExpiryDecision,
    confirm_absent: F,
) -> Result<DeploymentAttemptExpiryDecision, ApiError>
where
    F: FnOnce(u64) -> Result<(), ApiError>,
{
    match decision {
        DeploymentAttemptExpiryDecision::ExpiredAbsent { min_context_slot } => {
            confirm_absent(min_context_slot)?;
            Ok(DeploymentAttemptExpiryDecision::ExpiredAbsent { min_context_slot })
        }
        other => Ok(other),
    }
}

fn finalized_recovery_account_config(min_context_slot: u64) -> RpcAccountInfoConfig {
    RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64Zstd),
        commitment: Some(CommitmentConfig::finalized()),
        data_slice: None,
        min_context_slot: Some(min_context_slot),
    }
}

fn decode_recovery_ui_account(address: &Pubkey, account: UiAccount) -> Result<Account, ApiError> {
    account.decode().ok_or_else(|| {
        deployment_journal_error(format!(
            "finalized 账户 {} 的 Base64Zstd 数据无法解码；拒绝将其视为不存在",
            address
        ))
    })
}

fn get_finalized_recovery_account(
    client: &RpcClient,
    address: &Pubkey,
    min_context_slot: u64,
) -> Result<Option<Account>, ApiError> {
    let response = client
        .get_ui_account_with_config(address, finalized_recovery_account_config(min_context_slot))
        .map_err(|error| {
            deployment_journal_error(format!(
                "在最小上下文 slot {} 二次读取 finalized 账户 {} 失败: {error}",
                min_context_slot, address
            ))
        })?;
    if response.context.slot < min_context_slot {
        return Err(deployment_journal_error(format!(
            "finalized 账户 {} 二次回读 slot {} 早于签名历史 slot {}",
            address, response.context.slot, min_context_slot
        )));
    }
    response
        .value
        .map(|account| decode_recovery_ui_account(address, account))
        .transpose()
}

fn get_finalized_recovery_accounts(
    client: &RpcClient,
    addresses: &[Pubkey],
    min_context_slot: u64,
) -> Result<Vec<Option<Account>>, ApiError> {
    let response = client
        .get_multiple_ui_accounts_with_config(
            addresses,
            finalized_recovery_account_config(min_context_slot),
        )
        .map_err(|error| {
            deployment_journal_error(format!(
                "在最小上下文 slot {} 二次读取 finalized 账户组失败: {error}",
                min_context_slot
            ))
        })?;
    if response.context.slot < min_context_slot {
        return Err(deployment_journal_error(format!(
            "finalized 账户组二次回读 slot {} 早于签名历史 slot {}",
            response.context.slot, min_context_slot
        )));
    }
    if response.value.len() != addresses.len() {
        return Err(deployment_journal_error(format!(
            "finalized 账户组二次回读返回 {} 项，预期 {} 项",
            response.value.len(),
            addresses.len()
        )));
    }
    response
        .value
        .into_iter()
        .zip(addresses)
        .map(|(account, address)| {
            account
                .map(|account| decode_recovery_ui_account(address, account))
                .transpose()
        })
        .collect()
}

fn confirm_create_buffer_attempt_still_absent(
    client: &RpcClient,
    buffer_address: &Pubkey,
    min_context_slot: u64,
) -> Result<(), ApiError> {
    if get_finalized_recovery_account(client, buffer_address, min_context_slot)?.is_some() {
        return Err(deployment_journal_error(format!(
            "Buffer {} 已在签名历史 slot {} 之后出现；保持 attempt 活动状态并中止本次恢复",
            buffer_address, min_context_slot
        )));
    }
    Ok(())
}

fn validate_write_attempt_recovery_snapshot(
    previous_plan: &program_deploy::BufferWritePlan,
    current_plan: &program_deploy::BufferWritePlan,
    chunk_index: usize,
) -> Result<(), ApiError> {
    if previous_plan != current_plan {
        return Err(deployment_journal_error(format!(
            "Buffer finalized 写入计划在 chunk {} 过期判定后已变化；保持 attempt 活动状态并中止本次恢复",
            chunk_index
        )));
    }
    if !current_plan.pending_chunk_indexes.contains(&chunk_index) {
        return Err(deployment_journal_error(format!(
            "Buffer chunk {} 已不再缺失；保持 attempt 活动状态并中止本次恢复",
            chunk_index
        )));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn confirm_write_attempt_still_absent(
    client: &RpcClient,
    buffer_address: &Pubkey,
    expected_authority: &Pubkey,
    expected_program: &[u8],
    previous_plan: &program_deploy::BufferWritePlan,
    chunk_index: usize,
    min_context_slot: u64,
) -> Result<(), ApiError> {
    let account = get_finalized_recovery_account(client, buffer_address, min_context_slot)?
        .ok_or_else(|| {
            deployment_journal_error(format!(
                "Buffer {} 在 write attempt 二次回读时不存在；拒绝自动重签 chunk {}",
                buffer_address, chunk_index
            ))
        })?;
    let current_plan = program_deploy::verify_resume_buffer(
        expected_authority,
        expected_program,
        PROGRAM_WRITE_CHUNK_BYTES,
        &account.owner,
        account.executable,
        &account.data,
    )
    .map_err(|error| {
        deployment_journal_error(format!(
            "Buffer {} 在 write attempt 二次回读时校验失败: {error}",
            buffer_address
        ))
    })?;
    validate_write_attempt_recovery_snapshot(previous_plan, &current_plan, chunk_index)
}

fn confirm_deploy_attempt_still_absent(
    client: &RpcClient,
    program_id: &Pubkey,
    programdata_address: &Pubkey,
    min_context_slot: u64,
) -> Result<(), ApiError> {
    let accounts = get_finalized_recovery_accounts(
        client,
        &[*program_id, *programdata_address],
        min_context_slot,
    )?;
    if accounts.iter().any(Option::is_some) {
        return Err(deployment_journal_error(format!(
            "Program {} 或 ProgramData {} 已在签名历史 slot {} 之后出现；保持 attempt 活动状态并中止本次恢复",
            program_id, programdata_address, min_context_slot
        )));
    }
    Ok(())
}

fn deployment_attempt_is_active(status: &str) -> bool {
    matches!(
        status,
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED
            | wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED
            | wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_REQUIRES_RECONCILIATION
    )
}

fn deployment_attempt_is_blocking_terminal(status: &str) -> bool {
    matches!(
        status,
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED
            | wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED_FAILED
    )
}

fn new_program_deployment_attempt(
    record: &wallet_store::ProgramDeploymentRecord,
    stage: &str,
    chunk_index: Option<usize>,
    signature: &Signature,
    last_valid_block_height: u64,
) -> wallet_store::ProgramDeploymentAttemptRecord {
    wallet_store::ProgramDeploymentAttemptRecord {
        genesis_hash: record.genesis_hash.clone(),
        program_id: record.program_id.clone(),
        stage: stage.to_string(),
        buffer_address: record.buffer_address.clone(),
        chunk_index,
        signature: signature.to_string(),
        last_valid_block_height,
        status: wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED.to_string(),
        created_at: 0,
        updated_at: 0,
    }
}

fn validate_program_deployment_attempt_record(
    record: &wallet_store::ProgramDeploymentRecord,
    attempt: &wallet_store::ProgramDeploymentAttemptRecord,
) -> Result<(), ApiError> {
    let shape_is_valid = match attempt.stage.as_str() {
        wallet_store::PROGRAM_DEPLOYMENT_STAGE_CREATE_BUFFER
        | wallet_store::PROGRAM_DEPLOYMENT_STAGE_DEPLOY => attempt.chunk_index.is_none(),
        wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE => attempt.chunk_index.is_some(),
        _ => false,
    };
    let status_is_valid = deployment_attempt_is_active(&attempt.status)
        || deployment_attempt_is_blocking_terminal(&attempt.status)
        || attempt.status == wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EXPIRED_ABSENT;
    let signature_is_valid = Signature::from_str(&attempt.signature).is_ok_and(|signature| {
        signature != Signature::default() && signature.to_string() == attempt.signature
    });
    let buffer_is_valid = Pubkey::from_str(&attempt.buffer_address).is_ok_and(|buffer| {
        buffer != Pubkey::default() && buffer.to_string() == attempt.buffer_address
    });
    if attempt.genesis_hash != record.genesis_hash
        || attempt.program_id != record.program_id
        || !shape_is_valid
        || !status_is_valid
        || (attempt.buffer_address != record.buffer_address
            && !(attempt.status == wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EXPIRED_ABSENT
                && attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_CREATE_BUFFER))
        || !signature_is_valid
        || !buffer_is_valid
        || attempt.last_valid_block_height == 0
        || attempt.created_at == 0
        || attempt.updated_at < attempt.created_at
    {
        return Err(deployment_journal_error(format!(
            "Program {} 包含无效或不一致的 attempt 证据 {}，拒绝继续",
            record.program_id, attempt.signature
        )));
    }
    Ok(())
}

fn begin_deployment_attempt(
    journal: &mut Option<wallet_store::ProgramDeploymentRecord>,
    attempt: wallet_store::ProgramDeploymentAttemptRecord,
    journal_status: &str,
    completed_writes: usize,
) -> Result<(), ApiError> {
    let current = journal.as_ref().ok_or_else(|| ApiError {
        message: "部署 journal 不存在，拒绝记录签名 attempt".to_string(),
    })?;
    let updated = wallet_store::begin_program_deployment_attempt(
        current,
        attempt,
        journal_status,
        completed_writes,
    )
    .map_err(|message| ApiError { message })?;
    *journal = Some(updated);
    Ok(())
}

fn transition_deployment_attempt(
    journal: &mut Option<wallet_store::ProgramDeploymentRecord>,
    signature: &str,
    expected_attempt_status: &str,
    next_attempt_status: &str,
    journal_status: &str,
    completed_writes: usize,
) -> Result<(), ApiError> {
    let current = journal.as_ref().ok_or_else(|| ApiError {
        message: "部署 journal 不存在，拒绝迁移签名 attempt".to_string(),
    })?;
    let updated = wallet_store::transition_program_deployment_attempt(
        current,
        signature,
        expected_attempt_status,
        next_attempt_status,
        journal_status,
        completed_writes,
    )
    .map_err(|message| ApiError { message })?;
    *journal = Some(updated);
    Ok(())
}

fn reconcile_absent_deployment_attempt<F>(
    client: &RpcClient,
    journal: &mut Option<wallet_store::ProgramDeploymentRecord>,
    attempt: &wallet_store::ProgramDeploymentAttemptRecord,
    journal_status: &str,
    completed_writes: usize,
    confirm_absent: F,
) -> Result<bool, ApiError>
where
    F: FnOnce(&RpcClient, u64) -> Result<(), ApiError>,
{
    let stage_context = attempt
        .chunk_index
        .map(|index| format!("{} chunk {}", attempt.stage, index))
        .unwrap_or_else(|| attempt.stage.clone());
    let decision = gate_expired_absent_with(
        inspect_deployment_attempt_expiry(client, attempt)?,
        |min_context_slot| confirm_absent(client, min_context_slot),
    )?;
    match decision {
        DeploymentAttemptExpiryDecision::ExpiredAbsent { .. } => {
            transition_deployment_attempt(
                journal,
                &attempt.signature,
                &attempt.status,
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EXPIRED_ABSENT,
                journal_status,
                completed_writes,
            )?;
            Ok(true)
        }
        DeploymentAttemptExpiryDecision::FinalizedSucceeded => {
            transition_deployment_attempt(
                journal,
                &attempt.signature,
                &attempt.status,
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
                journal_status,
                completed_writes,
            )?;
            Err(deployment_journal_error(format!(
                "{} 签名 {} 已 finalized 成功，但对应业务状态缺失；已持久化冲突并禁止自动重签",
                stage_context, attempt.signature
            )))
        }
        DeploymentAttemptExpiryDecision::FinalizedFailed(error) => {
            transition_deployment_attempt(
                journal,
                &attempt.signature,
                &attempt.status,
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED_FAILED,
                journal_status,
                completed_writes,
            )?;
            Err(deployment_journal_error(format!(
                "{} 签名 {} 已 finalized 失败（{}）；已持久化终态，禁止自动重签",
                stage_context, attempt.signature, error
            )))
        }
        DeploymentAttemptExpiryDecision::NotExpired => Err(deployment_journal_error(format!(
            "{} 签名 {} 尚未越过 last valid block height {}，不得自动重签",
            stage_context, attempt.signature, attempt.last_valid_block_height
        ))),
        DeploymentAttemptExpiryDecision::HistoryContextTooOld {
            expiry_observation_slot,
            history_context_slot,
        } => Err(deployment_journal_error(format!(
            "{} 签名 {} 的历史上下文 slot {} 早于过期观测 slot {}；RPC 后端视图不一致，拒绝自动重签",
            stage_context, attempt.signature, history_context_slot, expiry_observation_slot
        ))),
        DeploymentAttemptExpiryDecision::SeenButNotFinalized => {
            Err(deployment_journal_error(format!(
                "{} 签名 {} 仍可在历史中观察到但尚未 finalized，不得自动重签",
                stage_context, attempt.signature
            )))
        }
    }
}

fn deployment_journal_error(message: impl Into<String>) -> ApiError {
    ApiError {
        message: format!("部署 journal 冲突: {}", message.into()),
    }
}

type DeploymentReceiptEvidence = (
    Option<u64>,
    Option<u32>,
    Vec<wallet_store::ProgramDeploymentAttemptRecord>,
);

fn load_deployment_receipt_evidence(
    journal: Option<&wallet_store::ProgramDeploymentRecord>,
) -> Result<DeploymentReceiptEvidence, ApiError> {
    let Some(record) = journal else {
        return Ok((None, None, Vec::new()));
    };
    let attempts =
        wallet_store::load_program_deployment_attempts(&record.genesis_hash, &record.program_id)
            .map_err(|message| ApiError { message })?;
    for attempt in &attempts {
        validate_program_deployment_attempt_record(record, attempt)?;
    }
    Ok((
        Some(record.revision),
        Some(record.attempt_evidence_version),
        attempts,
    ))
}

#[allow(clippy::too_many_arguments)]
fn validate_deployment_journal_binding(
    record: &wallet_store::ProgramDeploymentRecord,
    genesis_hash: &str,
    program_id: &Pubkey,
    program_sha256: &str,
    program_len: usize,
    max_data_len: usize,
    upgrade_authority: &Pubkey,
    requested_buffer: Option<&Pubkey>,
) -> Result<Pubkey, ApiError> {
    let expected_program_id = program_id.to_string();
    let expected_upgrade_authority = upgrade_authority.to_string();
    if record.genesis_hash != genesis_hash
        || record.program_id != expected_program_id
        || record.program_sha256 != program_sha256
        || record.program_len != program_len
        || record.max_data_len != max_data_len
        || record.upgrade_authority != expected_upgrade_authority
    {
        return Err(deployment_journal_error(format!(
            "Program {} 已有不同的部署意图，拒绝复用或覆盖；已记录 artifact={} len={} max_len={} upgrade_authority={}",
            program_id,
            record.program_sha256,
            record.program_len,
            record.max_data_len,
            record.upgrade_authority,
        )));
    }

    let stored_buffer = Pubkey::from_str(record.buffer_address.trim()).map_err(|_| {
        deployment_journal_error(format!(
            "Program {} 已记录无效 Buffer 地址 {}",
            program_id, record.buffer_address
        ))
    })?;
    if stored_buffer == Pubkey::default() || record.buffer_address != stored_buffer.to_string() {
        return Err(deployment_journal_error(format!(
            "Program {} 已记录非 canonical Buffer 地址 {}",
            program_id, record.buffer_address
        )));
    }
    if record.created_at == 0 || record.updated_at < record.created_at {
        return Err(deployment_journal_error(format!(
            "Program {} 的部署 journal 时间戳无效",
            program_id
        )));
    }
    if requested_buffer.is_some_and(|requested| requested != &stored_buffer) {
        return Err(deployment_journal_error(format!(
            "请求恢复的 Buffer 与已记录 Buffer {} 不一致",
            stored_buffer
        )));
    }
    Ok(stored_buffer)
}

fn deployment_status_is_known(status: &str) -> bool {
    matches!(
        status,
        DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED
            | DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE
            | DEPLOYMENT_STATUS_BUFFER_READY
            | DEPLOYMENT_STATUS_WRITE_SIGNED
            | DEPLOYMENT_STATUS_WRITE_CONFIRMED
            | DEPLOYMENT_STATUS_WRITE_RECONCILE
            | DEPLOYMENT_STATUS_BUFFER_FINALIZED
            | DEPLOYMENT_STATUS_DEPLOY_SIGNED
            | DEPLOYMENT_STATUS_DEPLOY_RECONCILE
            | DEPLOYMENT_STATUS_DEPLOY_FINALIZED
            | DEPLOYMENT_STATUS_FINALIZED
    )
}

#[allow(clippy::too_many_arguments)]
fn update_deployment_journal(
    record: &wallet_store::ProgramDeploymentRecord,
    status: &str,
    create_signature: Option<&str>,
    create_last_valid_block_height: Option<u64>,
    last_write_signature: Option<&str>,
    last_write_chunk_index: Option<usize>,
    last_write_last_valid_block_height: Option<u64>,
    completed_writes: usize,
    deploy_signature: Option<&str>,
    deploy_last_valid_block_height: Option<u64>,
) -> Result<wallet_store::ProgramDeploymentRecord, ApiError> {
    wallet_store::update_program_deployment_progress(
        record,
        status,
        create_signature,
        create_last_valid_block_height,
        last_write_signature,
        last_write_chunk_index,
        last_write_last_valid_block_height,
        completed_writes,
        deploy_signature,
        deploy_last_valid_block_height,
    )
    .map_err(|message| ApiError { message })
}

#[allow(clippy::too_many_arguments)]
fn transition_deployment_journal(
    journal: &mut Option<wallet_store::ProgramDeploymentRecord>,
    status: &str,
    create_signature: Option<&str>,
    create_last_valid_block_height: Option<u64>,
    last_write_signature: Option<&str>,
    last_write_chunk_index: Option<usize>,
    last_write_last_valid_block_height: Option<u64>,
    completed_writes: usize,
    deploy_signature: Option<&str>,
    deploy_last_valid_block_height: Option<u64>,
) -> Result<(), ApiError> {
    let current = journal.as_ref().ok_or_else(|| ApiError {
        message: "部署 journal 不存在，拒绝更新部署状态".to_string(),
    })?;
    let updated = update_deployment_journal(
        current,
        status,
        create_signature,
        create_last_valid_block_height,
        last_write_signature,
        last_write_chunk_index,
        last_write_last_valid_block_height,
        completed_writes,
        deploy_signature,
        deploy_last_valid_block_height,
    )?;
    *journal = Some(updated);
    Ok(())
}

#[cfg(test)]
mod deployment_journal_tests {
    use super::*;

    #[test]
    fn program_upgrade_requires_squads_member() {
        let signer = Pubkey::new_unique();
        let multisig = SquadsMultisig {
            create_key: Pubkey::new_unique(),
            config_authority: Pubkey::default(),
            threshold: 1,
            time_lock: 0,
            transaction_index: 0,
            stale_transaction_index: 0,
            rent_collector: None,
            bump: 0,
            members: vec![SquadsMember {
                key: signer,
                permissions: squads_v4::Permissions::all(),
            }],
        };
        assert!(require_squads_member(&multisig, &signer).is_ok());
        assert!(require_squads_member(&multisig, &Pubkey::new_unique()).is_err());
    }

    #[test]
    fn parses_programdata_upgrade_authority() {
        let authority = Pubkey::new_unique();
        let mut data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
            slot: 42,
            upgrade_authority_address: Some(authority),
        })
        .unwrap();
        data.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
        assert_eq!(
            loader_upgrade_authority_from_programdata(&data).unwrap(),
            authority
        );

        let mut immutable = bincode::serialize(&UpgradeableLoaderState::ProgramData {
            slot: 42,
            upgrade_authority_address: None,
        })
        .unwrap();
        immutable.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
        assert!(loader_upgrade_authority_from_programdata(&immutable).is_err());
    }

    fn record(
        program_id: &Pubkey,
        authority: &Pubkey,
        buffer: &Pubkey,
    ) -> wallet_store::ProgramDeploymentRecord {
        wallet_store::ProgramDeploymentRecord {
            genesis_hash: "devnet-genesis".to_string(),
            program_id: program_id.to_string(),
            program_sha256: "11".repeat(32),
            program_len: 1024,
            max_data_len: 2048,
            upgrade_authority: authority.to_string(),
            buffer_address: buffer.to_string(),
            status: DEPLOYMENT_STATUS_BUFFER_READY.to_string(),
            create_signature: Some(Signature::new_unique().to_string()),
            create_last_valid_block_height: Some(42),
            last_write_signature: None,
            last_write_chunk_index: None,
            last_write_last_valid_block_height: None,
            completed_writes: 0,
            deploy_signature: None,
            deploy_last_valid_block_height: None,
            attempt_evidence_version: wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EVIDENCE_VERSION,
            revision: 0,
            created_at: 1,
            updated_at: 1,
        }
    }

    #[test]
    fn journal_binding_accepts_only_the_recorded_deployment_intent() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let buffer = Pubkey::new_unique();
        let record = record(&program_id, &authority, &buffer);

        let stored_buffer = validate_deployment_journal_binding(
            &record,
            "devnet-genesis",
            &program_id,
            &"11".repeat(32),
            1024,
            2048,
            &authority,
            Some(&buffer),
        )
        .unwrap();
        assert_eq!(stored_buffer, buffer);

        assert!(validate_deployment_journal_binding(
            &record,
            "devnet-genesis",
            &program_id,
            &"22".repeat(32),
            1024,
            2048,
            &authority,
            Some(&buffer),
        )
        .is_err());
        assert!(validate_deployment_journal_binding(
            &record,
            "devnet-genesis",
            &program_id,
            &"11".repeat(32),
            1024,
            2048,
            &authority,
            Some(&Pubkey::new_unique()),
        )
        .is_err());
        let mut noncanonical_buffer = record.clone();
        noncanonical_buffer.buffer_address.push(' ');
        assert!(validate_deployment_journal_binding(
            &noncanonical_buffer,
            "devnet-genesis",
            &program_id,
            &"11".repeat(32),
            1024,
            2048,
            &authority,
            None,
        )
        .is_err());
        let mut zero_buffer = record.clone();
        zero_buffer.buffer_address = Pubkey::default().to_string();
        assert!(validate_deployment_journal_binding(
            &zero_buffer,
            "devnet-genesis",
            &program_id,
            &"11".repeat(32),
            1024,
            2048,
            &authority,
            None,
        )
        .is_err());
        let mut invalid_timestamps = record.clone();
        invalid_timestamps.updated_at = 0;
        assert!(validate_deployment_journal_binding(
            &invalid_timestamps,
            "devnet-genesis",
            &program_id,
            &"11".repeat(32),
            1024,
            2048,
            &authority,
            None,
        )
        .is_err());
    }

    #[test]
    fn journal_status_allowlist_rejects_unknown_states() {
        assert!(deployment_status_is_known(
            DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED
        ));
        assert!(deployment_status_is_known(
            DEPLOYMENT_STATUS_WRITE_CONFIRMED
        ));
        assert!(deployment_status_is_known(DEPLOYMENT_STATUS_FINALIZED));
        assert!(!deployment_status_is_known("retry_everything"));
    }

    #[test]
    fn expiry_classifier_requires_strict_finalized_height_and_absent_history() {
        assert_eq!(
            classify_deployment_attempt_expiry(41, 42, None, 100, 101),
            DeploymentAttemptExpiryDecision::NotExpired
        );
        assert_eq!(
            classify_deployment_attempt_expiry(42, 42, None, 101, 102),
            DeploymentAttemptExpiryDecision::NotExpired
        );
        assert_eq!(
            classify_deployment_attempt_expiry(43, 42, None, 102, 102),
            DeploymentAttemptExpiryDecision::ExpiredAbsent {
                min_context_slot: 102
            }
        );
    }

    #[test]
    fn expiry_classifier_rejects_history_older_than_expiry_observation() {
        let decision = classify_deployment_attempt_expiry(43, 42, None, 499, 500);
        assert_eq!(
            decision,
            DeploymentAttemptExpiryDecision::HistoryContextTooOld {
                expiry_observation_slot: 500,
                history_context_slot: 499,
            }
        );
        let gated = gate_expired_absent_with(decision, |_| {
            panic!("stale history must not reach business-state readback")
        })
        .unwrap();
        assert!(matches!(
            gated,
            DeploymentAttemptExpiryDecision::HistoryContextTooOld { .. }
        ));
    }

    #[test]
    fn expired_absent_gate_passes_exact_history_slot_once() {
        let observed_slot = std::cell::Cell::new(None);
        let decision = gate_expired_absent_with(
            DeploymentAttemptExpiryDecision::ExpiredAbsent {
                min_context_slot: 987,
            },
            |slot| {
                assert!(observed_slot.replace(Some(slot)).is_none());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(observed_slot.get(), Some(987));
        assert_eq!(
            decision,
            DeploymentAttemptExpiryDecision::ExpiredAbsent {
                min_context_slot: 987
            }
        );
    }

    #[test]
    fn expired_absent_gate_fails_before_authorizing_transition() {
        let transition_authorized = std::cell::Cell::new(false);
        let result = gate_expired_absent_with(
            DeploymentAttemptExpiryDecision::ExpiredAbsent {
                min_context_slot: 654,
            },
            |slot| {
                assert_eq!(slot, 654);
                Err(deployment_journal_error("二次业务回读已前进"))
            },
        );
        if result.is_ok() {
            transition_authorized.set(true);
        }
        assert!(result.is_err());
        assert!(!transition_authorized.get());

        let non_expired =
            gate_expired_absent_with(DeploymentAttemptExpiryDecision::NotExpired, |_| {
                panic!("non-expired decision must not run the absence callback")
            })
            .unwrap();
        assert_eq!(non_expired, DeploymentAttemptExpiryDecision::NotExpired);
    }

    #[test]
    fn recovery_account_config_is_finalized_and_history_slot_bound() {
        let config = finalized_recovery_account_config(321);
        assert_eq!(config.encoding, Some(UiAccountEncoding::Base64Zstd));
        assert_eq!(config.commitment, Some(CommitmentConfig::finalized()));
        assert_eq!(config.min_context_slot, Some(321));
        assert!(config.data_slice.is_none());
    }

    #[test]
    fn recovery_account_decode_failure_is_not_treated_as_absence() {
        let address = Pubkey::new_unique();
        let malformed = UiAccount {
            lamports: 1,
            data: UiAccountData::Binary(
                "not-valid-base64-zstd".to_string(),
                UiAccountEncoding::Base64Zstd,
            ),
            owner: Pubkey::new_unique().to_string(),
            executable: false,
            rent_epoch: 0,
            space: None,
        };
        let error = decode_recovery_ui_account(&address, malformed).unwrap_err();
        assert!(error.message.contains("拒绝将其视为不存在"));
    }

    #[test]
    fn write_recovery_requires_the_exact_chunk_and_unchanged_snapshot() {
        let initial = program_deploy::BufferWritePlan {
            completed_chunks: 1,
            pending_chunk_indexes: vec![1, 2],
        };
        let unchanged = program_deploy::BufferWritePlan {
            completed_chunks: 1,
            pending_chunk_indexes: vec![1, 2],
        };
        assert!(validate_write_attempt_recovery_snapshot(&initial, &unchanged, 2).is_ok());
        assert!(validate_write_attempt_recovery_snapshot(&initial, &unchanged, 0).is_err());

        let advanced_other_chunk = program_deploy::BufferWritePlan {
            completed_chunks: 2,
            pending_chunk_indexes: vec![2],
        };
        assert!(
            validate_write_attempt_recovery_snapshot(&initial, &advanced_other_chunk, 2).is_err()
        );

        let target_completed = program_deploy::BufferWritePlan {
            completed_chunks: 2,
            pending_chunk_indexes: vec![1],
        };
        assert!(validate_write_attempt_recovery_snapshot(&initial, &target_completed, 2).is_err());
    }

    #[test]
    fn expiry_classifier_blocks_every_observed_history_state() {
        let non_finalized = TransactionStatus {
            slot: 1,
            confirmations: Some(1),
            status: Ok(()),
            err: None,
            confirmation_status: None,
        };
        assert_eq!(
            classify_deployment_attempt_expiry(43, 42, Some(&non_finalized), 200, 199),
            DeploymentAttemptExpiryDecision::SeenButNotFinalized
        );

        let finalized_success = TransactionStatus {
            slot: 2,
            confirmations: None,
            status: Ok(()),
            err: None,
            confirmation_status: None,
        };
        assert_eq!(
            classify_deployment_attempt_expiry(43, 42, Some(&finalized_success), 201, 200),
            DeploymentAttemptExpiryDecision::FinalizedSucceeded
        );

        let transaction_error = solana_sdk::transaction::TransactionError::AccountNotFound;
        let finalized_failure = TransactionStatus {
            slot: 3,
            confirmations: None,
            status: Err(transaction_error.clone()),
            err: Some(transaction_error),
            confirmation_status: None,
        };
        assert_eq!(
            classify_deployment_attempt_expiry(43, 42, Some(&finalized_failure), 202, 201),
            DeploymentAttemptExpiryDecision::FinalizedFailed("AccountNotFound".to_string())
        );
    }

    #[test]
    fn malformed_attempt_evidence_is_rejected_before_recovery() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let buffer = Pubkey::new_unique();
        let record = record(&program_id, &authority, &buffer);
        let attempt = wallet_store::ProgramDeploymentAttemptRecord {
            genesis_hash: record.genesis_hash.clone(),
            program_id: record.program_id.clone(),
            stage: wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE.to_string(),
            buffer_address: record.buffer_address.clone(),
            chunk_index: Some(0),
            signature: Signature::new_unique().to_string(),
            last_valid_block_height: 42,
            status: wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED.to_string(),
            created_at: 1,
            updated_at: 1,
        };
        assert!(validate_program_deployment_attempt_record(&record, &attempt).is_ok());

        let mut invalid_signature = attempt.clone();
        invalid_signature.signature = "not-a-signature".to_string();
        let mut default_signature = attempt.clone();
        default_signature.signature = Signature::default().to_string();
        let mut noncanonical_signature = attempt.clone();
        noncanonical_signature.signature.push(' ');
        let mut zero_buffer = attempt.clone();
        zero_buffer.buffer_address = Pubkey::default().to_string();
        let mut noncanonical_buffer = attempt.clone();
        noncanonical_buffer.buffer_address.push(' ');
        let mut zero_height = attempt.clone();
        zero_height.last_valid_block_height = 0;
        let mut zero_created_at = attempt.clone();
        zero_created_at.created_at = 0;
        let mut reversed_timestamps = attempt.clone();
        reversed_timestamps.updated_at = 0;
        for invalid in [
            invalid_signature,
            default_signature,
            noncanonical_signature,
            zero_buffer,
            noncanonical_buffer,
            zero_height,
            zero_created_at,
            reversed_timestamps,
        ] {
            assert!(validate_program_deployment_attempt_record(&record, &invalid).is_err());
        }
    }

    #[test]
    fn generic_deploy_request_rejects_unknown_policy_fields() {
        let request = serde_json::from_value::<DeployProgramRequest>(serde_json::json!({
            "program_keypair_json": "[]",
            "expected_program_id": Pubkey::new_unique().to_string(),
            "expected_upgrade_authority": Pubkey::new_unique().to_string(),
            "expected_genesis_hash": solana_sdk::hash::Hash::new_unique().to_string(),
            "expected_program_sha256": "11".repeat(32),
            "program_so_base64": "f0VMRg==",
            "network": "devnet",
            "require_authenticated_keystore": true
        }));

        assert!(request.is_err());
    }

    #[test]
    fn canonical_program_id_uses_generic_journal_rules() {
        let program_id = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let buffer = Pubkey::new_unique();
        let record = record(&program_id, &authority, &buffer);

        assert_eq!(
            validate_deployment_journal_binding(
                &record,
                "devnet-genesis",
                &program_id,
                &"11".repeat(32),
                1024,
                2048,
                &authority,
                Some(&buffer),
            )
            .unwrap(),
            buffer
        );
    }

    #[test]
    fn deployment_capacity_errors_have_retryable_http_statuses() {
        assert_eq!(
            ApiError {
                message: PROGRAM_DEPLOY_BUSY_MESSAGE.to_string(),
            }
            .status_code(),
            StatusCode::CONFLICT
        );
        assert_eq!(
            ApiError {
                message: SBF_VERIFY_BUSY_MESSAGE.to_string(),
            }
            .status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}

#[derive(Deserialize)]
struct ProgramDeploySourceRequest {
    source_dir: String,
    #[serde(default)]
    build: bool,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct ProgramDeploySourceResponse {
    source_dir: String,
    built: bool,
    build_status: Option<String>,
    build_job_id: Option<String>,
    build_error: Option<String>,
    build_command: Option<String>,
    build_template: Option<String>,
    build_stdout: Option<String>,
    build_stderr: Option<String>,
    program_so_path: Option<String>,
    program_so_name: Option<String>,
    program_so_base64: Option<String>,
    program_so_sha256: Option<String>,
    approved_program_sha256: Option<String>,
    program_so_size: Option<usize>,
    program_keypair_path: Option<String>,
    expected_program_id: Option<String>,
    manifest_program_id: Option<String>,
    manifest_network: Option<String>,
    manifest_genesis_hash: Option<String>,
    manifest_upgrade_authority: Option<String>,
    manifest_owner_admin: Option<String>,
    manifest_operational_admin: Option<String>,
    build_available: bool,
    build_blocked_reason: Option<String>,
    warnings: Vec<String>,
}

struct ProgramSourceArtifacts {
    program_so_path: Option<PathBuf>,
    program_so_name: Option<String>,
    program_so_base64: Option<String>,
    program_so_sha256: Option<String>,
    approved_program_sha256: Option<String>,
    program_so_size: Option<usize>,
    program_keypair_path: Option<PathBuf>,
    expected_program_id: Option<String>,
    manifest_program_id: Option<String>,
    manifest_network: Option<String>,
    manifest_genesis_hash: Option<String>,
    manifest_upgrade_authority: Option<String>,
    manifest_owner_admin: Option<String>,
    manifest_operational_admin: Option<String>,
    warnings: Vec<String>,
}

#[derive(Clone)]
struct ProgramSourceBuildJob {
    source_dir: PathBuf,
    status: String,
    build_command: Option<String>,
    build_template: Option<String>,
    build_stdout: Option<String>,
    build_stderr: Option<String>,
    build_error: Option<String>,
    warnings: Vec<String>,
}

struct ProgramSourceBuildOutcome {
    command: String,
    template: String,
    stdout: String,
    stderr: String,
    warnings: Vec<String>,
}

fn program_source_build_jobs(
) -> &'static Arc<tokio::sync::Mutex<HashMap<String, ProgramSourceBuildJob>>> {
    PROGRAM_SOURCE_BUILD_JOBS.get_or_init(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())))
}

fn program_source_error(message: impl Into<String>) -> ApiError {
    ApiError {
        message: message.into(),
    }
}

fn canonical_child_path(root: &FsPath, path: &FsPath, label: &str) -> Result<PathBuf, ApiError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| program_source_error(format!("读取 {label} 路径失败: {error}")))?;
    if !canonical.starts_with(root) {
        return Err(program_source_error(format!("{label} 必须位于源码目录内")));
    }
    Ok(canonical)
}

fn read_text_file_limited(
    path: &FsPath,
    max_bytes: usize,
    label: &str,
) -> Result<String, ApiError> {
    let metadata = fs::metadata(path)
        .map_err(|error| program_source_error(format!("读取 {label} 元数据失败: {error}")))?;
    if !metadata.is_file() {
        return Err(program_source_error(format!("{label} 不是普通文件")));
    }
    if metadata.len() > max_bytes as u64 {
        return Err(program_source_error(format!("{label} 文件过大")));
    }
    fs::read_to_string(path)
        .map_err(|error| program_source_error(format!("读取 {label} 失败: {error}")))
}

fn read_bytes_file_limited(
    path: &FsPath,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, ApiError> {
    let metadata = fs::metadata(path)
        .map_err(|error| program_source_error(format!("读取 {label} 元数据失败: {error}")))?;
    if !metadata.is_file() {
        return Err(program_source_error(format!("{label} 不是普通文件")));
    }
    if metadata.len() == 0 || metadata.len() > max_bytes as u64 {
        return Err(program_source_error(format!("{label} 大小无效")));
    }
    fs::read(path).map_err(|error| program_source_error(format!("读取 {label} 失败: {error}")))
}

fn first_existing_path_owned(root: &FsPath, candidates: &[String]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(|relative| root.join(relative))
        .filter_map(|path| canonical_child_path(root, &path, "候选产物").ok())
        .find(|path| path.is_file())
}

fn first_json_file(root: &FsPath, directory: &FsPath) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| canonical_child_path(root, &path, "JSON 产物").ok())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .min()
}

fn first_so_file(root: &FsPath, directory: &FsPath) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| canonical_child_path(root, &path, ".so 产物").ok())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("so"))
        })
        .min()
}

fn safe_artifact_stem(value: &str) -> Option<String> {
    let stem = value.trim().replace('-', "_");
    if stem.is_empty()
        || stem.len() > 128
        || !stem
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    Some(stem)
}

fn parse_quoted_toml_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        return rest
            .split_once('"')
            .map(|(value, _)| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
    trimmed
        .split('#')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn parse_anchor_program_name(root: &FsPath) -> Option<String> {
    let anchor = fs::read_to_string(root.join("Anchor.toml")).ok()?;
    let mut in_programs_section = false;
    for line in anchor.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_programs_section = trimmed.starts_with("[programs.");
            continue;
        }
        if !in_programs_section || trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            continue;
        };
        let Some(program_id) = parse_quoted_toml_value(value) else {
            continue;
        };
        if Pubkey::from_str(&program_id).is_ok() {
            if let Some(stem) = safe_artifact_stem(name) {
                return Some(stem);
            }
        }
    }
    None
}

fn parse_cargo_package_name(root: &FsPath) -> Option<String> {
    let cargo = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut in_package_section = false;
    for line in cargo.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_package_section = trimmed == "[package]";
            continue;
        }
        if !in_package_section || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            continue;
        };
        if name.trim() != "name" {
            continue;
        }
        let package_name = parse_quoted_toml_value(value)?;
        return safe_artifact_stem(&package_name);
    }
    None
}

fn detect_program_artifact_stem(root: &FsPath) -> Option<String> {
    parse_anchor_program_name(root).or_else(|| parse_cargo_package_name(root))
}

fn find_program_so_path(root: &FsPath, artifact_stem: Option<&str>) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(stem) = artifact_stem.and_then(safe_artifact_stem) {
        candidates.push(format!("target/verifiable/{stem}.so"));
        candidates.push(format!("target/deploy/{stem}.so"));
    }
    first_existing_path_owned(root, &candidates)
        .or_else(|| first_so_file(root, &root.join("target/verifiable")))
        .or_else(|| first_so_file(root, &root.join("target/deploy")))
}

fn find_program_keypair_path(root: &FsPath, artifact_stem: Option<&str>) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(stem) = artifact_stem.and_then(safe_artifact_stem) {
        candidates.push(format!(".keys/{stem}-program-keypair.json"));
        candidates.push(format!(".keys/{stem}-keypair.json"));
        candidates.push(format!("target/deploy/{stem}-keypair.json"));
    }
    first_existing_path_owned(root, &candidates)
        .or_else(|| first_json_file(root, &root.join(".keys")))
        .or_else(|| first_json_file(root, &root.join("target/deploy")))
}

fn find_release_manifest_path(root: &FsPath) -> Option<PathBuf> {
    fs::read_dir(root.join("docs"))
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| canonical_child_path(root, &path, "release manifest").ok())
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                return false;
            };
            path.is_file()
                && name.contains("RELEASE_MANIFEST")
                && name.ends_with(".json")
                && !name.contains(".example.")
        })
        .min()
}

fn parse_anchor_program_id(root: &FsPath) -> Option<String> {
    let anchor = fs::read_to_string(root.join("Anchor.toml")).ok()?;
    for line in anchor.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }
        let Some((_, value)) = trimmed.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        if Pubkey::from_str(value).is_ok() {
            return Some(value.to_string());
        }
    }
    None
}

fn path_string(path: &FsPath) -> String {
    path.to_string_lossy().to_string()
}

fn json_pointer_string(value: &Value, pointer: &str) -> Option<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.starts_with("REPLACE_WITH_"))
        .map(ToOwned::to_owned)
}

fn release_metadata_sha_and_len(
    root: &FsPath,
    artifact_stem: Option<&str>,
) -> Result<(Option<String>, Option<usize>), ApiError> {
    let mut candidates = Vec::new();
    if let Some(stem) = artifact_stem.and_then(safe_artifact_stem) {
        candidates.push(format!("target/verifiable/{stem}-build.json"));
    }
    let Some(metadata_path) = first_existing_path_owned(root, &candidates)
        .or_else(|| first_json_file(root, &root.join("target/verifiable")))
    else {
        return Ok((None, None));
    };
    let text = read_text_file_limited(&metadata_path, 128 * 1024, "build metadata")?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|_| program_source_error("build metadata 不是有效 JSON"))?;
    let sha = json_pointer_string(&value, "/artifacts/program/sha256")
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .map(|value| value.to_ascii_lowercase());
    let len = value
        .pointer("/artifacts/program/length")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok());
    Ok((sha, len))
}

type ReleaseManifestValues = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

fn release_manifest_values(root: &FsPath) -> Result<ReleaseManifestValues, ApiError> {
    let Some(manifest_path) = find_release_manifest_path(root) else {
        return Ok((None, None, None, None, None));
    };
    let text = read_text_file_limited(&manifest_path, 512 * 1024, "release manifest")?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|_| program_source_error("release manifest 不是有效 JSON"))?;
    Ok((
        json_pointer_string(&value, "/target/network"),
        json_pointer_string(&value, "/target/genesis_hash"),
        json_pointer_string(&value, "/target/upgrade_authority"),
        json_pointer_string(&value, "/target/owner_admin"),
        json_pointer_string(&value, "/target/admin"),
    ))
}

fn load_program_source_artifacts(
    root: &FsPath,
    artifact_stem: Option<&str>,
    fallback_network: Option<&str>,
    warn_missing_artifacts: bool,
) -> Result<ProgramSourceArtifacts, ApiError> {
    let program_so_path = find_program_so_path(root, artifact_stem);
    let program_keypair_path = find_program_keypair_path(root, artifact_stem);
    let (metadata_sha, metadata_len) = release_metadata_sha_and_len(root, artifact_stem)?;
    let (
        mut manifest_network,
        manifest_genesis_hash,
        manifest_upgrade_authority,
        manifest_owner_admin,
        manifest_operational_admin,
    ) = release_manifest_values(root)?;
    if manifest_network.is_none() {
        manifest_network = fallback_network
            .map(str::trim)
            .filter(|value| matches!(*value, "mainnet" | "devnet" | "testnet"))
            .map(ToOwned::to_owned);
    }
    let manifest_program_id = parse_anchor_program_id(root);

    let mut warnings = Vec::new();
    let mut program_so_base64 = None;
    let mut program_so_sha256 = None;
    let mut program_so_size = None;
    let mut program_so_name = None;
    if let Some(path) = program_so_path.as_ref() {
        let bytes = read_bytes_file_limited(path, MAX_PROGRAM_SO_BYTES, ".so 文件")?;
        let actual_sha = program_deploy::sha256_hex(&bytes);
        let actual_len = bytes.len();
        if let Some(expected_sha) = metadata_sha.as_ref() {
            if expected_sha != &actual_sha {
                warnings.push(format!(
                    "build metadata SHA-256 为 {expected_sha}，但 .so 实际为 {actual_sha}"
                ));
            }
        }
        if let Some(expected_len) = metadata_len {
            if expected_len != actual_len {
                warnings.push(format!(
                    "build metadata 长度为 {expected_len}，但 .so 实际为 {actual_len}"
                ));
            }
        }
        program_so_base64 = Some(BASE64.encode(&bytes));
        program_so_sha256 = Some(actual_sha);
        program_so_size = Some(actual_len);
        program_so_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned);
    } else if warn_missing_artifacts {
        warnings.push("未找到 target/verifiable/*.so 或 target/deploy/*.so".to_string());
    }

    let mut expected_program_id = None;
    if let Some(path) = program_keypair_path.as_ref() {
        let mut json = read_text_file_limited(
            path,
            program_deploy::MAX_PROGRAM_KEYPAIR_JSON_BYTES,
            "Program keypair",
        )?;
        let keypair =
            program_deploy::parse_program_keypair_json(&json).map_err(program_source_error)?;
        json.zeroize();
        expected_program_id = Some(keypair.pubkey().to_string());
        if let (Some(manifest_id), Some(keypair_id)) =
            (manifest_program_id.as_ref(), expected_program_id.as_ref())
        {
            if manifest_id != keypair_id {
                warnings.push(format!(
                    "Anchor.toml Program ID 为 {manifest_id}，但 Program keypair 派生为 {keypair_id}"
                ));
            }
        }
    } else if warn_missing_artifacts {
        warnings.push("未找到 .keys/*.json 或 target/deploy/*-keypair.json".to_string());
    }

    Ok(ProgramSourceArtifacts {
        program_so_path,
        program_so_name,
        program_so_base64,
        program_so_sha256: program_so_sha256.clone(),
        approved_program_sha256: metadata_sha.or(program_so_sha256),
        program_so_size,
        program_keypair_path,
        expected_program_id,
        manifest_program_id,
        manifest_network,
        manifest_genesis_hash,
        manifest_upgrade_authority,
        manifest_owner_admin,
        manifest_operational_admin,
        warnings,
    })
}

fn truncate_build_log(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= PROGRAM_SOURCE_BUILD_LOG_BYTES {
        return text.to_string();
    }
    let start = text.len().saturating_sub(PROGRAM_SOURCE_BUILD_LOG_BYTES);
    format!("...[truncated]{}", &text[start..])
}

#[derive(Clone)]
struct ProgramSourceBuildPlan {
    command: Vec<String>,
    display_command: String,
    template: String,
}

impl ProgramSourceBuildPlan {
    fn new(template: &str, command: &[&str]) -> Self {
        Self {
            command: command.iter().map(|part| (*part).to_string()).collect(),
            display_command: command.join(" "),
            template: template.to_string(),
        }
    }
}

fn program_source_build_plans(root: &FsPath) -> (Vec<ProgramSourceBuildPlan>, Option<String>) {
    let contains_source_keys = root.join(".keys").exists();
    let mut plans = Vec::new();
    if !contains_source_keys && root.join("scripts/build-verifiable.sh").is_file() {
        plans.push(ProgramSourceBuildPlan::new(
            "项目脚本",
            &["bash", "scripts/build-verifiable.sh"],
        ));
    }
    if !contains_source_keys && root.join("Makefile").is_file() {
        plans.push(ProgramSourceBuildPlan::new(
            "项目 Makefile",
            &["make", "build"],
        ));
    }
    if root.join("Anchor.toml").is_file() {
        plans.push(ProgramSourceBuildPlan::new(
            "sol-safekey 内置 Anchor 模板",
            &["anchor", "build"],
        ));
    }
    if root.join("Cargo.toml").is_file() {
        plans.push(ProgramSourceBuildPlan::new(
            "sol-safekey 内置 Cargo SBF 模板",
            &["cargo", "build-sbf"],
        ));
    }
    if plans.is_empty() {
        (
            plans,
            Some(if contains_source_keys {
                "源码目录包含 .keys；已跳过项目脚本和 Makefile，且未识别 Anchor.toml 或 Cargo.toml，无法使用内置模板编译".to_string()
            } else {
                "未识别 Anchor.toml、Cargo.toml 或构建脚本".to_string()
            }),
        )
    } else {
        (plans, None)
    }
}

fn program_source_keys_build_warning(root: &FsPath) -> Option<String> {
    root.join(".keys").exists().then(|| {
        "源码目录包含 .keys；为避免项目脚本读取签名材料，已跳过 scripts/build-verifiable.sh 和 Makefile，仅使用 sol-safekey 内置 Anchor/Cargo 构建模板".to_string()
    })
}

async fn run_program_source_build(
    root: &FsPath,
    plan: &ProgramSourceBuildPlan,
) -> Result<(String, String), ApiError> {
    let executable = plan
        .command
        .first()
        .ok_or_else(|| program_source_error("构建命令为空"))?;
    let mut process = Command::new(executable);
    process.args(plan.command.iter().skip(1)).current_dir(root);
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        let candidate_bins = [
            home.join(".nvm/versions/node/v20.19.5/bin"),
            home.join(".avm/bin"),
            home.join(".cargo/bin"),
            home.join(".local/share/solana/install/active_release/bin"),
        ];
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = std::env::split_paths(&current_path).collect::<Vec<_>>();
        for bin in candidate_bins.into_iter().rev() {
            if bin.is_dir() {
                paths.insert(0, bin);
            }
        }
        if let Ok(path) = std::env::join_paths(paths) {
            process.env("PATH", path);
        }
    }
    let output = timeout(
        Duration::from_secs(PROGRAM_SOURCE_BUILD_TIMEOUT_SECS),
        process.output(),
    )
    .await
    .map_err(|_| program_source_error("构建超时"))?
    .map_err(|error| program_source_error(format!("启动构建失败: {error}")))?;
    let stdout = truncate_build_log(&output.stdout);
    let stderr = truncate_build_log(&output.stderr);
    if !output.status.success() {
        return Err(program_source_error(format!(
            "{} 构建失败: {}\n{}",
            plan.template, output.status, stderr
        )));
    }
    Ok((stdout, stderr))
}

async fn execute_program_source_build(
    source_dir: &FsPath,
    build_plans: &[ProgramSourceBuildPlan],
    build_blocked_reason: Option<String>,
    artifact_stem: Option<&str>,
) -> Result<ProgramSourceBuildOutcome, ApiError> {
    if build_plans.is_empty() {
        return Err(program_source_error(
            build_blocked_reason.unwrap_or_else(|| "当前源码目录不可自动编译".to_string()),
        ));
    }

    let mut warnings = Vec::new();
    let mut last_error = None;
    for plan in build_plans.iter() {
        match run_program_source_build(source_dir, plan).await {
            Ok((stdout, stderr)) => {
                if find_program_so_path(source_dir, artifact_stem).is_none() {
                    let message = format!(
                        "{}: 构建命令已结束，但未生成 target/verifiable/*.so 或 target/deploy/*.so",
                        plan.display_command
                    );
                    warnings.push(message.clone());
                    last_error = Some(message);
                    continue;
                }
                if !warnings.is_empty() {
                    warnings.push(format!("已改用 {} 完成编译", plan.template));
                }
                return Ok(ProgramSourceBuildOutcome {
                    command: plan.display_command.clone(),
                    template: plan.template.clone(),
                    stdout,
                    stderr,
                    warnings,
                });
            }
            Err(error) => {
                let message = format!("{}: {}", plan.display_command, error.message);
                warnings.push(message.clone());
                last_error = Some(message);
            }
        }
    }

    Err(program_source_error(last_error.unwrap_or_else(|| {
        build_blocked_reason.unwrap_or_else(|| "构建完成后未找到可部署 .so".to_string())
    })))
}

fn next_program_source_build_job_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or_default();
    format!("build-{millis}")
}

async fn start_program_source_build_job(
    source_dir: PathBuf,
    build_plans: Vec<ProgramSourceBuildPlan>,
    build_blocked_reason: Option<String>,
    artifact_stem: Option<String>,
    initial_warnings: Vec<String>,
) -> (String, Option<String>, Option<String>) {
    let job_id = next_program_source_build_job_id();
    let build_command = build_plans.first().map(|plan| plan.display_command.clone());
    let build_template = build_plans.first().map(|plan| plan.template.clone());
    let job = ProgramSourceBuildJob {
        source_dir: source_dir.clone(),
        status: "running".to_string(),
        build_command: build_command.clone(),
        build_template: build_template.clone(),
        build_stdout: None,
        build_stderr: None,
        build_error: None,
        warnings: initial_warnings,
    };
    program_source_build_jobs()
        .lock()
        .await
        .insert(job_id.clone(), job);

    let jobs = Arc::clone(program_source_build_jobs());
    let task_job_id = job_id.clone();
    tokio::spawn(async move {
        let result = execute_program_source_build(
            &source_dir,
            &build_plans,
            build_blocked_reason,
            artifact_stem.as_deref(),
        )
        .await;
        let mut jobs = jobs.lock().await;
        if let Some(job) = jobs.get_mut(&task_job_id) {
            match result {
                Ok(outcome) => {
                    job.status = "completed".to_string();
                    job.build_command = Some(outcome.command);
                    job.build_template = Some(outcome.template);
                    job.build_stdout = Some(outcome.stdout);
                    job.build_stderr = Some(outcome.stderr);
                    job.warnings.extend(outcome.warnings);
                }
                Err(error) => {
                    job.status = "failed".to_string();
                    job.build_error = Some(error.message);
                }
            }
        }
    });

    (job_id, build_command, build_template)
}

async fn program_deploy_source(
    Json(req): Json<ProgramDeploySourceRequest>,
) -> Result<Json<ProgramDeploySourceResponse>, ApiError> {
    let source_dir_raw = req.source_dir.trim();
    if source_dir_raw.is_empty() || source_dir_raw.len() > 4096 {
        return Err(program_source_error("源码目录路径无效"));
    }
    let source_dir = PathBuf::from(source_dir_raw)
        .canonicalize()
        .map_err(|error| program_source_error(format!("源码目录不存在或不可访问: {error}")))?;
    if !source_dir.is_dir() {
        return Err(program_source_error("源码目录不是目录"));
    }

    let (build_plans, build_blocked_reason) = program_source_build_plans(&source_dir);
    let build_available = !build_plans.is_empty();
    let mut build_command = build_plans.first().map(|plan| plan.display_command.clone());
    let mut build_template = build_plans.first().map(|plan| plan.template.clone());
    let artifact_stem = detect_program_artifact_stem(&source_dir);
    let built = false;
    let mut build_status = None;
    let mut build_job_id = None;
    let build_error = None;
    let build_stdout = None;
    let build_stderr = None;
    let mut build_warnings = Vec::new();
    if let Some(warning) = program_source_keys_build_warning(&source_dir) {
        build_warnings.push(warning);
    }
    if req.build {
        if build_plans.is_empty() {
            return Err(program_source_error(
                build_blocked_reason
                    .clone()
                    .unwrap_or_else(|| "当前源码目录不可自动编译".to_string()),
            ));
        }
        let (job_id, job_command, job_template) = start_program_source_build_job(
            source_dir.clone(),
            build_plans.clone(),
            build_blocked_reason.clone(),
            artifact_stem.clone(),
            build_warnings.clone(),
        )
        .await;
        build_command = job_command;
        build_template = job_template;
        build_status = Some("running".to_string());
        build_job_id = Some(job_id);
        build_warnings.push("编译已在后台开始，完成后会自动重新读取部署信息".to_string());
    }

    let mut artifacts = load_program_source_artifacts(
        &source_dir,
        artifact_stem.as_deref(),
        req.network.as_deref(),
        true,
    )?;
    build_warnings.append(&mut artifacts.warnings);

    Ok(Json(ProgramDeploySourceResponse {
        source_dir: path_string(&source_dir),
        built,
        build_status,
        build_job_id,
        build_error,
        build_command,
        build_template,
        build_stdout,
        build_stderr,
        program_so_path: artifacts
            .program_so_path
            .as_ref()
            .map(|path| path_string(path)),
        program_so_name: artifacts.program_so_name,
        program_so_base64: artifacts.program_so_base64,
        program_so_sha256: artifacts.program_so_sha256,
        approved_program_sha256: artifacts.approved_program_sha256,
        program_so_size: artifacts.program_so_size,
        program_keypair_path: artifacts
            .program_keypair_path
            .as_ref()
            .map(|path| path_string(path)),
        expected_program_id: artifacts.expected_program_id,
        manifest_program_id: artifacts.manifest_program_id,
        manifest_network: artifacts.manifest_network,
        manifest_genesis_hash: artifacts.manifest_genesis_hash,
        manifest_upgrade_authority: artifacts.manifest_upgrade_authority,
        manifest_owner_admin: artifacts.manifest_owner_admin,
        manifest_operational_admin: artifacts.manifest_operational_admin,
        build_available,
        build_blocked_reason,
        warnings: build_warnings,
    }))
}

async fn program_deploy_source_build_status(
    Path(job_id): Path<String>,
) -> Result<Json<ProgramDeploySourceResponse>, ApiError> {
    let job = {
        let jobs = program_source_build_jobs().lock().await;
        jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| program_source_error("构建任务不存在或已过期"))?
    };
    let (build_plans, build_blocked_reason) = program_source_build_plans(&job.source_dir);
    let build_available = !build_plans.is_empty();
    let artifact_stem = detect_program_artifact_stem(&job.source_dir);
    let mut warnings = job.warnings.clone();
    if let Some(error) = job.build_error.as_ref() {
        warnings.push(error.clone());
    }
    let mut artifacts = load_program_source_artifacts(
        &job.source_dir,
        artifact_stem.as_deref(),
        None,
        job.status != "running",
    )?;
    warnings.append(&mut artifacts.warnings);

    Ok(Json(ProgramDeploySourceResponse {
        source_dir: path_string(&job.source_dir),
        built: job.status == "completed",
        build_status: Some(job.status),
        build_job_id: Some(job_id),
        build_error: job.build_error,
        build_command: job.build_command,
        build_template: job.build_template,
        build_stdout: job.build_stdout,
        build_stderr: job.build_stderr,
        program_so_path: artifacts
            .program_so_path
            .as_ref()
            .map(|path| path_string(path)),
        program_so_name: artifacts.program_so_name,
        program_so_base64: artifacts.program_so_base64,
        program_so_sha256: artifacts.program_so_sha256,
        approved_program_sha256: artifacts.approved_program_sha256,
        program_so_size: artifacts.program_so_size,
        program_keypair_path: artifacts
            .program_keypair_path
            .as_ref()
            .map(|path| path_string(path)),
        expected_program_id: artifacts.expected_program_id,
        manifest_program_id: artifacts.manifest_program_id,
        manifest_network: artifacts.manifest_network,
        manifest_genesis_hash: artifacts.manifest_genesis_hash,
        manifest_upgrade_authority: artifacts.manifest_upgrade_authority,
        manifest_owner_admin: artifacts.manifest_owner_admin,
        manifest_operational_admin: artifacts.manifest_operational_admin,
        build_available,
        build_blocked_reason,
        warnings,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgramDeploymentJournalRequest {
    network: String,
    expected_genesis_hash: String,
    expected_program_id: String,
    expected_program_sha256: String,
    program_len: usize,
    max_data_len: usize,
    expected_upgrade_authority: String,
}

#[derive(Serialize)]
struct ProgramDeploymentJournalResponse {
    network: String,
    genesis_hash: String,
    write_chunk_bytes: usize,
    write_chunk_count: usize,
    journal: Option<ProgramDeploymentJournalView>,
    deployment_attempts: Vec<wallet_store::ProgramDeploymentAttemptRecord>,
}

#[derive(Serialize)]
struct ProgramDeploymentJournalView {
    genesis_hash: String,
    program_id: String,
    program_sha256: String,
    program_len: usize,
    max_data_len: usize,
    upgrade_authority: String,
    buffer_address: String,
    status: String,
    create_signature: Option<String>,
    create_last_valid_block_height: Option<u64>,
    last_write_signature: Option<String>,
    last_write_chunk_index: Option<usize>,
    last_write_last_valid_block_height: Option<u64>,
    completed_writes: usize,
    deploy_signature: Option<String>,
    deploy_last_valid_block_height: Option<u64>,
    attempt_evidence_version: u32,
    revision: u64,
    created_at: u64,
    updated_at: u64,
}

impl From<&wallet_store::ProgramDeploymentRecord> for ProgramDeploymentJournalView {
    fn from(record: &wallet_store::ProgramDeploymentRecord) -> Self {
        Self {
            genesis_hash: record.genesis_hash.clone(),
            program_id: record.program_id.clone(),
            program_sha256: record.program_sha256.clone(),
            program_len: record.program_len,
            max_data_len: record.max_data_len,
            upgrade_authority: record.upgrade_authority.clone(),
            buffer_address: record.buffer_address.clone(),
            status: record.status.clone(),
            create_signature: record.create_signature.clone(),
            create_last_valid_block_height: record.create_last_valid_block_height,
            last_write_signature: record.last_write_signature.clone(),
            last_write_chunk_index: record.last_write_chunk_index,
            last_write_last_valid_block_height: record.last_write_last_valid_block_height,
            completed_writes: record.completed_writes,
            deploy_signature: record.deploy_signature.clone(),
            deploy_last_valid_block_height: record.deploy_last_valid_block_height,
            attempt_evidence_version: record.attempt_evidence_version,
            revision: record.revision,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

struct ProgramDeploymentJournalIntent {
    expected_genesis_hash: solana_sdk::hash::Hash,
    program_id: Pubkey,
    program_sha256: String,
    program_len: usize,
    max_data_len: usize,
    upgrade_authority: Pubkey,
}

fn parse_canonical_deployment_pubkey(value: &str, label: &str) -> Result<Pubkey, ApiError> {
    let trimmed = value.trim();
    let pubkey = Pubkey::from_str(trimmed).map_err(|_| ApiError {
        message: format!("无效的{label}"),
    })?;
    if pubkey == Pubkey::default() || pubkey.to_string() != trimmed {
        return Err(ApiError {
            message: format!("{label}必须是 canonical 非零公钥"),
        });
    }
    Ok(pubkey)
}

fn validate_program_deployment_journal_request(
    req: &ProgramDeploymentJournalRequest,
) -> Result<ProgramDeploymentJournalIntent, ApiError> {
    if req.program_len == 0
        || req.program_len > MAX_PROGRAM_SO_BYTES
        || req.max_data_len < req.program_len
        || req.max_data_len > MAX_PROGRAM_SO_BYTES
    {
        return Err(ApiError {
            message: format!(
                "Program 长度必须满足 1 <= program_len <= max_data_len <= {}",
                MAX_PROGRAM_SO_BYTES
            ),
        });
    }
    let program_sha256 = req.expected_program_sha256.trim().to_ascii_lowercase();
    if program_sha256.len() != 64 || !program_sha256.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err(ApiError {
            message: "预期 Program SHA-256 必须是 64 位十六进制".to_string(),
        });
    }
    let expected_genesis_text = req.expected_genesis_hash.trim();
    let expected_genesis_hash =
        solana_sdk::hash::Hash::from_str(expected_genesis_text).map_err(|_| ApiError {
            message: "无效的预期 genesis hash".to_string(),
        })?;
    if expected_genesis_hash == solana_sdk::hash::Hash::default()
        || expected_genesis_hash.to_string() != expected_genesis_text
    {
        return Err(ApiError {
            message: "预期 genesis hash 必须是 canonical 非零值".to_string(),
        });
    }
    Ok(ProgramDeploymentJournalIntent {
        expected_genesis_hash,
        program_id: parse_canonical_deployment_pubkey(&req.expected_program_id, "预期 Program ID")?,
        program_sha256,
        program_len: req.program_len,
        max_data_len: req.max_data_len,
        upgrade_authority: parse_canonical_deployment_pubkey(
            &req.expected_upgrade_authority,
            "预期 Upgrade Authority",
        )?,
    })
}

async fn program_deployment_journal(
    Json(req): Json<ProgramDeploymentJournalRequest>,
) -> Result<Json<ProgramDeploymentJournalResponse>, ApiError> {
    if req.network.trim().is_empty() {
        return Err(ApiError {
            message: "部署网络不能为空".to_string(),
        });
    }
    let intent = validate_program_deployment_journal_request(&req)?;
    let selector = rpc_selector(Some(req.network.trim()))?;
    let network = selector.network;
    let client = RpcClient::new_with_timeout_and_commitment(
        selector.url.to_string(),
        Duration::from_secs(RPC_QUERY_TIMEOUT_SECS),
        CommitmentConfig::finalized(),
    );
    let actual_genesis_hash = client.get_genesis_hash().map_err(|error| ApiError {
        message: format!("读取 RPC genesis hash 失败: {error}"),
    })?;
    if actual_genesis_hash != intent.expected_genesis_hash {
        return Err(ApiError {
            message: format!(
                "RPC genesis hash 为 {}，预期为 {}；拒绝读取其他集群的部署记录",
                actual_genesis_hash, intent.expected_genesis_hash
            ),
        });
    }
    let genesis_hash = actual_genesis_hash.to_string();
    let program_id = intent.program_id.to_string();
    let (journal, deployment_attempts) =
        wallet_store::load_program_deployment_snapshot(&genesis_hash, &program_id)
            .map_err(|message| ApiError { message })?;
    if let Some(record) = journal.as_ref() {
        if !deployment_status_is_known(&record.status)
            || record.attempt_evidence_version
                != wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EVIDENCE_VERSION
        {
            return Err(deployment_journal_error(format!(
                "Program {} 的部署记录状态或证据版本无效",
                intent.program_id
            )));
        }
        validate_deployment_journal_binding(
            record,
            &genesis_hash,
            &intent.program_id,
            &intent.program_sha256,
            intent.program_len,
            intent.max_data_len,
            &intent.upgrade_authority,
            None,
        )?;
        for attempt in &deployment_attempts {
            validate_program_deployment_attempt_record(record, attempt)?;
        }
    } else if !deployment_attempts.is_empty() {
        return Err(deployment_journal_error(
            "部署 attempt 缺少对应 journal，拒绝返回撕裂快照",
        ));
    }
    Ok(Json(ProgramDeploymentJournalResponse {
        network,
        genesis_hash,
        write_chunk_bytes: PROGRAM_WRITE_CHUNK_BYTES,
        write_chunk_count: intent.program_len.div_ceil(PROGRAM_WRITE_CHUNK_BYTES),
        journal: journal.as_ref().map(ProgramDeploymentJournalView::from),
        deployment_attempts,
    }))
}

#[cfg(test)]
mod generic_program_deployment_policy_tests {
    use super::*;

    fn unique_temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!(
            "sol-safekey-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn valid_journal_request() -> ProgramDeploymentJournalRequest {
        ProgramDeploymentJournalRequest {
            network: "devnet".to_string(),
            expected_genesis_hash: solana_sdk::hash::Hash::new_unique().to_string(),
            expected_program_id: Pubkey::new_unique().to_string(),
            expected_program_sha256: "AB".repeat(32),
            program_len: 1_024,
            max_data_len: 2_048,
            expected_upgrade_authority: Pubkey::new_unique().to_string(),
        }
    }

    #[test]
    fn generic_journal_intent_normalizes_hash_without_hidden_policy() {
        let request = valid_journal_request();
        let intent = validate_program_deployment_journal_request(&request).unwrap();
        assert_eq!(intent.program_sha256, "ab".repeat(32));
        assert_eq!(intent.program_id.to_string(), request.expected_program_id);
        assert_eq!(
            intent.upgrade_authority.to_string(),
            request.expected_upgrade_authority
        );
    }

    #[test]
    fn generic_journal_intent_rejects_unsafe_boundaries() {
        let mut request = valid_journal_request();
        request.program_len = 0;
        assert!(validate_program_deployment_journal_request(&request).is_err());

        let mut request = valid_journal_request();
        request.max_data_len = request.program_len - 1;
        assert!(validate_program_deployment_journal_request(&request).is_err());

        let mut request = valid_journal_request();
        request.expected_program_sha256 = "xyz".to_string();
        assert!(validate_program_deployment_journal_request(&request).is_err());

        let mut request = valid_journal_request();
        request.expected_program_id = Pubkey::default().to_string();
        assert!(validate_program_deployment_journal_request(&request).is_err());

        let mut request = valid_journal_request();
        request.expected_genesis_hash = solana_sdk::hash::Hash::default().to_string();
        assert!(validate_program_deployment_journal_request(&request).is_err());
    }

    #[test]
    fn unknown_network_labels_never_fall_back_to_mainnet() {
        assert!(parse_rpc_selector("devnett").is_err());
        assert!(parse_rpc_selector("rpc:devnett:https%3A%2F%2Fexample.invalid").is_err());
        assert_eq!(parse_rpc_selector("mainnet").unwrap().network, "mainnet");
    }

    #[cfg(unix)]
    #[test]
    fn program_source_artifact_discovery_rejects_symlinks_outside_source_dir() {
        use std::os::unix::fs::symlink;

        let base = unique_temp_path("source-artifact-symlink");
        let source = base.join("source");
        let outside = base.join("outside");
        fs::create_dir_all(source.join("target/deploy")).unwrap();
        fs::create_dir_all(source.join(".keys")).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let outside_so = outside.join("escaped.so");
        let outside_keypair = outside.join("escaped-keypair.json");
        fs::write(&outside_so, b"not-a-program").unwrap();
        fs::write(&outside_keypair, b"[]").unwrap();
        symlink(&outside_so, source.join("target/deploy/escaped.so")).unwrap();
        symlink(&outside_keypair, source.join(".keys/escaped-keypair.json")).unwrap();

        let canonical_source = source.canonicalize().unwrap();
        assert!(find_program_so_path(&canonical_source, None).is_none());
        assert!(find_program_keypair_path(&canonical_source, None).is_none());
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeployProgramRequest {
    #[serde(default)]
    wallet_id: Option<String>,
    #[serde(default)]
    private_key: Option<String>,
    #[serde(default)]
    secret_key: Option<String>,
    #[serde(default)]
    keystore_json: Option<String>,
    #[serde(default)]
    encrypted_key: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    program_keypair_json: Option<String>,
    #[serde(default)]
    program_keypair_path: Option<String>,
    expected_program_id: String,
    expected_upgrade_authority: String,
    expected_genesis_hash: String,
    expected_program_sha256: String,
    program_so_base64: String,
    network: String,
    #[serde(default)]
    max_data_len: Option<usize>,
    #[serde(default)]
    resume_buffer_address: Option<String>,
}

impl DeployProgramRequest {
    fn take_wallet_auth(&mut self) -> WalletAuthRequest {
        WalletAuthRequest {
            wallet_id: self.wallet_id.take(),
            private_key: self.private_key.take(),
            secret_key: self.secret_key.take(),
            keystore_json: self.keystore_json.take(),
            encrypted_key: self.encrypted_key.take(),
            password: self.password.take(),
        }
    }
}

impl Drop for DeployProgramRequest {
    fn drop(&mut self) {
        if let Some(value) = self.program_keypair_json.as_mut() {
            value.zeroize();
        }
        self.take_wallet_auth().clear_secrets();
    }
}

#[derive(Serialize)]
struct DeployProgramResponse {
    program_id: String,
    programdata_address: String,
    buffer_address: String,
    authority: String,
    network: String,
    genesis_hash: String,
    program_bytes: usize,
    max_data_len: usize,
    program_sha256: String,
    temporary_buffer_rent_lamports: u64,
    program_rent_lamports: u64,
    programdata_rent_lamports: u64,
    rent_lamports: u64,
    estimated_transaction_fees_lamports: u64,
    fee_rate_reserve_lamports: u64,
    recovery_write_reserve_lamports: u64,
    total_fee_budget_lamports: u64,
    estimated_required_balance_lamports: u64,
    create_buffer_signature: Option<String>,
    skipped_write_chunks: usize,
    write_signatures: Vec<String>,
    deploy_signature: String,
    finalized_slot: u64,
    deployed_slot: u64,
    readback_verified: bool,
    journal_revision: Option<u64>,
    attempt_evidence_version: Option<u32>,
    deployment_attempts: Vec<wallet_store::ProgramDeploymentAttemptRecord>,
    status: String,
}

async fn deploy_generic_program(
    Json(req): Json<DeployProgramRequest>,
) -> Result<Json<DeployProgramResponse>, ApiError> {
    deploy_program(Json(req)).await
}

async fn deploy_program(
    Json(mut req): Json<DeployProgramRequest>,
) -> Result<Json<DeployProgramResponse>, ApiError> {
    let _deploy_guard = PROGRAM_DEPLOY_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .try_lock()
        .map_err(|_| ApiError {
            message: PROGRAM_DEPLOY_BUSY_MESSAGE.to_string(),
        })?;
    let expected_upgrade_authority = Pubkey::from_str(req.expected_upgrade_authority.trim())
        .map_err(|_| ApiError {
            message: "无效的预期 Upgrade Authority".to_string(),
        })?;

    let program_bytes = decode_program_binary_base64(&req.program_so_base64)?;

    let program_keypair_json = match (
        req.program_keypair_json.take(),
        req.program_keypair_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(json), None) => json,
        (None, Some(path)) => {
            let path = PathBuf::from(path)
                .canonicalize()
                .map_err(|error| ApiError {
                    message: format!("Program keypair 路径不可访问: {error}"),
                })?;
            read_text_file_limited(
                &path,
                program_deploy::MAX_PROGRAM_KEYPAIR_JSON_BYTES,
                "Program keypair",
            )?
        }
        (Some(mut json), Some(_)) => {
            json.zeroize();
            return Err(ApiError {
                message: "Program keypair JSON 与路径只能提供一个".to_string(),
            });
        }
        (None, None) => {
            return Err(ApiError {
                message: "必须提供 Program keypair JSON 或 Program keypair 路径".to_string(),
            });
        }
    };
    let mut program_keypair_json = Zeroizing::new(program_keypair_json);
    let program_keypair_result = program_deploy::parse_program_keypair_json(&program_keypair_json);
    program_keypair_json.zeroize();
    let program_keypair = program_keypair_result.map_err(|message| ApiError { message })?;
    let program_id =
        program_deploy::require_program_id(&program_keypair, req.expected_program_id.trim())
            .map_err(|message| ApiError { message })?;

    let program_sha256 =
        program_deploy::require_sha256(&program_bytes, &req.expected_program_sha256)
            .map_err(|message| ApiError { message })?;
    let program_bytes = verify_program_binary_offline(program_bytes).await?;

    let requested_resume_buffer_address = req
        .resume_buffer_address
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            Pubkey::from_str(value).map_err(|_| ApiError {
                message: "无效的恢复 Buffer 地址".to_string(),
            })
        })
        .transpose()?;
    let max_data_len = req.max_data_len.unwrap_or(program_bytes.len());
    if max_data_len < program_bytes.len() {
        return Err(ApiError {
            message: "最大 Program 数据长度不能小于 .so 文件大小".to_string(),
        });
    }
    if max_data_len > MAX_PROGRAM_SO_BYTES {
        return Err(ApiError {
            message: format!(
                "最大 Program 数据长度不能超过 {} MB",
                MAX_PROGRAM_SO_BYTES / 1024 / 1024
            ),
        });
    }

    if req.network.trim().is_empty() {
        return Err(ApiError {
            message: "部署网络不能为空".to_string(),
        });
    }
    let selector = rpc_selector(Some(req.network.trim()))?;
    let rpc_url = selector.url;
    let network = selector.network;
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url.to_string(),
        Duration::from_secs(PROGRAM_DEPLOY_RPC_TIMEOUT_SECS),
        CommitmentConfig::confirmed(),
    );

    let expected_genesis_hash = solana_sdk::hash::Hash::from_str(req.expected_genesis_hash.trim())
        .map_err(|_| ApiError {
            message: "无效的预期 genesis hash".to_string(),
        })?;
    let actual_genesis_hash = client.get_genesis_hash().map_err(|error| ApiError {
        message: format!("读取 RPC genesis hash 失败: {error}"),
    })?;
    if actual_genesis_hash != expected_genesis_hash {
        return Err(ApiError {
            message: format!(
                "RPC genesis hash 为 {}，预期为 {}；已在花费前中止部署",
                actual_genesis_hash, expected_genesis_hash
            ),
        });
    }

    let genesis_hash = actual_genesis_hash.to_string();
    let mut deployment_journal =
        wallet_store::find_program_deployment(&genesis_hash, &program_id.to_string())
            .map_err(|message| ApiError { message })?;
    let journal_buffer_address = deployment_journal
        .as_ref()
        .map(|record| {
            if !deployment_status_is_known(&record.status) {
                return Err(deployment_journal_error(format!(
                    "Program {} 的记录状态 {} 无法识别，拒绝继续",
                    program_id, record.status
                )));
            }
            validate_deployment_journal_binding(
                record,
                &genesis_hash,
                &program_id,
                &program_sha256,
                program_bytes.len(),
                max_data_len,
                &expected_upgrade_authority,
                requested_resume_buffer_address.as_ref(),
            )
        })
        .transpose()?;
    let deployment_attempts = if let Some(record) = deployment_journal.as_ref() {
        let attempts = wallet_store::load_program_deployment_attempts(
            &record.genesis_hash,
            &record.program_id,
        )
        .map_err(|message| ApiError { message })?;
        for attempt in &attempts {
            validate_program_deployment_attempt_record(record, attempt)?;
        }
        attempts
    } else {
        Vec::new()
    };

    let programdata_address = get_program_data_address(&program_id);
    let deployment_state_addresses = vec![program_id, programdata_address];
    let existing_accounts = client
        .get_multiple_accounts_with_commitment(
            &deployment_state_addresses,
            CommitmentConfig::finalized(),
        )
        .map_err(|error| ApiError {
            message: format!("部署前查询 Program 与 ProgramData 失败: {error}"),
        })?;
    let existing_finalized_slot = existing_accounts.context.slot;
    if existing_accounts.value.len() != deployment_state_addresses.len() {
        return Err(ApiError {
            message: "部署前 finalized 账户查询结果不完整，拒绝部署".to_string(),
        });
    }
    let mut existing_values = existing_accounts.value.into_iter();
    let existing_program = existing_values.next().flatten();
    let existing_programdata = existing_values.next().flatten();

    let existing_verified_readback = match (existing_program, existing_programdata) {
        (None, None) => None,
        (Some(program), Some(programdata)) => Some(
            program_deploy::verify_deployment_readback(
                &program_id,
                &expected_upgrade_authority,
                &program_bytes,
                max_data_len,
                &program.owner,
                program.executable,
                &program.data,
                &programdata.owner,
                &programdata.data,
            )
            .map_err(|error| ApiError {
                message: format!(
                    "目标 Program 已存在，但 finalized 状态与本次发布制品不一致，拒绝覆盖或升级: {error}"
                ),
            })?,
        ),
        _ => {
            return Err(ApiError {
                message: format!(
                    "目标 Program 与 ProgramData 只有一个存在（Program ID: {}），拒绝继续部署",
                    program_id
                ),
            })
        }
    };

    let buffer_lamports = client
        .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::size_of_buffer(
            program_bytes.len(),
        ))
        .map_err(|e| ApiError {
            message: format!("计算 buffer 租金失败: {}", e),
        })?;
    let program_lamports = client
        .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::size_of_program())
        .map_err(|e| ApiError {
            message: format!("计算 program 租金失败: {}", e),
        })?;
    let programdata_lamports = client
        .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::size_of_programdata(
            max_data_len,
        ))
        .map_err(|e| ApiError {
            message: format!("计算 ProgramData 租金失败: {}", e),
        })?;
    let final_rent_lamports = program_lamports
        .checked_add(programdata_lamports)
        .ok_or_else(|| ApiError {
            message: "Program 与 ProgramData 最终租金总额溢出".to_string(),
        })?;

    let wallet_auth = req.take_wallet_auth();
    let payer = run_keystore_task(move || wallet_auth.keypair()).await?;
    let payer_pubkey = payer.pubkey();
    if expected_upgrade_authority != payer_pubkey {
        return Err(ApiError {
            message: format!(
                "预期 Upgrade Authority 为 {}，但付款钱包为 {}；首次部署必须由该 authority 付款并持有升级权限",
                expected_upgrade_authority, payer_pubkey
            ),
        });
    }
    if let Some(readback) = existing_verified_readback {
        if readback.program_sha256 != program_sha256 {
            return Err(ApiError {
                message: "finalized Program SHA-256 与本地已验证哈希不一致".to_string(),
            });
        }
        for attempt in deployment_attempts.iter().filter(|attempt| {
            attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_DEPLOY
                && deployment_attempt_is_active(&attempt.status)
        }) {
            let completed_writes = deployment_journal
                .as_ref()
                .map(|record| record.completed_writes)
                .unwrap_or_default();
            transition_deployment_attempt(
                &mut deployment_journal,
                &attempt.signature,
                &attempt.status,
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
                DEPLOYMENT_STATUS_FINALIZED,
                completed_writes,
            )?;
        }
        if deployment_journal
            .as_ref()
            .is_some_and(|record| record.status != DEPLOYMENT_STATUS_FINALIZED)
        {
            let completed_writes = deployment_journal
                .as_ref()
                .map(|record| record.completed_writes)
                .unwrap_or_default();
            transition_deployment_journal(
                &mut deployment_journal,
                DEPLOYMENT_STATUS_FINALIZED,
                None,
                None,
                None,
                None,
                None,
                completed_writes,
                None,
                None,
            )?;
        }
        let (journal_revision, attempt_evidence_version, deployment_attempts) =
            load_deployment_receipt_evidence(deployment_journal.as_ref())?;
        let buffer_address = deployment_journal
            .as_ref()
            .map(|record| record.buffer_address.clone())
            .unwrap_or_default();
        let create_buffer_signature = deployment_journal
            .as_ref()
            .and_then(|record| record.create_signature.clone());
        let deploy_signature = deployment_journal
            .as_ref()
            .and_then(|record| record.deploy_signature.clone())
            .unwrap_or_default();
        let write_signatures = deployment_attempts
            .iter()
            .filter(|attempt| attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE)
            .map(|attempt| attempt.signature.clone())
            .collect();
        return Ok(Json(DeployProgramResponse {
            program_id: program_id.to_string(),
            programdata_address: programdata_address.to_string(),
            buffer_address,
            authority: readback.upgrade_authority.to_string(),
            network,
            genesis_hash,
            program_bytes: program_bytes.len(),
            max_data_len,
            program_sha256,
            temporary_buffer_rent_lamports: buffer_lamports,
            program_rent_lamports: program_lamports,
            programdata_rent_lamports: programdata_lamports,
            rent_lamports: final_rent_lamports,
            estimated_transaction_fees_lamports: 0,
            fee_rate_reserve_lamports: 0,
            recovery_write_reserve_lamports: 0,
            total_fee_budget_lamports: 0,
            estimated_required_balance_lamports: 0,
            create_buffer_signature,
            skipped_write_chunks: program_bytes.chunks(PROGRAM_WRITE_CHUNK_BYTES).count(),
            write_signatures,
            deploy_signature,
            finalized_slot: existing_finalized_slot,
            deployed_slot: readback.deployed_slot,
            readback_verified: true,
            journal_revision,
            attempt_evidence_version,
            deployment_attempts,
            status: "already_deployed_verified".to_string(),
        }));
    }

    let deploy_attempts = deployment_attempts
        .iter()
        .filter(|attempt| attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_DEPLOY)
        .collect::<Vec<_>>();
    for attempt in &deploy_attempts {
        if deployment_attempt_is_blocking_terminal(&attempt.status) {
            return Err(deployment_journal_error(format!(
                "deploy attempt {} 已处于终态 {}，但 finalized Program 不存在；禁止自动重签",
                attempt.signature, attempt.status
            )));
        }
        if deployment_attempt_is_active(&attempt.status) {
            let completed_writes = deployment_journal
                .as_ref()
                .map(|record| record.completed_writes)
                .unwrap_or_default();
            reconcile_absent_deployment_attempt(
                &client,
                &mut deployment_journal,
                attempt,
                DEPLOYMENT_STATUS_BUFFER_FINALIZED,
                completed_writes,
                |client, min_context_slot| {
                    confirm_deploy_attempt_still_absent(
                        client,
                        &program_id,
                        &programdata_address,
                        min_context_slot,
                    )
                },
            )?;
        }
    }
    if deployment_journal.as_ref().is_some_and(|record| {
        matches!(
            record.status.as_str(),
            DEPLOYMENT_STATUS_DEPLOY_SIGNED
                | DEPLOYMENT_STATUS_DEPLOY_RECONCILE
                | DEPLOYMENT_STATUS_DEPLOY_FINALIZED
                | DEPLOYMENT_STATUS_FINALIZED
        ) && deploy_attempts.is_empty()
    }) {
        return Err(deployment_journal_error(
            "journal 处于 deploy 状态但缺少完整 signature/last-valid-block-height attempt 证据",
        ));
    }

    let mut buffer_keypair = (journal_buffer_address.is_none()
        && requested_resume_buffer_address.is_none())
    .then(Keypair::new);
    let mut buffer_address = journal_buffer_address
        .or(requested_resume_buffer_address)
        .or_else(|| buffer_keypair.as_ref().map(|keypair| keypair.pubkey()))
        .ok_or_else(|| ApiError {
            message: "无法确定部署 Buffer 地址".to_string(),
        })?;
    let mut rotating_create_buffer = false;

    let (mut write_plan, resume_buffer_lamports) = if buffer_keypair.is_none() {
        let response = client
            .get_account_with_commitment(&buffer_address, CommitmentConfig::finalized())
            .map_err(|error| ApiError {
                message: format!("读取恢复 Buffer {} 失败: {error}", buffer_address),
            })?;
        if let Some(account) = response.value {
            if account.lamports < buffer_lamports {
                return Err(ApiError {
                    message: format!(
                        "恢复 Buffer {} 余额为 {} lamports，低于当前所需免租金额 {}",
                        buffer_address, account.lamports, buffer_lamports
                    ),
                });
            }
            let plan = program_deploy::verify_resume_buffer(
                &payer_pubkey,
                &program_bytes,
                PROGRAM_WRITE_CHUNK_BYTES,
                &account.owner,
                account.executable,
                &account.data,
            )
            .map_err(|error| ApiError {
                message: format!("恢复 Buffer {} 校验失败: {error}", buffer_address),
            })?;
            for attempt in deployment_attempts.iter().filter(|attempt| {
                attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_CREATE_BUFFER
                    && attempt.buffer_address == buffer_address.to_string()
                    && deployment_attempt_is_active(&attempt.status)
            }) {
                transition_deployment_attempt(
                    &mut deployment_journal,
                    &attempt.signature,
                    &attempt.status,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    plan.completed_chunks,
                )?;
            }
            (plan, account.lamports)
        } else {
            let record = deployment_journal.as_ref().ok_or_else(|| ApiError {
                message: format!(
                    "恢复 Buffer {} 在 finalized 状态下不存在，且没有可审计的 create attempt；拒绝自动创建新 Buffer",
                    buffer_address
                ),
            })?;
            if record.completed_writes != 0
                || deployment_attempts.iter().any(|attempt| {
                    matches!(
                        attempt.stage.as_str(),
                        wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE
                            | wallet_store::PROGRAM_DEPLOYMENT_STAGE_DEPLOY
                    )
                })
            {
                return Err(deployment_journal_error(format!(
                    "Buffer {} 不存在，但 journal 已包含 write/deploy 证据；拒绝轮换地址",
                    buffer_address
                )));
            }
            let create_attempts = deployment_attempts
                .iter()
                .filter(|attempt| {
                    attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_CREATE_BUFFER
                        && attempt.buffer_address == buffer_address.to_string()
                })
                .collect::<Vec<_>>();
            if let Some(attempt) = create_attempts
                .iter()
                .find(|attempt| deployment_attempt_is_blocking_terminal(&attempt.status))
            {
                return Err(deployment_journal_error(format!(
                    "create-buffer attempt {} 已处于终态 {}，但 finalized Buffer 不存在；禁止自动轮换",
                    attempt.signature, attempt.status
                )));
            }
            let active_create_attempts = create_attempts
                .iter()
                .filter(|attempt| deployment_attempt_is_active(&attempt.status))
                .copied()
                .collect::<Vec<_>>();
            if active_create_attempts.len() > 1 {
                return Err(deployment_journal_error(
                    "同一 Buffer 存在多个活动 create attempt，拒绝自动恢复",
                ));
            }
            if let Some(attempt) = active_create_attempts.first() {
                reconcile_absent_deployment_attempt(
                    &client,
                    &mut deployment_journal,
                    attempt,
                    DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE,
                    0,
                    |client, min_context_slot| {
                        confirm_create_buffer_attempt_still_absent(
                            client,
                            &buffer_address,
                            min_context_slot,
                        )
                    },
                )?;
            } else if !create_attempts.iter().any(|attempt| {
                attempt.status == wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EXPIRED_ABSENT
            }) {
                return Err(deployment_journal_error(format!(
                    "Buffer {} 不存在且缺少完整 create signature/last-valid-block-height 证据",
                    buffer_address
                )));
            }
            let replacement = Keypair::new();
            buffer_address = replacement.pubkey();
            buffer_keypair = Some(replacement);
            rotating_create_buffer = true;
            (
                program_deploy::BufferWritePlan {
                    completed_chunks: 0,
                    pending_chunk_indexes: (0..program_bytes
                        .chunks(PROGRAM_WRITE_CHUNK_BYTES)
                        .count())
                        .collect(),
                },
                0,
            )
        }
    } else {
        (
            program_deploy::BufferWritePlan {
                completed_chunks: 0,
                pending_chunk_indexes: (0..program_bytes.chunks(PROGRAM_WRITE_CHUNK_BYTES).count())
                    .collect(),
            },
            0,
        )
    };

    if buffer_keypair.is_none() {
        let chain_completed = write_plan.completed_chunks;
        let total_chunks = chain_completed
            .checked_add(write_plan.pending_chunk_indexes.len())
            .ok_or_else(|| deployment_journal_error("Buffer chunk 总数溢出"))?;
        if let Some(record) = deployment_journal.as_ref() {
            if record.attempt_evidence_version == 0 {
                if record.completed_writes > chain_completed {
                    return Err(deployment_journal_error(format!(
                        "旧 journal 的完成计数 {} 大于 finalized Buffer 的 {}；该旧值可能来自 confirmed 状态，拒绝自动迁移",
                        record.completed_writes, chain_completed
                    )));
                }
                match record.status.as_str() {
                    DEPLOYMENT_STATUS_WRITE_SIGNED
                    | DEPLOYMENT_STATUS_WRITE_CONFIRMED
                    | DEPLOYMENT_STATUS_WRITE_RECONCILE => {
                        let last_index = record.last_write_chunk_index.ok_or_else(|| {
                            deployment_journal_error(
                                "旧 write journal 缺少 chunk index，无法证明此前尝试均已落地",
                            )
                        })?;
                        if last_index >= total_chunks
                            || write_plan
                                .pending_chunk_indexes
                                .iter()
                                .any(|pending| *pending < last_index)
                        {
                            return Err(deployment_journal_error(
                                "旧 write journal 存在无法还原的早期 pending chunk，拒绝自动重签",
                            ));
                        }
                        if write_plan.pending_chunk_indexes.contains(&last_index)
                            && !deployment_attempts.iter().any(|attempt| {
                                attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE
                                    && attempt.chunk_index == Some(last_index)
                            })
                        {
                            return Err(deployment_journal_error(
                                "旧 write journal 的最后一个 pending chunk 缺少完整签名证据",
                            ));
                        }
                    }
                    DEPLOYMENT_STATUS_BUFFER_READY => {
                        if chain_completed < record.completed_writes {
                            return Err(deployment_journal_error(format!(
                                "finalized Buffer 仅完成 {} 块，少于旧 journal 的 {} 块",
                                chain_completed, record.completed_writes
                            )));
                        }
                    }
                    DEPLOYMENT_STATUS_BUFFER_FINALIZED
                    | DEPLOYMENT_STATUS_DEPLOY_SIGNED
                    | DEPLOYMENT_STATUS_DEPLOY_RECONCILE
                    | DEPLOYMENT_STATUS_DEPLOY_FINALIZED
                    | DEPLOYMENT_STATUS_FINALIZED => {
                        if !write_plan.pending_chunk_indexes.is_empty() {
                            return Err(deployment_journal_error(
                                "旧 journal 声称 Buffer 已 finalized，但回读仍存在 pending chunk",
                            ));
                        }
                    }
                    DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED
                    | DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE => {
                        if record.completed_writes != 0 {
                            return Err(deployment_journal_error(
                                "旧 create-buffer journal 含非零写入进度，拒绝迁移",
                            ));
                        }
                    }
                    _ => {}
                }
            } else if chain_completed < record.completed_writes {
                return Err(deployment_journal_error(format!(
                    "finalized Buffer 仅完成 {} 块，少于 journal 的 {} 块",
                    chain_completed, record.completed_writes
                )));
            }
        }

        for attempt in deployment_attempts
            .iter()
            .filter(|attempt| attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE)
        {
            let index = attempt
                .chunk_index
                .ok_or_else(|| deployment_journal_error("write attempt 缺少 chunk index"))?;
            if index >= total_chunks {
                return Err(deployment_journal_error(format!(
                    "write attempt {} 的 chunk index {} 超出总块数 {}",
                    attempt.signature, index, total_chunks
                )));
            }
            let pending = write_plan.pending_chunk_indexes.contains(&index);
            if pending && deployment_attempt_is_blocking_terminal(&attempt.status) {
                return Err(deployment_journal_error(format!(
                    "write chunk {} 的 attempt {} 已处于终态 {}，但 finalized Buffer 仍未包含该块",
                    index, attempt.signature, attempt.status
                )));
            }
            if !deployment_attempt_is_active(&attempt.status) {
                continue;
            }
            let completed_for_transition = deployment_journal
                .as_ref()
                .map(|_| chain_completed)
                .unwrap_or(chain_completed);
            if pending {
                reconcile_absent_deployment_attempt(
                    &client,
                    &mut deployment_journal,
                    attempt,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    completed_for_transition,
                    |client, min_context_slot| {
                        confirm_write_attempt_still_absent(
                            client,
                            &buffer_address,
                            &payer_pubkey,
                            &program_bytes,
                            &write_plan,
                            index,
                            min_context_slot,
                        )
                    },
                )?;
            } else {
                transition_deployment_attempt(
                    &mut deployment_journal,
                    &attempt.signature,
                    &attempt.status,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    completed_for_transition,
                )?;
            }
        }

        if let Some(record) = deployment_journal.as_ref() {
            if record.attempt_evidence_version == 0 {
                let promoted = wallet_store::promote_program_deployment_attempt_evidence(
                    record,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    chain_completed,
                )
                .map_err(|message| ApiError { message })?;
                deployment_journal = Some(promoted);
            } else if record.completed_writes != chain_completed
                || matches!(
                    record.status.as_str(),
                    DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED
                        | DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE
                        | DEPLOYMENT_STATUS_WRITE_SIGNED
                        | DEPLOYMENT_STATUS_WRITE_CONFIRMED
                        | DEPLOYMENT_STATUS_WRITE_RECONCILE
                        | DEPLOYMENT_STATUS_DEPLOY_SIGNED
                        | DEPLOYMENT_STATUS_DEPLOY_RECONCILE
                        | DEPLOYMENT_STATUS_DEPLOY_FINALIZED
                )
            {
                transition_deployment_journal(
                    &mut deployment_journal,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    None,
                    None,
                    None,
                    None,
                    None,
                    chain_completed,
                    None,
                    None,
                )?;
            }
        }
    }

    let create_buffer_ixs = if buffer_keypair.is_some() {
        Some(
            loader_v3_instruction::create_buffer(
                &payer_pubkey,
                &buffer_address,
                &payer_pubkey,
                buffer_lamports,
                program_bytes.len(),
            )
            .map_err(|e| ApiError {
                message: format!("创建 buffer 指令失败: {}", e),
            })?,
        )
    } else {
        None
    };

    #[allow(deprecated)]
    let deploy_ixs = loader_v3_instruction::deploy_with_max_program_len(
        &payer_pubkey,
        &program_id,
        &buffer_address,
        &payer_pubkey,
        program_lamports,
        max_data_len,
    )
    .map_err(|e| ApiError {
        message: format!("创建 deploy 指令失败: {}", e),
    })?;

    let fee_blockhash = client.get_latest_blockhash().map_err(|error| ApiError {
        message: format!("获取交易费估算 blockhash 失败: {error}"),
    })?;
    let create_buffer_fee = create_buffer_ixs
        .as_deref()
        .map(|instructions| {
            estimate_instruction_fee(
                &client,
                instructions,
                &payer_pubkey,
                &fee_blockhash,
                "create-buffer",
            )
        })
        .transpose()?
        .unwrap_or(0);
    let mut write_fee_by_chunk_len = HashMap::<usize, u64>::new();
    let mut write_fees = 0u64;
    let mut estimated_write_fee_lamports = 0u64;
    for index in &write_plan.pending_chunk_indexes {
        let start = index
            .checked_mul(PROGRAM_WRITE_CHUNK_BYTES)
            .ok_or_else(|| ApiError {
                message: "Program 写入偏移超出范围".to_string(),
            })?;
        let chunk = program_bytes
            .get(
                start
                    ..start
                        .saturating_add(PROGRAM_WRITE_CHUNK_BYTES)
                        .min(program_bytes.len()),
            )
            .ok_or_else(|| ApiError {
                message: "Program 写入块超出制品范围".to_string(),
            })?;
        let fee = if let Some(fee) = write_fee_by_chunk_len.get(&chunk.len()) {
            *fee
        } else {
            let offset = u32::try_from(start).map_err(|_| ApiError {
                message: "Program 写入偏移超出范围".to_string(),
            })?;
            let instruction = loader_v3_instruction::write(
                &buffer_address,
                &payer_pubkey,
                offset,
                chunk.to_vec(),
            );
            let fee = estimate_instruction_fee(
                &client,
                &[instruction],
                &payer_pubkey,
                &fee_blockhash,
                "write-buffer",
            )?;
            write_fee_by_chunk_len.insert(chunk.len(), fee);
            fee
        };
        estimated_write_fee_lamports = estimated_write_fee_lamports.max(fee);
        write_fees = write_fees.checked_add(fee).ok_or_else(|| ApiError {
            message: "Program 写入交易费总额溢出".to_string(),
        })?;
    }
    let deploy_fee = estimate_instruction_fee(
        &client,
        &deploy_ixs,
        &payer_pubkey,
        &fee_blockhash,
        "deploy",
    )?;
    let estimated_transaction_fees_lamports = create_buffer_fee
        .checked_add(write_fees)
        .and_then(|value| value.checked_add(deploy_fee))
        .ok_or_else(|| ApiError {
            message: "部署交易费总额溢出".to_string(),
        })?;
    let fee_budget = program_deploy::deployment_fee_budget(
        estimated_transaction_fees_lamports,
        estimated_write_fee_lamports,
        write_plan.pending_chunk_indexes.len(),
    )
    .map_err(|message| ApiError { message })?;
    let rent_budget = program_deploy::deployment_rent_budget(
        buffer_lamports,
        program_lamports,
        programdata_lamports,
        buffer_keypair.is_none().then_some(resume_buffer_lamports),
    )
    .map_err(|message| ApiError { message })?;
    let required_rent_balance = rent_budget.required_rent_balance_lamports;
    let estimated_required_balance_lamports = required_rent_balance
        .checked_add(fee_budget.total_fee_budget_lamports)
        .ok_or_else(|| ApiError {
            message: "部署所需余额总额溢出".to_string(),
        })?;
    let payer_balance = client
        .get_balance(&payer_pubkey)
        .map_err(|error| ApiError {
            message: format!("查询部署钱包余额失败: {error}"),
        })?;
    if payer_balance < estimated_required_balance_lamports {
        return Err(ApiError {
            message: format!(
                "部署钱包余额不足：当前 {} lamports，至少需要 {} lamports（租金需求 {} + 当前估算交易费 {} + 费率预留 {} + 恢复写入预留 {}）",
                payer_balance,
                estimated_required_balance_lamports,
                required_rent_balance,
                estimated_transaction_fees_lamports,
                fee_budget.fee_rate_reserve_lamports,
                fee_budget.recovery_write_reserve_lamports
            ),
        });
    }

    if deployment_journal.is_none() && buffer_keypair.is_none() {
        let record = wallet_store::ProgramDeploymentRecord {
            genesis_hash: genesis_hash.clone(),
            program_id: program_id.to_string(),
            program_sha256: program_sha256.clone(),
            program_len: program_bytes.len(),
            max_data_len,
            upgrade_authority: expected_upgrade_authority.to_string(),
            buffer_address: buffer_address.to_string(),
            status: DEPLOYMENT_STATUS_BUFFER_READY.to_string(),
            create_signature: None,
            create_last_valid_block_height: None,
            last_write_signature: None,
            last_write_chunk_index: None,
            last_write_last_valid_block_height: None,
            completed_writes: write_plan.completed_chunks,
            deploy_signature: None,
            deploy_last_valid_block_height: None,
            attempt_evidence_version: wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EVIDENCE_VERSION,
            revision: 0,
            created_at: 0,
            updated_at: 0,
        };
        let (stored, inserted) = wallet_store::reserve_program_deployment(record)
            .map_err(|message| ApiError { message })?;
        if !inserted {
            validate_deployment_journal_binding(
                &stored,
                &genesis_hash,
                &program_id,
                &program_sha256,
                program_bytes.len(),
                max_data_len,
                &expected_upgrade_authority,
                Some(&buffer_address),
            )?;
            return Err(deployment_journal_error(format!(
                "另一部署请求已抢先预留 Buffer {}；当前请求未发送交易，请重新进入恢复流程",
                stored.buffer_address
            )));
        }
        deployment_journal = Some(stored);
    }

    if let (Some(instructions), Some(buffer_signer)) =
        (create_buffer_ixs.as_ref(), buffer_keypair.as_ref())
    {
        let (blockhash, last_valid_block_height) = client
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            .map_err(|error| ApiError {
                message: format!("获取 create-buffer blockhash 失败: {error}"),
            })?;
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&payer_pubkey),
            &[&payer, buffer_signer],
            blockhash,
        );
        let local_signature = transaction
            .signatures
            .first()
            .copied()
            .ok_or_else(|| ApiError {
                message: "create-buffer 交易缺少本地签名".to_string(),
            })?;
        let record = wallet_store::ProgramDeploymentRecord {
            genesis_hash: genesis_hash.clone(),
            program_id: program_id.to_string(),
            program_sha256: program_sha256.clone(),
            program_len: program_bytes.len(),
            max_data_len,
            upgrade_authority: expected_upgrade_authority.to_string(),
            buffer_address: buffer_address.to_string(),
            status: DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED.to_string(),
            create_signature: Some(local_signature.to_string()),
            create_last_valid_block_height: Some(last_valid_block_height),
            last_write_signature: None,
            last_write_chunk_index: None,
            last_write_last_valid_block_height: None,
            completed_writes: 0,
            deploy_signature: None,
            deploy_last_valid_block_height: None,
            attempt_evidence_version: wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_EVIDENCE_VERSION,
            revision: 0,
            created_at: 0,
            updated_at: 0,
        };
        let create_attempt = new_program_deployment_attempt(
            &record,
            wallet_store::PROGRAM_DEPLOYMENT_STAGE_CREATE_BUFFER,
            None,
            &local_signature,
            last_valid_block_height,
        );
        if rotating_create_buffer {
            let current = deployment_journal
                .as_ref()
                .ok_or_else(|| deployment_journal_error("Buffer 轮换时部署 journal 不存在"))?;
            let stored = wallet_store::rotate_program_deployment_create_attempt(
                current,
                create_attempt,
                DEPLOYMENT_STATUS_CREATE_BUFFER_SIGNED,
            )
            .map_err(|message| ApiError { message })?;
            deployment_journal = Some(stored);
        } else {
            let (stored, inserted) =
                wallet_store::reserve_program_deployment_with_attempt(record, create_attempt)
                    .map_err(|message| ApiError { message })?;
            if !inserted {
                validate_deployment_journal_binding(
                    &stored,
                    &genesis_hash,
                    &program_id,
                    &program_sha256,
                    program_bytes.len(),
                    max_data_len,
                    &expected_upgrade_authority,
                    None,
                )?;
                return Err(deployment_journal_error(format!(
                    "另一部署请求已抢先预留 Buffer {}；当前 Buffer {} 的本地签名交易尚未发送，请重新进入恢复流程",
                    stored.buffer_address, buffer_address
                )));
            }
            deployment_journal = Some(stored);
        }
        let context = format!("Program ID: {program_id}；Buffer: {buffer_address}");
        let submission = submit_signed_transaction_once(
            &client,
            &transaction,
            "创建 Buffer",
            &context,
            CommitmentConfig::finalized(),
            "finalized",
            Duration::from_secs(90),
        )
        .await;
        match submission {
            Ok((signature, _)) => {
                if signature != local_signature {
                    return Err(deployment_journal_error(
                        "RPC 返回的 create-buffer 签名与本地已持久化签名不一致",
                    ));
                }
                transition_deployment_attempt(
                    &mut deployment_journal,
                    &local_signature.to_string(),
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED,
                    DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE,
                    0,
                )?;
                let finalized_account = client
                    .get_account_with_commitment(&buffer_address, CommitmentConfig::finalized())
                    .map_err(|error| ApiError {
                        message: format!(
                            "create-buffer 签名已 finalized，但回读 Buffer {} 失败: {error}",
                            buffer_address
                        ),
                    })?
                    .value
                    .ok_or_else(|| {
                        deployment_journal_error(format!(
                            "create-buffer 签名 {} 已 finalized，但 Buffer {} 不存在",
                            local_signature, buffer_address
                        ))
                    })?;
                write_plan = program_deploy::verify_resume_buffer(
                    &payer_pubkey,
                    &program_bytes,
                    PROGRAM_WRITE_CHUNK_BYTES,
                    &finalized_account.owner,
                    finalized_account.executable,
                    &finalized_account.data,
                )
                .map_err(|error| {
                    deployment_journal_error(format!(
                        "create-buffer 签名已 finalized，但 Buffer 回读校验失败: {error}"
                    ))
                })?;
                transition_deployment_attempt(
                    &mut deployment_journal,
                    &local_signature.to_string(),
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
                    DEPLOYMENT_STATUS_BUFFER_READY,
                    write_plan.completed_chunks,
                )?;
            }
            Err(error) => {
                let journal_error = transition_deployment_attempt(
                    &mut deployment_journal,
                    &local_signature.to_string(),
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_REQUIRES_RECONCILIATION,
                    DEPLOYMENT_STATUS_CREATE_BUFFER_RECONCILE,
                    0,
                )
                .err()
                .map(|journal_error| format!("；journal 更新失败: {}", journal_error.message))
                .unwrap_or_default();
                return Err(ApiError {
                    message: format!(
                        "{}；Buffer {} 已记录，本地签名 {}，last valid block height {}{}",
                        error.message,
                        buffer_address,
                        local_signature,
                        last_valid_block_height,
                        journal_error
                    ),
                });
            }
        }
    }

    let create_buffer_signature = deployment_journal
        .as_ref()
        .and_then(|record| record.create_signature.clone());

    let mut write_signatures = Vec::with_capacity(write_plan.pending_chunk_indexes.len());
    let completed_writes = write_plan.completed_chunks;
    let total_pending_writes = write_plan.pending_chunk_indexes.len();
    for (position, index) in write_plan.pending_chunk_indexes.iter().copied().enumerate() {
        let start = index
            .checked_mul(PROGRAM_WRITE_CHUNK_BYTES)
            .ok_or_else(|| ApiError {
                message: "Program 写入偏移超出范围".to_string(),
            })?;
        let end = start
            .saturating_add(PROGRAM_WRITE_CHUNK_BYTES)
            .min(program_bytes.len());
        let chunk = program_bytes.get(start..end).ok_or_else(|| ApiError {
            message: "Program 写入块超出制品范围".to_string(),
        })?;
        let offset = u32::try_from(start).map_err(|_| ApiError {
            message: "Program 写入偏移超出范围".to_string(),
        })?;
        let write_ix =
            loader_v3_instruction::write(&buffer_address, &payer_pubkey, offset, chunk.to_vec());
        let (blockhash, write_last_valid_block_height) = client
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            .map_err(|error| ApiError {
                message: format!("获取 write-buffer blockhash 失败: {error}"),
            })?;
        let transaction = Transaction::new_signed_with_payer(
            &[write_ix],
            Some(&payer_pubkey),
            &[&payer],
            blockhash,
        );
        let local_signature = transaction
            .signatures
            .first()
            .copied()
            .ok_or_else(|| ApiError {
                message: "write-buffer 交易缺少本地签名".to_string(),
            })?;
        let attempt = new_program_deployment_attempt(
            deployment_journal.as_ref().ok_or_else(|| ApiError {
                message: "部署 journal 不存在，拒绝写入 Buffer".to_string(),
            })?,
            wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE,
            Some(index),
            &local_signature,
            write_last_valid_block_height,
        );
        begin_deployment_attempt(
            &mut deployment_journal,
            attempt,
            DEPLOYMENT_STATUS_WRITE_SIGNED,
            completed_writes,
        )?;
        let context = format!(
            "Program ID: {program_id}；Buffer: {buffer_address}；写入进度: {}/{}；chunk index: {}",
            position + 1,
            total_pending_writes,
            index
        );
        let submission = submit_signed_transaction_once(
            &client,
            &transaction,
            "写入 Buffer",
            &context,
            CommitmentConfig::confirmed(),
            "confirmed",
            Duration::from_secs(60),
        )
        .await;
        match submission {
            Ok((signature, _)) => {
                if signature != local_signature {
                    return Err(deployment_journal_error(format!(
                        "write chunk {} 的 RPC 签名与本地已持久化签名不一致",
                        index
                    )));
                }
                transition_deployment_attempt(
                    &mut deployment_journal,
                    &local_signature.to_string(),
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED,
                    DEPLOYMENT_STATUS_WRITE_CONFIRMED,
                    completed_writes,
                )?;
                write_signatures.push(signature.to_string());
            }
            Err(error) => {
                let journal_error = transition_deployment_attempt(
                    &mut deployment_journal,
                    &local_signature.to_string(),
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
                    wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_REQUIRES_RECONCILIATION,
                    DEPLOYMENT_STATUS_WRITE_RECONCILE,
                    completed_writes,
                )
                .err()
                .map(|journal_error| format!("；journal 更新失败: {}", journal_error.message))
                .unwrap_or_default();
                return Err(ApiError {
                    message: format!(
                        "{}；Buffer {} 的 chunk {} 已记录本地签名 {}{}",
                        error.message, buffer_address, index, local_signature, journal_error
                    ),
                });
            }
        }
    }

    let buffer_finalized_deadline = Instant::now() + Duration::from_secs(120);
    let finalized_completed_writes = loop {
        let response = client
            .get_account_with_commitment(&buffer_address, CommitmentConfig::finalized())
            .map_err(|error| ApiError {
                message: format!(
                    "部署前 finalized 回读 Buffer {} 失败: {error}",
                    buffer_address
                ),
            })?;
        let account = response.value.ok_or_else(|| ApiError {
            message: format!("部署前 Buffer {} 不存在", buffer_address),
        })?;
        let finalized_plan = program_deploy::verify_resume_buffer(
            &payer_pubkey,
            &program_bytes,
            PROGRAM_WRITE_CHUNK_BYTES,
            &account.owner,
            account.executable,
            &account.data,
        )
        .map_err(|error| ApiError {
            message: format!("部署前 Buffer {} 回读校验失败: {error}", buffer_address),
        })?;
        if finalized_plan.pending_chunk_indexes.is_empty() {
            break finalized_plan.completed_chunks;
        }
        if Instant::now() >= buffer_finalized_deadline {
            return Err(ApiError {
                message: format!(
                    "Buffer {} 仍有 {} 个块未 finalized；请保留该地址并使用恢复部署，不能盲目创建新 Buffer",
                    buffer_address,
                    finalized_plan.pending_chunk_indexes.len()
                ),
            });
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    let current_journal = deployment_journal.as_ref().ok_or_else(|| ApiError {
        message: "部署 journal 不存在，拒绝完成 Buffer 回读".to_string(),
    })?;
    let active_attempts = wallet_store::load_active_program_deployment_attempts(
        &current_journal.genesis_hash,
        &current_journal.program_id,
    )
    .map_err(|message| ApiError { message })?;
    for attempt in active_attempts
        .iter()
        .filter(|attempt| attempt.stage == wallet_store::PROGRAM_DEPLOYMENT_STAGE_WRITE)
    {
        transition_deployment_attempt(
            &mut deployment_journal,
            &attempt.signature,
            &attempt.status,
            wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
            DEPLOYMENT_STATUS_WRITE_CONFIRMED,
            finalized_completed_writes,
        )?;
    }
    transition_deployment_journal(
        &mut deployment_journal,
        DEPLOYMENT_STATUS_BUFFER_FINALIZED,
        None,
        None,
        None,
        None,
        None,
        finalized_completed_writes,
        None,
        None,
    )?;

    let (blockhash, deploy_last_valid_block_height) = client
        .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        .map_err(|error| ApiError {
            message: format!("获取 deploy blockhash 失败: {error}"),
        })?;
    let deploy_transaction = Transaction::new_signed_with_payer(
        &deploy_ixs,
        Some(&payer_pubkey),
        &[&payer, &program_keypair],
        blockhash,
    );
    let local_deploy_signature =
        deploy_transaction
            .signatures
            .first()
            .copied()
            .ok_or_else(|| ApiError {
                message: "deploy 交易缺少本地签名".to_string(),
            })?;
    simulate_program_deployment_transaction(&client, &deploy_transaction, "deploy")?;
    let deploy_attempt = new_program_deployment_attempt(
        deployment_journal.as_ref().ok_or_else(|| ApiError {
            message: "部署 journal 不存在，拒绝记录 deploy attempt".to_string(),
        })?,
        wallet_store::PROGRAM_DEPLOYMENT_STAGE_DEPLOY,
        None,
        &local_deploy_signature,
        deploy_last_valid_block_height,
    );
    begin_deployment_attempt(
        &mut deployment_journal,
        deploy_attempt,
        DEPLOYMENT_STATUS_DEPLOY_SIGNED,
        finalized_completed_writes,
    )?;
    let deploy_context = format!("Program ID: {program_id}；Buffer: {buffer_address}");
    let deploy_submission = submit_signed_transaction_once(
        &client,
        &deploy_transaction,
        "部署",
        &deploy_context,
        CommitmentConfig::finalized(),
        "finalized",
        Duration::from_secs(90),
    )
    .await;
    let (deploy_signature, finalized_slot) = match deploy_submission {
        Ok(result) => result,
        Err(error) => {
            let journal_error = transition_deployment_attempt(
                &mut deployment_journal,
                &local_deploy_signature.to_string(),
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
                wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_REQUIRES_RECONCILIATION,
                DEPLOYMENT_STATUS_DEPLOY_RECONCILE,
                finalized_completed_writes,
            )
            .err()
            .map(|journal_error| format!("；journal 更新失败: {}", journal_error.message))
            .unwrap_or_default();
            return Err(ApiError {
                message: format!(
                    "{}；Program {} 已记录本地部署签名 {}，last valid block height {}{}",
                    error.message,
                    program_id,
                    local_deploy_signature,
                    deploy_last_valid_block_height,
                    journal_error
                ),
            });
        }
    };
    if deploy_signature != local_deploy_signature {
        return Err(deployment_journal_error(
            "RPC 返回的 deploy 签名与本地已持久化签名不一致",
        ));
    }
    transition_deployment_attempt(
        &mut deployment_journal,
        &local_deploy_signature.to_string(),
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_SIGNED,
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED,
        DEPLOYMENT_STATUS_DEPLOY_FINALIZED,
        finalized_completed_writes,
    )?;
    let readback = wait_for_finalized_deployment_readback(
        &client,
        &program_id,
        &programdata_address,
        &expected_upgrade_authority,
        &program_bytes,
        max_data_len,
        finalized_slot,
    )
    .await
    .map_err(|error| ApiError {
        message: format!(
            "{}；Program ID: {}，部署签名: {}",
            error.message, program_id, deploy_signature
        ),
    })?;
    if readback.program_sha256 != program_sha256 {
        return Err(ApiError {
            message: "finalized 回读 SHA-256 与本地已验证哈希不一致".to_string(),
        });
    }
    transition_deployment_attempt(
        &mut deployment_journal,
        &local_deploy_signature.to_string(),
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_CONFIRMED,
        wallet_store::PROGRAM_DEPLOYMENT_ATTEMPT_FINALIZED,
        DEPLOYMENT_STATUS_FINALIZED,
        finalized_completed_writes,
    )?;
    let (journal_revision, attempt_evidence_version, deployment_attempts) =
        load_deployment_receipt_evidence(deployment_journal.as_ref())?;

    Ok(Json(DeployProgramResponse {
        program_id: program_id.to_string(),
        programdata_address: programdata_address.to_string(),
        buffer_address: buffer_address.to_string(),
        authority: readback.upgrade_authority.to_string(),
        network,
        genesis_hash,
        program_bytes: program_bytes.len(),
        max_data_len,
        program_sha256,
        temporary_buffer_rent_lamports: buffer_lamports,
        program_rent_lamports: program_lamports,
        programdata_rent_lamports: programdata_lamports,
        rent_lamports: final_rent_lamports,
        estimated_transaction_fees_lamports,
        fee_rate_reserve_lamports: fee_budget.fee_rate_reserve_lamports,
        recovery_write_reserve_lamports: fee_budget.recovery_write_reserve_lamports,
        total_fee_budget_lamports: fee_budget.total_fee_budget_lamports,
        estimated_required_balance_lamports,
        create_buffer_signature,
        skipped_write_chunks: write_plan.completed_chunks,
        write_signatures,
        deploy_signature: deploy_signature.to_string(),
        finalized_slot,
        deployed_slot: readback.deployed_slot,
        readback_verified: true,
        journal_revision,
        attempt_evidence_version,
        deployment_attempts,
        status: "finalized".to_string(),
    }))
}

#[derive(Deserialize)]
struct ProgramInfoRequest {
    program_id: String,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct ProgramInfoResponse {
    program_id: String,
    programdata_address: String,
    exists: bool,
    executable: bool,
    owner: String,
    lamports: u64,
    data_len: usize,
    network: String,
}

async fn program_info(
    Json(req): Json<ProgramInfoRequest>,
) -> Result<Json<ProgramInfoResponse>, ApiError> {
    let program_id = Pubkey::from_str(req.program_id.trim()).map_err(|_| ApiError {
        message: "无效的 Program ID".to_string(),
    })?;
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let network = network_name(req.network.as_deref());
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let programdata_address = get_program_data_address(&program_id);
    let Some(account) = client
        .get_account_with_commitment(&program_id, CommitmentConfig::confirmed())
        .map_err(|e| ApiError {
            message: format!("查询 Program 失败: {}", e),
        })?
        .value
    else {
        return Ok(Json(ProgramInfoResponse {
            program_id: program_id.to_string(),
            programdata_address: programdata_address.to_string(),
            exists: false,
            executable: false,
            owner: "".to_string(),
            lamports: 0,
            data_len: 0,
            network,
        }));
    };

    Ok(Json(ProgramInfoResponse {
        program_id: program_id.to_string(),
        programdata_address: programdata_address.to_string(),
        exists: true,
        executable: account.executable,
        owner: account.owner.to_string(),
        lamports: account.lamports,
        data_len: account.data.len(),
        network,
    }))
}

// ============= Squads v4 Multisig =============

#[derive(Deserialize)]
struct SquadsCreateRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    members: Vec<String>,
    threshold: u16,
    #[serde(default)]
    time_lock: Option<u32>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct SquadsMemberResponse {
    key: String,
    permissions: u8,
}

#[derive(Serialize)]
struct SquadsCreateResponse {
    multisig: String,
    vault: String,
    create_key: String,
    signature: String,
    threshold: u16,
    members: Vec<SquadsMemberResponse>,
    creation_fee_lamports: u64,
    network: String,
    status: String,
}

async fn squads_create(
    Json(req): Json<SquadsCreateRequest>,
) -> Result<Json<SquadsCreateResponse>, ApiError> {
    if req.members.is_empty() {
        return Err(ApiError {
            message: "至少需要一个多签成员".to_string(),
        });
    }
    let mut members = req
        .members
        .iter()
        .map(|member| {
            squads_v4::parse_pubkey(member, "成员地址").map(|key| SquadsMember {
                key,
                permissions: SquadsPermissions::all(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|message| ApiError { message })?;
    members.sort_by_key(|member| member.key);
    members.dedup_by_key(|member| member.key);

    if req.threshold == 0 || usize::from(req.threshold) > members.len() {
        return Err(ApiError {
            message: "阈值必须大于 0 且不能超过成员数量".to_string(),
        });
    }

    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }

    let payer = req.wallet.keypair()?;
    if !members.iter().any(|member| member.key == payer.pubkey()) {
        return Err(ApiError {
            message: "当前签名钱包必须是多签成员".to_string(),
        });
    }
    let create_key = Keypair::new();
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let program_config = load_squads_program_config(&client)?;
    let (ix, multisig) = squads_v4::multisig_create_ix(
        &create_key.pubkey(),
        &payer.pubkey(),
        &program_config.treasury,
        req.threshold,
        members.clone(),
        req.time_lock.unwrap_or(0),
        req.memo,
    )
    .map_err(|message| ApiError { message })?;
    let signature = sign_and_send(&client, vec![ix], &[&payer, &create_key], &payer.pubkey())?;
    let vault = squads_v4::vault_pda(&multisig, 0);

    Ok(Json(SquadsCreateResponse {
        multisig: multisig.to_string(),
        vault: vault.to_string(),
        create_key: create_key.pubkey().to_string(),
        signature,
        threshold: req.threshold,
        members: members
            .into_iter()
            .map(|member| SquadsMemberResponse {
                key: member.key.to_string(),
                permissions: member.permissions.mask,
            })
            .collect(),
        creation_fee_lamports: program_config.multisig_creation_fee,
        network,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct SquadsInfoRequest {
    multisig: String,
    #[serde(default)]
    proposal: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct SquadsInfoResponse {
    multisig: String,
    vault: String,
    create_key: String,
    threshold: u16,
    time_lock: u32,
    transaction_index: u64,
    stale_transaction_index: u64,
    members: Vec<SquadsMemberResponse>,
    proposal: Option<SquadsProposalResponse>,
    network: String,
}

#[derive(Serialize)]
struct SquadsProposalResponse {
    address: String,
    transaction_index: u64,
    status: String,
    approved: Vec<String>,
    rejected: Vec<String>,
    cancelled: Vec<String>,
}

async fn squads_info(
    Json(req): Json<SquadsInfoRequest>,
) -> Result<Json<SquadsInfoResponse>, ApiError> {
    let multisig_key = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig = load_squads_multisig(&client, &multisig_key)?;
    let proposal = if let Some(proposal) = req
        .proposal
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let proposal_key = squads_v4::parse_pubkey(proposal, "提案地址")
            .map_err(|message| ApiError { message })?;
        let proposal = load_squads_proposal(&client, &proposal_key)?;
        Some(SquadsProposalResponse {
            address: proposal_key.to_string(),
            transaction_index: proposal.transaction_index,
            status: proposal.status.label().to_string(),
            approved: proposal
                .approved
                .into_iter()
                .map(|key| key.to_string())
                .collect(),
            rejected: proposal
                .rejected
                .into_iter()
                .map(|key| key.to_string())
                .collect(),
            cancelled: proposal
                .cancelled
                .into_iter()
                .map(|key| key.to_string())
                .collect(),
        })
    } else {
        None
    };

    Ok(Json(SquadsInfoResponse {
        multisig: multisig_key.to_string(),
        vault: squads_v4::vault_pda(&multisig_key, 0).to_string(),
        create_key: multisig.create_key.to_string(),
        threshold: multisig.threshold,
        time_lock: multisig.time_lock,
        transaction_index: multisig.transaction_index,
        stale_transaction_index: multisig.stale_transaction_index,
        members: multisig
            .members
            .into_iter()
            .map(|member| SquadsMemberResponse {
                key: member.key.to_string(),
                permissions: member.permissions.mask,
            })
            .collect(),
        proposal,
        network,
    }))
}

#[derive(Deserialize)]
struct SquadsProposalsRequest {
    multisig: String,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    limit: Option<u64>,
}

#[derive(Serialize)]
struct SquadsProposalsResponse {
    multisig: String,
    vault: String,
    proposals: Vec<SquadsProposalResponse>,
    latest_transaction_index: u64,
    network: String,
}

async fn squads_proposals(
    Json(req): Json<SquadsProposalsRequest>,
) -> Result<Json<SquadsProposalsResponse>, ApiError> {
    let multisig_key = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig = load_squads_multisig(&client, &multisig_key)?;
    let limit = req.limit.unwrap_or(20).clamp(1, 50);
    let start = multisig.transaction_index;
    let end = start.saturating_sub(limit.saturating_sub(1));
    let mut proposals = Vec::new();

    for index in (end..=start).rev() {
        let proposal_key = squads_v4::proposal_pda(&multisig_key, index);
        if let Ok(proposal) = load_squads_proposal(&client, &proposal_key) {
            proposals.push(SquadsProposalResponse {
                address: proposal_key.to_string(),
                transaction_index: proposal.transaction_index,
                status: proposal.status.label().to_string(),
                approved: proposal
                    .approved
                    .into_iter()
                    .map(|key| key.to_string())
                    .collect(),
                rejected: proposal
                    .rejected
                    .into_iter()
                    .map(|key| key.to_string())
                    .collect(),
                cancelled: proposal
                    .cancelled
                    .into_iter()
                    .map(|key| key.to_string())
                    .collect(),
            });
        }
    }

    Ok(Json(SquadsProposalsResponse {
        multisig: multisig_key.to_string(),
        vault: squads_v4::vault_pda(&multisig_key, 0).to_string(),
        proposals,
        latest_transaction_index: multisig.transaction_index,
        network,
    }))
}

#[derive(Serialize)]
struct SquadsProposalCreateResponse {
    multisig: String,
    vault: String,
    transaction: String,
    proposal: String,
    transaction_index: u64,
    signature: String,
    network: String,
    status: String,
}

#[derive(Deserialize)]
struct SquadsSolTransferProposalRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    to_address: String,
    amount: DecimalAmount,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

async fn squads_sol_transfer_proposal(
    Json(req): Json<SquadsSolTransferProposalRequest>,
) -> Result<Json<SquadsProposalCreateResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig_key = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let to_pubkey = squads_v4::parse_pubkey(&req.to_address, "接收地址")
        .map_err(|message| ApiError { message })?;
    let amount_lamports = sol_to_lamports(&req.amount)?;
    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }

    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig = load_squads_multisig(&client, &multisig_key)?;
    let transaction_index = next_squads_transaction_index(&multisig)?;
    let vault = squads_v4::vault_pda(&multisig_key, 0);
    let transfer_ix = squads_v4::sol_transfer_ix(&vault, &to_pubkey, amount_lamports);
    let (tx_create_ix, transaction, vault, _) = squads_v4::vault_transaction_create_ix(
        &multisig_key,
        &signer.pubkey(),
        transaction_index,
        0,
        &[transfer_ix],
        req.memo,
    )
    .map_err(|message| ApiError { message })?;
    let (proposal_ix, proposal) =
        squads_v4::proposal_create_ix(&multisig_key, &signer.pubkey(), transaction_index, false)
            .map_err(|message| ApiError { message })?;
    let signature = sign_and_send(
        &client,
        vec![tx_create_ix, proposal_ix],
        &[&signer],
        &signer.pubkey(),
    )?;

    Ok(Json(SquadsProposalCreateResponse {
        multisig: multisig_key.to_string(),
        vault: vault.to_string(),
        transaction: transaction.to_string(),
        proposal: proposal.to_string(),
        transaction_index,
        signature,
        network,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct SquadsTokenTransferProposalRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    mint: String,
    #[serde(default)]
    recipient: Option<String>,
    amount: DecimalAmount,
    #[allow(dead_code)]
    #[serde(default)]
    decimals: Option<u8>,
    #[serde(default)]
    source_token_account: Option<String>,
    #[serde(default)]
    destination_token_account: Option<String>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

async fn squads_token_transfer_proposal(
    Json(req): Json<SquadsTokenTransferProposalRequest>,
) -> Result<Json<SquadsProposalCreateResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig_key = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let mint =
        squads_v4::parse_pubkey(&req.mint, "Token Mint").map_err(|message| ApiError { message })?;
    let recipient = if let Some(recipient) = req
        .recipient
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(
            squads_v4::parse_pubkey(recipient, "接收钱包地址")
                .map_err(|message| ApiError { message })?,
        )
    } else {
        None
    };
    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }

    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let mint_info = token_mint_info_from_rpc(&client, &mint)?;
    let decimals = mint_info.decimals;
    let token_amount = token_amount_to_raw(&req.amount, decimals)?;
    let multisig = load_squads_multisig(&client, &multisig_key)?;
    let transaction_index = next_squads_transaction_index(&multisig)?;
    let vault = squads_v4::vault_pda(&multisig_key, 0);
    let source = if let Some(source) = req
        .source_token_account
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        squads_v4::parse_pubkey(source, "来源 Token 账户")
            .map_err(|message| ApiError { message })?
    } else {
        squads_v4::associated_token_address(&vault, &mint)
    };
    let destination_override = req
        .destination_token_account
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let destination = if let Some(destination) = destination_override {
        squads_v4::parse_pubkey(destination, "接收 Token 账户")
            .map_err(|message| ApiError { message })?
    } else if let Some(recipient) = recipient {
        squads_v4::associated_token_address(&recipient, &mint)
    } else {
        return Err(ApiError {
            message: "接收钱包地址或接收 Token 账户不能为空".to_string(),
        });
    };
    let transfer_ix = squads_v4::token_transfer_checked_ix(
        &source,
        &mint,
        &destination,
        &vault,
        token_amount,
        decimals,
    );
    let mut inner_instructions = Vec::new();
    if destination_override.is_none() {
        if let Some(recipient) = recipient {
            inner_instructions.push(squads_v4::create_associated_token_account_idempotent_ix(
                &vault, &recipient, &mint,
            ));
        }
    }
    inner_instructions.push(transfer_ix);
    let (tx_create_ix, transaction, vault, _) = squads_v4::vault_transaction_create_ix(
        &multisig_key,
        &signer.pubkey(),
        transaction_index,
        0,
        &inner_instructions,
        req.memo,
    )
    .map_err(|message| ApiError { message })?;
    let (proposal_ix, proposal) =
        squads_v4::proposal_create_ix(&multisig_key, &signer.pubkey(), transaction_index, false)
            .map_err(|message| ApiError { message })?;
    let signature = sign_and_send(
        &client,
        vec![tx_create_ix, proposal_ix],
        &[&signer],
        &signer.pubkey(),
    )?;

    Ok(Json(SquadsProposalCreateResponse {
        multisig: multisig_key.to_string(),
        vault: vault.to_string(),
        transaction: transaction.to_string(),
        proposal: proposal.to_string(),
        transaction_index,
        signature,
        network,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct SquadsPrepareUpgradeBufferRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    program_so_base64: String,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct SquadsPrepareUpgradeBufferResponse {
    multisig: String,
    vault: String,
    buffer_address: String,
    create_signature: String,
    authority_signature: String,
    network: String,
    program_bytes: usize,
    rent_lamports: u64,
    write_signatures: Vec<String>,
    status: String,
}

async fn squads_prepare_upgrade_buffer(
    Json(req): Json<SquadsPrepareUpgradeBufferRequest>,
) -> Result<Json<SquadsPrepareUpgradeBufferResponse>, ApiError> {
    let program_bytes = decode_program_binary_base64(&req.program_so_base64)?;
    let program_bytes = verify_program_binary_offline(program_bytes).await?;

    let payer = req.wallet.keypair()?;
    let payer_pubkey = payer.pubkey();
    let multisig = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let vault = squads_v4::vault_pda(&multisig, 0);
    let buffer_keypair = Keypair::new();
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig_state = load_squads_multisig(&client, &multisig)?;
    require_squads_member(&multisig_state, &payer_pubkey)?;
    let buffer_lamports = client
        .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::size_of_buffer(
            program_bytes.len(),
        ))
        .map_err(|e| ApiError {
            message: format!("计算 buffer 租金失败: {}", e),
        })?;

    let create_buffer_ixs = loader_v3_instruction::create_buffer(
        &payer_pubkey,
        &buffer_keypair.pubkey(),
        &payer_pubkey,
        buffer_lamports,
        program_bytes.len(),
    )
    .map_err(|e| ApiError {
        message: format!("创建 buffer 指令失败: {}", e),
    })?;
    let create_signature = sign_and_send(
        &client,
        create_buffer_ixs,
        &[&payer, &buffer_keypair],
        &payer_pubkey,
    )?;

    let mut write_signatures = Vec::new();
    for (index, chunk) in program_bytes.chunks(PROGRAM_WRITE_CHUNK_BYTES).enumerate() {
        let offset = index
            .checked_mul(PROGRAM_WRITE_CHUNK_BYTES)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| ApiError {
                message: "Program 写入偏移超出范围".to_string(),
            })?;
        let write_ix = loader_v3_instruction::write(
            &buffer_keypair.pubkey(),
            &payer_pubkey,
            offset,
            chunk.to_vec(),
        );
        write_signatures.push(sign_and_send(
            &client,
            vec![write_ix],
            &[&payer],
            &payer_pubkey,
        )?);
    }

    let set_authority_ix =
        squads_v4::set_buffer_authority_ix(&buffer_keypair.pubkey(), &payer_pubkey, &vault);
    let authority_signature = sign_and_send_single(&client, set_authority_ix, &payer)?;

    Ok(Json(SquadsPrepareUpgradeBufferResponse {
        multisig: multisig.to_string(),
        vault: vault.to_string(),
        buffer_address: buffer_keypair.pubkey().to_string(),
        create_signature,
        authority_signature,
        network,
        program_bytes: program_bytes.len(),
        rent_lamports: buffer_lamports,
        write_signatures,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct SquadsProgramUpgradeProposalRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    program_id: String,
    buffer_address: String,
    #[serde(default)]
    spill_address: Option<String>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

async fn squads_program_upgrade_proposal(
    Json(req): Json<SquadsProgramUpgradeProposalRequest>,
) -> Result<Json<SquadsProposalCreateResponse>, ApiError> {
    let program_id = squads_v4::parse_pubkey(&req.program_id, "Program ID")
        .map_err(|message| ApiError { message })?;

    let signer = req.wallet.keypair()?;
    let multisig_key = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let buffer = squads_v4::parse_pubkey(&req.buffer_address, "buffer 地址")
        .map_err(|message| ApiError { message })?;
    let spill = if let Some(spill) = req
        .spill_address
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        squads_v4::parse_pubkey(spill, "spill 地址").map_err(|message| ApiError { message })?
    } else {
        signer.pubkey()
    };
    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }

    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig = load_squads_multisig(&client, &multisig_key)?;
    require_squads_member(&multisig, &signer.pubkey())?;
    let transaction_index = next_squads_transaction_index(&multisig)?;
    let vault = squads_v4::vault_pda(&multisig_key, 0);
    require_program_upgrade_authority(&client, &program_id, &vault)?;
    require_upgrade_buffer_authority(&client, &buffer, &vault)?;
    let upgrade_ix = squads_v4::upgrade_program_ix(&program_id, &buffer, &vault, &spill);
    let (tx_create_ix, transaction, vault, _) = squads_v4::vault_transaction_create_ix(
        &multisig_key,
        &signer.pubkey(),
        transaction_index,
        0,
        &[upgrade_ix],
        req.memo,
    )
    .map_err(|message| ApiError { message })?;
    let (proposal_ix, proposal) =
        squads_v4::proposal_create_ix(&multisig_key, &signer.pubkey(), transaction_index, false)
            .map_err(|message| ApiError { message })?;
    let signature = sign_and_send(
        &client,
        vec![tx_create_ix, proposal_ix],
        &[&signer],
        &signer.pubkey(),
    )?;

    Ok(Json(SquadsProposalCreateResponse {
        multisig: multisig_key.to_string(),
        vault: vault.to_string(),
        transaction: transaction.to_string(),
        proposal: proposal.to_string(),
        transaction_index,
        signature,
        network,
        status: "success".to_string(),
    }))
}

#[derive(Deserialize)]
struct SquadsVoteRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    proposal: String,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct SquadsActionResponse {
    signature: String,
    status: String,
    network: String,
}

async fn squads_proposal_approve(
    Json(req): Json<SquadsVoteRequest>,
) -> Result<Json<SquadsActionResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let proposal = squads_v4::parse_pubkey(&req.proposal, "提案地址")
        .map_err(|message| ApiError { message })?;
    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let ix = squads_v4::proposal_approve_ix(&multisig, &proposal, &signer.pubkey(), req.memo)
        .map_err(|message| ApiError { message })?;
    let signature = sign_and_send_single(&client, ix, &signer)?;
    Ok(Json(SquadsActionResponse {
        signature,
        status: "success".to_string(),
        network,
    }))
}

async fn squads_proposal_reject(
    Json(req): Json<SquadsVoteRequest>,
) -> Result<Json<SquadsActionResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let proposal = squads_v4::parse_pubkey(&req.proposal, "提案地址")
        .map_err(|message| ApiError { message })?;
    if let Some(memo) = &req.memo {
        validate_text_len(memo, "memo", MAX_TEXT_FIELD_CHARS)?;
    }
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let ix = squads_v4::proposal_reject_ix(&multisig, &proposal, &signer.pubkey(), req.memo)
        .map_err(|message| ApiError { message })?;
    let signature = sign_and_send_single(&client, ix, &signer)?;
    Ok(Json(SquadsActionResponse {
        signature,
        status: "success".to_string(),
        network,
    }))
}

#[derive(Deserialize)]
struct SquadsExecuteRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    proposal: String,
    transaction_index: u64,
    #[serde(default)]
    network: Option<String>,
}

async fn squads_proposal_execute(
    Json(req): Json<SquadsExecuteRequest>,
) -> Result<Json<SquadsActionResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let proposal = squads_v4::parse_pubkey(&req.proposal, "提案地址")
        .map_err(|message| ApiError { message })?;
    let transaction = squads_v4::transaction_pda(&multisig, req.transaction_index);
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let vault_transaction = load_squads_vault_transaction(&client, &transaction)?;
    if vault_transaction.multisig != multisig || vault_transaction.index != req.transaction_index {
        return Err(ApiError {
            message: "Squads 交易账户与请求参数不匹配".to_string(),
        });
    }
    let ix = squads_v4::vault_transaction_execute_ix(
        &multisig,
        &transaction,
        &proposal,
        &signer.pubkey(),
        &vault_transaction.message,
        vault_transaction.vault_index,
        u8::try_from(vault_transaction.ephemeral_signer_bumps.len()).map_err(|_| ApiError {
            message: "Squads ephemeral signer 数量超出范围".to_string(),
        })?,
    )
    .map_err(|message| ApiError { message })?;
    let signature = sign_and_send_single(&client, ix, &signer)?;
    Ok(Json(SquadsActionResponse {
        signature,
        status: "success".to_string(),
        network,
    }))
}

#[derive(Deserialize)]
struct SquadsSetProgramAuthorityRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    multisig: String,
    program_id: String,
    #[serde(default)]
    network: Option<String>,
}

#[derive(Serialize)]
struct SquadsSetProgramAuthorityResponse {
    program_id: String,
    programdata_address: String,
    new_authority: String,
    signature: String,
    network: String,
    status: String,
}

async fn squads_set_program_authority(
    Json(req): Json<SquadsSetProgramAuthorityRequest>,
) -> Result<Json<SquadsSetProgramAuthorityResponse>, ApiError> {
    let signer = req.wallet.keypair()?;
    let multisig = squads_v4::parse_pubkey(&req.multisig, "多签地址")
        .map_err(|message| ApiError { message })?;
    let program_id = squads_v4::parse_pubkey(&req.program_id, "Program ID")
        .map_err(|message| ApiError { message })?;
    let vault = squads_v4::vault_pda(&multisig, 0);
    let (client, network) = rpc_client_for(req.network.as_deref())?;
    let multisig_state = load_squads_multisig(&client, &multisig)?;
    require_squads_member(&multisig_state, &signer.pubkey())?;
    let programdata_address =
        require_program_upgrade_authority(&client, &program_id, &signer.pubkey())?;
    let ix = squads_v4::set_program_upgrade_authority_ix(&program_id, &signer.pubkey(), &vault);
    let signature = sign_and_send_single(&client, ix, &signer)?;
    Ok(Json(SquadsSetProgramAuthorityResponse {
        program_id: program_id.to_string(),
        programdata_address: programdata_address.to_string(),
        new_authority: vault.to_string(),
        signature,
        network,
        status: "success".to_string(),
    }))
}

// ============= 2FA Operations (4-6) =============

// 4. Setup 2FA
#[derive(Deserialize)]
struct Setup2faRequest {
    hardware_fingerprint: String,
    master_password: String,
    #[serde(default)]
    account: Option<String>,
    #[serde(default)]
    issuer: Option<String>,
}
#[derive(Serialize)]
struct Setup2faResponse {
    totp_secret: String,
    qr_code_url: String,
}

async fn setup_2fa(Json(req): Json<Setup2faRequest>) -> Result<Json<Setup2faResponse>, ApiError> {
    require_nonempty(req.hardware_fingerprint.trim(), "硬件指纹")?;
    require_nonempty(req.master_password.as_str(), "主密码")?;
    validate_text_len(&req.hardware_fingerprint, "硬件指纹", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.master_password, "主密码", MAX_TEXT_FIELD_CHARS)?;
    let account = validate_optional_label(req.account, "账户名称")?
        .unwrap_or_else(|| "sol-safekey".to_string());
    let issuer =
        validate_optional_label(req.issuer, "发行者")?.unwrap_or_else(|| "Sol SafeKey".to_string());

    let totp_secret = sol_safekey::derive_totp_secret_from_hardware_and_password(
        &req.hardware_fingerprint,
        &req.master_password,
        &account,
        &issuer,
    )
    .map_err(|e| ApiError {
        message: format!("生成 TOTP 失败: {}", e),
    })?;

    // Generate QR code URL
    let qr_code_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        urlencoding::encode(&issuer),
        urlencoding::encode(&account),
        urlencoding::encode(&totp_secret),
        urlencoding::encode(&issuer)
    );

    Ok(Json(Setup2faResponse {
        totp_secret,
        qr_code_url,
    }))
}

// 5. Create Triple-Factor Wallet
#[derive(Deserialize)]
struct CreateTripleFactorRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    totp_secret: String,
    hardware_fingerprint: String,
    master_password: String,
    question_index: usize,
    security_answer: String,
}
#[derive(Serialize)]
struct CreateTripleFactorResponse {
    encrypted_wallet: String,
    public_key: String,
}

async fn create_triple_factor_wallet(
    Json(req): Json<CreateTripleFactorRequest>,
) -> Result<Json<CreateTripleFactorResponse>, ApiError> {
    require_nonempty(req.totp_secret.trim(), "TOTP secret")?;
    require_nonempty(req.hardware_fingerprint.trim(), "硬件指纹")?;
    require_nonempty(req.master_password.as_str(), "主密码")?;
    require_nonempty(req.security_answer.trim(), "安全答案")?;
    if req.question_index > MAX_SECURITY_QUESTION_INDEX {
        return Err(ApiError {
            message: format!("安全问题索引不能超过 {}", MAX_SECURITY_QUESTION_INDEX),
        });
    }
    validate_text_len(&req.totp_secret, "TOTP secret", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.hardware_fingerprint, "硬件指纹", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.master_password, "主密码", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.security_answer, "安全答案", MAX_TEXT_FIELD_CHARS)?;
    let keypair = req.wallet.keypair()?;
    let private_key = keypair.to_base58_string();

    let encrypted = sol_safekey::encrypt_with_triple_factor(
        &private_key,
        &req.totp_secret,
        &req.hardware_fingerprint,
        &req.master_password,
        req.question_index,
        &req.security_answer,
    )
    .map_err(|e| ApiError {
        message: format!("加密失败: {}", e),
    })?;

    // Get public key from private key
    let public_key = KeyManager::get_public_key(&private_key).map_err(|e| ApiError {
        message: format!("获取公钥失败: {}", e),
    })?;

    Ok(Json(CreateTripleFactorResponse {
        encrypted_wallet: encrypted,
        public_key,
    }))
}

// 6. Unlock Triple-Factor Wallet
#[derive(Deserialize)]
struct UnlockTripleFactorRequest {
    encrypted_wallet: String,
    hardware_fingerprint: String,
    master_password: String,
    security_answer: String,
    totp_code: String,
}
#[derive(Serialize)]
struct UnlockTripleFactorResponse {
    public_key: String,
    unlocked: bool,
}

async fn unlock_triple_factor_wallet(
    Json(req): Json<UnlockTripleFactorRequest>,
) -> Result<Json<UnlockTripleFactorResponse>, ApiError> {
    require_nonempty(req.encrypted_wallet.trim(), "三因素钱包")?;
    require_nonempty(req.hardware_fingerprint.trim(), "硬件指纹")?;
    require_nonempty(req.master_password.as_str(), "主密码")?;
    require_nonempty(req.security_answer.trim(), "安全答案")?;
    require_nonempty(req.totp_code.trim(), "TOTP 验证码")?;
    validate_text_len(&req.encrypted_wallet, "三因素钱包", MAX_JSON_BODY_BYTES)?;
    validate_text_len(&req.hardware_fingerprint, "硬件指纹", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.master_password, "主密码", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.security_answer, "安全答案", MAX_TEXT_FIELD_CHARS)?;
    validate_text_len(&req.totp_code, "TOTP 验证码", MAX_LABEL_CHARS)?;
    let (decrypted, _question, _index) = sol_safekey::decrypt_with_triple_factor_and_2fa(
        &req.encrypted_wallet,
        &req.hardware_fingerprint,
        &req.master_password,
        &req.security_answer,
        &req.totp_code,
    )
    .map_err(|e| ApiError {
        message: format!("解密失败: {}", e),
    })?;

    let public_key = KeyManager::get_public_key(&decrypted).map_err(|e| ApiError {
        message: format!("获取公钥失败: {}", e),
    })?;

    Ok(Json(UnlockTripleFactorResponse {
        public_key,
        unlocked: true,
    }))
}

// ============= Pump.fun Operations (15-18) =============

// 15. Pump.fun Sell Token
#[derive(Deserialize)]
struct PumpfunSellRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    mint: String,
    #[serde(default)]
    amount: Option<DecimalAmount>,
    #[serde(default)]
    sell_percent: Option<u64>,
    #[serde(default)]
    slippage: Option<u64>,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct PumpfunSellResponse {
    status: String,
    signature: String,
    dex: String,
    market: String,
    sold_raw_amount: String,
    decimals: u8,
    source_account: String,
}

async fn pumpfun_sell(
    Json(req): Json<PumpfunSellRequest>,
) -> Result<Json<PumpfunSellResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;

    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let slippage = normalize_slippage_bps(req.slippage)?;
    let sell_percent = normalize_sell_percent_bps(req.sell_percent)?;

    let execution = execute_pump_sell(
        keypair,
        req.mint,
        &rpc_url,
        req.amount,
        sell_percent,
        slippage,
        DexType::PumpFun,
    )
    .await?;

    Ok(Json(PumpfunSellResponse {
        status: "success".to_string(),
        signature: execution.signature,
        dex: pump_sell_dex_value(execution.dex_type).to_string(),
        market: pump_sell_market_value(execution.dex_type).to_string(),
        sold_raw_amount: execution.sold_raw_amount.to_string(),
        decimals: execution.decimals,
        source_account: execution.source_account,
    }))
}

// 17. Pump.fun Cashback
#[derive(Deserialize)]
struct CashbackInfoRequest {
    owner: String,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct CashbackInfoResponse {
    owner: String,
    network: String,
    accumulator: String,
    amount_lamports: u64,
    ui_amount_string: String,
    asset: String,
    available: bool,
}

async fn cashback_info_handler(
    req: CashbackInfoRequest,
    dex_type: DexType,
) -> Result<Json<CashbackInfoResponse>, ApiError> {
    let owner = Pubkey::from_str(&req.owner).map_err(|_| ApiError {
        message: "无效的钱包地址".to_string(),
    })?;
    let network = network_name(req.network.as_deref()).to_string();
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (accumulator, amount_lamports, asset) = cashback_info(&client, &owner, dex_type)?;
    Ok(Json(CashbackInfoResponse {
        owner: owner.to_string(),
        network,
        accumulator: accumulator.to_string(),
        amount_lamports,
        ui_amount_string: lamports_to_sol(amount_lamports).to_string(),
        asset,
        available: amount_lamports > 0,
    }))
}

async fn pumpfun_cashback_info(
    Json(req): Json<CashbackInfoRequest>,
) -> Result<Json<CashbackInfoResponse>, ApiError> {
    cashback_info_handler(req, DexType::PumpFun).await
}

async fn pumpswap_cashback_info(
    Json(req): Json<CashbackInfoRequest>,
) -> Result<Json<CashbackInfoResponse>, ApiError> {
    cashback_info_handler(req, DexType::PumpSwap).await
}

#[derive(Deserialize)]
struct PumpfunCashbackRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct PumpfunCashbackResponse {
    status: String,
    message: String,
    signature: String,
    amount_lamports: u64,
    ui_amount_string: String,
    asset: String,
}

async fn pumpfun_cashback(
    Json(req): Json<PumpfunCashbackRequest>,
) -> Result<Json<PumpfunCashbackResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;
    let owner = keypair.pubkey();
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (_accumulator, amount_lamports, asset) =
        cashback_info(&rpc_client, &owner, DexType::PumpFun)?;
    if amount_lamports == 0 {
        return Err(ApiError {
            message: "当前没有可领取的 Pump.fun 返现".to_string(),
        });
    }
    let signature = execute_cashback_claim(keypair, &rpc_url, DexType::PumpFun).await?;
    Ok(Json(PumpfunCashbackResponse {
        status: "success".to_string(),
        message: format!(
            "已提交 Pump.fun 返现领取，领取前可领取金额约 {} {}",
            lamports_to_sol(amount_lamports),
            asset
        ),
        signature,
        amount_lamports,
        ui_amount_string: lamports_to_sol(amount_lamports).to_string(),
        asset,
    }))
}

// 16. PumpSwap Sell Token
#[derive(Deserialize)]
struct PumpswapSellRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    mint: String,
    #[serde(default)]
    amount: Option<DecimalAmount>,
    #[serde(default)]
    sell_percent: Option<u64>,
    #[serde(default)]
    slippage: Option<u64>,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct PumpswapSellResponse {
    status: String,
    signature: String,
    dex: String,
    market: String,
    sold_raw_amount: String,
    decimals: u8,
    source_account: String,
}

async fn pumpswap_sell(
    Json(req): Json<PumpswapSellRequest>,
) -> Result<Json<PumpswapSellResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;

    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let slippage = normalize_slippage_bps(req.slippage)?;
    let sell_percent = normalize_sell_percent_bps(req.sell_percent)?;

    let execution = execute_pump_sell(
        keypair,
        req.mint,
        &rpc_url,
        req.amount,
        sell_percent,
        slippage,
        DexType::PumpSwap,
    )
    .await?;

    Ok(Json(PumpswapSellResponse {
        status: "success".to_string(),
        signature: execution.signature,
        dex: pump_sell_dex_value(execution.dex_type).to_string(),
        market: pump_sell_market_value(execution.dex_type).to_string(),
        sold_raw_amount: execution.sold_raw_amount.to_string(),
        decimals: execution.decimals,
        source_account: execution.source_account,
    }))
}

// 18. PumpSwap Cashback
#[derive(Deserialize)]
struct PumpswapCashbackRequest {
    #[serde(flatten)]
    wallet: WalletAuthRequest,
    #[serde(default)]
    network: Option<String>,
}
#[derive(Serialize)]
struct PumpswapCashbackResponse {
    status: String,
    message: String,
    signature: String,
    amount_lamports: u64,
    ui_amount_string: String,
    asset: String,
}

async fn pumpswap_cashback(
    Json(req): Json<PumpswapCashbackRequest>,
) -> Result<Json<PumpswapCashbackResponse>, ApiError> {
    let keypair = req.wallet.keypair()?;
    let owner = keypair.pubkey();
    let rpc_url = get_rpc_url(req.network.as_deref())?;
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let (_accumulator, amount_lamports, asset) =
        cashback_info(&rpc_client, &owner, DexType::PumpSwap)?;
    if amount_lamports == 0 {
        return Err(ApiError {
            message: "当前没有可领取的 PumpSwap 返现".to_string(),
        });
    }
    let signature = execute_cashback_claim(keypair, &rpc_url, DexType::PumpSwap).await?;
    Ok(Json(PumpswapCashbackResponse {
        status: "success".to_string(),
        message: format!(
            "已提交 PumpSwap 返现领取，领取前可领取金额约 {} {}",
            lamports_to_sol(amount_lamports),
            asset
        ),
        signature,
        amount_lamports,
        ui_amount_string: lamports_to_sol(amount_lamports).to_string(),
        asset,
    }))
}

// ============= Static File Serving =============

async fn serve_assets(uri: Uri) -> impl IntoResponse {
    let path = uri.path();
    tracing::info!("Serving asset: {}", path);

    // URL decode the path to handle encoded characters like %5B -> [ and %5D -> ]
    let decoded_path = match urlencoding::decode(path) {
        Ok(decoded) => decoded.to_string(),
        Err(e) => {
            tracing::warn!("Failed to decode path: {}, error: {}", path, e);
            path.to_string()
        }
    };

    let path_to_serve = if decoded_path == "/" {
        "index.html".to_string()
    } else {
        let trimmed = decoded_path.trim_start_matches('/');
        if trimmed.ends_with('/') {
            format!("{}index.html", trimmed)
        } else {
            trimmed.to_string()
        }
    };

    tracing::info!("Looking for asset: {}", path_to_serve);

    let asset = Assets::get(&path_to_serve)
        .map(|content| (path_to_serve.clone(), content))
        .or_else(|| {
            let index_path = format!("{}/index.html", path_to_serve.trim_end_matches('/'));
            Assets::get(&index_path).map(|content| (index_path, content))
        });

    match asset {
        Some((asset_path, content)) => {
            let mime = mime_guess::from_path(&asset_path)
                .first_or_octet_stream()
                .to_string();
            tracing::info!("Found asset: {}, mime: {}", asset_path, mime);
            asset_response(
                StatusCode::OK,
                Some(&mime),
                Body::from(content.data.to_vec()),
            )
        }
        None => {
            tracing::warn!(
                "Asset not found: {}, falling back to index.html",
                path_to_serve
            );
            if let Some(index) = Assets::get("index.html") {
                asset_response(
                    StatusCode::OK,
                    Some("text/html"),
                    Body::from(index.data.to_vec()),
                )
            } else {
                asset_response(StatusCode::NOT_FOUND, None, Body::from("Not Found"))
            }
        }
    }
}

fn asset_response(status: StatusCode, content_type: Option<&str>, body: Body) -> Response {
    let mut builder = Response::builder().status(status);
    if let Some(content_type) = content_type {
        builder = builder.header("Content-Type", content_type);
    }
    builder.body(body).unwrap_or_else(|e| {
        tracing::error!("Failed to build asset response: {}", e);
        let mut response = Response::new(Body::from(r#"{"error":"Internal Server Error"}"#));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
            .headers_mut()
            .insert("content-type", HeaderValue::from_static("application/json"));
        response
    })
}

#[derive(Debug)]
struct ApiError {
    message: String,
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(json!({ "error": self.message }));
        (status, body).into_response()
    }
}
impl ApiError {
    fn status_code(&self) -> StatusCode {
        let message = self.message.as_str();
        if message == SBF_VERIFY_BUSY_MESSAGE {
            StatusCode::TOO_MANY_REQUESTS
        } else if message == PROGRAM_DEPLOY_BUSY_MESSAGE
            || message.contains("目标 Program 或 ProgramData 已存在")
            || message.contains("部署 journal 冲突")
        {
            StatusCode::CONFLICT
        } else if message.contains("RPC 节点暂时无法读取")
            || message.contains("卖出执行异常")
            || message.contains("卖出成功但未返回交易签名")
        {
            StatusCode::BAD_GATEWAY
        } else if message.contains("未找到") {
            StatusCode::NOT_FOUND
        } else if message.contains("密码校验失败")
            || message.contains("钱包解锁失败")
            || message.contains("keystore 校验失败")
            || message.contains("keystore 解密失败")
            || message.contains("解密失败")
        {
            StatusCode::UNAUTHORIZED
        } else if message.contains("已禁用") {
            StatusCode::FORBIDDEN
        } else if message.contains("查询失败")
            || message.contains("查询余额失败")
            || message.contains("查询交易记录失败")
            || message.contains("查询 Token")
            || message.contains("查询 Program 失败")
            || message.contains("获取 blockhash 失败")
            || message.contains("get_account")
            || message.contains("getProgramAccounts")
            || message.contains("rpc")
            || message.contains("RPC")
            || message.contains("Connection rate limits exceeded")
            || message.contains("429")
            || message.contains("timed out")
            || message.contains("timeout")
            || message.contains("error sending request")
        {
            StatusCode::BAD_GATEWAY
        } else if message.contains("需要")
            || message.contains("必须")
            || message.contains("无效")
            || message.contains("过大")
            || message.contains("不能为空")
            || message.contains("请使用")
            || message.contains("超出")
            || message.contains("不能超过")
            || message.contains("余额为 0")
            || message.contains("余额")
            || message.contains("未找到可用 PumpSwap 池")
            || message.contains("获取 Pump.fun 参数失败")
            || message.contains("获取 PumpSwap 参数失败")
            || message.contains("No pool found")
            || message.contains("Bonding curve not found")
            || message.contains("bonding curve")
            || message.contains("卖出失败")
        {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError {
            message: err.to_string(),
        }
    }
}
