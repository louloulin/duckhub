//! Cache implementations for DuckHub

use duckhub_common::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Cache trait for different implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from cache
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    /// Set a value in cache with optional TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Delete a key from cache
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if cache is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Clear all cache entries
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub total_keys: u64,
    pub memory_usage: u64,
}

/// Cache backend enum
pub enum CacheBackend {
    Sled(SledCache),
    Memory(MemoryCache),
    Redis(RedisCache),
}

/// Cache manager that can switch between different implementations
pub struct CacheManager {
    backend: CacheBackend,
}

impl CacheManager {
    /// Create a new cache manager with sled backend
    pub async fn new_sled(path: &str) -> Result<Self> {
        let cache = SledCache::new(path).await?;
        Ok(Self {
            backend: CacheBackend::Sled(cache),
        })
    }

    /// Create a new cache manager with memory backend
    pub fn new_memory(default_ttl: Duration) -> Self {
        let cache = MemoryCache::new(default_ttl);
        Self {
            backend: CacheBackend::Memory(cache),
        }
    }

    /// Create a new cache manager with Redis backend
    pub async fn new_redis(url: &str) -> Result<Self> {
        let cache = RedisCache::new(url).await?;
        Ok(Self {
            backend: CacheBackend::Redis(cache),
        })
    }
}

#[async_trait]
impl Cache for CacheManager {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.get(key).await,
            CacheBackend::Memory(cache) => cache.get(key).await,
            CacheBackend::Redis(cache) => cache.get(key).await,
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.set(key, value, ttl).await,
            CacheBackend::Memory(cache) => cache.set(key, value, ttl).await,
            CacheBackend::Redis(cache) => cache.set(key, value, ttl).await,
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.delete(key).await,
            CacheBackend::Memory(cache) => cache.delete(key).await,
            CacheBackend::Redis(cache) => cache.delete(key).await,
        }
    }

    async fn health_check(&self) -> Result<bool> {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.health_check().await,
            CacheBackend::Memory(cache) => cache.health_check().await,
            CacheBackend::Redis(cache) => cache.health_check().await,
        }
    }

    async fn clear(&self) -> Result<()> {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.clear().await,
            CacheBackend::Memory(cache) => cache.clear().await,
            CacheBackend::Redis(cache) => cache.clear().await,
        }
    }

    async fn stats(&self) -> Result<CacheStats> {
        match &self.backend {
            CacheBackend::Sled(cache) => cache.stats().await,
            CacheBackend::Memory(cache) => cache.stats().await,
            CacheBackend::Redis(cache) => cache.stats().await,
        }
    }
}

mod sled_cache;
mod memory_cache;
mod redis_cache;

pub use sled_cache::*;
pub use memory_cache::*;
pub use redis_cache::*;
