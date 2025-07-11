//! 数据采集处理器

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error, instrument};
use validator::Validate;
use crate::{AppState, success_response, error_response};

/// 数据源创建请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDataSourceRequest {
    /// 数据源名称
    #[validate(length(min = 1, max = 100, message = "数据源名称长度必须在1-100字符之间"))]
    pub name: String,
    /// 数据源类型
    pub source_type: String,
    /// 连接配置
    pub config: serde_json::Value,
    /// 描述
    pub description: Option<String>,
    /// 是否启用
    pub enabled: Option<bool>,
}

/// 数据源响应
#[derive(Debug, Serialize)]
pub struct DataSourceResponse {
    pub id: Uuid,
    pub name: String,
    pub source_type: String,
    pub status: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 获取数据采集状态
#[instrument(skip(app_state))]
pub async fn get_ingestion_status(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取数据采集状态");

    match app_state.ingestion_service.get_status().await {
        Ok(status) => Ok(success_response(status)),
        Err(e) => {
            error!("获取数据采集状态失败: {}", e);
            Ok(error_response("获取数据采集状态失败", 500))
        }
    }
}

/// 列出数据源
#[instrument(skip(app_state))]
pub async fn list_data_sources(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("列出数据源");

    match app_state.ingestion_service.list_sources().await {
        Ok(sources) => {
            let response: Vec<DataSourceResponse> = sources.into_iter().map(|source| {
                DataSourceResponse {
                    id: source.id,
                    name: source.name,
                    source_type: source.source_type,
                    status: source.status,
                    description: source.description,
                    enabled: source.enabled,
                    created_at: source.created_at,
                    updated_at: source.updated_at,
                }
            }).collect();

            Ok(success_response(response))
        }
        Err(e) => {
            error!("列出数据源失败: {}", e);
            Ok(error_response("列出数据源失败", 500))
        }
    }
}

/// 创建数据源
#[instrument(skip(app_state, request))]
pub async fn create_data_source(
    app_state: web::Data<AppState>,
    request: web::Json<CreateDataSourceRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("创建数据源: {}", request.name);

    let create_request = duckhub_data_ingestion::CreateSourceRequest {
        name: request.name.clone(),
        source_type: request.source_type.clone(),
        config: request.config.clone(),
        description: request.description.clone(),
        enabled: request.enabled.unwrap_or(true),
    };

    match app_state.ingestion_service.create_source(create_request).await {
        Ok(source) => {
            let response = DataSourceResponse {
                id: source.id,
                name: source.name,
                source_type: source.source_type,
                status: source.status,
                description: source.description,
                enabled: source.enabled,
                created_at: source.created_at,
                updated_at: source.updated_at,
            };

            Ok(success_response(response))
        }
        Err(e) => {
            error!("创建数据源失败: {}", e);
            Ok(error_response(&format!("创建数据源失败: {}", e), 500))
        }
    }
}

/// 更新数据源
#[instrument(skip(app_state, request))]
pub async fn update_data_source(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<CreateDataSourceRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    let source_id = path.into_inner();
    info!("更新数据源: {}", source_id);

    let update_request = duckhub_data_ingestion::UpdateSourceRequest {
        name: Some(request.name.clone()),
        config: Some(request.config.clone()),
        description: request.description.clone(),
        enabled: request.enabled,
    };

    match app_state.ingestion_service.update_source(&source_id, update_request).await {
        Ok(source) => {
            let response = DataSourceResponse {
                id: source.id,
                name: source.name,
                source_type: source.source_type,
                status: source.status,
                description: source.description,
                enabled: source.enabled,
                created_at: source.created_at,
                updated_at: source.updated_at,
            };

            Ok(success_response(response))
        }
        Err(e) => {
            error!("更新数据源失败: {}", e);
            Ok(error_response(&format!("更新数据源失败: {}", e), 500))
        }
    }
}

/// 删除数据源
#[instrument(skip(app_state))]
pub async fn delete_data_source(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> ActixResult<HttpResponse> {
    let source_id = path.into_inner();
    info!("删除数据源: {}", source_id);

    match app_state.ingestion_service.delete_source(&source_id).await {
        Ok(_) => Ok(success_response(serde_json::json!({
            "message": "数据源删除成功"
        }))),
        Err(e) => {
            error!("删除数据源失败: {}", e);
            Ok(error_response(&format!("删除数据源失败: {}", e), 500))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_data_source_request_validation() {
        let valid_request = CreateDataSourceRequest {
            name: "test_source".to_string(),
            source_type: "kafka".to_string(),
            config: serde_json::json!({"broker": "localhost:9092"}),
            description: Some("Test data source".to_string()),
            enabled: Some(true),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateDataSourceRequest {
            name: "".to_string(), // 空名称
            source_type: "kafka".to_string(),
            config: serde_json::json!({}),
            description: None,
            enabled: None,
        };
        assert!(invalid_request.validate().is_err());
    }
}
