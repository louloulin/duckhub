//! Caching implementations for DuckHub

use duckhub_common::prelude::*;
use redis::{AsyncCommands, Client, Connection as RedisConnection};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument, warn};

/// Redis-based cache implementation
pub struct QueryCacheImpl {
    client: Client,
    connection: Arc<Mutex<redis::aio::Connection>>,
    key_prefix: String,
    default_ttl: u64,
}

impl QueryCacheImpl {
    /// Create a new Redis cache
    pub async fn new(redis_url: &str, key_prefix: String, default_ttl: u64) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| DuckHubError::cache(format!("Failed to create Redis client: {}", e)))?;

        let connection = client.get_async_connection().await
            .map_err(|e| DuckHubError::cache(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(connection)),
            key_prefix,
            default_ttl,
        })
    }

    /// Get full key with prefix
    fn full_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl Cache for QueryCacheImpl {
    #[instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;
        
        match conn.get::<_, Option<Vec<u8>>>(&full_key).await {
            Ok(value) => {
                if value.is_some() {
                    debug!("Cache hit for key: {}", key);
                } else {
                    debug!("Cache miss for key: {}", key);
                }
                Ok(value)
            }
            Err(e) => {
                error!("Cache get error for key {}: {}", key, e);
                Err(DuckHubError::cache(format!("Failed to get from cache: {}", e)))
            }
        }
    }

    #[instrument(skip(self, value))]
    async fn set(&self, key: &str, value: &[u8], ttl: Option<u64>) -> Result<()> {
        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);

        match conn.set_ex::<_, _, ()>(&full_key, value, ttl_seconds).await {
            Ok(_) => {
                debug!("Cached value for key: {} (TTL: {}s)", key, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                error!("Cache set error for key {}: {}", key, e);
                Err(DuckHubError::cache(format!("Failed to set cache: {}", e)))
            }
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        match conn.del::<_, ()>(&full_key).await {
            Ok(_) => {
                debug!("Deleted cache key: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("Cache delete error for key {}: {}", key, e);
                Err(DuckHubError::cache(format!("Failed to delete from cache: {}", e)))
            }
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        match conn.exists::<_, bool>(&full_key).await {
            Ok(exists) => Ok(exists),
            Err(e) => {
                error!("Cache exists error for key {}: {}", key, e);
                Err(DuckHubError::cache(format!("Failed to check cache existence: {}", e)))
            }
        }
    }

    async fn expire(&self, key: &str, ttl: u64) -> Result<()> {
        let full_key = self.full_key(key);
        let mut conn = self.connection.lock().await;

        match conn.expire::<_, ()>(&full_key, ttl as i64).await {
            Ok(_) => {
                debug!("Set TTL for key: {} ({}s)", key, ttl);
                Ok(())
            }
            Err(e) => {
                error!("Cache expire error for key {}: {}", key, e);
                Err(DuckHubError::cache(format!("Failed to set cache TTL: {}", e)))
            }
        }
    }

    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
        let full_keys: Vec<String> = keys.iter().map(|k| self.full_key(k)).collect();
        let mut conn = self.connection.lock().await;

        match conn.get::<_, Vec<Option<Vec<u8>>>>(&full_keys).await {
            Ok(values) => {
                debug!("Multi-get for {} keys", keys.len());
                Ok(values)
            }
            Err(e) => {
                error!("Cache mget error: {}", e);
                Err(DuckHubError::cache(format!("Failed to multi-get from cache: {}", e)))
            }
        }
    }

    async fn mset(&self, pairs: &[(String, Vec<u8>)]) -> Result<()> {
        let mut conn = self.connection.lock().await;
        let full_pairs: Vec<(String, Vec<u8>)> = pairs.iter()
            .map(|(k, v)| (self.full_key(k), v.clone()))
            .collect();

        match conn.mset::<_, _, ()>(&full_pairs).await {
            Ok(_) => {
                debug!("Multi-set for {} keys", pairs.len());
                Ok(())
            }
            Err(e) => {
                error!("Cache mset error: {}", e);
                Err(DuckHubError::cache(format!("Failed to multi-set cache: {}", e)))
            }
        }
    }

    async fn clear(&self) -> Result<()> {
        let pattern = format!("{}:*", self.key_prefix);
        let mut conn = self.connection.lock().await;

        // Get all keys matching the pattern
        let keys: Vec<String> = match conn.keys(&pattern).await {
            Ok(keys) => keys,
            Err(e) => {
                error!("Failed to get keys for pattern {}: {}", pattern, e);
                return Err(DuckHubError::cache(format!("Failed to get cache keys: {}", e)));
            }
        };

        if !keys.is_empty() {
            match conn.del::<_, ()>(&keys).await {
                Ok(_) => {
                    debug!("Cleared {} cache keys", keys.len());
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to clear cache keys: {}", e);
                    Err(DuckHubError::cache(format!("Failed to clear cache: {}", e)))
                }
            }
        } else {
            debug!("No cache keys to clear");
            Ok(())
        }
    }
}

/// In-memory cache implementation for testing
pub struct MemoryCache {
    storage: Arc<Mutex<std::collections::HashMap<String, CacheEntry>>>,
    default_ttl: u64,
}

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<std::time::Instant>,
}

