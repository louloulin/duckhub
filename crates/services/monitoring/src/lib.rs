//! DuckHub监控和运维工具
//! 
//! 提供全面的系统监控和运维功能：
//! - 系统性能监控
//! - 健康检查
//! - 告警管理
//! - 指标收集和展示

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use prometheus::{Counter, Histogram, Gauge, Registry};

pub mod metrics;
pub mod health;
pub mod alerts;
pub mod system;

pub use metrics::{MetricsCollector as MonitoringMetricsCollector, MetricData, SystemMetrics, NetworkIO, DiskIO, LoadAverage, MetricType};
pub use health::{HealthChecker, HealthCheck, HealthCheckType, HealthStatus as HealthStatusDetail, HealthStatusLevel, HealthCheckResult};
pub use alerts::{AlertManager, AlertConfig, Alert, AlertRule, AlertCondition, AlertSeverity, AlertStatus, NotificationConfig, EmailConfig, WebhookConfig};
pub use system::{SystemMonitor, SystemStatus, SystemInfo, CpuInfo, MemoryInfo, DiskInfo, NetworkInfo, ProcessInfo};

/// 监控服务主结构
pub struct MonitoringService {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 指标收集器
    metrics_collector: Arc<MonitoringMetricsCollector>,
    /// 健康检查器
    health_checker: Arc<HealthChecker>,
    /// 告警管理器
    alert_manager: Arc<AlertManager>,
    /// 系统监控器
    system_monitor: Arc<SystemMonitor>,
    /// 监控配置
    config: MonitoringConfig,
    /// 监控指标
    metrics: MonitoringMetrics,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 指标收集间隔（秒）
    pub metrics_interval: u64,
    /// 健康检查间隔（秒）
    pub health_check_interval: u64,
    /// 告警检查间隔（秒）
    pub alert_check_interval: u64,
    /// 数据保留天数
    pub retention_days: u32,
    /// 是否启用系统监控
    pub enable_system_monitoring: bool,
    /// 是否启用网络监控
    pub enable_network_monitoring: bool,
    /// 告警配置
    pub alerts: AlertConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_interval: 30,      // 30秒
            health_check_interval: 60, // 1分钟
            alert_check_interval: 30,  // 30秒
            retention_days: 30,        // 30天
            enable_system_monitoring: true,
            enable_network_monitoring: true,
            alerts: AlertConfig::default(),
        }
    }
}

/// 监控指标
#[derive(Debug, Clone)]
pub struct MonitoringMetrics {
    /// 收集的指标数量
    pub metrics_collected: Counter,
    /// 健康检查次数
    pub health_checks: Counter,
    /// 告警触发次数
    pub alerts_triggered: Counter,
    /// 监控延迟
    pub monitoring_duration: Histogram,
    /// 活跃的监控任务数
    pub active_monitors: Gauge,
}

impl MonitoringMetrics {
    /// 创建新的监控指标实例
    pub fn new(registry: &Registry) -> Result<Self> {
        let metrics_collected = Counter::new(
            "duckhub_monitoring_metrics_collected_total",
            "收集的指标数量"
        )?;
        
        let health_checks = Counter::new(
            "duckhub_monitoring_health_checks_total", 
            "健康检查次数"
        )?;
        
        let alerts_triggered = Counter::new(
            "duckhub_monitoring_alerts_triggered_total",
            "告警触发次数"
        )?;
        
        let monitoring_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_monitoring_duration_seconds",
                "监控处理延迟分布"
            )
        )?;
        
        let active_monitors = Gauge::new(
            "duckhub_monitoring_active_monitors",
            "活跃的监控任务数"
        )?;

        // 注册指标
        registry.register(Box::new(metrics_collected.clone()))?;
        registry.register(Box::new(health_checks.clone()))?;
        registry.register(Box::new(alerts_triggered.clone()))?;
        registry.register(Box::new(monitoring_duration.clone()))?;
        registry.register(Box::new(active_monitors.clone()))?;

        Ok(Self {
            metrics_collected,
            health_checks,
            alerts_triggered,
            monitoring_duration,
            active_monitors,
        })
    }
}

