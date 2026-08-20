use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use aws_lc_rs::rsa::{OaepPublicEncryptingKey, PublicEncryptingKey, OAEP_SHA256_MGF1SHA256};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use zeroize::Zeroizing;

/// Must match `DEFAULT_API_PORT` in `src/lib/api.ts`
const SOL_SAFEKEY_API_PORT: u16 = 3841;
const MAX_PROXY_BODY_BYTES: usize = 12 * 1024 * 1024;
const MAX_DOWNLOAD_FILE_BYTES: usize = 4 * 1024 * 1024;
const MAX_SECURE_PUBLIC_KEY_PEM_BYTES: usize = 2 * 1024;
const PROGRAM_DEPLOY_PROXY_TIMEOUT_SECS: u64 = 60 * 60;
const SECURE_BODY_HEADER: &str = "x-sol-safekey-secure-body";
const SECURE_BODY_VERSION: &str = "1";
const DAPP_WINDOW_LABEL: &str = "dapp";
const DAPP_SIGN_REQUEST_EVENT: &str = "dapp://sign-request";
const DAPP_REQUEST_TTL_MS: u64 = 3 * 60 * 1000;

#[derive(Clone)]
struct AllowedDapp {
    id: &'static str,
    name: &'static str,
    url: &'static str,
}

#[derive(Clone)]
struct DappSession {
    app_id: String,
    app_name: String,
    url: String,
    wallet_public_key: String,
    network: String,
    opened_at_ms: u64,
}

#[derive(Clone, Serialize)]
struct DappSignRequestEvent {
    request_id: String,
    app_id: String,
    app_name: String,
    app_url: String,
    method: String,
    wallet_public_key: String,
    network: String,
    transaction_base64: String,
    transaction_format: String,
    created_at_ms: u64,
}

#[derive(Clone)]
struct DappPendingRequest {
    event: DappSignRequestEvent,
    result: Option<DappSignResult>,
}

#[derive(Clone, Serialize, Deserialize)]
struct DappSignResult {
    approved: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    signature: Option<String>,
    #[serde(default)]
    raw_transaction: Option<String>,
    #[serde(default)]
    recent_blockhash: Option<String>,
}

#[derive(Serialize)]
struct DappPollResponse {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<DappSignResult>,
}

#[derive(Default)]
struct DappBridgeState {
    session: Mutex<Option<DappSession>>,
    requests: Mutex<HashMap<String, DappPendingRequest>>,
}

#[derive(Serialize)]
struct ProxyResponse {
    status: u16,
    body: String,
}

#[derive(Deserialize)]
struct ProxyRequestHeader {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct SecureSessionResponse {
    version: String,
    public_key_pem: String,
}

fn encrypt_secure_body(body: &str, public_key_pem: &str) -> Result<String, String> {
    if public_key_pem.len() > MAX_SECURE_PUBLIC_KEY_PEM_BYTES {
        return Err("invalid secure API public key: PEM is too large".to_string());
    }
    let mut public_key_der = [0_u8; 1024];
    let (label, public_key_der) =
        pem_rfc7468::decode(public_key_pem.as_bytes(), &mut public_key_der)
            .map_err(|e| format!("invalid secure API public key PEM: {}", e))?;
    if label != "PUBLIC KEY" {
        return Err("invalid secure API public key PEM label".to_string());
    }
    let public_key = PublicEncryptingKey::from_der(public_key_der)
        .map_err(|e| format!("invalid secure API public key: {}", e))?;
    if public_key.key_size_bits() != 2048 {
        return Err("invalid secure API public key size".to_string());
    }
    let public_key = OaepPublicEncryptingKey::new(public_key)
        .map_err(|_| "failed to initialize secure API public key".to_string())?;
    let mut rng = OsRng;
    let mut aes_key = Zeroizing::new([0_u8; 32]);
    rng.fill_bytes(&mut *aes_key);
    let mut iv = [0_u8; 12];
    rng.fill_bytes(&mut iv);
    let cipher = Aes256Gcm::new_from_slice(&*aes_key)
        .map_err(|_| "failed to initialize request encryption")?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&iv), body.as_bytes())
        .map_err(|_| "failed to encrypt request body")?;
    let mut encrypted_key = vec![0_u8; public_key.ciphertext_size()];
    let encrypted_key_len = public_key
        .encrypt(&OAEP_SHA256_MGF1SHA256, &*aes_key, &mut encrypted_key, None)
        .map_err(|_| "failed to encrypt request key".to_string())?
        .len();
    encrypted_key.truncate(encrypted_key_len);

    Ok(json!({
      "version": 1,
      "encrypted_key": BASE64.encode(encrypted_key),
      "iv": BASE64.encode(iv),
      "ciphertext": BASE64.encode(ciphertext),
    })
    .to_string())
}

