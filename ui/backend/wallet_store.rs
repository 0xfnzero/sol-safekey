use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

const DEFAULT_DATABASE_PATH: &str = "data/sol-safekey.sqlite3";
const MAX_NONCE_ACCOUNTS_PER_OWNER_NETWORK: usize = 100;

fn store_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SavedWallet {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub keystore_json: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Serialize)]
pub struct WalletSummary {
    pub id: String,
    pub name: String,
    pub public_key: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Clone, Serialize)]
pub struct NonceAccountRecord {
    pub id: String,
    pub wallet_id: Option<String>,
    pub owner: String,
    pub network: String,
    pub nonce_account: String,
    pub signature: String,
    pub created_at: u64,
}

#[derive(Clone, Serialize)]
pub struct WalletTokenAssetRecord {
    pub owner: String,
    pub network: String,
    pub account: String,
    pub mint: String,
    pub amount: String,
    pub ui_amount_string: String,
    pub decimals: u8,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub logo_uri: Option<String>,
    pub updated_at: u64,
}

#[derive(Clone, Serialize)]
pub struct TokenMetadataRecord {
    pub network: String,
    pub mint: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub logo_uri: Option<String>,
    pub updated_at: u64,
}

#[derive(Clone, Serialize)]
pub struct WalletAssetsRecord {
    pub owner: String,
    pub network: String,
    pub sol_balance: f64,
    pub tokens: Vec<WalletTokenAssetRecord>,
    pub updated_at: u64,
}

impl From<SavedWallet> for WalletSummary {
    fn from(wallet: SavedWallet) -> Self {
        Self {
            id: wallet.id,
            name: wallet.name,
            public_key: wallet.public_key,
            created_at: wallet.created_at,
            updated_at: wallet.updated_at,
        }
    }
}

fn now_unix_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| format!("系统时间错误: {}", e))
}

fn fallback_wallet_name(public_key: &str) -> String {
    if public_key.len() > 12 {
        format!(
            "Wallet {}...{}",
            &public_key[..4],
            &public_key[public_key.len() - 4..]
        )
    } else {
        "Wallet".to_string()
    }
}

fn normalize_wallet_name(name: Option<String>, public_key: &str) -> String {
    name.as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fallback_wallet_name(public_key))
}

fn ensure_data_dir() -> Result<(), String> {
    let database_path = database_path();
    if let Some(parent) = database_path.parent() {
        #[cfg(unix)]
        {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)
                .map_err(|e| format!("创建数据目录失败: {}", e))?;
        }
        #[cfg(not(unix))]
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {}", e))?;
    }
    Ok(())
}

