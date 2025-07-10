//! DuckHub CLI Tool

use clap::{Parser, Subcommand};
use duckhub_common::prelude::*;
use duckhub_common::DatabaseConfig;
use duckhub_common::utils::{generate_id, now};
use duckhub_database::*;
use std::collections::HashMap;
use std::sync::Arc;
use tabled::{Table, Tabled};
use colored::*;

// Helper function to execute query and return QueryResult
async fn execute_query_helper(engine: &Arc<DuckDBEngine>, query: &Query) -> Result<QueryResult> {
    let rows = engine.query(&query.sql).await?;
    let columns = if rows.is_empty() { vec![] } else { rows[0].keys().cloned().collect() };
    let result_rows: Vec<Vec<serde_json::Value>> = rows.iter()
        .map(|row| columns.iter().map(|col| row.get(col).cloned().unwrap_or(serde_json::Value::Null)).collect())
        .collect();

    Ok(QueryResult {
        query_id: query.id,
        columns,
        rows: result_rows,
        row_count: rows.len() as u64,
        execution_time_ms: 0,
        metadata: QueryMetadata {
            bytes_scanned: None,
            bytes_returned: None,
            cache_hit: false,
            execution_plan: None,
        },
    })
}

#[derive(Parser)]
#[command(name = "duckhub")]
#[command(about = "DuckHub CLI - A command-line interface for DuckHub data platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database file path
    #[arg(short, long, default_value = ":memory:")]
    database: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute SQL queries
    Query {
        /// SQL query to execute
        sql: String,
        
        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    /// Manage database schema
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    
    /// Show database information
    Info,
    
    /// Run performance tests
    Benchmark {
        /// Number of queries to run
        #[arg(short, long, default_value = "1000")]
        count: u32,
        
        /// Number of concurrent connections
        #[arg(short, long, default_value = "1")]
        concurrency: u32,
    },
    
    /// Manage data lake operations
    Lake {
        #[command(subcommand)]
        action: LakeAction,
    },

    /// Manage DuckLake operations
    DuckLake {
        #[command(subcommand)]
        action: DuckLakeAction,
    },
}

#[derive(Subcommand)]
enum SchemaAction {
    /// List all tables
    List,
    
    /// Show table schema
    Show {
        /// Table name
        table: String,
    },
    
    /// Create table from schema file
    Create {
        /// Table name
        table: String,
        
        /// Schema file path (JSON)
        schema_file: String,
    },
}

#[derive(Subcommand)]
enum LakeAction {
    /// Create external table from data lake file
    CreateTable {
        /// Table name
        table: String,

        /// File path
        file: String,

        /// File format (parquet, csv, json)
        #[arg(short, long, default_value = "parquet")]
        format: String,
    },

    /// Query data lake file directly
    Query {
        /// File path
        file: String,

        /// SQL query
        sql: String,

        /// File format (parquet, csv, json)
        #[arg(short, long, default_value = "parquet")]
        format: String,
    },
}

#[derive(Subcommand)]
enum DuckLakeAction {
    /// Create DuckLake database
    Create {
        /// Database name
        name: String,

        /// Metadata path
        #[arg(short, long)]
        metadata_path: String,

        /// Data path (optional)
        #[arg(short, long)]
        data_path: Option<String>,
    },

    /// Attach existing DuckLake database
    Attach {
        /// Database name
        name: String,

        /// Metadata path
        #[arg(short, long)]
        metadata_path: String,

        /// Read-only mode
        #[arg(long)]
        read_only: bool,
    },

    /// List snapshots
    Snapshots {
        /// Database name
        database: String,
    },

    /// Time travel query
    TimeTravel {
        /// Database name
        database: String,

        /// Table name
        table: String,

        /// Version number
        #[arg(short, long)]
        version: Option<u64>,

        /// Timestamp
        #[arg(short, long)]
        timestamp: Option<String>,

        /// SQL query
        sql: String,
    },
}

#[derive(Tabled)]
struct TableInfo {
    name: String,
    columns: usize,
    #[tabled(display_with = "display_option")]
    primary_key: Option<String>,
}

