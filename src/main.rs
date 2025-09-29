use clap::{Parser, Subcommand};
use sol_safekey::{
    encrypt_key, decrypt_key, generate_encryption_key_simple,
    encrypt_with_triple_factor, decrypt_with_triple_factor_and_2fa,
    derive_totp_secret_from_hardware_and_password,
    totp::*, hardware_fingerprint::*, security_question::*
};
use solana_sdk::signer::Signer;
use std::{fs, process, io::{self, Write}};
use std::time::{SystemTime, UNIX_EPOCH};
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
    /// 生成keypair格式私钥
    GenKeypair {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "keypair.json")]
        output: String,
    },
    /// 生成字符串格式私钥（可选加密）
    GenKey {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "keystore.json")]
        output: String,
        /// 分段数量
        #[arg(short = 's', long, default_value = "1")]
        segments: usize,
        /// 密码（可选，最多10位）- 提供密码则生成加密私钥
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// 生成加密的keystore文件
    GenKeystore {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "keystore.json")]
        output: String,
        /// 密码（最多10位）
        #[arg(short = 'p', long)]
        password: String,
    },
    /// 加密私钥
    Encrypt {
        /// 私钥字符串
        #[arg(short = 'k', long)]
        private_key: String,
        /// 密码（最多10位）
        #[arg(short = 'p', long)]
        password: String,
    },
    /// 解密私钥
    Decrypt {
        /// 加密数据
        #[arg(short = 'e', long)]
        encrypted_key: String,
        /// 密码
        #[arg(short = 'p', long)]
        password: String,
    },
    /// 解锁文件中的私钥
    Unlock {
        /// 文件路径
        #[arg(short = 'f', long)]
        file_path: String,
        /// 密码
        #[arg(short = 'p', long)]
        password: String,
    },
    /// 查看私钥对应的钱包地址
    Address {
        /// 私钥字符串（明文私钥）
        #[arg(short = 'k', long, group = "input")]
        private_key: Option<String>,
        /// 加密的私钥字符串
        #[arg(short = 'e', long, group = "input")]
        encrypted_key: Option<String>,
        /// 文件路径（包含私钥的文件）
        #[arg(short = 'f', long, group = "input")]
        file_path: Option<String>,
        /// 密码（解密加密私钥时需要）
        #[arg(short = 'p', long)]
        password: Option<String>,
    },
    /// 设置 TOTP 2FA 认证
    SetupTotp {
        /// 账户名称
        #[arg(short = 'a', long, default_value = "master-key")]
        account: String,
        /// 输出配置文件路径
        #[arg(short = 'o', long, default_value = "totp-config.json")]
        output: String,
    },
    /// 生成 TOTP 验证码
    GenerateTotp {
        /// TOTP 配置文件路径
        #[arg(short = 'c', long, default_value = "totp-config.json")]
        config_file: String,
    },
    /// 使用 TOTP 生成加密私钥
    GenSecureTotp {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "keystore.json")]
        output: String,
        /// TOTP 配置文件路径
        #[arg(short = 'c', long, default_value = "totp-config.json")]
        totp_config: String,
    },
    /// 使用 TOTP 解锁私钥
    UnlockTotp {
        /// 加密文件路径
        #[arg(short = 'f', long)]
        file_path: String,
        /// TOTP 配置文件路径
        #[arg(short = 'c', long, default_value = "totp-config.json")]
        totp_config: String,
    },
    /// 调试 TOTP 时间窗口
    DebugTotp {
        /// TOTP 配置文件路径
        #[arg(short = 'c', long, default_value = "totp-config.json")]
        totp_config: String,
        /// 要检查的时间窗口数量
        #[arg(short = 'w', long, default_value = "5")]
        windows: i32,
    },
    /// 设置 2FA 认证（硬件指纹 + 主密码 + 安全问题）
    #[command(name = "setup-2fa")]
    Setup2FA,
    /// 使用三因子加密生成安全钱包
    #[command(name = "gen-2fa-wallet")]
    Gen2FAWallet {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "secure-wallet.json")]
        output: String,
    },
    /// 使用三因子 + 2FA 验证码解锁钱包
    #[command(name = "unlock-2fa-wallet")]
    Unlock2FAWallet {
        /// 加密文件路径
        #[arg(short = 'f', long)]
        file_path: String,
    },
}


