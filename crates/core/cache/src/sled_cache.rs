//! Sled-based cache implementation

use crate::{Cache, CacheStats};
use async_trait::async_trait;
use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Cache entry with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<u64>, // Unix timestamp in seconds
}

impl CacheEntry {
    fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        let expires_at = ttl.map(|duration| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + duration.as_secs()
        });

        Self { data, expires_at }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
}

/// Sled-based cache implementation
pub struct SledCache {
    db: sled::Db,
    stats: Arc<RwLock<CacheStats>>,
}

impl SledCache {
    /// Create a new sled cache
    #[instrument(skip_all)]
    pub async fn new(path: &str) -> Result<Self> {
        info!("初始化Sled缓存，路径: {}", path);
        
        let db = sled::open(path)
            .map_err(|e| DuckHubError::cache(format!("打开Sled数据库失败: {}", e)))?;

        let stats = Arc::new(RwLock::new(CacheStats {
            hit_count: 0,
            miss_count: 0,
            total_keys: db.len() as u64,
            memory_usage: 0, // Sled manages memory internally
        }));

        Ok(Self { db, stats })
    }

    /// Clean up expired entries
    #[instrument(skip(self))]
    async fn cleanup_expired(&self) -> Result<()> {
        let mut expired_keys = Vec::new();
        
        for result in self.db.iter() {
            match result {
                Ok((key, value)) => {
                    if let Ok(entry) = bincode::deserialize::<CacheEntry>(&value) {
                        if entry.is_expired() {
                            expired_keys.push(key);
                        }
                    }
                }
                Err(e) => {
                    warn!("遍历缓存条目时出错: {}", e);
                }
            }
        }

        for key in expired_keys {
            if let Err(e) = self.db.remove(&key) {
                warn!("删除过期缓存条目失败: {}", e);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Cache for SledCache {
    #[instrument(skip(self))]
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        debug!("从缓存获取键: {}", key);

        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => {
                match bincode::deserialize::<CacheEntry>(&value) {
                    Ok(entry) => {
                        if entry.is_expired() {
                            // 删除过期条目
                            let _ = self.db.remove(key.as_bytes());
                            
                            // 更新统计
                            let mut stats = self.stats.write().await;
                            stats.miss_count += 1;
                            stats.total_keys = self.db.len() as u64;
                            
                            debug!("缓存条目已过期: {}", key);
                            Ok(None)
                        } else {
                            match serde_json::from_slice(&entry.data) {
                                Ok(data) => {
                                    // 更新统计
                                    let mut stats = self.stats.write().await;
                                    stats.hit_count += 1;
                                    
                                    debug!("缓存命中: {}", key);
                                    Ok(Some(data))
                                }
                                Err(e) => {
                                    error!("反序列化缓存数据失败: {}", e);
                                    Err(DuckHubError::cache(format!("反序列化失败: {}", e)))
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("反序列化缓存条目失败: {}", e);
                        Err(DuckHubError::cache(format!("反序列化条目失败: {}", e)))
                    }
                }
            }
            Ok(None) => {
                // 更新统计
                let mut stats = self.stats.write().await;
                stats.miss_count += 1;
                
                debug!("缓存未命中: {}", key);
                Ok(None)
            }
            Err(e) => {
                error!("从Sled获取数据失败: {}", e);
                Err(DuckHubError::cache(format!("获取数据失败: {}", e)))
            }
        }
    }

    #[instrument(skip(self, value))]
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        debug!("设置缓存键: {}, TTL: {:?}", key, ttl);

        let data = serde_json::to_vec(value)
            .map_err(|e| DuckHubError::cache(format!("序列化失败: {}", e)))?;

        let entry = CacheEntry::new(data, ttl);
        let entry_bytes = bincode::serialize(&entry)
            .map_err(|e| DuckHubError::cache(format!("序列化条目失败: {}", e)))?;

        self.db
            .insert(key.as_bytes(), entry_bytes)
            .map_err(|e| DuckHubError::cache(format!("插入数据失败: {}", e)))?;

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_keys = self.db.len() as u64;

        debug!("缓存设置成功: {}", key);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        debug!("删除缓存键: {}", key);

        self.db
            .remove(key.as_bytes())
            .map_err(|e| DuckHubError::cache(format!("删除数据失败: {}", e)))?;

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_keys = self.db.len() as u64;

        debug!("缓存删除成功: {}", key);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<bool> {
        // 尝试写入和读取测试数据
        let test_key = "__health_check__";
        let test_value = "ok";

        match self.set(test_key, &test_value, Some(Duration::from_secs(1))).await {
            Ok(_) => {
                match self.get::<String>(test_key).await {
                    Ok(Some(value)) if value == test_value => {
                        let _ = self.delete(test_key).await;
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    #[instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        info!("清空所有缓存");

        self.db
            .clear()
            .map_err(|e| DuckHubError::cache(format!("清空缓存失败: {}", e)))?;

        // 重置统计
        let mut stats = self.stats.write().await;
        *stats = CacheStats {
            hit_count: 0,
            miss_count: 0,
            total_keys: 0,
            memory_usage: 0,
        };

        info!("缓存清空完成");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn stats(&self) -> Result<CacheStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}
