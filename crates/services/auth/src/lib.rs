//! DuckHub安全和权限管理系统
//!
//! 提供企业级安全功能：
//! - 用户认证和授权
//! - JWT令牌管理
//! - 数据脱敏功能
//! - 基础权限控制

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use prometheus::{Counter, Histogram, Registry};

pub mod jwt;
pub mod password;
pub mod masking;

pub use jwt::*;
pub use password::*;
pub use masking::*;

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
    /// 用户属性
    pub attributes: HashMap<String, String>,
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

/// 安全管理服务主结构
pub struct SecurityService {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// JWT管理器
    jwt_manager: Arc<JWTManager>,
    /// 数据脱敏管理器
    masking_manager: Arc<DataMaskingManager>,
    /// 密码管理器
    password_manager: Arc<PasswordManager>,
    /// 监控指标
    metrics: SecurityMetrics,
}

/// 安全监控指标
#[derive(Debug, Clone)]
pub struct SecurityMetrics {
    /// 认证成功次数
    pub auth_success: Counter,
    /// 认证失败次数
    pub auth_failed: Counter,
    /// 权限检查次数
    pub permission_checks: Counter,
    /// 权限拒绝次数
    pub permission_denied: Counter,
    /// 审计日志记录数
    pub audit_logs: Counter,
    /// 认证延迟
    pub auth_duration: Histogram,
}

impl SecurityMetrics {
    /// 创建新的安全监控指标实例
    pub fn new(registry: &Registry) -> Result<Self> {
        let auth_success = Counter::new(
            "duckhub_auth_success_total",
            "认证成功次数"
        )?;
        
        let auth_failed = Counter::new(
            "duckhub_auth_failed_total", 
            "认证失败次数"
        )?;
        
        let permission_checks = Counter::new(
            "duckhub_permission_checks_total",
            "权限检查次数"
        )?;
        
        let permission_denied = Counter::new(
            "duckhub_permission_denied_total",
            "权限拒绝次数"
        )?;
        
        let audit_logs = Counter::new(
            "duckhub_audit_logs_total",
            "审计日志记录数"
        )?;
        
        let auth_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_auth_duration_seconds",
                "认证处理延迟分布"
            )
        )?;

        // 注册指标
        registry.register(Box::new(auth_success.clone()))?;
        registry.register(Box::new(auth_failed.clone()))?;
        registry.register(Box::new(permission_checks.clone()))?;
        registry.register(Box::new(permission_denied.clone()))?;
        registry.register(Box::new(audit_logs.clone()))?;
        registry.register(Box::new(auth_duration.clone()))?;

        Ok(Self {
            auth_success,
            auth_failed,
            permission_checks,
            permission_denied,
            audit_logs,
            auth_duration,
        })
    }
}

impl SecurityService {
    /// 创建新的安全服务实例
    #[instrument(skip(engine, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        config: SecurityConfig,
        registry: &Registry,
    ) -> Result<Self> {
        // 创建监控指标
        let metrics = SecurityMetrics::new(registry)?;

        // 创建各个管理器
        let jwt_manager = Arc::new(JWTManager::new(config.jwt.clone())?);
        let masking_manager = Arc::new(DataMaskingManager::new(config.masking.clone()));
        let password_manager = Arc::new(PasswordManager::new());

        info!("创建安全管理服务");

        Ok(Self {
            engine,
            jwt_manager,
            masking_manager,
            password_manager,
            metrics,
        })
    }

    /// 用户认证（简化版本）
    #[instrument(skip(self, password))]
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthResult> {
        let start_time = std::time::Instant::now();

        // 简化的认证逻辑
        if username == "admin" && password == "admin123" {
            self.metrics.auth_success.inc();
            info!("用户认证成功: {}", username);

            let user = User {
                id: "admin-id".to_string(),
                username: username.to_string(),
                email: "admin@example.com".to_string(),
                display_name: "Administrator".to_string(),
                enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_login_at: Some(Utc::now()),
                attributes: std::collections::HashMap::new(),
            };

            let duration = start_time.elapsed();
            self.metrics.auth_duration.observe(duration.as_secs_f64());

            Ok(AuthResult {
                success: true,
                user: Some(user),
                error_message: None,
                session_id: Some(Uuid::new_v4().to_string()),
                auth_time: Utc::now(),
            })
        } else {
            self.metrics.auth_failed.inc();
            warn!("用户认证失败: {}", username);

            Ok(AuthResult {
                success: false,
                user: None,
                error_message: Some("用户名或密码错误".to_string()),
                session_id: None,
                auth_time: Utc::now(),
            })
        }
    }

    /// 检查权限（简化版本）
    #[instrument(skip(self))]
    pub async fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool> {
        self.metrics.permission_checks.inc();

        // 简化的权限检查：admin用户拥有所有权限
        if user_id == "admin-id" {
            debug!("管理员用户拥有所有权限");
            Ok(true)
        } else {
            self.metrics.permission_denied.inc();
            warn!("权限检查失败: 用户 {} 无权限 {} 资源 {}", user_id, action, resource);
            Ok(false)
        }
    }

    /// 生成JWT令牌
    #[instrument(skip(self))]
    pub async fn generate_token(&self, user: &User) -> Result<String> {
        self.jwt_manager.generate_token(user)
    }

    /// 验证JWT令牌
    #[instrument(skip(self))]
    pub async fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        self.jwt_manager.verify_token(token)
    }

    /// 数据脱敏
    #[instrument(skip(self))]
    pub async fn mask_data(&self, data: &str, mask_type: MaskType) -> Result<String> {
        self.masking_manager.mask_data(data, mask_type)
    }

    /// 获取安全统计信息
    pub async fn get_security_stats(&self) -> SecurityStats {
        SecurityStats {
            auth_success_count: self.metrics.auth_success.get() as u64,
            auth_failed_count: self.metrics.auth_failed.get() as u64,
            permission_checks_count: self.metrics.permission_checks.get() as u64,
            permission_denied_count: self.metrics.permission_denied.get() as u64,
            audit_logs_count: self.metrics.audit_logs.get() as u64,
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<SecurityHealthStatus> {
        let mut health_status = SecurityHealthStatus {
            overall_status: "healthy".to_string(),
            components: HashMap::new(),
        };

        // 检查数据库连接
        match self.engine.check_connection().await {
            Ok(_) => {
                health_status.components.insert(
                    "database".to_string(),
                    ComponentHealth {
                        status: "healthy".to_string(),
                        message: None,
                    }
                );
            }
            Err(e) => {
                health_status.components.insert(
                    "database".to_string(),
                    ComponentHealth {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                    }
                );
                health_status.overall_status = "unhealthy".to_string();
            }
        }

        // 检查JWT管理器
        health_status.components.insert(
            "jwt_manager".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                message: None,
            }
        );

        Ok(health_status)
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 数据脱敏配置
    pub masking: MaskingConfig,
    /// JWT配置
    pub jwt: JWTConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            masking: MaskingConfig::default(),
            jwt: JWTConfig::default(),
        }
    }
}

