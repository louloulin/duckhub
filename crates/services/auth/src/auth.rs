//! 用户认证管理模块

use crate::rbac::User;
use crate::password::PasswordManager;
use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};

/// 认证管理器
pub struct AuthManager {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 密码管理器
    password_manager: PasswordManager,
    /// 认证配置
    config: AuthConfig,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 密码最小长度
    pub min_password_length: usize,
    /// 密码最大长度
    pub max_password_length: usize,
    /// 是否需要大写字母
    pub require_uppercase: bool,
    /// 是否需要小写字母
    pub require_lowercase: bool,
    /// 是否需要数字
    pub require_numbers: bool,
    /// 是否需要特殊字符
    pub require_special_chars: bool,
    /// 密码过期天数
    pub password_expiry_days: u32,
    /// 最大登录失败次数
    pub max_login_attempts: u32,
    /// 账户锁定时间（分钟）
    pub lockout_duration_minutes: u32,
    /// 会话超时时间（分钟）
    pub session_timeout_minutes: u32,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            min_password_length: 8,
            max_password_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            password_expiry_days: 90,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            session_timeout_minutes: 480, // 8小时
        }
    }
}

/// 认证结果
#[derive(Debug, Clone, Serialize)]
pub struct AuthResult {
    /// 是否成功
    pub success: bool,
    /// 用户信息
    pub user: Option<User>,
    /// 错误消息
    pub error_message: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 认证时间
    pub auth_time: DateTime<Utc>,
}

/// 用户凭据
#[derive(Debug, Clone)]
pub struct UserCredentials {
    /// 用户ID
    pub user_id: String,
    /// 密码哈希
    pub password_hash: String,
    /// 盐值
    pub salt: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 密码过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 登录失败次数
    pub failed_attempts: u32,
    /// 锁定时间
    pub locked_until: Option<DateTime<Utc>>,
}

/// 用户会话
#[derive(Debug, Clone, Serialize)]
pub struct UserSession {
    /// 会话ID
    pub session_id: String,
    /// 用户ID
    pub user_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: DateTime<Utc>,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 是否活跃
    pub active: bool,
}

impl AuthManager {
    /// 创建新的认证管理器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>, config: AuthConfig) -> Result<Self> {
        let password_manager = PasswordManager::new();
        
        let manager = Self {
            engine,
            password_manager,
            config,
        };

        // 初始化数据库表
        manager.initialize_tables().await?;

        info!("认证管理器初始化完成");
        Ok(manager)
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        // 创建用户凭据表
        let create_credentials_sql = r#"
            CREATE TABLE IF NOT EXISTS auth_credentials (
                user_id VARCHAR PRIMARY KEY,
                password_hash VARCHAR NOT NULL,
                salt VARCHAR NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                expires_at TIMESTAMP,
                failed_attempts INTEGER DEFAULT 0,
                locked_until TIMESTAMP
            )
        "#;
        self.engine.execute(create_credentials_sql).await?;

        // 创建用户会话表
        let create_sessions_sql = r#"
            CREATE TABLE IF NOT EXISTS auth_sessions (
                session_id VARCHAR PRIMARY KEY,
                user_id VARCHAR NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                expires_at TIMESTAMP NOT NULL,
                last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                ip_address VARCHAR,
                user_agent TEXT,
                active BOOLEAN DEFAULT true
            )
        "#;
        self.engine.execute(create_sessions_sql).await?;