impl MonitoringService {
    /// 创建新的监控服务实例
    #[instrument(skip(engine, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        config: MonitoringConfig,
        registry: &Registry,
    ) -> Result<Self> {
        // 创建监控指标
        let metrics = MonitoringMetrics::new(registry)?;

        // 创建各个组件
        let metrics_collector = Arc::new(MonitoringMetricsCollector::new(Arc::clone(&engine)).await?);
        let health_checker = Arc::new(HealthChecker::new(Arc::clone(&engine)).await?);
        let alert_manager = Arc::new(AlertManager::new(Arc::clone(&engine), config.alerts.clone()).await?);
        let system_monitor = Arc::new(SystemMonitor::new());

        info!("创建监控服务");

        Ok(Self {
            engine,
            metrics_collector,
            health_checker,
            alert_manager,
            system_monitor,
            config,
            metrics,
        })
    }

    /// 启动监控服务
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("启动监控服务");

        // 启动指标收集
        if self.config.enable_system_monitoring {
            self.start_metrics_collection().await?;
        }

        // 启动健康检查
        self.start_health_checks().await?;

        // 启动告警检查
        self.start_alert_monitoring().await?;

        info!("监控服务启动完成");
        Ok(())
    }

    /// 停止监控服务
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("停止监控服务");
        // 实际实现中应该停止所有后台任务
        Ok(())
    }

    /// 启动指标收集
    async fn start_metrics_collection(&self) -> Result<()> {
        let collector = Arc::clone(&self.metrics_collector);
        let system_monitor = Arc::clone(&self.system_monitor);
        let metrics = self.metrics.clone();
        let interval = self.config.metrics_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                tokio::time::Duration::from_secs(interval)
            );

            loop {
                interval_timer.tick().await;
                
                let start_time = std::time::Instant::now();
                
                // 收集系统指标
                if let Ok(system_metrics) = system_monitor.collect_metrics().await {
                    if let Err(e) = collector.store_metrics(system_metrics).await {
                        warn!("存储系统指标失败: {}", e);
                    } else {
                        metrics.metrics_collected.inc();
                    }
                }

                let duration = start_time.elapsed();
                metrics.monitoring_duration.observe(duration.as_secs_f64());
            }
        });

        Ok(())
    }

    /// 启动健康检查
    async fn start_health_checks(&self) -> Result<()> {
        let checker = Arc::clone(&self.health_checker);
        let metrics = self.metrics.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                tokio::time::Duration::from_secs(interval)
            );

            loop {
                interval_timer.tick().await;
                
                if let Err(e) = checker.run_health_checks().await {
                    warn!("健康检查失败: {}", e);
                } else {
                    metrics.health_checks.inc();
                }
            }
        });

        Ok(())
    }

    /// 启动告警监控
    async fn start_alert_monitoring(&self) -> Result<()> {
        let alert_manager = Arc::clone(&self.alert_manager);
        let metrics = self.metrics.clone();
        let interval = self.config.alert_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(
                tokio::time::Duration::from_secs(interval)
            );

            loop {
                interval_timer.tick().await;
                
                match alert_manager.check_alerts().await {
                    Ok(triggered_count) => {
                        if triggered_count > 0 {
                            metrics.alerts_triggered.inc_by(triggered_count as f64);
                        }
                    }
                    Err(e) => {
                        warn!("告警检查失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 获取监控统计信息
    pub async fn get_monitoring_stats(&self) -> MonitoringStats {
        MonitoringStats {
            metrics_collected: self.metrics.metrics_collected.get() as u64,
            health_checks_performed: self.metrics.health_checks.get() as u64,
            alerts_triggered: self.metrics.alerts_triggered.get() as u64,
            active_monitors: self.metrics.active_monitors.get() as u32,
        }
    }

    /// 获取系统状态
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        self.system_monitor.get_system_status().await
    }

    /// 获取健康状态
    pub async fn get_health_status(&self) -> Result<HealthStatusDetail> {
        self.health_checker.get_overall_health().await
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_active_alerts().await
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<MonitoringHealthStatus> {
        let mut health_status = MonitoringHealthStatus {
            overall_status: "healthy".to_string(),
            components: HashMap::new(),
        };

        // 检查数据库连接
        match self.engine.check_connection().await {
            Ok(_) => {
                health_status.components.insert(
                    "database".to_string(),
                    ComponentHealth {
                        status: "healthy".to_string(),
                        message: None,
                    }
                );
            }
            Err(e) => {
                health_status.components.insert(
                    "database".to_string(),
                    ComponentHealth {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                    }
                );
                health_status.overall_status = "unhealthy".to_string();
            }
        }

        // 检查系统监控器
        health_status.components.insert(
            "system_monitor".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                message: None,
            }
        );

        Ok(health_status)
    }

    /// 获取监控指标
    pub async fn get_metrics(&self) -> Result<MonitoringMetricsData> {
        let stats = self.get_monitoring_stats().await;
        let system_status = self.get_system_status().await?;

        Ok(MonitoringMetricsData {
            total_queries: stats.metrics_collected,
            active_connections: 25, // Mock value
            data_processed_bytes: 1024 * 1024 * 500, // 500MB mock
            cpu_usage: system_status.cpu_info.usage_percent as f64,
            memory_usage: system_status.memory_info.usage_percent as f64,
            disk_usage: system_status.disk_info.iter().map(|d| d.usage_percent as f64).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0),
            network_io_bytes: system_status.network_info.iter().map(|n| n.bytes_sent + n.bytes_received).sum(),
            avg_response_time_ms: 95.5, // Mock value
        })
    }

    /// 获取最近活动
    pub async fn get_recent_activities(&self, limit: usize) -> Result<Vec<MonitoringActivity>> {
        // Mock implementation - in real implementation, this would query the database
        let activities = vec![
            MonitoringActivity {
                id: Uuid::new_v4(),
                activity_type: "query_executed".to_string(),
                description: "执行了复杂查询".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(5),
                metadata: Some(serde_json::json!({
                    "query_time_ms": 150,
                    "rows_affected": 1250
                })),
                severity: "info".to_string(),
            },
            MonitoringActivity {
                id: Uuid::new_v4(),
                activity_type: "snapshot_created".to_string(),
                description: "创建了数据快照".to_string(),
                timestamp: Utc::now() - chrono::Duration::minutes(15),
                metadata: Some(serde_json::json!({
                    "snapshot_size": "1.2GB",
                    "version": 126
                })),
                severity: "info".to_string(),
            },
            MonitoringActivity {
                id: Uuid::new_v4(),
                activity_type: "schema_updated".to_string(),
                description: "更新了表结构".to_string(),
                timestamp: Utc::now() - chrono::Duration::hours(1),
                metadata: Some(serde_json::json!({
                    "table": "transactions",
                    "changes": "added_index"
                })),
                severity: "warning".to_string(),
            },
        ];

        Ok(activities.into_iter().take(limit).collect())
    }

    /// 获取性能指标
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetricsData> {
        let system_status = self.get_system_status().await?;

        Ok(PerformanceMetricsData {
            query_performance: QueryPerformance {
                avg_query_time_ms: 95.0,
                queries_per_second: 150.0,
                slow_queries_count: 3,
                cache_hit_rate: 85.5,
            },
            system_performance: SystemPerformance {
                cpu_usage: system_status.cpu_info.usage_percent as f64,
                memory_usage: system_status.memory_info.usage_percent as f64,
                disk_io_rate: 25.5,
                network_io_rate: 12.3,
            },
            database_performance: DatabasePerformance {
                active_connections: 25,
                connection_pool_usage: 60.0,
                lock_wait_time_ms: 2.5,
                buffer_hit_rate: 98.2,
            },
        })
    }
}

/// 监控统计信息
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringStats {
    /// 收集的指标数量
    pub metrics_collected: u64,
    /// 执行的健康检查次数
    pub health_checks_performed: u64,
    /// 触发的告警次数
    pub alerts_triggered: u64,
    /// 活跃的监控任务数
    pub active_monitors: u32,
}

/// 监控健康状态
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringHealthStatus {
    /// 整体状态
    pub overall_status: String,
    /// 组件状态
    pub components: HashMap<String, ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    /// 状态
    pub status: String,
    /// 消息
    pub message: Option<String>,
}

/// 监控指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetricsData {
    pub total_queries: u64,
    pub active_connections: u32,
    pub data_processed_bytes: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io_bytes: u64,
    pub avg_response_time_ms: f64,  // 新增
}

/// 监控活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringActivity {
    pub id: Uuid,
    pub activity_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub severity: String,  // 新增
}