/// HTTP from Rust → avoids WKWebView `fetch` URL issues with localhost /api.
#[tauri::command]
async fn proxy_api_request(
    method: String,
    path: String,
    headers: Option<Vec<ProxyRequestHeader>>,
    body: Option<String>,
    secure_proxy: Option<bool>,
) -> Result<ProxyResponse, String> {
    let path = path.trim_start_matches('/');
    if path.is_empty()
        || path.contains("://")
        || path.contains('\\')
        || path.split('/').any(|part| part == "..")
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_' | b'.'))
    {
        return Err("invalid API path".to_string());
    }
    if let Some(b) = body.as_ref() {
        if b.len() > MAX_PROXY_BODY_BYTES {
            return Err("request body too large".to_string());
        }
    }
    let url = format!("http://127.0.0.1:{}/api/{}", SOL_SAFEKEY_API_PORT, path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            PROGRAM_DEPLOY_PROXY_TIMEOUT_SECS,
        ))
        .build()
        .map_err(|e| e.to_string())?;

    let method_upper = method.to_uppercase();
    let secure_proxy = secure_proxy.unwrap_or(false);
    let mut req = match method_upper.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => return Err(format!("unsupported HTTP method: {}", method)),
    };

    req = req.header("Content-Type", "application/json");
    if let Some(headers) = headers {
        for header in headers {
            let name = header.name.to_ascii_lowercase();
            if matches!(name.as_str(), "content-type" | SECURE_BODY_HEADER) {
                req = req.header(name, header.value);
            }
        }
    }
    if let Ok(token) = std::env::var("SOL_SAFEKEY_API_TOKEN") {
        if !token.trim().is_empty() {
            req = req.header("X-Sol-SafeKey-Token", token);
        }
    }
    if let Some(b) = body {
        if secure_proxy && matches!(method_upper.as_str(), "POST" | "PUT" | "PATCH") {
            let b = Zeroizing::new(b);
            let session_url = format!(
                "http://127.0.0.1:{}/api/secure/session",
                SOL_SAFEKEY_API_PORT
            );
            let session_resp = client
                .get(session_url)
                .send()
                .await
                .map_err(|e| format!("failed to initialize secure API session: {}", e))?;
            if !session_resp.status().is_success() {
                return Err(format!(
                    "failed to initialize secure API session: HTTP {}",
                    session_resp.status().as_u16()
                ));
            }
            let session = session_resp
                .json::<SecureSessionResponse>()
                .await
                .map_err(|e| format!("invalid secure API session: {}", e))?;
            if session.version != SECURE_BODY_VERSION {
                return Err("unsupported secure API session version".to_string());
            }
            let encrypted_body = encrypt_secure_body(&b, &session.public_key_pem)?;
            req = req.header(SECURE_BODY_HEADER, SECURE_BODY_VERSION);
            req = req.body(encrypted_body);
        } else {
            req = req.body(b);
        }
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let body = resp.text().await.map_err(|e| e.to_string())?;

    Ok(ProxyResponse { status, body })
}

fn is_allowed_external_https_url(url: &str) -> bool {
    let trimmed = url.trim();
    if !trimmed.starts_with("https://") || trimmed.len() > 2048 {
        return false;
    }
    if trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return false;
    }
    let Some(rest) = trimmed.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if host.is_empty() || host.starts_with('.') || host.ends_with('.') || host.contains('@') {
        return false;
    }
    true
}

