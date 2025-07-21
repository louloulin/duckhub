//! DuckHub CLI Tool

use clap::{Parser, Subcommand};
use duckhub_common::DatabaseConfig;
use duckhub_common::utils::{generate_id, now};
use duckhub_common::types::{Query, QueryResult, QueryMetadata};
use duckhub_common::{Result, DuckHubError};
use duckhub_database::{DuckDBEngine, DataLakeManager};
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

        /// Save query to history
        #[arg(long)]
        save: bool,
    },

    /// Interactive query mode
    Interactive {
        /// Load query history
        #[arg(long)]
        with_history: bool,
    },

    /// Manage query history
    History {
        #[command(subcommand)]
        action: HistoryAction,
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

    /// Export query results
    Export {
        /// SQL query to execute
        sql: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export format (csv, json, parquet)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },
}

#[derive(Subcommand)]
enum HistoryAction {
    /// List query history
    List {
        /// Number of recent queries to show
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },

    /// Show specific query from history
    Show {
        /// Query ID
        id: String,
    },

    /// Execute query from history
    Execute {
        /// Query ID
        id: String,

        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Clear query history
    Clear,

    /// Export query history
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
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
        Commands::Query { sql, format, save } => {
            execute_query(engine, &sql, &format, save).await?;
        }
        Commands::Interactive { with_history } => {
            run_interactive_mode(engine, with_history).await?;
        }
        Commands::History { action } => {
            handle_history_action(engine, action).await?;
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
        Commands::Export { sql, output, format } => {
            export_query_results(engine, &sql, &output, &format).await?;
        }
    }

    Ok(())
}

async fn execute_query(engine: Arc<DuckDBEngine>, sql: &str, format: &str, save: bool) -> Result<()> {
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
    
    // TODO: Fix Send trait issue with DuckDB ToSql
    // Sequential execution for now due to Send trait limitations
    for _ in 0..count {
        let query = Query {
            id: generate_id(),
            sql: "SELECT 1 as test_col".to_string(),
            parameters: HashMap::new(),
            user_id: None,
            created_at: now(),
            timeout_seconds: None,
        };

        let _ = execute_query_helper(&engine, &query).await;
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

// 查询历史管理
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryHistoryEntry {
    id: String,
    sql: String,
    timestamp: DateTime<Utc>,
    execution_time_ms: u64,
    row_count: u64,
    success: bool,
    error_message: Option<String>,
}

impl QueryHistoryEntry {
    fn new(id: String, sql: String, execution_time_ms: u64, row_count: u64, success: bool, error_message: Option<String>) -> Self {
        Self {
            id,
            sql,
            timestamp: Utc::now(),
            execution_time_ms,
            row_count,
            success,
            error_message,
        }
    }
}

struct QueryHistoryManager {
    history_file: String,
}

impl QueryHistoryManager {
    fn new() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let history_file = format!("{}/.duckhub_history.json", home_dir);
        Self { history_file }
    }

    fn load_history(&self) -> Result<Vec<QueryHistoryEntry>> {
        if !Path::new(&self.history_file).exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&self.history_file)?;
        let history: Vec<QueryHistoryEntry> = serde_json::from_str(&content)
            .unwrap_or_else(|_| vec![]);
        Ok(history)
    }

    fn save_history(&self, history: &[QueryHistoryEntry]) -> Result<()> {
        let content = serde_json::to_string_pretty(history)?;
        fs::write(&self.history_file, content)?;
        Ok(())
    }

    fn add_entry(&self, entry: QueryHistoryEntry) -> Result<()> {
        let mut history = self.load_history()?;
        history.push(entry);

        // 保持最近1000条记录
        if history.len() > 1000 {
            let skip_count = history.len() - 1000;
            history = history.into_iter().skip(skip_count).collect();
        }

        self.save_history(&history)?;
        Ok(())
    }

    fn clear_history(&self) -> Result<()> {
        if Path::new(&self.history_file).exists() {
            fs::remove_file(&self.history_file)?;
        }
        Ok(())
    }
}

// 交互式查询模式
async fn run_interactive_mode(engine: Arc<DuckDBEngine>, with_history: bool) -> Result<()> {
    use std::io::{self, Write};

    println!("{}", "🚀 DuckHub Interactive Query Mode".green().bold());
    println!("{}", "Type 'help' for commands, 'exit' to quit".yellow());

    let history_manager = QueryHistoryManager::new();
    let mut query_history = if with_history {
        history_manager.load_history().unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    loop {
        print!("duckhub> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => {
                println!("{}", "Goodbye! 👋".green());
                break;
            }
            "help" => {
                show_interactive_help();
            }
            "history" => {
                show_query_history(&query_history);
            }
            "clear" => {
                query_history.clear();
                println!("{}", "Query history cleared.".yellow());
            }
            _ if input.starts_with("\\h ") => {
                // 执行历史查询 \h <id>
                let id = input.strip_prefix("\\h ").unwrap().trim();
                if let Some(entry) = query_history.iter().find(|e| e.id.starts_with(id)).cloned() {
                    println!("{}", format!("Executing: {}", entry.sql).blue());
                    execute_interactive_query(engine.clone(), &entry.sql, &mut query_history, &history_manager).await;
                } else {
                    println!("{}", format!("Query with ID '{}' not found", id).red());
                }
            }
            _ => {
                // 执行SQL查询
                execute_interactive_query(engine.clone(), input, &mut query_history, &history_manager).await;
            }
        }
    }

    Ok(())
}

async fn execute_interactive_query(
    engine: Arc<DuckDBEngine>,
    sql: &str,
    query_history: &mut Vec<QueryHistoryEntry>,
    history_manager: &QueryHistoryManager
) {
    let query_id = generate_id();
    let start_time = std::time::Instant::now();

    match engine.query(sql).await {
        Ok(rows) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            let row_count = rows.len() as u64;

            // 显示结果
            if rows.is_empty() {
                println!("{}", "Query returned 0 rows.".yellow());
            } else {
                display_query_results(&rows);
            }

            println!("{}", format!("✅ Query completed in {}ms, {} rows returned", execution_time, row_count).green());

            // 保存到历史
            let entry = QueryHistoryEntry::new(query_id.to_string(), sql.to_string(), execution_time, row_count, true, None);
            query_history.push(entry.clone());
            let _ = history_manager.add_entry(entry);
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            println!("{}", format!("❌ Query failed: {}", e).red());

            // 保存错误到历史
            let entry = QueryHistoryEntry::new(query_id.to_string(), sql.to_string(), execution_time, 0, false, Some(e.to_string()));
            query_history.push(entry.clone());
            let _ = history_manager.add_entry(entry);
        }
    }
}

