//! 密码管理模块

use duckhub_common::prelude::*;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use tracing::{debug, instrument};

/// 密码管理器
pub struct PasswordManager {
    /// Argon2实例
    argon2: Argon2<'static>,
}

impl PasswordManager {
    /// 创建新的密码管理器
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// 哈希密码
    #[instrument(skip(self, password))]
    pub fn hash_password(&self, password: &str) -> Result<(String, String)> {
        // 生成随机盐值
        let salt = SaltString::generate(&mut OsRng);
        
        // 哈希密码
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DuckHubError::validation(format!("密码哈希失败: {}", e)))?;

        debug!("密码哈希完成");
        Ok((password_hash.to_string(), salt.to_string()))
    }

    /// 验证密码
    #[instrument(skip(self, password, stored_hash))]
    pub fn verify_password(&self, password: &str, stored_hash: &str, _salt: &str) -> Result<bool> {
        // 解析存储的哈希
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|e| DuckHubError::validation(format!("密码哈希解析失败: {}", e)))?;

        // 验证密码
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => {
                debug!("密码验证成功");
                Ok(true)
            }
            Err(_) => {
                debug!("密码验证失败");
                Ok(false)
            }
        }
    }

    /// 生成随机密码
    pub fn generate_password(&self, length: usize) -> String {
        use rand::Rng;
        
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\
                                !@#$%^&*()_+-=[]{}|;:,.<>?";
        
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 检查密码强度
    pub fn check_password_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;
        let mut feedback = Vec::new();

        // 长度检查
        if password.len() >= 8 {
            score += 1;
        } else {
            feedback.push("密码长度至少8位".to_string());
        }

        if password.len() >= 12 {
            score += 1;
        }

        // 字符类型检查
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        } else {
            feedback.push("需要包含小写字母".to_string());
        }

        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        } else {
            feedback.push("需要包含大写字母".to_string());
        }

        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        } else {
            feedback.push("需要包含数字".to_string());
        }

        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        } else {
            feedback.push("需要包含特殊字符".to_string());
        }

        // 复杂性检查
        if password.len() > 16 {
            score += 1;
        }

        // 避免常见模式
        if !self.has_common_patterns(password) {
            score += 1;
        } else {
            feedback.push("避免使用常见模式".to_string());
        }

        let strength_level = match score {
            0..=2 => StrengthLevel::Weak,
            3..=4 => StrengthLevel::Fair,
            5..=6 => StrengthLevel::Good,
            _ => StrengthLevel::Strong,
        };

        PasswordStrength {
            level: strength_level,
            score,
            max_score: 8,
            feedback,
        }
    }

    /// 检查是否包含常见模式
    fn has_common_patterns(&self, password: &str) -> bool {
        let common_patterns = [
            "123456", "password", "qwerty", "abc123", "admin",
            "letmein", "welcome", "monkey", "dragon", "master",
        ];

        let lower_password = password.to_lowercase();
        common_patterns.iter().any(|&pattern| lower_password.contains(pattern))
    }
}

impl Default for PasswordManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 密码强度
#[derive(Debug, Clone)]
pub struct PasswordStrength {
    /// 强度等级
    pub level: StrengthLevel,
    /// 得分
    pub score: u8,
    /// 最大得分
    pub max_score: u8,
    /// 改进建议
    pub feedback: Vec<String>,
}

/// 强度等级
#[derive(Debug, Clone, PartialEq)]
pub enum StrengthLevel {
    /// 弱
    Weak,
    /// 一般
    Fair,
    /// 良好
    Good,
    /// 强
    Strong,
}

impl std::fmt::Display for StrengthLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrengthLevel::Weak => write!(f, "弱"),
            StrengthLevel::Fair => write!(f, "一般"),
            StrengthLevel::Good => write!(f, "良好"),
            StrengthLevel::Strong => write!(f, "强"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let manager = PasswordManager::new();
        let password = "test_password_123!";
        
        let (hash, salt) = manager.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(!salt.is_empty());
        
        // 验证正确密码
        assert!(manager.verify_password(password, &hash, &salt).unwrap());
        
        // 验证错误密码
        assert!(!manager.verify_password("wrong_password", &hash, &salt).unwrap());
    }

    #[test]
    fn test_password_generation() {
        let manager = PasswordManager::new();
        let password = manager.generate_password(12);
        
        assert_eq!(password.len(), 12);
        assert!(password.chars().any(|c| c.is_alphabetic()));
    }

    #[test]
    fn test_password_strength() {
        let manager = PasswordManager::new();
        
        // 弱密码
        let weak = manager.check_password_strength("123");
        assert_eq!(weak.level, StrengthLevel::Weak);
        
        // 强密码
        let strong = manager.check_password_strength("MyStr0ng!P@ssw0rd2024");
        assert!(matches!(strong.level, StrengthLevel::Good | StrengthLevel::Strong));
    }

    #[test]
    fn test_common_patterns() {
        let manager = PasswordManager::new();
        
        assert!(manager.has_common_patterns("password123"));
        assert!(manager.has_common_patterns("admin2024"));
        assert!(!manager.has_common_patterns("MyUniqueP@ssw0rd!"));
    }
}
