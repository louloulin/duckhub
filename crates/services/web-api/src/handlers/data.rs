// 数据探索API处理器
// 提供数据库表信息、Schema查询、数据预览等功能

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use duckhub_database::{DuckDBEngine, QueryResult};
use crate::{AppState, success_response, error_response};

/// 格式化字节大小为人类可读格式
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// 表信息结构 (扩展版本，匹配前端期望格式)
#[derive(Debug, Serialize, Clone)]
pub struct TableInfo {
    pub name: String,
    pub rows: u64,           // 前端期望字段名
    pub size: String,        // 前端期望格式化的大小字符串
    pub schema_version: u32, // 前端期望字段名
    pub last_modified: String, // 前端期望字段名
    pub description: Option<String>,
    // 保留原有字段用于内部计算
    pub size_bytes: u64,
    pub column_count: u32,
    pub table_type: String,
}

/// 列信息结构
#[derive(Debug, Serialize, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub position: u32,
    pub description: Option<String>,
}

/// 表Schema结构
#[derive(Debug, Serialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub indexes: Vec<IndexInfo>,
    pub row_count: u64,
    pub size_bytes: u64,
}

/// 外键信息
#[derive(Debug, Serialize)]
pub struct ForeignKeyInfo {
    pub column_name: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

/// 索引信息
#[derive(Debug, Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String,
}

/// Schema变更信息
#[derive(Debug, Serialize, Clone)]
pub struct SchemaChange {
    pub change_type: String, // "add_column", "drop_column", "modify_column", "add_index", "drop_index"
    pub table: String,
    pub column: Option<String>,
    pub old_definition: Option<String>,
    pub new_definition: Option<String>,
    pub description: String,
    pub impact: String, // "low", "medium", "high"
}

/// Schema版本信息
#[derive(Debug, Serialize)]
pub struct SchemaVersion {
    pub version: u32,
    pub timestamp: String,
    pub author: String,
    pub description: String,
    pub changes: Vec<SchemaChange>,
    pub compatibility: String, // "backward", "forward", "breaking", "full"
}

/// Schema演进查询参数
#[derive(Debug, Deserialize)]
pub struct SchemaEvolutionQuery {
    pub table: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 表统计信息
#[derive(Debug, Serialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: u64,
    pub size_bytes: u64,
    pub column_stats: Vec<ColumnStats>,
    pub last_updated: String,
}

/// 列统计信息
#[derive(Debug, Serialize)]
pub struct ColumnStats {
    pub column_name: String,
    pub data_type: String,
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub avg_length: Option<f64>,
}

/// 表数据查询参数
#[derive(Debug, Deserialize)]
pub struct TableDataQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>, // ASC or DESC
    pub filters: Option<HashMap<String, String>>,
}

/// 表数据响应
#[derive(Debug, Serialize)]
pub struct TableDataResponse {
    pub table_name: String,
    pub columns: Vec<String>,
    pub data: Vec<Vec<serde_json::Value>>,
    pub total_rows: u64,
    pub returned_rows: u32,
    pub has_more: bool,
}

/// 获取所有表列表
#[instrument(skip(app_state))]
pub async fn get_tables(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取数据库表列表");
    
    // 从DuckDB获取真实的表列表
    let tables = match get_real_table_list(&app_state.engine).await {
        Ok(real_tables) => real_tables,
        Err(e) => {
            error!("获取表列表失败: {}", e);
            vec![]
        }
    };
    
    info!("成功获取 {} 个表的信息", tables.len());
    Ok(success_response(tables))
}

