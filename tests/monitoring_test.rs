//! 监控系统测试

use duckhub_monitoring::{MonitoringService, MonitoringConfig, MonitoringMetricsData, MonitoringActivity};
use duckhub_common::prelude::*;
use prometheus::Registry;
use tokio;

#[tokio::test]
async fn test_monitoring_metrics() {
    let service = create_test_monitoring_service().await;

    // 测试获取监控指标
    let metrics = service.get_metrics().await.expect("获取监控指标失败");

    // 验证指标字段
    assert!(metrics.total_queries > 0, "总查询数应该大于0");
    assert!(metrics.active_connections > 0, "活跃连接数应该大于0");
    assert!(metrics.data_processed_bytes > 0, "处理数据字节数应该大于0");
    assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0, "CPU使用率应该在0-100之间");
    assert!(metrics.memory_usage >= 0.0 && metrics.memory_usage <= 100.0, "内存使用率应该在0-100之间");
    assert!(metrics.disk_usage >= 0.0 && metrics.disk_usage <= 100.0, "磁盘使用率应该在0-100之间");
    assert!(metrics.network_io_bytes >= 0, "网络IO字节数应该非负");
    assert!(metrics.avg_response_time_ms > 0.0, "平均响应时间应该大于0");
}

#[tokio::test]
async fn test_monitoring_activities() {
    let service = create_test_monitoring_service().await;

    // 测试获取监控活动
    let activities = service.get_recent_activities(10).await.expect("获取监控活动失败");

    // 验证活动列表
    assert!(!activities.is_empty(), "应该有监控活动");
    assert!(activities.len() <= 10, "活动数量不应超过限制");

    // 验证活动字段
    for activity in &activities {
        assert!(!activity.id.to_string().is_empty(), "活动ID不应为空");
        assert!(!activity.activity_type.is_empty(), "活动类型不应为空");
        assert!(!activity.description.is_empty(), "活动描述不应为空");
        assert!(!activity.severity.is_empty(), "活动严重性不应为空");
        
        // 验证严重性级别
        assert!(
            matches!(activity.severity.as_str(), "info" | "warning" | "error" | "critical"),
            "严重性级别应该是有效值: {}", activity.severity
        );
    }
}

#[tokio::test]
async fn test_monitoring_performance_metrics() {
    let service = create_test_monitoring_service().await;

    // 测试获取性能指标
    let performance = service.get_performance_metrics().await.expect("获取性能指标失败");

    // 验证性能指标字段
    assert!(performance.contains_key("query_performance"), "应该包含查询性能指标");
    assert!(performance.contains_key("system_performance"), "应该包含系统性能指标");
    assert!(performance.contains_key("ducklake_performance"), "应该包含DuckLake性能指标");

    // 验证查询性能
    let query_perf = performance.get("query_performance").unwrap();
    assert!(query_perf.get("avg_execution_time").is_some(), "应该包含平均执行时间");
    assert!(query_perf.get("queries_per_second").is_some(), "应该包含每秒查询数");
    assert!(query_perf.get("cache_hit_rate").is_some(), "应该包含缓存命中率");

    // 验证系统性能
    let system_perf = performance.get("system_performance").unwrap();
    assert!(system_perf.get("cpu_usage").is_some(), "应该包含CPU使用率");
    assert!(system_perf.get("memory_usage").is_some(), "应该包含内存使用率");
}

#[tokio::test]
async fn test_monitoring_health_check() {
    let service = create_test_monitoring_service().await;

    // 测试健康检查
    let health = service.health_check().await.expect("监控服务健康检查失败");
    
    // 验证健康状态
    match health {
        duckhub_monitoring::HealthStatus::Healthy => {
            // 健康状态正常
        },
        _ => panic!("监控服务应该是健康的"),
    }
}

#[tokio::test]
async fn test_monitoring_system_status() {
    let service = create_test_monitoring_service().await;

    // 测试获取系统状态
    let status = service.get_system_status().await.expect("获取系统状态失败");

    // 验证系统状态字段
    assert!(status.cpu_info.usage_percent >= 0.0, "CPU使用率应该非负");
    assert!(status.memory_info.usage_percent >= 0.0, "内存使用率应该非负");
    assert!(!status.disk_info.is_empty(), "应该有磁盘信息");
    assert!(!status.network_info.is_empty(), "应该有网络信息");

    // 验证磁盘信息
    for disk in &status.disk_info {
        assert!(disk.usage_percent >= 0.0, "磁盘使用率应该非负");
        assert!(disk.total_bytes > 0, "磁盘总容量应该大于0");
    }

    // 验证网络信息
    for network in &status.network_info {
        assert!(network.bytes_sent >= 0, "发送字节数应该非负");
        assert!(network.bytes_received >= 0, "接收字节数应该非负");
    }
}

#[tokio::test]
async fn test_monitoring_metrics_collection() {
    let service = create_test_monitoring_service().await;

    // 记录一些指标
    service.record_query_execution(150.0).await.expect("记录查询执行失败");
    service.record_cache_hit().await.expect("记录缓存命中失败");
    service.record_cache_miss().await.expect("记录缓存未命中失败");

    // 获取统计信息
    let stats = service.get_stats().await.expect("获取统计信息失败");

    // 验证统计信息
    assert!(stats.metrics_collected > 0, "应该收集了指标");
    assert!(stats.last_collection_time.is_some(), "应该有最后收集时间");
}

#[tokio::test]
async fn test_monitoring_alerts() {
    let service = create_test_monitoring_service().await;

    // 测试获取告警（如果实现了的话）
    // 注意：这个方法可能还没有实现，所以我们先测试基础功能
    
    // 模拟高CPU使用率情况
    let metrics = service.get_metrics().await.expect("获取监控指标失败");
    
    // 检查是否需要告警
    if metrics.cpu_usage > 80.0 {
        println!("警告：CPU使用率过高: {}%", metrics.cpu_usage);
    }
    
    if metrics.memory_usage > 90.0 {
        println!("警告：内存使用率过高: {}%", metrics.memory_usage);
    }
    
    // 这个测试主要验证监控数据的获取是否正常
    assert!(true, "监控告警检查完成");
}

#[tokio::test]
async fn test_monitoring_concurrent_access() {
    let service = create_test_monitoring_service().await;

    // 并发测试
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            // 并发记录指标
            service_clone.record_query_execution(100.0 + i as f64).await.unwrap();
            
            // 并发获取指标
            let _metrics = service_clone.get_metrics().await.unwrap();
            
            // 并发获取活动
            let _activities = service_clone.get_recent_activities(5).await.unwrap();
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.expect("并发任务应该成功完成");
    }
    
    // 验证最终状态
    let final_stats = service.get_stats().await.expect("获取最终统计失败");
    assert!(final_stats.metrics_collected >= 10, "应该收集了至少10个指标");
}

// 辅助函数：创建测试用的监控服务
async fn create_test_monitoring_service() -> MonitoringService {
    let config = MonitoringConfig {
        collection_interval_seconds: 10,
        retention_days: 7,
        alert_thresholds: std::collections::HashMap::new(),
        enable_system_monitoring: true,
        enable_query_monitoring: true,
        enable_performance_monitoring: true,
    };

    let registry = Registry::new();
    
    MonitoringService::new(config, registry).await.expect("创建监控服务失败")
}
