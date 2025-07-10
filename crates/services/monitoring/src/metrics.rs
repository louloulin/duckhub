//! 指标收集模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, debug, instrument};

/// 指标收集器
pub struct MetricsCollector {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
}

/// 指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    /// 指标名称
    pub name: String,
    /// 指标值
    pub value: f64,
    /// 指标标签
    pub labels: HashMap<String, String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 指标类型
    pub metric_type: MetricType,
}

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器
    Counter,
    /// 仪表盘
    Gauge,
    /// 直方图
    Histogram,
    /// 摘要
    Summary,
}

/// 系统指标
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    /// CPU使用率
    pub cpu_usage: f64,
    /// 内存使用率
    pub memory_usage: f64,
    /// 磁盘使用率
    pub disk_usage: f64,
    /// 网络IO
    pub network_io: NetworkIO,
    /// 磁盘IO
    pub disk_io: DiskIO,
    /// 进程数
    pub process_count: u32,
    /// 负载平均值
    pub load_average: LoadAverage,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 网络IO统计
#[derive(Debug, Clone, Serialize)]
pub struct NetworkIO {
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 发送包数
    pub packets_sent: u64,
}

/// 磁盘IO统计
#[derive(Debug, Clone, Serialize)]
pub struct DiskIO {
    /// 读取字节数
    pub bytes_read: u64,
    /// 写入字节数
    pub bytes_written: u64,
    /// 读取次数
    pub read_count: u64,
    /// 写入次数
    pub write_count: u64,
}

