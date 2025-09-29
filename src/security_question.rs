use colored::*;
use std::io::{self, Write};

/// 预定义的安全问题列表
pub const SECURITY_QUESTIONS: &[&str] = &[
    "您母亲的姓名是？",
    "您出生的城市是？",
    "您小学的名称是？",
    "您最喜欢的电影是？",
    "您的第一个宠物叫什么名字？",
    "您父亲的生日是？(格式: YYYYMMDD)",
    "您配偶的名字是？",
    "您最好朋友的名字是？",
];

/// 安全问题和答案
#[derive(Debug, Clone)]
pub struct SecurityQuestion {
    pub question_index: usize,
    pub question: String,
    pub answer_hash: String, // 存储答案的哈希值
}

impl SecurityQuestion {
    /// 交互式选择安全问题
    pub fn setup_interactive() -> Result<(usize, String), String> {
        println!();
        println!("{}", "请选择一个安全问题:".bright_cyan().bold());
        println!();

        for (i, question) in SECURITY_QUESTIONS.iter().enumerate() {
            println!("  {}. {}", i + 1, question);
        }
        println!();

        // 选择问题
        let question_index = loop {
            print!("{} ", "请输入问题编号 (1-8):".bright_yellow());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| format!("读取输入失败: {}", e))?;

            match input.trim().parse::<usize>() {
                Ok(num) if num >= 1 && num <= SECURITY_QUESTIONS.len() => {
                    break num - 1;
                }
                _ => {
                    println!("{}", "❌ 无效的编号，请重新输入".red());
                }
            }
        };

        let question = SECURITY_QUESTIONS[question_index].to_string();
        println!();
        println!("您选择的问题: {}", question.bright_green());

        // 输入答案（明文可见，无需确认）
        let answer = loop {
            print!("{} ", "请输入答案:".bright_yellow());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| format!("读取输入失败: {}", e))?;

            let ans = input.trim();
            if ans.is_empty() {
                println!("{}", "❌ 答案不能为空".red());
                continue;
            }

            break ans.to_lowercase(); // 转小写以避免大小写问题
        };

        println!("{}", "✅ 安全问题已设置".bright_green());

        Ok((question_index, answer))
    }

    /// 验证答案
    pub fn verify_interactive(question_index: usize) -> Result<String, String> {
        if question_index >= SECURITY_QUESTIONS.len() {
            return Err("无效的问题索引".to_string());
        }

        let question = SECURITY_QUESTIONS[question_index];
        println!();
        println!("{}", "安全问题验证".bright_cyan().bold());
        println!("问题: {}", question.bright_yellow());
        println!();

        for attempt in 1..=3 {
            print!("{} ", "请输入答案:".bright_yellow());
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .map_err(|e| format!("读取输入失败: {}", e))?;

            let answer = input.trim().to_lowercase();
            if !answer.is_empty() {
                return Ok(answer);
            }

            if attempt < 3 {
                println!("{}", "❌ 答案不能为空，请重试".red());
            }
        }

        Err("安全问题验证失败".to_string())
    }

    /// 生成答案的哈希值（用于验证）
    pub fn hash_answer(answer: &str) -> String {
        use ring::digest;
        let normalized = answer.trim().to_lowercase();
        let hash = digest::digest(&digest::SHA256, normalized.as_bytes());
        hex::encode(hash.as_ref())
    }

    /// 验证答案是否正确
    pub fn verify_answer(answer: &str, expected_hash: &str) -> bool {
        Self::hash_answer(answer) == expected_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_answer() {
        let answer = "Beijing";
        let hash = SecurityQuestion::hash_answer(answer);

        // 相同答案产生相同哈希
        assert_eq!(hash, SecurityQuestion::hash_answer("beijing"));
        assert_eq!(hash, SecurityQuestion::hash_answer(" Beijing "));

        // 不同答案产生不同哈希
        assert_ne!(hash, SecurityQuestion::hash_answer("Shanghai"));
    }

    #[test]
    fn test_verify_answer() {
        let answer = "Beijing";
        let hash = SecurityQuestion::hash_answer(answer);

        assert!(SecurityQuestion::verify_answer("Beijing", &hash));
        assert!(SecurityQuestion::verify_answer("beijing", &hash));
        assert!(SecurityQuestion::verify_answer(" BEIJING ", &hash));
        assert!(!SecurityQuestion::verify_answer("Shanghai", &hash));
    }
}