//! DuckHub数据采集服务
//!
//! 提供实时和批量数据采集能力，支持多种数据源：
//! - Kafka消息队列
//! - WebSocket实时数据流
//! - REST API数据拉取
//! - 文件系统数据源
//! - 数据库连接器

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};
use prometheus::{Counter, Histogram, Gauge, Registry};

pub mod sources;
pub mod processors;
pub mod connectors;
pub mod scheduler;
pub mod config;

pub use sources::{DataRecord, DataSource, DataSourceStats, KafkaDataSource, WebSocketDataSource, RestApiDataSource};
pub use processors::{DataProcessor, ProcessingResult, ProcessingStats, DataCleaningProcessor, DataValidationProcessor};
pub use connectors::{DataConnector, WriteResult, ConnectorStats, MySQLConnector, PostgreSQLConnector, FileSystemConnector};
pub use scheduler::{TaskScheduler, TaskDefinition, ScheduledTask, TaskStatus, TaskStats, SchedulerConfig};
pub use config::{IngestionConfig, DataSourceConfig, ProcessorConfig, MonitoringConfig, ProcessorType};

/// 数据采集服务主结构
pub struct DataIngestionService {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 服务配置
    config: IngestionConfig,
    /// 数据源管理器
    sources: Arc<RwLock<HashMap<String, Box<dyn DataSource + Send + Sync>>>>,
    /// 数据处理器
    processors: Arc<RwLock<HashMap<String, Box<dyn DataProcessor + Send + Sync>>>>,
    /// 任务调度器
    scheduler: Arc<TaskScheduler>,
    /// 监控指标
    metrics: IngestionMetrics,
    /// 运行状态
    is_running: Arc<RwLock<bool>>,
}

/// 数据采集监控指标
#[derive(Debug, Clone)]
pub struct IngestionMetrics {
    /// 处理的记录总数
    pub records_processed: Counter,
    /// 处理失败的记录数
    pub records_failed: Counter,
    /// 数据处理延迟
    pub processing_duration: Histogram,
    /// 活跃的数据源数量
    pub active_sources: Gauge,
    /// 队列中的任务数量
    pub queued_tasks: Gauge,
}

impl IngestionMetrics {
    /// 创建新的监控指标实例
    pub fn new(registry: &Registry) -> Result<Self> {
        let records_processed = Counter::new(
            "duckhub_ingestion_records_processed_total",
            "处理的记录总数"
        )?;

        let records_failed = Counter::new(
            "duckhub_ingestion_records_failed_total",
            "处理失败的记录数"
        )?;

        let processing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_ingestion_processing_duration_seconds",
                "数据处理延迟分布"
            )
        )?;

        let active_sources = Gauge::new(
            "duckhub_ingestion_active_sources",
            "活跃的数据源数量"
        )?;

        let queued_tasks = Gauge::new(
            "duckhub_ingestion_queued_tasks",
            "队列中的任务数量"
        )?;

        // 注册指标
        registry.register(Box::new(records_processed.clone()))?;
        registry.register(Box::new(records_failed.clone()))?;
        registry.register(Box::new(processing_duration.clone()))?;
        registry.register(Box::new(active_sources.clone()))?;
        registry.register(Box::new(queued_tasks.clone()))?;

        Ok(Self {
            records_processed,
            records_failed,
            processing_duration,
            active_sources,
            queued_tasks,
        })
    }
}

