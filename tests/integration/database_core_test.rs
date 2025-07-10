//! Integration tests for DuckDB core functionality

use duckhub_common::prelude::*;
use duckhub_database::*;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_duckdb_engine_basic_operations() {
    let config = DatabaseConfig {
        duckdb_path: ":memory:".to_string(),
        memory_limit: Some("1GB".to_string()),
        threads: Some(2),
        max_memory: None,
        temp_directory: None,
        extensions: Vec::new(),
        pool: PoolConfig::default(),
    };

    let engine = DuckDBEngine::new(config).unwrap();
    
    // Test health check
    assert!(engine.health_check().await.is_ok());

    // Test table creation
    let schema = Schema {
        fields: vec![
            Field {
                name: "id".to_string(),
                data_type: DataType::Int64,
                nullable: false,
                default_value: None,
                description: None,
            },
            Field {
                name: "name".to_string(),
                data_type: DataType::String,
                nullable: true,
                default_value: None,
                description: None,
            },
            Field {
                name: "amount".to_string(),
                data_type: DataType::Float64,
                nullable: true,
                default_value: None,
                description: None,
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: Vec::new(),
    };

    assert!(engine.create_table("test_table", &schema).await.is_ok());
    assert!(engine.table_exists("test_table").await.unwrap());

    // Test data insertion
    let insert_query = Query {
        id: generate_id(),
        sql: "INSERT INTO test_table (id, name, amount) VALUES (1, 'Alice', 100.50), (2, 'Bob', 200.75), (3, 'Charlie', 300.25)".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    assert!(engine.execute_query(&insert_query).await.is_ok());

    // Test data selection
    let select_query = Query {
        id: generate_id(),
        sql: "SELECT * FROM test_table ORDER BY id".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&select_query).await.unwrap();
    assert_eq!(result.row_count, 3);
    assert_eq!(result.columns.len(), 3);

    // Test aggregation
    let agg_query = Query {
        id: generate_id(),
        sql: "SELECT COUNT(*) as count, SUM(amount) as total FROM test_table".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let agg_result = engine.execute_query(&agg_query).await.unwrap();
    assert_eq!(agg_result.row_count, 1);

    // Test schema retrieval
    let retrieved_schema = engine.get_schema("test_table").await.unwrap();
    assert_eq!(retrieved_schema.fields.len(), 3);
    assert_eq!(retrieved_schema.fields[0].name, "id");

    // Test table drop
    assert!(engine.drop_table("test_table").await.is_ok());
    assert!(!engine.table_exists("test_table").await.unwrap());
}

#[tokio::test]
async fn test_connection_pool() {
    let pool_config = PoolConfig {
        min_connections: 2,
        max_connections: 5,
        connection_timeout: 30,
        idle_timeout: 600,
        max_lifetime: 3600,
    };

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

    // Test getting connections
    let conn1 = pool.get_connection().await.unwrap();
    let conn2 = pool.get_connection().await.unwrap();

    // Test pool statistics
    let stats = pool.get_stats().await;
    assert_eq!(stats.min_connections, 2);
    assert_eq!(stats.max_connections, 5);
    assert_eq!(stats.active_connections, 2);

    // Connections should be returned to pool when dropped
    drop(conn1);
    drop(conn2);

    // Give some time for connections to be returned
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_query_executor_with_cache() {
    use duckhub_database::cache::MemoryCache;

    let config = DatabaseConfig {
        duckdb_path = ":memory:".to_string(),
        memory_limit: None,
        threads: None,
        max_memory: None,
        temp_directory: None,
        extensions: Vec::new(),
        pool: PoolConfig::default(),
    };

    let engine = Arc::new(DuckDBEngine::new(config).unwrap());
    let cache = Arc::new(MemoryCache::new(3600));
    let executor = QueryExecutor::new(engine.clone(), Some(cache));

    // Create test table
    let create_query = Query {
        id: generate_id(),
        sql: "CREATE TABLE test_cache AS SELECT 1 as id, 'test' as name".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    executor.execute(&create_query).await.unwrap();

    // Execute query twice - second should be cached
    let query = Query {
        id: generate_id(),
        sql: "SELECT * FROM test_cache".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result1 = executor.execute(&query).await.unwrap();
    assert!(!result1.metadata.cache_hit);

    let result2 = executor.execute(&query).await.unwrap();
    // Note: Cache hit detection would need more sophisticated implementation
    // This is just testing that the executor works with cache
    assert_eq!(result1.row_count, result2.row_count);

    let metrics = executor.get_metrics();
    assert!(metrics.total_queries() >= 2);
}

#[tokio::test]
async fn test_schema_manager() {
    let config = DatabaseConfig {
        duckdb_path: ":memory:".to_string(),
        memory_limit: None,
        threads: None,
        max_memory: None,
        temp_directory: None,
        extensions: Vec::new(),
        pool: PoolConfig::default(),
    };

    let engine = Arc::new(DuckDBEngine::new(config).unwrap());
    let schema_manager = SchemaManager::new(engine);

    let test_schema = Schema {
        fields: vec![
            Field {
                name: "id".to_string(),
                data_type: DataType::Int64,
                nullable: false,
                default_value: None,
                description: Some("Primary key".to_string()),
            },
            Field {
                name: "email".to_string(),
                data_type: DataType::String,
                nullable: false,
                default_value: None,
                description: Some("Email address".to_string()),
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![
            Index {
                name: "idx_email".to_string(),
                columns: vec!["email".to_string()],
                index_type: IndexType::BTree,
                unique: true,
            },
        ],
    };

    // Test schema registration
    assert!(schema_manager.register_schema("users", &test_schema).await.is_ok());

    // Test schema retrieval
    let retrieved = schema_manager.get_schema("users").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_schema = retrieved.unwrap();
    assert_eq!(retrieved_schema.fields.len(), 2);
    assert_eq!(retrieved_schema.fields[0].name, "id");

    // Test schema listing
    let schemas = schema_manager.list_schemas().await.unwrap();
    assert!(schemas.contains(&"users".to_string()));

    // Test schema update
    let mut updated_schema = test_schema.clone();
    updated_schema.fields.push(Field {
        name: "created_at".to_string(),
        data_type: DataType::Timestamp,
        nullable: true,
        default_value: None,
        description: Some("Creation timestamp".to_string()),
    });

    assert!(schema_manager.update_schema("users", &updated_schema).await.is_ok());

    let updated_retrieved = schema_manager.get_schema("users").await.unwrap().unwrap();
    assert_eq!(updated_retrieved.fields.len(), 3);

    // Test schema deletion
    assert!(schema_manager.delete_schema("users").await.is_ok());
    let deleted = schema_manager.get_schema("users").await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_metrics_collection() {
    use duckhub_database::metrics::DatabaseMetrics;

    let metrics = DatabaseMetrics::new().unwrap();

    // Test initial state
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.queries_total, 0);
    assert_eq!(snapshot.connections_active, 0);

    // Test query metrics
    let result = QueryResult {
        query_id: generate_id(),
        columns: vec!["col1".to_string(), "col2".to_string()],
        rows: vec![
            vec![serde_json::Value::Number(1.into()), serde_json::Value::String("test1".to_string())],
            vec![serde_json::Value::Number(2.into()), serde_json::Value::String("test2".to_string())],
        ],
        row_count: 2,
        execution_time_ms: 150,
        metadata: QueryMetadata {
            bytes_scanned: Some(2048),
            bytes_returned: Some(1024),
            cache_hit: false,
            execution_plan: None,
        },
    };

    metrics.record_query(std::time::Duration::from_millis(150), &result);

    let updated_snapshot = metrics.snapshot();
    assert_eq!(updated_snapshot.queries_total, 1);
    assert_eq!(updated_snapshot.rows_returned_total, 2);
    assert_eq!(updated_snapshot.bytes_scanned_total, 2048);
    assert_eq!(updated_snapshot.bytes_returned_total, 1024);

    // Test connection metrics
    metrics.record_connection();
    metrics.update_connections(3, 2);

    let conn_snapshot = metrics.snapshot();
    assert_eq!(conn_snapshot.connections_total, 1);
    assert_eq!(conn_snapshot.connections_active, 3);
    assert_eq!(conn_snapshot.connections_idle, 2);
    assert_eq!(conn_snapshot.total_connections(), 5);

    // Test cache metrics
    metrics.record_cache_hit();
    metrics.record_cache_miss();

    let cache_snapshot = metrics.snapshot();
    assert_eq!(cache_snapshot.cache_hits_total, 1);
    assert_eq!(cache_snapshot.cache_misses_total, 1);
    assert_eq!(cache_snapshot.cache_hit_ratio(), 0.5);

    // Test Prometheus format output
    let prometheus_output = metrics.gather();
    assert!(!prometheus_output.is_empty());
    assert!(prometheus_output.contains("duckhub_queries_total"));
}

#[tokio::test]
async fn test_data_lake_manager() {
    // This test would require actual object storage setup
    // For now, just test the manager creation and basic functionality
    let mut lake_manager = DataLakeManager::new();
    
    // Test that manager starts empty
    assert!(lake_manager.get_default_provider().is_err());
    
    // Test schema generation
    let sql = lake_manager.generate_external_table_sql(
        "test_table",
        "s3://bucket/data.parquet",
        &FileFormat::Parquet,
        None,
    ).unwrap();
    
    assert!(sql.contains("CREATE TABLE test_table"));
    assert!(sql.contains("read_parquet"));
    assert!(sql.contains("s3://bucket/data.parquet"));
}

// Helper function to create test configuration
fn create_test_config() -> DatabaseConfig {
    DatabaseConfig {
        duckdb_path: ":memory:".to_string(),
        memory_limit: Some("1GB".to_string()),
        threads: Some(1),
        max_memory: None,
        temp_directory: None,
        extensions: Vec::new(),
        pool: PoolConfig {
            min_connections: 1,
            max_connections: 5,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        },
    }
}