fn spawn_system_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open external browser: {error}"))
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open external browser: {error}"))
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open external browser: {error}"))
    }
}

fn reveal_file_in_system_file_manager(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to reveal download file: {error}"))
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.to_string_lossy()))
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to reveal download file: {error}"))
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let directory = path.parent().unwrap_or_else(|| Path::new("/"));
        std::process::Command::new("xdg-open")
            .arg(directory)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open download directory: {error}"))
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

fn allowed_dapp(app_id: &str) -> Option<AllowedDapp> {
    match app_id.trim().to_ascii_lowercase().as_str() {
        "jupiter" => Some(AllowedDapp {
            id: "jupiter",
            name: "Jupiter",
            url: "https://jup.ag/swap",
        }),
        "pumpfun" => Some(AllowedDapp {
            id: "pumpfun",
            name: "pump.fun",
            url: "https://pump.fun/",
        }),
        "raydium" => Some(AllowedDapp {
            id: "raydium",
            name: "Raydium",
            url: "https://raydium.io/swap/",
        }),
        "meteora" => Some(AllowedDapp {
            id: "meteora",
            name: "Meteora",
            url: "https://app.meteora.ag/",
        }),
        _ => None,
    }
}

fn is_likely_solana_pubkey(value: &str) -> bool {
    let trimmed = value.trim();
    (32..=44).contains(&trimmed.len())
        && trimmed
            .bytes()
            .all(|b| matches!(b, b'1'..=b'9' | b'A'..=b'H' | b'J'..=b'N' | b'P'..=b'Z' | b'a'..=b'k' | b'm'..=b'z'))
}

fn validate_dapp_method(method: &str) -> Result<String, String> {
    let normalized = method.trim();
    match normalized {
        "signTransaction"
        | "signAllTransactions"
        | "signAndSendTransaction"
        | "sendTransaction" => Ok(normalized.to_string()),
        _ => Err("unsupported dapp signing method".to_string()),
    }
}

fn validate_transaction_format(format: &str) -> Result<String, String> {
    let normalized = format.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "legacy" | "versioned" | "v0" | "auto" => Ok(normalized),
        _ => Err("unsupported transaction format".to_string()),
    }
}

fn validate_dapp_transaction_base64(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 4096 {
        return Err("invalid transaction payload".to_string());
    }
    if BASE64.decode(trimmed).is_err() {
        return Err("transaction payload is not valid base64".to_string());
    }
    Ok(trimmed.to_string())
}

fn dapp_request_id() -> String {
    let mut random = [0_u8; 8];
    OsRng.fill_bytes(&mut random);
    format!("dapp-{}-{}", now_ms(), BASE64.encode(random))
        .replace(['+', '/', '='], "")
}

