//! API密钥轮换管理模块

use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn, error, instrument};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// 密钥ID
    pub id: String,
    /// 密钥值
    pub key: String,
    /// 密钥名称
    pub name: String,
    /// 所属用户ID
    pub user_id: String,
    /// 权限范围
    pub scopes: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 是否激活
    pub is_active: bool,
    /// 使用次数
    pub usage_count: u64,
    /// 速率限制（每分钟请求数）
    pub rate_limit: Option<u32>,
}

/// 密钥轮换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// 密钥默认有效期（天）
    pub default_expiry_days: u32,
    /// 轮换提醒时间（天）
    pub rotation_warning_days: u32,
    /// 最大密钥数量（每用户）
    pub max_keys_per_user: u32,
    /// 自动轮换间隔（天）
    pub auto_rotation_interval_days: Option<u32>,
    /// 密钥长度
    pub key_length: usize,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            default_expiry_days: 90,
            rotation_warning_days: 7,
            max_keys_per_user: 10,
            auto_rotation_interval_days: Some(30),
            key_length: 32,
        }
    }
}

/// 密钥轮换管理器
pub struct KeyRotationManager {
    /// 配置
    config: KeyRotationConfig,
    /// 密钥存储（实际应用中应使用数据库）
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// 用户密钥索引
    user_keys: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl KeyRotationManager {
    /// 创建新的密钥轮换管理器
    pub fn new(config: KeyRotationConfig) -> Self {
        Self {
            config,
            keys: Arc::new(RwLock::new(HashMap::new())),
            user_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 生成新的API密钥
    #[instrument(skip(self))]
    pub async fn generate_api_key(
        &self,
        user_id: &str,
        name: &str,
        scopes: Vec<String>,
        expires_in_days: Option<u32>,
    ) -> Result<ApiKey> {
        // 检查用户密钥数量限制
        let user_keys = self.user_keys.read().await;
        if let Some(keys) = user_keys.get(user_id) {
            if keys.len() >= self.config.max_keys_per_user as usize {
                return Err(DuckHubError::validation(
                    format!("用户密钥数量已达上限: {}", self.config.max_keys_per_user)
                ));
            }
        }
        drop(user_keys);

        // 生成密钥
        let key_id = Uuid::new_v4().to_string();
        let key_value = self.generate_secure_key();
        let now = Utc::now();
        let expires_at = expires_in_days
            .or(Some(self.config.default_expiry_days))
            .map(|days| now + Duration::days(days as i64));

        let api_key = ApiKey {
            id: key_id.clone(),
            key: key_value,
            name: name.to_string(),
            user_id: user_id.to_string(),
            scopes,
            created_at: now,
            expires_at,
            last_used_at: None,
            is_active: true,
            usage_count: 0,
            rate_limit: Some(100), // 默认每分钟100次请求
        };

        // 存储密钥
        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), api_key.clone());
        drop(keys);

        // 更新用户密钥索引
        let mut user_keys = self.user_keys.write().await;
        user_keys.entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(key_id);

        info!("生成新API密钥: user_id={}, key_id={}, name={}", user_id, api_key.id, name);
        Ok(api_key)
    }

    /// 验证API密钥
    #[instrument(skip(self))]
    pub async fn validate_api_key(&self, key: &str) -> Result<ApiKey> {
        let mut keys = self.keys.write().await;
        
        // 查找密钥
        let api_key = keys.values_mut()
            .find(|k| k.key == key && k.is_active)
            .ok_or_else(|| DuckHubError::validation("无效的API密钥".to_string()))?;

        // 检查过期时间
        if let Some(expires_at) = api_key.expires_at {
            if Utc::now() > expires_at {
                api_key.is_active = false;
                return Err(DuckHubError::validation("API密钥已过期".to_string()));
            }
        }

        // 更新使用信息
        api_key.last_used_at = Some(Utc::now());
        api_key.usage_count += 1;

        Ok(api_key.clone())
    }

    /// 轮换API密钥
    #[instrument(skip(self))]
    pub async fn rotate_api_key(&self, key_id: &str) -> Result<ApiKey> {
        let keys = self.keys.read().await;
        let old_key = keys.get(key_id)
            .ok_or_else(|| DuckHubError::validation("密钥不存在".to_string()))?
            .clone();
        drop(keys);

        // 生成新密钥
        let new_key = self.generate_api_key(
            &old_key.user_id,
            &format!("{} (轮换)", old_key.name),
            old_key.scopes,
            None,
        ).await?;

        // 停用旧密钥
        self.revoke_api_key(key_id).await?;

        info!("API密钥轮换完成: old_key_id={}, new_key_id={}", key_id, new_key.id);
        Ok(new_key)
    }

    /// 撤销API密钥
    #[instrument(skip(self))]
    pub async fn revoke_api_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        if let Some(api_key) = keys.get_mut(key_id) {
            api_key.is_active = false;
            info!("API密钥已撤销: key_id={}", key_id);
            Ok(())
        } else {
            Err(DuckHubError::validation("密钥不存在".to_string()))
        }
    }

    /// 获取用户的所有密钥
    #[instrument(skip(self))]
    pub async fn get_user_keys(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let user_keys = self.user_keys.read().await;
        let key_ids = user_keys.get(user_id)
            .ok_or_else(|| DuckHubError::validation("用户无API密钥".to_string()))?;

        let keys = self.keys.read().await;
        let user_api_keys: Vec<ApiKey> = key_ids.iter()
            .filter_map(|id| keys.get(id))
            .cloned()
            .collect();

        Ok(user_api_keys)
    }

    /// 检查即将过期的密钥
    #[instrument(skip(self))]
    pub async fn get_expiring_keys(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        let warning_time = Utc::now() + Duration::days(self.config.rotation_warning_days as i64);
        
        let expiring_keys: Vec<ApiKey> = keys.values()
            .filter(|key| {
                key.is_active && 
                key.expires_at.map_or(false, |exp| exp <= warning_time)
            })
            .cloned()
            .collect();

        Ok(expiring_keys)
    }

    /// 生成安全的密钥字符串
    fn generate_secure_key(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        
        let mut rng = rand::thread_rng();
        let key: String = (0..self.config.key_length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        
        format!("dk_{}", key) // DuckHub密钥前缀
    }
}
