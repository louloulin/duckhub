//! 增强的CORS安全中间件

use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn, error, instrument};
use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::Response,
};

/// CORS安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsSecurityConfig {
    /// 允许的源
    pub allowed_origins: HashSet<String>,
    /// 允许的方法
    pub allowed_methods: HashSet<String>,
    /// 允许的头部
    pub allowed_headers: HashSet<String>,
    /// 暴露的头部
    pub exposed_headers: HashSet<String>,
    /// 是否允许凭证
    pub allow_credentials: bool,
    /// 预检请求缓存时间（秒）
    pub max_age: u32,
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否记录CORS违规
    pub log_violations: bool,
}

impl Default for CorsSecurityConfig {
    fn default() -> Self {
        let mut allowed_origins = HashSet::new();
        allowed_origins.insert("http://localhost:3000".to_string());
        allowed_origins.insert("https://duckhub.app".to_string());

        let mut allowed_methods = HashSet::new();
        allowed_methods.insert("GET".to_string());
        allowed_methods.insert("POST".to_string());
        allowed_methods.insert("PUT".to_string());
        allowed_methods.insert("DELETE".to_string());
        allowed_methods.insert("OPTIONS".to_string());

        let mut allowed_headers = HashSet::new();
        allowed_headers.insert("content-type".to_string());
        allowed_headers.insert("authorization".to_string());
        allowed_headers.insert("x-api-key".to_string());
        allowed_headers.insert("x-request-id".to_string());

        let mut exposed_headers = HashSet::new();
        exposed_headers.insert("x-ratelimit-limit".to_string());
        exposed_headers.insert("x-ratelimit-remaining".to_string());
        exposed_headers.insert("x-ratelimit-reset".to_string());

        Self {
            allowed_origins,
            allowed_methods,
            allowed_headers,
            exposed_headers,
            allow_credentials: true,
            max_age: 86400, // 24小时
            strict_mode: true,
            log_violations: true,
        }
    }
}

/// CORS安全中间件
pub struct CorsSecurityMiddleware {
    config: CorsSecurityConfig,
}

impl CorsSecurityMiddleware {
    /// 创建新的CORS安全中间件
    pub fn new(config: CorsSecurityConfig) -> Self {
        Self { config }
    }

    /// 检查源是否被允许
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allowed_origins.contains("*") {
            return true;
        }
        
        self.config.allowed_origins.contains(origin) ||
        self.config.allowed_origins.iter().any(|allowed| {
            // 支持通配符匹配
            if allowed.contains("*") {
                let pattern = allowed.replace("*", ".*");
                regex::Regex::new(&pattern)
                    .map(|re| re.is_match(origin))
                    .unwrap_or(false)
            } else {
                false
            }
        })
    }

    /// 检查方法是否被允许
    fn is_method_allowed(&self, method: &str) -> bool {
        self.config.allowed_methods.contains(method)
    }

    /// 检查头部是否被允许
    fn are_headers_allowed(&self, headers: &[String]) -> bool {
        headers.iter().all(|header| {
            let header_lower = header.to_lowercase();
            self.config.allowed_headers.contains(&header_lower) ||
            // 允许简单头部
            matches!(header_lower.as_str(), 
                "accept" | "accept-language" | "content-language" | 
                "content-type" | "origin" | "referer" | "user-agent"
            )
        })
    }

    /// 处理预检请求
    #[instrument(skip(self))]
    fn handle_preflight(&self, headers: &HeaderMap) -> Result<Response<axum::body::Body>, StatusCode> {
        // 检查Origin
        let origin = headers.get("origin")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        if !self.is_origin_allowed(origin) {
            if self.config.log_violations {
                warn!("CORS违规: 不允许的源 - {}", origin);
            }
            return Err(StatusCode::FORBIDDEN);
        }

        // 检查请求方法
        let method = headers.get("access-control-request-method")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        if !self.is_method_allowed(method) {
            if self.config.log_violations {
                warn!("CORS违规: 不允许的方法 - {}", method);
            }
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }

        // 检查请求头部
        if let Some(headers_value) = headers.get("access-control-request-headers") {
            if let Ok(headers_str) = headers_value.to_str() {
                let requested_headers: Vec<String> = headers_str
                    .split(',')
                    .map(|h| h.trim().to_lowercase())
                    .collect();

                if !self.are_headers_allowed(&requested_headers) {
                    if self.config.log_violations {
                        warn!("CORS违规: 不允许的头部 - {:?}", requested_headers);
                    }
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }

        // 构建预检响应
        let mut response = Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(axum::body::Body::empty())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let response_headers = response.headers_mut();
        
        // 设置CORS头部
        response_headers.insert("access-control-allow-origin", 
            HeaderValue::from_str(origin).unwrap());
        
        response_headers.insert("access-control-allow-methods", 
            HeaderValue::from_str(&self.config.allowed_methods.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap());
        
        response_headers.insert("access-control-allow-headers", 
            HeaderValue::from_str(&self.config.allowed_headers.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap());
        
        if !self.config.exposed_headers.is_empty() {
            response_headers.insert("access-control-expose-headers", 
                HeaderValue::from_str(&self.config.exposed_headers.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap());
        }
        
        if self.config.allow_credentials {
            response_headers.insert("access-control-allow-credentials", 
                HeaderValue::from_static("true"));
        }
        
        response_headers.insert("access-control-max-age", 
            HeaderValue::from_str(&self.config.max_age.to_string()).unwrap());

        // 安全头部
        response_headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
        response_headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
        response_headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
        response_headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));

        debug!("CORS预检请求处理成功: origin={}, method={}", origin, method);
        Ok(response)
    }

    /// 添加CORS头部到响应
    #[instrument(skip(self, response))]
    fn add_cors_headers(&self, response: &mut Response<axum::body::Body>, origin: Option<&str>) {
        let headers = response.headers_mut();

        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                headers.insert("access-control-allow-origin", 
                    HeaderValue::from_str(origin).unwrap());
                
                if self.config.allow_credentials {
                    headers.insert("access-control-allow-credentials", 
                        HeaderValue::from_static("true"));
                }
                
                if !self.config.exposed_headers.is_empty() {
                    headers.insert("access-control-expose-headers", 
                        HeaderValue::from_str(&self.config.exposed_headers.iter().cloned().collect::<Vec<_>>().join(", ")).unwrap());
                }
            }
        }

        // 添加安全头部
        headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
        headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
        headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
        headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
        
        // CSP头部
        headers.insert("content-security-policy", 
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"));
    }
}

/// CORS安全中间件处理函数
#[instrument(skip(middleware, request, next))]
pub async fn cors_security_middleware(
    middleware: &CorsSecurityMiddleware,
    request: Request,
    next: Next,
) -> Result<Response<axum::body::Body>, StatusCode> {
    let headers = request.headers();
    let method = request.method();
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());

    // 处理预检请求
    if method == Method::OPTIONS {
        return middleware.handle_preflight(headers);
    }

    // 检查Origin（对于非简单请求）
    if let Some(origin) = origin {
        if middleware.config.strict_mode && !middleware.is_origin_allowed(origin) {
            if middleware.config.log_violations {
                warn!("CORS违规: 不允许的源 - {}", origin);
            }
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // 继续处理请求
    let mut response = next.run(request).await;
    
    // 添加CORS头部
    middleware.add_cors_headers(&mut response, origin);

    Ok(response)
}
