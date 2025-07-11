//! 认证处理器

use actix_web::{web, HttpResponse, Result as ActixResult, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn, instrument};
use validator::Validate;
use crate::{AppState, success_response, error_response};

/// 登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// 用户名或邮箱
    #[validate(length(min = 3, max = 100, message = "用户名长度必须在3-100字符之间"))]
    pub username: String,
    /// 密码
    #[validate(length(min = 6, max = 100, message = "密码长度必须在6-100字符之间"))]
    pub password: String,
    /// 记住我
    pub remember_me: Option<bool>,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// 访问令牌
    pub access_token: String,
    /// 刷新令牌
    pub refresh_token: String,
    /// 令牌类型
    pub token_type: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 用户信息
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    /// 用户ID
    pub id: Uuid,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 角色
    pub roles: Vec<String>,
    /// 权限
    pub permissions: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
}

/// 刷新令牌请求
#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    /// 刷新令牌
    #[validate(length(min = 1, message = "刷新令牌不能为空"))]
    pub refresh_token: String,
}

/// 用户登录
#[instrument(skip(app_state, request))]
pub async fn login(
    app_state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    // 验证请求
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("用户登录尝试: {}", request.username);

    // 验证用户凭证
    match app_state.auth_service.authenticate(&request.username, &request.password).await {
        Ok(user) => {
            // 生成访问令牌
            let access_token = match app_state.auth_service.generate_access_token(&user).await {
                Ok(token) => token,
                Err(e) => {
                    error!("生成访问令牌失败: {}", e);
                    return Ok(error_response("登录失败", 500));
                }
            };

            // 生成刷新令牌
            let refresh_token = match app_state.auth_service.generate_refresh_token(&user).await {
                Ok(token) => token,
                Err(e) => {
                    error!("生成刷新令牌失败: {}", e);
                    return Ok(error_response("登录失败", 500));
                }
            };

            // 更新最后登录时间
            if let Err(e) = app_state.auth_service.update_last_login(&user.id).await {
                warn!("更新最后登录时间失败: {}", e);
            }

            // 获取用户权限
            let permissions = match app_state.permission_service.get_user_permissions(&user.roles).await {
                Ok(perms) => perms,
                Err(e) => {
                    warn!("获取用户权限失败: {}", e);
                    Vec::new()
                }
            };

            let response = LoginResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: app_state.config.jwt.access_token_expiry,
                user: UserInfo {
                    id: user.id,
                    username: user.username.clone(),
                    email: user.email.clone(),
                    display_name: user.username.clone(),
                    roles: user.roles.clone(),
                    permissions,
                    created_at: user.created_at,
                    last_login_at: user.last_login,
                },
            };

            info!("用户 {} 登录成功", request.username);
            Ok(success_response(response))
        }
        Err(e) => {
            warn!("用户 {} 登录失败: {}", request.username, e);
            Ok(error_response("用户名或密码错误", 401))
        }
    }
}

/// 用户登出
#[instrument(skip(app_state, req))]
pub async fn logout(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    // 从请求头获取令牌
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // 将令牌加入黑名单
                if let Err(e) = app_state.auth_service.revoke_token(token).await {
                    warn!("撤销令牌失败: {}", e);
                }
            }
        }
    }

    info!("用户登出");
    Ok(success_response(serde_json::json!({
        "message": "登出成功"
    })))
}

/// 刷新访问令牌
#[instrument(skip(app_state, request))]
pub async fn refresh_token(
    app_state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("刷新访问令牌");

    match app_state.auth_service.refresh_access_token(&request.refresh_token).await {
        Ok(auth_response) => {
            let response = serde_json::json!({
                "access_token": auth_response.token,
                "refresh_token": auth_response.token, // For simplicity, use same token
                "token_type": "Bearer",
                "expires_in": app_state.config.jwt.access_token_expiry
            });

            Ok(success_response(response))
        }
        Err(e) => {
            warn!("刷新令牌失败: {}", e);
            Ok(error_response("刷新令牌无效或已过期", 401))
        }
    }
}

/// 获取用户资料
#[instrument(skip(app_state, req))]
pub async fn get_profile(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    // 从请求中获取用户ID（由认证中间件设置）
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return Ok(error_response("未找到用户信息", 401));
        }
    };

    info!("获取用户资料: {}", user_id);

    match app_state.auth_service.get_user_by_id(&user_id).await {
        Ok(user) => {
            // 获取用户权限
            let permissions = match app_state.permission_service.get_user_permissions(&user.roles).await {
                Ok(perms) => perms,
                Err(e) => {
                    warn!("获取用户权限失败: {}", e);
                    Vec::new()
                }
            };

            let user_info = UserInfo {
                id: user.id,
                username: user.username.clone(),
                email: user.email.clone(),
                display_name: user.username.clone(),
                roles: user.roles.clone(),
                permissions,
                created_at: user.created_at,
                last_login_at: user.last_login,
            };

            Ok(success_response(user_info))
        }
        Err(e) => {
            error!("获取用户资料失败: {}", e);
            Ok(error_response("获取用户资料失败", 500))
        }
    }
}

/// 更新用户资料请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    /// 显示名称
    #[validate(length(min = 1, max = 100, message = "显示名称长度必须在1-100字符之间"))]
    pub display_name: Option<String>,
    /// 邮箱
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
}

/// 更新用户资料
#[instrument(skip(app_state, req, request))]
pub async fn update_profile(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    request: web::Json<UpdateProfileRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            return Ok(error_response("未找到用户信息", 401));
        }
    };

    info!("更新用户资料: {}", user_id);

    match app_state.auth_service.update_user_profile(&user_id, serde_json::to_value(request.into_inner()).unwrap()).await {
        Ok(updated_user) => {
            let user_info = UserInfo {
                id: updated_user.id,
                username: updated_user.username.clone(),
                email: updated_user.email.clone(),
                display_name: updated_user.username.clone(),
                roles: updated_user.roles.clone(),
                permissions: Vec::new(), // 权限不在此处更新
                created_at: updated_user.created_at,
                last_login_at: updated_user.last_login,
            };

            Ok(success_response(user_info))
        }
        Err(e) => {
            error!("更新用户资料失败: {}", e);
            Ok(error_response("更新用户资料失败", 500))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            remember_me: Some(true),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = LoginRequest {
            username: "ab".to_string(), // 太短
            password: "123".to_string(), // 太短
            remember_me: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_refresh_token_request_validation() {
        let valid_request = RefreshTokenRequest {
            refresh_token: "valid_token".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = RefreshTokenRequest {
            refresh_token: "".to_string(), // 空字符串
        };
        assert!(invalid_request.validate().is_err());
    }
}
