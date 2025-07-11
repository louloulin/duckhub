//! DuckLake处理器 - 时间旅行和Schema管理

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, error, instrument};
use validator::Validate;
use crate::{AppState, success_response, error_response};

/// 数据库信息
#[derive(Debug, Serialize)]
pub struct DatabaseInfo {
    /// 数据库名称
    pub name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 表数量
    pub table_count: u32,
    /// 数据大小（字节）
    pub size_bytes: u64,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 快照信息
#[derive(Debug, Serialize)]
pub struct SnapshotInfo {
    /// 快照ID
    pub id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 描述
    pub description: Option<String>,
    /// 数据大小（字节）
    pub size_bytes: u64,
    /// 表数量
    pub table_count: u32,
}

/// 时间旅行查询请求
#[derive(Debug, Deserialize, Validate)]
pub struct TimeTravelQueryRequest {
    /// SQL查询语句
    #[validate(length(min = 1, max = 10000, message = "SQL查询长度必须在1-10000字符之间"))]
    pub sql: String,
    /// 目标时间戳或快照ID
    pub target: TimeTravelTarget,
    /// 查询参数
    pub parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// 时间旅行目标
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimeTravelTarget {
    /// 时间戳
    #[serde(rename = "timestamp")]
    Timestamp(DateTime<Utc>),
    /// 快照ID
    #[serde(rename = "snapshot")]
    Snapshot(String),
    /// 版本号
    #[serde(rename = "version")]
    Version(u64),
}

/// Schema信息
#[derive(Debug, Serialize)]
pub struct SchemaInfo {
    /// 数据库名称
    pub database: String,
    /// 表列表
    pub tables: Vec<TableInfo>,
    /// Schema版本
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 表信息
#[derive(Debug, Serialize)]
pub struct TableInfo {
    /// 表名
    pub name: String,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 行数
    pub row_count: u64,
    /// 数据大小（字节）
    pub size_bytes: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 列信息
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可为空
    pub nullable: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 注释
    pub comment: Option<String>,
}

/// 列出数据库
#[instrument(skip(app_state))]
pub async fn list_databases(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("列出数据库");

    match app_state.engine.list_databases().await {
        Ok(databases) => {
            let db_info: Vec<DatabaseInfo> = databases.into_iter().map(|db| {
                DatabaseInfo {
                    name: db.name,
                    created_at: db.created_at,
                    table_count: 0, // Mock value - field not available in DatabaseInfo
                    size_bytes: db.size.parse::<u64>().unwrap_or(0),
                    last_updated: db.last_accessed.unwrap_or(db.created_at),
                }
            }).collect();

            Ok(success_response(db_info))
        }
        Err(e) => {
            error!("列出数据库失败: {}", e);
            Ok(error_response("列出数据库失败", 500))
        }
    }
}

/// 列出快照
#[instrument(skip(app_state))]
pub async fn list_snapshots(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let database_name = path.into_inner();
    info!("列出数据库 {} 的快照", database_name);

    match app_state.engine.list_snapshots(&database_name).await {
        Ok(snapshots) => {
            let snapshot_info: Vec<SnapshotInfo> = snapshots.into_iter().map(|snapshot| {
                SnapshotInfo {
                    id: snapshot.id,
                    created_at: snapshot.created_at,
                    description: snapshot.description,
                    size_bytes: snapshot.size_bytes,
                    table_count: snapshot.table_count,
                }
            }).collect();

            Ok(success_response(snapshot_info))
        }
        Err(e) => {
            error!("列出快照失败: {}", e);
            Ok(error_response("列出快照失败", 500))
        }
    }
}

/// 时间旅行查询
#[instrument(skip(app_state, request))]
pub async fn time_travel_query(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    request: web::Json<TimeTravelQueryRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    let database_name = path.into_inner();
    info!("执行时间旅行查询，数据库: {}", database_name);

    let time_travel_request = duckhub_common::types::TimeTravelQueryRequest {
        database: database_name,
        table: "".to_string(), // 从SQL中提取表名或使用默认值
        target: match &request.target {
            TimeTravelTarget::Timestamp(ts) => duckhub_common::types::TimeTravelTarget::Timestamp(*ts),
            TimeTravelTarget::Snapshot(id) => duckhub_common::types::TimeTravelTarget::Version(id.parse().unwrap_or(0)),
            TimeTravelTarget::Version(v) => duckhub_common::types::TimeTravelTarget::Version(*v),
        },
        sql: request.sql.clone(),
    };

    match app_state.engine.execute_time_travel_query(time_travel_request).await {
        Ok(result) => {
            let response = serde_json::json!({
                "query_id": result.query_id,
                "execution_time_ms": result.execution_time_ms,
                "row_count": result.row_count,
                "data": result.results,
                "target_info": {
                    "type": match request.target {
                        TimeTravelTarget::Timestamp(_) => "timestamp",
                        TimeTravelTarget::Snapshot(_) => "snapshot",
                        TimeTravelTarget::Version(_) => "version",
                    }
                }
            });

            Ok(success_response(response))
        }
        Err(e) => {
            error!("时间旅行查询失败: {}", e);
            Ok(error_response(&format!("时间旅行查询失败: {}", e), 500))
        }
    }
}

/// 获取Schema信息
#[instrument(skip(app_state))]
pub async fn get_schema(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let database_name = path.into_inner();
    info!("获取数据库 {} 的Schema信息", database_name);

    match app_state.engine.get_schema(&database_name).await {
        Ok(schema) => {
            let tables: Vec<TableInfo> = schema.tables.into_iter().map(|table| {
                let columns: Vec<ColumnInfo> = table.columns.into_iter().map(|col| {
                    ColumnInfo {
                        name: col.name,
                        data_type: col.data_type,
                        nullable: col.nullable,
                        default_value: col.default_value,
                        comment: col.comment,
                    }
                }).collect();

                TableInfo {
                    name: table.name,
                    columns,
                    row_count: table.row_count,
                    size_bytes: table.size_bytes,
                    created_at: table.created_at,
                    last_updated: table.last_updated,
                }
            }).collect();

            let schema_info = SchemaInfo {
                database: database_name,
                tables,
                version: schema.version as u32,
                last_updated: schema.last_updated,
            };

            Ok(success_response(schema_info))
        }
        Err(e) => {
            error!("获取Schema信息失败: {}", e);
            Ok(error_response("获取Schema信息失败", 500))
        }
    }
}

/// 创建快照请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSnapshotRequest {
    /// 快照描述
    #[validate(length(max = 500, message = "描述长度不能超过500字符"))]
    pub description: Option<String>,
    /// 是否包含所有表
    pub include_all_tables: Option<bool>,
    /// 指定表列表（如果不包含所有表）
    pub tables: Option<Vec<String>>,
}

/// 创建快照
#[instrument(skip(app_state, request))]
pub async fn create_snapshot(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    request: web::Json<CreateSnapshotRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    let database_name = path.into_inner();
    info!("为数据库 {} 创建快照", database_name);

    let create_request = duckhub_common::types::CreateSnapshotRequest {
        database: database_name,
        table: None,
        description: request.description.clone(),
        include_all_tables: request.include_all_tables,
        tables: request.tables.clone(),
        compression_level: Some(6), // 默认压缩级别
        include_metadata: Some(true), // 默认包含元数据
    };

    match app_state.engine.create_snapshot(create_request).await {
        Ok(snapshot) => {
            let snapshot_info = SnapshotInfo {
                id: snapshot.id,
                created_at: snapshot.created_at,
                description: snapshot.description,
                size_bytes: snapshot.size_bytes,
                table_count: snapshot.table_count,
            };

            Ok(success_response(snapshot_info))
        }
        Err(e) => {
            error!("创建快照失败: {}", e);
            Ok(error_response(&format!("创建快照失败: {}", e), 500))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_travel_query_request_validation() {
        let valid_request = TimeTravelQueryRequest {
            sql: "SELECT * FROM users".to_string(),
            target: TimeTravelTarget::Timestamp(Utc::now()),
            parameters: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = TimeTravelQueryRequest {
            sql: "".to_string(), // 空SQL
            target: TimeTravelTarget::Version(1),
            parameters: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_create_snapshot_request_validation() {
        let valid_request = CreateSnapshotRequest {
            description: Some("Test snapshot".to_string()),
            include_all_tables: Some(true),
            tables: None,
        };
        assert!(valid_request.validate().is_ok());
    }
}