fn database_path() -> PathBuf {
    std::env::var("SOL_SAFEKEY_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_DATABASE_PATH))
}

fn open_connection() -> Result<Connection, String> {
    ensure_data_dir()?;
    let database_path = database_path();
    let conn = Connection::open(&database_path).map_err(|e| format!("打开数据库失败: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&database_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("设置数据库权限失败: {}", e))?;
    }
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("初始化数据库失败: {}", e))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("初始化数据库失败: {}", e))?;
    init_schema(&conn)?;
    Ok(conn)
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| format!("检查数据库列失败: {}", e))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("检查数据库列失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("检查数据库列失败: {}", e))?;
    if !columns.iter().any(|name| name == column) {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
            [],
        )
        .map_err(|e| format!("迁移数据库列失败: {}", e))?;
    }
    Ok(())
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS wallets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            public_key TEXT NOT NULL UNIQUE,
            keystore_json TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nonce_accounts (
            id TEXT PRIMARY KEY,
            wallet_id TEXT,
            owner TEXT NOT NULL,
            network TEXT NOT NULL,
            nonce_account TEXT NOT NULL UNIQUE,
            signature TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(wallet_id) REFERENCES wallets(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_nonce_accounts_owner_network
            ON nonce_accounts(owner, network, created_at DESC);

        CREATE TABLE IF NOT EXISTS wallet_assets (
            owner TEXT NOT NULL,
            network TEXT NOT NULL,
            sol_balance REAL NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY(owner, network)
        );

        CREATE TABLE IF NOT EXISTS wallet_token_assets (
            owner TEXT NOT NULL,
            network TEXT NOT NULL,
            account TEXT NOT NULL,
            mint TEXT NOT NULL,
            amount TEXT NOT NULL,
            ui_amount_string TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            name TEXT,
            symbol TEXT,
            logo_uri TEXT,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY(owner, network, account),
            FOREIGN KEY(owner, network) REFERENCES wallet_assets(owner, network) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_wallet_token_assets_owner_network_mint
            ON wallet_token_assets(owner, network, mint);

        CREATE TABLE IF NOT EXISTS token_metadata (
            network TEXT NOT NULL,
            mint TEXT NOT NULL,
            name TEXT,
            symbol TEXT,
            logo_uri TEXT,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY(network, mint)
        );
        "#,
    )
    .map_err(|e| format!("初始化数据库表失败: {}", e))?;
    ensure_column(conn, "wallet_token_assets", "name", "TEXT")?;
    ensure_column(conn, "wallet_token_assets", "symbol", "TEXT")?;
    ensure_column(conn, "wallet_token_assets", "logo_uri", "TEXT")?;
    Ok(())
}

fn row_to_wallet(row: &rusqlite::Row<'_>) -> rusqlite::Result<SavedWallet> {
    Ok(SavedWallet {
        id: row.get(0)?,
        name: row.get(1)?,
        public_key: row.get(2)?,
        keystore_json: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn row_to_nonce(row: &rusqlite::Row<'_>) -> rusqlite::Result<NonceAccountRecord> {
    Ok(NonceAccountRecord {
        id: row.get(0)?,
        wallet_id: row.get(1)?,
        owner: row.get(2)?,
        network: row.get(3)?,
        nonce_account: row.get(4)?,
        signature: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn row_to_token_asset(row: &rusqlite::Row<'_>) -> rusqlite::Result<WalletTokenAssetRecord> {
    Ok(WalletTokenAssetRecord {
        owner: row.get(0)?,
        network: row.get(1)?,
        account: row.get(2)?,
        mint: row.get(3)?,
        amount: row.get(4)?,
        ui_amount_string: row.get(5)?,
        decimals: row.get(6)?,
        name: row.get(7)?,
        symbol: row.get(8)?,
        logo_uri: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_token_metadata(row: &rusqlite::Row<'_>) -> rusqlite::Result<TokenMetadataRecord> {
    Ok(TokenMetadataRecord {
        network: row.get(0)?,
        mint: row.get(1)?,
        name: row.get(2)?,
        symbol: row.get(3)?,
        logo_uri: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn load() -> Result<Vec<SavedWallet>, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库读锁已损坏".to_string())?;
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, public_key, keystore_json, created_at, updated_at \
             FROM wallets ORDER BY updated_at DESC, created_at DESC",
        )
        .map_err(|e| format!("读取钱包列表失败: {}", e))?;
    let rows = stmt
        .query_map([], row_to_wallet)
        .map_err(|e| format!("读取钱包列表失败: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取钱包列表失败: {}", e))
}

pub fn list_summaries() -> Result<Vec<WalletSummary>, String> {
    Ok(load()?.into_iter().map(WalletSummary::from).collect())
}

pub fn find(wallet_id: &str) -> Result<SavedWallet, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库读锁已损坏".to_string())?;
    let conn = open_connection()?;
    conn.query_row(
        "SELECT id, name, public_key, keystore_json, created_at, updated_at FROM wallets WHERE id = ?1",
        params![wallet_id],
        row_to_wallet,
    )
    .optional()
    .map_err(|e| format!("读取钱包失败: {}", e))?
    .ok_or_else(|| "未找到已保存钱包".to_string())
}

pub fn upsert(
    keystore_json: String,
    public_key: String,
    name: Option<String>,
) -> Result<SavedWallet, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let conn = open_connection()?;
    let now = now_unix_secs()?;
    let name = normalize_wallet_name(name, &public_key);
    let existing: Option<SavedWallet> = conn
        .query_row(
            "SELECT id, name, public_key, keystore_json, created_at, updated_at FROM wallets WHERE public_key = ?1",
            params![public_key],
            row_to_wallet,
        )
        .optional()
        .map_err(|e| format!("读取钱包失败: {}", e))?;

    let wallet = if let Some(mut wallet) = existing {
        wallet.name = name;
        wallet.keystore_json = keystore_json;
        wallet.updated_at = now;
        conn.execute(
            "UPDATE wallets SET name = ?1, keystore_json = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                wallet.name,
                wallet.keystore_json,
                wallet.updated_at,
                wallet.id
            ],
        )
        .map_err(|e| format!("保存钱包失败: {}", e))?;
        wallet
    } else {
        let wallet = SavedWallet {
            id: Uuid::new_v4().simple().to_string(),
            name,
            public_key,
            keystore_json,
            created_at: now,
            updated_at: now,
        };
        conn.execute(
            "INSERT INTO wallets (id, name, public_key, keystore_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                wallet.id,
                wallet.name,
                wallet.public_key,
                wallet.keystore_json,
                wallet.created_at,
                wallet.updated_at,
            ],
        )
        .map_err(|e| format!("保存钱包失败: {}", e))?;
        wallet
    };

    Ok(wallet)
}

pub fn update_metadata(
    wallet_id: &str,
    name: &str,
    keystore_json: String,
) -> Result<SavedWallet, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let name = name.trim();
    if name.is_empty() {
        return Err("钱包名称不能为空".to_string());
    }

    let conn = open_connection()?;
    let now = now_unix_secs()?;
    let updated = conn
        .execute(
            "UPDATE wallets SET name = ?1, keystore_json = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, keystore_json, now, wallet_id],
        )
        .map_err(|e| format!("更新钱包失败: {}", e))?;
    if updated == 0 {
        return Err("未找到已保存钱包".to_string());
    }
    conn.query_row(
        "SELECT id, name, public_key, keystore_json, created_at, updated_at FROM wallets WHERE id = ?1",
        params![wallet_id],
        row_to_wallet,
    )
    .map_err(|e| format!("读取钱包失败: {}", e))
}

pub fn delete(wallet_id: &str) -> Result<(), String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let conn = open_connection()?;
    let deleted = conn
        .execute("DELETE FROM wallets WHERE id = ?1", params![wallet_id])
        .map_err(|e| format!("删除钱包失败: {}", e))?;
    if deleted == 0 {
        return Err("未找到已保存钱包".to_string());
    }
    Ok(())
}

pub fn add_nonce_account(
    wallet_id: Option<&str>,
    owner: &str,
    network: &str,
    nonce_account: &str,
    signature: &str,
) -> Result<NonceAccountRecord, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let conn = open_connection()?;
    let now = now_unix_secs()?;
    let record = NonceAccountRecord {
        id: Uuid::new_v4().simple().to_string(),
        wallet_id: wallet_id.map(ToOwned::to_owned),
        owner: owner.to_string(),
        network: network.to_string(),
        nonce_account: nonce_account.to_string(),
        signature: signature.to_string(),
        created_at: now,
    };
    let inserted = conn
        .execute(
            "INSERT OR IGNORE INTO nonce_accounts \
             (id, wallet_id, owner, network, nonce_account, signature, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.id,
                record.wallet_id,
                record.owner,
                record.network,
                record.nonce_account,
                record.signature,
                record.created_at,
            ],
        )
        .map_err(|e| format!("保存 Nonce 账户失败: {}", e))?;
    if inserted == 0 {
        return conn
            .query_row(
                "SELECT id, wallet_id, owner, network, nonce_account, signature, created_at \
                 FROM nonce_accounts WHERE nonce_account = ?1",
                params![nonce_account],
                row_to_nonce,
            )
            .map_err(|e| format!("读取已保存 Nonce 账户失败: {}", e));
    }
    conn.execute(
        "DELETE FROM nonce_accounts
         WHERE owner = ?1 AND network = ?2 AND id NOT IN (
             SELECT id FROM nonce_accounts
             WHERE owner = ?1 AND network = ?2
             ORDER BY created_at DESC
             LIMIT ?3
         )",
        params![owner, network, MAX_NONCE_ACCOUNTS_PER_OWNER_NETWORK as i64],
    )
    .map_err(|e| format!("清理旧 Nonce 账户失败: {}", e))?;
    Ok(record)
}

pub fn list_nonce_accounts(owner: &str, network: &str) -> Result<Vec<NonceAccountRecord>, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库读锁已损坏".to_string())?;
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, wallet_id, owner, network, nonce_account, signature, created_at \
             FROM nonce_accounts WHERE owner = ?1 AND network = ?2 ORDER BY created_at DESC LIMIT ?3",
        )
        .map_err(|e| format!("读取 Nonce 账户列表失败: {}", e))?;
    let rows = stmt
        .query_map(
            params![owner, network, MAX_NONCE_ACCOUNTS_PER_OWNER_NETWORK as i64],
            row_to_nonce,
        )
        .map_err(|e| format!("读取 Nonce 账户列表失败: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取 Nonce 账户列表失败: {}", e))
}

pub fn get_wallet_assets(owner: &str, network: &str) -> Result<Option<WalletAssetsRecord>, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库读锁已损坏".to_string())?;
    let conn = open_connection()?;
    let header = conn
        .query_row(
            "SELECT sol_balance, updated_at FROM wallet_assets WHERE owner = ?1 AND network = ?2",
            params![owner, network],
            |row| Ok((row.get::<_, f64>(0)?, row.get::<_, u64>(1)?)),
        )
        .optional()
        .map_err(|e| format!("读取钱包资产缓存失败: {}", e))?;
    let Some((sol_balance, updated_at)) = header else {
        return Ok(None);
    };

    let mut stmt = conn
        .prepare(
            "SELECT owner, network, account, mint, amount, ui_amount_string, decimals, name, symbol, logo_uri, updated_at \
             FROM wallet_token_assets \
             WHERE owner = ?1 AND network = ?2 \
             ORDER BY LENGTH(amount) DESC, amount DESC, mint ASC, account ASC",
        )
        .map_err(|e| format!("读取 Token 资产缓存失败: {}", e))?;
    let rows = stmt
        .query_map(params![owner, network], row_to_token_asset)
        .map_err(|e| format!("读取 Token 资产缓存失败: {}", e))?;
    let tokens = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取 Token 资产缓存失败: {}", e))?;

    Ok(Some(WalletAssetsRecord {
        owner: owner.to_string(),
        network: network.to_string(),
        sol_balance,
        tokens,
        updated_at,
    }))
}

