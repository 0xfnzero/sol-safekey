use base64::engine::general_purpose;
use base64::Engine;
use ring::digest;

/// 简单的XOR加密，使用密钥和固定nonce
fn xor_encrypt_decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());

    // 使用密钥生成一个更长的keystream
    let mut keystream = Vec::new();
    let mut i: u32 = 0;
    while keystream.len() < data.len() {
        // 混合密钥和位置信息来生成keystream
        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(key);
        ctx.update(&i.to_le_bytes());
        let hash = ctx.finish();
        keystream.extend_from_slice(hash.as_ref());
        i += 1;
    }

    // XOR加密/解密
    for (i, &byte) in data.iter().enumerate() {
        result.push(byte ^ keystream[i % keystream.len()]);
    }

    result
}

/// 加密私钥，返回简单的base64编码字符串
pub fn encrypt_key(secret_key: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    let data = secret_key.as_bytes();
    let encrypted = xor_encrypt_decrypt(data, encryption_key);
    Ok(general_purpose::STANDARD.encode(encrypted))
}

/// 解密私钥，从base64字符串解密
pub fn decrypt_key(encrypted_data: &str, encryption_key: &[u8; 32]) -> Result<String, String> {
    // 解码 Base64
    let ciphertext = general_purpose::STANDARD.decode(encrypted_data)
        .map_err(|_| "Invalid encrypted data format".to_string())?;

    // 解密数据
    let decrypted = xor_encrypt_decrypt(&ciphertext, encryption_key);

    // 转换为字符串
    String::from_utf8(decrypted)
        .map_err(|_| "Invalid UTF-8 data in decrypted content".to_string())
}

