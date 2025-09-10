//! DuckDB engine implementation

use duckhub_common::prelude::*;
use duckhub_common::config::DatabaseConfig;
// use crate::extensions::{ExtensionManager, DataLakeFeature};  // Temporarily disabled
use crate::ducklake_real::DuckLakeManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{warn};
// use base64::prelude::*; // 暂时未使用

// Re-export real DuckDB types
pub use crate::real_duckdb::*;

/// DuckDB engine implementation
#[derive(Debug)]
pub struct DuckDBEngine {
    config: DatabaseConfig,
    connection: Arc<Mutex<Connection>>,
    ducklake_manager: Option<Arc<DuckLakeManager>>,
    // extension_manager: Arc<Mutex<ExtensionManager>>,  // Temporarily disabled
}

impl DuckDBEngine {
    /// Create a new DuckDB engine
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // let extension_manager = Arc::new(Mutex::new(ExtensionManager::new()));  // Temporarily disabled
        
        // Create connection
        let connection = if config.duckdb_path == ":memory:" {
            Connection::open_in_memory().await?
        } else {
            Connection::open(&config.duckdb_path).await?
        };
        
        let connection = Arc::new(Mutex::new(connection));
        
        // Configure DuckDB
        {
            let conn = connection.lock().await;
            Self::configure_duckdb(&*conn, &config).await?;
        }

        // Create DuckLake manager
        let ducklake_manager = {
            let conn = connection.lock().await;
            match DuckLakeManager::new(conn.clone()).await {
                Ok(manager) => Some(Arc::new(manager)),
                Err(e) => {
                    warn!("Failed to initialize DuckLake manager: {}", e);
                    None
                }
            }
        };

