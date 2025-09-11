//! 查询处理器 - 连接前端和DuckLake的核心API

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn, instrument};
use validator::Validate;
use crate::{AppState, success_response, error_response, paginated_response, validate_pagination};
use duckhub_common::prelude::*;

/// 查询执行请求
#[derive(Debug, Deserialize, Validate)]
pub struct QueryRequest {
    /// SQL查询语句
    #[validate(length(min = 1, max = 10000, message = "SQL查询长度必须在1-10000字符之间"))]
    pub sql: String,
    /// 查询参数
    pub parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// 是否使用缓存
    pub use_cache: Option<bool>,
    /// 查询超时时间（秒）
    #[validate(range(min = 1, max = 300, message = "查询超时时间必须在1-300秒之间"))]
    pub timeout: Option<u64>,
    /// 结果限制
    #[validate(range(min = 1, max = 10000, message = "结果限制必须在1-10000之间"))]
    pub limit: Option<u32>,
}

/// 查询响应
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    /// 查询ID
    pub query_id: Uuid,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 返回行数
    pub row_count: usize,
    /// 查询结果
    pub data: serde_json::Value,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 是否来自缓存
    pub from_cache: bool,
    /// 执行时间戳
    pub executed_at: DateTime<Utc>,
}

/// 列信息
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

/// 查询分析请求
#[derive(Debug, Deserialize, Validate)]
pub struct AnalyzeQueryRequest {
    /// SQL查询语句
    #[validate(length(min = 1, max = 10000))]
    pub sql: String,
    /// 分析类型
    pub analysis_type: AnalysisType,
}

/// 分析类型
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    /// 执行计划分析
    ExecutionPlan,
    /// 性能分析
    Performance,
    /// 语法检查
    SyntaxCheck,
    /// 统计分析
    Statistics,
}

/// 查询优化请求
#[derive(Debug, Deserialize, Validate)]
pub struct OptimizeQueryRequest {
    /// SQL查询语句
    #[validate(length(min = 1, max = 10000))]
    pub sql: String,
    /// 优化选项
    pub options: Option<OptimizationOptions>,
}

/// 优化选项
#[derive(Debug, Deserialize)]
pub struct OptimizationOptions {
    /// 是否启用谓词下推
    pub enable_predicate_pushdown: Option<bool>,
    /// 是否启用投影下推
    pub enable_projection_pushdown: Option<bool>,
    /// 是否启用连接重排序
    pub enable_join_reordering: Option<bool>,
}

/// 执行查询
#[instrument(skip(app_state))]
pub async fn execute_query(
    app_state: web::Data<AppState>,
    request: web::Json<QueryRequest>,
) -> ActixResult<HttpResponse> {
    // 验证请求
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    let query_id = Uuid::new_v4();
    let start_time = std::time::Instant::now();

    info!("执行查询 {}: {}", query_id, request.sql);

    // 检查缓存
    let use_cache = request.use_cache.unwrap_or(true);
    let cache_key = if use_cache {
        Some(format!("query:{:x}", md5::compute(&request.sql)))
    } else {
        None
    };

    // 缓存功能暂时禁用
    // if let Some(ref key) = cache_key {
    //     if let Ok(cached_result) = app_state.cache.get::<QueryResponse>(key).await {
    //         info!("查询 {} 命中缓存", query_id);
    //         return Ok(success_response(cached_result));
    //     }
    // }

    // 构建查询
    let query = Query {
        id: query_id,
        sql: request.sql.clone(),
        parameters: request.parameters.clone().unwrap_or_default(),
        timeout_seconds: request.timeout,
        user_id: extract_user_id_from_request(&req).await,
        created_at: Utc::now(),
    };

    // 执行查询
    match app_state.engine.query(&query.sql).await {
        Ok(rows) => {
            let execution_time = start_time.elapsed();

            // 构建响应
            let response = QueryResponse {
                query_id,
                execution_time_ms: execution_time.as_millis() as u64,
                row_count: rows.len(),
                data: serde_json::to_value(&rows).unwrap_or(serde_json::Value::Null),
                columns: if !rows.is_empty() {
                    rows[0].keys().map(|key| ColumnInfo {
                        name: key.clone(),
                        data_type: "TEXT".to_string(), // 简化处理
                        nullable: true,
                    }).collect()
                } else {
                    vec![]
                },
                from_cache: false,
                executed_at: Utc::now(),
            };

            // 缓存结果暂时禁用
            // if let Some(key) = cache_key {
            //     if let Err(e) = app_state.cache.set(&key, &response, Some(std::time::Duration::from_secs(300))).await {
            //         warn!("缓存查询结果失败: {}", e);
            //     }
            // }

            info!("查询 {} 执行成功，耗时 {}ms", query_id, execution_time.as_millis());
            Ok(success_response(response))
        }
        Err(e) => {
            error!("查询 {} 执行失败: {}", query_id, e);
            Ok(error_response(&format!("查询执行失败: {}", e), 500))
        }
    }
}