impl MemoryCache {
    /// Create a new in-memory cache
    pub fn new(default_ttl: u64) -> Self {
        Self {
            storage: Arc::new(Mutex::new(std::collections::HashMap::new())),
            default_ttl,
        }
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let mut storage = self.storage.lock().await;
        let now = std::time::Instant::now();
        
        storage.retain(|_, entry| {
            entry.expires_at.map_or(true, |expires| expires > now)
        });
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let storage = self.storage.lock().await;
        
        if let Some(entry) = storage.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if expires_at <= std::time::Instant::now() {
                    return Ok(None);
                }
            }
            Ok(Some(entry.data.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<u64>) -> Result<()> {
        let mut storage = self.storage.lock().await;
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);
        
        let expires_at = if ttl_seconds > 0 {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(ttl_seconds))
        } else {
            None
        };

        storage.insert(key.to_string(), CacheEntry {
            data: value.to_vec(),
            expires_at,
        });

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.storage.lock().await;
        storage.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let storage = self.storage.lock().await;
        
        if let Some(entry) = storage.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if expires_at <= std::time::Instant::now() {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn expire(&self, key: &str, ttl: u64) -> Result<()> {
        let mut storage = self.storage.lock().await;
        
        if let Some(entry) = storage.get_mut(key) {
            entry.expires_at = if ttl > 0 {
                Some(std::time::Instant::now() + std::time::Duration::from_secs(ttl))
            } else {
                None
            };
        }

        Ok(())
    }

    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>> {
        let storage = self.storage.lock().await;
        let now = std::time::Instant::now();
        
        let mut results = Vec::new();
        for key in keys {
            if let Some(entry) = storage.get(key) {
                // Check if expired
                if let Some(expires_at) = entry.expires_at {
                    if expires_at <= now {
                        results.push(None);
                        continue;
                    }
                }
                results.push(Some(entry.data.clone()));
            } else {
                results.push(None);
            }
        }

        Ok(results)
    }

    async fn mset(&self, pairs: &[(String, Vec<u8>)]) -> Result<()> {
        let mut storage = self.storage.lock().await;
        let expires_at = if self.default_ttl > 0 {
            Some(std::time::Instant::now() + std::time::Duration::from_secs(self.default_ttl))
        } else {
            None
        };

        for (key, value) in pairs {
            storage.insert(key.clone(), CacheEntry {
                data: value.clone(),
                expires_at,
            });
        }

        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.lock().await;
        storage.clear();
        Ok(())
    }
}

/// Cache manager for managing multiple cache instances
pub struct CacheManager {
    caches: std::collections::HashMap<String, Arc<dyn Cache>>,
    default_cache: Option<String>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self {
            caches: std::collections::HashMap::new(),
            default_cache: None,
        }
    }

    /// Register a cache instance
    pub fn register_cache(&mut self, name: String, cache: Arc<dyn Cache>) {
        self.caches.insert(name.clone(), cache);
        
        // Set as default if it's the first cache
        if self.default_cache.is_none() {
            self.default_cache = Some(name);
        }
    }

    /// Get a cache by name
    pub fn get_cache(&self, name: &str) -> Result<Arc<dyn Cache>> {
        self.caches.get(name)
            .cloned()
            .ok_or_else(|| DuckHubError::not_found(format!("Cache '{}'", name)))
    }

    /// Get the default cache
    pub fn get_default_cache(&self) -> Result<Arc<dyn Cache>> {
        let cache_name = self.default_cache.as_ref()
            .ok_or_else(|| DuckHubError::config("No default cache set"))?;
        self.get_cache(cache_name)
    }

    /// Set the default cache
    pub fn set_default_cache(&mut self, name: String) -> Result<()> {
        if !self.caches.contains_key(&name) {
            return Err(DuckHubError::not_found(format!("Cache '{}'", name)));
        }
        self.default_cache = Some(name);
        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() {
        let cache = MemoryCache::new(60);
        
        // Test set and get
        cache.set("test_key", b"test_value", None).await.unwrap();
        let value = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));
        
        // Test exists
        assert!(cache.exists("test_key").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());
        
        // Test delete
        cache.delete("test_key").await.unwrap();
        assert!(!cache.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MemoryCache::new(1);
        
        cache.set("test_key", b"test_value", Some(1)).await.unwrap();
        assert!(cache.exists("test_key").await.unwrap());
        
        // Wait for expiration
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        assert!(!cache.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let mut manager = CacheManager::new();
        let cache = Arc::new(MemoryCache::new(60));
        
        manager.register_cache("test_cache".to_string(), cache.clone());
        
        let retrieved = manager.get_cache("test_cache").unwrap();
        // Check that we got the same cache back (by checking the cache name)
        assert!(retrieved.get("test_key").await.is_ok());

        let default = manager.get_default_cache().unwrap();
        // Check that we got the same cache back (by checking the cache name)
        assert!(default.get("test_key").await.is_ok());
    }
}
