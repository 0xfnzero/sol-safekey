//! Interactive Menu Module
//!
//! Provides a simple interactive interface - no need to memorize commands
//! 提供简单的交互式界面 - 无需记住命令

use std::io::{self, Write};
use colored::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use crate::KeyManager;

/// Language selection
#[derive(Clone, Copy, PartialEq)]
enum Language {
    English,
    Chinese,
}

/// Text strings for bilingual UI
struct Texts {
    // Main menu
    title: &'static str,
    core_functions: &'static str,
    create_plain: &'static str,
    create_encrypted: &'static str,
    decrypt: &'static str,
    exit: &'static str,
    select_option: &'static str,
    goodbye: &'static str,
    invalid_option: &'static str,
    continue_use: &'static str,

    // Plain key creation
    create_plain_title: &'static str,
    keypair_generated: &'static str,
    public_key: &'static str,
    private_key: &'static str,
    output_method: &'static str,
    display_only: &'static str,
    save_to_file: &'static str,
    select: &'static str,
    file_path: &'static str,
    file_saved: &'static str,
    security_warning: &'static str,
    plaintext_warning: &'static str,
    save_securely: &'static str,
    dont_share: &'static str,
    recommend_encrypted: &'static str,

    // Encrypted key creation
    create_encrypted_title: &'static str,
    choose_method: &'static str,
    generate_new: &'static str,
    import_existing: &'static str,
    generating: &'static str,
    enter_private_key: &'static str,
    private_key_empty: &'static str,
    keypair_ready: &'static str,
    keystore_recommended: &'static str,
    show_encrypted_string: &'static str,
    keystore_created: &'static str,
    private_key_encrypted: &'static str,
    important_note: &'static str,
    keep_safe: &'static str,
    lost_password_warning: &'static str,
    backup_recommended: &'static str,
    encrypted_private_key: &'static str,
    keep_safe_both: &'static str,

    // Key decryption
    decrypt_title: &'static str,
    input_method: &'static str,
    from_keystore: &'static str,
    from_encrypted_string: &'static str,
    encrypted_key: &'static str,
    enter_password: &'static str,
    decrypt_success: &'static str,
    file_not_exist: &'static str,
    dont_share_warning: &'static str,
    delete_plaintext: &'static str,
    use_encryption: &'static str,

    // Password
    set_password: &'static str,
    new_password: &'static str,
    confirm_password: &'static str,
    password_empty: &'static str,
    password_min_length: &'static str,
    password_mismatch: &'static str,
    password_set: &'static str,

    // Errors
    invalid_choice: &'static str,
    write_failed: &'static str,
}

impl Texts {
    fn chinese() -> Self {
        Self {
            title: "  Sol-SafeKey - Solana 密钥管理工具",
            core_functions: "核心功能 (只需3个操作):",
            create_plain: "  {}  创建明文私钥",
            create_encrypted: "  {}  创建加密私钥",
            decrypt: "  {}  解密私钥",
            exit: "  {}  退出",
            select_option: "请输入选项 [0-3]: ",
            goodbye: "👋 再见！",
            invalid_option: "❌ 无效选项，请重新选择",
            continue_use: "是否继续使用? [Y/n]: ",

            create_plain_title: "  创建明文私钥",
            keypair_generated: "✅ 密钥对生成成功！",
            public_key: "公钥地址:",
            private_key: "私钥:",
            output_method: "输出方式:",
            display_only: "  1. 仅显示 (当前已显示)",
            save_to_file: "  2. 保存到文件",
            select: "请选择 [1/2]: ",
            file_path: "文件路径 (默认: keypair.json): ",
            file_saved: "✅ 已保存到文件",
            security_warning: "⚠️  安全警告:",
            plaintext_warning: "  • 明文私钥非常不安全",
            save_securely: "  • 请立即保存到安全位置",
            dont_share: "  • 不要分享给任何人",
            recommend_encrypted: "  • 建议使用 '创建加密私钥' 功能",

            create_encrypted_title: "  创建加密私钥",
            choose_method: "选择方式:",
            generate_new: "  1. 生成新的密钥对并加密",
            import_existing: "  2. 导入现有私钥并加密",
            generating: "🎲 生成新的密钥对...",
            enter_private_key: "请输入私钥 (base58 格式): ",
            private_key_empty: "私钥不能为空",
            keypair_ready: "✅ 密钥对准备完成",
            keystore_recommended: "  1. 保存为 Keystore 文件 (推荐)",
            show_encrypted_string: "  2. 显示加密字符串",
            keystore_created: "  ✅ Keystore 创建成功！",
            private_key_encrypted: "🔒 私钥已加密保存",
            important_note: "⚠️  重要提示:",
            keep_safe: "  • 请妥善保管 Keystore 文件和密码",
            lost_password_warning: "  • 丢失密码将无法恢复钱包",
            backup_recommended: "  • 建议备份到安全位置",
            encrypted_private_key: "加密后的私钥:",
            keep_safe_both: "⚠️  提示: 请妥善保管加密私钥和密码",

            decrypt_title: "  解密私钥",
            input_method: "输入方式:",
            from_keystore: "  1. 从 Keystore 文件读取",
            from_encrypted_string: "  2. 输入加密字符串",
            encrypted_key: "加密的私钥: ",
            enter_password: "请输入密码: ",
            decrypt_success: "  ✅ 解密成功！",
            file_not_exist: "文件不存在: {}",
            dont_share_warning: "  • 请勿分享私钥给任何人",
            delete_plaintext: "  • 使用完毕后请立即删除明文私钥文件",
            use_encryption: "  • 建议使用加密方式保存",

            set_password: "设置加密密码 (至少 10 个字符):",
            new_password: "新密码: ",
            confirm_password: "确认密码: ",
            password_empty: "密码不能为空",
            password_min_length: "密码长度至少 10 个字符",
            password_mismatch: "两次密码不一致",
            password_set: "✅ 密码设置成功",

            invalid_choice: "无效选项",
            write_failed: "写入文件失败: {}",
        }
    }