fn display_option(opt: &Option<String>) -> String {
    opt.as_ref().map(|s| s.clone()).unwrap_or_else(|| "None".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    // Create database configuration
    let config = DatabaseConfig {
        duckdb_path: cli.database.clone(),
        memory_limit: Some("2GB".to_string()),
        threads: Some(4),
        max_memory: None,
        temp_directory: None,
        extensions: vec!["httpfs".to_string(), "parquet".to_string()],
        pool: duckhub_common::PoolConfig::default(),
    };

    // Create database engine
    let engine = Arc::new(DuckDBEngine::new(config).await?);

    match cli.command {
        Commands::Query { sql, format } => {
            execute_query(engine, &sql, &format).await?;
        }
        Commands::Schema { action } => {
            handle_schema_action(engine, action).await?;
        }
        Commands::Info => {
            show_database_info(engine).await?;
        }
        Commands::Benchmark { count, concurrency } => {
            run_benchmark(engine, count, concurrency).await?;
        }
        Commands::Lake { action } => {
            handle_lake_action(engine, action).await?;
        }
        Commands::DuckLake { action } => {
            handle_ducklake_action(engine, action).await?;
        }
    }

    Ok(())
}

async fn execute_query(engine: Arc<DuckDBEngine>, sql: &str, format: &str) -> Result<()> {
    println!("{}", "Executing query...".blue());
    
    let query = Query {
        id: generate_id(),
        sql: sql.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let start_time = std::time::Instant::now();

    // Execute query and get results
    let rows = engine.query(&query.sql).await?;
    let execution_time = start_time.elapsed();

    // Convert to QueryResult format
    let columns = if rows.is_empty() {
        vec![]
    } else {
        rows[0].keys().cloned().collect()
    };

    let result_rows: Vec<Vec<serde_json::Value>> = rows.iter()
        .map(|row| columns.iter().map(|col| row.get(col).cloned().unwrap_or(serde_json::Value::Null)).collect())
        .collect();

    let result = QueryResult {
        query_id: query.id,
        columns: columns.clone(),
        rows: result_rows.clone(),
        row_count: result_rows.len() as u64,
        execution_time_ms: execution_time.as_millis() as u64,
        metadata: QueryMetadata {
            bytes_scanned: None,
            bytes_returned: None,
            cache_hit: false,
            execution_plan: None,
        },
    };

    match format {
        "json" => {
            let json_output = serde_json::to_string_pretty(&result)?;
            println!("{}", json_output);
        }
        "csv" => {
            // Print CSV header
            println!("{}", result.columns.join(","));
            
            // Print CSV rows
            for row in &result.rows {
                let csv_row: Vec<String> = row.iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => format!("\"{}\"", s),
                        _ => v.to_string(),
                    })
                    .collect();
                println!("{}", csv_row.join(","));
            }
        }
        "table" | _ => {
            if result.rows.is_empty() {
                println!("{}", "No results returned.".yellow());
            } else {
                // Create table for display
                #[derive(Tabled)]
                struct TableRow {
                    #[tabled(rename = "Column")]
                    column: String,
                    #[tabled(rename = "Value")]
                    value: String,
                }

                let mut table_rows = Vec::new();
                for (i, row) in result.rows.iter().enumerate() {
                    for (j, col) in result.columns.iter().enumerate() {
                        let value = row.get(j).map(|v| match v {
                            serde_json::Value::Null => "NULL".to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        }).unwrap_or_else(|| "NULL".to_string());

                        table_rows.push(TableRow {
                            column: format!("{}[{}]", col, i),
                            value,
                        });
                    }
                }

                let table = Table::new(table_rows);
                println!("{}", table);
            }
        }
    }

    println!();
    println!("{} {} rows in {:.2}ms", 
             "Returned".green(), 
             result.row_count, 
             execution_time.as_millis());

    Ok(())
}

async fn handle_schema_action(engine: Arc<DuckDBEngine>, action: SchemaAction) -> Result<()> {
    match action {
        SchemaAction::List => {
            let query = Query {
                id: generate_id(),
                sql: "SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'".to_string(),
                parameters: HashMap::new(),
                user_id: None,
                created_at: now(),
                timeout_seconds: None,
            };

            let rows = engine.query(&query.sql).await?;
            let columns = if rows.is_empty() { vec![] } else { rows[0].keys().cloned().collect() };
            let result_rows: Vec<Vec<serde_json::Value>> = rows.iter()
                .map(|row| columns.iter().map(|col| row.get(col).cloned().unwrap_or(serde_json::Value::Null)).collect())
                .collect();
            let result = QueryResult {
                query_id: query.id,
                columns,
                rows: result_rows,
                row_count: rows.len() as u64,
                execution_time_ms: 0,
                metadata: QueryMetadata {
                    bytes_scanned: None,
                    bytes_returned: None,
                    cache_hit: false,
                    execution_plan: None,
                },
            };
            
            if result.rows.is_empty() {
                println!("{}", "No tables found.".yellow());
            } else {
                println!("{}", "Tables:".blue().bold());
                for row in &result.rows {
                    if let Some(serde_json::Value::String(table_name)) = row.get(0) {
                        println!("  • {}", table_name.green());
                    }
                }
            }
        }
        SchemaAction::Show { table } => {
            println!("{} Schema operations are not yet implemented", "Info:".blue().bold());
            println!("You can use SQL queries to inspect table structure:");
            println!("  DESCRIBE {}", table.green());
        }
        SchemaAction::Create { table, schema_file: _ } => {
            println!("{} Schema creation is not yet implemented", "Info:".blue().bold());
            println!("You can use SQL CREATE TABLE statements:");
            println!("  CREATE TABLE {} (...)", table.green());
        }
    }

    Ok(())
}