fn print_colored_help() {
    println!("{}", "Solana安全密钥管理工具 | Solana Security Key Management Tool".bright_cyan().bold());
    println!();

    println!("{}", "Usage:".bright_yellow().bold());
    println!("  {} {}", "sol-safekey".bright_green(), "<COMMAND>".bright_white());
    println!();

    println!("{}", "Commands:".bright_yellow().bold());

    // 生成命令部分
    println!("  {} {}", "🔑 生成命令 | Generation Commands:".bright_red().bold(), "");
    println!("    {} {}", "gen-keypair".bright_green(), " 生成keypair格式私钥".white());
    println!("    {} {}", "gen-key".bright_green(), "     生成字符串格式私钥（可选加密）".white());
    println!("    {} {}", "gen-keystore".bright_green(), "生成加密的keystore文件".white());
    println!("    {} {}", "unlock".bright_green(), "      从加密文件中解锁私钥".white());
    println!();

    // 加密解密命令部分
    println!("  {} {}", "🔐 加密/解密命令 | Encryption/Decryption Commands:".bright_red().bold(), "");
    println!("    {} {}", "encrypt".bright_green(), "     加密已有私钥（需要提供私钥字符串）".white());
    println!("    {} {}", "decrypt".bright_green(), "     解密加密的私钥字符串".white());
    println!();

    // 查询命令部分
    println!("  {} {}", "🔍 查询命令 | Query Commands:".bright_red().bold(), "");
    println!("    {} {}", "address".bright_green(), "     查看私钥对应的钱包地址".white());
    println!();


    // 2FA 命令部分
    println!("  {} {}", "🔐 2FA 三因子安全命令 | 2FA Triple-Factor Security:".bright_red().bold(), "");
    println!("    {} {}", "setup-2fa".bright_green(), "        设置 2FA（硬件指纹 + 主密码 + 安全问题）".white());
    println!("    {} {}", "gen-2fa-wallet".bright_green(), "    生成三因子钱包 + keystore备份".white());
    println!("                       生成两个文件: 1) 三因子钱包(仅当前设备) 2) keystore备份(跨设备)");
    println!("    {} {}", "unlock-2fa-wallet".bright_green(), "  解锁三因子钱包（需要主密码 + 安全问题 + 2FA验证码）".white());
    println!();

    // 使用示例
    println!("  {} {}", "📖 使用示例 | Usage Examples:".bright_red().bold(), "");
    println!("    {} {}", "sol-safekey".bright_green(), "gen-keypair -o wallet.json".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "gen-key -s 3 -o keys.json".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "gen-keystore -p mypass -o keystore.json".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "unlock -f keystore.json -p mypass".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "encrypt -k YOUR_PRIVATE_KEY -p mypass".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "decrypt -e ENCRYPTED_KEY -p mypass".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "address -k YOUR_PRIVATE_KEY".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "address -e ENCRYPTED_KEY -p mypass".bright_white());
    println!("    {} {}", "sol-safekey".bright_green(), "address -f keys.json".bright_white());
    println!();
    println!("  {} {}", "🔥 2FA 三因子工作流程 | 2FA Triple-Factor Workflow:".bright_magenta().bold(), "");
    println!("    {} {}", "1. sol-safekey".bright_green(), "setup-2fa                        # 首次设置（扫描二维码 + 设置安全问题）".bright_white());
    println!("    {} {}", "2. sol-safekey".bright_green(), "gen-2fa-wallet -o wallet.json     # 生成钱包（两个文件）".bright_white());
    println!("    {} {}", "   输出:".bright_blue(), "wallet.json (三因子) + <地址前缀>_keystore.json (跨设备备份)".bright_white());
    println!("    {} {}", "3a. sol-safekey".bright_green(), "unlock-2fa-wallet -f wallet.json  # 解锁三因子钱包".bright_white());
    println!("    {} {}", "3b. sol-safekey".bright_green(), "unlock -f <前缀>_keystore.json -p <密码>  # 跨设备解锁备份".bright_white());
    println!();

    // 常用选项
    println!("  {} {}", "📝 常用选项 | Common Options:".bright_red().bold(), "");
    println!("    {} {}", "-o, --output".bright_magenta(), "     输出文件路径（gen命令使用）".white());
    println!("    {} {}", "-s, --segments".bright_magenta(), "   私钥分段数量".white());
    println!("    {} {}", "-p, --password".bright_magenta(), "   密码（最多10位）".white());
    println!("    {} {}", "-k, --private-key".bright_magenta(), " 私钥字符串（encrypt命令使用）".white());
    println!("    {} {}", "-e, --encrypted-key".bright_magenta(), " 加密数据（decrypt命令使用）".white());
    println!("    {} {}", "-f, --file-path".bright_magenta(), "  文件路径（unlock命令使用）".white());
    println!();

    // 重要说明
    println!("{} {}", "🔑 2FA 主密码说明 | Master Password Info:".bright_cyan().bold(), "");
    println!("{} {}", "  • 主密码:".bright_white(), "您自己设置的强密码，用于派生 2FA 密钥".white());
    println!("{} {}", "  • 输入方式:".bright_white(), "程序会提示时输入，输入时不显示字符（安全输入）".white());
    println!("{} {}", "  • 重要性:".bright_white(), "主密码丢失将无法恢复，请务必记住".bright_red());
    println!("{} {}", "  • 一致性:".bright_white(), "相同主密码总是生成相同的 2FA 密钥".white());
    println!();

    // 提示信息
    println!("{} {}", "💡 提示 | Tip:".bright_yellow().bold(), "使用 'sol-safekey <command> --help' 查看具体命令的详细选项".bright_white());
    println!("{} {}", " ".repeat(13), "Use 'sol-safekey <command> --help' for detailed options of specific commands".bright_white());
    println!();

    // 选项
    println!("{}", "Options:".bright_yellow().bold());
    println!("  {} {}", "-h, --help".bright_magenta(), "     Print help".white());
    println!("  {} {}", "-V, --version".bright_magenta(), "  Print version".white());
}