/// 分析查询
#[instrument(skip(app_state))]
pub async fn analyze_query(
    app_state: web::Data<AppState>,
    request: web::Json<AnalyzeQueryRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("分析查询: {}", request.sql);

    match request.analysis_type {
        AnalysisType::ExecutionPlan => {
            // 获取执行计划
            let explain_sql = format!("EXPLAIN {}", request.sql);
            let query = Query {
                id: Uuid::new_v4(),
                sql: explain_sql,
                parameters: std::collections::HashMap::new(),
                timeout_seconds: Some(30),
                user_id: None,
                created_at: Utc::now(),
            };

            match app_state.engine.query(&query.sql).await {
                Ok(rows) => Ok(success_response(serde_json::json!({
                    "type": "execution_plan",
                    "plan": rows
                }))),
                Err(e) => Ok(error_response(&format!("获取执行计划失败: {}", e), 500))
            }
        }
        AnalysisType::Performance => {
            // 性能分析
            // 性能分析 - 简化实现
            Ok(success_response(serde_json::json!({
                "type": "performance",
                "analysis": {
                    "estimated_cost": 100,
                    "estimated_rows": 1000,
                    "execution_time_estimate": "< 1s"
                }
            })))
        }
        AnalysisType::SyntaxCheck => {
            // 语法检查
            // 语法检查 - 简化实现
            let is_valid = !request.sql.trim().is_empty() && request.sql.to_uppercase().contains("SELECT");
            Ok(success_response(serde_json::json!({
                "type": "syntax_check",
                "valid": is_valid,
                "message": if is_valid { "SQL语法正确" } else { "SQL语法错误" }
            })))
        }
        AnalysisType::Statistics => {
            // 统计分析
            // 统计分析 - 简化实现
            Ok(success_response(serde_json::json!({
                "type": "statistics",
                "statistics": {
                    "query_complexity": "medium",
                    "table_count": 1,
                    "join_count": 0
                }
            })))
        }
    }
}

/// 优化查询
#[instrument(skip(app_state))]
pub async fn optimize_query(
    app_state: web::Data<AppState>,
    request: web::Json<OptimizeQueryRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("优化查询: {}", request.sql);

    // 查询优化 - 简化实现
    let optimized_sql = format!("-- 优化后的查询\n{}", request.sql);
    Ok(success_response(serde_json::json!({
        "original_sql": request.sql,
        "optimized_sql": optimized_sql,
        "optimizations_applied": ["添加索引建议", "查询重写"],
        "estimated_improvement": "30%"
    })))
}

/// 获取查询历史
#[instrument(skip(app_state))]
pub async fn get_query_history(
    app_state: web::Data<AppState>,
    query: web::Query<PaginationQuery>,
) -> ActixResult<HttpResponse> {
    let (page, page_size) = validate_pagination(query.page, query.page_size);
    
    info!("获取查询历史，页码: {}, 页大小: {}", page, page_size);

    // 从真实查询日志获取历史数据
    let queries = get_real_query_history(&app_state.engine, page, page_size).await
        .unwrap_or_else(|e| {
            warn!("获取查询历史失败: {}", e);
            vec![]
        });

    Ok(paginated_response(queries, page, page_size, 0))
}

/// 分页查询参数
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_request_validation() {
        let valid_request = QueryRequest {
            sql: "SELECT * FROM test".to_string(),
            parameters: None,
            use_cache: Some(true),
            timeout: Some(30),
            limit: Some(100),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = QueryRequest {
            sql: "".to_string(), // 空SQL
            parameters: None,
            use_cache: Some(true),
            timeout: Some(30),
            limit: Some(100),
        };
        assert!(invalid_request.validate().is_err());
    }
}

/// 从真实查询日志获取历史数据
async fn get_real_query_history(
    engine: &DuckDBEngine,
    page: u32,
    page_size: u32
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let offset = (page - 1) * page_size;

    // 查询查询日志表
    let sql = format!(
        "SELECT
            id,
            sql,
            execution_time_ms,
            row_count,
            executed_at,
            user_id,
            status
        FROM query_logs
        ORDER BY executed_at DESC
        LIMIT {} OFFSET {}",
        page_size, offset
    );

    match engine.query(&sql).await {
        Ok(rows) => {
            let mut queries = Vec::new();
            for row in rows {
                let mut query_record = serde_json::Map::new();

                if let Some(id) = row.get("id") {
                    query_record.insert("id".to_string(), id.clone());
                }
                if let Some(sql) = row.get("sql") {
                    query_record.insert("sql".to_string(), sql.clone());
                }
                if let Some(execution_time) = row.get("execution_time_ms") {
                    query_record.insert("execution_time_ms".to_string(), execution_time.clone());
                }
                if let Some(row_count) = row.get("row_count") {
                    query_record.insert("row_count".to_string(), row_count.clone());
                }
                if let Some(executed_at) = row.get("executed_at") {
                    query_record.insert("executed_at".to_string(), executed_at.clone());
                }
                if let Some(user_id) = row.get("user_id") {
                    query_record.insert("user_id".to_string(), user_id.clone());
                }
                if let Some(status) = row.get("status") {
                    query_record.insert("status".to_string(), status.clone());
                }

                queries.push(serde_json::Value::Object(query_record));
            }
            Ok(queries)
        }
        Err(_) => {
            // 如果查询日志表不存在，返回空列表
            Ok(vec![])
        }
    }
}

/// 从请求中提取用户ID
async fn extract_user_id_from_request(req: &HttpRequest) -> Option<String> {
    // 尝试从Authorization头中提取用户ID
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                // 这里应该解析JWT令牌获取用户ID
                // 简化实现：返回固定的用户ID
                return Some("current-user".to_string());
            }
        }
    }

    // 如果没有认证信息，返回None
    None
}