fn dapp_provider_script(
    dapp: &AllowedDapp,
    wallet_public_key: &str,
    network: &str,
) -> Result<String, String> {
    let wallet_public_key = serde_json::to_string(wallet_public_key).map_err(|e| e.to_string())?;
    let network = serde_json::to_string(network).map_err(|e| e.to_string())?;
    let app_id = serde_json::to_string(dapp.id).map_err(|e| e.to_string())?;
    let app_name = serde_json::to_string(dapp.name).map_err(|e| e.to_string())?;
    Ok(format!(
        r#"
(function () {{
  const walletPublicKey = {wallet_public_key};
  const network = {network};
  const appId = {app_id};
  const appName = {app_name};
  const listeners = new Map();
  let connected = true;

  function sleep(ms) {{ return new Promise((resolve) => setTimeout(resolve, ms)); }}
  function tauriInvoke(command, args) {{
    const api = window.__TAURI__ && window.__TAURI__.core;
    if (!api || typeof api.invoke !== "function") {{
      throw new Error("Sol SafeKey bridge is unavailable");
    }}
    return api.invoke(command, args || {{}});
  }}
  function bytesToBase64(bytes) {{
    const view = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    let binary = "";
    for (let i = 0; i < view.length; i += 0x8000) {{
      binary += String.fromCharCode.apply(null, Array.from(view.subarray(i, i + 0x8000)));
    }}
    return btoa(binary);
  }}
  function base64ToBytes(base64) {{
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes;
  }}
  function transactionFormat(transaction) {{
    if (transaction && (transaction.version !== undefined || transaction.message?.addressTableLookups)) return "versioned";
    return "legacy";
  }}
  function serializeTransaction(transaction) {{
    if (!transaction || typeof transaction.serialize !== "function") {{
      throw new Error("Invalid Solana transaction");
    }}
    try {{
      return transaction.serialize({{ requireAllSignatures: false, verifySignatures: false }});
    }} catch (_) {{
      return transaction.serialize();
    }}
  }}
  function hydrateSignedTransaction(original, rawTransaction) {{
    const bytes = base64ToBytes(rawTransaction);
    if (original?.constructor && typeof original.constructor.deserialize === "function") {{
      return original.constructor.deserialize(bytes);
    }}
    if (original?.constructor && typeof original.constructor.from === "function") {{
      return original.constructor.from(bytes);
    }}
    return bytes;
  }}
  async function requestSignature(method, transaction) {{
    const transactionBase64 = bytesToBase64(serializeTransaction(transaction));
    const transaction_format = transactionFormat(transaction);
    const requestId = await tauriInvoke("dapp_submit_sign_request", {{
      method,
      transaction_base64: transactionBase64,
      transaction_format,
    }});
    const started = Date.now();
    while (Date.now() - started < 180000) {{
      const poll = await tauriInvoke("dapp_poll_sign_request", {{ request_id: requestId }});
      if (poll.status === "approved") return poll.result || {{}};
      if (poll.status === "rejected") throw new Error(poll.result?.error || "User rejected the request");
      if (poll.status === "expired") throw new Error("Sol SafeKey signing request expired");
      await sleep(500);
    }}
    throw new Error("Sol SafeKey signing request timed out");
  }}
  function emit(event, value) {{
    const handlers = listeners.get(event);
    if (!handlers) return;
    handlers.forEach((handler) => {{
      try {{ handler(value); }} catch (_) {{}}
    }});
  }}
  const publicKey = {{
    toBase58: () => walletPublicKey,
    toString: () => walletPublicKey,
    equals: (other) => String(other?.toBase58 ? other.toBase58() : other) === walletPublicKey,
  }};
  const provider = {{
    isPhantom: true,
    isSolflare: true,
    isSolSafeKey: true,
    appId,
    appName,
    network,
    get publicKey() {{ return connected ? publicKey : null; }},
    get isConnected() {{ return connected; }},
    async connect() {{
      connected = true;
      emit("connect", publicKey);
      return {{ publicKey }};
    }},
    async disconnect() {{
      connected = false;
      emit("disconnect");
    }},
    on(event, handler) {{
      if (!listeners.has(event)) listeners.set(event, new Set());
      listeners.get(event).add(handler);
      return this;
    }},
    off(event, handler) {{
      listeners.get(event)?.delete(handler);
      return this;
    }},
    async signTransaction(transaction) {{
      const result = await requestSignature("signTransaction", transaction);
      if (!result.raw_transaction) throw new Error("Sol SafeKey did not return a signed transaction");
      return hydrateSignedTransaction(transaction, result.raw_transaction);
    }},
    async signAllTransactions(transactions) {{
      const signed = [];
      for (const transaction of transactions || []) {{
        signed.push(await this.signTransaction(transaction));
      }}
      return signed;
    }},
    async signAndSendTransaction(input) {{
      const transaction = input?.transaction || input;
      const result = await requestSignature("signAndSendTransaction", transaction);
      if (!result.signature) throw new Error("Sol SafeKey did not return a transaction signature");
      return {{ signature: result.signature }};
    }},
    async sendTransaction(transaction) {{
      const result = await requestSignature("sendTransaction", transaction);
      if (!result.signature) throw new Error("Sol SafeKey did not return a transaction signature");
      return result.signature;
    }},
    async signMessage() {{
      throw new Error("Sol SafeKey DApp mode does not support message signing yet");
    }},
  }};
  Object.defineProperty(window, "solana", {{ value: provider, configurable: true }});
  window.phantom = window.phantom || {{}};
  Object.defineProperty(window.phantom, "solana", {{ value: provider, configurable: true }});
  Object.defineProperty(window, "solflare", {{ value: provider, configurable: true }});
  window.dispatchEvent(new Event("solana#initialized"));
}})();
"#
    ))
}

