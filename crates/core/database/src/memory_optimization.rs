//! Memory Pool and Object Pool Optimization
//! 
//! This module provides high-performance memory management for DuckLake
//! stream processing, including object pooling and memory allocation strategies.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::NonNull;
use tracing::{debug, warn, error};
use serde::{Serialize, Deserialize};

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    pub initial_pool_size: usize,
    pub max_pool_size: usize,
    pub chunk_size: usize,
    pub enable_monitoring: bool,
    pub gc_threshold: f64,
    pub preallocation_enabled: bool,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_pool_size: 1024,
            max_pool_size: 10240,
            chunk_size: 4096, // 4KB chunks
            enable_monitoring: true,
            gc_threshold: 0.8, // Trigger GC when 80% full
            preallocation_enabled: true,
        }
    }
}

/// High-performance memory pool
#[derive(Debug)]
pub struct MemoryPool {
    config: MemoryPoolConfig,
    free_chunks: Arc<Mutex<VecDeque<MemoryChunk>>>,
    allocated_chunks: Arc<RwLock<Vec<MemoryChunk>>>,
    metrics: Arc<MemoryPoolMetrics>,
}

/// Memory chunk
#[derive(Debug, Clone)]
pub struct MemoryChunk {
    pub ptr: NonNull<u8>,
    pub size: usize,
    pub allocated_at: std::time::Instant,
    pub id: u64,
}

unsafe impl Send for MemoryChunk {}
unsafe impl Sync for MemoryChunk {}

/// Memory pool metrics
#[derive(Debug)]
pub struct MemoryPoolMetrics {
    pub total_allocated: prometheus::Gauge,
    pub total_freed: prometheus::Counter,
    pub pool_size: prometheus::Gauge,
    pub allocation_failures: prometheus::Counter,
    pub gc_runs: prometheus::Counter,
    pub memory_usage: prometheus::Gauge,
}

impl Default for MemoryPoolMetrics {
    fn default() -> Self {
        Self {
            total_allocated: prometheus::Gauge::new("memory_pool_allocated_bytes", "Total allocated memory in bytes").unwrap(),
            total_freed: prometheus::Counter::new("memory_pool_freed_bytes_total", "Total freed memory in bytes").unwrap(),
            pool_size: prometheus::Gauge::new("memory_pool_size", "Current memory pool size").unwrap(),
            allocation_failures: prometheus::Counter::new("memory_pool_allocation_failures_total", "Total allocation failures").unwrap(),
            gc_runs: prometheus::Counter::new("memory_pool_gc_runs_total", "Total garbage collection runs").unwrap(),
            memory_usage: prometheus::Gauge::new("memory_pool_usage_ratio", "Memory pool usage ratio").unwrap(),
        }
    }
}

impl MemoryPool {
    /// Create a new memory pool
    pub async fn new(config: MemoryPoolConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = Self {
            config: config.clone(),
            free_chunks: Arc::new(Mutex::new(VecDeque::new())),
            allocated_chunks: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(MemoryPoolMetrics::default()),
        };

        // Pre-allocate initial chunks if enabled
        if config.preallocation_enabled {
            pool.preallocate_chunks().await?;
        }

        Ok(pool)
    }

    /// Pre-allocate memory chunks
    async fn preallocate_chunks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut free_chunks = self.free_chunks.lock().await;
        
        for i in 0..self.config.initial_pool_size {
            let chunk = self.allocate_system_chunk(i as u64)?;
            free_chunks.push_back(chunk);
        }