    fn english() -> Self {
        Self {
            title: "  Sol-SafeKey - Solana Key Management Tool",
            core_functions: "Core Functions (3 operations):",
            create_plain: "  {}  Create Plain Private Key",
            create_encrypted: "  {}  Create Encrypted Private Key",
            decrypt: "  {}  Decrypt Private Key",
            exit: "  {}  Exit",
            select_option: "Select option [0-3]: ",
            goodbye: "👋 Goodbye!",
            invalid_option: "❌ Invalid option, please try again",
            continue_use: "Continue? [Y/n]: ",

            create_plain_title: "  Create Plain Private Key",
            keypair_generated: "✅ Keypair generated successfully!",
            public_key: "Public Key:",
            private_key: "Private Key:",
            output_method: "Output Method:",
            display_only: "  1. Display Only (already shown)",
            save_to_file: "  2. Save to File",
            select: "Select [1/2]: ",
            file_path: "File path (default: keypair.json): ",
            file_saved: "✅ Saved to file",
            security_warning: "⚠️  Security Warning:",
            plaintext_warning: "  • Plaintext private key is very insecure",
            save_securely: "  • Save to a secure location immediately",
            dont_share: "  • Never share with anyone",
            recommend_encrypted: "  • Consider using 'Create Encrypted Private Key'",

            create_encrypted_title: "  Create Encrypted Private Key",
            choose_method: "Choose Method:",
            generate_new: "  1. Generate new keypair and encrypt",
            import_existing: "  2. Import existing private key and encrypt",
            generating: "🎲 Generating new keypair...",
            enter_private_key: "Enter private key (base58 format): ",
            private_key_empty: "Private key cannot be empty",
            keypair_ready: "✅ Keypair ready",
            keystore_recommended: "  1. Save as Keystore file (Recommended)",
            show_encrypted_string: "  2. Show encrypted string",
            keystore_created: "  ✅ Keystore created successfully!",
            private_key_encrypted: "🔒 Private key encrypted and saved",
            important_note: "⚠️  Important:",
            keep_safe: "  • Keep Keystore file and password safe",
            lost_password_warning: "  • Lost password = lost wallet",
            backup_recommended: "  • Backup to a secure location",
            encrypted_private_key: "Encrypted Private Key:",
            keep_safe_both: "⚠️  Note: Keep encrypted key and password safe",

            decrypt_title: "  Decrypt Private Key",
            input_method: "Input Method:",
            from_keystore: "  1. From Keystore file",
            from_encrypted_string: "  2. Enter encrypted string",
            encrypted_key: "Encrypted key: ",
            enter_password: "Enter password: ",
            decrypt_success: "  ✅ Decryption successful!",
            file_not_exist: "File not found: {}",
            dont_share_warning: "  • Never share private key with anyone",
            delete_plaintext: "  • Delete plaintext key file after use",
            use_encryption: "  • Consider using encryption for storage",

            set_password: "Set encryption password (minimum 10 characters):",
            new_password: "New password: ",
            confirm_password: "Confirm password: ",
            password_empty: "Password cannot be empty",
            password_min_length: "Password must be at least 10 characters",
            password_mismatch: "Passwords do not match",
            password_set: "✅ Password set successfully",

            invalid_choice: "Invalid choice",
            write_failed: "Write failed: {}",
        }
    }
}

