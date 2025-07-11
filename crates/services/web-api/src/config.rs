//! Web API服务配置

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Web API服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebApiConfig {
    /// 服务主机地址
    pub host: String,
    /// 服务端口
    pub port: u16,
    /// 工作线程数
    pub workers: usize,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 最大请求体大小（字节）
    pub max_request_size: usize,
    /// JWT配置
    pub jwt: JwtConfig,
    /// CORS配置
    pub cors: CorsConfig,
    /// 限流配置
    pub rate_limit: RateLimitConfig,
    /// 缓存配置
    pub cache: CacheConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

impl Default for WebApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            request_timeout: 30,
            max_request_size: 10 * 1024 * 1024, // 10MB
            jwt: JwtConfig::default(),
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cache: CacheConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// JWT配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT密钥
    pub secret: String,
    /// 访问令牌过期时间（秒）
    pub access_token_expiry: u64,
    /// 刷新令牌过期时间（秒）
    pub refresh_token_expiry: u64,
    /// 发行者
    pub issuer: String,
    /// 受众
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "duckhub-secret-key-change-in-production".to_string(),
            access_token_expiry: 3600,      // 1小时
            refresh_token_expiry: 86400 * 7, // 7天
            issuer: "duckhub".to_string(),
            audience: "duckhub-users".to_string(),
        }
    }
}

/// CORS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 允许的源
    pub allowed_origins: Vec<String>,
    /// 允许的方法
    pub allowed_methods: Vec<String>,
    /// 允许的头部
    pub allowed_headers: Vec<String>,
    /// 最大年龄（秒）
    pub max_age: u64,
    /// 是否允许凭证
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
            ],
            max_age: 3600,
            allow_credentials: true,
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用限流
    pub enabled: bool,
    /// 每分钟请求数限制
    pub requests_per_minute: u32,
    /// 突发请求数限制
    pub burst_size: u32,
    /// 限流窗口大小（秒）
    pub window_size: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_size: 100,
            window_size: 60,
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 默认缓存TTL（秒）
    pub default_ttl: u64,
    /// 查询结果缓存TTL（秒）
    pub query_result_ttl: u64,
    /// 用户会话缓存TTL（秒）
    pub session_ttl: u64,
    /// 最大缓存大小（条目数）
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl: 300,      // 5分钟
            query_result_ttl: 600, // 10分钟
            session_ttl: 3600,     // 1小时
            max_size: 10000,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 是否启用JSON格式
    pub json_format: bool,
    /// 是否记录请求详情
    pub log_requests: bool,
    /// 是否记录响应详情
    pub log_responses: bool,
    /// 慢查询阈值（毫秒）
    pub slow_query_threshold: u64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json_format: false,
            log_requests: true,
            log_responses: false,
            slow_query_threshold: 1000, // 1秒
        }
    }
}

impl WebApiConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("DUCKHUB_API"))
            .build()?;

        cfg.try_deserialize()
    }

    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;

        cfg.try_deserialize()
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("端口号不能为0".to_string());
        }

        if self.workers == 0 {
            return Err("工作线程数不能为0".to_string());
        }

        if self.request_timeout == 0 {
            return Err("请求超时时间不能为0".to_string());
        }

        if self.max_request_size == 0 {
            return Err("最大请求体大小不能为0".to_string());
        }

        if self.jwt.secret.is_empty() {
            return Err("JWT密钥不能为空".to_string());
        }

        if self.jwt.access_token_expiry == 0 {
            return Err("访问令牌过期时间不能为0".to_string());
        }

        Ok(())
    }

    /// 获取请求超时时间
    pub fn request_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.request_timeout)
    }

    /// 获取访问令牌过期时间
    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.jwt.access_token_expiry)
    }

    /// 获取刷新令牌过期时间
    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.jwt.refresh_token_expiry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WebApiConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.workers > 0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = WebApiConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = 8080;
        config.jwt.secret = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_duration_methods() {
        let config = WebApiConfig::default();
        assert_eq!(config.request_timeout_duration(), Duration::from_secs(30));
        assert_eq!(config.access_token_duration(), Duration::from_secs(3600));
    }
}
