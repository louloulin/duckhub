//! 审计日志模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, error, instrument};

/// 审计管理器
pub struct AuditManager {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 审计配置
    config: AuditConfig,
}

/// 审计配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 是否启用审计
    pub enabled: bool,
    /// 日志保留天数
    pub retention_days: u32,
    /// 是否记录查询操作
    pub log_queries: bool,
    /// 是否记录数据变更
    pub log_data_changes: bool,
    /// 是否记录认证事件
    pub log_auth_events: bool,
    /// 是否记录权限检查
    pub log_permission_checks: bool,
    /// 敏感字段列表
    pub sensitive_fields: Vec<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 365, // 1年
            log_queries: true,
            log_data_changes: true,
            log_auth_events: true,
            log_permission_checks: false, // 权限检查频繁，默认不记录
            sensitive_fields: vec![
                "password".to_string(),
                "credit_card".to_string(),
                "ssn".to_string(),
                "phone".to_string(),
            ],
        }
    }
}

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件ID
    pub id: String,
    /// 用户ID
    pub user_id: Option<String>,
    /// 操作类型
    pub action: String,
    /// 资源类型
    pub resource: String,
    /// 事件时间
    pub timestamp: DateTime<Utc>,
    /// 是否成功
    pub success: bool,
    /// 详细信息
    pub details: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
}

/// 审计查询结果
#[derive(Debug, Clone, Serialize)]
pub struct AuditQueryResult {
    /// 事件列表
    pub events: Vec<AuditEvent>,
    /// 总数
    pub total_count: u64,
    /// 页码
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
}

/// 审计查询条件
#[derive(Debug, Clone)]
pub struct AuditQuery {
    /// 用户ID
    pub user_id: Option<String>,
    /// 操作类型
    pub action: Option<String>,
    /// 资源类型
    pub resource: Option<String>,
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 是否成功
    pub success: Option<bool>,
    /// 页码
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self {
            user_id: None,
            action: None,
            resource: None,
            start_time: None,
            end_time: None,
            success: None,
            page: 1,
            page_size: 50,
        }
    }
}

impl AuditManager {
    /// 创建新的审计管理器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>, config: AuditConfig) -> Result<Self> {
        let manager = Self {
            engine,
            config,
        };

        // 初始化数据库表
        manager.initialize_tables().await?;

