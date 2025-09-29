use base64::engine::general_purpose;
use base64::Engine;
use ring::digest;

pub mod totp;
pub mod secure_totp;
pub mod hardware_fingerprint;
pub mod security_question;

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

/// 从主密码和2FA密钥派生固定关联值
pub fn derive_2fa_salt(master_password: &str, totp_secret: &str) -> String {
    use ring::digest;

    // 使用主密码和TOTP密钥生成固定的盐值
    let combined = format!("{}:TOTP_SALT:{}", master_password, totp_secret);
    let context = digest::digest(&digest::SHA256, combined.as_bytes());

    // 转换为hex字符串作为固定盐值
    hex::encode(context.as_ref())
}

/// 从主密码和2FA密钥生成固定的加密密钥
pub fn generate_2fa_encryption_key_stable(master_password: &str, totp_secret: &str) -> [u8; 32] {
    use ring::digest;

    // 组合主密码和TOTP密钥生成固定密钥
    let combined = format!("{}:2FA_STABLE_KEY:{}", master_password, totp_secret);
    let context = digest::digest(&digest::SHA256, combined.as_bytes());

    let mut key = [0u8; 32];
    key.copy_from_slice(context.as_ref());
    key
}

/// 双重加密：第一层用固定密钥，第二层用当前验证码
pub fn double_encrypt(data: &str, master_password: &str, totp_secret: &str, totp_code: &str) -> Result<String, String> {
    // 第一层：用固定密钥加密
    let stable_key = generate_2fa_encryption_key_stable(master_password, totp_secret);
    let first_encrypted = encrypt_key(data, &stable_key)?;

    // 第二层：用当前验证码再加密
    let totp_key = generate_encryption_key_simple(totp_code);
    let double_encrypted = encrypt_key(&first_encrypted, &totp_key)?;

    Ok(double_encrypted)
}

/// 双重解密：需要固定密钥和当前验证码
pub fn double_decrypt(encrypted_data: &str, master_password: &str, totp_secret: &str, totp_code: &str) -> Result<String, String> {
    // 第一层：用当前验证码解密
    let totp_key = generate_encryption_key_simple(totp_code);
    let first_decrypted = decrypt_key(encrypted_data, &totp_key)?;

    // 第二层：用固定密钥解密
    let stable_key = generate_2fa_encryption_key_stable(master_password, totp_secret);
    let final_decrypted = decrypt_key(&first_decrypted, &stable_key)?;

    Ok(final_decrypted)
}


/// 实用安全方案：主密码加密 + 2FA验证码验证
/// 设计原则：
/// 1. 用主密码加密私钥（可长期解密）
/// 2. 2FA密钥从主密码派生（保持一致性）
/// 3. 解锁时验证当前2FA验证码（真正的2FA）
/// 4. 用户只需输入：主密码 + 当前验证码
pub fn practical_secure_encrypt(
    private_key: &str,
    master_password: &str,
    account: &str,
    issuer: &str,
) -> Result<String, String> {
    use serde_json::json;

    // 从主密码派生2FA密钥（保证一致性）
    let _totp_secret = derive_totp_secret_from_password(master_password, account, issuer)?;

    // 创建数据包：私钥 + 派生信息
    let data_package = json!({
        "private_key": private_key,
        "account": account,
        "issuer": issuer,
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let package_str = data_package.to_string();

    // 使用主密码加密（简单可靠）
    let encryption_key = generate_encryption_key_simple(master_password);
    let encrypted = encrypt_key(&package_str, &encryption_key)?;

    Ok(encrypted)
}

/// 实用安全解密（主密码 + 当前2FA验证码验证）
pub fn practical_secure_decrypt_with_2fa_verification(
    encrypted_data: &str,
    master_password: &str,
    current_totp_code: &str,
) -> Result<(String, String), String> {
    // 第一步：用主密码解密
    let encryption_key = generate_encryption_key_simple(master_password);
    let package_str = decrypt_key(encrypted_data, &encryption_key)?;

    // 第二步：解析数据包
    let package: serde_json::Value = serde_json::from_str(&package_str)
        .map_err(|_| "数据包格式错误或主密码错误")?;

    let private_key = package["private_key"]
        .as_str()
        .ok_or("私钥数据缺失")?
        .to_string();

    let account = package["account"]
        .as_str()
        .ok_or("账户信息缺失")?;

    let issuer = package["issuer"]
        .as_str()
        .ok_or("发行商信息缺失")?;

    // 第三步：从主密码重新派生2FA密钥
    let derived_totp_secret = derive_totp_secret_from_password(master_password, account, issuer)?;

    // 第四步：验证当前2FA验证码
    verify_current_totp_code(&derived_totp_secret, current_totp_code)?;

    Ok((private_key, derived_totp_secret))
}

/// 从主密码派生TOTP密钥（保证一致性）
fn derive_totp_secret_from_password(password: &str, account: &str, issuer: &str) -> Result<String, String> {
    use ring::pbkdf2;
    use data_encoding::BASE32_NOPAD;
    use std::num::NonZeroU32;

    let salt = format!("sol-safekey-totp-{}-{}", issuer, account);
    let iterations = NonZeroU32::new(100_000)
        .ok_or("Invalid iteration count")?;

    let mut secret = [0u8; 20]; // 160 bits for TOTP
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt.as_bytes(),
        password.as_bytes(),
        &mut secret,
    );

    Ok(BASE32_NOPAD.encode(&secret))
}

/// 从硬件指纹 + 主密码派生 2FA 密钥（确定性）
pub fn derive_totp_secret_from_hardware_and_password(
    hardware_fingerprint: &str,
    master_password: &str,
    account: &str,
    issuer: &str,
) -> Result<String, String> {
    use ring::pbkdf2;
    use data_encoding::BASE32_NOPAD;
    use std::num::NonZeroU32;

    // 组合硬件指纹和主密码作为密钥材料
    let key_material = format!("{}::{}", hardware_fingerprint, master_password);
    let salt = format!("sol-safekey-2fa-{}-{}", issuer, account);
    let iterations = NonZeroU32::new(100_000)
        .ok_or("Invalid iteration count")?;

    let mut secret = [0u8; 20]; // 160 bits for TOTP
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt.as_bytes(),
        key_material.as_bytes(),
        &mut secret,
    );

    Ok(BASE32_NOPAD.encode(&secret))
}

