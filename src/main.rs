use clap::{Parser, Subcommand};
use sol_safekey::{encrypt_key, decrypt_key};
use solana_sdk::signer::Signer;
use std::{fs, process};
use serde_json;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "sol-safekey")]
#[command(about = "Solana安全密钥管理工具 | Solana Security Key Management Tool")]
#[command(after_help = "🔑 生成命令 | Generation Commands:
  gen          生成Solana私钥（兼容模式） | Generate Solana private key (compatibility mode)
  gen-keypair  生成keypair格式私钥 | Generate keypair format private key
  gen-key      生成字符串格式私钥 | Generate string format private key
  gen-secure   生成加密私钥 | Generate encrypted private key

🔐 加密/解密命令 | Encryption/Decryption Commands:
  encrypt      加密已有私钥（需要提供私钥字符串） | Encrypt existing private key with password
  decrypt      解密加密的私钥字符串 | Decrypt encrypted private key string
  unlock       从加密文件中解锁私钥 | Decrypt private key from encrypted file

📖 使用示例 | Usage Examples:
  sol-safekey gen-keypair -o wallet.json
  sol-safekey gen-key -s 3 -o keys.json
  sol-safekey gen-secure -s 2 -p mypass -o secure.json
  sol-safekey encrypt -k YOUR_PRIVATE_KEY -p mypass
  sol-safekey unlock -f secure.json -p mypass

📝 常用选项 | Common Options:
  -o, --output     输出文件路径（gen命令使用） | Output file path (for gen commands)
  -s, --segments   私钥分段数量 | Number of segments to split the key
  -p, --password   密码（最多10位） | Password (max 10 characters)
  -k, --private-key 私钥字符串（encrypt命令使用） | Private key string (for encrypt command)
  -e, --encrypted-key 加密数据（decrypt命令使用） | Encrypted data (for decrypt command)
  -f, --file-path  文件路径（unlock命令使用） | File path (for unlock command)

💡 提示 | Tip: 使用 'sol-safekey <command> --help' 查看具体命令的详细选项
Use 'sol-safekey <command> --help' for detailed options of specific commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 生成新的Solana私钥（兼容模式）
    Gen {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "private_key.json")]
        output: String,
        /// 分段数量
        #[arg(short = 's', long, default_value = "1")]
        segments: usize,
    },
    /// 生成keypair格式私钥
    GenKeypair {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "keypair.json")]
        output: String,
    },
    /// 生成字符串格式私钥
    GenKey {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "private-key.json")]
        output: String,
        /// 分段数量
        #[arg(short = 's', long, default_value = "1")]
        segments: usize,
    },
    /// 生成加密私钥
    GenSecure {
        /// 输出文件路径
        #[arg(short = 'o', long, default_value = "enc-private-key.json")]
        output: String,
        /// 分段数量
        #[arg(short = 's', long, default_value = "1")]
        segments: usize,
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
}


fn generate_encryption_key(password: &str) -> [u8; 32] {
    // 补齐到10位
    let padded_password = format!("{:0<10}", password);

    // 将密码混入加密密钥
    let mut encryption_key: [u8; 32] = *b"my_secret_key_32_bytes_encryptio";
    for (i, c) in padded_password.chars().enumerate() {
        if i < 32 {
            encryption_key[i] = c as u8;
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

fn save_private_key_to_file(private_key: &str, public_key: &str, file_path: &str) -> Result<(), String> {
    let data = serde_json::json!({
        "private_key": private_key,
        "public_key": public_key,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    fs::write(file_path, serde_json::to_string_pretty(&data).unwrap())
        .map_err(|e| format!("无法保存文件: {}", e))
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Gen { output, segments } => {
            println!("🔑 正在生成新的Solana私钥...");
            println!();

            // 生成新的密钥对
            let (private_key, public_key) = generate_new_keypair();

            // 显示结果
            println!("✅ 成功生成新的Solana密钥对!");
            println!();
            println!("🆔 公钥地址:");
            println!("{}", public_key);
            println!();

            // 根据segments参数决定是否分段显示
            if *segments > 1 {
                let key_segments = split_private_key_into_segments(&private_key, *segments);
                println!("🔐 私钥 (分{}段显示):", segments);
                for (i, segment) in key_segments.iter().enumerate() {
                    println!("段{}: {}", i + 1, segment);
                }
                println!();
            }

            println!("🔐 完整私钥:");
            println!("{}", private_key);
            println!();

            // 保存到文件
            match save_private_key_to_file(&private_key, &public_key, output) {
                Ok(()) => {
                    println!("💾 私钥已保存到文件: {}", output);
                    println!("⚠️  警告: 请妥善保管你的私钥文件，不要泄露给他人！");
                }
                Err(e) => {
                    eprintln!("❌ 保存文件失败: {}", e);
                    process::exit(1);
                }
            }
        }
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
        Commands::GenKey { output, segments } => {
            println!("🔑 正在生成新的Solana私钥字符串...");
            println!();

            // 生成新的密钥对
            let (private_key, public_key) = generate_new_keypair();

            // 显示结果
            println!("✅ 成功生成新的Solana私钥字符串!");
            println!();
            println!("🆔 公钥地址:");
            println!("{}", public_key);
            println!();

            // 分段处理
            let key_segments = split_private_key_into_segments(&private_key, *segments);

            if *segments > 1 {
                println!("🔐 私钥 (分{}段显示):", segments);
                for (i, segment) in key_segments.iter().enumerate() {
                    println!("段{}: {}", i + 1, segment);
                }
                println!();
            }

            println!("🔐 完整私钥:");
            println!("{}", private_key);
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
        Commands::GenSecure { output, segments, password } => {
            // 验证密码长度
            if password.len() > 10 {
                eprintln!("❌ 错误: 密码长度不能超过10位");
                process::exit(1);
            }

            println!("🔑 正在生成新的加密Solana私钥...");
            println!();

            // 生成新的密钥对
            let (private_key, public_key) = generate_new_keypair();

            // 显示公钥
            println!("✅ 成功生成新的Solana密钥对!");
            println!();
            println!("🆔 公钥地址:");
            println!("{}", public_key);
            println!();

            // 加密私钥
            match encrypt_private_key(&private_key, password) {
                Ok(encrypted_data) => {
                    // 分段处理加密数据
                    let encrypted_segments = split_private_key_into_segments(&encrypted_data, *segments);

                    if *segments > 1 {
                        println!("🔐 加密私钥 (分{}段显示):", segments);
                        for (i, segment) in encrypted_segments.iter().enumerate() {
                            println!("段{}: {}", i + 1, segment);
                        }
                        println!();
                    }

                    println!("🔐 完整加密私钥:");
                    println!("{}", encrypted_data);
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
                    println!("💡 提示: 请保存好这个完整的加密密钥，解密时需要用到");
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
    }
}
