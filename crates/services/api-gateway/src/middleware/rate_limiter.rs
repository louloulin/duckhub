//! API限流中间件
//! 基于Redis的分布式限流实现，支持多种限流策略

use duckhub_common::prelude::*;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn, error, instrument};

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 默认限制 (请求数/时间窗口)
    pub default_limit: u64,
    /// 时间窗口 (秒)
    pub window_seconds: u64,
    /// Redis连接字符串
    pub redis_url: String,
    /// 是否启用限流
    pub enabled: bool,
    /// 特定路径的限制配置
    pub path_limits: HashMap<String, PathLimitConfig>,
    /// 用户级别限制
    pub user_limits: HashMap<String, UserLimitConfig>,
}

/// 路径限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathLimitConfig {
    /// 请求限制
    pub limit: u64,
    /// 时间窗口
    pub window_seconds: u64,
    /// 是否需要认证
    pub require_auth: bool,
}

/// 用户限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLimitConfig {
    /// 请求限制
    pub limit: u64,
    /// 时间窗口
    pub window_seconds: u64,
    /// 用户类型
    pub user_type: UserType,
}

/// 用户类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserType {
    /// 免费用户
    Free,
    /// 付费用户
    Premium,
    /// 企业用户
    Enterprise,
    /// 管理员
    Admin,
}

/// 限流器状态
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// 当前请求数
    pub current_requests: u64,
    /// 限制数量
    pub limit: u64,
    /// 重置时间
    pub reset_time: u64,
    /// 剩余请求数
    pub remaining: u64,
}

/// 限流中间件
pub struct RateLimiter {
    /// Redis客户端
    redis_client: Arc<RedisClient>,
    /// 配置
    config: RateLimitConfig,
    /// 内存缓存 (用于Redis不可用时的降级)
    memory_cache: Arc<RwLock<HashMap<String, RateLimitState>>>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut path_limits = HashMap::new();
        
        // API路径限制配置
        path_limits.insert("/api/auth/login".to_string(), PathLimitConfig {
            limit: 5,
            window_seconds: 300, // 5分钟
            require_auth: false,
        });
        
        path_limits.insert("/api/query/execute".to_string(), PathLimitConfig {
            limit: 100,
            window_seconds: 60, // 1分钟
            require_auth: true,
        });
        
        path_limits.insert("/api/data/upload".to_string(), PathLimitConfig {
            limit: 10,
            window_seconds: 60,
            require_auth: true,
        });

        let mut user_limits = HashMap::new();
        user_limits.insert("free".to_string(), UserLimitConfig {
            limit: 1000,
            window_seconds: 3600, // 1小时
            user_type: UserType::Free,
        });
        
        user_limits.insert("premium".to_string(), UserLimitConfig {
            limit: 10000,
            window_seconds: 3600,
            user_type: UserType::Premium,
        });

        Self {
            default_limit: 100,
            window_seconds: 60,
            redis_url: "redis://localhost:6379".to_string(),
            enabled: true,
            path_limits,
            user_limits,
        }
    }
}

impl RateLimiter {
    /// 创建新的限流器
    pub async fn new(config: RateLimitConfig) -> Result<Self> {
        let redis_client = Arc::new(
            RedisClient::open(config.redis_url.clone())
                .map_err(|e| DuckHubError::internal(format!("Redis连接失败: {}", e)))?
        );

        // 测试Redis连接
        let mut conn = redis_client.get_async_connection().await
            .map_err(|e| DuckHubError::internal(format!("Redis连接测试失败: {}", e)))?;
        
        let _: String = conn.ping().await
            .map_err(|e| DuckHubError::internal(format!("Redis ping失败: {}", e)))?;

        Ok(Self {
            redis_client,
            config,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 检查限流
    #[instrument(skip(self))]
    pub async fn check_rate_limit(&self, key: &str, limit: u64, window_seconds: u64) -> Result<RateLimitState> {
        if !self.config.enabled {
            return Ok(RateLimitState {
                current_requests: 0,
                limit,
                reset_time: 0,
                remaining: limit,
            });
        }

        // 尝试使用Redis
        match self.check_redis_rate_limit(key, limit, window_seconds).await {
            Ok(state) => Ok(state),
            Err(e) => {
                warn!("Redis限流检查失败，使用内存降级: {}", e);
                self.check_memory_rate_limit(key, limit, window_seconds).await
            }
        }
    }

    /// Redis限流检查
    async fn check_redis_rate_limit(&self, key: &str, limit: u64, window_seconds: u64) -> Result<RateLimitState> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| DuckHubError::internal(format!("Redis连接失败: {}", e)))?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let window_start = now - (now % window_seconds);
        let redis_key = format!("rate_limit:{}:{}", key, window_start);

        // 使用Redis INCR命令实现原子性计数
        let current_requests: u64 = conn.incr(&redis_key, 1).await
            .map_err(|e| DuckHubError::internal(format!("Redis INCR失败: {}", e)))?;

        // 设置过期时间
        if current_requests == 1 {
            let _: () = conn.expire(&redis_key, window_seconds as usize).await
                .map_err(|e| DuckHubError::internal(format!("Redis EXPIRE失败: {}", e)))?;
        }

        let reset_time = window_start + window_seconds;
        let remaining = if current_requests > limit { 0 } else { limit - current_requests };

        Ok(RateLimitState {
            current_requests,
            limit,
            reset_time,
            remaining,
        })
    }

    /// 内存限流检查 (降级方案)
    async fn check_memory_rate_limit(&self, key: &str, limit: u64, window_seconds: u64) -> Result<RateLimitState> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let window_start = now - (now % window_seconds);
        let cache_key = format!("{}:{}", key, window_start);

        let mut cache = self.memory_cache.write().await;
        
        // 清理过期的缓存项
        cache.retain(|k, state| {
            let key_time: u64 = k.split(':').last().unwrap_or("0").parse().unwrap_or(0);
            key_time + window_seconds > now
        });

        let state = cache.entry(cache_key).or_insert(RateLimitState {
            current_requests: 0,
            limit,
            reset_time: window_start + window_seconds,
            remaining: limit,
        });

        state.current_requests += 1;
        state.remaining = if state.current_requests > limit { 0 } else { limit - state.current_requests };

        Ok(state.clone())
    }