        info!("认证数据库表初始化完成");
        Ok(())
    }

    /// 用户认证
    #[instrument(skip(self, password))]
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthResult> {
        // 查询用户信息
        let user_query = "SELECT id, username, email, display_name, enabled FROM rbac_users WHERE username = ? AND enabled = true";
        let user_result = self.engine.query_with_params(user_query, &[username]).await?;
        
        if user_result.rows.is_empty() {
            return Ok(AuthResult {
                success: false,
                user: None,
                error_message: Some("用户不存在或已禁用".to_string()),
                session_id: None,
                auth_time: Utc::now(),
            });
        }

        let user_row = &user_result.rows[0];
        let user_id = user_row.get(0).and_then(|v| v.as_ref()).unwrap_or("").to_string();

        // 检查账户是否被锁定
        if self.is_account_locked(&user_id).await? {
            return Ok(AuthResult {
                success: false,
                user: None,
                error_message: Some("账户已被锁定".to_string()),
                session_id: None,
                auth_time: Utc::now(),
            });
        }

        // 验证密码
        match self.verify_password(&user_id, password).await {
            Ok(true) => {
                // 密码正确，重置失败次数
                self.reset_failed_attempts(&user_id).await?;
                
                // 创建会话
                let session_id = self.create_session(&user_id, None, None).await?;
                
                // 更新最后登录时间
                self.update_last_login(&user_id).await?;

                let user = User {
                    id: user_id,
                    username: user_row.get(1).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
                    email: user_row.get(2).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
                    display_name: user_row.get(3).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
                    enabled: true,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    last_login_at: Some(Utc::now()),
                    attributes: std::collections::HashMap::new(),
                };

                Ok(AuthResult {
                    success: true,
                    user: Some(user),
                    error_message: None,
                    session_id: Some(session_id),
                    auth_time: Utc::now(),
                })
            }
            Ok(false) => {
                // 密码错误，增加失败次数
                self.increment_failed_attempts(&user_id).await?;
                
                Ok(AuthResult {
                    success: false,
                    user: None,
                    error_message: Some("密码错误".to_string()),
                    session_id: None,
                    auth_time: Utc::now(),
                })
            }
            Err(e) => {
                error!("密码验证失败: {}", e);
                Ok(AuthResult {
                    success: false,
                    user: None,
                    error_message: Some("认证失败".to_string()),
                    session_id: None,
                    auth_time: Utc::now(),
                })
            }
        }
    }

    /// 设置用户密码
    #[instrument(skip(self, password))]
    pub async fn set_password(&self, user_id: &str, password: &str) -> Result<()> {
        // 验证密码强度
        self.validate_password(password)?;

        // 生成密码哈希
        let (hash, salt) = self.password_manager.hash_password(password)?;

        // 计算过期时间
        let expires_at = Utc::now() + chrono::Duration::days(self.config.password_expiry_days as i64);

        // 更新或插入凭据
        let upsert_sql = r#"
            INSERT OR REPLACE INTO auth_credentials 
            (user_id, password_hash, salt, updated_at, expires_at, failed_attempts, locked_until)
            VALUES (?, ?, ?, ?, ?, 0, NULL)
        "#;
        
        self.engine.execute_with_params(
            upsert_sql,
            &[
                user_id,
                &hash,
                &salt,
                &Utc::now().to_rfc3339(),
                &expires_at.to_rfc3339(),
            ]
        ).await?;

        info!("用户 {} 密码已更新", user_id);
        Ok(())
    }

    /// 验证密码
    async fn verify_password(&self, user_id: &str, password: &str) -> Result<bool> {
        let query = "SELECT password_hash, salt FROM auth_credentials WHERE user_id = ?";
        let result = self.engine.query_with_params(query, &[user_id]).await?;
        
        if result.rows.is_empty() {
            return Ok(false);
        }

        let row = &result.rows[0];
        let stored_hash = row.get(0).and_then(|v| v.as_ref()).unwrap_or("");
        let salt = row.get(1).and_then(|v| v.as_ref()).unwrap_or("");

        self.password_manager.verify_password(password, stored_hash, salt)
    }

    /// 检查账户是否被锁定
    async fn is_account_locked(&self, user_id: &str) -> Result<bool> {
        let query = "SELECT locked_until FROM auth_credentials WHERE user_id = ?";
        let result = self.engine.query_with_params(query, &[user_id]).await?;
        
        if result.rows.is_empty() {
            return Ok(false);
        }

        if let Some(locked_until_str) = result.rows[0].get(0).and_then(|v| v.as_ref()) {
            if let Ok(locked_until) = chrono::DateTime::parse_from_rfc3339(locked_until_str) {
                return Ok(locked_until.with_timezone(&Utc) > Utc::now());
            }
        }

        Ok(false)
    }

    /// 增加失败次数
    async fn increment_failed_attempts(&self, user_id: &str) -> Result<()> {
        let update_sql = r#"
            UPDATE auth_credentials 
            SET failed_attempts = failed_attempts + 1,
                locked_until = CASE 
                    WHEN failed_attempts + 1 >= ? THEN datetime('now', '+' || ? || ' minutes')
                    ELSE locked_until 
                END
            WHERE user_id = ?
        "#;
        
        self.engine.execute_with_params(
            update_sql,
            &[
                &self.config.max_login_attempts.to_string(),
                &self.config.lockout_duration_minutes.to_string(),
                user_id,
            ]
        ).await?;

        Ok(())
    }

    /// 重置失败次数
    async fn reset_failed_attempts(&self, user_id: &str) -> Result<()> {
        let update_sql = "UPDATE auth_credentials SET failed_attempts = 0, locked_until = NULL WHERE user_id = ?";
        self.engine.execute_with_params(update_sql, &[user_id]).await?;
        Ok(())
    }

    /// 创建会话
    async fn create_session(&self, user_id: &str, ip_address: Option<&str>, user_agent: Option<&str>) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::minutes(self.config.session_timeout_minutes as i64);

        let insert_sql = r#"
            INSERT INTO auth_sessions (session_id, user_id, expires_at, ip_address, user_agent)
            VALUES (?, ?, ?, ?, ?)
        "#;
        
        self.engine.execute_with_params(
            insert_sql,
            &[
                &session_id,
                user_id,
                &expires_at.to_rfc3339(),
                &ip_address.unwrap_or("").to_string(),
                &user_agent.unwrap_or("").to_string(),
            ]
        ).await?;

        Ok(session_id)
    }

    /// 更新最后登录时间
    async fn update_last_login(&self, user_id: &str) -> Result<()> {
        let update_sql = "UPDATE rbac_users SET last_login_at = ? WHERE id = ?";
        self.engine.execute_with_params(
            update_sql,
            &[&Utc::now().to_rfc3339(), user_id]
        ).await?;
        Ok(())
    }

    /// 验证密码强度
    fn validate_password(&self, password: &str) -> Result<()> {
        if password.len() < self.config.min_password_length {
            return Err(DuckHubError::validation(format!(
                "密码长度不能少于{}位", 
                self.config.min_password_length
            )));
        }

        if password.len() > self.config.max_password_length {
            return Err(DuckHubError::validation(format!(
                "密码长度不能超过{}位", 
                self.config.max_password_length
            )));
        }

        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(DuckHubError::validation("密码必须包含大写字母".to_string()));
        }

        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(DuckHubError::validation("密码必须包含小写字母".to_string()));
        }

        if self.config.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(DuckHubError::validation("密码必须包含数字".to_string()));
        }

        if self.config.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(DuckHubError::validation("密码必须包含特殊字符".to_string()));
        }

        Ok(())
    }
}
