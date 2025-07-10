//! JWT令牌管理模块

use crate::User;
use duckhub_common::prelude::*;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, instrument};

/// JWT管理器
pub struct JWTManager {
    /// 编码密钥
    encoding_key: EncodingKey,
    /// 解码密钥
    decoding_key: DecodingKey,
    /// JWT配置
    config: JWTConfig,
}

/// JWT配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWTConfig {
    /// 密钥
    pub secret: String,
    /// 发行者
    pub issuer: String,
    /// 受众
    pub audience: String,
    /// 过期时间（小时）
    pub expiration_hours: u32,
    /// 刷新令牌过期时间（天）
    pub refresh_expiration_days: u32,
    /// 算法
    pub algorithm: String,
}

impl Default for JWTConfig {
    fn default() -> Self {
        Self {
            secret: "your-secret-key-change-in-production".to_string(),
            issuer: "duckhub".to_string(),
            audience: "duckhub-users".to_string(),
            expiration_hours: 24,
            refresh_expiration_days: 30,
            algorithm: "HS256".to_string(),
        }
    }
}

/// JWT声明
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    /// 主题（用户ID）
    pub sub: String,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 发行者
    pub iss: String,
    /// 受众
    pub aud: String,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
    /// 生效时间
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// 令牌类型
    pub token_type: String,
}

/// 令牌对
#[derive(Debug, Serialize)]
pub struct TokenPair {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: String,
    /// 过期时间（秒）
    pub expires_in: u32,
}

impl JWTManager {
    /// 创建新的JWT管理器
    pub fn new(config: JWTConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.secret.as_ref());

        Ok(Self {
            encoding_key,
            decoding_key,
            config,
        })
    }

    /// 生成访问令牌
    #[instrument(skip(self))]
    pub fn generate_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.expiration_hours as i64);

        let claims = TokenClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| DuckHubError::validation(format!("JWT编码失败: {}", e)))?;

        debug!("生成访问令牌成功，用户: {}", user.username);
        Ok(token)
    }

    /// 生成刷新令牌
    #[instrument(skip(self))]
    pub fn generate_refresh_token(&self, user: &User) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.refresh_expiration_days as i64);

        let claims = TokenClaims {
            sub: user.id.clone(),
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
        };

        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &self.encoding_key)
            .map_err(|e| DuckHubError::validation(format!("JWT编码失败: {}", e)))?;

        debug!("生成刷新令牌成功，用户: {}", user.username);
        Ok(token)
    }

    /// 生成令牌对
    #[instrument(skip(self))]
    pub fn generate_token_pair(&self, user: &User) -> Result<TokenPair> {
        let access_token = self.generate_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.expiration_hours * 3600,
        })
    }

    /// 验证令牌
    #[instrument(skip(self))]
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| DuckHubError::validation(format!("JWT验证失败: {}", e)))?;

        debug!("令牌验证成功，用户: {}", token_data.claims.username);
        Ok(token_data.claims)
    }

    /// 刷新令牌
    #[instrument(skip(self))]
    pub fn refresh_token(&self, refresh_token: &str, user: &User) -> Result<TokenPair> {
        // 验证刷新令牌
        let claims = self.verify_token(refresh_token)?;
        
        // 检查令牌类型
        if claims.token_type != "refresh" {
            return Err(DuckHubError::validation("无效的刷新令牌类型".to_string()));
        }

        // 检查用户ID是否匹配
        if claims.sub != user.id {
            return Err(DuckHubError::validation("令牌用户不匹配".to_string()));
        }

        // 生成新的令牌对
        self.generate_token_pair(user)
    }

    /// 从令牌中提取用户ID
    pub fn extract_user_id(&self, token: &str) -> Result<String> {
        let claims = self.verify_token(token)?;
        Ok(claims.sub)
    }

    /// 检查令牌是否即将过期
    pub fn is_token_expiring_soon(&self, token: &str, threshold_minutes: i64) -> Result<bool> {
        let claims = self.verify_token(token)?;
        let exp_time = DateTime::from_timestamp(claims.exp, 0)
            .ok_or_else(|| DuckHubError::validation("无效的过期时间".to_string()))?;
        let threshold_time = Utc::now() + Duration::minutes(threshold_minutes);
        
        Ok(exp_time <= threshold_time)
    }

    /// 获取令牌剩余有效时间
    pub fn get_token_remaining_time(&self, token: &str) -> Result<Duration> {
        let claims = self.verify_token(token)?;
        let exp_time = DateTime::from_timestamp(claims.exp, 0)
            .ok_or_else(|| DuckHubError::validation("无效的过期时间".to_string()))?;
        let now = Utc::now();
        
        if exp_time > now {
            Ok(exp_time - now)
        } else {
            Ok(Duration::zero())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_user() -> User {
        User {
            id: "test-user-id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_jwt_generation_and_verification() {
        let config = JWTConfig::default();
        let jwt_manager = JWTManager::new(config).unwrap();
        let user = create_test_user();

        // 生成令牌
        let token = jwt_manager.generate_token(&user).unwrap();
        assert!(!token.is_empty());

        // 验证令牌
        let claims = jwt_manager.verify_token(&token).unwrap();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.email, user.email);
    }

    #[test]
    fn test_token_pair_generation() {
        let config = JWTConfig::default();
        let jwt_manager = JWTManager::new(config).unwrap();
        let user = create_test_user();

        let token_pair = jwt_manager.generate_token_pair(&user).unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert_eq!(token_pair.token_type, "Bearer");
    }

    #[test]
    fn test_token_refresh() {
        let config = JWTConfig::default();
        let jwt_manager = JWTManager::new(config).unwrap();
        let user = create_test_user();

        // 生成初始令牌对
        let initial_pair = jwt_manager.generate_token_pair(&user).unwrap();
        
        // 使用刷新令牌生成新的令牌对
        let new_pair = jwt_manager.refresh_token(&initial_pair.refresh_token, &user).unwrap();
        assert!(!new_pair.access_token.is_empty());
        assert_ne!(initial_pair.access_token, new_pair.access_token);
    }

    #[test]
    fn test_user_id_extraction() {
        let config = JWTConfig::default();
        let jwt_manager = JWTManager::new(config).unwrap();
        let user = create_test_user();

        let token = jwt_manager.generate_token(&user).unwrap();
        let extracted_id = jwt_manager.extract_user_id(&token).unwrap();
        assert_eq!(extracted_id, user.id);
    }

    #[test]
    fn test_invalid_token() {
        let config = JWTConfig::default();
        let jwt_manager = JWTManager::new(config).unwrap();

        let result = jwt_manager.verify_token("invalid.token.here");
        assert!(result.is_err());
    }
}
