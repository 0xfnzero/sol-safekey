use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use aws_lc_rs::rsa::{OaepPublicEncryptingKey, PublicEncryptingKey, OAEP_SHA256_MGF1SHA256};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// Must match `DEFAULT_API_PORT` in `src/lib/api.ts`
const SOL_SAFEKEY_API_PORT: u16 = 3841;
const MAX_PROXY_BODY_BYTES: usize = 12 * 1024 * 1024;
const MAX_DOWNLOAD_FILE_BYTES: usize = 4 * 1024 * 1024;
const MAX_SECURE_PUBLIC_KEY_PEM_BYTES: usize = 2 * 1024;
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
        .invoke_handler(tauri::generate_handler![
            proxy_api_request,
            open_external_url,
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
