//! 数据脱敏模块

use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, instrument};

/// 数据脱敏管理器
pub struct DataMaskingManager {
    /// 脱敏配置
    config: MaskingConfig,
    /// 脱敏规则
    rules: HashMap<MaskType, MaskingRule>,
}

/// 脱敏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfig {
    /// 是否启用脱敏
    pub enabled: bool,
    /// 默认脱敏字符
    pub default_mask_char: char,
    /// 保留前缀长度
    pub preserve_prefix_length: usize,
    /// 保留后缀长度
    pub preserve_suffix_length: usize,
    /// 自定义脱敏规则
    pub custom_rules: HashMap<String, String>,
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_mask_char: '*',
            preserve_prefix_length: 2,
            preserve_suffix_length: 2,
            custom_rules: HashMap::new(),
        }
    }
}

/// 脱敏类型
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MaskType {
    /// 邮箱
    Email,
    /// 电话号码
    Phone,
    /// 身份证号
    IdCard,
    /// 银行卡号
    BankCard,
    /// 姓名
    Name,
    /// 地址
    Address,
    /// 密码
    Password,
    /// 自定义
    Custom(String),
}

/// 脱敏规则
#[derive(Debug, Clone)]
pub struct MaskingRule {
    /// 正则表达式
    pub pattern: Option<Regex>,
    /// 脱敏函数
    pub mask_fn: fn(&str, &MaskingConfig) -> String,
    /// 描述
    pub description: String,
}

impl DataMaskingManager {
    /// 创建新的数据脱敏管理器
    pub fn new(config: MaskingConfig) -> Self {
        let mut manager = Self {
            config,
            rules: HashMap::new(),
        };

        // 初始化内置脱敏规则
        manager.initialize_rules();
        manager
    }

    /// 初始化脱敏规则
    fn initialize_rules(&mut self) {
        // 邮箱脱敏规则
        self.rules.insert(
            MaskType::Email,
            MaskingRule {
                pattern: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").ok(),
                mask_fn: Self::mask_email,
                description: "邮箱地址脱敏".to_string(),
            },
        );

        // 电话号码脱敏规则
        self.rules.insert(
            MaskType::Phone,
            MaskingRule {
                pattern: Regex::new(r"^1[3-9]\d{9}$").ok(),
                mask_fn: Self::mask_phone,
                description: "手机号码脱敏".to_string(),
            },
        );

        // 身份证号脱敏规则
        self.rules.insert(
            MaskType::IdCard,
            MaskingRule {
                pattern: Regex::new(r"^\d{17}[\dXx]$").ok(),
                mask_fn: Self::mask_id_card,
                description: "身份证号脱敏".to_string(),
            },
        );

        // 银行卡号脱敏规则
        self.rules.insert(
            MaskType::BankCard,
            MaskingRule {
                pattern: Regex::new(r"^\d{16,19}$").ok(),
                mask_fn: Self::mask_bank_card,
                description: "银行卡号脱敏".to_string(),
            },
        );

        // 姓名脱敏规则
        self.rules.insert(
            MaskType::Name,
            MaskingRule {
                pattern: None,
                mask_fn: Self::mask_name,
                description: "姓名脱敏".to_string(),
            },
        );

        // 地址脱敏规则
        self.rules.insert(
            MaskType::Address,
            MaskingRule {
                pattern: None,
                mask_fn: Self::mask_address,
                description: "地址脱敏".to_string(),
            },
        );

        // 密码脱敏规则
        self.rules.insert(
            MaskType::Password,
            MaskingRule {
                pattern: None,
                mask_fn: Self::mask_password,
                description: "密码完全脱敏".to_string(),
            },
        );
    }

    /// 数据脱敏
    #[instrument(skip(self, data))]
    pub fn mask_data(&self, data: &str, mask_type: MaskType) -> Result<String> {
        if !self.config.enabled {
            return Ok(data.to_string());
        }

        if let Some(rule) = self.rules.get(&mask_type) {
            // 如果有正则表达式，先验证格式
            if let Some(pattern) = &rule.pattern {
                if !pattern.is_match(data) {
                    debug!("数据格式不匹配脱敏规则: {:?}", mask_type);
                    return Ok(self.mask_generic(data));
                }
            }

            let masked = (rule.mask_fn)(data, &self.config);
            debug!("数据脱敏完成: {:?}", mask_type);
            Ok(masked)
        } else {
            // 使用通用脱敏
            Ok(self.mask_generic(data))
        }
    }

    /// 批量数据脱敏
    #[instrument(skip(self, data_map))]
    pub fn mask_data_batch(&self, data_map: HashMap<String, (String, MaskType)>) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        
        for (key, (data, mask_type)) in data_map {
            let masked_data = self.mask_data(&data, mask_type)?;
            result.insert(key, masked_data);
        }