/// 选择语言
fn select_language() -> Result<Language, String> {
    println!("\n{}", "=".repeat(50).cyan());
    println!("{}", "  Language / 语言选择".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    println!();
    println!("  {}  English", "1.".green().bold());
    println!("  {}  中文", "2.".green().bold());
    println!();
    print!("Select / 选择 [1/2]: ");
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).map_err(|e| e.to_string())?;
    let choice = choice.trim();

    match choice {
        "1" => Ok(Language::English),
        "2" => Ok(Language::Chinese),
        _ => {
            println!("\n{}", "❌ Invalid option / 无效选项".red());
            select_language()
        }
    }
}

/// 显示主菜单并处理用户选择
pub fn show_main_menu() -> Result<(), String> {
    // 首先选择语言
    let lang = select_language()?;
    let texts = match lang {
        Language::Chinese => Texts::chinese(),
        Language::English => Texts::english(),
    };

    loop {
        println!("\n{}", "=".repeat(50).cyan());
        println!("{}", texts.title.cyan().bold());
        println!("{}", "=".repeat(50).cyan());
        println!();
        println!("{}", texts.core_functions);
        println!();
        println!("  {}  {}", "1.".green().bold(), &texts.create_plain[6..]);
        println!("  {}  {}", "2.".green().bold(), &texts.create_encrypted[6..]);
        println!("  {}  {}", "3.".green().bold(), &texts.decrypt[6..]);
        println!("  {}  {}", "0.".red().bold(), &texts.exit[6..]);
        println!();
        print!("{}", texts.select_option);
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).map_err(|e| e.to_string())?;
        let choice = choice.trim();

        match choice {
            "1" => create_plain_key_interactive(&texts)?,
            "2" => create_encrypted_key_interactive(&texts)?,
            "3" => decrypt_key_interactive(&texts)?,
            "0" => {
                println!("\n{}", texts.goodbye.cyan());
                break;
            }
            _ => {
                println!("\n{}", texts.invalid_option.red());
                continue;
            }
        }

        // 询问是否继续
        println!();
        print!("{}", texts.continue_use);
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut continue_choice = String::new();
        io::stdin().read_line(&mut continue_choice).map_err(|e| e.to_string())?;
        let continue_choice = continue_choice.trim().to_lowercase();

        if continue_choice == "n" || continue_choice == "no" {
            println!("\n{}", texts.goodbye.cyan());
            break;
        }
    }

    Ok(())
}

/// 功能1: 创建明文私钥
fn create_plain_key_interactive(texts: &Texts) -> Result<(), String> {
    println!("\n{}", "=".repeat(50).yellow());
    println!("{}", texts.create_plain_title.yellow().bold());
    println!("{}", "=".repeat(50).yellow());
    println!();

    // 生成密钥对
    let keypair = KeyManager::generate_keypair();
    let pubkey = keypair.pubkey();
    let private_key = keypair.to_base58_string();

    println!("{}", texts.keypair_generated.green().bold());
    println!();
    println!("{} {}", texts.public_key.cyan(), pubkey.to_string().white().bold());
    println!("{} {}", texts.private_key.red().bold(), private_key);
    println!();

    // 询问输出方式
    println!("{}",texts.output_method);
    println!("{}", texts.display_only);
    println!("{}", texts.save_to_file);
    println!();
    print!("{}", texts.select);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut output_choice = String::new();
    io::stdin().read_line(&mut output_choice).map_err(|e| e.to_string())?;
    let output_choice = output_choice.trim();

    if output_choice == "2" {
        print!("{}", texts.file_path);
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut file_path = String::new();
        io::stdin().read_line(&mut file_path).map_err(|e| e.to_string())?;
        let file_path = file_path.trim();
        let file_path = if file_path.is_empty() {
            "keypair.json"
        } else {
            file_path
        };

        // 保存为 Solana keypair JSON 格式 (数组格式)
        let bytes = keypair.to_bytes();
        let json = serde_json::to_string(&bytes.to_vec())
            .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))?;

        std::fs::write(file_path, json)
            .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))?;

        println!();
        println!("{}", texts.file_saved.green());
        println!("{} {}", texts.file_path.trim_end_matches(':'), file_path);
    }

    println!();
    println!("{}", texts.security_warning.yellow().bold());
    println!("{}", texts.plaintext_warning);
    println!("{}", texts.save_securely);
    println!("{}", texts.dont_share);
    println!("{}", texts.recommend_encrypted);

    Ok(())
}

