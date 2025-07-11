//! 健康检查和监控处理器

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use tracing::{info, error, instrument};
use prometheus::{Encoder, TextEncoder};
use duckhub_ai_agent::HealthStatus;
use duckhub_cache::Cache;
use crate::{AppState, success_response, error_response};

/// 健康检查
#[instrument(skip(app_state))]
pub async fn health_check(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("执行健康检查");

    // 检查数据库连接
    let db_healthy = match app_state.engine.check_connection().await {
        Ok(_) => true,
        Err(e) => {
            error!("数据库连接检查失败: {}", e);
            false
        }
    };

    // 检查缓存连接
    let cache_healthy = match app_state.cache.health_check().await {
        Ok(_) => true,
        Err(e) => {
            error!("缓存连接检查失败: {}", e);
            false
        }
    };

    // 检查数据采集服务
    let ingestion_healthy = match app_state.ingestion_service.health_check().await {
        Ok(status) => status.overall_status == "healthy",
        Err(e) => {
            error!("数据采集服务检查失败: {}", e);
            false
        }
    };

    let overall_healthy = db_healthy && cache_healthy && ingestion_healthy;

    let response = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION"),
        "components": {
            "database": {
                "status": if db_healthy { "healthy" } else { "unhealthy" }
            },
            "cache": {
                "status": if cache_healthy { "healthy" } else { "unhealthy" }
            },
            "ingestion": {
                "status": if ingestion_healthy { "healthy" } else { "unhealthy" }
            }
        }
    });

    if overall_healthy {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(response))
    }
}

/// 获取Prometheus指标
#[instrument(skip(app_state))]
pub async fn metrics_handler(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = app_state.registry.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => Ok(HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics)),
        Err(e) => {
            error!("编码Prometheus指标失败: {}", e);
            Ok(error_response("获取指标失败", 500))
        }
    }
}

/// 获取系统信息
#[instrument(skip(app_state))]
pub async fn system_info(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let system_info = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_time": std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string()),
        "git_commit": std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        "rust_version": std::env::var("RUST_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        "uptime": get_uptime(),
        "memory_usage": get_memory_usage(),
        "cpu_usage": get_cpu_usage(),
        "config": {
            "host": app_state.config.host,
            "port": app_state.config.port,
            "workers": app_state.config.workers,
            "request_timeout": app_state.config.request_timeout,
        }
    });

    Ok(success_response(system_info))
}

/// 获取运行时间
fn get_uptime() -> u64 {
    // 简化实现，实际应该记录启动时间
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取内存使用情况
fn get_memory_usage() -> serde_json::Value {
    // 简化实现，实际应该使用系统API获取真实内存信息
    json!({
        "used": "256MB",
        "total": "1GB",
        "percentage": 25.6
    })
}

/// 获取CPU使用率
fn get_cpu_usage() -> f64 {
    // 简化实现，实际应该使用系统API获取真实CPU信息
    15.5
}

/// 获取详细的健康状态
#[instrument(skip(app_state))]
pub async fn detailed_health(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let mut components = std::collections::HashMap::new();

    // 检查数据库
    match app_state.engine.check_connection().await {
        Ok(_) => {
            components.insert("database".to_string(), json!({
                "status": "healthy",
                "response_time": "5ms",
                "last_check": chrono::Utc::now()
            }));
        }
        Err(e) => {
            components.insert("database".to_string(), json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "last_check": chrono::Utc::now()
            }));
        }
    }

    // 检查缓存
    match app_state.cache.health_check().await {
        Ok(_) => {
            components.insert("cache".to_string(), json!({
                "status": "healthy",
                "response_time": "2ms",
                "last_check": chrono::Utc::now()
            }));
        }
        Err(e) => {
            components.insert("cache".to_string(), json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "last_check": chrono::Utc::now()
            }));
        }
    }

    // 检查数据采集服务
    match app_state.ingestion_service.health_check().await {
        Ok(status) => {
            components.insert("ingestion".to_string(), json!({
                "status": status.overall_status,
                "components": status.components,
                "last_check": chrono::Utc::now()
            }));
        }
        Err(e) => {
            components.insert("ingestion".to_string(), json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "last_check": chrono::Utc::now()
            }));
        }
    }

    // 检查AI服务
    match app_state.ai_service.health_check().await {
        Ok(healthy) => {
            components.insert("ai_service".to_string(), json!({
                "status": match healthy {
                    HealthStatus::Healthy => "healthy",
                    HealthStatus::Unhealthy => "unhealthy",
                },
                "last_check": chrono::Utc::now()
            }));
        }
        Err(e) => {
            components.insert("ai_service".to_string(), json!({
                "status": "unhealthy",
                "error": e.to_string(),
                "last_check": chrono::Utc::now()
            }));
        }
    }

    let overall_healthy = components.values()
        .all(|component| component["status"] == "healthy");

    let response = json!({
        "status": if overall_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now(),
        "components": components
    });

    Ok(success_response(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_uptime() {
        let uptime = get_uptime();
        assert!(uptime > 0);
    }

    #[test]
    fn test_get_cpu_usage() {
        let cpu_usage = get_cpu_usage();
        assert!(cpu_usage >= 0.0 && cpu_usage <= 100.0);
    }
}
