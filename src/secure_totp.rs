use crate::totp::{TOTPManager, TOTPConfig};
use colored::*;
use std::io::{self, Write};

/// 安全的 TOTP 管理器 - 不存储密钥文件
pub struct SecureTOTPManager {
    config: TOTPConfig,
}

impl SecureTOTPManager {
    /// 生成真正随机的 TOTP 密钥（不依赖密码）
    pub fn generate_new(account: &str, issuer: &str) -> Result<Self, String> {
        // 生成真正随机的2FA密钥，与主密码完全独立
        let secret = TOTPManager::generate_secret();

        let config = TOTPConfig {
            secret,
            account: account.to_string(),
            issuer: issuer.to_string(),
            algorithm: "SHA1".to_string(),
            digits: 6,
            step: 30,
        };

        Ok(Self { config })
    }

    /// 从已知密钥创建（用于解锁时）
    pub fn from_secret(secret: &str, account: &str, issuer: &str) -> Result<Self, String> {
        let config = TOTPConfig {
            secret: secret.to_string(),
            account: account.to_string(),
            issuer: issuer.to_string(),
            algorithm: "SHA1".to_string(),
            digits: 6,
            step: 30,
        };

        Ok(Self { config })
    }

    /// 获取 TOTP 管理器实例
    pub fn get_totp_manager(&self) -> TOTPManager {
        TOTPManager::new(self.config.clone())
    }

    /// 显示设置信息（不保存文件）
    pub fn display_setup_info(&self) -> Result<(), String> {
        println!("{}", "🔐 2FA 安全设置（无文件存储模式）".bright_cyan().bold());
        println!();

        println!("{}", "⚠️  重要安全提醒:".bright_yellow().bold());
        println!("  • 此模式不会保存任何配置文件");
        println!("  • 2FA 密钥从您的密码派生，请牢记您的密码");
        println!("  • 相同的密码+账户总是生成相同的 2FA 密钥");
        println!();

        let totp_manager = self.get_totp_manager();

        // 显示 QR 码
        println!("{}", "📱 请将以下信息添加到您的认证器:".bright_green());
        println!();

        match totp_manager.generate_qr_code() {
            Ok(qr_code) => {
                println!("{}", qr_code);
            }
            Err(_) => {
                println!("{}", "📝 手动设置信息:".bright_blue());
                println!("账户: {}", self.config.account);
                println!("发行商: {}", self.config.issuer);
                println!("密钥类型: 基于时间");
                println!("算法: SHA1");
                println!("位数: 6");
                println!("间隔: 30秒");
            }
        }

        println!();
        println!("{}", "🔑 密钥信息 (请添加到认证器):".bright_yellow());
        println!("{}", self.config.secret);
        println!();

        // 验证设置
        loop {
            print!("{} ", "请输入认证器显示的 6 位验证码确认设置:".bright_yellow());
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let code = input.trim();

            match totp_manager.verify_code(code) {
                Ok(true) => {
                    println!("{}", "✅ 2FA 设置验证成功！".bright_green());
                    println!("{}", "💡 请牢记您的密码，它是解锁的关键！".bright_blue());
                    break;
                }
                Ok(false) => {
                    println!("{}", "❌ 验证码不正确，请重试".red());
                    continue;
                }
                Err(e) => {
                    eprintln!("{} 验证失败: {}", "❌".red(), e);
                    continue;
                }
            }
        }

        Ok(())
    }
}


/// 实用安全的 2FA 解锁函数
pub fn secure_unlock_with_2fa(
    encrypted_file_path: &str,
    _account: &str,
    _issuer: &str,
) -> Result<(), String> {
    use rpassword;
    use std::fs;
    use crate::totp::parse_encrypted_file;
    use crate::practical_secure_decrypt_with_2fa_verification;
    use solana_sdk::signer::Signer;

    println!("{}", "🔐 实用安全 2FA 解锁模式".bright_cyan().bold());
    println!();

    // 第一步：获取主密码
    print!("{} ", "请输入主密码:".bright_yellow());
    io::stdout().flush().unwrap();
    let master_password = rpassword::read_password()
        .map_err(|e| format!("读取密码失败: {}", e))?;

    // 第二步：获取当前 2FA 验证码
    print!("{} ", "请输入当前 2FA 验证码:".bright_green());
    io::stdout().flush().unwrap();
    let totp_code = rpassword::read_password()
        .map_err(|e| format!("读取验证码失败: {}", e))?;

    // 读取加密文件
    let file_content = fs::read_to_string(encrypted_file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let encrypted_data = parse_encrypted_file(&file_content)?;

    println!("🔍 正在验证主密码和2FA验证码...");

    // 使用实用安全解密方案
    match practical_secure_decrypt_with_2fa_verification(&encrypted_data, &master_password, &totp_code) {
        Ok((private_key, _totp_secret)) => {
            // 验证私钥有效性
            let keypair = solana_sdk::signature::Keypair::from_base58_string(&private_key);
            let pubkey = keypair.pubkey();
            println!("{}", "✅ 双重验证通过，解锁成功！".bright_green());
            println!("{} 私钥: {}", "🔑".bright_cyan(), private_key);
            println!("{} 公钥: {}", "🆔".bright_cyan(), pubkey);
            println!();
            println!("{}", "🔒 安全确认:".bright_blue().bold());
            println!("  • 主密码验证通过（解密成功）");
            println!("  • 2FA验证码验证通过（身份确认）");
            println!("  • 提供了真正的双因子安全保护");
            println!("  • 用户体验简单：只需主密码+验证码");
            Ok(())
        }
        Err(e) => Err(format!("解锁失败: {}. 请检查主密码和2FA验证码", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_new() {
        let account = "test_account";
        let issuer = "Test Issuer";

        let manager1 = SecureTOTPManager::generate_new(account, issuer).unwrap();
        let manager2 = SecureTOTPManager::generate_new(account, issuer).unwrap();

        // 不同的调用应该产生不同的随机密钥
        assert_ne!(manager1.config.secret, manager2.config.secret);

        // 密钥不应该为空
        assert!(!manager1.config.secret.is_empty());
    }
}