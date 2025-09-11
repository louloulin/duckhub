//! 数据加密服务
//! 提供AES-256加密、密码哈希、数字签名等安全功能

use crate::prelude::*;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, error, instrument};

/// 加密配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// 主密钥 (base64编码)
    pub master_key: String,
    /// 密钥轮换间隔 (小时)
    pub key_rotation_hours: u64,
    /// 是否启用字段级加密
    pub enable_field_encryption: bool,
    /// 敏感字段列表
    pub sensitive_fields: Vec<String>,
    /// Argon2配置
    pub argon2_config: Argon2Config,
}

/// Argon2密码哈希配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Config {
    /// 内存成本 (KB)
    pub memory_cost: u32,
    /// 时间成本 (迭代次数)
    pub time_cost: u32,
    /// 并行度
    pub parallelism: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            master_key: general_purpose::STANDARD.encode(&Aes256Gcm::generate_key(OsRng)),
            key_rotation_hours: 24 * 7, // 一周
            enable_field_encryption: true,
            sensitive_fields: vec![
                "password".to_string(),
                "email".to_string(),
                "phone".to_string(),
                "ssn".to_string(),
                "credit_card".to_string(),
                "bank_account".to_string(),
            ],
            argon2_config: Argon2Config {
                memory_cost: 65536, // 64MB
                time_cost: 3,
                parallelism: 4,
            },
        }
    }
}

/// 加密结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// 加密后的数据 (base64编码)
    pub ciphertext: String,
    /// 随机数 (base64编码)
    pub nonce: String,
    /// 密钥版本
    pub key_version: u32,
    /// 加密算法
    pub algorithm: String,
}

/// 密钥信息
#[derive(Debug, Clone)]
struct KeyInfo {
    /// 密钥
    key: Key<Aes256Gcm>,
    /// 版本
    version: u32,
    /// 创建时间
    created_at: chrono::DateTime<chrono::Utc>,
}

/// 数据加密服务
pub struct EncryptionService {
    /// 配置
    config: EncryptionConfig,
    /// 当前密钥
    current_key: Arc<RwLock<KeyInfo>>,
    /// 历史密钥 (用于解密旧数据)
    key_history: Arc<RwLock<HashMap<u32, KeyInfo>>>,
    /// Argon2实例
    argon2: Argon2<'static>,
}

