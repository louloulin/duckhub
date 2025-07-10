//! Connection pool management for DuckDB

use duckhub_common::prelude::*;
use crate::duckdb::{DuckDBEngine, DuckDBConnection};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn, instrument};

/// Connection pool for managing DuckDB connections
pub struct ConnectionPool {
    connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    config: PoolConfig,
    database_config: DatabaseConfig,
}

/// A pooled connection wrapper
pub struct PooledConnection {
    connection: DuckDBConnection,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
}

impl PooledConnection {
    fn new(connection: DuckDBConnection) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    fn is_idle(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }

    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: PoolConfig, database_config: DatabaseConfig) -> Result<Self> {
        let pool = Self {
            connections: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(config.max_connections as usize)),
            config,
            database_config,
        };

        // Initialize minimum connections
        pool.initialize_connections().await?;

        // Start background maintenance task
        pool.start_maintenance_task();

        Ok(pool)
    }

    /// Initialize minimum connections
    async fn initialize_connections(&self) -> Result<()> {
        let mut connections = self.connections.lock().await;
        
        for _ in 0..self.config.min_connections {
            let conn = self.create_connection().await?;
            connections.push_back(PooledConnection::new(conn));
        }

        info!("Initialized {} connections in pool", self.config.min_connections);
        Ok(())
    }

    /// Create a new database connection
    async fn create_connection(&self) -> Result<DuckDBConnection> {
        DuckDBConnection::new(&self.database_config.duckdb_path)
    }

    /// Get a connection from the pool
    #[instrument(skip(self))]
    pub async fn get_connection(&self) -> Result<PooledConnectionGuard> {
        // Acquire semaphore permit
        let permit = self.semaphore.clone()
            .acquire_owned()
            .await
            .map_err(|_| DuckHubError::database("Failed to acquire connection permit"))?;

        let mut connections = self.connections.lock().await;

        // Try to get an existing connection
        if let Some(mut conn) = connections.pop_front() {
            // Check if connection is still valid
            if !conn.is_expired(Duration::from_secs(self.config.max_lifetime)) {
                conn.mark_used();
                debug!("Reusing existing connection");
                return Ok(PooledConnectionGuard::new(conn, permit, self.connections.clone()));
            } else {
                debug!("Connection expired, creating new one");
            }
        }

        // Create new connection if none available or expired
        drop(connections); // Release lock before creating connection
        
        let new_conn = self.create_connection().await?;
        let pooled_conn = PooledConnection::new(new_conn);
        
        debug!("Created new connection");
        Ok(PooledConnectionGuard::new(pooled_conn, permit, self.connections.clone()))
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let connections = self.connections.lock().await;
        let available_connections = connections.len();
        let active_connections = self.config.max_connections - self.semaphore.available_permits() as u32;

        PoolStats {
            total_connections: self.config.max_connections,
            active_connections,
            available_connections: available_connections as u32,
            min_connections: self.config.min_connections,
            max_connections: self.config.max_connections,
        }
    }

    /// 获取活跃连接数
    pub fn active_connections(&self) -> usize {
        (self.config.max_connections - self.semaphore.available_permits() as u32) as usize
    }

    /// 获取空闲连接数
    pub async fn idle_connections(&self) -> usize {
        let connections = self.connections.lock().await;
        connections.len()
    }

    /// 获取最大连接数
    pub fn max_connections(&self) -> usize {
        self.config.max_connections as usize
    }

    /// 获取等待队列长度
    pub fn queue_length(&self) -> usize {
        // 这里返回等待获取连接的线程数
        // 由于Semaphore没有直接提供等待队列长度，我们使用一个近似值
        let active = self.active_connections();
        let max = self.max_connections();
        if active >= max {
            // 如果活跃连接数达到最大值，可能有等待的请求
            // 这里返回一个估计值，实际实现可能需要更复杂的跟踪
            0 // 简化实现
        } else {
            0
        }
    }

    /// Start background maintenance task
    fn start_maintenance_task(&self) {
        let connections = self.connections.clone();
        let config = self.config.clone();
        let database_config = self.database_config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Run every minute
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::maintain_pool(&connections, &config, &database_config).await {
                    error!("Pool maintenance failed: {}", e);
                }
            }
        });
    }

    /// Maintain the connection pool
    async fn maintain_pool(
        connections: &Arc<Mutex<VecDeque<PooledConnection>>>,
        config: &PoolConfig,
        database_config: &DatabaseConfig,
    ) -> Result<()> {
        let mut conns = connections.lock().await;
        let mut removed_count = 0;

        // Remove expired and idle connections
        conns.retain(|conn| {
            let expired = conn.is_expired(Duration::from_secs(config.max_lifetime));
            let idle = conn.is_idle(Duration::from_secs(config.idle_timeout));
            
            if expired || (idle && conns.len() > config.min_connections as usize) {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        if removed_count > 0 {
            debug!("Removed {} expired/idle connections", removed_count);
        }

        // Ensure minimum connections
        while conns.len() < config.min_connections as usize {
            match DuckDBConnection::new(&database_config.duckdb_path) {
                Ok(conn) => {
                    conns.push_back(PooledConnection::new(conn));
                    debug!("Added connection to maintain minimum pool size");
                }
                Err(e) => {
                    error!("Failed to create connection during maintenance: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Connection guard that returns connection to pool when dropped
pub struct PooledConnectionGuard {
    connection: Option<PooledConnection>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    pool_connections: Arc<Mutex<VecDeque<PooledConnection>>>,
}

impl PooledConnectionGuard {
    fn new(
        connection: PooledConnection,
        permit: tokio::sync::OwnedSemaphorePermit,
        pool_connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    ) -> Self {
        Self {
            connection: Some(connection),
            _permit: permit,
            pool_connections,
        }
    }

    /// Get reference to the underlying connection
    pub fn connection(&self) -> &DuckDBConnection {
        &self.connection.as_ref().unwrap().connection
    }

    /// Get mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut DuckDBConnection {
        &mut self.connection.as_mut().unwrap().connection
    }
}

impl Drop for PooledConnectionGuard {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let pool_connections = self.pool_connections.clone();
            
            // Return connection to pool asynchronously
            tokio::spawn(async move {
                let mut connections = pool_connections.lock().await;
                connections.push_back(connection);
            });
        }
    }
}

/// Pool manager for creating and managing multiple pools
pub struct PoolManager {
    pools: dashmap::DashMap<String, Arc<ConnectionPool>>,
}

impl PoolManager {
    /// Create a new pool manager
    pub fn new() -> Self {
        Self {
            pools: dashmap::DashMap::new(),
        }
    }

    /// Get or create a connection pool
    pub async fn get_pool(
        &self,
        name: &str,
        config: PoolConfig,
        database_config: DatabaseConfig,
    ) -> Result<Arc<ConnectionPool>> {
        if let Some(pool) = self.pools.get(name) {
            return Ok(pool.clone());
        }

        let pool = Arc::new(ConnectionPool::new(config, database_config).await?);
        self.pools.insert(name.to_string(), pool.clone());
        
        info!("Created new connection pool: {}", name);
        Ok(pool)
    }

    /// Remove a connection pool
    pub fn remove_pool(&self, name: &str) -> Option<Arc<ConnectionPool>> {
        self.pools.remove(name).map(|(_, pool)| pool)
    }

    /// Get all pool names
    pub fn pool_names(&self) -> Vec<String> {
        self.pools.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get statistics for all pools
    pub async fn get_all_stats(&self) -> HashMap<String, PoolStats> {
        let mut stats = HashMap::new();
        
        for entry in self.pools.iter() {
            let name = entry.key().clone();
            let pool = entry.value();
            stats.insert(name, pool.get_stats().await);
        }
        
        stats
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub available_connections: u32,
    pub min_connections: u32,
    pub max_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let pool_config = PoolConfig::default();
        let db_config = DatabaseConfig {
            duckdb_path: ":memory:".to_string(),
            memory_limit: None,
            threads: None,
            max_memory: None,
            temp_directory: None,
            extensions: Vec::new(),
            pool: pool_config.clone(),
        };

        let pool = ConnectionPool::new(pool_config, db_config).await.unwrap();
        let stats = pool.get_stats().await;
        
        assert_eq!(stats.min_connections, 1);
        assert_eq!(stats.max_connections, 10);
    }

    #[tokio::test]
    async fn test_get_connection() {
        let pool_config = PoolConfig::default();
        let db_config = DatabaseConfig {
            duckdb_path: ":memory:".to_string(),
            memory_limit: None,
            threads: None,
            max_memory: None,
            temp_directory: None,
            extensions: Vec::new(),
            pool: pool_config.clone(),
        };

        let pool = ConnectionPool::new(pool_config, db_config).await.unwrap();
        let _conn = pool.get_connection().await.unwrap();
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.active_connections, 1);
    }
}
