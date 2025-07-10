//! 告警管理模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};

/// 告警管理器
pub struct AlertManager {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 告警配置
    config: AlertConfig,
    /// 告警规则
    rules: Vec<AlertRule>,
}

/// 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// 是否启用告警
    pub enabled: bool,
    /// 告警检查间隔（秒）
    pub check_interval: u64,
    /// 告警保留天数
    pub retention_days: u32,
    /// 通知配置
    pub notifications: NotificationConfig,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 60,
            retention_days: 30,
            notifications: NotificationConfig::default(),
        }
    }
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 邮件通知
    pub email: Option<EmailConfig>,
    /// Webhook通知
    pub webhook: Option<WebhookConfig>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            email: None,
            webhook: None,
        }
    }
}

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP服务器
    pub smtp_server: String,
    /// SMTP端口
    pub smtp_port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 发件人
    pub from: String,
    /// 收件人列表
    pub to: Vec<String>,
}

/// Webhook配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook URL
    pub url: String,
    /// 请求头
    pub headers: HashMap<String, String>,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: String,
    /// 指标名称
    pub metric_name: String,
    /// 条件
    pub condition: AlertCondition,
    /// 阈值
    pub threshold: f64,
    /// 持续时间（秒）
    pub duration: u64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
}

/// 告警条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
}

/// 告警严重级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 告警
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    /// 告警ID
    pub id: String,
    /// 规则ID
    pub rule_id: String,
    /// 告警名称
    pub name: String,
    /// 告警描述
    pub description: String,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 告警状态
    pub status: AlertStatus,
    /// 触发时间
    pub triggered_at: DateTime<Utc>,
    /// 解决时间
    pub resolved_at: Option<DateTime<Utc>>,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 详细信息
    pub details: HashMap<String, String>,
}

/// 告警状态
#[derive(Debug, Clone, Serialize)]
pub enum AlertStatus {
    /// 触发
    Triggered,
    /// 已解决
    Resolved,
    /// 已确认
    Acknowledged,
}

impl AlertManager {
    /// 创建新的告警管理器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>, config: AlertConfig) -> Result<Self> {
        let rules = Self::default_alert_rules();
        
        let manager = Self {
            engine,
            config,
            rules,
        };

        // 初始化数据库表
        manager.initialize_tables().await?;

