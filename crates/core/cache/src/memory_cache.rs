//! In-memory cache implementation

use crate::{Cache, CacheStats};
use async_trait::async_trait;
use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Memory cache entry with expiration
#[derive(Debug, Clone)]
struct MemoryCacheEntry {
    data: Vec<u8>,
    expires_at: Option<Instant>,
}

impl MemoryCacheEntry {
    fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|duration| Instant::now() + duration);
        Self { data, expires_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() > expires_at
        } else {
            false
        }
    }
}

/// In-memory cache implementation
pub struct MemoryCache {
    storage: Arc<RwLock<HashMap<String, MemoryCacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    default_ttl: Duration,
}

impl MemoryCache {
    /// Create a new in-memory cache
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats {
                hit_count: 0,
                miss_count: 0,
                total_keys: 0,
                memory_usage: 0,
            })),
            default_ttl,
        }
    }

    /// Clean up expired entries
    #[instrument(skip(self))]
    async fn cleanup_expired(&self) {
        let mut storage = self.storage.write().await;
        let mut stats = self.stats.write().await;
        
        let initial_count = storage.len();
        storage.retain(|_, entry| !entry.is_expired());
        let removed_count = initial_count - storage.len();
        
        if removed_count > 0 {
            debug!("清理了 {} 个过期的内存缓存条目", removed_count);
            stats.total_keys = storage.len() as u64;
            
            // 重新计算内存使用量
            let memory_usage: usize = storage
                .values()
                .map(|entry| entry.data.len())
                .sum();
            stats.memory_usage = memory_usage as u64;
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    #[instrument(skip(self))]
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        debug!("从内存缓存获取键: {}", key);

        // 先清理过期条目
        self.cleanup_expired().await;

        let storage = self.storage.read().await;
        let mut stats = self.stats.write().await;

        match storage.get(key) {
            Some(entry) => {
                if entry.is_expired() {
                    stats.miss_count += 1;
                    debug!("内存缓存条目已过期: {}", key);
                    Ok(None)
                } else {
                    match serde_json::from_slice(&entry.data) {
                        Ok(data) => {
                            stats.hit_count += 1;
                            debug!("内存缓存命中: {}", key);
                            Ok(Some(data))
                        }
                        Err(e) => {
                            stats.miss_count += 1;
                            Err(DuckHubError::cache(format!("反序列化失败: {}", e)))
                        }
                    }
                }
            }
            None => {
                stats.miss_count += 1;
                debug!("内存缓存未命中: {}", key);
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, value))]
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        debug!("设置内存缓存键: {}, TTL: {:?}", key, ttl);

        let data = serde_json::to_vec(value)
            .map_err(|e| DuckHubError::cache(format!("序列化失败: {}", e)))?;

        let ttl = ttl.unwrap_or(self.default_ttl);
        let entry = MemoryCacheEntry::new(data.clone(), Some(ttl));

        {
            let mut storage = self.storage.write().await;
            storage.insert(key.to_string(), entry);
        }

        // 更新统计
        {
            let storage = self.storage.read().await;
            let mut stats = self.stats.write().await;
            stats.total_keys = storage.len() as u64;
            
            // 重新计算内存使用量
            let memory_usage: usize = storage
                .values()
                .map(|entry| entry.data.len())
                .sum();
            stats.memory_usage = memory_usage as u64;
        }

        debug!("内存缓存设置成功: {}", key);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        debug!("删除内存缓存键: {}", key);

        {
            let mut storage = self.storage.write().await;
            storage.remove(key);
        }

        // 更新统计
        {
            let storage = self.storage.read().await;
            let mut stats = self.stats.write().await;
            stats.total_keys = storage.len() as u64;
            
            // 重新计算内存使用量
            let memory_usage: usize = storage
                .values()
                .map(|entry| entry.data.len())
                .sum();
            stats.memory_usage = memory_usage as u64;
        }

        debug!("内存缓存删除成功: {}", key);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool> {
        // 内存缓存总是健康的
        Ok(true)
    }

    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        info!("清空所有内存缓存");

        {
            let mut storage = self.storage.write().await;
            storage.clear();
        }

        // 重置统计
        {
            let mut stats = self.stats.write().await;
            *stats = CacheStats {
                hit_count: 0,
                miss_count: 0,
                total_keys: 0,
                memory_usage: 0,
            };
        }

        info!("内存缓存清空完成");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}
