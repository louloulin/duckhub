//! 基于Rig框架的工具系统
//! 为AI Agent提供数据库查询、分析等工具

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_query_analytics::QueryAnalyticsService;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{info, warn, debug, instrument};
use rig::{
    tool::{Tool, ToolSet},
    completion::ToolDefinition,
};
use anyhow::{Result as AnyhowResult, Context};
use uuid::Uuid;

/// 数据库查询工具
#[derive(Debug, Clone)]
pub struct DatabaseQueryTool {
    engine: Arc<DuckDBEngine>,
    query_service: Arc<QueryAnalyticsService>,
}

impl DatabaseQueryTool {
    pub fn new(engine: Arc<DuckDBEngine>, query_service: Arc<QueryAnalyticsService>) -> Self {
        Self {
            engine,
            query_service,
        }
    }
}

#[derive(Debug, Deserialize)]
struct QueryToolInput {
    sql_query: String,
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct QueryToolOutput {
    success: bool,
    row_count: usize,
    execution_time_ms: u64,
    data: Vec<Value>,
    error: Option<String>,
}

impl Tool for DatabaseQueryTool {
    const NAME: &'static str = "database_query";

    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = QueryToolInput;
    type Output = QueryToolOutput;

    fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "执行SQL查询并返回结果。用于查询数据库中的金融数据。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "sql_query": {
                        "type": "string",
                        "description": "要执行的SQL查询语句"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "限制返回的行数，默认为100",
                        "minimum": 1,
                        "maximum": 1000
                    }
                },
                "required": ["sql_query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let start_time = std::time::Instant::now();
        
        // 添加LIMIT子句（如果没有的话）
        let mut sql_query = args.sql_query.trim().to_string();
        let limit = args.limit.unwrap_or(100);
        
        if !sql_query.to_uppercase().contains("LIMIT") {
            sql_query = format!("{} LIMIT {}", sql_query, limit);
        }

        debug!("执行SQL查询: {}", sql_query);

        match self.query_service.execute_query(&sql_query).await {
            Ok(result) => {
                let execution_time = start_time.elapsed();
                info!("查询执行成功，返回{}行数据，耗时{}ms", 
                     result.row_count, execution_time.as_millis());
                
                Ok(QueryToolOutput {
                    success: true,
                    row_count: result.row_count,
                    execution_time_ms: execution_time.as_millis() as u64,
                    data: result.data,
                    error: None,
                })
            },
            Err(e) => {
                warn!("查询执行失败: {}", e);
                Ok(QueryToolOutput {
                    success: false,
                    row_count: 0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    data: vec![],
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

/// 表结构查询工具
#[derive(Debug, Clone)]
pub struct TableSchemaTool {
    engine: Arc<DuckDBEngine>,
}

impl TableSchemaTool {
    pub fn new(engine: Arc<DuckDBEngine>) -> Self {
        Self { engine }
    }
}

#[derive(Debug, Deserialize)]
struct SchemaToolInput {
    table_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct SchemaToolOutput {
    success: bool,
    tables: Vec<TableSchema>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct TableSchema {
    table_name: String,
    columns: Vec<ColumnSchema>,
}

#[derive(Debug, Serialize)]
struct ColumnSchema {
    column_name: String,
    data_type: String,
    is_nullable: bool,
    default_value: Option<String>,
}

impl Tool for TableSchemaTool {
    const NAME: &'static str = "table_schema";

    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = SchemaToolInput;
    type Output = SchemaToolOutput;

    fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取数据库表结构信息。如果不指定表名，返回所有表的结构。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "table_name": {
                        "type": "string",
                        "description": "要查询的表名，如果为空则返回所有表"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("查询表结构: {:?}", args.table_name);

        let sql = if let Some(table_name) = args.table_name {
            format!(
                "SELECT table_name, column_name, data_type, is_nullable, column_default 
                 FROM information_schema.columns 
                 WHERE table_name = '{}' 
                 ORDER BY table_name, ordinal_position",
                table_name
            )
        } else {
            "SELECT table_name, column_name, data_type, is_nullable, column_default 
             FROM information_schema.columns 
             ORDER BY table_name, ordinal_position".to_string()
        };

        match self.engine.execute_query(&sql).await {
            Ok(rows) => {
                let mut tables: std::collections::HashMap<String, Vec<ColumnSchema>> = 
                    std::collections::HashMap::new();

                for row in rows {
                    if let (Some(table_name), Some(column_name), Some(data_type)) = (
                        row.get("table_name").and_then(|v| v.as_str()),
                        row.get("column_name").and_then(|v| v.as_str()),
                        row.get("data_type").and_then(|v| v.as_str()),
                    ) {
                        let is_nullable = row.get("is_nullable")
                            .and_then(|v| v.as_str())
                            .map(|s| s.eq_ignore_ascii_case("YES"))
                            .unwrap_or(false);

                        let default_value = row.get("column_default")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let column = ColumnSchema {
                            column_name: column_name.to_string(),
                            data_type: data_type.to_string(),
                            is_nullable,
                            default_value,
                        };

                        tables.entry(table_name.to_string())
                            .or_insert_with(Vec::new)
                            .push(column);
                    }
                }

                let table_schemas: Vec<TableSchema> = tables.into_iter()
                    .map(|(table_name, columns)| TableSchema { table_name, columns })
                    .collect();

                info!("成功获取{}个表的结构信息", table_schemas.len());

                Ok(SchemaToolOutput {
                    success: true,
                    tables: table_schemas,
                    error: None,
                })
            },
            Err(e) => {
                warn!("获取表结构失败: {}", e);
                Ok(SchemaToolOutput {
                    success: false,
                    tables: vec![],
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

/// 数据统计工具
#[derive(Debug, Clone)]
pub struct DataStatsTool {
    engine: Arc<DuckDBEngine>,
}

impl DataStatsTool {
    pub fn new(engine: Arc<DuckDBEngine>) -> Self {
        Self { engine }
    }
}

#[derive(Debug, Deserialize)]
struct StatsToolInput {
    table_name: String,
    column_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatsToolOutput {
    success: bool,
    table_name: String,
    row_count: Option<u64>,
    column_stats: Vec<ColumnStats>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ColumnStats {
    column_name: String,
    data_type: String,
    null_count: u64,
    distinct_count: Option<u64>,
    min_value: Option<Value>,
    max_value: Option<Value>,
    avg_value: Option<f64>,
}

impl Tool for DataStatsTool {
    const NAME: &'static str = "data_stats";

    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = StatsToolInput;
    type Output = StatsToolOutput;

    fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "获取表或列的统计信息，包括行数、空值数量、唯一值数量、最小值、最大值、平均值等。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "table_name": {
                        "type": "string",
                        "description": "要统计的表名"
                    },
                    "column_name": {
                        "type": "string",
                        "description": "要统计的列名，如果为空则统计所有列"
                    }
                },
                "required": ["table_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("获取数据统计: 表={}, 列={:?}", args.table_name, args.column_name);

        // 首先获取行数
        let row_count_sql = format!("SELECT COUNT(*) as row_count FROM {}", args.table_name);
        let row_count = match self.engine.execute_query(&row_count_sql).await {
            Ok(rows) => rows.first()
                .and_then(|row| row.get("row_count"))
                .and_then(|v| v.as_u64()),
            Err(_) => None,
        };

        // 获取列信息
        let columns_sql = format!(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = '{}'",
            args.table_name
        );

        let columns_info = match self.engine.execute_query(&columns_sql).await {
            Ok(rows) => rows,
            Err(e) => {
                return Ok(StatsToolOutput {
                    success: false,
                    table_name: args.table_name,
                    row_count,
                    column_stats: vec![],
                    error: Some(format!("获取列信息失败: {}", e)),
                });
            }
        };

        let mut column_stats = Vec::new();

        for column_info in columns_info {
            if let (Some(column_name), Some(data_type)) = (
                column_info.get("column_name").and_then(|v| v.as_str()),
                column_info.get("data_type").and_then(|v| v.as_str()),
            ) {
                // 如果指定了列名，只统计该列
                if let Some(ref target_column) = args.column_name {
                    if column_name != target_column {
                        continue;
                    }
                }

                let stats = self.get_column_stats(&args.table_name, column_name, data_type).await;
                column_stats.push(stats);
            }
        }

        info!("成功获取表{}的统计信息，包含{}列", args.table_name, column_stats.len());

        Ok(StatsToolOutput {
            success: true,
            table_name: args.table_name,
            row_count,
            column_stats,
            error: None,
        })
    }
}

impl DataStatsTool {
    async fn get_column_stats(&self, table_name: &str, column_name: &str, data_type: &str) -> ColumnStats {
        let mut stats = ColumnStats {
            column_name: column_name.to_string(),
            data_type: data_type.to_string(),
            null_count: 0,
            distinct_count: None,
            min_value: None,
            max_value: None,
            avg_value: None,
        };

        // 基础统计
        let basic_stats_sql = format!(
            "SELECT 
                COUNT(*) - COUNT({}) as null_count,
                COUNT(DISTINCT {}) as distinct_count
             FROM {}",
            column_name, column_name, table_name
        );

        if let Ok(rows) = self.engine.execute_query(&basic_stats_sql).await {
            if let Some(row) = rows.first() {
                stats.null_count = row.get("null_count").and_then(|v| v.as_u64()).unwrap_or(0);
                stats.distinct_count = row.get("distinct_count").and_then(|v| v.as_u64());
            }
        }

        // 数值类型的额外统计
        if data_type.contains("INT") || data_type.contains("DECIMAL") || 
           data_type.contains("FLOAT") || data_type.contains("DOUBLE") {
            let numeric_stats_sql = format!(
                "SELECT MIN({}) as min_val, MAX({}) as max_val, AVG({}) as avg_val FROM {}",
                column_name, column_name, column_name, table_name
            );

            if let Ok(rows) = self.engine.execute_query(&numeric_stats_sql).await {
                if let Some(row) = rows.first() {
                    stats.min_value = row.get("min_val").cloned();
                    stats.max_value = row.get("max_val").cloned();
                    stats.avg_value = row.get("avg_val").and_then(|v| v.as_f64());
                }
            }
        }

        stats
    }
}

/// 创建完整的工具集
pub fn create_rig_toolset(
    engine: Arc<DuckDBEngine>,
    query_service: Arc<QueryAnalyticsService>,
) -> ToolSet {
    ToolSet::new()
        .with_tool(DatabaseQueryTool::new(engine.clone(), query_service))
        .with_tool(TableSchemaTool::new(engine.clone()))
        .with_tool(DataStatsTool::new(engine))
}