/// 负载平均值
#[derive(Debug, Clone, Serialize)]
pub struct LoadAverage {
    /// 1分钟负载
    pub one_minute: f64,
    /// 5分钟负载
    pub five_minutes: f64,
    /// 15分钟负载
    pub fifteen_minutes: f64,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>) -> Result<Self> {
        let collector = Self { engine };
        
        // 初始化数据库表
        collector.initialize_tables().await?;
        
        info!("指标收集器初始化完成");
        Ok(collector)
    }

    /// 初始化数据库表
    async fn initialize_tables(&self) -> Result<()> {
        // 创建指标表
        let create_metrics_sql = r#"
            CREATE TABLE IF NOT EXISTS monitoring_metrics (
                id VARCHAR PRIMARY KEY,
                name VARCHAR NOT NULL,
                value DOUBLE NOT NULL,
                labels JSON,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                metric_type VARCHAR NOT NULL
            )
        "#;
        self.engine.execute(create_metrics_sql).await?;

        // 创建系统指标表
        let create_system_metrics_sql = r#"
            CREATE TABLE IF NOT EXISTS monitoring_system_metrics (
                id VARCHAR PRIMARY KEY,
                cpu_usage DOUBLE NOT NULL,
                memory_usage DOUBLE NOT NULL,
                disk_usage DOUBLE NOT NULL,
                network_bytes_received BIGINT,
                network_bytes_sent BIGINT,
                network_packets_received BIGINT,
                network_packets_sent BIGINT,
                disk_bytes_read BIGINT,
                disk_bytes_written BIGINT,
                disk_read_count BIGINT,
                disk_write_count BIGINT,
                process_count INTEGER,
                load_one_minute DOUBLE,
                load_five_minutes DOUBLE,
                load_fifteen_minutes DOUBLE,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        self.engine.execute(create_system_metrics_sql).await?;

        // 创建索引
        let create_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_metrics_name ON monitoring_metrics(name)",
            "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON monitoring_metrics(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp ON monitoring_system_metrics(timestamp)",
        ];

        for index_sql in &create_indexes {
            self.engine.execute(index_sql).await?;
        }

        info!("指标数据库表初始化完成");
        Ok(())
    }

    /// 存储指标数据
    #[instrument(skip(self, metric))]
    pub async fn store_metric(&self, metric: MetricData) -> Result<()> {
        let insert_sql = format!(
            r#"INSERT INTO monitoring_metrics (id, name, value, labels, timestamp, metric_type)
               VALUES ('{}', '{}', {}, '{}', '{}', '{}')"#,
            uuid::Uuid::new_v4(),
            metric.name,
            metric.value,
            serde_json::to_string(&metric.labels).unwrap_or_default(),
            metric.timestamp.to_rfc3339(),
            format!("{:?}", metric.metric_type)
        );

        self.engine.execute(&insert_sql).await?;
        debug!("存储指标: {}", metric.name);
        Ok(())
    }

    /// 存储系统指标
    #[instrument(skip(self, metrics))]
    pub async fn store_metrics(&self, metrics: SystemMetrics) -> Result<()> {
        let insert_sql = format!(
            r#"INSERT INTO monitoring_system_metrics (
                id, cpu_usage, memory_usage, disk_usage,
                network_bytes_received, network_bytes_sent,
                network_packets_received, network_packets_sent,
                disk_bytes_read, disk_bytes_written,
                disk_read_count, disk_write_count,
                process_count, load_one_minute, load_five_minutes, load_fifteen_minutes,
                timestamp
            ) VALUES (
                '{}', {}, {}, {},
                {}, {}, {}, {},
                {}, {}, {}, {},
                {}, {}, {}, {},
                '{}'
            )"#,
            uuid::Uuid::new_v4(),
            metrics.cpu_usage,
            metrics.memory_usage,
            metrics.disk_usage,
            metrics.network_io.bytes_received,
            metrics.network_io.bytes_sent,
            metrics.network_io.packets_received,
            metrics.network_io.packets_sent,
            metrics.disk_io.bytes_read,
            metrics.disk_io.bytes_written,
            metrics.disk_io.read_count,
            metrics.disk_io.write_count,
            metrics.process_count,
            metrics.load_average.one_minute,
            metrics.load_average.five_minutes,
            metrics.load_average.fifteen_minutes,
            metrics.timestamp.to_rfc3339()
        );

        self.engine.execute(&insert_sql).await?;
        debug!("存储系统指标");
        Ok(())
    }

    /// 查询指标数据
    #[instrument(skip(self))]
    pub async fn query_metrics(
        &self,
        name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MetricData>> {
        let query_sql = format!(
            r#"SELECT name, value, labels, timestamp, metric_type
               FROM monitoring_metrics
               WHERE name = '{}' AND timestamp BETWEEN '{}' AND '{}'
               ORDER BY timestamp DESC"#,
            name,
            start_time.to_rfc3339(),
            end_time.to_rfc3339()
        );

        let result = self.engine.query(&query_sql).await?;
        let mut metrics = Vec::new();

        for row in &result {
            if let Ok(metric) = self.parse_metric_from_row(row) {
                metrics.push(metric);
            }
        }

        Ok(metrics)
    }

    /// 查询系统指标
    #[instrument(skip(self))]
    pub async fn query_system_metrics(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<SystemMetrics>> {
        let query_sql = format!(
            r#"SELECT * FROM monitoring_system_metrics
               WHERE timestamp BETWEEN '{}' AND '{}'
               ORDER BY timestamp DESC"#,
            start_time.to_rfc3339(),
            end_time.to_rfc3339()
        );

        let result = self.engine.query(&query_sql).await?;
        let mut metrics = Vec::new();

        for row in &result {
            if let Ok(metric) = self.parse_system_metrics_from_row(row) {
                metrics.push(metric);
            }
        }

        Ok(metrics)
    }

    /// 清理过期指标
    #[instrument(skip(self))]
    pub async fn cleanup_expired_metrics(&self, retention_days: u32) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let delete_metrics_sql = format!(
            "DELETE FROM monitoring_metrics WHERE timestamp < '{}'",
            cutoff_date.to_rfc3339()
        );
        let result1 = self.engine.execute(&delete_metrics_sql).await?;
        
        let delete_system_metrics_sql = format!(
            "DELETE FROM monitoring_system_metrics WHERE timestamp < '{}'",
            cutoff_date.to_rfc3339()
        );
        let result2 = self.engine.execute(&delete_system_metrics_sql).await?;
        
        info!("清理了过期的监控指标");
        Ok((result1 + result2) as u64)
    }

    /// 解析指标数据行
    fn parse_metric_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<MetricData> {
        Ok(MetricData {
            name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            value: row.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            labels: row.get("labels")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            timestamp: row.get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            metric_type: MetricType::Gauge, // 简化处理
        })
    }

    /// 解析系统指标数据行
    fn parse_system_metrics_from_row(&self, row: &HashMap<String, serde_json::Value>) -> Result<SystemMetrics> {
        Ok(SystemMetrics {
            cpu_usage: row.get("cpu_usage").and_then(|v| v.as_f64()).unwrap_or(0.0),
            memory_usage: row.get("memory_usage").and_then(|v| v.as_f64()).unwrap_or(0.0),
            disk_usage: row.get("disk_usage").and_then(|v| v.as_f64()).unwrap_or(0.0),
            network_io: NetworkIO {
                bytes_received: row.get("network_bytes_received").and_then(|v| v.as_u64()).unwrap_or(0),
                bytes_sent: row.get("network_bytes_sent").and_then(|v| v.as_u64()).unwrap_or(0),
                packets_received: row.get("network_packets_received").and_then(|v| v.as_u64()).unwrap_or(0),
                packets_sent: row.get("network_packets_sent").and_then(|v| v.as_u64()).unwrap_or(0),
            },
            disk_io: DiskIO {
                bytes_read: row.get("disk_bytes_read").and_then(|v| v.as_u64()).unwrap_or(0),
                bytes_written: row.get("disk_bytes_written").and_then(|v| v.as_u64()).unwrap_or(0),
                read_count: row.get("disk_read_count").and_then(|v| v.as_u64()).unwrap_or(0),
                write_count: row.get("disk_write_count").and_then(|v| v.as_u64()).unwrap_or(0),
            },
            process_count: row.get("process_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            load_average: LoadAverage {
                one_minute: row.get("load_one_minute").and_then(|v| v.as_f64()).unwrap_or(0.0),
                five_minutes: row.get("load_five_minutes").and_then(|v| v.as_f64()).unwrap_or(0.0),
                fifteen_minutes: row.get("load_fifteen_minutes").and_then(|v| v.as_f64()).unwrap_or(0.0),
            },
            timestamp: row.get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        })
    }
}
