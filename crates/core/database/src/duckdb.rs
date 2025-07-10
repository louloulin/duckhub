//! DuckDB engine implementation

use duckhub_common::prelude::*;
use crate::extensions::{ExtensionManager, DataLakeFeature};
use duckdb::{Connection, ToSql};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};
use base64::prelude::*;

/// DuckDB database engine implementation
pub struct DuckDBEngine {
    connection: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
    extension_manager: Arc<Mutex<ExtensionManager>>,
}

impl DuckDBEngine {
    /// Create a new DuckDB engine instance
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        let connection = if config.duckdb_path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(&config.duckdb_path)
        }
        .map_err(|e| DuckHubError::database(format!("Failed to open DuckDB: {}", e)))?;

        // Configure DuckDB settings and extensions
        let mut extension_manager = ExtensionManager::new();
        Self::configure_duckdb(&connection, &config, &mut extension_manager).await?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            config,
            extension_manager: Arc::new(Mutex::new(extension_manager)),
        })
    }

    /// Configure DuckDB with optimal settings and extensions
    async fn configure_duckdb(conn: &Connection, config: &DatabaseConfig, extension_manager: &mut ExtensionManager) -> Result<()> {
        // Set memory limit if specified
        if let Some(memory_limit) = &config.memory_limit {
            conn.execute(&format!("SET memory_limit='{}'", memory_limit), [])
                .map_err(|e| DuckHubError::database(format!("Failed to set memory limit: {}", e)))?;
        }

        // Set thread count if specified
        if let Some(threads) = config.threads {
            conn.execute(&format!("SET threads={}", threads), [])
                .map_err(|e| DuckHubError::database(format!("Failed to set threads: {}", e)))?;
        }

        // Set temp directory if specified
        if let Some(temp_dir) = &config.temp_directory {
            conn.execute(&format!("SET temp_directory='{}'", temp_dir), [])
                .map_err(|e| DuckHubError::database(format!("Failed to set temp directory: {}", e)))?;
        }

        // Setup data lake extensions using the extension manager
        extension_manager.setup_data_lake_extensions(conn).await?;

        // Load any additional extensions specified in config
        for extension_name in &config.extensions {
            if !extension_manager.is_extension_available(extension_name) {
                // Try to install and load the extension
                if let Err(e) = conn.execute(&format!("INSTALL '{}'", extension_name), []) {
                    debug!("Extension {} might already be installed: {}", extension_name, e);
                }

                conn.execute(&format!("LOAD '{}'", extension_name), [])
                    .map_err(|e| DuckHubError::database(format!("Failed to load extension {}: {}", extension_name, e)))?;

                info!("Successfully loaded additional extension: {}", extension_name);
            }
        }

        // Enable optimizations
        conn.execute("SET enable_optimizer=true", [])
            .map_err(|e| DuckHubError::database(format!("Failed to enable optimizer: {}", e)))?;

        conn.execute("SET enable_profiling=true", [])
            .map_err(|e| DuckHubError::database(format!("Failed to enable profiling: {}", e)))?;

        Ok(())
    }

    /// Execute a raw SQL statement
    pub async fn execute_raw(&self, sql: &str) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.execute(sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to execute SQL: {}", e)))?;
        Ok(())
    }

    /// Prepare a statement for repeated execution
    pub async fn prepare(&self, sql: &str) -> Result<DuckDBStatement> {
        let conn = self.connection.lock().await;
        let stmt = conn.prepare(sql)
            .map_err(|e| DuckHubError::database(format!("Failed to prepare statement: {}", e)))?;
        Ok(DuckDBStatement::new(stmt))
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.connection.lock().await;
        
        // Get memory usage
        let mut stmt = conn.prepare("SELECT * FROM pragma_database_size()")
            .map_err(|e| DuckHubError::database(format!("Failed to prepare stats query: {}", e)))?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })
        .map_err(|e| DuckHubError::database(format!("Failed to execute stats query: {}", e)))?;

        let mut total_size = 0i64;
        for row in rows {
            let (_, size) = row
                .map_err(|e| DuckHubError::database(format!("Failed to read stats row: {}", e)))?;
            total_size += size;
        }

        Ok(DatabaseStats {
            total_size_bytes: total_size as u64,
            table_count: self.get_table_count().await?,
            connection_count: 1, // Single connection for now
        })
    }

    /// Get count of tables in database
    async fn get_table_count(&self) -> Result<u32> {
        let conn = self.connection.lock().await;
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'main'")
            .map_err(|e| DuckHubError::database(format!("Failed to prepare table count query: {}", e)))?;

        let count: i64 = stmt.query_row([], |row| row.get(0))
            .map_err(|e| DuckHubError::database(format!("Failed to get table count: {}", e)))?;

        Ok(count as u32)
    }

    /// Get extension manager
    pub async fn extension_manager(&self) -> Arc<Mutex<ExtensionManager>> {
        self.extension_manager.clone()
    }

    /// Enable specific data lake features
    pub async fn enable_data_lake_features(&self, features: Vec<DataLakeFeature>) -> Result<()> {
        let conn = self.connection.lock().await;
        let mut ext_manager = self.extension_manager.lock().await;
        ext_manager.enable_features(&conn, features).await
    }

    /// Configure S3 access
    pub async fn configure_s3(&self, config: crate::extensions::S3Config) -> Result<()> {
        let conn = self.connection.lock().await;
        let ext_manager = self.extension_manager.lock().await;
        ext_manager.configure_s3(&conn, &config).await
    }

    /// Configure Azure Blob Storage access
    pub async fn configure_azure(&self, config: crate::extensions::AzureConfig) -> Result<()> {
        let conn = self.connection.lock().await;
        let ext_manager = self.extension_manager.lock().await;
        ext_manager.configure_azure(&conn, &config).await
    }

    /// Test data lake connectivity
    pub async fn test_data_lake_connectivity(&self) -> Result<crate::extensions::DataLakeConnectivityReport> {
        let conn = self.connection.lock().await;
        let ext_manager = self.extension_manager.lock().await;
        ext_manager.test_data_lake_connectivity(&conn).await
    }
}

