//! DuckHub Web API服务
//!
//! 提供RESTful API接口，连接前端和后端服务：
//! - 数据查询和分析API
//! - 数据采集管理API
//! - AI Agent交互API
//! - 系统监控和管理API
//! - 用户认证和权限API

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_cache::CacheManager;
use duckhub_security_service::{AuthService, PermissionService};
use duckhub_data_ingestion::DataIngestionService;
use duckhub_query_analytics::QueryAnalyticsService;
use duckhub_ai_agent::AIAgentService;
use duckhub_monitoring::MonitoringService;

use actix_web::{web, App, HttpServer, middleware::Logger, Result as ActixResult};
use actix_cors::Cors;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, instrument};
use prometheus::Registry;

pub mod handlers;
pub mod middleware;
pub mod config;
pub mod models;
pub mod utils;

pub use handlers::*;
pub use middleware::*;
pub use config::*;
pub use models::*;
pub use utils::*;

/// Web API服务主结构
pub struct WebApiService {
    /// 服务配置
    config: WebApiConfig,
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 缓存管理器
    cache: Arc<CacheManager>,
    /// 认证服务
    auth_service: Arc<AuthService>,
    /// 权限服务
    permission_service: Arc<PermissionService>,
    /// 数据采集服务
    ingestion_service: Arc<DataIngestionService>,
    /// 查询分析服务
    analytics_service: Arc<QueryAnalyticsService>,
    /// AI Agent服务
    ai_service: Arc<AIAgentService>,
    /// 监控服务
    monitoring_service: Arc<MonitoringService>,
    /// Prometheus注册表
    registry: Arc<Registry>,
}

impl WebApiService {
    /// 创建新的Web API服务实例
    #[instrument(skip_all)]
    pub async fn new(
        config: WebApiConfig,
        engine: Arc<DuckDBEngine>,
        cache: Arc<CacheManager>,
        auth_service: Arc<AuthService>,
        permission_service: Arc<PermissionService>,
        ingestion_service: Arc<DataIngestionService>,
        analytics_service: Arc<QueryAnalyticsService>,
        ai_service: Arc<AIAgentService>,
        monitoring_service: Arc<MonitoringService>,
        registry: Arc<Registry>,
    ) -> Result<Self> {
        info!("创建Web API服务");

        Ok(Self {
            config,
            engine,
            cache,
            auth_service,
            permission_service,
            ingestion_service,
            analytics_service,
            ai_service,
            monitoring_service,
            registry,
        })
    }

    /// 启动Web API服务
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let bind_address = format!("{}:{}", self.config.host, self.config.port);
        info!("启动Web API服务，监听地址: {}", bind_address);

        // 创建应用状态
        let app_state = AppState {
            engine: Arc::clone(&self.engine),
            cache: Arc::clone(&self.cache),
            auth_service: Arc::clone(&self.auth_service),
            permission_service: Arc::clone(&self.permission_service),
            ingestion_service: Arc::clone(&self.ingestion_service),
            analytics_service: Arc::clone(&self.analytics_service),
            ai_service: Arc::clone(&self.ai_service),
            monitoring_service: Arc::clone(&self.monitoring_service),
            registry: Arc::clone(&self.registry),
            config: self.config.clone(),
        };

        // 启动HTTP服务器
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .wrap(Logger::default())
                .wrap(
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_method()
                        .allow_any_header()
                        .max_age(3600)
                )
                .wrap(AuthMiddleware::new())
                .wrap(MetricsMiddleware::new())
                .configure(configure_routes)
        })
        .bind(&bind_address)
        .map_err(|e| DuckHubError::network(format!("绑定地址失败: {}", e)))?
        .run()
        .await
        .map_err(|e| DuckHubError::network(format!("启动服务器失败: {}", e)))?;

        Ok(())
    }
}

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<DuckDBEngine>,
    pub cache: Arc<CacheManager>,
    pub auth_service: Arc<AuthService>,
    pub permission_service: Arc<PermissionService>,
    pub ingestion_service: Arc<DataIngestionService>,
    pub analytics_service: Arc<QueryAnalyticsService>,
    pub ai_service: Arc<AIAgentService>,
    pub monitoring_service: Arc<MonitoringService>,
    pub registry: Arc<Registry>,
    pub config: WebApiConfig,
}

/// 配置路由
fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 健康检查
        .route("/health", web::get().to(health_check))
        .route("/metrics", web::get().to(metrics_handler))
        
        // 认证相关
        .service(
            web::scope("/api/v1/auth")
                .route("/login", web::post().to(login))
                .route("/logout", web::post().to(logout))
                .route("/refresh", web::post().to(refresh_token))
                .route("/profile", web::get().to(get_profile))
        )
        
        // 数据查询相关
        .service(
            web::scope("/api/v1/query")
                .route("/execute", web::post().to(execute_query))
                .route("/analyze", web::post().to(analyze_query))
                .route("/optimize", web::post().to(optimize_query))
                .route("/history", web::get().to(get_query_history))
        )
        
        // 数据采集相关
        .service(
            web::scope("/api/v1/ingestion")
                .route("/status", web::get().to(get_ingestion_status))
                .route("/sources", web::get().to(list_data_sources))
                .route("/sources", web::post().to(create_data_source))
                .route("/sources/{id}", web::put().to(update_data_source))
                .route("/sources/{id}", web::delete().to(delete_data_source))
        )
        
        // AI Agent相关
        .service(
            web::scope("/api/v1/ai")
                .route("/chat", web::post().to(ai_chat))
                .route("/analyze", web::post().to(ai_analyze))
                .route("/suggest", web::post().to(ai_suggest))
                .route("/session", web::post().to(create_ai_session))
                .route("/session/{session_id}/history", web::get().to(get_session_history))
                .route("/nlp-query", web::post().to(process_nlp_query))
        )
        
        // 监控相关
        .service(
            web::scope("/api/v1/monitoring")
                .route("/dashboard", web::get().to(get_dashboard_data))
                .route("/alerts", web::get().to(get_alerts))
                .route("/performance", web::get().to(get_performance_metrics))
        )
        
        // 数据探索相关
        .service(
            web::scope("/api/v1/data")
                .route("/tables", web::get().to(get_tables))
                .route("/tables/{name}/schema", web::get().to(get_table_schema))
                .route("/tables/{name}/data", web::get().to(get_table_data))
                .route("/tables/{name}/stats", web::get().to(get_table_stats))
                .route("/tables/{name}/preview", web::get().to(preview_table))
        )

        // 仪表板相关
        .service(
            web::scope("/api/v1/dashboard")
                .route("/metrics", web::get().to(get_dashboard_metrics))
                .route("/query-trends", web::get().to(get_query_trends))
                .route("/system-health", web::get().to(get_system_health_dashboard))
        )

        // DuckLake相关
        .service(
            web::scope("/api/v1/ducklake")
                .route("/databases", web::get().to(list_databases))
                .route("/databases/{name}/snapshots", web::get().to(list_snapshots))
                .route("/databases/{name}/time-travel", web::post().to(time_travel_query))
                .route("/databases/{name}/schema", web::get().to(get_schema))
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use prometheus::Registry;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_check))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