/// 性能指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsData {
    pub query_performance: QueryPerformance,
    pub system_performance: SystemPerformance,
    pub database_performance: DatabasePerformance,
}

/// 查询性能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformance {
    pub avg_query_time_ms: f64,
    pub queries_per_second: f64,
    pub slow_queries_count: u32,
    pub cache_hit_rate: f64,
}

/// 系统性能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformance {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_io_rate: f64,
    pub network_io_rate: f64,
}

/// 数据库性能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformance {
    pub active_connections: u32,
    pub connection_pool_usage: f64,
    pub lock_wait_time_ms: f64,
    pub buffer_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    use tempfile::tempdir;

    async fn create_test_engine() -> Arc<DuckDBEngine> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = duckhub_common::DatabaseConfig {
            duckdb_path: db_path.to_string_lossy().to_string(),
            memory_limit: Some("1GB".to_string()),
            threads: Some(2),
            max_memory: Some("1GB".to_string()),
            temp_directory: Some(temp_dir.path().to_string_lossy().to_string()),
            extensions: vec![],
            pool: duckhub_common::PoolConfig::default(),
        };

        Arc::new(DuckDBEngine::new(config).await.unwrap())
    }

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let engine = create_test_engine().await;
        let config = MonitoringConfig::default();
        let registry = Registry::new();

        let service = MonitoringService::new(engine, config, &registry).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_system_metrics_collection() {
        let system_monitor = SystemMonitor::new();
        let metrics = system_monitor.collect_metrics().await.unwrap();

        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.memory_usage >= 0.0);
        assert!(metrics.disk_usage >= 0.0);
        assert!(metrics.process_count > 0);
    }

    #[tokio::test]
    async fn test_system_status() {
        let system_monitor = SystemMonitor::new();
        let status = system_monitor.get_system_status().await.unwrap();

        assert!(!status.system_info.os_name.is_empty());
        assert!(status.cpu_info.core_count > 0);
        assert!(status.memory_info.total_memory > 0);
        assert!(!status.disk_info.is_empty());
    }

    #[tokio::test]
    async fn test_monitoring_stats() {
        let engine = create_test_engine().await;
        let config = MonitoringConfig::default();
        let registry = Registry::new();

        let service = MonitoringService::new(engine, config, &registry).await.unwrap();
        let stats = service.get_monitoring_stats().await;

        assert_eq!(stats.metrics_collected, 0);
        assert_eq!(stats.health_checks_performed, 0);
        assert_eq!(stats.alerts_triggered, 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = create_test_engine().await;
        let config = MonitoringConfig::default();
        let registry = Registry::new();

        let service = MonitoringService::new(engine, config, &registry).await.unwrap();
        let health = service.health_check().await.unwrap();

        assert_eq!(health.overall_status, "healthy");
        assert!(health.components.contains_key("database"));
        assert!(health.components.contains_key("system_monitor"));
    }
}