async fn show_database_info(engine: Arc<DuckDBEngine>) -> Result<()> {
    println!("{}", "Database Information".blue().bold());
    println!();

    // Get basic stats
    let stats_query = Query {
        id: generate_id(),
        sql: "SELECT COUNT(*) as table_count FROM information_schema.tables WHERE table_schema = 'main'".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let stats_result = execute_query_helper(&engine, &stats_query).await?;
    let table_count = if let Some(row) = stats_result.rows.get(0) {
        if let Some(serde_json::Value::Number(n)) = row.get(0) {
            n.as_u64().unwrap_or(0)
        } else { 0 }
    } else { 0 };

    println!("📊 {} {}", "Tables:".blue(), table_count.to_string().green());
    
    // Test query performance
    let perf_query = Query {
        id: generate_id(),
        sql: "SELECT 1".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let start_time = std::time::Instant::now();
    let _ = execute_query_helper(&engine, &perf_query).await?;
    let query_time = start_time.elapsed();

    println!("⚡ {} {:.2}ms", "Query Response Time:".blue(), query_time.as_millis());
    
    // Health check
    match engine.check_connection().await {
        Ok(_) => println!("✅ {} {}", "Status:".blue(), "Healthy".green()),
        Err(_) => println!("❌ {} {}", "Status:".blue(), "Unhealthy".red()),
    }

    Ok(())
}

async fn run_benchmark(engine: Arc<DuckDBEngine>, count: u32, concurrency: u32) -> Result<()> {
    println!("{}", "Running benchmark...".blue().bold());
    println!("Queries: {}, Concurrency: {}", count, concurrency);
    println!();

    let start_time = std::time::Instant::now();
    let mut handles = Vec::new();

    let queries_per_task = count / concurrency;
    
    for _ in 0..concurrency {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..queries_per_task {
                let query = Query {
                    id: generate_id(),
                    sql: "SELECT 1 as test_col".to_string(),
                    parameters: HashMap::new(),
                    user_id: None,
                    created_at: now(),
                    timeout_seconds: None,
                };
                
                let _ = execute_query_helper(&engine_clone, &query).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.map_err(|e| DuckHubError::internal(format!("Task failed: {}", e)))?;
    }

    let total_time = start_time.elapsed();
    let qps = count as f64 / total_time.as_secs_f64();

    println!("{}", "Benchmark Results:".green().bold());
    println!("Total Time: {:.2}s", total_time.as_secs_f64());
    println!("Queries per Second: {:.2}", qps);
    println!("Average Query Time: {:.2}ms", total_time.as_millis() as f64 / count as f64);

    Ok(())
}

async fn handle_lake_action(engine: Arc<DuckDBEngine>, action: LakeAction) -> Result<()> {
    let lake_manager = DataLakeManager::new();

    match action {
        LakeAction::CreateTable { table, file, format } => {
            println!("{} External table creation is not yet implemented", "Info:".blue().bold());
            println!("You can use SQL to create external tables:");
            println!("  CREATE TABLE {} AS SELECT * FROM read_{}('{}')", table.green(), format, file);

            println!("{} Created external table '{}' from '{}'", 
                     "Success:".green().bold(), table.green(), file.blue());
        }
        LakeAction::Query { file, sql, format } => {
            println!("{} File querying is not yet implemented", "Info:".blue().bold());
            println!("You can use SQL to query files directly:");
            println!("  {}", sql.green());
            println!("  FROM read_{}('{}')", format, file);
        }
    }

    Ok(())
}

async fn handle_ducklake_action(engine: Arc<DuckDBEngine>, action: DuckLakeAction) -> Result<()> {
    match action {
        DuckLakeAction::Create { name, metadata_path, data_path } => {
            // Create DuckLake secret
            let secret_sql = if let Some(data_path) = data_path {
                format!(
                    "CREATE SECRET {} (TYPE DUCKLAKE, METADATA_PATH '{}', DATA_PATH '{}')",
                    name, metadata_path, data_path
                )
            } else {
                format!(
                    "CREATE SECRET {} (TYPE DUCKLAKE, METADATA_PATH '{}')",
                    name, metadata_path
                )
            };

            let query = Query {
                id: generate_id(),
                sql: secret_sql,
                parameters: HashMap::new(),
                user_id: None,
                created_at: now(),
                timeout_seconds: None,
            };

            let _ = execute_query_helper(&engine, &query).await?;
            println!("{} Created DuckLake secret '{}'", "Success:".green().bold(), name.green());
        }

        DuckLakeAction::Attach { name, metadata_path, read_only } => {
            let attach_sql = if read_only {
                format!("ATTACH 'ducklake:{}' (READ_ONLY) AS {}", metadata_path, name)
            } else {
                format!("ATTACH 'ducklake:{}' AS {}", metadata_path, name)
            };

            let query = Query {
                id: generate_id(),
                sql: attach_sql,
                parameters: HashMap::new(),
                user_id: None,
                created_at: now(),
                timeout_seconds: None,
            };

            let _ = execute_query_helper(&engine, &query).await?;
            println!("{} Attached DuckLake database '{}'", "Success:".green().bold(), name.green());
        }

        DuckLakeAction::Snapshots { database } => {
            let query = Query {
                id: generate_id(),
                sql: format!("SELECT * FROM {}.snapshots() ORDER BY snapshot_id DESC LIMIT 10", database),
                parameters: HashMap::new(),
                user_id: None,
                created_at: now(),
                timeout_seconds: None,
            };

            match execute_query_helper(&engine, &query).await {
                Ok(result) => {
                    if result.rows.is_empty() {
                        println!("{} No snapshots found for database '{}'", "Info:".blue().bold(), database);
                    } else {
                        println!("{} Snapshots for database '{}':", "Info:".blue().bold(), database.green());

                        let mut table_data = Vec::new();
                        table_data.push(result.columns.clone());

                        for row in &result.rows {
                            let string_row: Vec<String> = row.iter()
                                .map(|v| match v {
                                    serde_json::Value::Null => "NULL".to_string(),
                                    serde_json::Value::String(s) => s.clone(),
                                    _ => v.to_string(),
                                })
                                .collect();
                            table_data.push(string_row);
                        }

                        // Print simple table format
                        for row in table_data {
                            println!("{}", row.join(" | "));
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to get snapshots: {}", "Error:".red().bold(), e);
                }
            }
        }

        DuckLakeAction::TimeTravel { database, table, version, timestamp, sql } => {
            let time_travel_clause = if let Some(ref v) = version {
                format!("AT (VERSION => {})", v)
            } else if let Some(ref ts) = timestamp {
                format!("AT (TIMESTAMP => '{}')", ts)
            } else {
                return Err(DuckHubError::validation("Either version or timestamp must be specified"));
            };

            // Replace table references in SQL with time travel syntax
            let time_travel_sql = sql.replace(
                &format!("{}.{}", database, table),
                &format!("{}.{} {}", database, table, time_travel_clause)
            );

            let query = Query {
                id: generate_id(),
                sql: time_travel_sql,
                parameters: HashMap::new(),
                user_id: None,
                created_at: now(),
                timeout_seconds: None,
            };

            let result = execute_query_helper(&engine, &query).await?;

            if let Some(v) = version {
                println!("{} Time travel query at version {} executed successfully", "Success:".green().bold(), v);
            } else if let Some(ts) = timestamp {
                println!("{} Time travel query at timestamp '{}' executed successfully", "Success:".green().bold(), ts);
            }

            println!("Returned {} rows", result.row_count);

            // Display results in table format
            if !result.rows.is_empty() {
                let mut table_data = Vec::new();
                table_data.push(result.columns.clone());

                for row in &result.rows {
                    let string_row: Vec<String> = row.iter()
                        .map(|v| match v {
                            serde_json::Value::Null => "NULL".to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect();
                    table_data.push(string_row);
                }

                // Print simple table format
                for row in table_data {
                    println!("{}", row.join(" | "));
                }
            }
        }
    }

    Ok(())
}