    /// 获取限流键
    fn get_rate_limit_key(&self, headers: &HeaderMap, path: &str, user_id: Option<&str>) -> String {
        // 优先使用用户ID
        if let Some(uid) = user_id {
            return format!("user:{}", uid);
        }

        // 使用IP地址
        if let Some(ip) = self.get_client_ip(headers) {
            return format!("ip:{}:{}", ip, path);
        }

        // 降级到路径
        format!("path:{}", path)
    }

    /// 获取客户端IP
    fn get_client_ip(&self, headers: &HeaderMap) -> Option<String> {
        // 检查常见的代理头
        let ip_headers = [
            "x-forwarded-for",
            "x-real-ip",
            "cf-connecting-ip",
            "x-client-ip",
        ];

        for header_name in &ip_headers {
            if let Some(header_value) = headers.get(header_name) {
                if let Ok(ip_str) = header_value.to_str() {
                    // 取第一个IP (处理逗号分隔的情况)
                    let ip = ip_str.split(',').next().unwrap_or("").trim();
                    if !ip.is_empty() && ip != "unknown" {
                        return Some(ip.to_string());
                    }
                }
            }
        }

        None
    }

    /// 获取路径限制配置
    fn get_path_limit(&self, path: &str) -> Option<&PathLimitConfig> {
        // 精确匹配
        if let Some(config) = self.config.path_limits.get(path) {
            return Some(config);
        }

        // 前缀匹配
        for (pattern, config) in &self.config.path_limits {
            if path.starts_with(pattern) {
                return Some(config);
            }
        }

        None
    }

    /// 获取用户限制配置
    fn get_user_limit(&self, user_type: &str) -> Option<&UserLimitConfig> {
        self.config.user_limits.get(user_type)
    }
}

/// 限流中间件函数
#[instrument(skip(rate_limiter, request, next))]
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let path = request.uri().path();
    
    // 提取用户信息 (从JWT token或session)
    let user_id = extract_user_id(headers);
    let user_type = extract_user_type(headers);

    // 获取限流配置
    let (limit, window_seconds) = if let Some(path_config) = rate_limiter.get_path_limit(path) {
        (path_config.limit, path_config.window_seconds)
    } else if let Some(user_config) = user_type.as_ref().and_then(|t| rate_limiter.get_user_limit(t)) {
        (user_config.limit, user_config.window_seconds)
    } else {
        (rate_limiter.config.default_limit, rate_limiter.config.window_seconds)
    };

    // 生成限流键
    let rate_limit_key = rate_limiter.get_rate_limit_key(headers, path, user_id.as_deref());

    // 检查限流
    match rate_limiter.check_rate_limit(&rate_limit_key, limit, window_seconds).await {
        Ok(state) => {
            debug!("限流检查: key={}, current={}, limit={}, remaining={}", 
                   rate_limit_key, state.current_requests, state.limit, state.remaining);

            // 添加限流头信息
            request.headers_mut().insert("x-ratelimit-limit", limit.to_string().parse().unwrap());
            request.headers_mut().insert("x-ratelimit-remaining", state.remaining.to_string().parse().unwrap());
            request.headers_mut().insert("x-ratelimit-reset", state.reset_time.to_string().parse().unwrap());

            if state.current_requests > limit {
                warn!("限流触发: key={}, requests={}, limit={}", rate_limit_key, state.current_requests, limit);
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }

            // 继续处理请求
            let response = next.run(request).await;
            Ok(response)
        }
        Err(e) => {
            error!("限流检查失败: {}", e);
            // 限流检查失败时，允许请求通过但记录错误
            let response = next.run(request).await;
            Ok(response)
        }
    }
}

/// 从请求头提取用户ID
fn extract_user_id(headers: &HeaderMap) -> Option<String> {
    // 从Authorization头提取JWT token中的用户ID
    // 这里是简化实现，实际应该解析JWT
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                // 简化：从token中提取用户ID
                // 实际实现应该验证和解析JWT
                return Some("user_from_jwt".to_string());
            }
        }
    }
    None
}

/// 从请求头提取用户类型
fn extract_user_type(headers: &HeaderMap) -> Option<String> {
    // 从自定义头或JWT中提取用户类型
    if let Some(user_type_header) = headers.get("x-user-type") {
        if let Ok(user_type) = user_type_header.to_str() {
            return Some(user_type.to_string());
        }
    }
    Some("free".to_string()) // 默认为免费用户
}
