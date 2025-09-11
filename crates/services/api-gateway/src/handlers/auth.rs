//! 认证相关的API处理器

use duckhub_common::prelude::*;
use duckhub_auth::{JWTManager, JWTConfig, KeyRotationManager, ApiKey};
use serde::{Deserialize, Serialize};
use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use tracing::{info, warn, error, instrument};
use std::sync::Arc;

/// 应用状态
pub struct AuthAppState {
    pub jwt_manager: Arc<JWTManager>,
    pub key_rotation_manager: Arc<KeyRotationManager>,
}

/// 令牌刷新请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// 刷新令牌
    pub refresh_token: String,
}

/// 令牌刷新响应
#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    /// 新的访问令牌
    pub access_token: String,
    /// 新的刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: String,
    /// 过期时间（秒）
    pub expires_in: u32,
}

/// API密钥生成请求
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    /// 密钥名称
    pub name: String,
    /// 权限范围
    pub scopes: Vec<String>,
    /// 过期天数（可选）
    pub expires_in_days: Option<u32>,
}

/// API密钥响应
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    /// 密钥ID
    pub id: String,
    /// 密钥值（仅在创建时返回）
    pub key: Option<String>,
    /// 密钥名称
    pub name: String,
    /// 权限范围
    pub scopes: Vec<String>,
    /// 创建时间
    pub created_at: String,
    /// 过期时间
    pub expires_at: Option<String>,
    /// 是否激活
    pub is_active: bool,
    /// 使用次数
    pub usage_count: u64,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            key: None, // 安全考虑，不返回密钥值
            name: key.name,
            scopes: key.scopes,
            created_at: key.created_at.to_rfc3339(),
            expires_at: key.expires_at.map(|dt| dt.to_rfc3339()),
            is_active: key.is_active,
            usage_count: key.usage_count,
        }
    }
}

/// 刷新访问令牌
#[instrument(skip(state, request))]
pub async fn refresh_token(
    State(state): State<Arc<AuthAppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    info!("收到令牌刷新请求");

    // 刷新令牌
    let (new_access_token, new_refresh_token) = state.jwt_manager
        .refresh_access_token(&request.refresh_token)
        .map_err(|e| {
            warn!("令牌刷新失败: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let response = RefreshTokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1小时
    };

    info!("令牌刷新成功");
    Ok(Json(response))
}

/// 撤销令牌
#[instrument(skip(state, request))]
pub async fn revoke_token(
    State(state): State<Arc<AuthAppState>>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<StatusCode, StatusCode> {
    info!("收到令牌撤销请求");

    state.jwt_manager
        .revoke_token(&request.refresh_token)
        .map_err(|e| {
            warn!("令牌撤销失败: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    info!("令牌撤销成功");
    Ok(StatusCode::NO_CONTENT)
}

/// 创建API密钥
#[instrument(skip(state, request))]
pub async fn create_api_key(
    State(state): State<Arc<AuthAppState>>,
    user_id: String, // 从JWT中提取
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, StatusCode> {
    info!("收到API密钥创建请求: user_id={}, name={}", user_id, request.name);

    let api_key = state.key_rotation_manager
        .generate_api_key(&user_id, &request.name, request.scopes, request.expires_in_days)
        .await
        .map_err(|e| {
            error!("API密钥创建失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut response = ApiKeyResponse::from(api_key.clone());
    response.key = Some(api_key.key); // 仅在创建时返回密钥值

    info!("API密钥创建成功: key_id={}", api_key.id);
    Ok(Json(response))
}

/// 获取用户的API密钥列表
#[instrument(skip(state))]
pub async fn list_api_keys(
    State(state): State<Arc<AuthAppState>>,
    user_id: String, // 从JWT中提取
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    info!("收到API密钥列表请求: user_id={}", user_id);

    let keys = state.key_rotation_manager
        .get_user_keys(&user_id)
        .await
        .map_err(|e| {
            error!("获取API密钥列表失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<ApiKeyResponse> = keys.into_iter()
        .map(ApiKeyResponse::from)
        .collect();

    info!("API密钥列表获取成功: count={}", response.len());
    Ok(Json(response))
}

/// 轮换API密钥
#[instrument(skip(state))]
pub async fn rotate_api_key(
    State(state): State<Arc<AuthAppState>>,
    Path(key_id): Path<String>,
) -> Result<Json<ApiKeyResponse>, StatusCode> {
    info!("收到API密钥轮换请求: key_id={}", key_id);

    let new_key = state.key_rotation_manager
        .rotate_api_key(&key_id)
        .await
        .map_err(|e| {
            error!("API密钥轮换失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut response = ApiKeyResponse::from(new_key.clone());
    response.key = Some(new_key.key); // 返回新密钥值

    info!("API密钥轮换成功: old_key_id={}, new_key_id={}", key_id, new_key.id);
    Ok(Json(response))
}

/// 撤销API密钥
#[instrument(skip(state))]
pub async fn revoke_api_key(
    State(state): State<Arc<AuthAppState>>,
    Path(key_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    info!("收到API密钥撤销请求: key_id={}", key_id);

    state.key_rotation_manager
        .revoke_api_key(&key_id)
        .await
        .map_err(|e| {
            error!("API密钥撤销失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("API密钥撤销成功: key_id={}", key_id);
    Ok(StatusCode::NO_CONTENT)
}

/// 获取即将过期的密钥
#[instrument(skip(state))]
pub async fn get_expiring_keys(
    State(state): State<Arc<AuthAppState>>,
) -> Result<Json<Vec<ApiKeyResponse>>, StatusCode> {
    info!("收到即将过期密钥查询请求");

    let keys = state.key_rotation_manager
        .get_expiring_keys()
        .await
        .map_err(|e| {
            error!("获取即将过期密钥失败: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<ApiKeyResponse> = keys.into_iter()
        .map(ApiKeyResponse::from)
        .collect();

    info!("即将过期密钥查询成功: count={}", response.len());
    Ok(Json(response))
}