impl DataIngestionService {
    /// 创建新的数据采集服务实例
    #[instrument(skip(engine, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        config: IngestionConfig,
        registry: &Registry,
    ) -> Result<Self> {
        // 验证配置
        config.validate().map_err(|e| DuckHubError::validation(e))?;

        // 创建监控指标
        let metrics = IngestionMetrics::new(registry)?;

        // 创建任务调度器
        let scheduler_config = SchedulerConfig::default();
        let scheduler = Arc::new(TaskScheduler::new(scheduler_config));

        info!("创建数据采集服务: {}", config.service_name);

        Ok(Self {
            engine,
            config,
            sources: Arc::new(RwLock::new(HashMap::new())),
            processors: Arc::new(RwLock::new(HashMap::new())),
            scheduler,
            metrics,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// 启动数据采集服务
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DuckHubError::validation("数据采集服务已在运行中"));
        }
        *is_running = true;
        drop(is_running);

        info!("启动数据采集服务");

        // 初始化数据源
        self.initialize_sources().await?;

        // 初始化处理器
        self.initialize_processors().await?;

        // 启动任务调度器
        self.scheduler.start().await?;

        // 启动数据处理管道
        self.start_processing_pipeline().await?;

        info!("数据采集服务启动完成");
        Ok(())
    }

    /// 停止数据采集服务
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        info!("停止数据采集服务");

        // 停止任务调度器
        self.scheduler.stop().await?;

        // 停止所有数据源
        let mut sources = self.sources.write().await;
        for (name, source) in sources.iter_mut() {
            if let Err(e) = source.stop().await {
                warn!("停止数据源 {} 时发生错误: {}", name, e);
            }
        }

        info!("数据采集服务已停止");
        Ok(())
    }

    /// 初始化数据源
    async fn initialize_sources(&self) -> Result<()> {
        let mut sources = self.sources.write().await;

        for (name, source_config) in &self.config.sources {
            if !source_config.enabled {
                continue;
            }

            let data_source: Box<dyn DataSource + Send + Sync> = match &source_config.source_type {
                DataSourceType::MessageQueue(_) => {
                    Box::new(KafkaDataSource::new(name.clone(), source_config.clone()))
                }
                DataSourceType::WebSocket(_) => {
                    Box::new(WebSocketDataSource::new(name.clone(), source_config.clone()))
                }
                DataSourceType::RestApi(_) => {
                    Box::new(RestApiDataSource::new(name.clone(), source_config.clone()))
                }
                _ => {
                    warn!("不支持的数据源类型: {:?}", source_config.source_type);
                    continue;
                }
            };

            sources.insert(name.clone(), data_source);
            info!("初始化数据源: {}", name);
        }

        // 更新活跃数据源指标
        self.metrics.active_sources.set(sources.len() as f64);
        Ok(())
    }

    /// 初始化处理器
    async fn initialize_processors(&self) -> Result<()> {
        let mut processors = self.processors.write().await;

        for (name, processor_config) in &self.config.processors {
            if !processor_config.enabled {
                continue;
            }

            let data_processor: Box<dyn DataProcessor + Send + Sync> = match processor_config.processor_type {
                ProcessorType::DataCleaning => {
                    Box::new(DataCleaningProcessor::new(name.clone(), processor_config.clone()))
                }
                ProcessorType::DataValidation => {
                    Box::new(DataValidationProcessor::new(name.clone(), processor_config.clone()))
                }
                _ => {
                    warn!("不支持的处理器类型: {:?}", processor_config.processor_type);
                    continue;
                }
            };

            processors.insert(name.clone(), data_processor);
            info!("初始化处理器: {}", name);
        }

        Ok(())
    }

    /// 启动数据处理管道
    async fn start_processing_pipeline(&self) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<DataRecord>(1000);

        // 启动所有数据源
        let mut sources = self.sources.write().await;
        for (name, source) in sources.iter_mut() {
            let sender = tx.clone();
            if let Err(e) = source.start(sender).await {
                error!("启动数据源 {} 失败: {}", name, e);
            } else {
                info!("数据源 {} 已启动", name);
            }
        }
        drop(sources);

        // 启动数据处理循环
        let service = Arc::new(self.clone());
        tokio::spawn(async move {
            while let Some(record) = rx.recv().await {
                if let Err(e) = service.process_record(record).await {
                    error!("处理数据记录时发生错误: {}", e);
                    service.metrics.records_failed.inc();
                } else {
                    service.metrics.records_processed.inc();
                }
            }
        });

        Ok(())
    }

    /// 处理单个数据记录
    async fn process_record(&self, mut record: DataRecord) -> Result<()> {
        let start_time = std::time::Instant::now();

        // 应用处理器链
        let processors = self.processors.read().await;
        for (name, processor) in processors.iter() {
            match processor.process(record.clone()).await {
                Ok(result) => {
                    if result.success {
                        if let Some(processed_data) = result.data {
                            record = processed_data;
                        }
                        debug!("处理器 {} 处理成功", name);
                    } else {
                        warn!("处理器 {} 处理失败: {:?}", name, result.error);
                        return Err(DuckHubError::validation(
                            result.error.unwrap_or_else(|| "处理失败".to_string())
                        ));
                    }
                }
                Err(e) => {
                    error!("处理器 {} 执行错误: {}", name, e);
                    return Err(e);
                }
            }
        }
        drop(processors);

        // 记录处理时间
        let duration = start_time.elapsed();
        self.metrics.processing_duration.observe(duration.as_secs_f64());

        // TODO: 将处理后的数据写入目标存储
        debug!("数据记录处理完成: {}", record.id);
        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self) -> ServiceStatus {
        let is_running = *self.is_running.read().await;
        let sources_count = self.sources.read().await.len();
        let processors_count = self.processors.read().await.len();
        let task_stats = self.scheduler.get_task_stats().await;

        ServiceStatus {
            is_running,
            sources_count: sources_count as u32,
            processors_count: processors_count as u32,
            active_tasks: task_stats.running_tasks as u32,
            total_records_processed: self.metrics.records_processed.get() as u64,
            total_records_failed: self.metrics.records_failed.get() as u64,
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut health_status = HealthStatus {
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

        // 检查数据源状态
        let sources = self.sources.read().await;
        for (name, source) in sources.iter() {
            match source.health_check().await {
                Ok(true) => {
                    health_status.components.insert(
                        format!("source_{}", name),
                        ComponentHealth {
                            status: "healthy".to_string(),
                            message: None,
                        }
                    );
                }
                Ok(false) | Err(_) => {
                    health_status.components.insert(
                        format!("source_{}", name),
                        ComponentHealth {
                            status: "unhealthy".to_string(),
                            message: Some("数据源连接异常".to_string()),
                        }
                    );
                    health_status.overall_status = "degraded".to_string();
                }
            }
        }

        Ok(health_status)
    }
}

// 为了支持clone，需要手动实现
impl Clone for DataIngestionService {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            config: self.config.clone(),
            sources: Arc::clone(&self.sources),
            processors: Arc::clone(&self.processors),
            scheduler: Arc::clone(&self.scheduler),
            metrics: self.metrics.clone(),
            is_running: Arc::clone(&self.is_running),
        }
    }
}