pub fn save_wallet_assets(
    owner: &str,
    network: &str,
    sol_balance: f64,
    tokens: &[WalletTokenAssetRecord],
) -> Result<u64, String> {
    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let mut conn = open_connection()?;
    let now = now_unix_secs()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("保存钱包资产缓存失败: {}", e))?;
    tx.execute(
        "INSERT INTO wallet_assets (owner, network, sol_balance, updated_at) \
         VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(owner, network) DO UPDATE SET \
         sol_balance = excluded.sol_balance, updated_at = excluded.updated_at",
        params![owner, network, sol_balance, now],
    )
    .map_err(|e| format!("保存钱包资产缓存失败: {}", e))?;
    tx.execute(
        "DELETE FROM wallet_token_assets WHERE owner = ?1 AND network = ?2",
        params![owner, network],
    )
    .map_err(|e| format!("清理 Token 资产缓存失败: {}", e))?;
    for token in tokens {
        tx.execute(
            "INSERT INTO wallet_token_assets \
             (owner, network, account, mint, amount, ui_amount_string, decimals, name, symbol, logo_uri, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                owner,
                network,
                token.account,
                token.mint,
                token.amount,
                token.ui_amount_string,
                token.decimals,
                token.name,
                token.symbol,
                token.logo_uri,
                now,
            ],
        )
        .map_err(|e| format!("保存 Token 资产缓存失败: {}", e))?;
    }
    tx.commit()
        .map_err(|e| format!("提交钱包资产缓存失败: {}", e))?;
    Ok(now)
}