        self.metrics.pool_size.set(free_chunks.len() as f64);
        debug!("Pre-allocated {} memory chunks", free_chunks.len());
        Ok(())
    }

    /// Allocate a memory chunk from the pool
    pub async fn allocate(&self, size: usize) -> Result<MemoryChunk, Box<dyn std::error::Error>> {
        // Try to get a chunk from the free pool first
        {
            let mut free_chunks = self.free_chunks.lock().await;
            if let Some(chunk) = free_chunks.pop_front() {
                if chunk.size >= size {
                    // Update metrics
                    self.metrics.total_allocated.add(chunk.size as f64);
                    self.metrics.pool_size.set(free_chunks.len() as f64);
                    
                    // Move to allocated chunks
                    {
                        let mut allocated = self.allocated_chunks.write().await;
                        allocated.push(chunk.clone());
                    }
                    
                    return Ok(chunk);
                } else {
                    // Chunk is too small, put it back and allocate new one
                    free_chunks.push_back(chunk);
                }
            }
        }

        // No suitable chunk available, allocate new one
        let chunk_id = {
            let allocated = self.allocated_chunks.read().await;
            allocated.len() as u64
        };

        let chunk = self.allocate_system_chunk(chunk_id)?;
        
        // Update metrics
        self.metrics.total_allocated.add(chunk.size as f64);
        
        // Add to allocated chunks
        {
            let mut allocated = self.allocated_chunks.write().await;
            allocated.push(chunk.clone());
        }

        Ok(chunk)
    }

    /// Deallocate a memory chunk back to the pool
    pub async fn deallocate(&self, chunk: MemoryChunk) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from allocated chunks
        {
            let mut allocated = self.allocated_chunks.write().await;
            allocated.retain(|c| c.id != chunk.id);
        }

        // Check if pool is full
        {
            let free_chunks = self.free_chunks.lock().await;
            if free_chunks.len() >= self.config.max_pool_size {
                // Pool is full, actually free the memory
                self.free_system_chunk(&chunk)?;
                self.metrics.total_freed.inc_by(chunk.size as f64);
                return Ok(());
            }
        }

        // Add back to free pool
        {
            let mut free_chunks = self.free_chunks.lock().await;
            free_chunks.push_back(chunk.clone());
            self.metrics.pool_size.set(free_chunks.len() as f64);
        }

        self.metrics.total_freed.inc_by(chunk.size as f64);
        Ok(())
    }

    /// Allocate memory chunk from system
    fn allocate_system_chunk(&self, id: u64) -> Result<MemoryChunk, Box<dyn std::error::Error>> {
        let layout = Layout::from_size_align(self.config.chunk_size, 8)
            .map_err(|e| format!("Invalid layout: {}", e))?;

        let ptr = unsafe { System.alloc(layout) };
        
        if ptr.is_null() {
            self.metrics.allocation_failures.inc();
            return Err("Failed to allocate memory".into());
        }

        Ok(MemoryChunk {
            ptr: NonNull::new(ptr).unwrap(),
            size: self.config.chunk_size,
            allocated_at: std::time::Instant::now(),
            id,
        })
    }

    /// Free memory chunk to system
    fn free_system_chunk(&self, chunk: &MemoryChunk) -> Result<(), Box<dyn std::error::Error>> {
        let layout = Layout::from_size_align(chunk.size, 8)
            .map_err(|e| format!("Invalid layout: {}", e))?;

        unsafe {
            System.dealloc(chunk.ptr.as_ptr(), layout);
        }

        Ok(())
    }

    /// Run garbage collection
    pub async fn garbage_collect(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut freed_count = 0;
        let gc_threshold_time = std::time::Instant::now() - std::time::Duration::from_secs(300); // 5 minutes

        // Clean up old allocated chunks
        {
            let mut allocated = self.allocated_chunks.write().await;
            let initial_len = allocated.len();
            
            allocated.retain(|chunk| {
                if chunk.allocated_at < gc_threshold_time {
                    // This chunk is old, free it
                    if let Err(e) = self.free_system_chunk(chunk) {
                        warn!("Failed to free old chunk: {}", e);
                        return true; // Keep it if we can't free it
                    }
                    freed_count += 1;
                    false
                } else {
                    true
                }
            });

            debug!("GC freed {} old allocated chunks", initial_len - allocated.len());
        }

        // Clean up excess free chunks
        {
            let mut free_chunks = self.free_chunks.lock().await;
            let target_size = self.config.initial_pool_size;
            
            while free_chunks.len() > target_size {
                if let Some(chunk) = free_chunks.pop_back() {
                    if let Err(e) = self.free_system_chunk(&chunk) {
                        warn!("Failed to free excess chunk: {}", e);
                        free_chunks.push_back(chunk);
                        break;
                    }
                    freed_count += 1;
                }
            }

            self.metrics.pool_size.set(free_chunks.len() as f64);
        }

        self.metrics.gc_runs.inc();
        debug!("Garbage collection completed, freed {} chunks", freed_count);
        Ok(freed_count)
    }

    /// Get memory usage statistics
    pub async fn get_usage_stats(&self) -> MemoryUsageStats {
        let free_count = {
            let free_chunks = self.free_chunks.lock().await;
            free_chunks.len()
        };

        let allocated_count = {
            let allocated = self.allocated_chunks.read().await;
            allocated.len()
        };

        let total_memory = (free_count + allocated_count) * self.config.chunk_size;
        let used_memory = allocated_count * self.config.chunk_size;
        let usage_ratio = if total_memory > 0 {
            used_memory as f64 / total_memory as f64
        } else {
            0.0
        };

        self.metrics.memory_usage.set(usage_ratio);

        MemoryUsageStats {
            total_chunks: free_count + allocated_count,
            free_chunks: free_count,
            allocated_chunks: allocated_count,
            total_memory,
            used_memory,
            usage_ratio,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    pub total_chunks: usize,
    pub free_chunks: usize,
    pub allocated_chunks: usize,
    pub total_memory: usize,
    pub used_memory: usize,
    pub usage_ratio: f64,
}

/// Object pool for reusing expensive objects
pub struct ObjectPool<T> {
    objects: Arc<Mutex<VecDeque<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
    metrics: Arc<ObjectPoolMetrics>,
}

impl<T> std::fmt::Debug for ObjectPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectPool")
            .field("max_size", &self.max_size)
            .field("metrics", &self.metrics)
            .finish()
    }
}

