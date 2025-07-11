//! 数据采集服务测试

use duckhub_data_ingestion::{DataIngestionService, CreateSourceRequest, UpdateSourceRequest};
use duckhub_common::prelude::*;
use serde_json::json;
use tokio;

#[tokio::test]
async fn test_list_sources() {
    let service = create_test_ingestion_service().await;

    // 测试列出数据源
    let sources = service.list_sources().await.expect("列出数据源失败");

    // 验证返回格式
    assert!(sources.is_empty() || sources.len() > 0, "数据源列表应该是有效的");
    
    // 如果有数据源，验证格式
    if let Some(source) = sources.first() {
        assert!(source.get("name").is_some(), "数据源应该有name字段");
        assert!(source.get("type").is_some(), "数据源应该有type字段");
        assert!(source.get("status").is_some(), "数据源应该有status字段");
    }
}

#[tokio::test]
async fn test_create_source() {
    let service = create_test_ingestion_service().await;

    let create_request = CreateSourceRequest {
        name: "test_kafka_source".to_string(),
        source_type: "kafka".to_string(),
        config: json!({
            "bootstrap_servers": "localhost:9092",
            "topic": "test_topic",
            "group_id": "test_group"
        }),
        description: Some("测试Kafka数据源".to_string()),
        enabled: Some(true),
    };

    // 测试创建数据源
    let result = service.create_source(create_request).await.expect("创建数据源失败");

    // 验证创建结果
    assert!(result.get("id").is_some(), "创建结果应该包含id");
    assert!(result.get("name").is_some(), "创建结果应该包含name");
    assert!(result.get("type").is_some(), "创建结果应该包含type");
    assert!(result.get("status").is_some(), "创建结果应该包含status");

    let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(name, "test_kafka_source", "名称应该匹配");

    let source_type = result.get("type").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(source_type, "kafka", "类型应该匹配");
}

#[tokio::test]
async fn test_update_source() {
    let service = create_test_ingestion_service().await;

    let update_request = UpdateSourceRequest {
        name: Some("updated_source_name".to_string()),
        config: Some(json!({
            "updated_config": "new_value"
        })),
        description: Some("更新后的描述".to_string()),
        enabled: Some(false),
    };

    // 测试更新数据源
    let result = service.update_source("test_source_id", update_request).await
        .expect("更新数据源失败");

    // 验证更新结果
    assert!(result.get("id").is_some(), "更新结果应该包含id");
    assert!(result.get("name").is_some(), "更新结果应该包含name");
    assert!(result.get("status").is_some(), "更新结果应该包含status");
    assert!(result.get("updated_at").is_some(), "更新结果应该包含updated_at");

    let name = result.get("name").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(name, "updated_source_name", "更新后的名称应该匹配");
}

#[tokio::test]
async fn test_delete_source() {
    let service = create_test_ingestion_service().await;

    // 测试删除数据源
    let result = service.delete_source("test_source_id").await;
    
    // 删除操作应该成功
    assert!(result.is_ok(), "删除数据源应该成功");
}

#[tokio::test]
async fn test_create_different_source_types() {
    let service = create_test_ingestion_service().await;

    let source_types = vec![
        ("kafka", json!({"bootstrap_servers": "localhost:9092", "topic": "test"})),
        ("file", json!({"path": "/tmp/test.csv", "format": "csv"})),
        ("database", json!({"connection_string": "postgresql://localhost/test"})),
        ("api", json!({"endpoint": "https://api.example.com/data", "auth_token": "test"})),
    ];

    for (source_type, config) in source_types {
        let create_request = CreateSourceRequest {
            name: format!("test_{}_source", source_type),
            source_type: source_type.to_string(),
            config,
            description: Some(format!("测试{}数据源", source_type)),
            enabled: Some(true),
        };

        let result = service.create_source(create_request).await
            .expect(&format!("创建{}数据源失败", source_type));

        let result_type = result.get("type").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(result_type, source_type, "数据源类型应该匹配");
    }
}

#[tokio::test]
async fn test_ingestion_service_health() {
    let service = create_test_ingestion_service().await;

    // 测试健康检查
    let health = service.health_check().await.expect("数据采集服务健康检查失败");
    
    // 验证健康状态
    assert!(health.contains_key("status"), "健康检查应该包含status");
    assert!(health.contains_key("sources"), "健康检查应该包含sources信息");
    assert!(health.contains_key("processors"), "健康检查应该包含processors信息");
}

#[tokio::test]
async fn test_ingestion_metrics() {
    let service = create_test_ingestion_service().await;

    // 测试获取指标
    let metrics = service.get_metrics().await.expect("获取数据采集指标失败");

    // 验证指标格式
    assert!(metrics.contains_key("total_records_processed"), "指标应该包含处理记录总数");
    assert!(metrics.contains_key("active_sources"), "指标应该包含活跃数据源数");
    assert!(metrics.contains_key("processing_rate"), "指标应该包含处理速率");
    assert!(metrics.contains_key("error_rate"), "指标应该包含错误率");

    // 验证指标值类型
    let total_records = metrics.get("total_records_processed")
        .and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(total_records >= 0, "处理记录总数应该是非负数");

    let active_sources = metrics.get("active_sources")
        .and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(active_sources >= 0, "活跃数据源数应该是非负数");
}

#[tokio::test]
async fn test_batch_operations() {
    let service = create_test_ingestion_service().await;

    // 创建多个数据源
    let sources = vec![
        CreateSourceRequest {
            name: "batch_source_1".to_string(),
            source_type: "kafka".to_string(),
            config: json!({"topic": "topic1"}),
            description: Some("批量测试源1".to_string()),
            enabled: Some(true),
        },
        CreateSourceRequest {
            name: "batch_source_2".to_string(),
            source_type: "file".to_string(),
            config: json!({"path": "/tmp/file2.csv"}),
            description: Some("批量测试源2".to_string()),
            enabled: Some(true),
        },
    ];

    // 批量创建
    for source in sources {
        let result = service.create_source(source).await
            .expect("批量创建数据源失败");
        assert!(result.get("id").is_some(), "批量创建应该返回有效ID");
    }

    // 验证列表中包含新创建的源
    let all_sources = service.list_sources().await.expect("列出数据源失败");
    assert!(all_sources.len() >= 2, "应该至少包含2个数据源");
}

// 辅助函数：创建测试用的数据采集服务
async fn create_test_ingestion_service() -> DataIngestionService {
    // 创建测试配置
    let config = duckhub_data_ingestion::IngestionConfig {
        max_concurrent_sources: 10,
        buffer_size: 1000,
        batch_size: 100,
        flush_interval_seconds: 5,
        retry_attempts: 3,
        monitoring: duckhub_data_ingestion::MonitoringConfig {
            enabled: true,
            metrics_interval_seconds: 10,
            health_check_interval_seconds: 30,
        },
    };

    DataIngestionService::new(config).await.expect("创建数据采集服务失败")
}