/// 验证当前TOTP验证码
fn verify_current_totp_code(totp_secret: &str, current_code: &str) -> Result<(), String> {
    use crate::totp::{TOTPConfig, TOTPManager};

    let config = TOTPConfig {
        secret: totp_secret.to_string(),
        account: "wallet".to_string(),
        issuer: "Sol-SafeKey".to_string(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        step: 30,
    };

    let totp_manager = TOTPManager::new(config);

    match totp_manager.verify_code(current_code) {
        Ok(true) => Ok(()),
        Ok(false) => Err("验证失败，请检查主密码、安全问题答案或2FA验证码".to_string()),
        Err(e) => Err(format!("验证失败: {}", e)),
    }
}

/// 兼容旧版本的单参数函数（用于非2FA场景）
pub fn generate_encryption_key_simple(password: &str) -> [u8; 32] {
    use ring::digest;
    let context = digest::digest(&digest::SHA256, password.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(context.as_ref());
    key
}

// ============================================================================
// 新的三因子加密方案：硬件指纹 + 主密码 + 安全问题答案
// ============================================================================

/// 生成三因子组合加密密钥
/// 输入：硬件指纹 + 主密码 + 安全问题答案
pub fn generate_triple_factor_key(
    hardware_fingerprint: &str,
    master_password: &str,
    security_answer: &str,
) -> [u8; 32] {
    use ring::pbkdf2;
    use std::num::NonZeroU32;

    // 组合三个因子作为密钥材料
    let key_material = format!(
        "HW:{}|PASS:{}|QA:{}",
        hardware_fingerprint,
        master_password,
        security_answer.trim().to_lowercase()
    );

    // 使用 PBKDF2 派生强密钥
    let salt = b"sol-safekey-triple-factor-v1";
    let iterations = NonZeroU32::new(200_000).unwrap();

    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        key_material.as_bytes(),
        &mut key,
    );

    key
}

/// 新的加密方案：使用三因子加密私钥和2FA密钥
/// 输入：私钥, 2FA密钥, 硬件指纹, 主密码, 安全问题索引, 安全问题答案
pub fn encrypt_with_triple_factor(
    private_key: &str,
    twofa_secret: &str,
    hardware_fingerprint: &str,
    master_password: &str,
    question_index: usize,
    security_answer: &str,
) -> Result<String, String> {
    use serde_json::json;

    // 生成三因子加密密钥
    let encryption_key = generate_triple_factor_key(
        hardware_fingerprint,
        master_password,
        security_answer,
    );

    // 创建数据包：包含私钥、2FA密钥、安全问题索引
    let data_package = json!({
        "private_key": private_key,
        "twofa_secret": twofa_secret,
        "question_index": question_index,
        "version": "triple_factor_v1",
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let package_str = data_package.to_string();

    // 使用三因子密钥加密
    let encrypted = encrypt_key(&package_str, &encryption_key)?;

    Ok(encrypted)
}

/// 新的解密方案：使用三因子解密并验证2FA验证码
/// 输入：加密数据, 硬件指纹, 主密码, 安全问题答案, 当前2FA验证码
pub fn decrypt_with_triple_factor_and_2fa(
    encrypted_data: &str,
    hardware_fingerprint: &str,
    master_password: &str,
    security_answer: &str,
    twofa_code: &str,
) -> Result<(String, String, usize), String> {
    // 生成三因子解密密钥
    let decryption_key = generate_triple_factor_key(
        hardware_fingerprint,
        master_password,
        security_answer,
    );

    // 解密数据
    let decrypted = decrypt_key(encrypted_data, &decryption_key)
        .map_err(|_| "解密失败，请检查主密码、安全问题答案是否正确")?;

    // 解析 JSON 数据包
    let data: serde_json::Value = serde_json::from_str(&decrypted)
        .map_err(|_| "解密失败，请检查主密码、安全问题答案是否正确")?;

    let private_key = data["private_key"]
        .as_str()
        .ok_or("缺少私钥数据")?
        .to_string();

    let twofa_secret = data["twofa_secret"]
        .as_str()
        .ok_or("缺少2FA密钥数据")?
        .to_string();

    let question_index = data["question_index"]
        .as_u64()
        .ok_or("缺少安全问题索引")? as usize;

    // 验证2FA验证码
    verify_current_totp_code(&twofa_secret, twofa_code)?;

    Ok((private_key, twofa_secret, question_index))
}

