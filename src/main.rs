use clap::{Parser, Subcommand};
use sol_safekey::{
    encrypt_key, decrypt_key, generate_encryption_key_simple,
    encrypt_with_triple_factor, decrypt_with_triple_factor_and_2fa,
    derive_totp_secret_from_hardware_and_password,
    totp::*, hardware_fingerprint::*, security_question::*,
};
use solana_sdk::signer::Signer;
use std::{fs, process, io::{self, Write}};
use serde_json;
use colored::*;
use rpassword;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "sol-safekey")]
#[command(about = "Solana安全密钥管理工具 | Solana Security Key Management Tool")]
#[command(disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show help message
    #[arg(long, short, help = "Show help information")]
    help: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动交互式菜单 | Start interactive menu (create/encrypt/decrypt keys)
    Start,

    /// 设置 2FA 认证（硬件指纹 + 主密码 + 安全问题）| Setup 2FA authentication
    #[command(name = "setup-2fa")]
    Setup2FA,

    /// 使用三因子加密生成安全钱包 | Generate 2FA wallet
    #[command(name = "gen-2fa-wallet")]
    Gen2FAWallet {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "secure-wallet.json")]
        output: String,
    },

    /// 使用三因子 + 2FA 验证码解锁钱包 | Unlock 2FA wallet
    #[command(name = "unlock-2fa-wallet")]
    Unlock2FAWallet {
        /// 加密文件路径
        #[arg(short = 'f', long)]
        file_path: String,
    },

    /// Solana 操作命令（使用加密私钥）| Solana operations with encrypted keys
    #[command(name = "sol-ops")]
    SolOps {
        /// 加密钱包文件路径
        #[arg(short = 'f', long)]
        file_path: String,

        #[command(subcommand)]
        command: sol_safekey::solana_utils::SolanaOpsCommand,
    },
}


/// Print colored help message with bilingual content
fn print_colored_help() {
    println!("{}", "=".repeat(60).cyan());
    println!("{}", "  Sol-SafeKey - Solana 密钥管理工具".cyan().bold());
    println!("{}", "  Solana Security Key Management Tool".cyan());
    println!("{}", "=".repeat(60).cyan());
    println!();

    println!("{}", "Usage:".bright_yellow().bold());
    println!("  {} {}", "sol-safekey".bright_green(), "start               # 启动交互式菜单（推荐）".bright_white());
    println!("  {} {}", "sol-safekey".bright_green(), "<COMMAND>           # 运行特定命令".bright_white());
    println!();

    println!("{}", "核心命令 | Core Commands:".bright_yellow().bold());
    println!();
    println!("  {} {}", "start".bright_green().bold(), "启动交互式菜单 (创建/加密/解密私钥)".white());
    println!("        Start interactive menu (create/encrypt/decrypt keys)");
    println!("        {} 无需记忆命令，选择语言后跟随提示操作即可", "→".bright_cyan());
    println!("        {} No commands to remember, just follow the prompts", "→".bright_cyan());
    println!();

    println!("{}", "高级命令 | Advanced Commands:".bright_yellow().bold());
    println!();
    println!("  {} {}", "setup-2fa".bright_green().bold(), "设置 2FA 三因子认证".white());
    println!("            Setup 2FA triple-factor authentication");
    println!("            硬件指纹 + 主密码 + 安全问题 + 2FA验证码");
    println!();

    println!("  {} {}", "gen-2fa-wallet".bright_green().bold(), "生成 2FA 加密钱包".white());
    println!("                 Generate 2FA encrypted wallet");
    println!("                 生成两个文件: 三因子钱包 + 跨设备备份");
    println!();

    println!("  {} {}", "unlock-2fa-wallet".bright_green().bold(), "解锁 2FA 钱包".white());
    println!("                    Unlock 2FA wallet");
    println!();

    println!("  {} {}", "sol-ops".bright_green().bold(), "Solana 链上操作 (转账/查询余额等)".white());
    println!("          Solana operations (transfer/check balance)");
    println!();

    println!("{}", "使用示例 | Usage Examples:".bright_cyan().bold());
    println!();
    println!("  {} 交互式模式（推荐新手使用）:", "1.".bright_yellow());
    println!("     {} {}", "$".bright_white(), "sol-safekey start".bright_green());
    println!();

    println!("  {} 2FA 三因子安全钱包:", "2.".bright_yellow());
    println!("     {} {}", "$".bright_white(), "sol-safekey setup-2fa".bright_green());
    println!("     {} {}", "$".bright_white(), "sol-safekey gen-2fa-wallet -o wallet.json".bright_green());
    println!("     {} {}", "$".bright_white(), "sol-safekey unlock-2fa-wallet -f wallet.json".bright_green());
    println!();

    println!("  {} Solana 操作:", "3.".bright_yellow());
    println!("     {} {}", "$".bright_white(), "sol-safekey sol-ops -f wallet.json balance".bright_green());
    println!("     {} {}", "$".bright_white(), "sol-safekey sol-ops -f wallet.json transfer -t <地址> -a 0.1".bright_green());
    println!();

    println!("{}", "选项 | Options:".bright_yellow().bold());
    println!("  {} {}", "-h, --help".bright_magenta(), "     显示帮助信息 | Show help information".white());
    println!("  {} {}", "-V, --version".bright_magenta(), "  显示版本信息 | Show version information".white());
    println!();

    println!("{}", "💡 提示:".bright_green().bold());
    println!("   - 大多数用户只需要 {} 命令", "start".bright_cyan().bold());
    println!("   - 运行 {} 查看某个命令的详细说明", "sol-safekey <COMMAND> --help".bright_white());
    println!();
}