/// 获取表结构信息
#[instrument(skip(app_state))]
pub async fn get_table_schema(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> ActixResult<HttpResponse> {
    let table_name = path.into_inner();
    info!("获取表 {} 的结构信息", table_name);
    
    // 模拟不同表的Schema数据
    let schema = match table_name.as_str() {
        "transactions" => TableSchema {
            table_name: "transactions".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "BIGINT".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 1,
                    description: Some("交易ID".to_string()),
                },
                ColumnInfo {
                    name: "user_id".to_string(),
                    data_type: "BIGINT".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 2,
                    description: Some("用户ID".to_string()),
                },
                ColumnInfo {
                    name: "amount".to_string(),
                    data_type: "DECIMAL(18,2)".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 3,
                    description: Some("交易金额".to_string()),
                },
                ColumnInfo {
                    name: "currency".to_string(),
                    data_type: "VARCHAR(3)".to_string(),
                    nullable: false,
                    default_value: Some("'USD'".to_string()),
                    position: 4,
                    description: Some("货币类型".to_string()),
                },
                ColumnInfo {
                    name: "transaction_type".to_string(),
                    data_type: "VARCHAR(20)".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 5,
                    description: Some("交易类型".to_string()),
                },
                ColumnInfo {
                    name: "status".to_string(),
                    data_type: "VARCHAR(20)".to_string(),
                    nullable: false,
                    default_value: Some("'PENDING'".to_string()),
                    position: 6,
                    description: Some("交易状态".to_string()),
                },
                ColumnInfo {
                    name: "created_at".to_string(),
                    data_type: "TIMESTAMP".to_string(),
                    nullable: false,
                    default_value: Some("CURRENT_TIMESTAMP".to_string()),
                    position: 7,
                    description: Some("创建时间".to_string()),
                },
                ColumnInfo {
                    name: "updated_at".to_string(),
                    data_type: "TIMESTAMP".to_string(),
                    nullable: true,
                    default_value: None,
                    position: 8,
                    description: Some("更新时间".to_string()),
                },
            ],
            primary_keys: vec!["id".to_string()],
            foreign_keys: vec![
                ForeignKeyInfo {
                    column_name: "user_id".to_string(),
                    referenced_table: "users".to_string(),
                    referenced_column: "id".to_string(),
                }
            ],
            indexes: vec![
                IndexInfo {
                    name: "idx_transactions_user_id".to_string(),
                    columns: vec!["user_id".to_string()],
                    unique: false,
                    index_type: "BTREE".to_string(),
                },
                IndexInfo {
                    name: "idx_transactions_created_at".to_string(),
                    columns: vec!["created_at".to_string()],
                    unique: false,
                    index_type: "BTREE".to_string(),
                },
            ],
            row_count: 1_250_000,
            size_bytes: 125_000_000,
        },
        "users" => TableSchema {
            table_name: "users".to_string(),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "BIGINT".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 1,
                    description: Some("用户ID".to_string()),
                },
                ColumnInfo {
                    name: "username".to_string(),
                    data_type: "VARCHAR(50)".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 2,
                    description: Some("用户名".to_string()),
                },
                ColumnInfo {
                    name: "email".to_string(),
                    data_type: "VARCHAR(100)".to_string(),
                    nullable: false,
                    default_value: None,
                    position: 3,
                    description: Some("邮箱地址".to_string()),
                },
                ColumnInfo {
                    name: "first_name".to_string(),
                    data_type: "VARCHAR(50)".to_string(),
                    nullable: true,
                    default_value: None,
                    position: 4,
                    description: Some("名".to_string()),
                },
                ColumnInfo {
                    name: "last_name".to_string(),
                    data_type: "VARCHAR(50)".to_string(),
                    nullable: true,
                    default_value: None,
                    position: 5,
                    description: Some("姓".to_string()),
                },
                ColumnInfo {
                    name: "created_at".to_string(),
                    data_type: "TIMESTAMP".to_string(),
                    nullable: false,
                    default_value: Some("CURRENT_TIMESTAMP".to_string()),
                    position: 6,
                    description: Some("创建时间".to_string()),
                },
            ],
            primary_keys: vec!["id".to_string()],
            foreign_keys: vec![],
            indexes: vec![
                IndexInfo {
                    name: "idx_users_username".to_string(),
                    columns: vec!["username".to_string()],
                    unique: true,
                    index_type: "BTREE".to_string(),
                },
                IndexInfo {
                    name: "idx_users_email".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                    index_type: "BTREE".to_string(),
                },
            ],
            row_count: 50_000,
            size_bytes: 5_000_000,
        },
        _ => {
            error!("表 {} 不存在", table_name);
            return Ok(error_response("表不存在", 404));
        }
    };
    
    info!("成功获取表 {} 的结构信息，包含 {} 个列", table_name, schema.columns.len());
    Ok(success_response(schema))
}