impl EncryptionService {
    /// 创建新的加密服务
    pub async fn new(config: EncryptionConfig) -> Result<Self> {
        // 解码主密钥
        let master_key_bytes = general_purpose::STANDARD
            .decode(&config.master_key)
            .map_err(|e| DuckHubError::config(format!("主密钥解码失败: {}", e)))?;

        if master_key_bytes.len() != 32 {
            return Err(DuckHubError::config("主密钥长度必须为32字节"));
        }

        let key = Key::<Aes256Gcm>::from_slice(&master_key_bytes);
        let current_key = Arc::new(RwLock::new(KeyInfo {
            key: *key,
            version: 1,
            created_at: chrono::Utc::now(),
        }));

        // 配置Argon2
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                config.argon2_config.memory_cost,
                config.argon2_config.time_cost,
                config.argon2_config.parallelism,
                None,
            ).map_err(|e| DuckHubError::config(format!("Argon2参数配置失败: {}", e)))?,
        );

        Ok(Self {
            config,
            current_key,
            key_history: Arc::new(RwLock::new(HashMap::new())),
            argon2,
        })
    }

    /// 加密数据
    #[instrument(skip(self, plaintext))]
    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let key_info = self.current_key.read().await;
        let cipher = Aes256Gcm::new(&key_info.key);
        
        // 生成随机nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // 加密数据
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| DuckHubError::internal(format!("加密失败: {}", e)))?;

        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce),
            key_version: key_info.version,
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// 解密数据
    #[instrument(skip(self, encrypted_data))]
    pub async fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>> {
        // 获取对应版本的密钥
        let key = if encrypted_data.key_version == self.current_key.read().await.version {
            self.current_key.read().await.key
        } else {
            let key_history = self.key_history.read().await;
            key_history
                .get(&encrypted_data.key_version)
                .ok_or_else(|| DuckHubError::internal(format!("密钥版本 {} 不存在", encrypted_data.key_version)))?
                .key
        };

        let cipher = Aes256Gcm::new(&key);
        
        // 解码数据
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| DuckHubError::internal(format!("密文解码失败: {}", e)))?;
        
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|e| DuckHubError::internal(format!("nonce解码失败: {}", e)))?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| DuckHubError::internal(format!("解密失败: {}", e)))?;

        Ok(plaintext)
    }

    /// 加密字符串
    pub async fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedData> {
        self.encrypt(plaintext.as_bytes()).await
    }

    /// 解密字符串
    pub async fn decrypt_string(&self, encrypted_data: &EncryptedData) -> Result<String> {
        let plaintext = self.decrypt(encrypted_data).await?;
        String::from_utf8(plaintext)
            .map_err(|e| DuckHubError::internal(format!("UTF-8解码失败: {}", e)))
    }

    /// 哈希密码
    #[instrument(skip(self, password))]
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut ArgonOsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DuckHubError::internal(format!("密码哈希失败: {}", e)))?;
        
        Ok(password_hash.to_string())
    }

    /// 验证密码
    #[instrument(skip(self, password, hash))]
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DuckHubError::internal(format!("密码哈希解析失败: {}", e)))?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(DuckHubError::internal(format!("密码验证失败: {}", e))),
        }
    }

    /// 加密JSON对象中的敏感字段
    #[instrument(skip(self, data))]
    pub async fn encrypt_sensitive_fields(&self, data: &mut serde_json::Value) -> Result<()> {
        if !self.config.enable_field_encryption {
            return Ok(());
        }

        match data {
            serde_json::Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    if self.config.sensitive_fields.contains(key) {
                        if let serde_json::Value::String(s) = value {
                            let encrypted = self.encrypt_string(s).await?;
                            *value = serde_json::to_value(encrypted)?;
                        }
                    } else if value.is_object() || value.is_array() {
                        // 递归处理嵌套对象和数组
                        self.encrypt_sensitive_fields(value).await?;
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.encrypt_sensitive_fields(item).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 解密JSON对象中的敏感字段
    #[instrument(skip(self, data))]
    pub async fn decrypt_sensitive_fields(&self, data: &mut serde_json::Value) -> Result<()> {
        if !self.config.enable_field_encryption {
            return Ok(());
        }

        match data {
            serde_json::Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    if self.config.sensitive_fields.contains(key) {
                        if let Ok(encrypted_data) = serde_json::from_value::<EncryptedData>(value.clone()) {
                            let decrypted = self.decrypt_string(&encrypted_data).await?;
                            *value = serde_json::Value::String(decrypted);
                        }
                    } else if value.is_object() || value.is_array() {
                        // 递归处理嵌套对象和数组
                        self.decrypt_sensitive_fields(value).await?;
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.decrypt_sensitive_fields(item).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 轮换密钥
    #[instrument(skip(self))]
    pub async fn rotate_key(&self) -> Result<()> {
        let mut current_key = self.current_key.write().await;
        let mut key_history = self.key_history.write().await;

        // 将当前密钥移到历史记录
        key_history.insert(current_key.version, current_key.clone());

        // 生成新密钥
        let new_key = Aes256Gcm::generate_key(OsRng);
        current_key.key = new_key;
        current_key.version += 1;
        current_key.created_at = chrono::Utc::now();

        debug!("密钥轮换完成，新版本: {}", current_key.version);
        Ok(())
    }

    /// 检查是否需要轮换密钥
    pub async fn should_rotate_key(&self) -> bool {
        let current_key = self.current_key.read().await;
        let hours_since_creation = chrono::Utc::now()
            .signed_duration_since(current_key.created_at)
            .num_hours() as u64;
        
        hours_since_creation >= self.config.key_rotation_hours
    }

    /// 获取当前密钥版本
    pub async fn get_current_key_version(&self) -> u32 {
        self.current_key.read().await.version
    }

    /// 清理过期的历史密钥
    #[instrument(skip(self))]
    pub async fn cleanup_expired_keys(&self, retention_days: u64) -> Result<()> {
        let mut key_history = self.key_history.write().await;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);

        let expired_versions: Vec<u32> = key_history
            .iter()
            .filter(|(_, key_info)| key_info.created_at < cutoff_time)
            .map(|(&version, _)| version)
            .collect();

        for version in expired_versions {
            key_history.remove(&version);
            debug!("清理过期密钥版本: {}", version);
        }

        Ok(())
    }

    /// 数据脱敏
    #[instrument(skip(self, data))]
    pub fn mask_sensitive_data(&self, data: &str, field_name: &str) -> String {
        if !self.config.sensitive_fields.contains(&field_name.to_string()) {
            return data.to_string();
        }

        match field_name {
            "email" => {
                if let Some(at_pos) = data.find('@') {
                    let (local, domain) = data.split_at(at_pos);
                    if local.len() > 2 {
                        format!("{}***{}", &local[..2], domain)
                    } else {
                        format!("***{}", domain)
                    }
                } else {
                    "***".to_string()
                }
            }
            "phone" => {
                if data.len() > 4 {
                    format!("***-***-{}", &data[data.len()-4..])
                } else {
                    "***".to_string()
                }
            }
            "credit_card" | "bank_account" => {
                if data.len() > 4 {
                    format!("****-****-****-{}", &data[data.len()-4..])
                } else {
                    "****".to_string()
                }
            }
            _ => "***".to_string(),
        }
    }
}
