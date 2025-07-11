//! Redis-based cache implementation

use crate::{Cache, CacheStats};
use async_trait::async_trait;
use duckhub_common::prelude::*;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};

/// Redis-based cache implementation
pub struct RedisCache {
    client: Client,
    connection: Arc<Mutex<redis::aio::Connection>>,
    key_prefix: String,
    default_ttl: Duration,
    stats: Arc<RwLock<CacheStats>>,
}

impl RedisCache {
    /// Create a new Redis cache
    #[instrument(skip_all)]
    pub async fn new(url: &str) -> Result<Self> {
        info!("初始化Redis缓存，URL: {}", url);

        let client = Client::open(url)
            .map_err(|e| DuckHubError::cache(format!("连接Redis失败: {}", e)))?;

        let connection = client
            .get_async_connection()
            .await
            .map_err(|e| DuckHubError::cache(format!("获取Redis连接失败: {}", e)))?;

        let stats = Arc::new(RwLock::new(CacheStats {
            hit_count: 0,
            miss_count: 0,
            total_keys: 0,
            memory_usage: 0,
        }));

        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(connection)),
            key_prefix: "duckhub:cache:".to_string(),
            default_ttl: Duration::from_secs(3600), // 1 hour default
            stats,
        })
    }

    /// Get the full key with prefix
    fn full_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }
}

#[async_trait]
impl Cache for RedisCache {
    #[instrument(skip(self))]
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        debug!("从Redis缓存获取键: {}", key);

        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        match conn.get::<_, Option<String>>(&full_key).await {
            Ok(Some(value)) => {
                match serde_json::from_str(&value) {
                    Ok(data) => {
                        // 更新统计
                        let mut stats = self.stats.write().await;
                        stats.hit_count += 1;

                        debug!("Redis缓存命中: {}", key);
                        Ok(Some(data))
                    }
                    Err(e) => {
                        error!("反序列化Redis数据失败: {}", e);
                        
                        // 更新统计
                        let mut stats = self.stats.write().await;
                        stats.miss_count += 1;

                        Err(DuckHubError::cache(format!("反序列化失败: {}", e)))
                    }
                }
            }
            Ok(None) => {
                // 更新统计
                let mut stats = self.stats.write().await;
                stats.miss_count += 1;

                debug!("Redis缓存未命中: {}", key);
                Ok(None)
            }
            Err(e) => {
                error!("从Redis获取数据失败: {}", e);
                
                // 更新统计
                let mut stats = self.stats.write().await;
                stats.miss_count += 1;

                Err(DuckHubError::cache(format!("获取数据失败: {}", e)))
            }
        }
    }

    #[instrument(skip(self, value))]
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        debug!("设置Redis缓存键: {}, TTL: {:?}", key, ttl);

        let data = serde_json::to_string(value)
            .map_err(|e| DuckHubError::cache(format!("序列化失败: {}", e)))?;

        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        let ttl_seconds = ttl.unwrap_or(self.default_ttl).as_secs();

        match conn.set_ex::<_, _, ()>(&full_key, &data, ttl_seconds).await {
            Ok(_) => {
                debug!("Redis缓存设置成功: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("设置Redis缓存失败: {}", e);
                Err(DuckHubError::cache(format!("设置数据失败: {}", e)))
            }
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        debug!("删除Redis缓存键: {}", key);

        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        match conn.del::<_, ()>(&full_key).await {
            Ok(_) => {
                debug!("Redis缓存删除成功: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("删除Redis缓存失败: {}", e);
                Err(DuckHubError::cache(format!("删除数据失败: {}", e)))
            }
        }
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool> {
        let mut conn = self.connection.lock().await;

        // 使用简单的SET/GET测试来检查连接
        let test_key = format!("{}__health_check__", self.key_prefix);
        let test_value = "ok";

        match conn.set_ex::<_, _, ()>(&test_key, test_value, 1).await {
            Ok(_) => {
                match conn.get::<_, Option<String>>(&test_key).await {
                    Ok(Some(value)) if value == test_value => {
                        let _ = conn.del::<_, ()>(&test_key).await;
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Err(e) => {
                warn!("Redis健康检查失败: {}", e);
                Ok(false)
            }
        }
    }

    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        info!("清空Redis缓存（仅限前缀匹配的键）");

        let pattern = format!("{}*", self.key_prefix);
        let mut conn = self.connection.lock().await;

        // 获取所有匹配的键
        match conn.keys::<_, Vec<String>>(&pattern).await {
            Ok(keys) => {
                if !keys.is_empty() {
                    match conn.del::<_, ()>(&keys).await {
                        Ok(_) => {
                            info!("清空了 {} 个Redis缓存条目", keys.len());
                            
                            // 重置统计
                            let mut stats = self.stats.write().await;
                            *stats = CacheStats {
                                hit_count: 0,
                                miss_count: 0,
                                total_keys: 0,
                                memory_usage: 0,
                            };
                        }
                        Err(e) => {
                            error!("删除Redis缓存条目失败: {}", e);
                            return Err(DuckHubError::cache(format!("删除失败: {}", e)));
                        }
                    }
                } else {
                    info!("没有找到需要清空的Redis缓存条目");
                }
                Ok(())
            }
            Err(e) => {
                error!("获取Redis键列表失败: {}", e);
                Err(DuckHubError::cache(format!("获取键列表失败: {}", e)))
            }
        }
    }

    #[instrument(skip(self))]
    async fn stats(&self) -> Result<CacheStats> {
        let mut stats = self.stats.write().await;
        
        // 尝试获取当前键数量
        let pattern = format!("{}*", self.key_prefix);
        let mut conn = self.connection.lock().await;
        
        match conn.keys::<_, Vec<String>>(&pattern).await {
            Ok(keys) => {
                stats.total_keys = keys.len() as u64;
            }
            Err(e) => {
                warn!("获取Redis键数量失败: {}", e);
            }
        }

        Ok(stats.clone())
    }
}
