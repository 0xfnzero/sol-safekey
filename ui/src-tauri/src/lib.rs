use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};
use rsa::{pkcs8::DecodePublicKey, Oaep, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;

/// Must match `DEFAULT_API_PORT` in `src/lib/api.ts`
const SOL_SAFEKEY_API_PORT: u16 = 3841;
const MAX_PROXY_BODY_BYTES: usize = 12 * 1024 * 1024;
const SECURE_BODY_HEADER: &str = "x-sol-safekey-secure-body";
const SECURE_BODY_VERSION: &str = "1";

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
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .map_err(|e| format!("invalid secure API public key: {}", e))?;
    let mut rng = OsRng;
    let mut aes_key = [0_u8; 32];
    rng.fill_bytes(&mut aes_key);
    let mut iv = [0_u8; 12];
    rng.fill_bytes(&mut iv);
    let cipher = Aes256Gcm::new_from_slice(&aes_key)
        .map_err(|_| "failed to initialize request encryption")?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&iv), body.as_bytes())
        .map_err(|_| "failed to encrypt request body")?;
    let encrypted_key = public_key
        .encrypt(&mut rng, Oaep::new::<Sha256>(), &aes_key)
        .map_err(|e| format!("failed to encrypt request key: {}", e))?;

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
        .timeout(std::time::Duration::from_secs(600))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![proxy_api_request])
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
