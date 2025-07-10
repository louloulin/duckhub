//! Core traits for DuckHub components

use crate::{DuckHubError, Query, QueryResult, Result, Schema};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Trait for database connections and query execution
#[async_trait]
pub trait DatabaseEngine: Send + Sync {
    /// Execute a SQL query
    async fn execute_query(&self, query: &Query) -> Result<QueryResult>;

    /// Execute multiple queries in a batch
    async fn execute_batch(&self, queries: Vec<Query>) -> Result<Vec<QueryResult>>;

    /// Get table schema
    async fn get_schema(&self, table_name: &str) -> Result<Schema>;

    /// Check if table exists
    async fn table_exists(&self, table_name: &str) -> Result<bool>;

    /// Create table from schema
    async fn create_table(&self, table_name: &str, schema: &Schema) -> Result<()>;

    /// Drop table
    async fn drop_table(&self, table_name: &str) -> Result<()>;

    /// Get connection health status
    async fn health_check(&self) -> Result<()>;
}

/// Trait for data source connections
#[async_trait]
pub trait DataSourceConnector: Send + Sync {
    /// Connect to the data source
    async fn connect(&self) -> Result<Box<dyn DataConnection>>;

    /// Test connection without establishing it
    async fn test_connection(&self) -> Result<()>;

    /// Get supported operations
    fn supported_operations(&self) -> Vec<DataOperation>;
}

/// Trait for active data connections
#[async_trait]
pub trait DataConnection: Send + Sync {
    /// Fetch data with optional query/filter
    async fn fetch_data(&self, query: Option<&str>) -> Result<DataStream>;

    /// Get schema information
    async fn get_schema(&self) -> Result<Schema>;

    /// Close the connection
    async fn close(&self) -> Result<()>;
}

/// Data stream for streaming results
#[async_trait]
pub trait DataStream: Send + Sync {
    /// Get next batch of data
    async fn next_batch(&mut self) -> Result<Option<DataBatch>>;

    /// Get total estimated rows (if available)
    fn estimated_rows(&self) -> Option<u64>;

    /// Check if stream is exhausted
    fn is_exhausted(&self) -> bool;
}

/// Batch of data rows
#[derive(Debug, Clone)]
pub struct DataBatch {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub row_count: usize,
}

/// Supported data operations
#[derive(Debug, Clone, PartialEq)]
pub enum DataOperation {
    Read,
    Write,
    Stream,
    Batch,
    Schema,
}

/// Trait for caching implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set value with optional TTL
    async fn set(&self, key: &str, value: &[u8], ttl: Option<u64>) -> Result<()>;

    /// Delete key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Set TTL for existing key
    async fn expire(&self, key: &str, ttl: u64) -> Result<()>;

    /// Get multiple keys
    async fn mget(&self, keys: &[String]) -> Result<Vec<Option<Vec<u8>>>>;

    /// Set multiple key-value pairs
    async fn mset(&self, pairs: &[(String, Vec<u8>)]) -> Result<()>;

    /// Clear all cached data
    async fn clear(&self) -> Result<()>;
}

/// Trait for configuration management
pub trait ConfigProvider: Send + Sync {
    /// Get configuration value
    fn get<T>(&self, key: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned;

    /// Get configuration value with default
    fn get_or_default<T>(&self, key: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned;

    /// Check if configuration key exists
    fn has(&self, key: &str) -> bool;

    /// Get all configuration as a map
    fn get_all(&self) -> Result<HashMap<String, Value>>;
}

/// Trait for metrics collection
pub trait MetricsCollector: Send + Sync {
    /// Increment counter
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);

    /// Record histogram value
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Set gauge value
    fn set_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Record timing
    fn record_timing(&self, name: &str, duration_ms: f64, labels: &[(&str, &str)]);
}

/// Trait for health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Check component health
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get component name
    fn component_name(&self) -> &str;
}

/// Health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded { message: String },
    Unhealthy { message: String },
}

/// Trait for object storage operations
#[async_trait]
pub trait ObjectStorage: Send + Sync {
    /// Upload object
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Download object
    async fn get_object(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete object
    async fn delete_object(&self, key: &str) -> Result<()>;

    /// Check if object exists
    async fn object_exists(&self, key: &str) -> Result<bool>;

    /// List objects with prefix
    async fn list_objects(&self, prefix: &str) -> Result<Vec<ObjectInfo>>;

    /// Get object metadata
    async fn get_object_metadata(&self, key: &str) -> Result<ObjectMetadata>;
}

/// Object information
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,
}

/// Object metadata
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub size: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Trait for query optimization
pub trait QueryOptimizer: Send + Sync {
    /// Optimize a SQL query
    fn optimize(&self, sql: &str) -> Result<OptimizedQuery>;

    /// Get query execution plan
    fn explain(&self, sql: &str) -> Result<ExecutionPlan>;

    /// Estimate query cost
    fn estimate_cost(&self, sql: &str) -> Result<QueryCost>;
}

/// Optimized query result
#[derive(Debug, Clone)]
pub struct OptimizedQuery {
    pub original_sql: String,
    pub optimized_sql: String,
    pub optimizations_applied: Vec<String>,
    pub estimated_improvement: Option<f64>,
}

/// Query execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub plan_text: String,
    pub estimated_cost: QueryCost,
    pub operations: Vec<PlanOperation>,
}

/// Plan operation
#[derive(Debug, Clone)]
pub struct PlanOperation {
    pub operation_type: String,
    pub table_name: Option<String>,
    pub estimated_rows: Option<u64>,
    pub estimated_cost: f64,
}

/// Query cost estimation
#[derive(Debug, Clone)]
pub struct QueryCost {
    pub cpu_cost: f64,
    pub io_cost: f64,
    pub memory_cost: f64,
    pub network_cost: f64,
    pub total_cost: f64,
}

/// Trait for schema management
#[async_trait]
pub trait SchemaManager: Send + Sync {
    /// Register a new schema
    async fn register_schema(&self, name: &str, schema: &Schema) -> Result<()>;

    /// Get schema by name
    async fn get_schema(&self, name: &str) -> Result<Option<Schema>>;

    /// Update existing schema
    async fn update_schema(&self, name: &str, schema: &Schema) -> Result<()>;

    /// Delete schema
    async fn delete_schema(&self, name: &str) -> Result<()>;

    /// List all schemas
    async fn list_schemas(&self) -> Result<Vec<String>>;

    /// Validate schema compatibility
    async fn validate_compatibility(&self, old_schema: &Schema, new_schema: &Schema) -> Result<bool>;
}