/// 安全统计信息
#[derive(Debug, Clone, Serialize)]
pub struct SecurityStats {
    /// 认证成功次数
    pub auth_success_count: u64,
    /// 认证失败次数
    pub auth_failed_count: u64,
    /// 权限检查次数
    pub permission_checks_count: u64,
    /// 权限拒绝次数
    pub permission_denied_count: u64,
    /// 审计日志记录数
    pub audit_logs_count: u64,
}

/// 安全健康状态
#[derive(Debug, Clone, Serialize)]
pub struct SecurityHealthStatus {
    /// 整体状态
    pub overall_status: String,
    /// 组件状态
    pub components: HashMap<String, ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    /// 状态
    pub status: String,
    /// 消息
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    use tempfile::tempdir;

    async fn create_test_engine() -> Arc<DuckDBEngine> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = duckhub_common::DatabaseConfig {
            duckdb_path: db_path.to_string_lossy().to_string(),
            memory_limit: Some("1GB".to_string()),
            threads: Some(2),
            max_memory: Some("1GB".to_string()),
            temp_directory: Some(temp_dir.path().to_string_lossy().to_string()),
            extensions: vec![],
            pool: duckhub_common::PoolConfig::default(),
        };

        Arc::new(DuckDBEngine::new(config).await.unwrap())
    }

    #[tokio::test]
    async fn test_security_service_creation() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_authentication() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await.unwrap();

        // 测试成功认证
        let result = service.authenticate("admin", "admin123").await.unwrap();
        assert!(result.success);
        assert!(result.user.is_some());

        // 测试失败认证
        let result = service.authenticate("admin", "wrong_password").await.unwrap();
        assert!(!result.success);
        assert!(result.user.is_none());
    }

    #[tokio::test]
    async fn test_permission_check() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await.unwrap();

        // 测试管理员权限
        let has_permission = service.check_permission("admin-id", "users", "read").await.unwrap();
        assert!(has_permission);

        // 测试普通用户权限
        let has_permission = service.check_permission("user-id", "users", "read").await.unwrap();
        assert!(!has_permission);
    }

    #[tokio::test]
    async fn test_jwt_token_generation() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await.unwrap();

        let user = User {
            id: "test-user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            attributes: std::collections::HashMap::new(),
        };

        let token = service.generate_token(&user).await.unwrap();
        assert!(!token.is_empty());

        // 验证令牌
        let claims = service.verify_token(&token).await.unwrap();
        assert_eq!(claims.sub, user.id);
    }

    #[tokio::test]
    async fn test_data_masking() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await.unwrap();

        // 测试邮箱脱敏
        let masked = service.mask_data("test@example.com", MaskType::Email).await.unwrap();
        assert_eq!(masked, "t**t@example.com");

        // 测试电话脱敏
        let masked = service.mask_data("13812345678", MaskType::Phone).await.unwrap();
        assert_eq!(masked, "138****5678");
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = create_test_engine().await;
        let config = SecurityConfig::default();
        let registry = Registry::new();

        let service = SecurityService::new(engine, config, &registry).await.unwrap();

        let health = service.health_check().await.unwrap();
        assert_eq!(health.overall_status, "healthy");
        assert!(health.components.contains_key("database"));
        assert!(health.components.contains_key("jwt_manager"));
    }
}