#[async_trait]
impl DatabaseEngine for DuckDBEngine {
    #[instrument(skip(self, query))]
    async fn execute_query(&self, query: &Query) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();
        
        debug!("Executing query: {}", query.sql);
        
        let conn = self.connection.lock().await;
        let mut stmt = conn.prepare(&query.sql)
            .map_err(|e| DuckHubError::database(format!("Failed to prepare query: {}", e)))?;

        // Convert parameters to DuckDB format
        let _params = self.convert_parameters(&query.parameters)?;

        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            let column_count = row.as_ref().column_count();
            
            for i in 0..column_count {
                let value = match row.get_ref(i) {
                    Ok(val) => self.convert_value_to_json(val)?,
                    Err(e) => return Err(duckdb::Error::InvalidColumnType(i, e.to_string())),
                };
                values.push(value);
            }
            Ok(values)
        })
        .map_err(|e| DuckHubError::database(format!("Failed to execute query: {}", e)))?;

        let mut result_rows = Vec::new();
        let mut columns = Vec::new();
        
        // Get column names - we'll extract them from the statement metadata
        // For now, use generic column names
        if !result_rows.is_empty() {
            let column_count = result_rows[0].len();
            for i in 0..column_count {
                columns.push(format!("col_{}", i));
            }
        }

        // Collect all rows
        for row in rows {
            let row_data = row
                .map_err(|e| DuckHubError::database(format!("Failed to read row: {}", e)))?;
            result_rows.push(row_data);
        }

        let execution_time = start_time.elapsed();
        
        info!(
            "Query executed successfully in {}ms, returned {} rows",
            execution_time.as_millis(),
            result_rows.len()
        );

        Ok(QueryResult {
            query_id: query.id,
            columns,
            rows: result_rows,
            row_count: result_rows.len() as u64,
            execution_time_ms: execution_time.as_millis() as u64,
            metadata: QueryMetadata {
                bytes_scanned: None,
                bytes_returned: None,
                cache_hit: false,
                execution_plan: None,
            },
        })
    }

    async fn execute_batch(&self, queries: Vec<Query>) -> Result<Vec<QueryResult>> {
        let mut results = Vec::new();
        
        for query in queries {
            let result = self.execute_query(&query).await?;
            results.push(result);
        }
        
        Ok(results)
    }

    async fn get_schema(&self, table_name: &str) -> Result<Schema> {
        let conn = self.connection.lock().await;
        let sql = format!(
            "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
            table_name
        );
        
        let mut stmt = conn.prepare(&sql)
            .map_err(|e| DuckHubError::database(format!("Failed to prepare schema query: {}", e)))?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| DuckHubError::database(format!("Failed to execute schema query: {}", e)))?;

        let mut fields = Vec::new();
        for row in rows {
            let (column_name, data_type, is_nullable) = row
                .map_err(|e| DuckHubError::database(format!("Failed to read schema row: {}", e)))?;
            
            fields.push(Field {
                name: column_name,
                data_type: self.convert_duckdb_type(&data_type)?,
                nullable: is_nullable.to_lowercase() == "yes",
                default_value: None,
                description: None,
            });
        }

        Ok(Schema {
            fields,
            primary_key: None,
            indexes: Vec::new(),
        })
    }

    async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let conn = self.connection.lock().await;
        let sql = "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?";
        
        let mut stmt = conn.prepare(sql)
            .map_err(|e| DuckHubError::database(format!("Failed to prepare table exists query: {}", e)))?;
        
        let count: i64 = stmt.query_row([table_name], |row| row.get(0))
            .map_err(|e| DuckHubError::database(format!("Failed to check table existence: {}", e)))?;
        
        Ok(count > 0)
    }

    async fn create_table(&self, table_name: &str, schema: &Schema) -> Result<()> {
        let mut sql = format!("CREATE TABLE {} (", table_name);
        
        for (i, field) in schema.fields.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&format!(
                "{} {}{}",
                field.name,
                self.convert_to_duckdb_type(&field.data_type)?,
                if field.nullable { "" } else { " NOT NULL" }
            ));
        }
        
        sql.push(')');
        
        self.execute_raw(&sql).await
    }

    async fn drop_table(&self, table_name: &str) -> Result<()> {
        let sql = format!("DROP TABLE IF EXISTS {}", table_name);
        self.execute_raw(&sql).await
    }

    async fn health_check(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.execute("SELECT 1", [])
            .map_err(|e| DuckHubError::database(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

impl DuckDBEngine {
    /// Convert query parameters to DuckDB format
    fn convert_parameters(&self, _params: &HashMap<String, serde_json::Value>) -> Result<Vec<Box<dyn ToSql>>> {
        // For now, return empty params - parameter binding needs more work
        // TODO: Implement proper parameter conversion
        Ok(Vec::new())
    }

    /// Convert DuckDB value to JSON
    fn convert_value_to_json(&self, value: duckdb::types::ValueRef) -> Result<serde_json::Value> {
        use duckdb::types::ValueRef;
        
        let json_value = match value {
            ValueRef::Null => serde_json::Value::Null,
            ValueRef::Boolean(b) => serde_json::Value::Bool(b),
            ValueRef::TinyInt(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            ValueRef::SmallInt(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            ValueRef::Int(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            ValueRef::BigInt(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            ValueRef::Float(f) => serde_json::Value::Number(serde_json::Number::from_f64(f as f64).unwrap_or(serde_json::Number::from(0))),
            ValueRef::Double(f) => serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0))),
            ValueRef::Text(s) => serde_json::Value::String(s.to_string()),
            ValueRef::Blob(b) => serde_json::Value::String(base64::prelude::BASE64_STANDARD.encode(b)),
            _ => serde_json::Value::String(format!("{:?}", value)),
        };
        
        Ok(json_value)
    }

    /// Convert DuckDB type string to our DataType enum
    fn convert_duckdb_type(&self, duckdb_type: &str) -> Result<DataType> {
        let data_type = match duckdb_type.to_uppercase().as_str() {
            "BOOLEAN" => DataType::Boolean,
            "TINYINT" => DataType::Int8,
            "SMALLINT" => DataType::Int16,
            "INTEGER" | "INT" => DataType::Int32,
            "BIGINT" => DataType::Int64,
            "UTINYINT" => DataType::UInt8,
            "USMALLINT" => DataType::UInt16,
            "UINTEGER" => DataType::UInt32,
            "UBIGINT" => DataType::UInt64,
            "REAL" | "FLOAT" => DataType::Float32,
            "DOUBLE" => DataType::Float64,
            "VARCHAR" | "TEXT" | "STRING" => DataType::String,
            "BLOB" => DataType::Binary,
            "DATE" => DataType::Date,
            "TIME" => DataType::Time,
            "TIMESTAMP" => DataType::Timestamp,
            _ => DataType::String, // Default to string for unknown types
        };
        
        Ok(data_type)
    }

    /// Convert our DataType enum to DuckDB type string
    fn convert_to_duckdb_type(&self, data_type: &DataType) -> Result<String> {
        let duckdb_type = match data_type {
            DataType::Boolean => "BOOLEAN",
            DataType::Int8 => "TINYINT",
            DataType::Int16 => "SMALLINT",
            DataType::Int32 => "INTEGER",
            DataType::Int64 => "BIGINT",
            DataType::UInt8 => "UTINYINT",
            DataType::UInt16 => "USMALLINT",
            DataType::UInt32 => "UINTEGER",
            DataType::UInt64 => "UBIGINT",
            DataType::Float32 => "REAL",
            DataType::Float64 => "DOUBLE",
            DataType::String => "VARCHAR",
            DataType::Binary => "BLOB",
            DataType::Date => "DATE",
            DataType::Time => "TIME",
            DataType::Timestamp => "TIMESTAMP",
            DataType::Decimal { precision, scale } => {
                return Ok(format!("DECIMAL({}, {})", precision, scale));
            }
            _ => "VARCHAR", // Default to VARCHAR for complex types
        };
        
        Ok(duckdb_type.to_string())
    }
}

/// Wrapper for DuckDB prepared statement
pub struct DuckDBStatement {
    stmt: duckdb::Statement<'static>,
}

impl DuckDBStatement {
    fn new(stmt: duckdb::Statement<'static>) -> Self {
        Self { stmt }
    }
}

/// Database connection wrapper
pub struct DuckDBConnection {
    conn: Connection,
}

impl DuckDBConnection {
    pub fn new(path: &str) -> Result<Self> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(path)
        }
        .map_err(|e| DuckHubError::database(format!("Failed to open connection: {}", e)))?;

        Ok(Self { conn })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_size_bytes: u64,
    pub table_count: u32,
    pub connection_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_duckdb_engine_creation() {
        let config = DatabaseConfig {
            duckdb_path: ":memory:".to_string(),
            memory_limit: None,
            threads: None,
            max_memory: None,
            temp_directory: None,
            extensions: Vec::new(),
            pool: PoolConfig::default(),
        };

        let engine = DuckDBEngine::new(config).unwrap();
        assert!(engine.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_query_execution() {
        let config = DatabaseConfig {
            duckdb_path: ":memory:".to_string(),
            memory_limit: None,
            threads: None,
            max_memory: None,
            temp_directory: None,
            extensions: Vec::new(),
            pool: PoolConfig::default(),
        };

        let engine = DuckDBEngine::new(config).unwrap();
        
        let query = Query {
            id: generate_id(),
            sql: "SELECT 1 as test_col".to_string(),
            parameters: HashMap::new(),
            user_id: None,
            created_at: now(),
            timeout_seconds: None,
        };

        let result = engine.execute_query(&query).await.unwrap();
        assert_eq!(result.row_count, 1);
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.columns[0], "test_col");
    }
}