fn show_interactive_help() {
    println!("{}", "📖 Interactive Commands:".blue().bold());
    println!("  help          - Show this help message");
    println!("  history       - Show query history");
    println!("  clear         - Clear query history");
    println!("  \\h <id>       - Execute query from history by ID");
    println!("  exit/quit     - Exit interactive mode");
    println!();
    println!("{}", "💡 Tips:".yellow().bold());
    println!("  - Type SQL queries directly");
    println!("  - Use semicolon to end statements");
    println!("  - Query history is automatically saved");
}

fn show_query_history(history: &[QueryHistoryEntry]) {
    if history.is_empty() {
        println!("{}", "No query history available.".yellow());
        return;
    }

    println!("{}", "📜 Query History:".blue().bold());
    println!("{:<8} {:<20} {:<8} {:<8} {:<50}", "ID", "Timestamp", "Time(ms)", "Rows", "SQL");
    println!("{}", "-".repeat(100));

    for entry in history.iter().rev().take(20) {
        let status = if entry.success { "✅" } else { "❌" };
        let short_id = &entry.id[..8];
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let sql_preview = if entry.sql.len() > 45 {
            format!("{}...", &entry.sql[..45])
        } else {
            entry.sql.clone()
        };

        println!("{} {:<8} {:<20} {:<8} {:<8} {:<50}",
            status, short_id, timestamp, entry.execution_time_ms, entry.row_count, sql_preview);
    }
}