/// 服务状态
#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    /// 是否运行中
    pub is_running: bool,
    /// 数据源数量
    pub sources_count: u32,
    /// 处理器数量
    pub processors_count: u32,
    /// 活跃任务数
    pub active_tasks: u32,
    /// 处理的记录总数
    pub total_records_processed: u64,
    /// 失败的记录总数
    pub total_records_failed: u64,
}

/// 健康状态
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
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

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;
    use tempfile::tempdir;

    async fn create_test_engine() -> Arc<DuckDBEngine> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = duckhub_common::DatabaseConfig {
            url: format!("sqlite://{}", db_path.display()),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 300,
            max_lifetime: 1800,
            pool: duckhub_common::PoolConfig::default(),
        };

        Arc::new(DuckDBEngine::new(config).await.unwrap())
    }

    #[tokio::test]
    async fn test_data_ingestion_service_creation() {
        let engine = create_test_engine().await;
        let config = IngestionConfig::default();
        let registry = Registry::new();

        let service = DataIngestionService::new(engine, config, &registry).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_service_start_stop() {
        let engine = create_test_engine().await;
        let config = IngestionConfig::default();
        let registry = Registry::new();

        let service = DataIngestionService::new(engine, config, &registry).await.unwrap();

        // 测试启动
        assert!(service.start().await.is_ok());

        let status = service.get_status().await;
        assert!(status.is_running);

        // 测试停止
        assert!(service.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = create_test_engine().await;
        let config = IngestionConfig::default();
        let registry = Registry::new();

        let service = DataIngestionService::new(engine, config, &registry).await.unwrap();

        let health = service.health_check().await.unwrap();
        assert_eq!(health.overall_status, "healthy");
        assert!(health.components.contains_key("database"));
    }

    #[tokio::test]
    async fn test_metrics_creation() {
        let registry = Registry::new();
        let metrics = IngestionMetrics::new(&registry);
        assert!(metrics.is_ok());
    }
}
