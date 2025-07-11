// 数据探索API处理器
// 提供数据库表信息、Schema查询、数据预览等功能

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::{AppState, success_response, error_response};

/// 表信息结构
#[derive(Debug, Serialize, Clone)]
pub struct TableInfo {
    pub name: String,
    pub row_count: u64,
    pub size_bytes: u64,
    pub column_count: u32,
    pub created_at: String,
    pub table_type: String,
    pub description: Option<String>,
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
    
    // 模拟数据 - 在实际实现中应该从DuckDB获取
    let tables = vec![
        TableInfo {
            name: "transactions".to_string(),
            row_count: 1_250_000,
            size_bytes: 125_000_000,
            column_count: 8,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            table_type: "TABLE".to_string(),
            description: Some("金融交易记录表".to_string()),
        },
        TableInfo {
            name: "users".to_string(),
            row_count: 50_000,
            size_bytes: 5_000_000,
            column_count: 12,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            table_type: "TABLE".to_string(),
            description: Some("用户信息表".to_string()),
        },
        TableInfo {
            name: "accounts".to_string(),
            row_count: 75_000,
            size_bytes: 7_500_000,
            column_count: 10,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            table_type: "TABLE".to_string(),
            description: Some("账户信息表".to_string()),
        },
        TableInfo {
            name: "market_data".to_string(),
            row_count: 5_000_000,
            size_bytes: 500_000_000,
            column_count: 15,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            table_type: "TABLE".to_string(),
            description: Some("市场数据表".to_string()),
        },
    ];
    
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

    // 模拟表数据
    let (columns, data, total_rows) = match table_name.as_str() {
        "transactions" => {
            let columns = vec![
                "id".to_string(),
                "user_id".to_string(),
                "amount".to_string(),
                "currency".to_string(),
                "transaction_type".to_string(),
                "status".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
            ];

            let mut data = Vec::new();
            for i in (offset as u64 + 1)..=(offset as u64 + limit as u64).min(offset as u64 + 50) {
                data.push(vec![
                    serde_json::Value::Number(serde_json::Number::from(i)),
                    serde_json::Value::Number(serde_json::Number::from(i % 1000 + 1)),
                    serde_json::Value::String(format!("{:.2}", (i as f64 * 123.45) % 10000.0)),
                    serde_json::Value::String("USD".to_string()),
                    serde_json::Value::String(if i % 2 == 0 { "DEPOSIT" } else { "WITHDRAWAL" }.to_string()),
                    serde_json::Value::String(if i % 3 == 0 { "COMPLETED" } else { "PENDING" }.to_string()),
                    serde_json::Value::String(format!("2024-01-{:02}T{:02}:00:00Z", (i % 30) + 1, (i % 24))),
                    if i % 4 == 0 {
                        serde_json::Value::String(format!("2024-01-{:02}T{:02}:30:00Z", (i % 30) + 1, (i % 24)))
                    } else {
                        serde_json::Value::Null
                    },
                ]);
            }

            (columns, data, 1_250_000u64)
        },
        "users" => {
            let columns = vec![
                "id".to_string(),
                "username".to_string(),
                "email".to_string(),
                "first_name".to_string(),
                "last_name".to_string(),
                "created_at".to_string(),
            ];

            let mut data = Vec::new();
            for i in (offset as u64 + 1)..=(offset as u64 + limit as u64).min(offset as u64 + 20) {
                data.push(vec![
                    serde_json::Value::Number(serde_json::Number::from(i)),
                    serde_json::Value::String(format!("user{}", i)),
                    serde_json::Value::String(format!("user{}@example.com", i)),
                    serde_json::Value::String(format!("First{}", i)),
                    serde_json::Value::String(format!("Last{}", i)),
                    serde_json::Value::String(format!("2024-01-{:02}T10:00:00Z", (i % 30) + 1)),
                ]);
            }

            (columns, data, 50_000u64)
        },
        _ => {
            error!("表 {} 不存在", table_name);
            return Ok(error_response("表不存在", 404));
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
