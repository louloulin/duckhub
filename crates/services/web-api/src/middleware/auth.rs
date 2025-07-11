//! 认证中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
    http::StatusCode,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;
use tracing::{warn, debug};

/// 认证中间件
pub struct AuthMiddleware {
    /// 跳过认证的路径
    skip_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            skip_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/api/v1/auth/login".to_string(),
                "/api/v1/auth/refresh".to_string(),
            ],
        }
    }

    pub fn with_skip_paths(mut self, paths: Vec<String>) -> Self {
        self.skip_paths.extend(paths);
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service,
            skip_paths: self.skip_paths.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    skip_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        
        // 检查是否需要跳过认证
        if self.skip_paths.iter().any(|skip_path| path.starts_with(skip_path)) {
            debug!("跳过认证检查: {}", path);
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // 提取Authorization头
        let auth_header = req.headers().get("Authorization");
        
        if let Some(auth_value) = auth_header {
            if let Ok(auth_str) = auth_value.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    // 验证JWT令牌
                    match validate_jwt_token(token) {
                        Ok(user_id) => {
                            // 将用户ID添加到请求扩展中
                            req.extensions_mut().insert(user_id);
                            debug!("认证成功，用户ID: {}", user_id);
                            
                            let fut = self.service.call(req);
                            return Box::pin(async move { fut.await });
                        }
                        Err(e) => {
                            warn!("JWT令牌验证失败: {}", e);
                        }
                    }
                }
            }
        }

        // 认证失败，返回401
        warn!("认证失败，路径: {}", path);
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("认证失败，请提供有效的访问令牌"))
        })
    }
}

/// 验证JWT令牌
fn validate_jwt_token(token: &str) -> Result<Uuid, String> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TokenClaims {
        pub sub: String,      // 用户ID
        pub username: String, // 用户名
        pub roles: Vec<String>, // 角色
        pub exp: usize,       // 过期时间
        pub iat: usize,       // 签发时间
    }

    // 在实际应用中，这个密钥应该从配置中获取
    let secret = "your-secret-key";
    let key = DecodingKey::from_secret(secret.as_ref());

    let validation = Validation::new(Algorithm::HS256);

    match decode::<TokenClaims>(token, &key, &validation) {
        Ok(token_data) => {
            let user_id = Uuid::parse_str(&token_data.claims.sub)
                .map_err(|e| format!("无效的用户ID格式: {}", e))?;
            Ok(user_id)
        }
        Err(e) => Err(format!("JWT解码失败: {}", e)),
    }
}

/// 生成JWT令牌
pub fn generate_jwt_token(user_id: &Uuid, secret: &str, expires_in: u64) -> Result<String, String> {
    use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,  // 用户ID
        exp: usize,   // 过期时间
        iat: usize,   // 签发时间
        iss: String,  // 签发者
        aud: String,  // 受众
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: now + expires_in as usize,
        iat: now,
        iss: "duckhub".to_string(),
        aud: "duckhub-users".to_string(),
    };

    let key = EncodingKey::from_secret(secret.as_ref());
    let header = Header::new(Algorithm::HS256);

    encode(&header, &claims, &key)
        .map_err(|e| format!("JWT编码失败: {}", e))
}

/// 从请求中提取用户ID
pub fn extract_user_id(req: &ServiceRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_generation_and_validation() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";
        let expires_in = 3600; // 1小时

        // 生成令牌
        let token = generate_jwt_token(&user_id, secret, expires_in).unwrap();
        assert!(!token.is_empty());

        // 验证令牌
        let validated_user_id = validate_jwt_token(&token).unwrap();
        assert_eq!(user_id, validated_user_id);
    }

    #[test]
    fn test_invalid_jwt_token() {
        let result = validate_jwt_token("invalid.token.here");
        assert!(result.is_err());
    }
}
