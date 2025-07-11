//! 数据模型

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// API响应基础结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 错误代码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    /// 消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// 请求ID
    pub request_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            code: None,
            message: Some("操作成功".to_string()),
            request_id,
            timestamp: Utc::now(),
        }
    }

    /// 创建错误响应
    pub fn error(error: String, code: u16, request_id: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            code: Some(code),
            message: None,
            request_id,
            timestamp: Utc::now(),
        }
    }
}

/// 分页信息
#[derive(Debug, Serialize)]
pub struct Pagination {
    /// 当前页码
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总记录数
    pub total: u64,
    /// 总页数
    pub total_pages: u32,
    /// 是否有下一页
    pub has_next: bool,
    /// 是否有上一页
    pub has_prev: bool,
}

/// 分页响应
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// 数据列表
    pub data: Vec<T>,
    /// 分页信息
    pub pagination: Pagination,
}

/// 用户模型
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// 用户ID
    pub id: Uuid,
    /// 用户名
    pub username: String,
    /// 邮箱
    pub email: String,
    /// 显示名称
    pub display_name: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 最后登录时间
    pub last_login_at: Option<DateTime<Utc>>,
}

/// 查询模型
#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    /// 查询ID
    pub id: Uuid,
    /// SQL语句
    pub sql: String,
    /// 查询参数
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    /// 超时时间
    pub timeout: Option<std::time::Duration>,
    /// 结果限制
    pub limit: Option<u32>,
    /// 用户ID
    pub user_id: Option<Uuid>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 查询结果
#[derive(Debug, Serialize)]
pub struct QueryResult {
    /// 查询ID
    pub query_id: Uuid,
    /// 列信息
    pub columns: Vec<Column>,
    /// 数据行
    pub rows: Vec<serde_json::Value>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 是否来自缓存
    pub from_cache: bool,
}

/// 列信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可为空
    pub nullable: bool,
}

/// 数据源模型
#[derive(Debug, Serialize, Deserialize)]
pub struct DataSource {
    /// 数据源ID
    pub id: Uuid,
    /// 名称
    pub name: String,
    /// 类型
    pub source_type: String,
    /// 状态
    pub status: String,
    /// 配置
    pub config: serde_json::Value,
    /// 描述
    pub description: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 监控指标
#[derive(Debug, Serialize)]
pub struct Metrics {
    /// 总查询数
    pub total_queries: u64,
    /// 活跃连接数
    pub active_connections: u32,
    /// 处理的数据量（字节）
    pub data_processed_bytes: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// CPU使用率
    pub cpu_usage: f64,
    /// 内存使用率
    pub memory_usage: f64,
    /// 磁盘使用率
    pub disk_usage: f64,
}

/// 告警信息
#[derive(Debug, Serialize)]
pub struct Alert {
    /// 告警ID
    pub id: Uuid,
    /// 告警类型
    pub alert_type: String,
    /// 严重程度
    pub severity: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 状态
    pub status: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 活动记录
#[derive(Debug, Serialize)]
pub struct Activity {
    /// 活动ID
    pub id: Uuid,
    /// 活动类型
    pub activity_type: String,
    /// 描述
    pub description: String,
    /// 用户ID
    pub user_id: Option<Uuid>,
    /// 严重程度
    pub severity: String,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 系统状态
#[derive(Debug, Serialize)]
pub struct SystemStatus {
    /// 整体状态
    pub overall_status: String,
    /// 组件状态
    pub components: std::collections::HashMap<String, ComponentStatus>,
    /// 最后检查时间
    pub last_check: DateTime<Utc>,
}

/// 组件状态
#[derive(Debug, Serialize)]
pub struct ComponentStatus {
    /// 状态
    pub status: String,
    /// 响应时间（毫秒）
    pub response_time_ms: Option<f64>,
    /// 错误信息
    pub error: Option<String>,
    /// 最后检查时间
    pub last_check: DateTime<Utc>,
}

/// 错误详情
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    pub details: Option<serde_json::Value>,
    /// 堆栈跟踪
    pub stack_trace: Option<String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let data = "test data";
        let request_id = "req-123".to_string();
        let response = ApiResponse::success(data, request_id.clone());
        
        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert_eq!(response.request_id, request_id);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let error = "Test error".to_string();
        let code = 400;
        let request_id = "req-123".to_string();
        let response = ApiResponse::error(error.clone(), code, request_id.clone());
        
        assert!(!response.success);
        assert_eq!(response.error, Some(error));
        assert_eq!(response.code, Some(code));
        assert_eq!(response.request_id, request_id);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination {
            page: 2,
            page_size: 10,
            total: 25,
            total_pages: 3,
            has_next: true,
            has_prev: true,
        };
        
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
    }
}