/// Open a URL in the system default browser (not the Tauri webview).
#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let url = url.trim().to_string();
    if !is_allowed_external_https_url(&url) {
        return Err("only https URLs can be opened externally".to_string());
    }
    spawn_system_browser(&url)
}

#[tauri::command]
fn open_dapp_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, DappBridgeState>,
    app_id: String,
    wallet_public_key: String,
    network: String,
) -> Result<(), String> {
    let dapp = allowed_dapp(&app_id).ok_or_else(|| "unsupported dapp".to_string())?;
    let wallet_public_key = wallet_public_key.trim().to_string();
    if !is_likely_solana_pubkey(&wallet_public_key) {
        return Err("invalid wallet public key".to_string());
    }
    let network = network.trim().to_string();
    if network.is_empty() || network.len() > 256 || network.chars().any(|ch| ch.is_control()) {
        return Err("invalid network".to_string());
    }

    let init_script = dapp_provider_script(&dapp, &wallet_public_key, &network)?;
    if let Some(existing) = app.get_webview_window(DAPP_WINDOW_LABEL) {
        let _ = existing.close();
    }
    let url = dapp
        .url
        .parse()
        .map_err(|error| format!("invalid dapp URL: {error}"))?;
    WebviewWindowBuilder::new(&app, DAPP_WINDOW_LABEL, WebviewUrl::External(url))
        .title(format!("Sol SafeKey DApp - {}", dapp.name))
        .inner_size(1220.0, 820.0)
        .resizable(true)
        .initialization_script(&init_script)
        .build()
        .map_err(|error| format!("failed to open dapp window: {error}"))?;

    let mut session = state
        .session
        .lock()
        .map_err(|_| "dapp session lock poisoned".to_string())?;
    *session = Some(DappSession {
        app_id: dapp.id.to_string(),
        app_name: dapp.name.to_string(),
        url: dapp.url.to_string(),
        wallet_public_key,
        network,
        opened_at_ms: now_ms(),
    });
    Ok(())
}