/// Encrypt private key with password
fn encrypt_private_key(private_key: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key_simple(password);
    encrypt_key(private_key, &encryption_key)
}

/// Decrypt private key with password
#[allow(dead_code)]
fn decrypt_private_key(encrypted_data: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key_simple(password);
    decrypt_key(encrypted_data, &encryption_key)
}

/// Check password strength (min 10 chars, at least 3 types of: upper/lower/digit/special)
fn check_password_strength(password: &str) -> Result<(), String> {
    // 检查密码长度下限
    if password.len() < sol_safekey::MIN_PASSWORD_LENGTH {
        return Err(format!("密码长度至少需要{}位", sol_safekey::MIN_PASSWORD_LENGTH));
    }

    // 检查密码长度上限
    if password.len() > sol_safekey::MAX_PASSWORD_LENGTH {
        return Err(format!(
            "密码长度必须在{}-{}位之间",
            sol_safekey::MIN_PASSWORD_LENGTH,
            sol_safekey::MAX_PASSWORD_LENGTH
        ));
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let strength_count = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if strength_count < 3 {
        return Err("密码强度不足，需包含大写、小写、数字、特殊字符中的至少3种".to_string());
    }

    Ok(())
}

/// Generate new Solana keypair and return as (private_key, public_key) strings
fn generate_new_keypair() -> (String, String) {
    let keypair = solana_sdk::signature::Keypair::new();
    let private_key = keypair.to_base58_string();
    let public_key = keypair.pubkey().to_string();
    (private_key, public_key)
}

/// Split private key into segments for distributed storage
#[allow(dead_code)]
fn split_private_key_into_segments(private_key: &str, segments: usize) -> Vec<String> {
    // 如果segments <= 1，返回完整的私钥作为单一段
    if segments <= 1 {
        return vec![private_key.to_string()];
    }

    let len = private_key.len();
    let base_segment_size = len / segments;
    let remainder = len % segments;
    let mut result = Vec::new();
    let mut start = 0;

    for i in 0..segments {
        // 前remainder个段多分配一个字符
        let segment_size = if i < remainder {
            base_segment_size + 1
        } else {
            base_segment_size
        };

        let end = start + segment_size;
        result.push(private_key[start..end].to_string());
        start = end;
    }

    result
}

/// Save keypair to JSON file (Solana standard format)
#[allow(dead_code)]
fn save_keypair_to_file(keypair: &solana_sdk::signature::Keypair, file_path: &str) -> Result<(), String> {
    let private_key_bytes = keypair.to_bytes();
    let data = serde_json::json!(private_key_bytes.to_vec());

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
}

/// Save private key as string with segments to JSON file
#[allow(dead_code)]
fn save_private_key_string_to_file(private_key: &str, public_key: &str, segments: &[String], file_path: &str) -> Result<(), String> {
    let data = serde_json::json!({
        "private_key": private_key,
        "public_key": public_key,
        "segments": segments,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
}

/// Save encrypted key with segments to JSON file
#[allow(dead_code)]
fn save_encrypted_key_to_file(encrypted_data: &str, public_key: &str, segments: &[String], file_path: &str) -> Result<(), String> {
    let data = serde_json::json!({
        "encrypted_private_key": encrypted_data,
        "public_key": public_key,
        "segments": segments,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
}

/// Save encrypted keystore to JSON file (standard format)
#[allow(dead_code)]
fn save_keystore_to_file(encrypted_data: &str, public_key: &str, file_path: &str) -> Result<(), String> {
    let data = serde_json::json!({
        "encrypted_private_key": encrypted_data,
        "public_key": public_key,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
}

fn main() {
    let cli = Cli::parse();

    // Show help if no command or help flag
    if cli.command.is_none() || cli.help {
        print_colored_help();
        return;
    }

    match cli.command.unwrap() {
        Commands::Start => {
            if let Err(e) = sol_safekey::interactive::show_main_menu() {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        Commands::Setup2FA => {
            let account = "wallet";
            let issuer = "Sol-SafeKey";

            println!("{}", "🔐 三因子 2FA 安全设置".bright_cyan().bold());
            println!();
            println!("{}", "⚠️  安全架构说明:".bright_yellow().bold());
            println!("  • 因子1: 硬件指纹（自动收集，绑定设备）");
            println!("  • 因子2: 主密码（您设置的强密码）");
            println!("  • 因子3: 安全问题答案（防止密码泄露）");
            println!("  • 2FA密钥: 从硬件指纹+主密码派生（确定性）");
            println!("  • 解锁需要: 主密码 + 安全问题答案 + 2FA动态验证码");
            println!();

            // 步骤1: 收集硬件指纹
            println!("{}", "步骤 1/4: 收集硬件指纹...".bright_blue());
            let hardware_fp = match HardwareFingerprint::collect() {
                Ok(fp) => {
                    println!("{} 硬件指纹已收集（SHA256哈希）", "✅".bright_green());
                    println!("   指纹预览: {}...", &fp.as_str()[..16]);
                    fp
                }
                Err(e) => {
                    eprintln!("{} 收集硬件指纹失败: {}", "❌".red(), e);
                    eprintln!("   此功能需要读取系统硬件信息");
                    process::exit(1);
                }
            };
            println!();

            // 步骤2: 设置主密码（需输入两次+强度检查）
            println!("{}", "步骤 2/4: 设置主密码".bright_blue());
            let master_password = loop {
                print!("{} ", "请输入主密码:".bright_yellow());
                io::stdout().flush().unwrap();
                let password = rpassword::read_password()
                    .expect("读取密码失败");

                if password.is_empty() {
                    eprintln!("{} 主密码不能为空", "❌".red());
                    continue;
                }

                // 检查密码强度
                if let Err(e) = check_password_strength(&password) {
                    eprintln!("{} {}", "❌".red(), e);
                    continue;
                }

                print!("{} ", "请再次输入主密码确认:".bright_yellow());
                io::stdout().flush().unwrap();
                let password_confirm = rpassword::read_password()
                    .expect("读取密码失败");

                if password != password_confirm {
                    eprintln!("{} 两次输入的密码不一致", "❌".red());
                    continue;
                }

                break password;
            };

            println!("{} 主密码设置成功", "✅".bright_green());
            println!();

            // 步骤3: 设置安全问题
            println!("{}", "步骤 3/4: 设置安全问题".bright_blue());
            let (question_index, _security_answer) = match SecurityQuestion::setup_interactive() {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("{} 设置安全问题失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            println!();

            // 步骤4: 从硬件指纹和主密码派生2FA密钥
            println!("{}", "步骤 4/4: 设置 2FA 动态验证码".bright_blue());

            // 从硬件指纹和主密码派生2FA密钥（确定性）
            let twofa_secret = match derive_totp_secret_from_hardware_and_password(
                hardware_fp.as_str(),
                &master_password,
                account,
                issuer,
            ) {
                Ok(secret) => secret,
                Err(e) => {
                    eprintln!("{} 派生2FA密钥失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            let config = TOTPConfig {
                secret: twofa_secret.clone(),
                account: account.to_string(),
                issuer: issuer.to_string(),
                algorithm: "SHA1".to_string(),
                digits: 6,
                step: 30,
            };

            let totp_manager = TOTPManager::new(config.clone());

            // 显示 QR 码
            println!("{}", "📱 请使用 Google Authenticator 或 Authy 扫描以下 QR 码：".bright_yellow());
            println!();
            match totp_manager.generate_qr_code() {
                Ok(qr_code) => {
                    println!("{}", qr_code);
                }
                Err(e) => {
                    eprintln!("{} QR 码生成失败: {}", "⚠️".yellow(), e);
                    println!("{}", "📝 请手动输入以下信息：".bright_yellow());
                    println!("{}", totp_manager.get_manual_setup_info());
                }
            }

            println!();
            println!("{} 或者手动输入密钥: {}", "🔑".bright_cyan(), twofa_secret.bright_white());
            println!();

            // 验证2FA设置
            loop {
                print!("{} ", "请输入认证器显示的 6 位验证码以确认设置:".bright_yellow());
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let code = input.trim();

                match totp_manager.verify_code(code) {
                    Ok(true) => {
                        println!("{}", "✅ 2FA 验证成功！".bright_green());
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

            println!();
            println!("{}", "🎉 三因子 2FA 设置完成！".bright_green().bold());
            println!();
            println!("{}", "📝 重要信息（请妥善保管）:".bright_yellow().bold());
            println!("  • 硬件指纹: 已绑定到当前设备");
            println!("  • 安全问题: 问题 {} - {}", question_index + 1, SECURITY_QUESTIONS[question_index]);
            println!("  • 2FA密钥: 已添加到认证器");
            println!();
            println!("{}", "💡 下一步: 使用 gen-2fa-wallet 命令生成安全钱包".bright_blue());
        }
        Commands::Gen2FAWallet { output } => {
            println!("{}", "🔐 生成三因子加密钱包".bright_cyan().bold());
            println!();

            // 生成新的Solana密钥对
            let (private_key, public_key) = generate_new_keypair();
            println!("公钥: {}", public_key.bright_cyan());
            println!();

            // 步骤1: 收集硬件指纹
            let hardware_fp = match HardwareFingerprint::collect() {
                Ok(fp) => fp,
                Err(e) => {
                    eprintln!("{} 收集硬件指纹失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            // 步骤2: 输入主密码
            print!("主密码: ");
            io::stdout().flush().unwrap();
            let master_password = rpassword::read_password()
                .map_err(|e| {
                    eprintln!("{} 读取密码失败: {}", "❌".red(), e);
                    process::exit(1);
                }).unwrap();

            if master_password.is_empty() {
                eprintln!("{} 主密码不能为空", "❌".red());
                process::exit(1);
            }

            // 步骤3: 回答安全问题
            println!();
            let (question_index, security_answer) = match SecurityQuestion::setup_interactive() {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("{} 设置安全问题失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            // 步骤4: 验证2FA
            println!();
            print!("2FA验证码: ");
            io::stdout().flush().unwrap();

            // 从硬件指纹和主密码派生2FA密钥（确定性）
            let twofa_secret = match derive_totp_secret_from_hardware_and_password(
                hardware_fp.as_str(),
                &master_password,
                "wallet",
                "Sol-SafeKey",
            ) {
                Ok(secret) => secret,
                Err(e) => {
                    eprintln!("{} 派生2FA密钥失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            let config = TOTPConfig {
                secret: twofa_secret.clone(),
                account: "wallet".to_string(),
                issuer: "Sol-SafeKey".to_string(),
                algorithm: "SHA1".to_string(),
                digits: 6,
                step: 30,
            };

            let totp_manager = TOTPManager::new(config.clone());

            // 验证2FA
            loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let code = input.trim();

                match totp_manager.verify_code(code) {
                    Ok(true) => break,
                    Ok(false) => {
                        print!("{} 验证码错误，请重试: ", "❌".red());
                        io::stdout().flush().unwrap();
                        continue;
                    }
                    Err(e) => {
                        eprintln!("{} 验证失败: {}", "❌".red(), e);
                        process::exit(1);
                    }
                }
            }

            // 使用三因子加密
            println!("{}", "正在加密...".bright_blue());
            match encrypt_with_triple_factor(
                &private_key,
                &twofa_secret,
                hardware_fp.as_str(),
                &master_password,
                question_index,
                &security_answer,
            ) {
                Ok(encrypted_data) => {
                    // 保存加密钱包
                    let data = serde_json::json!({
                        "encrypted_private_key": encrypted_data,
                        "public_key": public_key,
                        "version": "triple_factor_v1",
                        "question_index": question_index,
                        "created_at": chrono::Utc::now().to_rfc3339()
                    });

                    match fs::write(&output, serde_json::to_string_pretty(&data).unwrap()) {
                        Ok(()) => {
                            println!("{} 钱包已保存: {}", "✅".bright_green(), output.bright_white());
                            println!();

                            // 生成跨设备的 keystore 备份
                            println!("{}", "生成 Keystore 备份...".bright_blue());

                            // 使用简单的主密码加密
                            match encrypt_private_key(&private_key, &master_password) {
                                Ok(keystore_encrypted) => {
                                    // 使用钱包地址前8位作为文件名前缀
                                    let addr_prefix = &public_key[..8];
                                    let keystore_filename = format!("{}_keystore.json", addr_prefix);

                                    let keystore_data = serde_json::json!({
                                        "encrypted_private_key": keystore_encrypted,
                                        "public_key": public_key,
                                        "encryption_type": "password_only",
                                        "created_at": chrono::Utc::now().to_rfc3339(),
                                        "note": "此文件可在任何设备上使用主密码解锁"
                                    });

                                    match fs::write(&keystore_filename, serde_json::to_string_pretty(&keystore_data).unwrap()) {
                                        Ok(()) => {
                                            println!("{} Keystore 备份: {}", "✅".bright_green(), keystore_filename.bright_white());
                                            println!();
                                            println!("{}", "📝 备份说明:".bright_cyan());
                                            println!("  • 此文件仅用主密码加密，可在任何设备恢复");
                                            println!("  • 恢复命令: {}", format!("sol-safekey unlock -f {} -p <主密码>", keystore_filename).bright_white());
                                            println!();
                                            println!("{} 请妥善备份此文件到多个安全位置", "⚠️".yellow());
                                        }
                                        Err(e) => {
                                            eprintln!("{} 警告: Keystore 备份保存失败: {}", "⚠️".yellow(), e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{} 警告: Keystore 备份加密失败: {}", "⚠️".yellow(), e);
                                }
                            }
                            println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());

                            println!();
                            println!("{}", "🔒 安全架构:".bright_blue().bold());
                            println!("  ✓ 硬件指纹: 绑定到当前设备");
                            println!("  ✓ 主密码: 强密码保护");
                            println!("  ✓ 安全问题: 问题 {} - {}", question_index + 1, SECURITY_QUESTIONS[question_index]);
                            println!("  ✓ 2FA验证码: 动态验证（每30秒更新）");
                            println!();
                            println!("{}", "📁 生成的文件:".bright_cyan().bold());
                            println!();
                            println!("{} 文件 1: {} (三因子加密钱包)", "1️⃣".bright_white(), output.bright_green());
                            println!("   安全等级: ⭐⭐⭐⭐⭐ (最高)");
                            println!("   限制: 仅限当前设备使用");
                            println!("   解锁需要: 硬件指纹 + 主密码 + 安全问题 + 2FA验证码");
                            println!("   解锁命令: {}", format!("sol-safekey unlock-2fa-wallet -f {}", output).bright_white());
                            println!();
                            println!("{} 文件 2: {} (Keystore跨设备备份)", "2️⃣".bright_white(), format!("{}_keystore.json", &public_key[..8]).bright_green());
                            println!("   安全等级: ⭐⭐⭐ (中等)");
                            println!("   限制: 无设备限制");
                            println!("   解锁需要: 仅需主密码");
                            println!("   解锁命令: {}", format!("sol-safekey unlock -f {}_keystore.json -p <主密码>", &public_key[..8]).bright_white());
                            println!();
                            println!("{}", "❓ 为什么需要 Keystore 备份？".bright_yellow().bold());
                            println!("  • 硬件损坏: 如果当前设备损坏，三因子钱包将无法使用");
                            println!("  • 系统重装: 重装系统后硬件指纹可能改变");
                            println!("  • 跨设备访问: 在其他电脑/服务器上需要访问钱包");
                            println!("  • 应急恢复: Keystore是最后的保险，保证资金安全");
                            println!();
                            println!("{}", "🔓 如何恢复私钥（三种方式）:".bright_cyan().bold());
                            println!();
                            println!("{} 当前设备 - 使用三因子钱包（推荐）:", "方式1".bright_green());
                            println!("   {}", format!("sol-safekey unlock-2fa-wallet -f {}", output).bright_white());
                            println!("   输入: 主密码 → 安全问题答案 → 2FA验证码");
                            println!();
                            println!("{} 任意设备 - 使用 Keystore 备份:", "方式2".bright_yellow());
                            println!("   {}", format!("sol-safekey unlock -f {}_keystore.json -p <主密码>", &public_key[..8]).bright_white());
                            println!("   仅需输入主密码即可恢复");
                            println!();
                            println!("{} 任意设备 - 查看钱包地址:", "方式3".bright_green());
                            println!("   {}", format!("sol-safekey address -f {}_keystore.json -p <主密码>", &public_key[..8]).bright_white());
                            println!();
                            println!("{}", "⚠️  重要提醒:".bright_red().bold());
                            println!("  • {} - 日常使用（最安全）", output.bright_green());
                            println!("  • {}_keystore.json - 离线冷备份（多地备份）", &public_key[..8]);
                            println!("  • 主密码务必牢记，丢失无法恢复");
                            println!("  • 建议将 Keystore 备份到 U盘/云盘/纸质 等多个地方");
                        }
                        Err(e) => {
                            eprintln!("{} 保存文件失败: {}", "❌".red(), e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} 加密失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
        Commands::Unlock2FAWallet { file_path } => {
            println!("{}", "🔐 解锁三因子加密钱包".bright_cyan().bold());
            println!();

            // 读取加密文件
            let file_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("{} 读取文件失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            // 解析JSON
            let data: serde_json::Value = match serde_json::from_str(&file_content) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("{} 文件格式错误: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            let encrypted_data = data["encrypted_private_key"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let question_index = data["question_index"]
                .as_u64()
                .unwrap_or(0) as usize;

            if encrypted_data.is_empty() {
                eprintln!("{} 加密数据缺失", "❌".red());
                process::exit(1);
            }

            // 步骤1: 收集硬件指纹
            println!("{}", "步骤 1/3: 验证硬件指纹...".bright_blue());
            let hardware_fp = match HardwareFingerprint::collect() {
                Ok(fp) => {
                    println!("{} 硬件指纹验证通过", "✅".bright_green());
                    fp
                }
                Err(e) => {
                    eprintln!("{} 硬件指纹验证失败: {}", "❌".red(), e);
                    eprintln!("   此钱包可能在其他设备上创建");
                    process::exit(1);
                }
            };
            println!();

            // 步骤2: 输入主密码
            println!("{}", "步骤 2/3: 输入主密码".bright_blue());
            print!("{} ", "请输入主密码:".bright_yellow());
            io::stdout().flush().unwrap();
            let master_password = rpassword::read_password()
                .map_err(|e| {
                    eprintln!("{} 读取密码失败: {}", "❌".red(), e);
                    process::exit(1);
                }).unwrap();
            println!();

            // 步骤3: 回答安全问题
            println!("{}", "步骤 3/3: 回答安全问题".bright_blue());
            let security_answer = match SecurityQuestion::verify_interactive(question_index) {
                Ok(answer) => answer,
                Err(e) => {
                    eprintln!("{} 安全问题验证失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            println!();

            // 步骤4: 输入当前2FA验证码
            println!("{}", "步骤 4/4: 输入 2FA 动态验证码".bright_blue());
            print!("{} ", "请输入认证器显示的 6 位验证码:".bright_yellow());
            io::stdout().flush().unwrap();
            let mut twofa_code = String::new();
            io::stdin().read_line(&mut twofa_code).unwrap();
            let twofa_code = twofa_code.trim();
            println!();

            // 使用三因子解密并验证2FA
            println!("{}", "🔓 正在解密钱包...".bright_blue());
            match decrypt_with_triple_factor_and_2fa(
                &encrypted_data,
                hardware_fp.as_str(),
                &master_password,
                &security_answer,
                twofa_code,
            ) {
                Ok((private_key, _twofa_secret, _question_idx)) => {
                    // 验证私钥有效性
                    let keypair = solana_sdk::signature::Keypair::from_base58_string(&private_key);
                    let pubkey = keypair.pubkey();

                    println!("{}", "🎉 钱包解锁成功！".bright_green().bold());
                    println!();
                    println!("{} 私钥: {}", "🔑".bright_cyan(), private_key);
                    println!("{} 公钥: {}", "🆔".bright_cyan(), pubkey);
                    println!();
                    println!("{}", "✅ 三因子验证通过:".bright_green().bold());
                    println!("  ✓ 硬件指纹匹配");
                    println!("  ✓ 主密码正确");
                    println!("  ✓ 安全问题答案正确");
                    println!("  ✓ 2FA动态验证码正确");
                }
                Err(e) => {
                    eprintln!("{} 解锁失败: {}", "❌".red(), e);
                    eprintln!();
                    eprintln!("{} 可能的原因:", "💡".bright_yellow());
                    eprintln!("  • 主密码错误");
                    eprintln!("  • 安全问题答案错误");
                    eprintln!("  • 2FA验证码错误或已过期");
                    eprintln!("  • 硬件指纹不匹配（设备不同）");
                    process::exit(1);
                }
            }
        }
        Commands::SolOps { file_path, command } => {
            // Run Solana operations with encrypted keypair
            let args = sol_safekey::solana_utils::SolanaOpsArgs {
                command: command.clone(),
            };

            let runtime = tokio::runtime::Runtime::new().unwrap();
            match runtime.block_on(sol_safekey::solana_utils::execute_solana_ops(args, &file_path)) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{} Operation failed: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
    }
}