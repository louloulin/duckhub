//! Web API集成测试

use actix_web::{test, web, App};
use duckhub_web_api::{WebApiConfig, AppState, configure_routes};
use duckhub_common::prelude::*;
use serde_json::json;
use std::sync::Arc;
use prometheus::Registry;

/// 创建测试应用状态
async fn create_test_app_state() -> AppState {
    let config = WebApiConfig::default();
    let registry = Arc::new(Registry::new());
    
    // 创建模拟的服务实例
    let engine = Arc::new(
        duckhub_database::DuckDBEngine::new(":memory:").await.unwrap()
    );
    let cache = Arc::new(
        duckhub_cache::CacheManager::new().await.unwrap()
    );
    let auth_service = Arc::new(
        duckhub_security::AuthService::new(Arc::clone(&engine)).await.unwrap()
    );
    let permission_service = Arc::new(
        duckhub_security::PermissionService::new(Arc::clone(&engine)).await.unwrap()
    );
    let ingestion_service = Arc::new(
        duckhub_data_ingestion::DataIngestionService::new(
            Arc::clone(&engine),
            Arc::clone(&cache),
            Arc::clone(&registry),
        ).await.unwrap()
    );
    let analytics_service = Arc::new(
        duckhub_query_analytics::QueryAnalyticsService::new(
            Arc::clone(&engine),
            Arc::clone(&cache),
            Arc::clone(&registry),
        ).await.unwrap()
    );
    let ai_service = Arc::new(
        duckhub_ai_agent::AIAgentService::new().await.unwrap()
    );
    let monitoring_service = Arc::new(
        duckhub_monitoring::MonitoringService::new(Arc::clone(&registry)).await.unwrap()
    );

    AppState {
        engine,
        cache,
        auth_service,
        permission_service,
        ingestion_service,
        analytics_service,
        ai_service,
        monitoring_service,
        registry,
        config,
    }
}

#[actix_web::test]
async fn test_health_check() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}

#[actix_web::test]
async fn test_query_execution() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let query_request = json!({
        "sql": "SELECT 1 as test_column",
        "use_cache": false,
        "timeout": 30
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/query/execute")
        .set_json(&query_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于没有认证，应该返回401
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_login_endpoint() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let login_request = json!({
        "username": "testuser",
        "password": "testpassword",
        "remember_me": false
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于用户不存在，应该返回401
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_dashboard_data_endpoint() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/monitoring/dashboard")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于没有认证，应该返回401
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_data_sources_list() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/ingestion/sources")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于没有认证，应该返回401
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_ai_chat_endpoint() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let chat_request = json!({
        "message": "帮我查询用户数据",
        "session_id": "test-session"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/ai/chat")
        .set_json(&chat_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于没有认证，应该返回401
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_ducklake_databases_list() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/ducklake/databases")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 由于没有认证，应该返回401
    assert_eq!(resp.status(), 401);
}

/// 性能测试：并发请求处理
#[actix_web::test]
async fn test_concurrent_health_checks() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    let mut handles = Vec::new();
    
    for _ in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let req = test::TestRequest::get().uri("/health").to_request();
            let resp = test::call_service(&app_clone, req).await;
            resp.status().is_success()
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    
    // 所有请求都应该成功
    for result in results {
        assert!(result.unwrap());
    }
}

/// 测试请求验证
#[actix_web::test]
async fn test_request_validation() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .configure(configure_routes)
    ).await;

    // 测试无效的查询请求
    let invalid_query = json!({
        "sql": "", // 空SQL
        "timeout": 0 // 无效超时
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/query/execute")
        .set_json(&invalid_query)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 应该返回401（认证失败）或400（验证失败）
    assert!(resp.status().is_client_error());
}

#[cfg(test)]
mod load_tests {
    use super::*;
    use std::time::Instant;

    #[actix_web::test]
    async fn test_response_time() {
        let app_state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .configure(configure_routes)
        ).await;

        let start = Instant::now();
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        let duration = start.elapsed();

        assert!(resp.status().is_success());
        // 健康检查应该在100ms内完成
        assert!(duration.as_millis() < 100);
    }
}