fn encrypt_private_key(private_key: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key_simple(password);
    encrypt_key(private_key, &encryption_key)
}

fn decrypt_private_key(encrypted_data: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key_simple(password);
    decrypt_key(encrypted_data, &encryption_key)
}

/// 检查密码强度
fn check_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("密码长度至少需要8位".to_string());
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

fn generate_new_keypair() -> (String, String) {
    let keypair = solana_sdk::signature::Keypair::new();
    let private_key = keypair.to_base58_string();
    let public_key = keypair.pubkey().to_string();
    (private_key, public_key)
}

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

fn save_keypair_to_file(keypair: &solana_sdk::signature::Keypair, file_path: &str) -> Result<(), String> {
    let private_key_bytes = keypair.to_bytes();
    let data = serde_json::json!(private_key_bytes.to_vec());

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
}

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

    // 如果用户请求帮助或没有提供命令，显示彩色帮助
    if cli.help || cli.command.is_none() {
        print_colored_help();
        return;
    }

    let command = cli.command.unwrap();
    match &command {
        Commands::GenKeypair { output } => {
            println!("🔑 正在生成新的Solana keypair...");
            println!();

            // 生成新的密钥对
            let keypair = solana_sdk::signature::Keypair::new();
            let public_key = keypair.pubkey().to_string();

            // 显示结果
            println!("✅ 成功生成新的Solana keypair!");
            println!();
            println!("🆔 公钥地址:");
            println!("{}", public_key);
            println!();

            // 保存到文件
            match save_keypair_to_file(&keypair, output) {
                Ok(()) => {
                    println!("💾 Keypair已保存到文件: {}", output);
                    println!("⚠️  警告: 请妥善保管你的keypair文件，不要泄露给他人！");
                }
                Err(e) => {
                    eprintln!("❌ 保存文件失败: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::GenKey { output, segments, password } => {
            // 生成新的密钥对
            let (private_key, public_key) = generate_new_keypair();

            match password {
                Some(pwd) => {
                    println!("🔑 正在生成新的加密Solana私钥...");
                    println!();

                    // 显示公钥
                    println!("✅ 成功生成新的Solana密钥对!");
                    println!();
                    println!("🆔 公钥地址:");
                    println!("{}", public_key);
                    println!();

                    // 加密私钥
                    match encrypt_private_key(&private_key, pwd) {
                        Ok(encrypted_data) => {
                            // 分段处理加密数据
                            let encrypted_segments = split_private_key_into_segments(&encrypted_data, *segments);

                            println!("🔒 加密私钥已生成但不在终端显示（安全考虑）");
                            if *segments > 1 {
                                println!("📄 加密私钥将分{}段保存到文件中", segments);
                            }
                            println!();

                            // 保存到文件
                            match save_encrypted_key_to_file(&encrypted_data, &public_key, &encrypted_segments, output) {
                                Ok(()) => {
                                    println!("💾 加密私钥已保存到文件: {}", output);
                                    println!("⚠️  警告: 请妥善保管你的加密私钥文件和密码！");
                                }
                                Err(e) => {
                                    eprintln!("❌ 保存文件失败: {}", e);
                                    process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ 加密失败: {}", e);
                            process::exit(1);
                        }
                    }
                }
                None => {
                    println!("🔑 正在生成新的Solana私钥字符串...");
                    println!();

                    // 显示结果
                    println!("✅ 成功生成新的Solana私钥字符串!");
                    println!();
                    println!("🆔 公钥地址:");
                    println!("{}", public_key);
                    println!();

                    // 分段处理
                    let key_segments = split_private_key_into_segments(&private_key, *segments);

                    println!("🔒 私钥已生成但不在终端显示（安全考虑）");
                    if *segments > 1 {
                        println!("📄 私钥将分{}段保存到文件中", segments);
                    }
                    println!();

                    // 保存到文件
                    match save_private_key_string_to_file(&private_key, &public_key, &key_segments, output) {
                        Ok(()) => {
                            println!("💾 私钥字符串已保存到文件: {}", output);
                            println!("⚠️  警告: 请妥善保管你的私钥文件，不要泄露给他人！");
                        }
                        Err(e) => {
                            eprintln!("❌ 保存文件失败: {}", e);
                            process::exit(1);
                        }
                    }
                }
            }
        }
        Commands::GenKeystore { output, password } => {
            // 检查密码强度
            if let Err(e) = check_password_strength(password) {
                eprintln!("❌ 密码强度不足: {}", e);
                process::exit(1);
            }

            // 生成新的密钥对
            let (private_key, public_key) = generate_new_keypair();

            println!("🔑 正在生成新的加密Solana私钥...");
            println!();

            // 显示公钥
            println!("✅ 成功生成新的Solana密钥对!");
            println!();
            println!("🆔 公钥地址:");
            println!("{}", public_key);
            println!();

            // 加密私钥
            match encrypt_private_key(&private_key, password) {
                Ok(encrypted_data) => {
                    println!("🔒 加密私钥已生成但不在终端显示（安全考虑）");
                    println!();

                    // 保存到文件
                    match save_keystore_to_file(&encrypted_data, &public_key, output) {
                        Ok(()) => {
                            println!("💾 加密私钥已保存到文件: {}", output);
                            println!("⚠️  警告: 请妥善保管你的加密私钥文件和密码！");
                        }
                        Err(e) => {
                            eprintln!("❌ 保存文件失败: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ 加密失败: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Encrypt { private_key, password } => {
            // 检查密码强度
            if let Err(e) = check_password_strength(password) {
                eprintln!("❌ 密码强度不足: {}", e);
                process::exit(1);
            }

            // 验证私钥格式
            if private_key.is_empty() {
                eprintln!("❌ 错误: 私钥不能为空");
                process::exit(1);
            }

            // 加密私钥
            match encrypt_private_key(private_key, password) {
                Ok(encrypted_data) => {
                    println!("✅ 加密成功!");
                    println!();
                    println!("🔐 加密后的完整密钥:");
                    println!("{}", encrypted_data);
                    println!();
                    println!("💡 提示: 请妥善保存这个加密密钥，解密时需要用到");
                }
                Err(e) => {
                    eprintln!("❌ 加密失败: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Decrypt { encrypted_key, password } => {
            // 验证密码长度
            if password.len() > 10 {
                eprintln!("❌ 错误: 密码长度不能超过10位");
                process::exit(1);
            }

            // 验证加密数据
            if encrypted_key.is_empty() {
                eprintln!("❌ 错误: 加密密钥不能为空");
                process::exit(1);
            }

            // 解密私钥
            match decrypt_private_key(encrypted_key, password) {
                Ok(decrypted_key) => {
                    // 尝试验证解密后的私钥是否为有效的Solana私钥
                    let keypair = solana_sdk::signature::Keypair::from_base58_string(&decrypted_key);
                    let pubkey = keypair.pubkey();

                    println!("✅ 解密成功!");
                    println!();
                    println!("🔑 解密后的私钥:");
                    println!("{}", decrypted_key);
                    println!();
                    println!("🆔 对应的公钥:");
                    println!("{}", pubkey);
                }
                Err(e) => {
                    eprintln!("❌ 解密失败: {}", e);
                    eprintln!("💡 可能的原因: 密码错误或加密数据已损坏");
                    process::exit(1);
                }
            }
        }
        Commands::Unlock { file_path, password } => {
            // 验证密码长度
            if password.len() > 10 {
                eprintln!("❌ 错误: 密码长度不能超过10位");
                process::exit(1);
            }

            // 读取文件内容
            let file_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("❌ 读取文件失败: {}", e);
                    process::exit(1);
                }
            };

            // 尝试解析JSON文件
            let encrypted_data = match serde_json::from_str::<serde_json::Value>(&file_content) {
                Ok(json) => {
                    if let Some(encrypted_key) = json.get("encrypted_private_key") {
                        encrypted_key.as_str().unwrap_or("").to_string()
                    } else {
                        eprintln!("❌ 错误: 文件格式不正确，缺少encrypted_private_key字段");
                        process::exit(1);
                    }
                }
                Err(_) => {
                    // 如果不是JSON格式，尝试直接作为加密数据使用
                    file_content.trim().to_string()
                }
            };

            if encrypted_data.is_empty() {
                eprintln!("❌ 错误: 文件中没有找到有效的加密数据");
                process::exit(1);
            }

            // 解密私钥
            match decrypt_private_key(&encrypted_data, password) {
                Ok(decrypted_key) => {
                    // 尝试验证解密后的私钥是否为有效的Solana私钥
                    let keypair = solana_sdk::signature::Keypair::from_base58_string(&decrypted_key);
                    let pubkey = keypair.pubkey();

                    println!("✅ 文件解密成功!");
                    println!();
                    println!("📄 文件路径: {}", file_path);
                    println!();
                    println!("🔑 解密后的私钥:");
                    println!("{}", decrypted_key);
                    println!();
                    println!("🆔 对应的公钥:");
                    println!("{}", pubkey);
                }
                Err(e) => {
                    eprintln!("❌ 解密失败: {}", e);
                    eprintln!("💡 可能的原因: 密码错误或加密数据已损坏");
                    process::exit(1);
                }
            }
        }
        Commands::Address { private_key, encrypted_key, file_path, password } => {
            let final_private_key = if let Some(pk) = private_key {
                // 直接使用明文私钥
                pk.clone()
            } else if let Some(ek) = encrypted_key {
                // 解密加密的私钥
                match password {
                    Some(pwd) => {
                        // 验证密码长度
                        if pwd.len() > 10 {
                            eprintln!("❌ 错误: 密码长度不能超过10位");
                            process::exit(1);
                        }

                        match decrypt_private_key(ek, pwd) {
                            Ok(decrypted) => decrypted,
                            Err(e) => {
                                eprintln!("❌ 解密失败: {}", e);
                                eprintln!("💡 可能的原因: 密码错误或加密数据已损坏");
                                process::exit(1);
                            }
                        }
                    }
                    None => {
                        eprintln!("❌ 错误: 解密加密私钥需要提供密码 (-p)");
                        process::exit(1);
                    }
                }
            } else if let Some(fp) = file_path {
                // 从文件读取私钥
                let file_content = match fs::read_to_string(fp) {
                    Ok(content) => content,
                    Err(e) => {
                        eprintln!("❌ 读取文件失败: {}", e);
                        process::exit(1);
                    }
                };

                // 尝试解析JSON文件
                match serde_json::from_str::<serde_json::Value>(&file_content) {
                    Ok(json) => {
                        if let Some(private_key_value) = json.get("private_key") {
                            // 普通私钥文件
                            private_key_value.as_str().unwrap_or("").to_string()
                        } else if let Some(encrypted_key_value) = json.get("encrypted_private_key") {
                            // 加密私钥文件
                            let encrypted_data = encrypted_key_value.as_str().unwrap_or("").to_string();
                            if encrypted_data.is_empty() {
                                eprintln!("❌ 错误: 文件中没有找到有效的加密数据");
                                process::exit(1);
                            }

                            match password {
                                Some(pwd) => {
                                    // 验证密码长度
                                    if pwd.len() > 10 {
                                        eprintln!("❌ 错误: 密码长度不能超过10位");
                                        process::exit(1);
                                    }

                                    match decrypt_private_key(&encrypted_data, pwd) {
                                        Ok(decrypted) => decrypted,
                                        Err(e) => {
                                            eprintln!("❌ 解密失败: {}", e);
                                            eprintln!("💡 可能的原因: 密码错误或加密数据已损坏");
                                            process::exit(1);
                                        }
                                    }
                                }
                                None => {
                                    eprintln!("❌ 错误: 解密加密私钥文件需要提供密码 (-p)");
                                    process::exit(1);
                                }
                            }
                        } else if json.is_array() {
                            // Keypair格式文件（字节数组）
                            let bytes_vec: Vec<u8> = json.as_array()
                                .and_then(|arr| {
                                    arr.iter()
                                        .map(|v| v.as_u64().and_then(|n| if n <= 255 { Some(n as u8) } else { None }))
                                        .collect::<Option<Vec<u8>>>()
                                })
                                .unwrap_or_else(|| {
                                    eprintln!("❌ 错误: 无效的keypair字节数组格式");
                                    process::exit(1);
                                });

                            if bytes_vec.len() != 64 {
                                eprintln!("❌ 错误: keypair字节数组长度应为64，实际为{}", bytes_vec.len());
                                process::exit(1);
                            }

                            // 从字节数组重建keypair并获取私钥
                            let mut bytes_array = [0u8; 64];
                            bytes_array.copy_from_slice(&bytes_vec);

                            // 前32字节是私钥，后32字节是公钥
                            let secret_key: [u8; 32] = bytes_array[0..32].try_into().unwrap();
                            let keypair = solana_sdk::signature::Keypair::new_from_array(secret_key);
                            keypair.to_base58_string()
                        } else {
                            eprintln!("❌ 错误: 文件格式不正确，缺少private_key、encrypted_private_key字段或不是有效的keypair格式");
                            process::exit(1);
                        }
                    }
                    Err(_) => {
                        eprintln!("❌ 错误: 无法解析JSON文件");
                        process::exit(1);
                    }
                }
            } else {
                eprintln!("❌ 错误: 请提供私钥 (-k)、加密私钥 (-e) 或文件路径 (-f)");
                process::exit(1);
            };

            // 验证私钥并获取公钥地址
            let keypair = solana_sdk::signature::Keypair::from_base58_string(&final_private_key);
            let pubkey = keypair.pubkey();
            println!("✅ 私钥验证成功!");
            println!();
            println!("🆔 钱包地址:");
            println!("{}", pubkey);
        }
        Commands::SetupTotp { account, output } => {
            println!("{}", "🔐 设置 TOTP 2FA 认证...".bright_cyan().bold());
            println!();

            // 生成新的密钥
            let secret = TOTPManager::generate_secret();
            let config = TOTPConfig {
                secret: secret.clone(),
                account: account.clone(),
                ..Default::default()
            };

            let totp_manager = TOTPManager::new(config.clone());

            // 显示 QR 码
            println!("{}", "📱 请使用谷歌认证器、Authy 或其他 TOTP 应用扫描以下 QR 码：".bright_yellow());
            println!();
            match totp_manager.generate_qr_code() {
                Ok(qr_code) => {
                    println!("{}", qr_code);
                }
                Err(e) => {
                    eprintln!("{} QR 码生成失败: {}", "❌".red(), e);
                    println!("{}", "📝 请手动输入以下信息：".bright_yellow());
                    println!("{}", totp_manager.get_manual_setup_info());
                }
            }

            println!();
            println!("{} 或者手动输入密钥: {}", "🔑".bright_cyan(), secret.bright_white());
            println!();

            // 验证设置
            loop {
                print!("{} ", "请输入认证器显示的 6 位验证码以确认设置:".bright_yellow());
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let code = input.trim();

                match totp_manager.verify_code(code) {
                    Ok(true) => {
                        println!("{}", "✅ TOTP 设置成功！".bright_green());
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

            // 保存配置
            match save_totp_config(&config, output) {
                Ok(()) => {
                    println!("{} TOTP 配置已保存到: {}", "💾".bright_green(), output);
                    println!("{} 警告: 请安全备份此配置文件和您的认证器应用！", "⚠️".bright_yellow());
                }
                Err(e) => {
                    eprintln!("{} 保存配置失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
        Commands::GenerateTotp { config_file } => {
            let config = match load_totp_config(config_file) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            let totp_manager = TOTPManager::new(config);

            match totp_manager.generate_current_code() {
                Ok(code) => {
                    println!("{} 当前 TOTP 验证码: {}", "🔢".bright_cyan(), code.bright_white().bold());

                    // 显示剩余有效时间
                    let remaining = totp_manager.get_remaining_time();
                    println!("{} 剩余有效时间: {} 秒", "⏰".bright_yellow(), remaining.to_string().bright_white());
                }
                Err(e) => {
                    eprintln!("{} 生成验证码失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
        Commands::GenSecureTotp { output, totp_config } => {
            let config = match load_totp_config(totp_config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            let totp_manager = TOTPManager::new(config);

            // 提示用户输入 TOTP 验证码
            print!("{} ", "请输入 TOTP 验证码:".bright_yellow());
            io::stdout().flush().unwrap();
            let code = rpassword::read_password().unwrap();

            // 验证 TOTP 码
            match totp_manager.verify_code(&code) {
                Ok(true) => {
                    // 生成密钥对
                    let (private_key, public_key) = generate_new_keypair();

                    // 使用 TOTP 码作为密码进行加密
                    match encrypt_private_key(&private_key, &code) {
                        Ok(encrypted_data) => {
                            println!("{}", "✅ 密钥生成成功！".bright_green());
                            println!("{} 公钥地址: {}", "🆔".bright_cyan(), public_key);

                            // 保存加密私钥
                            if let Err(e) = save_encrypted_key_to_file(&encrypted_data, &public_key, &[], output) {
                                eprintln!("{} 保存文件失败: {}", "❌".red(), e);
                                process::exit(1);
                            }
                            println!("{} 加密私钥已保存到: {}", "💾".bright_green(), output);
                            println!("{} 使用 TOTP 验证码解锁时，请确保在同一个 30 秒时间窗口内！", "⚠️".bright_yellow());
                        }
                        Err(e) => {
                            eprintln!("{} 加密失败: {}", "❌".red(), e);
                            process::exit(1);
                        }
                    }
                }
                Ok(false) => {
                    eprintln!("{}", "❌ 2FA 验证码不正确".red());
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("{} 2FA 验证失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
        Commands::UnlockTotp { file_path, totp_config } => {
            let config = match load_totp_config(totp_config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            let totp_manager = TOTPManager::new(config);

            // 读取加密文件
            let file_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("{} 读取文件失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            let encrypted_data = match parse_encrypted_file(&file_content) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    process::exit(1);
                }
            };

            // 提示用户输入 TOTP 验证码
            print!("{} ", "请输入 TOTP 验证码解锁:".bright_yellow());
            io::stdout().flush().unwrap();
            let code = rpassword::read_password().unwrap();

            // 首先尝试标准验证（±30秒窗口）
            match totp_manager.verify_code(&code) {
                Ok(true) => {
                    match decrypt_private_key(&encrypted_data, &code) {
                        Ok(decrypted_key) => {
                            // 验证私钥有效性
                            let keypair = solana_sdk::signature::Keypair::from_base58_string(&decrypted_key);
                            let pubkey = keypair.pubkey();

                            println!("{}", "✅ 解锁成功！".bright_green());
                            println!("{} 私钥: {}", "🔑".bright_cyan(), decrypted_key);
                            println!("{} 公钥: {}", "🆔".bright_cyan(), pubkey);
                        }
                        Err(e) => {
                            eprintln!("{} 标准解密失败: {}", "❌".red(), e);
                            println!("{} 尝试扩展时间窗口解锁...", "🔄".bright_yellow());

                            // 尝试扩展时间窗口
                            if let Ok((is_valid, debug_info)) = totp_manager.verify_code_extended(&code) {
                                if is_valid {
                                    println!("{} 扩展验证通过，尝试解密安全时间窗口的验证码...", "✅".bright_green());

                                    // 尝试解密多个时间窗口的验证码
                                    if let Ok(codes) = totp_manager.get_codes_for_windows(1) {
                                        for (timestamp, window_code) in codes {
                                            if let Ok(decrypted_key) = decrypt_private_key(&encrypted_data, &window_code) {
                                                // 验证私钥有效性
                                                let keypair = solana_sdk::signature::Keypair::from_base58_string(&decrypted_key);
                                                let pubkey = keypair.pubkey();

                                                println!("{}", "✅ 解锁成功！(使用历史时间窗口)".bright_green());
                                                println!("{} 使用的时间戳: {}", "⏰".bright_blue(), timestamp);
                                                println!("{} 使用的验证码: {}", "🔢".bright_blue(), window_code);
                                                println!("{} 私钥: {}", "🔑".bright_cyan(), decrypted_key);
                                                println!("{} 公钥: {}", "🆔".bright_cyan(), pubkey);
                                                return;
                                            }
                                        }
                                    }
                                }

                                eprintln!("{} 所有时间窗口解密都失败了", "❌".red());
                                eprintln!("{} 调试信息:", "🔍".bright_blue());
                                eprintln!("{}", debug_info);
                            }

                            eprintln!("{} 可能的解决方案:", "💡".bright_blue());
                            eprintln!("  1. 确保系统时间准确（安全时间窗口：±30秒）");
                            eprintln!("  2. 检查 TOTP 配置文件是否正确");
                            eprintln!("  3. 使用 debug-totp 命令查看当前可用的验证码");
                            eprintln!("  4. 重新生成加密私钥");
                            process::exit(1);
                        }
                    }
                }
                Ok(false) => {
                    println!("{} 标准验证失败，尝试扩展时间窗口...", "⚠️".bright_yellow());

                    // 尝试扩展验证
                    match totp_manager.verify_code_extended(&code) {
                        Ok((true, _debug_info)) => {
                            println!("{} 扩展验证通过！", "✅".bright_green());
                            // 已经通过验证，但标准解密可能失败，尝试多窗口解密
                            if let Ok(codes) = totp_manager.get_codes_for_windows(1) {
                                for (_timestamp, window_code) in codes {
                                    if window_code == code {
                                        if let Ok(decrypted_key) = decrypt_private_key(&encrypted_data, &window_code) {
                                            let keypair = solana_sdk::signature::Keypair::from_base58_string(&decrypted_key);
                                            let pubkey = keypair.pubkey();

                                            println!("{}", "✅ 解锁成功！".bright_green());
                                            println!("{} 私钥: {}", "🔑".bright_cyan(), decrypted_key);
                                            println!("{} 公钥: {}", "🆔".bright_cyan(), pubkey);
                                            return;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                        Ok((false, debug_info)) => {
                            eprintln!("{}", "❌ 扩展验证也失败了".red());
                            eprintln!("{} 调试信息:", "🔍".bright_blue());
                            eprintln!("{}", debug_info);
                        }
                        Err(e) => {
                            eprintln!("{} 扩展验证出错: {}", "❌".red(), e);
                        }
                    }

                    eprintln!("{}", "❌ 2FA 验证码不正确".red());
                    eprintln!("{} 请使用 debug-totp 命令查看当前可用的验证码（时间窗口已优化为±30秒）", "💡".bright_blue());
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("{} 2FA 验证失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }
        }
        Commands::DebugTotp { totp_config, windows } => {
            let config = match load_totp_config(totp_config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    process::exit(1);
                }
            };
            let totp_manager = TOTPManager::new(config);

            println!("{}", "🔍 TOTP 时间窗口调试信息".bright_cyan().bold());
            println!();

            // 显示当前验证码
            match totp_manager.generate_current_code() {
                Ok(current_code) => {
                    println!("{} 当前 TOTP 验证码: {}", "🔢".bright_green(), current_code.bright_white().bold());
                    let remaining = totp_manager.get_remaining_time();
                    println!("{} 剩余有效时间: {} 秒", "⏰".bright_yellow(), remaining.to_string().bright_white());
                    println!();
                }
                Err(e) => {
                    eprintln!("{} 生成当前验证码失败: {}", "❌".red(), e);
                    process::exit(1);
                }
            }

            // 显示多个时间窗口的验证码
            match totp_manager.get_codes_for_windows(*windows) {
                Ok(codes) => {
                    println!("{} 时间窗口验证码列表（窗口数: {}）:", "📋".bright_blue(), windows);
                    println!();

                    for (timestamp, code) in codes {
                        let time_diff = timestamp as i64 - SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
                        let status = if time_diff.abs() <= 30 {
                            "🟢 当前"
                        } else if time_diff.abs() <= 90 {
                            "🟡 最近"
                        } else {
                            "🔴 较远"
                        };

                        println!("{} 时间戳: {} | 验证码: {} | 时差: {}秒",
                                status,
                                timestamp,
                                code.bright_white().bold(),
                                time_diff);
                    }
                    println!();
                    println!("{} 建议使用 🟢 或 🟡 标记的验证码进行解锁", "💡".bright_blue());
                }
                Err(e) => {
                    eprintln!("{} 生成时间窗口验证码失败: {}", "❌".red(), e);
                    process::exit(1);
                }
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

                    match fs::write(output, serde_json::to_string_pretty(&data).unwrap()) {
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
    }
}