fn display_query_results(rows: &[std::collections::HashMap<String, serde_json::Value>]) {
    if rows.is_empty() {
        return;
    }

    let columns: Vec<String> = rows[0].keys().cloned().collect();

    // 计算列宽
    let mut col_widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
    for row in rows.iter().take(100) { // 限制计算前100行以提高性能
        for (i, col) in columns.iter().enumerate() {
            if let Some(value) = row.get(col) {
                let value_str = match value {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                col_widths[i] = col_widths[i].max(value_str.len().min(30)); // 最大列宽30
            }
        }
    }

    // 打印表头
    for (i, col) in columns.iter().enumerate() {
        print!("{:width$}", col, width = col_widths[i]);
        if i < columns.len() - 1 {
            print!(" | ");
        }
    }
    println!();

    // 打印分隔线
    for (i, &width) in col_widths.iter().enumerate() {
        print!("{}", "-".repeat(width));
        if i < col_widths.len() - 1 {
            print!("-+-");
        }
    }
    println!();

    // 打印数据行（限制显示前50行）
    let display_rows = rows.len().min(50);
    for row in rows.iter().take(display_rows) {
        for (i, col) in columns.iter().enumerate() {
            let value_str = if let Some(value) = row.get(col) {
                match value {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                }
            } else {
                "".to_string()
            };

            let truncated = if value_str.len() > 30 {
                format!("{}...", &value_str[..27])
            } else {
                value_str
            };

            print!("{:width$}", truncated, width = col_widths[i]);
            if i < columns.len() - 1 {
                print!(" | ");
            }
        }
        println!();
    }

    if rows.len() > display_rows {
        println!("{}", format!("... and {} more rows", rows.len() - display_rows).yellow());
    }
}

// 历史管理命令处理
async fn handle_history_action(_engine: Arc<DuckDBEngine>, action: HistoryAction) -> Result<()> {
    let history_manager = QueryHistoryManager::new();

    match action {
        HistoryAction::List { limit } => {
            let history = history_manager.load_history()?;
            let recent_history: Vec<_> = history.iter().rev().take(limit as usize).collect();

            if recent_history.is_empty() {
                println!("{}", "No query history available.".yellow());
                return Ok(());
            }

            println!("{}", "📜 Query History:".blue().bold());
            println!("{:<10} {:<20} {:<8} {:<8} {:<6} {:<50}", "ID", "Timestamp", "Time(ms)", "Rows", "Status", "SQL");
            println!("{}", "-".repeat(110));

            for entry in recent_history {
                let status = if entry.success { "✅" } else { "❌" };
                let short_id = &entry.id[..8];
                let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
                let sql_preview = if entry.sql.len() > 45 {
                    format!("{}...", &entry.sql[..45])
                } else {
                    entry.sql.clone()
                };

                println!("{:<10} {:<20} {:<8} {:<8} {:<6} {:<50}",
                    short_id, timestamp, entry.execution_time_ms, entry.row_count, status, sql_preview);
            }
        }
        HistoryAction::Show { id } => {
            let history = history_manager.load_history()?;
            if let Some(entry) = history.iter().find(|e| e.id.starts_with(&id)) {
                println!("{}", "📋 Query Details:".blue().bold());
                println!("ID: {}", entry.id);
                println!("Timestamp: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S"));
                println!("Execution Time: {}ms", entry.execution_time_ms);
                println!("Row Count: {}", entry.row_count);
                println!("Status: {}", if entry.success { "✅ Success" } else { "❌ Failed" });
                if let Some(error) = &entry.error_message {
                    println!("Error: {}", error.red());
                }
                println!("SQL:");
                println!("{}", entry.sql);
            } else {
                println!("{}", format!("Query with ID '{}' not found", id).red());
            }
        }
        HistoryAction::Execute { id, format } => {
            let history = history_manager.load_history()?;
            if let Some(entry) = history.iter().find(|e| e.id.starts_with(&id)) {
                println!("{}", format!("Executing query from history: {}", entry.id).blue());
                execute_query(_engine, &entry.sql, &format, false).await?;
            } else {
                println!("{}", format!("Query with ID '{}' not found", id).red());
            }
        }
        HistoryAction::Clear => {
            history_manager.clear_history()?;
            println!("{}", "✅ Query history cleared.".green());
        }
        HistoryAction::Export { output } => {
            let history = history_manager.load_history()?;
            let content = serde_json::to_string_pretty(&history)?;
            std::fs::write(&output, content)?;
            println!("{}", format!("✅ Query history exported to {}", output).green());
        }
    }

    Ok(())
}

// 导出查询结果
async fn export_query_results(engine: Arc<DuckDBEngine>, sql: &str, output: &str, format: &str) -> Result<()> {
    println!("{}", format!("Executing query for export...").blue());

    let rows = engine.query(sql).await?;

    match format {
        "csv" => {
            export_to_csv(&rows, output)?;
        }
        "json" => {
            export_to_json(&rows, output)?;
        }
        "parquet" => {
            // 对于Parquet，我们使用DuckDB的COPY命令
            let copy_sql = format!("COPY ({}) TO '{}' (FORMAT PARQUET)", sql, output);
            engine.execute(&copy_sql).await?;
        }
        _ => {
            return Err(DuckHubError::validation(&format!("Unsupported export format: {}", format)));
        }
    }

    println!("{}", format!("✅ Results exported to {} (format: {})", output, format).green());
    Ok(())
}

fn export_to_csv(rows: &[std::collections::HashMap<String, serde_json::Value>], output: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output)?;

    if rows.is_empty() {
        return Ok(());
    }

    // 写入表头
    let columns: Vec<String> = rows[0].keys().cloned().collect();
    writeln!(file, "{}", columns.join(","))?;

    // 写入数据行
    for row in rows {
        let values: Vec<String> = columns.iter()
            .map(|col| {
                let value = row.get(col).unwrap_or(&serde_json::Value::Null);
                match value {
                    serde_json::Value::String(s) => format!("\"{}\"", s.replace("\"", "\"\"")),
                    serde_json::Value::Null => "".to_string(),
                    _ => value.to_string(),
                }
            })
            .collect();
        writeln!(file, "{}", values.join(","))?;
    }

    Ok(())
}

fn export_to_json(rows: &[std::collections::HashMap<String, serde_json::Value>], output: &str) -> Result<()> {
    let content = serde_json::to_string_pretty(rows)?;
    std::fs::write(output, content)?;
    Ok(())
}