/// Object pool metrics
#[derive(Debug)]
pub struct ObjectPoolMetrics {
    pub objects_created: prometheus::Counter,
    pub objects_reused: prometheus::Counter,
    pub pool_size: prometheus::Gauge,
    pub pool_hits: prometheus::Counter,
    pub pool_misses: prometheus::Counter,
}

impl Default for ObjectPoolMetrics {
    fn default() -> Self {
        Self {
            objects_created: prometheus::Counter::new("object_pool_objects_created_total", "Total objects created").unwrap(),
            objects_reused: prometheus::Counter::new("object_pool_objects_reused_total", "Total objects reused").unwrap(),
            pool_size: prometheus::Gauge::new("object_pool_size", "Current object pool size").unwrap(),
            pool_hits: prometheus::Counter::new("object_pool_hits_total", "Total pool hits").unwrap(),
            pool_misses: prometheus::Counter::new("object_pool_misses_total", "Total pool misses").unwrap(),
        }
    }
}

impl<T> ObjectPool<T>
where
    T: Send + 'static,
{
    /// Create a new object pool
    pub fn new<F>(factory: F, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            objects: Arc::new(Mutex::new(VecDeque::new())),
            factory: Arc::new(factory),
            max_size,
            metrics: Arc::new(ObjectPoolMetrics::default()),
        }
    }

    /// Get an object from the pool
    pub async fn get(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().await;
        
        if let Some(object) = objects.pop_front() {
            // Pool hit
            self.metrics.pool_hits.inc();
            self.metrics.objects_reused.inc();
            self.metrics.pool_size.set(objects.len() as f64);
            
            PooledObject {
                object: Some(object),
                pool: self.objects.clone(),
                metrics: self.metrics.clone(),
                max_size: self.max_size,
            }
        } else {
            // Pool miss, create new object
            self.metrics.pool_misses.inc();
            self.metrics.objects_created.inc();
            
            let object = (self.factory)();
            PooledObject {
                object: Some(object),
                pool: self.objects.clone(),
                metrics: self.metrics.clone(),
                max_size: self.max_size,
            }
        }
    }

    /// Get current pool size
    pub async fn size(&self) -> usize {
        let objects = self.objects.lock().await;
        objects.len()
    }

    /// Clear the pool
    pub async fn clear(&self) {
        let mut objects = self.objects.lock().await;
        objects.clear();
        self.metrics.pool_size.set(0.0);
    }
}

/// Pooled object wrapper that automatically returns to pool on drop
pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    metrics: Arc<ObjectPoolMetrics>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    /// Get a reference to the wrapped object
    pub fn as_ref(&self) -> Option<&T> {
        self.object.as_ref()
    }

    /// Get a mutable reference to the wrapped object
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.object.as_mut()
    }

    /// Take ownership of the object (prevents return to pool)
    pub fn take(mut self) -> Option<T> {
        self.object.take()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            // Try to return to pool
            if let Ok(mut pool) = self.pool.try_lock() {
                if pool.len() < self.max_size {
                    pool.push_back(object);
                    self.metrics.pool_size.set(pool.len() as f64);
                }
                // If pool is full or locked, object is dropped
            }
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().expect("PooledObject should always contain an object")
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().expect("PooledObject should always contain an object")
    }
}

/// Specialized pools for common objects

/// Buffer pool for byte vectors
pub type BufferPool = ObjectPool<Vec<u8>>;

impl BufferPool {
    /// Create a new buffer pool with specified capacity
    pub fn new_with_capacity(capacity: usize, max_size: usize) -> Self {
        Self::new(move || Vec::with_capacity(capacity), max_size)
    }
}

/// String pool for reusing string allocations
pub type StringPool = ObjectPool<String>;

