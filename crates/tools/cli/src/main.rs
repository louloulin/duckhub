//! DuckHub CLI Tool

use clap::{Parser, Subcommand};
use duckhub_common::prelude::*;
use duckhub_database::*;
use std::collections::HashMap;
use std::sync::Arc;
use tabled::{Table, Tabled};
use colored::*;

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
        pool: PoolConfig::default(),
    };

    // Create database engine
    let engine = Arc::new(DuckDBEngine::new(config)?);

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
    let result = engine.execute_query(&query).await?;
    let execution_time = start_time.elapsed();

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
                
                let table = Table::new(table_data);
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

            let result = engine.execute_query(&query).await?;
            
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
            if !engine.table_exists(&table).await? {
                println!("{} Table '{}' not found.", "Error:".red().bold(), table);
                return Ok(());
            }

            let schema = engine.get_schema(&table).await?;
            
            println!("{} {}", "Schema for table".blue().bold(), table.green().bold());
            println!();
            
            let mut field_data = Vec::new();
            field_data.push(vec!["Column".to_string(), "Type".to_string(), "Nullable".to_string()]);
            
            for field in &schema.fields {
                field_data.push(vec![
                    field.name.clone(),
                    format!("{:?}", field.data_type),
                    if field.nullable { "YES".to_string() } else { "NO".to_string() },
                ]);
            }
            
            let table = Table::new(field_data);
            println!("{}", table);
            
            if let Some(pk) = &schema.primary_key {
                println!();
                println!("{} {}", "Primary Key:".blue().bold(), pk.join(", ").green());
            }
        }
        SchemaAction::Create { table, schema_file } => {
            let schema_content = tokio::fs::read_to_string(&schema_file).await
                .map_err(|e| DuckHubError::io(e))?;
            
            let schema: Schema = serde_json::from_str(&schema_content)
                .map_err(|e| DuckHubError::validation(format!("Invalid schema file: {}", e)))?;
            
            engine.create_table(&table, &schema).await?;
            println!("{} Created table '{}'", "Success:".green().bold(), table.green());
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

    let stats_result = engine.execute_query(&stats_query).await?;
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
    engine.execute_query(&perf_query).await?;
    let query_time = start_time.elapsed();

    println!("⚡ {} {:.2}ms", "Query Response Time:".blue(), query_time.as_millis());
    
    // Health check
    match engine.health_check().await {
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
                
                let _ = engine_clone.execute_query(&query).await;
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
            let file_format = match format.as_str() {
                "parquet" => FileFormat::Parquet,
                "csv" => FileFormat::CSV,
                "json" => FileFormat::JSON,
                _ => return Err(DuckHubError::validation(format!("Unsupported format: {}", format))),
            };

            lake_manager.create_external_table(
                engine.as_ref(),
                &table,
                &file,
                &file_format,
                None,
            ).await?;

            println!("{} Created external table '{}' from '{}'", 
                     "Success:".green().bold(), table.green(), file.blue());
        }
        LakeAction::Query { file, sql, format } => {
            let file_format = match format.as_str() {
                "parquet" => FileFormat::Parquet,
                "csv" => FileFormat::CSV,
                "json" => FileFormat::JSON,
                _ => return Err(DuckHubError::validation(format!("Unsupported format: {}", format))),
            };

            let result = lake_manager.query_file(
                engine.as_ref(),
                &file,
                &file_format,
                &sql,
                None,
            ).await?;

            println!("{} Query executed on '{}'", "Success:".green().bold(), file.blue());
            println!("Returned {} rows", result.row_count);
        }
    }

    Ok(())
}
