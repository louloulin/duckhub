//! Common types used across DuckHub

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: Uuid,
    pub name: String,
    pub source_type: DataSourceType,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
}

/// Supported data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DataSourceType {
    /// Database connection
    Database(DatabaseConfig),
    /// REST API endpoint
    RestApi(ApiConfig),
    /// WebSocket connection
    WebSocket(WebSocketConfig),
    /// File system
    FileSystem(FileSystemConfig),
    /// Message queue
    MessageQueue(MessageQueueConfig),
    /// Object storage (S3, Azure Blob, GCS)
    ObjectStorage(ObjectStorageConfig),
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub driver: DatabaseDriver,
    pub pool_size: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

/// Supported database drivers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseDriver {
    PostgreSQL,
    MySQL,
    SQLite,
    DuckDB,
    ClickHouse,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub auth: Option<AuthConfig>,
    pub headers: HashMap<String, String>,
    pub timeout_seconds: Option<u64>,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub url: String,
    pub auth: Option<AuthConfig>,
    pub reconnect: bool,
    pub heartbeat_interval: Option<u64>,
}

/// File system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemConfig {
    pub path: String,
    pub format: FileFormat,
    pub compression: Option<CompressionType>,
    pub watch: bool,
}

/// Message queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueueConfig {
    pub broker_urls: Vec<String>,
    pub topic: String,
    pub consumer_group: Option<String>,
    pub auth: Option<AuthConfig>,
}

/// Object storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    pub provider: ObjectStorageProvider,
    pub bucket: String,
    pub prefix: Option<String>,
    pub credentials: StorageCredentials,
}

/// Object storage providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectStorageProvider {
    S3,
    AzureBlob,
    GoogleCloudStorage,
    MinIO,
}

/// Storage credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCredentials {
    pub access_key: String,
    pub secret_key: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub credentials: serde_json::Value,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Basic,
    Bearer,
    ApiKey,
    OAuth2,
    Custom,
}

/// Supported file formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileFormat {
    CSV,
    JSON,
    Parquet,
    ORC,
    Avro,
    Arrow,
}

/// Compression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Snappy,
    LZ4,
    Zstd,
    Brotli,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schema: Schema,
    pub source_id: Uuid,
    pub partition_config: Option<PartitionConfig>,
    pub metadata: DatasetMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub fields: Vec<Field>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<Index>,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
}

/// Data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    String,
    Binary,
    Date,
    Time,
    Timestamp,
    Decimal { precision: u8, scale: u8 },
    Array(Box<DataType>),
    Struct(Vec<Field>),
    Map { key: Box<DataType>, value: Box<DataType> },
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub unique: bool,
}

/// Index types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    Bitmap,
    FullText,
}

/// Partition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub strategy: PartitionStrategy,
    pub columns: Vec<String>,
    pub partition_size: Option<u64>,
}

/// Partition strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    Range,
    Hash,
    List,
    Time { interval: TimeInterval },
}

/// Time intervals for time-based partitioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInterval {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub row_count: Option<u64>,
    pub size_bytes: Option<u64>,
    pub last_updated: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Query definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub id: Uuid,
    pub sql: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub timeout_seconds: Option<u64>,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query_id: Uuid,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: u64,
    pub execution_time_ms: u64,
    pub metadata: QueryMetadata,
}

/// Query execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub bytes_scanned: Option<u64>,
    pub bytes_returned: Option<u64>,
    pub cache_hit: bool,
    pub execution_plan: Option<String>,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}
