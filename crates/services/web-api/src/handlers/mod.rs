//! API处理器模块

pub mod auth;
pub mod query;
pub mod ingestion;
pub mod ai;
pub mod monitoring;
pub mod ducklake;
pub mod health;
pub mod data;
pub mod dashboard;
pub mod analytics;
pub mod system;
pub mod files;
pub mod realtime;

pub use auth::*;
pub use query::*;
pub use ingestion::*;
pub use ai::*;
pub use monitoring::*;
pub use ducklake::*;
pub use health::*;
pub use data::*;
pub use dashboard::*;
pub use analytics::*;
pub use system::*;
pub use files::*;
pub use realtime::*;

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use crate::AppState;

/// 通用成功响应
pub fn success_response<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "data": data,
        "message": "操作成功"
    }))
}

/// 通用错误响应
pub fn error_response(message: &str, code: u16) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::from_u16(code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR))
        .json(json!({
            "success": false,
            "error": message,
            "code": code
        }))
}

/// 分页响应
pub fn paginated_response<T: serde::Serialize>(
    data: Vec<T>,
    page: u32,
    page_size: u32,
    total: u64,
) -> HttpResponse {
    let total_pages = (total as f64 / page_size as f64).ceil() as u32;
    
    HttpResponse::Ok().json(json!({
        "success": true,
        "data": data,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total,
            "total_pages": total_pages,
            "has_next": page < total_pages,
            "has_prev": page > 1
        }
    }))
}

/// 验证分页参数
pub fn validate_pagination(page: Option<u32>, page_size: Option<u32>) -> (u32, u32) {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).clamp(1, 100);
    (page, page_size)
}

/// 计算偏移量
pub fn calculate_offset(page: u32, page_size: u32) -> u32 {
    (page - 1) * page_size
}