        info!("告警管理器初始化完成");
        Ok(manager)
    }

    /// 默认告警规则
    fn default_alert_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                id: Uuid::new_v4().to_string(),
                name: "高CPU使用率".to_string(),
                description: "CPU使用率超过80%".to_string(),
                metric_name: "cpu_usage".to_string(),
                condition: AlertCondition::GreaterThan,
                threshold: 80.0,
                duration: 300, // 5分钟
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                id: Uuid::new_v4().to_string(),
                name: "高内存使用率".to_string(),
                description: "内存使用率超过85%".to_string(),
                metric_name: "memory_usage".to_string(),
                condition: AlertCondition::GreaterThan,
                threshold: 85.0,
                duration: 300,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                id: Uuid::new_v4().to_string(),
                name: "磁盘空间不足".to_string(),
                description: "磁盘使用率超过90%".to_string(),
                metric_name: "disk_usage".to_string(),
                condition: AlertCondition::GreaterThan,
                threshold: 90.0,
                duration: 600, // 10分钟
                severity: AlertSeverity::Error,
                enabled: true,
            },
        ]
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        // 创建告警表
        let create_alerts_sql = r#"
            CREATE TABLE IF NOT EXISTS monitoring_alerts (
                id VARCHAR PRIMARY KEY,
                rule_id VARCHAR NOT NULL,
                name VARCHAR NOT NULL,
                description TEXT,
                severity VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                triggered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                resolved_at TIMESTAMP,
                current_value DOUBLE NOT NULL,
                threshold DOUBLE NOT NULL,
                details JSON
            )
        "#;
        self.engine.execute(create_alerts_sql).await?;

        // 创建告警规则表
        let create_rules_sql = r#"
            CREATE TABLE IF NOT EXISTS monitoring_alert_rules (
                id VARCHAR PRIMARY KEY,
                name VARCHAR NOT NULL,
                description TEXT,
                metric_name VARCHAR NOT NULL,
                condition VARCHAR NOT NULL,
                threshold DOUBLE NOT NULL,
                duration BIGINT NOT NULL,
                severity VARCHAR NOT NULL,
                enabled BOOLEAN DEFAULT true
            )
        "#;
        self.engine.execute(create_rules_sql).await?;

        // 创建索引
        let create_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_alerts_rule_id ON monitoring_alerts(rule_id)",
            "CREATE INDEX IF NOT EXISTS idx_alerts_status ON monitoring_alerts(status)",
            "CREATE INDEX IF NOT EXISTS idx_alerts_triggered_at ON monitoring_alerts(triggered_at)",
        ];

        for index_sql in &create_indexes {
            self.engine.execute(index_sql).await?;
        }

        info!("告警数据库表初始化完成");
        Ok(())
    }

    /// 检查告警
    #[instrument(skip(self))]
    pub async fn check_alerts(&self) -> Result<u32> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut triggered_count = 0;

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            if let Ok(should_trigger) = self.evaluate_rule(rule).await {
                if should_trigger {
                    self.trigger_alert(rule).await?;
                    triggered_count += 1;
                }
            }
        }

        debug!("告警检查完成，触发 {} 个告警", triggered_count);
        Ok(triggered_count)
    }

    /// 评估告警规则
    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<bool> {
        // 简化实现：模拟指标值
        let current_value = match rule.metric_name.as_str() {
            "cpu_usage" => 75.0,    // 模拟75% CPU使用率
            "memory_usage" => 82.0, // 模拟82% 内存使用率
            "disk_usage" => 65.0,   // 模拟65% 磁盘使用率
            _ => 0.0,
        };

        let should_trigger = match rule.condition {
            AlertCondition::GreaterThan => current_value > rule.threshold,
            AlertCondition::LessThan => current_value < rule.threshold,
            AlertCondition::Equal => (current_value - rule.threshold).abs() < 0.01,
            AlertCondition::NotEqual => (current_value - rule.threshold).abs() >= 0.01,
        };

        debug!(
            "规则 {} 评估: 当前值={}, 阈值={}, 结果={}",
            rule.name, current_value, rule.threshold, should_trigger
        );

        Ok(should_trigger)
    }

    /// 触发告警
    async fn trigger_alert(&self, rule: &AlertRule) -> Result<()> {
        // 检查是否已有活跃告警
        let existing_alert_sql = format!(
            "SELECT id FROM monitoring_alerts WHERE rule_id = '{}' AND status = 'Triggered'",
            rule.id
        );
        
        let result = self.engine.query(&existing_alert_sql).await?;
        if !result.is_empty() {
            // 已有活跃告警，不重复触发
            return Ok(());
        }

        let alert = Alert {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            name: rule.name.clone(),
            description: rule.description.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Triggered,
            triggered_at: Utc::now(),
            resolved_at: None,
            current_value: 75.0, // 模拟值
            threshold: rule.threshold,
            details: HashMap::new(),
        };

        // 存储告警
        self.store_alert(&alert).await?;

        // 发送通知
        self.send_notification(&alert).await?;

        info!("触发告警: {}", alert.name);
        Ok(())
    }

    /// 存储告警
    async fn store_alert(&self, alert: &Alert) -> Result<()> {
        let insert_sql = format!(
            r#"INSERT INTO monitoring_alerts (id, rule_id, name, description, severity, status, triggered_at, current_value, threshold, details)
               VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, {}, '{}')"#,
            alert.id,
            alert.rule_id,
            alert.name,
            alert.description,
            format!("{:?}", alert.severity),
            format!("{:?}", alert.status),
            alert.triggered_at.to_rfc3339(),
            alert.current_value,
            alert.threshold,
            serde_json::to_string(&alert.details).unwrap_or_default()
        );

        self.engine.execute(&insert_sql).await?;
        Ok(())
    }

    /// 发送通知
    async fn send_notification(&self, alert: &Alert) -> Result<()> {
        // 简化实现：仅记录日志
        info!(
            "发送告警通知: {} - {} (严重级别: {:?})",
            alert.name, alert.description, alert.severity
        );

        // 实际实现中应该根据配置发送邮件或Webhook通知
        if let Some(_email_config) = &self.config.notifications.email {
            debug!("发送邮件通知");
        }

        if let Some(_webhook_config) = &self.config.notifications.webhook {
            debug!("发送Webhook通知");
        }

        Ok(())
    }

    /// 获取活跃告警
    #[instrument(skip(self))]
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let query_sql = r#"
            SELECT id, rule_id, name, description, severity, status, triggered_at, resolved_at, current_value, threshold, details
            FROM monitoring_alerts
            WHERE status = 'Triggered'
            ORDER BY triggered_at DESC
        "#;

        let result = self.engine.query(query_sql).await?;
        let mut alerts = Vec::new();

        for row in &result {
            if let Ok(alert) = self.parse_alert_from_row(row) {
                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    /// 解析告警数据行
    fn parse_alert_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<Alert> {
        Ok(Alert {
            id: row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            rule_id: row.get("rule_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: row.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            severity: AlertSeverity::Warning, // 简化处理
            status: AlertStatus::Triggered,   // 简化处理
            triggered_at: row.get("triggered_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            resolved_at: None,
            current_value: row.get("current_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            threshold: row.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.0),
            details: row.get("details")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
        })
    }
}