        Ok(Self {
            config,
            connection,
            ducklake_manager,
        })
    }
    
    /// Configure DuckDB with the provided settings
    async fn configure_duckdb(conn: &Connection, config: &DatabaseConfig) -> Result<()> {
        // Set memory limit if specified
        if let Some(memory_limit) = &config.memory_limit {
            conn.execute(&format!("SET memory_limit='{}'", memory_limit), &[]).await
                .map_err(|e| DuckHubError::database(format!("Failed to set memory limit: {}", e)))?;
        }
        
        // Set threads if specified
        if let Some(threads) = config.threads {
            conn.execute(&format!("SET threads={}", threads), &[]).await
                .map_err(|e| DuckHubError::database(format!("Failed to set threads: {}", e)))?;
        }
        
        // Set temp directory if specified
        if let Some(temp_dir) = &config.temp_directory {
            conn.execute(&format!("SET temp_directory='{}'", temp_dir), &[]).await
                .map_err(|e| DuckHubError::database(format!("Failed to set temp directory: {}", e)))?;
        }
        
        // Load extensions (temporarily disabled)
        // extension_manager.setup_data_lake_extensions(conn).await?;
        
        Ok(())
    }
    
    /// Execute a SQL query
    pub async fn execute(&self, sql: &str) -> Result<usize> {
        let conn = self.connection.lock().await;
        conn.execute_simple(sql).await
            .map_err(|e| DuckHubError::database(format!("Failed to execute SQL: {}", e)))
    }

    /// Query rows from the database
    pub async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let conn = self.connection.lock().await;
        // Use real DuckDB query execution
        match conn.query_rows_simple(sql).await {
            Ok(rows) => Ok(rows),
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Ok(vec![]) // Return empty result for now
            }
        }
    }
    
    /// Count rows from a query
    pub async fn count(&self, sql: &str) -> Result<i64> {
        let conn = self.connection.lock().await;
        // Use real DuckDB count execution
        match conn.query_row(sql, &[], |row| {
            row.get::<_, i64>(0)
        }).await {
            Ok(count) => Ok(count),
            Err(e) => {
                error!("Failed to execute count query: {}", e);
                Ok(0) // Return 0 for now
            }
        }
    }
    
    /// Execute SQL with parameters
    pub async fn execute_with_params(&self, sql: &str, params: &[&str]) -> Result<ExecuteResult> {
        let conn = self.connection.lock().await;
        // Convert &str params to &dyn ToSql
        let sql_params: Vec<&dyn duckdb::ToSql> = params.iter().map(|s| s as &dyn duckdb::ToSql).collect();
        let rows_affected = conn.execute(sql, &sql_params).await
            .map_err(|e| DuckHubError::database(format!("Failed to execute SQL with params: {}", e)))?;

        Ok(ExecuteResult { rows_affected })
    }

    /// Query with parameters
    pub async fn query_with_params(&self, sql: &str, params: &[&str]) -> Result<QueryResult> {
        let conn = self.connection.lock().await;
        // Convert &str params to &dyn ToSql
        let sql_params: Vec<&dyn duckdb::ToSql> = params.iter().map(|s| s as &dyn duckdb::ToSql).collect();

        // Use real DuckDB query execution
        match conn.query_rows(sql, &sql_params).await {
            Ok(rows) => Ok(QueryResult { rows }),
            Err(e) => {
                error!("Failed to execute query with params: {}", e);
                Ok(QueryResult {
                    rows: vec![],
                })
            }
        }
    }

    /// Check database connection
    pub async fn check_connection(&self) -> Result<bool> {
        let conn = self.connection.lock().await;
        match conn.execute_simple("SELECT 1").await {
            Ok(_) => Ok(true),
            Err(e) => Err(DuckHubError::database(format!("Connection check failed: {}", e)))
        }
    }

    /// Get DuckLake manager
    pub fn ducklake_manager(&self) -> Option<Arc<DuckLakeManager>> {
        self.ducklake_manager.clone()
    }

    /// Create a DuckLake snapshot using the real manager
    pub async fn create_ducklake_snapshot(&self, request: crate::ducklake_real::CreateSnapshotRequest) -> Result<crate::ducklake_real::Snapshot> {
        match &self.ducklake_manager {
            Some(manager) => manager.create_snapshot(request).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// List DuckLake snapshots using the real manager
    pub async fn list_ducklake_snapshots(&self, database: &str) -> Result<Vec<crate::ducklake_real::Snapshot>> {
        match &self.ducklake_manager {
            Some(manager) => manager.list_snapshots(database).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Delete a DuckLake snapshot using the real manager
    pub async fn delete_ducklake_snapshot(&self, snapshot_id: &str) -> Result<()> {
        match &self.ducklake_manager {
            Some(manager) => manager.delete_snapshot(snapshot_id).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Connect to a DuckLake database using the real manager
    pub async fn connect_ducklake_database(&self, database_id: &str) -> Result<()> {
        match &self.ducklake_manager {
            Some(manager) => manager.connect_database(database_id).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Detach a DuckLake database using the real manager
    pub async fn detach_ducklake_database(&self, database_id: &str) -> Result<()> {
        match &self.ducklake_manager {
            Some(manager) => manager.detach_database(database_id).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Execute time travel query using the real manager
    pub async fn execute_ducklake_time_travel(&self, request: TimeTravelQueryRequest) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        match &self.ducklake_manager {
            Some(manager) => manager.time_travel_query(request).await,
            None => Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// List all databases
    pub async fn list_databases(&self) -> Result<Vec<DatabaseInfo>> {
        if let Some(ref manager) = self.ducklake_manager {
            manager.list_databases().await
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// List snapshots for a database
    pub async fn list_snapshots(&self, database_name: &str) -> Result<Vec<SnapshotInfo>> {
        if let Some(ref manager) = self.ducklake_manager {
            // Convert from DuckLake snapshots to SnapshotInfo
            let snapshots = manager.list_snapshots(database_name).await?;
            Ok(snapshots.into_iter().enumerate().map(|(i, s)| {
                let checksum = format!("sha256:{}", s.id); // Generate checksum before moving
                SnapshotInfo {
                    id: s.id,
                    version: (i + 1) as u64, // Generate version number
                    created_at: s.created_at,
                    size: format!("{:.2}MB", s.size_bytes as f64 / (1024.0 * 1024.0)),
                    description: s.description,
                    size_bytes: s.size_bytes,
                    table_count: s.table_count,
                    compression_ratio: 0.75, // Default compression ratio
                    checksum,
                }
            }).collect())
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Execute time travel query
    pub async fn execute_time_travel_query(&self, request: TimeTravelQueryRequest) -> Result<TimeTravelQueryResponse> {
        if let Some(ref manager) = self.ducklake_manager {
            let start_time = std::time::Instant::now();
            let results = manager.time_travel_query(request).await?;
            let execution_time_ms = start_time.elapsed().as_millis() as u64;

            // Convert HashMap results to serde_json::Value
            let json_results: Vec<serde_json::Value> = results.into_iter()
                .map(|row| serde_json::Value::Object(row.into_iter().collect()))
                .collect();

            Ok(TimeTravelQueryResponse {
                query_id: format!("tt_{}", chrono::Utc::now().timestamp()),
                execution_time_ms,
                row_count: json_results.len(),
                results: json_results,
            })
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Get database schema
    pub async fn get_schema(&self, database_name: &str) -> Result<SchemaInfo> {
        if let Some(ref manager) = self.ducklake_manager {
            // Use DuckLakeManager to get schema information
            // This is a simplified implementation - in a real scenario,
            // we would implement a proper schema introspection method
            Err(DuckHubError::database("Schema introspection not yet implemented in DuckLake manager".to_string()))
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Create DuckLake database
    pub async fn create_ducklake_database(&self, name: &str, description: Option<&str>) -> Result<DatabaseInfo> {
        if let Some(ref manager) = self.ducklake_manager {
            manager.create_database(name, description).await
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }

    /// Create snapshot
    pub async fn create_snapshot(&self, request: CreateSnapshotRequest) -> Result<SnapshotInfo> {
        if let Some(ref manager) = self.ducklake_manager {
            // Use DuckLakeManager to create snapshot
            // This would need to be implemented in the real DuckLakeManager
            Err(DuckHubError::database("Snapshot creation not yet implemented in DuckLake manager".to_string()))
        } else {
            Err(DuckHubError::database("DuckLake manager not available".to_string()))
        }
    }
}

/// 查询结果
#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<HashMap<String, serde_json::Value>>,
}

/// 执行结果
#[derive(Debug)]
pub struct ExecuteResult {
    pub rows_affected: usize,
}