pub fn get_token_metadata(
    network: &str,
    mints: &[String],
) -> Result<Vec<TokenMetadataRecord>, String> {
    if mints.is_empty() {
        return Ok(Vec::new());
    }

    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库读锁已损坏".to_string())?;
    let conn = open_connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT network, mint, name, symbol, logo_uri, updated_at \
             FROM token_metadata WHERE network = ?1 AND mint = ?2",
        )
        .map_err(|e| format!("读取 Token 元数据缓存失败: {}", e))?;
    let mut records = Vec::new();
    for mint in mints {
        if let Some(record) = stmt
            .query_row(params![network, mint], row_to_token_metadata)
            .optional()
            .map_err(|e| format!("读取 Token 元数据缓存失败: {}", e))?
        {
            records.push(record);
        }
    }
    Ok(records)
}

pub fn save_token_metadata(network: &str, records: &[TokenMetadataRecord]) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }

    let _guard = store_lock()
        .lock()
        .map_err(|_| "数据库写锁已损坏".to_string())?;
    let mut conn = open_connection()?;
    let now = now_unix_secs()?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("保存 Token 元数据缓存失败: {}", e))?;
    for record in records {
        tx.execute(
            "INSERT INTO token_metadata (network, mint, name, symbol, logo_uri, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(network, mint) DO UPDATE SET \
             name = COALESCE(excluded.name, token_metadata.name), \
             symbol = COALESCE(excluded.symbol, token_metadata.symbol), \
             logo_uri = COALESCE(excluded.logo_uri, token_metadata.logo_uri), \
             updated_at = excluded.updated_at",
            params![
                network,
                record.mint,
                record.name,
                record.symbol,
                record.logo_uri,
                now,
            ],
        )
        .map_err(|e| format!("保存 Token 元数据缓存失败: {}", e))?;
    }
    tx.commit()
        .map_err(|e| format!("提交 Token 元数据缓存失败: {}", e))
}