        info!("审计管理器初始化完成");
        Ok(manager)
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        let create_audit_sql = r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id VARCHAR PRIMARY KEY,
                user_id VARCHAR,
                action VARCHAR NOT NULL,
                resource VARCHAR NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                details TEXT,
                ip_address VARCHAR,
                user_agent TEXT
            )
        "#;
        self.engine.execute(create_audit_sql).await?;

        // 创建索引以提高查询性能
        let create_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_audit_user_id ON audit_logs(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action)",
            "CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_logs(resource)",
            "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_audit_success ON audit_logs(success)",
        ];

        for index_sql in &create_indexes {
            self.engine.execute(index_sql).await?;
        }

        info!("审计数据库表初始化完成");
        Ok(())
    }

    /// 记录审计事件
    #[instrument(skip(self))]
    pub async fn log_event(&self, event: AuditEvent) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // 过滤敏感信息
        let filtered_details = self.filter_sensitive_data(event.details.as_deref());

        let insert_sql = r#"
            INSERT INTO audit_logs (id, user_id, action, resource, timestamp, success, details, ip_address, user_agent)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        self.engine.execute_with_params(
            insert_sql,
            &[
                &event.id,
                &event.user_id.unwrap_or_default(),
                &event.action,
                &event.resource,
                &event.timestamp.to_rfc3339(),
                &event.success.to_string(),
                &filtered_details.unwrap_or_default(),
                &event.ip_address.unwrap_or_default(),
                &event.user_agent.unwrap_or_default(),
            ]
        ).await?;

        Ok(())
    }

    /// 查询审计日志
    #[instrument(skip(self))]
    pub async fn query_events(&self, query: AuditQuery) -> Result<AuditQueryResult> {
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();

        // 构建WHERE条件
        if let Some(user_id) = &query.user_id {
            where_clauses.push("user_id = ?");
            params.push(user_id.as_str());
        }

        if let Some(action) = &query.action {
            where_clauses.push("action = ?");
            params.push(action.as_str());
        }

        if let Some(resource) = &query.resource {
            where_clauses.push("resource = ?");
            params.push(resource.as_str());
        }

        if let Some(start_time) = &query.start_time {
            where_clauses.push("timestamp >= ?");
            params.push(&start_time.to_rfc3339());
        }

        if let Some(end_time) = &query.end_time {
            where_clauses.push("timestamp <= ?");
            params.push(&end_time.to_rfc3339());
        }

        if let Some(success) = query.success {
            where_clauses.push("success = ?");
            params.push(if success { "true" } else { "false" });
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM audit_logs {}", where_clause);
        let count_result = self.engine.query_with_params(&count_sql, &params).await?;
        let total_count = count_result.rows.get(0)
            .and_then(|row| row.get(0))
            .and_then(|v| v.as_ref())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // 查询数据
        let offset = (query.page - 1) * query.page_size;
        let data_sql = format!(
            "SELECT id, user_id, action, resource, timestamp, success, details, ip_address, user_agent 
             FROM audit_logs {} 
             ORDER BY timestamp DESC 
             LIMIT {} OFFSET {}",
            where_clause, query.page_size, offset
        );

        let data_result = self.engine.query_with_params(&data_sql, &params).await?;
        let mut events = Vec::new();

        for row in data_result.rows {
            if let Ok(event) = self.parse_audit_event_from_row(&row) {
                events.push(event);
            }
        }

        Ok(AuditQueryResult {
            events,
            total_count,
            page: query.page,
            page_size: query.page_size,
        })
    }

    /// 清理过期日志
    #[instrument(skip(self))]
    pub async fn cleanup_expired_logs(&self) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        let delete_sql = "DELETE FROM audit_logs WHERE timestamp < ?";
        let result = self.engine.execute_with_params(delete_sql, &[&cutoff_date.to_rfc3339()]).await?;
        
        info!("清理了过期的审计日志");
        Ok(result.rows_affected)
    }

    /// 获取审计统计信息
    pub async fn get_audit_stats(&self) -> Result<AuditStats> {
        let stats_sql = r#"
            SELECT 
                COUNT(*) as total_events,
                COUNT(CASE WHEN success = true THEN 1 END) as success_events,
                COUNT(CASE WHEN success = false THEN 1 END) as failed_events,
                COUNT(DISTINCT user_id) as unique_users,
                COUNT(DISTINCT action) as unique_actions
            FROM audit_logs
            WHERE timestamp >= datetime('now', '-30 days')
        "#;

        let result = self.engine.query(stats_sql).await?;
        if let Some(row) = result.rows.get(0) {
            Ok(AuditStats {
                total_events: row.get(0).and_then(|v| v.as_ref()).and_then(|s| s.parse().ok()).unwrap_or(0),
                success_events: row.get(1).and_then(|v| v.as_ref()).and_then(|s| s.parse().ok()).unwrap_or(0),
                failed_events: row.get(2).and_then(|v| v.as_ref()).and_then(|s| s.parse().ok()).unwrap_or(0),
                unique_users: row.get(3).and_then(|v| v.as_ref()).and_then(|s| s.parse().ok()).unwrap_or(0),
                unique_actions: row.get(4).and_then(|v| v.as_ref()).and_then(|s| s.parse().ok()).unwrap_or(0),
            })
        } else {
            Ok(AuditStats::default())
        }
    }

    /// 过滤敏感数据
    fn filter_sensitive_data(&self, details: Option<&str>) -> Option<String> {
        if let Some(details_str) = details {
            let mut filtered = details_str.to_string();
            
            for sensitive_field in &self.config.sensitive_fields {
                // 简单的敏感字段过滤，实际应该更复杂
                if filtered.to_lowercase().contains(&sensitive_field.to_lowercase()) {
                    filtered = filtered.replace(sensitive_field, "***");
                }
            }
            
            Some(filtered)
        } else {
            None
        }
    }

    /// 解析审计事件数据行
    fn parse_audit_event_from_row(&self, row: &[Option<String>]) -> Result<AuditEvent> {
        Ok(AuditEvent {
            id: row.get(0).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
            user_id: row.get(1).and_then(|v| v.as_ref()).map(|s| s.to_string()),
            action: row.get(2).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
            resource: row.get(3).and_then(|v| v.as_ref()).unwrap_or("").to_string(),
            timestamp: row.get(4)
                .and_then(|v| v.as_ref())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            success: row.get(5)
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            details: row.get(6).and_then(|v| v.as_ref()).map(|s| s.to_string()),
            ip_address: row.get(7).and_then(|v| v.as_ref()).map(|s| s.to_string()),
            user_agent: row.get(8).and_then(|v| v.as_ref()).map(|s| s.to_string()),
        })
    }
}

/// 审计统计信息
#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    /// 总事件数
    pub total_events: u64,
    /// 成功事件数
    pub success_events: u64,
    /// 失败事件数
    pub failed_events: u64,
    /// 唯一用户数
    pub unique_users: u64,
    /// 唯一操作数
    pub unique_actions: u64,
}

impl Default for AuditStats {
    fn default() -> Self {
        Self {
            total_events: 0,
            success_events: 0,
            failed_events: 0,
            unique_users: 0,
            unique_actions: 0,
        }
    }
}