        Ok(result)
    }

    /// 检测数据类型并自动脱敏
    #[instrument(skip(self, data))]
    pub fn auto_mask(&self, data: &str) -> String {
        // 尝试检测数据类型
        if let Some(mask_type) = self.detect_data_type(data) {
            self.mask_data(data, mask_type).unwrap_or_else(|_| self.mask_generic(data))
        } else {
            self.mask_generic(data)
        }
    }

    /// 检测数据类型
    fn detect_data_type(&self, data: &str) -> Option<MaskType> {
        for (mask_type, rule) in &self.rules {
            if let Some(pattern) = &rule.pattern {
                if pattern.is_match(data) {
                    return Some(mask_type.clone());
                }
            }
        }
        None
    }

    /// 通用脱敏
    fn mask_generic(&self, data: &str) -> String {
        if data.len() <= self.config.preserve_prefix_length + self.config.preserve_suffix_length {
            return self.config.default_mask_char.to_string().repeat(data.len());
        }

        let prefix = &data[..self.config.preserve_prefix_length];
        let suffix = &data[data.len() - self.config.preserve_suffix_length..];
        let middle_len = data.len() - self.config.preserve_prefix_length - self.config.preserve_suffix_length;
        let middle = self.config.default_mask_char.to_string().repeat(middle_len);

        format!("{}{}{}", prefix, middle, suffix)
    }

    /// 邮箱脱敏
    fn mask_email(data: &str, config: &MaskingConfig) -> String {
        if let Some(at_pos) = data.find('@') {
            let (username, domain) = data.split_at(at_pos);
            let masked_username = if username.len() <= 2 {
                config.default_mask_char.to_string().repeat(username.len())
            } else {
                let first = &username[..1];
                let last = &username[username.len() - 1..];
                let middle = config.default_mask_char.to_string().repeat(username.len() - 2);
                format!("{}{}{}", first, middle, last)
            };
            format!("{}{}", masked_username, domain)
        } else {
            config.default_mask_char.to_string().repeat(data.len())
        }
    }

    /// 电话号码脱敏
    fn mask_phone(data: &str, config: &MaskingConfig) -> String {
        if data.len() == 11 {
            format!("{}****{}", &data[..3], &data[7..])
        } else {
            config.default_mask_char.to_string().repeat(data.len())
        }
    }

    /// 身份证号脱敏
    fn mask_id_card(data: &str, config: &MaskingConfig) -> String {
        if data.len() == 18 {
            format!("{}**********{}", &data[..4], &data[14..])
        } else {
            config.default_mask_char.to_string().repeat(data.len())
        }
    }

    /// 银行卡号脱敏
    fn mask_bank_card(data: &str, config: &MaskingConfig) -> String {
        if data.len() >= 8 {
            let prefix = &data[..4];
            let suffix = &data[data.len() - 4..];
            let middle = config.default_mask_char.to_string().repeat(data.len() - 8);
            format!("{}{}{}", prefix, middle, suffix)
        } else {
            config.default_mask_char.to_string().repeat(data.len())
        }
    }

    /// 姓名脱敏
    fn mask_name(data: &str, config: &MaskingConfig) -> String {
        let chars: Vec<char> = data.chars().collect();
        if chars.len() <= 1 {
            return config.default_mask_char.to_string();
        } else if chars.len() == 2 {
            format!("{}{}", chars[0], config.default_mask_char)
        } else {
            let first = chars[0];
            let last = chars[chars.len() - 1];
            let middle = config.default_mask_char.to_string().repeat(chars.len() - 2);
            format!("{}{}{}", first, middle, last)
        }
    }

    /// 地址脱敏
    fn mask_address(data: &str, config: &MaskingConfig) -> String {
        if data.len() <= 6 {
            config.default_mask_char.to_string().repeat(data.len())
        } else {
            let prefix = &data[..3];
            let suffix = &data[data.len() - 3..];
            let middle = config.default_mask_char.to_string().repeat(data.len() - 6);
            format!("{}{}{}", prefix, middle, suffix)
        }
    }

    /// 密码完全脱敏
    fn mask_password(data: &str, config: &MaskingConfig) -> String {
        config.default_mask_char.to_string().repeat(data.len().min(8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let email = "test@example.com";
        let masked = manager.mask_data(email, MaskType::Email).unwrap();
        assert_eq!(masked, "t**t@example.com");
    }

    #[test]
    fn test_phone_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let phone = "13812345678";
        let masked = manager.mask_data(phone, MaskType::Phone).unwrap();
        assert_eq!(masked, "138****5678");
    }

    #[test]
    fn test_id_card_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let id_card = "123456789012345678";
        let masked = manager.mask_data(id_card, MaskType::IdCard).unwrap();
        assert_eq!(masked, "1234**********5678");
    }

    #[test]
    fn test_bank_card_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let bank_card = "1234567890123456";
        let masked = manager.mask_data(bank_card, MaskType::BankCard).unwrap();
        assert_eq!(masked, "1234********3456");
    }

    #[test]
    fn test_name_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let name = "张三";
        let masked = manager.mask_data(name, MaskType::Name).unwrap();
        assert_eq!(masked, "张*");

        let long_name = "欧阳修";
        let masked_long = manager.mask_data(long_name, MaskType::Name).unwrap();
        assert_eq!(masked_long, "欧*修");
    }

    #[test]
    fn test_auto_detection() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let email = "test@example.com";
        let masked = manager.auto_mask(email);
        assert_eq!(masked, "t**t@example.com");

        let phone = "13812345678";
        let masked_phone = manager.auto_mask(phone);
        assert_eq!(masked_phone, "138****5678");
    }

    #[test]
    fn test_batch_masking() {
        let config = MaskingConfig::default();
        let manager = DataMaskingManager::new(config);

        let mut data_map = HashMap::new();
        data_map.insert("email".to_string(), ("test@example.com".to_string(), MaskType::Email));
        data_map.insert("phone".to_string(), ("13812345678".to_string(), MaskType::Phone));

        let result = manager.mask_data_batch(data_map).unwrap();
        assert_eq!(result.get("email").unwrap(), "t**t@example.com");
        assert_eq!(result.get("phone").unwrap(), "138****5678");
    }
}
