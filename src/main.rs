use clap::{Parser, Subcommand};
use sol_safekey::{encrypt_key, decrypt_key};
use solana_sdk::signer::Signer;
use std::{fs, process, env};
use serde_json;
use colored::*;
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose};

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
    /// 初始化工具，生成随机加密密钥
    Init {
        /// 强制重新生成密钥（覆盖现有的.env文件）
        #[arg(long)]
        force: bool,
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

    // 配置命令部分
    println!("  {} {}", "⚙️  配置命令 | Configuration Commands:".bright_red().bold(), "");
    println!("    {} {}", "init".bright_green(), "        初始化工具，生成随机加密密钥".white());
    println!();

    // 使用示例
    println!("  {} {}", "📖 使用示例 | Usage Examples:".bright_red().bold(), "");
    println!("    {} {}", "sol-safekey".bright_green(), "init".bright_white());
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

    // 常用选项
    println!("  {} {}", "📝 常用选项 | Common Options:".bright_red().bold(), "");
    println!("    {} {}", "-o, --output".bright_magenta(), "     输出文件路径（gen命令使用）".white());
    println!("    {} {}", "-s, --segments".bright_magenta(), "   私钥分段数量".white());
    println!("    {} {}", "-p, --password".bright_magenta(), "   密码（最多10位）".white());
    println!("    {} {}", "-k, --private-key".bright_magenta(), " 私钥字符串（encrypt命令使用）".white());
    println!("    {} {}", "-e, --encrypted-key".bright_magenta(), " 加密数据（decrypt命令使用）".white());
    println!("    {} {}", "-f, --file-path".bright_magenta(), "  文件路径（unlock命令使用）".white());
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

fn generate_encryption_key(password: &str) -> [u8; 32] {
    // 补齐到10位
    let padded_password = format!("{:0<10}", password);

    // 从环境变量读取基础密钥，如果没有则使用默认值
    let base_key = env::var("SOL_SAFEKEY_MASTER_KEY")
        .unwrap_or_else(|_| "my_secret_key_32_bytes_encryptio".to_string());

    // 确保基础密钥长度为32字节
    let mut encryption_key = [0u8; 32];
    let base_key_bytes = base_key.as_bytes();
    for i in 0..32 {
        encryption_key[i] = base_key_bytes.get(i).copied().unwrap_or(0);
    }

    // 将密码混入加密密钥
    for (i, c) in padded_password.chars().enumerate() {
        if i < 32 {
            encryption_key[i] ^= c as u8; // 使用XOR而不是直接替换
        }
    }

    encryption_key
}

fn encrypt_private_key(private_key: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key(password);
    encrypt_key(private_key, &encryption_key)
}

fn decrypt_private_key(encrypted_data: &str, password: &str) -> Result<String, String> {
    let encryption_key = generate_encryption_key(password);
    decrypt_key(encrypted_data, &encryption_key)
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
    // 尝试加载 .env 文件（如果存在）
    let _ = dotenv::dotenv();

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
                    // 验证密码长度
                    if pwd.len() > 10 {
                        eprintln!("❌ 错误: 密码长度不能超过10位");
                        process::exit(1);
                    }

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
            // 验证密码长度
            if password.len() > 10 {
                eprintln!("❌ 错误: 密码长度不能超过10位");
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
            // 验证密码长度
            if password.len() > 10 {
                eprintln!("❌ 错误: 密码长度不能超过10位");
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
        Commands::Init { force } => {
            let env_file = ".env";

            // 检查 .env 文件是否已存在
            if fs::metadata(env_file).is_ok() && !force {
                println!("⚠️  .env 文件已存在！");
                println!("💡 如果要重新生成密钥，请使用 --force 参数");
                println!("   例如: sol-safekey init --force");
                return;
            }

            // 生成32字节的随机密钥
            let mut master_key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut master_key);

            // 将字节转换为base64字符串以便存储
            let master_key_b64 = general_purpose::STANDARD.encode(&master_key);

            // 创建.env文件内容
            let env_content = format!(
                "# Sol-SafeKey 主密钥配置文件\n# 警告: 请妥善保管此文件，不要泄露给他人！\n# 此密钥用于加密/解密您的私钥\n\nSOL_SAFEKEY_MASTER_KEY={}\n",
                master_key_b64
            );

            // 写入.env文件
            match fs::write(env_file, env_content) {
                Ok(()) => {
                    if *force {
                        println!("✅ 已重新生成主密钥！");
                    } else {
                        println!("✅ 初始化完成！已生成随机主密钥");
                    }
                    println!("📄 配置文件: {}", env_file);
                    println!("🔑 主密钥: {}", master_key_b64);
                    println!();
                    println!("🌍 环境变量设置:");
                    println!("   变量名: SOL_SAFEKEY_MASTER_KEY");
                    println!("   变量值: {}", master_key_b64);
                    println!();
                    println!("💡 建议将环境变量添加到系统配置文件:");
                    println!("   macOS/Linux (zsh): echo 'export SOL_SAFEKEY_MASTER_KEY=\"{}\"' >> ~/.zshrc", master_key_b64);
                    println!("   macOS/Linux (bash): echo 'export SOL_SAFEKEY_MASTER_KEY=\"{}\"' >> ~/.bashrc", master_key_b64);
                    println!("   然后重新启动终端或运行: source ~/.zshrc");
                    println!();
                    println!("⚠️  重要提醒:");
                    println!("  1. 请备份 .env 文件和环境变量到安全位置");
                    println!("  2. 不要将 .env 文件提交到版本控制系统");
                    println!("  3. 如果丢失此密钥，将无法解密现有的加密私钥");
                    println!("  4. 环境变量优先级高于 .env 文件");
                }
                Err(e) => {
                    eprintln!("❌ 创建 .env 文件失败: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}