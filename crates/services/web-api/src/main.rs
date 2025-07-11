//! DuckHub Web API服务主程序

use duckhub_web_api::{WebApiService, WebApiConfig};
use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_common::{DatabaseConfig, PoolConfig};
use duckhub_ai_agent::AIAgentConfig;
use duckhub_cache::CacheManager;
use duckhub_security_service::{AuthService, PermissionService};
use duckhub_data_ingestion::{DataIngestionService, IngestionConfig};
use duckhub_query_analytics::{QueryAnalyticsService, QueryAnalyticsConfig};
use duckhub_ai_agent::AIAgentService;
use duckhub_monitoring::MonitoringService;

use std::sync::Arc;
use tracing::{info, error};
use prometheus::Registry;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("duckhub_web_api=info,info")
        .init();

    info!("启动DuckHub Web API服务");

    // 加载配置
    let config = WebApiConfig::from_env().unwrap_or_else(|_| {
        info!("使用默认配置");
        WebApiConfig::default()
    });

    // 验证配置
    if let Err(e) = config.validate() {
        error!("配置验证失败: {}", e);
        std::process::exit(1);
    }

    info!("配置加载完成: {}:{}", config.host, config.port);

    // 创建Prometheus注册表
    let registry = Arc::new(Registry::new());

    // 初始化数据库引擎
    let db_config = DatabaseConfig {
        duckdb_path: "duckhub.db".to_string(),
        memory_limit: Some("1GB".to_string()),
        threads: Some(4),
        max_memory: Some("1GB".to_string()),
        temp_directory: None,
        extensions: vec!["parquet".to_string(), "json".to_string()],
        pool: PoolConfig {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: 30,
            idle_timeout: 300,
            max_lifetime: 3600,
        },
    };

    let engine = Arc::new(
        DuckDBEngine::new(db_config).await
            .map_err(|e| DuckHubError::database(format!("初始化数据库失败: {}", e)))?
    );
    info!("数据库引擎初始化完成");

    // 初始化缓存管理器
    let cache = Arc::new(
        CacheManager::new_memory(std::time::Duration::from_secs(3600)) // 1小时TTL
    );
    info!("缓存管理器初始化完成");

    // 初始化认证服务
    let auth_service = Arc::new(
        AuthService::new(Arc::clone(&engine), "your-secret-key".to_string()).await
            .map_err(|e| DuckHubError::auth(format!("初始化认证服务失败: {}", e)))?
    );
    info!("认证服务初始化完成");

    // 初始化权限服务
    let permission_service = Arc::new(
        PermissionService::new(Arc::clone(&engine)).await
            .map_err(|e| DuckHubError::auth(format!("初始化权限服务失败: {}", e)))?
    );
    info!("权限服务初始化完成");

    // 初始化数据采集服务
    let ingestion_config = IngestionConfig {
        service_name: "duckhub-data-ingestion".to_string(),
        port: 8080,
        worker_threads: 4,
        batch_size: 1000,
        batch_timeout_secs: 30,
        max_retries: 3,
        retry_interval_ms: 1000,
        sources: std::collections::HashMap::new(),
        processors: std::collections::HashMap::new(),
        monitoring: duckhub_data_ingestion::config::MonitoringConfig::default(),
    };

    let ingestion_service = Arc::new(
        DataIngestionService::new(
            Arc::clone(&engine),
            ingestion_config,
            &*registry,
        ).await
            .map_err(|e| DuckHubError::database(format!("初始化数据采集服务失败: {}", e)))?
    );
    info!("数据采集服务初始化完成");

    // 初始化查询分析服务
    let analytics_config = QueryAnalyticsConfig {
        cache_size: 1000,
        query_timeout: 30,
        parallel_threads: 4,
        stats_collection_interval: 60,
        enable_optimization: true,
        enable_plan_cache: true,
    };

    let analytics_service = Arc::new(
        QueryAnalyticsService::new(
            Arc::clone(&engine),
            analytics_config,
            &*registry,
        ).await
            .map_err(|e| DuckHubError::database(format!("初始化查询分析服务失败: {}", e)))?
    );
    info!("查询分析服务初始化完成");

    // 初始化AI Agent服务
    let ai_config = AIAgentConfig {
        enable_nlp: true,
        enable_recommendations: true,
        enable_automation: true,
        enable_chat: true,
        openai_api_key: Some("your-api-key".to_string()),
        model_name: "deepseek-chat".to_string(),
        max_tokens: 4000,
        temperature: 0.7,
        max_conversation_history: 100,
    };

    let ai_service = Arc::new(
        AIAgentService::new(
            Arc::clone(&engine),
            Arc::clone(&analytics_service),
            ai_config,
            &*registry,
        ).await
            .map_err(|e| DuckHubError::database(format!("初始化AI Agent服务失败: {}", e)))?
    );
    info!("AI Agent服务初始化完成");

    // 初始化监控服务
    let monitoring_config = duckhub_monitoring::MonitoringConfig {
        metrics_interval: 10,
        health_check_interval: 30,
        alert_check_interval: 60,
        retention_days: 7,
        enable_system_monitoring: true,
        enable_network_monitoring: true,
        alerts: duckhub_monitoring::AlertConfig::default(),
    };

    let monitoring_service = Arc::new(
        MonitoringService::new(
            Arc::clone(&engine),
            monitoring_config,
            &*registry,
        ).await
            .map_err(|e| DuckHubError::database(format!("初始化监控服务失败: {}", e)))?
    );
    info!("监控服务初始化完成");

    // 创建Web API服务
    let web_api_service = WebApiService::new(
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
    ).await?;

    info!("所有服务初始化完成，启动Web API服务器");

    // 启动服务
    if let Err(e) = web_api_service.start().await {
        error!("启动Web API服务失败: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// 优雅关闭处理
async fn graceful_shutdown() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("安装Ctrl+C处理器失败");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("安装SIGTERM处理器失败")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("收到Ctrl+C信号，开始优雅关闭");
        },
        _ = terminate => {
            info!("收到SIGTERM信号，开始优雅关闭");
        },
    }
}

/// 健康检查
async fn health_check_loop() {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        // 这里可以添加定期健康检查逻辑
        tracing::debug!("执行健康检查");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_validation() {
        let config = WebApiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = WebApiConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.workers > 0);
    }
}
