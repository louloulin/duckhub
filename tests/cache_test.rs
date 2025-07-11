//! 缓存系统测试

use duckhub_cache::{Cache, CacheManager, CacheConfig, CacheBackend};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    value: f64,
}

#[tokio::test]
async fn test_cache_basic_operations() {
    // 创建内存缓存配置
    let config = CacheConfig {
        backend: "memory".to_string(),
        memory_max_size: Some(1000),
        memory_ttl_seconds: Some(300),
        redis_url: None,
        sled_path: None,
    };

    let cache_manager = CacheManager::new(config).await.expect("创建缓存管理器失败");

    // 测试数据
    let test_data = TestData {
        id: 1,
        name: "测试数据".to_string(),
        value: 123.45,
    };

    // 测试设置和获取
    cache_manager.set("test_key", &test_data, Some(Duration::from_secs(60)))
        .await
        .expect("设置缓存失败");

    let retrieved: Option<TestData> = cache_manager.get("test_key")
        .await
        .expect("获取缓存失败");

    assert_eq!(retrieved, Some(test_data));

    // 测试删除
    cache_manager.delete("test_key")
        .await
        .expect("删除缓存失败");

    let after_delete: Option<TestData> = cache_manager.get("test_key")
        .await
        .expect("获取缓存失败");

    assert_eq!(after_delete, None);
}

#[tokio::test]
async fn test_cache_health_check() {
    let config = CacheConfig {
        backend: "memory".to_string(),
        memory_max_size: Some(1000),
        memory_ttl_seconds: Some(300),
        redis_url: None,
        sled_path: None,
    };

    let cache_manager = CacheManager::new(config).await.expect("创建缓存管理器失败");

    // 测试健康检查
    let health = cache_manager.health_check().await.expect("健康检查失败");
    assert!(health, "缓存应该是健康的");
}

#[tokio::test]
async fn test_cache_stats() {
    let config = CacheConfig {
        backend: "memory".to_string(),
        memory_max_size: Some(1000),
        memory_ttl_seconds: Some(300),
        redis_url: None,
        sled_path: None,
    };

    let cache_manager = CacheManager::new(config).await.expect("创建缓存管理器失败");

    // 添加一些数据
    let test_data = TestData {
        id: 1,
        name: "测试".to_string(),
        value: 100.0,
    };

    cache_manager.set("key1", &test_data, None).await.expect("设置缓存失败");
    cache_manager.set("key2", &test_data, None).await.expect("设置缓存失败");

    // 获取统计信息
    let stats = cache_manager.stats().await.expect("获取统计信息失败");
    
    // 验证统计信息
    assert!(stats.total_keys >= 2, "应该至少有2个键");
    assert!(stats.memory_usage > 0, "内存使用应该大于0");
}

#[tokio::test]
async fn test_sled_cache() {
    let config = CacheConfig {
        backend: "sled".to_string(),
        memory_max_size: None,
        memory_ttl_seconds: None,
        redis_url: None,
        sled_path: Some("/tmp/duckhub_test_cache".to_string()),
    };

    let cache_manager = CacheManager::new(config).await.expect("创建Sled缓存管理器失败");

    let test_data = TestData {
        id: 2,
        name: "Sled测试".to_string(),
        value: 456.78,
    };

    // 测试Sled缓存操作
    cache_manager.set("sled_key", &test_data, None)
        .await
        .expect("Sled设置缓存失败");

    let retrieved: Option<TestData> = cache_manager.get("sled_key")
        .await
        .expect("Sled获取缓存失败");

    assert_eq!(retrieved, Some(test_data));

    // 测试健康检查
    let health = cache_manager.health_check().await.expect("Sled健康检查失败");
    assert!(health, "Sled缓存应该是健康的");

    // 清理测试数据
    cache_manager.clear().await.expect("清理Sled缓存失败");
}

#[tokio::test]
async fn test_cache_ttl() {
    let config = CacheConfig {
        backend: "memory".to_string(),
        memory_max_size: Some(1000),
        memory_ttl_seconds: Some(1), // 1秒TTL
        redis_url: None,
        sled_path: None,
    };

    let cache_manager = CacheManager::new(config).await.expect("创建缓存管理器失败");

    let test_data = TestData {
        id: 3,
        name: "TTL测试".to_string(),
        value: 789.01,
    };

    // 设置短TTL
    cache_manager.set("ttl_key", &test_data, Some(Duration::from_millis(100)))
        .await
        .expect("设置TTL缓存失败");

    // 立即获取应该成功
    let immediate: Option<TestData> = cache_manager.get("ttl_key")
        .await
        .expect("立即获取缓存失败");
    assert_eq!(immediate, Some(test_data));

    // 等待TTL过期
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 过期后获取应该返回None
    let expired: Option<TestData> = cache_manager.get("ttl_key")
        .await
        .expect("过期获取缓存失败");
    assert_eq!(expired, None);
}