/// 获取表数据（支持分页和过滤）
#[instrument(skip(app_state))]
pub async fn get_table_data(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<TableDataQuery>
) -> ActixResult<HttpResponse> {
    let table_name = path.into_inner();
    let limit = query.limit.unwrap_or(100).min(1000); // 最大1000行
    let offset = query.offset.unwrap_or(0);

    info!("获取表 {} 的数据，limit: {}, offset: {}", table_name, limit, offset);

    // 从真实数据库获取表数据
    let (columns, data, total_rows) = match get_real_table_data(&app_state.engine, &table_name, limit, offset).await {
        Ok((cols, rows, total)) => (cols, rows, total),
        Err(e) => {
            error!("获取表 {} 数据失败: {}", table_name, e);
            return Ok(error_response(&format!("获取表数据失败: {}", e), 500));
        }
    };

    let response = TableDataResponse {
        table_name: table_name.clone(),
        columns,
        data: data.clone(),
        total_rows,
        returned_rows: data.len() as u32,
        has_more: (offset as u64 + data.len() as u64) < total_rows,
    };

    info!("成功获取表 {} 的数据，返回 {} 行", table_name, response.returned_rows);
    Ok(success_response(response))
}

/// 获取表统计信息
#[instrument(skip(app_state))]
pub async fn get_table_stats(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> ActixResult<HttpResponse> {
    let table_name = path.into_inner();
    info!("获取表 {} 的统计信息", table_name);

    // 模拟统计数据
    let stats = match table_name.as_str() {
        "transactions" => TableStats {
            table_name: "transactions".to_string(),
            row_count: 1_250_000,
            size_bytes: 125_000_000,
            column_stats: vec![
                ColumnStats {
                    column_name: "id".to_string(),
                    data_type: "BIGINT".to_string(),
                    null_count: 0,
                    distinct_count: 1_250_000,
                    min_value: Some("1".to_string()),
                    max_value: Some("1250000".to_string()),
                    avg_length: None,
                },
                ColumnStats {
                    column_name: "amount".to_string(),
                    data_type: "DECIMAL(18,2)".to_string(),
                    null_count: 0,
                    distinct_count: 850_000,
                    min_value: Some("0.01".to_string()),
                    max_value: Some("999999.99".to_string()),
                    avg_length: None,
                },
                ColumnStats {
                    column_name: "currency".to_string(),
                    data_type: "VARCHAR(3)".to_string(),
                    null_count: 0,
                    distinct_count: 15,
                    min_value: Some("AUD".to_string()),
                    max_value: Some("USD".to_string()),
                    avg_length: Some(3.0),
                },
                ColumnStats {
                    column_name: "status".to_string(),
                    data_type: "VARCHAR(20)".to_string(),
                    null_count: 0,
                    distinct_count: 4,
                    min_value: Some("CANCELLED".to_string()),
                    max_value: Some("PENDING".to_string()),
                    avg_length: Some(8.5),
                },
            ],
            last_updated: Utc::now().to_rfc3339(),
        },
        "users" => TableStats {
            table_name: "users".to_string(),
            row_count: 50_000,
            size_bytes: 5_000_000,
            column_stats: vec![
                ColumnStats {
                    column_name: "id".to_string(),
                    data_type: "BIGINT".to_string(),
                    null_count: 0,
                    distinct_count: 50_000,
                    min_value: Some("1".to_string()),
                    max_value: Some("50000".to_string()),
                    avg_length: None,
                },
                ColumnStats {
                    column_name: "username".to_string(),
                    data_type: "VARCHAR(50)".to_string(),
                    null_count: 0,
                    distinct_count: 50_000,
                    min_value: Some("admin".to_string()),
                    max_value: Some("zzzuser".to_string()),
                    avg_length: Some(12.5),
                },
                ColumnStats {
                    column_name: "email".to_string(),
                    data_type: "VARCHAR(100)".to_string(),
                    null_count: 0,
                    distinct_count: 50_000,
                    min_value: Some("admin@example.com".to_string()),
                    max_value: Some("zzzuser@example.com".to_string()),
                    avg_length: Some(25.3),
                },
            ],
            last_updated: Utc::now().to_rfc3339(),
        },
        _ => {
            error!("表 {} 不存在", table_name);
            return Ok(error_response("表不存在", 404));
        }
    };

    info!("成功获取表 {} 的统计信息，包含 {} 个列的统计", table_name, stats.column_stats.len());
    Ok(success_response(stats))
}

/// 预览表数据（前100行）
#[instrument(skip(app_state))]
pub async fn preview_table(
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> ActixResult<HttpResponse> {
    let table_name = path.into_inner();
    info!("预览表 {} 的数据", table_name);

    // 重用get_table_data逻辑，但固定limit为100
    let query = TableDataQuery {
        limit: Some(100),
        offset: Some(0),
        order_by: None,
        order_direction: None,
        filters: None,
    };

    get_table_data(app_state, web::Path::from(table_name), web::Query(query)).await
}

/// 获取Schema演进历史
#[instrument(skip(_app_state))]
pub async fn get_schema_evolution(
    _app_state: web::Data<AppState>,
    query: web::Query<SchemaEvolutionQuery>
) -> ActixResult<HttpResponse> {
    info!("获取Schema演进历史，表: {:?}", query.table);

    // 模拟Schema演进历史数据
    let schema_versions = vec![
        SchemaVersion {
            version: 5,
            timestamp: "2024-01-11 14:30:25".to_string(),
            author: "admin".to_string(),
            description: "添加交易状态字段".to_string(),
            compatibility: "backward".to_string(),
            changes: vec![
                SchemaChange {
                    change_type: "add_column".to_string(),
                    table: "transactions".to_string(),
                    column: Some("status".to_string()),
                    old_definition: None,
                    new_definition: Some("VARCHAR(50) NOT NULL DEFAULT 'pending'".to_string()),
                    description: "添加交易状态字段，支持pending/completed/failed状态".to_string(),
                    impact: "low".to_string(),
                }
            ],
        },
        SchemaVersion {
            version: 4,
            timestamp: "2024-01-10 16:20:15".to_string(),
            author: "developer".to_string(),
            description: "修改金额字段精度".to_string(),
            compatibility: "breaking".to_string(),
            changes: vec![
                SchemaChange {
                    change_type: "modify_column".to_string(),
                    table: "transactions".to_string(),
                    column: Some("amount".to_string()),
                    old_definition: Some("DECIMAL(8,2)".to_string()),
                    new_definition: Some("DECIMAL(10,2)".to_string()),
                    description: "增加金额字段精度以支持更大金额".to_string(),
                    impact: "medium".to_string(),
                }
            ],
        },
        SchemaVersion {
            version: 3,
            timestamp: "2024-01-09 10:15:30".to_string(),
            author: "dba".to_string(),
            description: "添加索引优化查询性能".to_string(),
            compatibility: "full".to_string(),
            changes: vec![
                SchemaChange {
                    change_type: "add_index".to_string(),
                    table: "transactions".to_string(),
                    column: Some("user_id".to_string()),
                    old_definition: None,
                    new_definition: Some("INDEX idx_user_id (user_id)".to_string()),
                    description: "为user_id字段添加索引".to_string(),
                    impact: "low".to_string(),
                }
            ],
        },
        SchemaVersion {
            version: 2,
            timestamp: "2024-01-08 14:45:20".to_string(),
            author: "admin".to_string(),
            description: "初始Schema设计".to_string(),
            compatibility: "full".to_string(),
            changes: vec![
                SchemaChange {
                    change_type: "add_column".to_string(),
                    table: "transactions".to_string(),
                    column: Some("created_at".to_string()),
                    old_definition: None,
                    new_definition: Some("TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP".to_string()),
                    description: "添加创建时间字段".to_string(),
                    impact: "low".to_string(),
                }
            ],
        },
    ];

    // 根据查询参数过滤
    let filtered_versions: Vec<SchemaVersion> = if let Some(table_name) = &query.table {
        schema_versions.into_iter()
            .filter(|v| v.changes.iter().any(|c| c.table == *table_name))
            .collect()
    } else {
        schema_versions
    };

    // 应用分页
    let limit = query.limit.unwrap_or(50) as usize;
    let offset = query.offset.unwrap_or(0) as usize;
    let paginated_versions: Vec<SchemaVersion> = filtered_versions
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    info!("成功获取Schema演进历史，包含 {} 个版本", paginated_versions.len());
    Ok(success_response(paginated_versions))
}

/// 获取特定表的Schema演进历史
#[instrument(skip(_app_state))]
pub async fn get_table_schema_evolution(
    _app_state: web::Data<AppState>,
    path: web::Path<String>
) -> ActixResult<HttpResponse> {
    let table_name = path.into_inner();
    info!("获取表 {} 的Schema演进历史", table_name);

    let query = SchemaEvolutionQuery {
        table: Some(table_name.clone()),
        limit: Some(20),
        offset: Some(0),
    };

    get_schema_evolution(_app_state, web::Query(query)).await
}

/// 获取真实的表列表
async fn get_real_table_list(engine: &Arc<DuckDBEngine>) -> Result<Vec<TableInfo>, Box<dyn std::error::Error>> {
    // 查询系统表获取真实的表信息
    let sql = r#"
        SELECT
            table_name,
            estimated_size,
            table_type,
            table_comment
        FROM information_schema.tables
        WHERE table_schema = 'main'
        ORDER BY table_name
    "#;

    match engine.query(sql).await {
        Ok(rows) => {
            let mut tables = Vec::new();
            for row in rows {
                if let Some(table_name) = row.get("table_name") {
                    let table_name_str = table_name.to_string();

                    // 获取表的行数
                    let row_count = get_table_row_count(engine, &table_name_str).await.unwrap_or(0);

                    // 获取表的列数
                    let column_count = get_table_column_count(engine, &table_name_str).await.unwrap_or(0);

                    let size_bytes = row.get("estimated_size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let table_type = row.get("table_type").map(|v| v.to_string()).unwrap_or_else(|| "TABLE".to_string());
                    let description = row.get("table_comment").map(|v| v.to_string());

                    tables.push(TableInfo {
                        name: table_name_str,
                        rows: row_count,
                        size: format_size(size_bytes),
                        schema_version: 1, // 默认版本
                        last_modified: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        description,
                        size_bytes,
                        column_count,
                        table_type,
                    });
                }
            }
            Ok(tables)
        },
        Err(_) => {
            // 如果查询失败，返回空列表
            Ok(vec![])
        }
    }
}

/// 获取表的行数
async fn get_table_row_count(engine: &Arc<DuckDBEngine>, table_name: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let sql = format!("SELECT COUNT(*) FROM {}", table_name);
    match engine.query(&sql).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                if let Some(count) = row.get("count") {
                    return Ok(count.as_u64().unwrap_or(0));
                }
            }
            Ok(0)
        },
        Err(_) => Ok(0),
    }
}

