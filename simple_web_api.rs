use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware::Logger};
use serde_json::json;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 启动DuckHub Web API服务器...");
    println!("📍 服务地址: http://localhost:8080");
    println!("🔧 支持的API端点:");
    println!("   - GET  /health");
    println!("   - GET  /api/ducklake/databases");
    println!("   - POST /api/query/execute");
    println!("   - GET  /api/monitoring/dashboard");
    println!("   - POST /api/ai/chat");
    println!("✅ 前端可以连接到此服务进行功能验证");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .route("/ducklake/databases", web::get().to(list_databases))
                    .route("/ducklake/databases/{database_name}/snapshots", web::get().to(list_snapshots))
                    .route("/ducklake/time-travel", web::post().to(time_travel_query))
                    .route("/query/execute", web::post().to(execute_query))
                    .route("/monitoring/dashboard", web::get().to(get_dashboard_data))
                    .route("/ai/chat", web::post().to(ai_chat))
                    .route("/settings/config", web::get().to(get_config))
                    .route("/settings/config", web::post().to(save_config))
                    .route("/ducklake/schema/{database_name}", web::get().to(get_schema))
                    .route("/ducklake/snapshots", web::post().to(create_snapshot))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": {
            "database": "healthy",
            "cache": "healthy",
            "analytics": "healthy"
        }
    })))
}

async fn list_databases() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": [
            {
                "id": "db1",
                "name": "financial_data",
                "status": "active",
                "size": "2.5GB",
                "created_at": "2024-01-01T00:00:00Z",
                "last_accessed": "2024-01-11T10:00:00Z"
            },
            {
                "id": "db2",
                "name": "user_analytics",
                "status": "active",
                "size": "1.8GB",
                "created_at": "2024-01-05T00:00:00Z",
                "last_accessed": "2024-01-11T09:30:00Z"
            }
        ]
    })))
}

async fn list_snapshots(path: web::Path<String>) -> Result<HttpResponse> {
    let _database_name = path.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": [
            {
                "id": "snap1",
                "version": 126,
                "created_at": "2024-01-10T16:20:15Z",
                "size": "1.2GB",
                "description": "Daily backup"
            },
            {
                "id": "snap2",
                "version": 125,
                "created_at": "2024-01-10T12:15:30Z",
                "size": "1.1GB",
                "description": "Pre-migration backup"
            }
        ]
    })))
}

async fn time_travel_query(body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let _request = body.into_inner();
    
    // 模拟查询执行
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "query_id": format!("tt_{}", chrono::Utc::now().timestamp()),
            "execution_time_ms": 150,
            "row_count": 1250,
            "results": [
                {
                    "id": 1,
                    "amount": 1500.00,
                    "user_id": "user123",
                    "timestamp": "2024-01-10T15:30:00Z"
                },
                {
                    "id": 2,
                    "amount": 2200.50,
                    "user_id": "user456",
                    "timestamp": "2024-01-10T16:45:00Z"
                }
            ]
        }
    })))
}

async fn execute_query(body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let request = body.into_inner();
    let _sql = request.get("sql").and_then(|v| v.as_str()).unwrap_or("");
    
    // 模拟查询执行
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "query_id": format!("q_{}", chrono::Utc::now().timestamp()),
            "execution_time_ms": 95,
            "row_count": 500,
            "columns": [
                {"name": "id", "data_type": "INTEGER", "nullable": false},
                {"name": "amount", "data_type": "DECIMAL", "nullable": false},
                {"name": "user_id", "data_type": "VARCHAR", "nullable": false}
            ],
            "data": [
                {"id": 1, "amount": 1500.00, "user_id": "user123"},
                {"id": 2, "amount": 2200.50, "user_id": "user456"}
            ]
        }
    })))
}

async fn get_dashboard_data() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "core_metrics": {
                "total_queries": 15420,
                "active_connections": 23,
                "avg_response_time": 95,
                "system_uptime": "7d 12h 30m"
            },
            "ducklake_metrics": {
                "active_databases": 3,
                "total_snapshots": 127,
                "time_travel_queries": 1250,
                "schema_evolutions": 15
            },
            "query_trends": [
                {"time": "10:00", "queries": 120, "avg_time": 85},
                {"time": "11:00", "queries": 150, "avg_time": 92},
                {"time": "12:00", "queries": 180, "avg_time": 78}
            ]
        }
    })))
}

async fn ai_chat(body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let request = body.into_inner();
    let message = request.get("message").and_then(|v| v.as_str()).unwrap_or("");
    
    // 简单的关键词响应
    let response = if message.contains("时间旅行") || message.contains("历史") {
        "我为您推荐一个时间旅行查询方案。建议查询版本126的数据，该版本包含完整的交易记录。"
    } else if message.contains("schema") || message.contains("表结构") {
        "我分析了您的Schema演进需求。建议为transactions表添加status字段，这是一个向后兼容的安全操作。"
    } else if message.contains("快照") {
        "基于您的数据增长模式，我建议创建一个新的快照。当前数据变化率较高，及时快照可以保护重要数据状态。"
    } else if message.contains("性能") || message.contains("优化") {
        "我检测到您的查询性能可以进一步优化。建议为user_id字段添加索引，预计可提升查询速度40%。"
    } else {
        "我理解您的需求。基于DuckLake的强大功能，我可以为您提供数据查询、版本管理和性能优化建议。"
    };
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "response": response,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "metadata": {
                "sql_query": "SELECT * FROM transactions WHERE amount > 1000",
                "execution_time": 125,
                "result_count": 1250
            }
        }
    })))
}

async fn get_config() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "snapshot_retention": {
                "enabled": true,
                "retention_days": 30,
                "max_snapshots": 100,
                "auto_cleanup": true
            },
            "performance": {
                "memory_limit": "2GB",
                "thread_count": 4,
                "cache_size": "512MB",
                "query_timeout": 30
            }
        }
    })))
}

async fn save_config(body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let _config = body.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "配置保存成功"
    })))
}

async fn get_schema(path: web::Path<String>) -> Result<HttpResponse> {
    let _database_name = path.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "tables": [
                {
                    "name": "transactions",
                    "columns": [
                        {"name": "id", "type": "INTEGER", "nullable": false},
                        {"name": "amount", "type": "DECIMAL", "nullable": false},
                        {"name": "user_id", "type": "VARCHAR", "nullable": false},
                        {"name": "created_at", "type": "TIMESTAMP", "nullable": false}
                    ]
                },
                {
                    "name": "users",
                    "columns": [
                        {"name": "id", "type": "VARCHAR", "nullable": false},
                        {"name": "name", "type": "VARCHAR", "nullable": false},
                        {"name": "email", "type": "VARCHAR", "nullable": true}
                    ]
                }
            ]
        }
    })))
}

async fn create_snapshot(body: web::Json<serde_json::Value>) -> Result<HttpResponse> {
    let request = body.into_inner();
    let description = request.get("description").and_then(|v| v.as_str()).unwrap_or("手动创建的快照");
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "data": {
            "id": format!("snap_{}", chrono::Utc::now().timestamp()),
            "version": 127,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "size": "1.3GB",
            "description": description
        }
    })))
}
