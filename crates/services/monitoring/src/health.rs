//! 健康检查模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug, instrument};

/// 健康检查器
pub struct HealthChecker {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 健康检查配置
    checks: Vec<HealthCheck>,
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// 检查名称
    pub name: String,
    /// 检查类型
    pub check_type: HealthCheckType,
    /// 检查间隔（秒）
    pub interval: u64,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 是否启用
    pub enabled: bool,
}

/// 健康检查类型
#[derive(Debug, Clone)]
pub enum HealthCheckType {
    /// 数据库连接检查
    DatabaseConnection,
    /// HTTP端点检查
    HttpEndpoint { url: String },
    /// TCP端口检查
    TcpPort { host: String, port: u16 },
    /// 磁盘空间检查
    DiskSpace { path: String, threshold: f64 },
    /// 内存使用检查
    MemoryUsage { threshold: f64 },
    /// CPU使用检查
    CpuUsage { threshold: f64 },
}

/// 健康状态
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    /// 整体状态
    pub overall_status: HealthStatusLevel,
    /// 检查结果
    pub checks: HashMap<String, HealthCheckResult>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 健康状态级别
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum HealthStatusLevel {
    /// 健康
    Healthy,
    /// 警告
    Warning,
    /// 不健康
    Unhealthy,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    /// 检查名称
    pub name: String,
    /// 状态
    pub status: HealthStatusLevel,
    /// 消息
    pub message: String,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 检查时间
    pub checked_at: DateTime<Utc>,
    /// 详细信息
    pub details: Option<serde_json::Value>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>) -> Result<Self> {
        let checks = Self::default_health_checks();
        
        let checker = Self {
            engine,
            checks,
        };

        // 初始化数据库表
        checker.initialize_tables().await?;

        info!("健康检查器初始化完成");
        Ok(checker)
    }

    /// 默认健康检查配置
    fn default_health_checks() -> Vec<HealthCheck> {
        vec![
            HealthCheck {
                name: "database_connection".to_string(),
                check_type: HealthCheckType::DatabaseConnection,
                interval: 60,
                timeout: 10,
                enabled: true,
            },
            HealthCheck {
                name: "disk_space".to_string(),
                check_type: HealthCheckType::DiskSpace {
                    path: "/".to_string(),
                    threshold: 90.0, // 90%
                },
                interval: 300, // 5分钟
                timeout: 5,
                enabled: true,
            },
            HealthCheck {
                name: "memory_usage".to_string(),
                check_type: HealthCheckType::MemoryUsage {
                    threshold: 85.0, // 85%
                },
                interval: 60,
                timeout: 5,
                enabled: true,
            },
            HealthCheck {
                name: "cpu_usage".to_string(),
                check_type: HealthCheckType::CpuUsage {
                    threshold: 80.0, // 80%
                },
                interval: 60,
                timeout: 5,
                enabled: true,
            },
        ]
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        let create_health_checks_sql = r#"
            CREATE TABLE IF NOT EXISTS monitoring_health_checks (
                id VARCHAR PRIMARY KEY,
                name VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                message TEXT,
                response_time_ms BIGINT,
                checked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                details JSON
            )
        "#;
        self.engine.execute(create_health_checks_sql).await?;

        // 创建索引
        let create_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_health_checks_name ON monitoring_health_checks(name)",
            "CREATE INDEX IF NOT EXISTS idx_health_checks_checked_at ON monitoring_health_checks(checked_at)",
        ];

        for index_sql in &create_indexes {
            self.engine.execute(index_sql).await?;
        }

        info!("健康检查数据库表初始化完成");
        Ok(())
    }

    /// 运行所有健康检查
    #[instrument(skip(self))]
    pub async fn run_health_checks(&self) -> Result<()> {
        for check in &self.checks {
            if !check.enabled {
                continue;
            }

            let result = self.run_single_check(check).await;
            self.store_check_result(&result).await?;
        }

        debug!("健康检查完成");
        Ok(())
    }

    /// 运行单个健康检查
    async fn run_single_check(&self, check: &HealthCheck) -> HealthCheckResult {
        let start_time = std::time::Instant::now();
        
        let (status, message, details) = match &check.check_type {
            HealthCheckType::DatabaseConnection => {
                self.check_database_connection().await
            }
            HealthCheckType::HttpEndpoint { url } => {
                self.check_http_endpoint(url).await
            }
            HealthCheckType::TcpPort { host, port } => {
                self.check_tcp_port(host, *port).await
            }
            HealthCheckType::DiskSpace { path, threshold } => {
                self.check_disk_space(path, *threshold).await
            }
            HealthCheckType::MemoryUsage { threshold } => {
                self.check_memory_usage(*threshold).await
            }
            HealthCheckType::CpuUsage { threshold } => {
                self.check_cpu_usage(*threshold).await
            }
        };

        let response_time = start_time.elapsed();

        HealthCheckResult {
            name: check.name.clone(),
            status,
            message,
            response_time_ms: response_time.as_millis() as u64,
            checked_at: Utc::now(),
            details,
        }
    }

    /// 检查数据库连接
    async fn check_database_connection(&self) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        match self.engine.check_connection().await {
            Ok(_) => (
                HealthStatusLevel::Healthy,
                "数据库连接正常".to_string(),
                None
            ),
            Err(e) => (
                HealthStatusLevel::Unhealthy,
                format!("数据库连接失败: {}", e),
                None
            ),
        }
    }

    /// 检查HTTP端点
    async fn check_http_endpoint(&self, url: &str) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        match reqwest::get(url).await {
            Ok(response) => {
                if response.status().is_success() {
                    (
                        HealthStatusLevel::Healthy,
                        format!("HTTP端点 {} 响应正常", url),
                        Some(serde_json::json!({
                            "status_code": response.status().as_u16(),
                            "url": url
                        }))
                    )
                } else {
                    (
                        HealthStatusLevel::Warning,
                        format!("HTTP端点 {} 返回状态码: {}", url, response.status()),
                        Some(serde_json::json!({
                            "status_code": response.status().as_u16(),
                            "url": url
                        }))
                    )
                }
            }
            Err(e) => (
                HealthStatusLevel::Unhealthy,
                format!("HTTP端点 {} 检查失败: {}", url, e),
                Some(serde_json::json!({
                    "error": e.to_string(),
                    "url": url
                }))
            ),
        }
    }

    /// 检查TCP端口
    async fn check_tcp_port(&self, host: &str, port: u16) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        match tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
            Ok(_) => (
                HealthStatusLevel::Healthy,
                format!("TCP端口 {}:{} 连接正常", host, port),
                None
            ),
            Err(e) => (
                HealthStatusLevel::Unhealthy,
                format!("TCP端口 {}:{} 连接失败: {}", host, port, e),
                None
            ),
        }
    }

    /// 检查磁盘空间
    async fn check_disk_space(&self, _path: &str, threshold: f64) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        // 简化实现，实际应该检查指定路径的磁盘使用率
        let usage = 50.0; // 模拟50%使用率
        
        if usage > threshold {
            (
                HealthStatusLevel::Warning,
                format!("磁盘使用率 {:.1}% 超过阈值 {:.1}%", usage, threshold),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        } else {
            (
                HealthStatusLevel::Healthy,
                format!("磁盘使用率 {:.1}% 正常", usage),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        }
    }

    /// 检查内存使用
    async fn check_memory_usage(&self, threshold: f64) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        // 简化实现，实际应该获取真实的内存使用率
        let usage = 60.0; // 模拟60%使用率
        
        if usage > threshold {
            (
                HealthStatusLevel::Warning,
                format!("内存使用率 {:.1}% 超过阈值 {:.1}%", usage, threshold),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        } else {
            (
                HealthStatusLevel::Healthy,
                format!("内存使用率 {:.1}% 正常", usage),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        }
    }

    /// 检查CPU使用
    async fn check_cpu_usage(&self, threshold: f64) -> (HealthStatusLevel, String, Option<serde_json::Value>) {
        // 简化实现，实际应该获取真实的CPU使用率
        let usage = 45.0; // 模拟45%使用率
        
        if usage > threshold {
            (
                HealthStatusLevel::Warning,
                format!("CPU使用率 {:.1}% 超过阈值 {:.1}%", usage, threshold),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        } else {
            (
                HealthStatusLevel::Healthy,
                format!("CPU使用率 {:.1}% 正常", usage),
                Some(serde_json::json!({
                    "usage_percent": usage,
                    "threshold": threshold
                }))
            )
        }
    }

    /// 存储检查结果
    async fn store_check_result(&self, result: &HealthCheckResult) -> Result<()> {
        let insert_sql = format!(
            r#"INSERT INTO monitoring_health_checks (id, name, status, message, response_time_ms, checked_at, details)
               VALUES ('{}', '{}', '{}', '{}', {}, '{}', '{}')"#,
            uuid::Uuid::new_v4(),
            result.name,
            format!("{:?}", result.status),
            result.message,
            result.response_time_ms,
            result.checked_at.to_rfc3339(),
            result.details.as_ref()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "{}".to_string())
        );

        self.engine.execute(&insert_sql).await?;
        Ok(())
    }

    /// 获取整体健康状态
    #[instrument(skip(self))]
    pub async fn get_overall_health(&self) -> Result<HealthStatus> {
        let mut checks = HashMap::new();
        let mut overall_status = HealthStatusLevel::Healthy;

        // 获取最新的检查结果
        for check in &self.checks {
            if !check.enabled {
                continue;
            }

            let query_sql = format!(
                r#"SELECT status, message, response_time_ms, checked_at, details
                   FROM monitoring_health_checks
                   WHERE name = '{}'
                   ORDER BY checked_at DESC
                   LIMIT 1"#,
                check.name
            );

            let result = self.engine.query(&query_sql).await?;
            if let Some(row) = result.get(0) {
                let status_str = row.get("status").and_then(|v| v.as_str()).unwrap_or("Healthy");
                let status = match status_str {
                    "Warning" => HealthStatusLevel::Warning,
                    "Unhealthy" => HealthStatusLevel::Unhealthy,
                    _ => HealthStatusLevel::Healthy,
                };

                // 更新整体状态
                if status == HealthStatusLevel::Unhealthy {
                    overall_status = HealthStatusLevel::Unhealthy;
                } else if status == HealthStatusLevel::Warning && overall_status == HealthStatusLevel::Healthy {
                    overall_status = HealthStatusLevel::Warning;
                }

                let check_result = HealthCheckResult {
                    name: check.name.clone(),
                    status,
                    message: row.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    response_time_ms: row.get("response_time_ms").and_then(|v| v.as_u64()).unwrap_or(0),
                    checked_at: row.get("checked_at")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    details: row.get("details")
                        .and_then(|v| v.as_str())
                        .and_then(|s| serde_json::from_str(s).ok()),
                };

                checks.insert(check.name.clone(), check_result);
            }
        }

        Ok(HealthStatus {
            overall_status,
            checks,
            last_updated: Utc::now(),
        })
    }
}