/// 获取表的列数
async fn get_table_column_count(engine: &Arc<DuckDBEngine>, table_name: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let sql = format!(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = '{}'",
        table_name
    );
    match engine.query(&sql).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                if let Some(count) = row.get("count") {
                    return Ok(count.as_u64().unwrap_or(0) as u32);
                }
            }
            Ok(0)
        },
        Err(_) => Ok(0),
    }
}

/// 从真实数据库获取表数据
async fn get_real_table_data(
    engine: &Arc<DuckDBEngine>,
    table_name: &str,
    limit: u32,
    offset: u32
) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>, u64), Box<dyn std::error::Error>> {
    // 首先检查表是否存在
    let check_sql = format!(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = '{}'",
        table_name
    );

    match engine.query(&check_sql).await {
        Ok(rows) => {
            if let Some(row) = rows.first() {
                if let Some(count) = row.get("count") {
                    if count.as_u64().unwrap_or(0) == 0 {
                        return Err(format!("表 {} 不存在", table_name).into());
                    }
                }
            }
        }
        Err(e) => return Err(format!("检查表存在性失败: {}", e).into()),
    }

    // 获取表的列信息
    let columns_sql = format!(
        "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
        table_name
    );

    let columns = match engine.query(&columns_sql).await {
        Ok(rows) => {
            rows.iter()
                .filter_map(|row| row.get("column_name").map(|v| v.to_string()))
                .collect::<Vec<String>>()
        }
        Err(e) => return Err(format!("获取列信息失败: {}", e).into()),
    };

    if columns.is_empty() {
        return Ok((vec![], vec![], 0));
    }

    // 获取总行数
    let count_sql = format!("SELECT COUNT(*) as total FROM {}", table_name);
    let total_rows = match engine.query(&count_sql).await {
        Ok(rows) => {
            rows.first()
                .and_then(|row| row.get("total"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
        }
        Err(_) => 0,
    };

    // 获取数据
    let data_sql = format!(
        "SELECT * FROM {} LIMIT {} OFFSET {}",
        table_name, limit, offset
    );

    let data = match engine.query(&data_sql).await {
        Ok(rows) => {
            rows.iter()
                .map(|row| {
                    columns.iter()
                        .map(|col| {
                            row.get(col)
                                .map(|v| match v {
                                    serde_json::Value::String(s) => serde_json::Value::String(s.clone()),
                                    serde_json::Value::Number(n) => serde_json::Value::Number(n.clone()),
                                    serde_json::Value::Bool(b) => serde_json::Value::Bool(*b),
                                    serde_json::Value::Null => serde_json::Value::Null,
                                    _ => serde_json::Value::String(v.to_string()),
                                })
                                .unwrap_or(serde_json::Value::Null)
                        })
                        .collect()
                })
                .collect()
        }
        Err(e) => return Err(format!("获取表数据失败: {}", e).into()),
    };

    Ok((columns, data, total_rows))
}