#[tauri::command]
fn dapp_submit_sign_request(
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<'_, DappBridgeState>,
    method: String,
    transaction_base64: String,
    transaction_format: String,
) -> Result<String, String> {
    if window.label() != DAPP_WINDOW_LABEL {
        return Err("dapp signing requests are only accepted from the dapp window".to_string());
    }
    let method = validate_dapp_method(&method)?;
    let transaction_base64 = validate_dapp_transaction_base64(&transaction_base64)?;
    let transaction_format = validate_transaction_format(&transaction_format)?;
    let session = state
        .session
        .lock()
        .map_err(|_| "dapp session lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "no active dapp session".to_string())?;
    if now_ms().saturating_sub(session.opened_at_ms) > 12 * 60 * 60 * 1000 {
        return Err("dapp session expired".to_string());
    }

    let request_id = dapp_request_id();
    let event = DappSignRequestEvent {
        request_id: request_id.clone(),
        app_id: session.app_id,
        app_name: session.app_name,
        app_url: session.url,
        method,
        wallet_public_key: session.wallet_public_key,
        network: session.network,
        transaction_base64,
        transaction_format,
        created_at_ms: now_ms(),
    };

    let mut requests = state
        .requests
        .lock()
        .map_err(|_| "dapp request lock poisoned".to_string())?;
    requests.retain(|_, pending| {
        now_ms().saturating_sub(pending.event.created_at_ms) <= DAPP_REQUEST_TTL_MS
            && pending.result.is_none()
    });
    requests.insert(
        request_id.clone(),
        DappPendingRequest {
            event: event.clone(),
            result: None,
        },
    );
    drop(requests);

    app.emit_to("main", DAPP_SIGN_REQUEST_EVENT, event)
        .map_err(|error| format!("failed to notify main window: {error}"))?;
    Ok(request_id)
}

#[tauri::command]
fn dapp_poll_sign_request(
    state: tauri::State<'_, DappBridgeState>,
    request_id: String,
) -> Result<DappPollResponse, String> {
    let request_id = request_id.trim();
    let mut requests = state
        .requests
        .lock()
        .map_err(|_| "dapp request lock poisoned".to_string())?;
    let Some(pending) = requests.get(request_id) else {
        return Ok(DappPollResponse {
            status: "expired",
            result: None,
        });
    };
    if now_ms().saturating_sub(pending.event.created_at_ms) > DAPP_REQUEST_TTL_MS {
        requests.remove(request_id);
        return Ok(DappPollResponse {
            status: "expired",
            result: None,
        });
    }
    if let Some(result) = pending.result.clone() {
        requests.remove(request_id);
        return Ok(DappPollResponse {
            status: if result.approved { "approved" } else { "rejected" },
            result: Some(result),
        });
    }
    Ok(DappPollResponse {
        status: "pending",
        result: None,
    })
}

#[tauri::command]
fn resolve_dapp_sign_request(
    state: tauri::State<'_, DappBridgeState>,
    request_id: String,
    result: DappSignResult,
) -> Result<(), String> {
    let request_id = request_id.trim();
    let mut requests = state
        .requests
        .lock()
        .map_err(|_| "dapp request lock poisoned".to_string())?;
    let pending = requests
        .get_mut(request_id)
        .ok_or_else(|| "dapp signing request is no longer pending".to_string())?;
    if now_ms().saturating_sub(pending.event.created_at_ms) > DAPP_REQUEST_TTL_MS {
        requests.remove(request_id);
        return Err("dapp signing request expired".to_string());
    }
    pending.result = Some(result);
    Ok(())
}

#[tauri::command]
fn pick_source_directory() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .set_title("Select Solana Program Source Directory")
        .pick_folder()
        .map(|path| path.to_string_lossy().to_string()))
}

fn safe_download_filename(filename: &str) -> Result<String, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty()
        || trimmed.len() > 160
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.chars().any(|ch| ch.is_control())
    {
        return Err("invalid download filename".to_string());
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        return Err(
            "download filename may only contain letters, numbers, dots, dashes, and underscores"
                .to_string(),
        );
    }
    Ok(trimmed.to_string())
}

fn downloads_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME directory is unavailable".to_string())?;
    Ok(home.join("Downloads"))
}

fn non_overwriting_path(directory: &Path, filename: &str) -> PathBuf {
    let candidate = directory.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(filename);
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 1..10_000 {
        let next_name = match extension {
            Some(extension) if !extension.is_empty() => format!("{stem}-{index}.{extension}"),
            _ => format!("{stem}-{index}"),
        };
        let next_path = directory.join(next_name);
        if !next_path.exists() {
            return next_path;
        }
    }
    directory.join(format!("{stem}-{}", uuid_like_timestamp()))
}

fn uuid_like_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "now".to_string())
}