impl StringPool {
    /// Create a new string pool
    pub fn new_string_pool(max_size: usize) -> Self {
        Self::new(String::new, max_size)
    }
}

/// HashMap pool for reusing hash map allocations
pub type HashMapPool<K, V> = ObjectPool<std::collections::HashMap<K, V>>;

impl<K, V> HashMapPool<K, V>
where
    K: std::hash::Hash + Eq + Send + 'static,
    V: Send + 'static,
{
    /// Create a new HashMap pool
    pub fn new_hashmap_pool(max_size: usize) -> Self {
        Self::new(std::collections::HashMap::new, max_size)
    }
}

/// Memory manager that coordinates all memory optimization
#[derive(Debug)]
pub struct MemoryManager {
    memory_pool: Arc<MemoryPool>,
    buffer_pool: Arc<BufferPool>,
    string_pool: Arc<StringPool>,
    config: MemoryPoolConfig,
}

impl MemoryManager {
    /// Create a new memory manager
    pub async fn new(config: MemoryPoolConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let memory_pool = Arc::new(MemoryPool::new(config.clone()).await?);
        let buffer_pool = Arc::new(BufferPool::new_with_capacity(4096, 1000));
        let string_pool = Arc::new(StringPool::new_string_pool(1000));

        Ok(Self {
            memory_pool,
            buffer_pool,
            string_pool,
            config,
        })
    }

    /// Get memory pool
    pub fn memory_pool(&self) -> Arc<MemoryPool> {
        self.memory_pool.clone()
    }

    /// Get buffer pool
    pub fn buffer_pool(&self) -> Arc<BufferPool> {
        self.buffer_pool.clone()
    }

    /// Get string pool
    pub fn string_pool(&self) -> Arc<StringPool> {
        self.string_pool.clone()
    }

    /// Run comprehensive garbage collection
    pub async fn garbage_collect_all(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let freed_chunks = self.memory_pool.garbage_collect().await?;
        
        // Clear object pools if they're too large
        if self.buffer_pool.size().await > 500 {
            self.buffer_pool.clear().await;
        }
        
        if self.string_pool.size().await > 500 {
            self.string_pool.clear().await;
        }

        Ok(freed_chunks)
    }

    /// Get comprehensive memory statistics
    pub async fn get_memory_stats(&self) -> MemoryManagerStats {
        let memory_stats = self.memory_pool.get_usage_stats().await;
        let buffer_pool_size = self.buffer_pool.size().await;
        let string_pool_size = self.string_pool.size().await;

        MemoryManagerStats {
            memory_pool: memory_stats,
            buffer_pool_size,
            string_pool_size,
        }
    }
}

/// Comprehensive memory manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagerStats {
    pub memory_pool: MemoryUsageStats,
    pub buffer_pool_size: usize,
    pub string_pool_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_pool() {
        let config = MemoryPoolConfig::default();
        let pool = MemoryPool::new(config).await.unwrap();

        // Test allocation
        let chunk1 = pool.allocate(1024).await.unwrap();
        assert_eq!(chunk1.size, 4096); // Should be chunk_size

        // Test deallocation
        pool.deallocate(chunk1).await.unwrap();

        // Test usage stats
        let stats = pool.get_usage_stats().await;
        assert!(stats.total_chunks > 0);
    }

    #[tokio::test]
    async fn test_object_pool() {
        let pool = ObjectPool::new(|| Vec::<u8>::with_capacity(1024), 10);

        // Get object from pool
        let mut obj1 = pool.get().await;
        obj1.push(42);
        assert_eq!(obj1.len(), 1);

        // Drop object (should return to pool)
        drop(obj1);

        // Get another object (should reuse the previous one)
        let obj2 = pool.get().await;
        assert_eq!(obj2.capacity(), 1024);
    }

    #[tokio::test]
    async fn test_buffer_pool() {
        let pool = BufferPool::new_with_capacity(1024, 10);

        let mut buffer = pool.get().await;
        buffer.extend_from_slice(b"hello world");
        assert_eq!(buffer.len(), 11);

        drop(buffer);

        // Pool should have one object now
        assert_eq!(pool.size().await, 1);
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let config = MemoryPoolConfig::default();
        let manager = MemoryManager::new(config).await.unwrap();

        // Test getting pools
        let buffer_pool = manager.buffer_pool();
        let _buffer = buffer_pool.get().await;

        // Test stats
        let stats = manager.get_memory_stats().await;
        assert!(stats.memory_pool.total_chunks > 0);
    }
}