/// 功能2: 创建加密私钥
fn create_encrypted_key_interactive(texts: &Texts) -> Result<(), String> {
    println!("\n{}", "=".repeat(50).yellow());
    println!("{}", texts.create_encrypted_title.yellow().bold());
    println!("{}", "=".repeat(50).yellow());
    println!();

    // 询问是生成新的还是导入现有私钥
    println!("{}", texts.choose_method);
    println!("{}", texts.generate_new);
    println!("{}", texts.import_existing);
    println!();
    print!("{}", texts.select);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut source_choice = String::new();
    io::stdin().read_line(&mut source_choice).map_err(|e| e.to_string())?;
    let source_choice = source_choice.trim();

    let keypair = match source_choice {
        "1" => {
            // 生成新密钥对
            println!();
            println!("{}", texts.generating.cyan());
            KeyManager::generate_keypair()
        }
        "2" => {
            // 导入现有私钥
            println!();
            print!("{}", texts.enter_private_key);
            io::stdout().flush().map_err(|e| e.to_string())?;

            let mut private_key = String::new();
            io::stdin().read_line(&mut private_key).map_err(|e| e.to_string())?;
            let private_key = private_key.trim();

            if private_key.is_empty() {
                return Err(texts.private_key_empty.to_string());
            }

            Keypair::from_base58_string(private_key)
        }
        _ => {
            return Err(texts.invalid_choice.to_string());
        }
    };

    let pubkey = keypair.pubkey();

    println!();
    println!("{}", texts.keypair_ready.green());
    println!("{} {}", texts.public_key.cyan(), pubkey);
    println!();

    // 获取密码
    let password = read_password_confirmed(texts)?;

    // 询问输出方式
    println!();
    println!("{}", texts.output_method);
    println!("{}", texts.keystore_recommended);
    println!("{}", texts.show_encrypted_string);
    println!();
    print!("{}", texts.select);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut output_choice = String::new();
    io::stdin().read_line(&mut output_choice).map_err(|e| e.to_string())?;
    let output_choice = output_choice.trim();

    match output_choice {
        "1" => {
            // 保存为文件
            print!("{}", texts.file_path.replace("keypair", "wallet"));
            io::stdout().flush().map_err(|e| e.to_string())?;

            let mut file_path = String::new();
            io::stdin().read_line(&mut file_path).map_err(|e| e.to_string())?;
            let file_path = file_path.trim();
            let file_path = if file_path.is_empty() {
                "wallet.json"
            } else {
                file_path
            };

            let keystore_json = KeyManager::keypair_to_encrypted_json(&keypair, &password)?;
            std::fs::write(file_path, keystore_json)
                .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))?;

            println!();
            println!("{}", "=".repeat(50).green());
            println!("{}", texts.keystore_created.green().bold());
            println!("{}", "=".repeat(50).green());
            println!();
            println!("{} {}", texts.file_path.trim_end_matches(':'), file_path);
            println!("{} {}", texts.public_key.cyan(), pubkey);
            println!("{}", texts.private_key_encrypted.green());
            println!();
            println!("{}", texts.important_note.yellow().bold());
            println!("{}", texts.keep_safe);
            println!("{}", texts.lost_password_warning);
            println!("{}", texts.backup_recommended);
        }
        "2" => {
            // 显示加密字符串
            let private_key = keypair.to_base58_string();
            let encrypted = KeyManager::encrypt_with_password(&private_key, &password)?;

            println!();
            println!("{}", texts.keypair_ready.green().bold());
            println!();
            println!("{} {}", texts.public_key.cyan(), pubkey);
            println!("{}", texts.encrypted_private_key.cyan());
            println!("{}", encrypted);
            println!();
            println!("{}", texts.keep_safe_both.yellow());
        }
        _ => {
            return Err(texts.invalid_choice.to_string());
        }
    }

    Ok(())
}