#[tauri::command]
fn save_download_file(filename: String, content: String) -> Result<String, String> {
    if content.len() > MAX_DOWNLOAD_FILE_BYTES {
        return Err("download file is too large".to_string());
    }
    let filename = safe_download_filename(&filename)?;
    let directory = downloads_dir()?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("failed to create Downloads directory: {error}"))?;
    let path = non_overwriting_path(&directory, &filename);
    std::fs::write(&path, content.as_bytes())
        .map_err(|error| format!("failed to write download file: {error}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_download_file_location(path: String) -> Result<(), String> {
    let raw_path = PathBuf::from(path.trim());
    if !raw_path.is_absolute() {
        return Err("download path must be absolute".to_string());
    }
    let downloads = downloads_dir()?;
    let canonical_downloads = downloads
        .canonicalize()
        .map_err(|error| format!("failed to read Downloads directory: {error}"))?;
    let canonical_path = raw_path
        .canonicalize()
        .map_err(|error| format!("download file is unavailable: {error}"))?;
    if !canonical_path.starts_with(&canonical_downloads) {
        return Err("can only open files saved under Downloads".to_string());
    }
    reveal_file_in_system_file_manager(&canonical_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DappBridgeState::default())
        .invoke_handler(tauri::generate_handler![
            proxy_api_request,
            open_external_url,
            open_dapp_window,
            dapp_submit_sign_request,
            dapp_poll_sign_request,
            resolve_dapp_sign_request,
            pick_source_directory,
            save_download_file,
            open_download_file_location
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lc_rs::{
        encoding::{AsDer, PublicKeyX509Der},
        rsa::{KeySize, OaepPrivateDecryptingKey, PrivateDecryptingKey},
    };

    #[test]
    fn secure_body_envelope_round_trips_with_backend_key_format() {
        let private_key = PrivateDecryptingKey::generate(KeySize::Rsa2048).unwrap();
        let public_key = private_key.public_key();
        let public_key_der = AsDer::<PublicKeyX509Der<'static>>::as_der(&public_key).unwrap();
        let public_key_pem = pem_rfc7468::encode_string(
            "PUBLIC KEY",
            pem_rfc7468::LineEnding::LF,
            public_key_der.as_ref(),
        )
        .unwrap();
        assert!(public_key_pem.starts_with("-----BEGIN PUBLIC KEY-----\n"));
        assert!(public_key_pem.ends_with("-----END PUBLIC KEY-----\n"));

        let body = r#"{"password":"not-a-real-password","value":42}"#;
        let envelope: serde_json::Value =
            serde_json::from_str(&encrypt_secure_body(body, &public_key_pem).unwrap()).unwrap();
        assert_eq!(envelope["version"].as_u64(), Some(1));

        let encrypted_key = BASE64
            .decode(envelope["encrypted_key"].as_str().unwrap())
            .unwrap();
        let iv = BASE64.decode(envelope["iv"].as_str().unwrap()).unwrap();
        let ciphertext = BASE64
            .decode(envelope["ciphertext"].as_str().unwrap())
            .unwrap();

        let private_key = OaepPrivateDecryptingKey::new(private_key).unwrap();
        let mut decrypted_key = Zeroizing::new(vec![0_u8; private_key.min_output_size()]);
        let aes_key_len = private_key
            .decrypt(
                &OAEP_SHA256_MGF1SHA256,
                &encrypted_key,
                decrypted_key.as_mut_slice(),
                None,
            )
            .unwrap()
            .len();
        assert_eq!(aes_key_len, 32);

        let cipher = Aes256Gcm::new_from_slice(&decrypted_key[..aes_key_len]).unwrap();
        let plaintext = Zeroizing::new(
            cipher
                .decrypt(Nonce::from_slice(&iv), ciphertext.as_ref())
                .unwrap(),
        );
        assert_eq!(plaintext.as_slice(), body.as_bytes());
    }

    #[test]
    fn secure_body_rejects_non_spki_pem_label() {
        let invalid_pem = "-----BEGIN RSA PUBLIC KEY-----\nAA==\n-----END RSA PUBLIC KEY-----\n";
        let error = encrypt_secure_body("{}", invalid_pem).unwrap_err();
        assert_eq!(error, "invalid secure API public key PEM label");
    }

    #[test]
    fn external_url_guard_accepts_only_structural_https_urls() {
        assert!(is_allowed_external_https_url(
            "https://solscan.io/tx/abc?cluster=devnet"
        ));
        assert!(!is_allowed_external_https_url("http://solscan.io/tx/abc"));
        assert!(!is_allowed_external_https_url("https://"));
        assert!(!is_allowed_external_https_url("https://@example.com"));
        assert!(!is_allowed_external_https_url(
            "https://user:pass@example.com"
        ));
        assert!(!is_allowed_external_https_url("https://.example.com"));
        assert!(!is_allowed_external_https_url("https://example.com."));
        assert!(!is_allowed_external_https_url("https://example.com\n.evil"));
    }
}
