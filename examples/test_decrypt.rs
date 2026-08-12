//! Test keystore decryption with a password from CLI argument
//!
//! Usage: cargo run -p sol-safekey --example test_decrypt <password>

use sol_safekey::{decrypt_key_to_bytes, generate_encryption_key_simple, KeyManager, Signer};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let password = if args.len() >= 2 {
        args[1].clone()
    } else {
        eprintln!("Usage: cargo run -p sol-safekey --example test_decrypt <password>");
        std::process::exit(1);
    };

    let keystore_path = if args.len() >= 3 {
        args[2].clone()
    } else {
        "config/dev/keystore.json".to_string()
    };

    let content = std::fs::read_to_string(&keystore_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", keystore_path, e);
        std::process::exit(1);
    });

    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    let encrypted = data["encrypted_private_key"].as_str().unwrap();
    let pubkey = data["public_key"].as_str().unwrap();

    eprintln!("Keystore: {}", keystore_path);
    eprintln!("Public key: {}", pubkey);
    eprintln!("Password len: {}", password.len());

    let key = generate_encryption_key_simple(&password);
    eprintln!(
        "Encryption key: {:02x}{:02x}{:02x}{:02x}...",
        key[0], key[1], key[2], key[3]
    );

    match decrypt_key_to_bytes(encrypted, &key) {
        Ok(bytes) => {
            let preview: Vec<String> = bytes
                .iter()
                .take(16)
                .map(|b| format!("{:02x}", b))
                .collect();
            eprintln!(
                "Decrypted {} bytes, first 16: {}",
                bytes.len(),
                preview.join(" ")
            );
            eprintln!("is_ascii: {}", bytes.iter().all(|b| b.is_ascii()));
            eprintln!("is_utf8: {}", String::from_utf8(bytes.clone()).is_ok());

            // Check if it looks like base58
            let all_base58 = bytes.iter().all(|b| {
                let c = *b as char;
                c.is_ascii()
                    && (('1'..='9').contains(&c)
                        || ('A'..='H').contains(&c)
                        || ('J'..='N').contains(&c)
                        || ('P'..='Z').contains(&c)
                        || c.is_ascii_lowercase())
            });
            eprintln!("all_base58_chars: {}", all_base58);
        }
        Err(e) => eprintln!("decrypt_key_to_bytes failed: {}", e),
    }

    // Also try with trimmed password
    let trimmed = password.trim();
    if trimmed != password {
        eprintln!("\n--- Trying trimmed password (len={}) ---", trimmed.len());
        let key2 = generate_encryption_key_simple(trimmed);
        eprintln!(
            "Encryption key: {:02x}{:02x}{:02x}{:02x}...",
            key2[0], key2[1], key2[2], key2[3]
        );

        match decrypt_key_to_bytes(encrypted, &key2) {
            Ok(bytes) => {
                let preview: Vec<String> = bytes
                    .iter()
                    .take(16)
                    .map(|b| format!("{:02x}", b))
                    .collect();
                eprintln!(
                    "Decrypted {} bytes, first 16: {}",
                    bytes.len(),
                    preview.join(" ")
                );
                eprintln!("is_ascii: {}", bytes.iter().all(|b| b.is_ascii()));
                eprintln!("is_utf8: {}", String::from_utf8(bytes.clone()).is_ok());
            }
            Err(e) => eprintln!("decrypt failed: {}", e),
        }

        match KeyManager::keypair_from_encrypted_json(&content, trimmed) {
            Ok(kp) => {
                println!("SUCCESS with trimmed! keypair: {}", kp.pubkey());
                if kp.pubkey().to_string() == pubkey {
                    println!("Public key MATCHES!");
                }
            }
            Err(e) => eprintln!("keypair_from_encrypted_json (trimmed) failed: {}", e),
        }
    }

    match KeyManager::keypair_from_encrypted_json(&content, &password) {
        Ok(kp) => {
            println!("SUCCESS! keypair: {}", kp.pubkey());
            if kp.pubkey().to_string() == pubkey {
                println!("Public key MATCHES!");
            } else {
                println!("WARNING: mismatch! expected {}", pubkey);
            }
        }
        Err(e) => eprintln!("keypair_from_encrypted_json failed: {}", e),
    }
}