/// 功能3: 解密私钥
fn decrypt_key_interactive(texts: &Texts) -> Result<(), String> {
    println!("\n{}", "=".repeat(50).yellow());
    println!("{}", texts.decrypt_title.yellow().bold());
    println!("{}", "=".repeat(50).yellow());
    println!();

    // 选择输入方式
    println!("{}", texts.input_method);
    println!("{}", texts.from_keystore);
    println!("{}", texts.from_encrypted_string);
    println!();
    print!("{}", texts.select);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input_choice = String::new();
    io::stdin().read_line(&mut input_choice).map_err(|e| e.to_string())?;
    let input_choice = input_choice.trim();

    let (private_key, pubkey) = match input_choice {
        "1" => {
            // 从文件读取
            print!("{}", texts.file_path.trim_end_matches("(默认: keypair.json): ").trim_end_matches("(default: keypair.json): "));
            io::stdout().flush().map_err(|e| e.to_string())?;

            let mut file_path = String::new();
            io::stdin().read_line(&mut file_path).map_err(|e| e.to_string())?;
            let file_path = file_path.trim();

            if !std::path::Path::new(file_path).exists() {
                return Err(format!("{}", texts.file_not_exist.replace("{}", file_path)));
            }

            println!();
            let password = prompt_password(texts.enter_password, texts)?;

            let keystore_json = std::fs::read_to_string(file_path)
                .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))?;

            let keypair = KeyManager::keypair_from_encrypted_json(&keystore_json, &password)?;
            let pubkey = keypair.pubkey();
            let private_key = keypair.to_base58_string();

            (private_key, pubkey)
        }
        "2" => {
            // 输入加密字符串
            print!("{}", texts.encrypted_key);
            io::stdout().flush().map_err(|e| e.to_string())?;

            let mut encrypted = String::new();
            io::stdin().read_line(&mut encrypted).map_err(|e| e.to_string())?;
            let encrypted = encrypted.trim();

            println!();
            let password = prompt_password(texts.enter_password, texts)?;

            let private_key = KeyManager::decrypt_with_password(encrypted, &password)?;
            let keypair = Keypair::from_base58_string(&private_key);
            let pubkey = keypair.pubkey();

            (private_key, pubkey)
        }
        _ => {
            return Err(texts.invalid_choice.to_string());
        }
    };

    println!();
    println!("{}", "=".repeat(50).green());
    println!("{}", texts.decrypt_success.green().bold());
    println!("{}", "=".repeat(50).green());
    println!();
    println!("{} {}", texts.public_key.cyan(), pubkey);
    println!("{} {}", texts.private_key.red().bold(), private_key);
    println!();

    // 询问输出方式
    println!("{}", texts.output_method);
    println!("{}", texts.display_only);
    println!("{}", texts.save_to_file);
    println!();
    print!("{}", texts.select);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut output_choice = String::new();
    io::stdin().read_line(&mut output_choice).map_err(|e| e.to_string())?;
    let output_choice = output_choice.trim();

    if output_choice == "2" {
        let default_filename = if texts.file_path.contains("默认") {
            "decrypted_key.txt"
        } else {
            "decrypted_key.txt"
        };

        print!("{}", texts.file_path.replace("keypair.json", default_filename));
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut file_path = String::new();
        io::stdin().read_line(&mut file_path).map_err(|e| e.to_string())?;
        let file_path = file_path.trim();
        let file_path = if file_path.is_empty() {
            default_filename
        } else {
            file_path
        };

        let content = format!("{} {}\n{} {}\n", texts.public_key, pubkey, texts.private_key.trim_end_matches(':'), private_key);
        std::fs::write(file_path, content)
            .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))?;

        println!();
        println!("{}", texts.file_saved.green());
        println!("{} {}", texts.file_path.trim_end_matches(':'), file_path);
    }

    println!();
    println!("{}", texts.security_warning.yellow().bold());
    println!("{}", texts.dont_share_warning);
    println!("{}", texts.delete_plaintext);
    println!("{}", texts.use_encryption);

    Ok(())
}

/// 读取密码（隐藏输入）
/// Prompt and read password securely
fn prompt_password(prompt: &str, texts: &Texts) -> Result<String, String> {
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| e.to_string())?;
    rpassword::read_password()
        .map_err(|e| format!("{}", texts.write_failed.replace("{}", &e.to_string())))
}

/// Read password with confirmation and validation
fn read_password_confirmed(texts: &Texts) -> Result<String, String> {
    println!("{}", texts.set_password);

    let password = prompt_password(texts.new_password, texts)?;

    if password.is_empty() {
        return Err(texts.password_empty.to_string());
    }

    if password.len() < 10 {
        return Err(texts.password_min_length.to_string());
    }

    let password_confirm = prompt_password(texts.confirm_password, texts)?;

    if password != password_confirm {
        return Err(texts.password_mismatch.to_string());
    }

    println!("{}", texts.password_set.green());
    Ok(password)
}
