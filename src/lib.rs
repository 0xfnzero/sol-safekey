use base64::engine::general_purpose;
use base64::Engine;
use ring::aead::{Aad, LessSafeKey, UnboundKey, Nonce, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    iv: String,
    ciphertext: String,
}

/// 加密私钥，返回一个包含IV和密文的完整加密字符串
pub fn encrypt_key(secret_key: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    let rng = SystemRandom::new();

    // 生成 96 位随机 Nonce
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes).map_err(|_| "Failed to generate nonce".to_string())?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // 创建加密器
    let unbound_key = UnboundKey::new(&AES_256_GCM, encryption_key)
        .map_err(|_| "Invalid encryption key".to_string())?;
    let key = LessSafeKey::new(unbound_key);

    // 加密数据
    let mut secret_key_bytes = secret_key.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut secret_key_bytes)
        .map_err(|_| "Encryption failed".to_string())?;

    let encrypted_data = EncryptedData {
        iv: general_purpose::STANDARD.encode(nonce_bytes),
        ciphertext: general_purpose::STANDARD.encode(secret_key_bytes),
    };

    serde_json::to_string(&encrypted_data)
        .map_err(|_| "Failed to serialize encrypted data".to_string())
}

/// 解密私钥，从完整的加密字符串中提取IV和密文进行解密
pub fn decrypt_key(encrypted_data: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    // 解析加密数据
    let data: EncryptedData = serde_json::from_str(encrypted_data)
        .map_err(|_| "Invalid encrypted data format".to_string())?;

    // 解码 Base64 编码的 nonce 和密文
    let nonce_bytes = general_purpose::STANDARD.decode(&data.iv)
        .map_err(|_| "Invalid IV format".to_string())?;
    let mut ciphertext = general_purpose::STANDARD.decode(&data.ciphertext)
        .map_err(|_| "Invalid ciphertext format".to_string())?;

    // 创建解密器
    let unbound_key = UnboundKey::new(&AES_256_GCM, encryption_key)
        .map_err(|_| "Invalid encryption key".to_string())?;
    let key = LessSafeKey::new(unbound_key);
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes)
        .map_err(|_| "Invalid nonce".to_string())?;

    // 解密数据
    let plaintext = key.open_in_place(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| "Decryption failed - incorrect password or corrupted data".to_string())?;

    // 转换为字符串
    String::from_utf8(plaintext.to_vec())
        .map_err(|_| "Invalid UTF-8 data in decrypted content".to_string())
}

/// 兼容性函数：保持原有的API以支持旧代码
pub fn encrypt_key_legacy(secret_key: &str, encryption_key: &[u8; 32]) -> (String, String) {
    let result = encrypt_key(secret_key, encryption_key).expect("Encryption failed");
    let data: EncryptedData = serde_json::from_str(&result).expect("Parse failed");
    (data.iv, data.ciphertext)
}

/// 兼容性函数：保持原有的API以支持旧代码
pub fn decrypt_key_legacy(nonce_base64: &str, ciphertext_base64: &str, encryption_key: &[u8; 32]) -> String {
    let encrypted_data = EncryptedData {
        iv: nonce_base64.to_string(),
        ciphertext: ciphertext_base64.to_string(),
    };
    let data_str = serde_json::to_string(&encrypted_data).expect("Serialize failed");
    decrypt_key(&data_str, encryption_key).expect("Decryption failed")
}