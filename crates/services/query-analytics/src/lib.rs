//! 查询分析服务 - DuckHub金融数据平台

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use prometheus::{Counter, Histogram, Gauge, Registry};

pub mod optimizer;
pub mod analyzer;
pub mod window_functions;
pub mod time_series;

pub use optimizer::{QueryOptimizer, TableStats, ColumnStats, ExecutionPlan, OptimizationRule};
pub use analyzer::{ComplexAnalyzer, AnalysisType, StatisticalAnalysis, TrendAnalysis, AnomalyDetection, AnomalyPoint};
pub use window_functions::{WindowFunctionProcessor, WindowFunctionType, WindowFunctionConfig, WindowFrame, FrameType, FrameBound};
pub use time_series::{TimeSeriesAnalyzer, TimeSeriesAnalysisType, TimeSeriesStats, TimeRange, BasicStats, TrendInfo, SeasonalityInfo};

/// 查询分析服务
pub struct QueryAnalyticsService {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 查询优化器
    optimizer: Arc<QueryOptimizer>,
    /// 复杂分析器
    analyzer: Arc<ComplexAnalyzer>,
    /// 窗口函数处理器
    window_processor: Arc<WindowFunctionProcessor>,
    /// 时间序列分析器
    time_series_analyzer: Arc<TimeSeriesAnalyzer>,
    /// 配置
    config: QueryAnalyticsConfig,
    /// 监控指标
    metrics: QueryAnalyticsMetrics,
}

/// 查询分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalyticsConfig {
    /// 查询缓存大小
    pub cache_size: usize,
    /// 查询超时时间（秒）
    pub query_timeout: u64,
    /// 并行查询线程数
    pub parallel_threads: usize,
    /// 统计信息收集间隔（秒）
    pub stats_collection_interval: u64,
    /// 是否启用查询优化
    pub enable_optimization: bool,
    /// 是否启用执行计划缓存
    pub enable_plan_cache: bool,
}

impl Default for QueryAnalyticsConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            query_timeout: 300,
            parallel_threads: 4,
            stats_collection_interval: 60,
            enable_optimization: true,
            enable_plan_cache: true,
        }
    }
}

/// 查询分析监控指标
#[derive(Debug)]
pub struct QueryAnalyticsMetrics {
    /// 查询总数
    pub queries_total: Counter,
    /// 查询执行时间
    pub query_duration: Histogram,
    /// 优化后的查询数
    pub optimized_queries: Counter,
    /// 缓存命中数
    pub cache_hits: Counter,
    /// 缓存未命中数
    pub cache_misses: Counter,
    /// 当前活跃查询数
    pub active_queries: Gauge,
}

impl QueryAnalyticsMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let queries_total = Counter::new(
            "duckhub_query_analytics_queries_total",
            "Total number of queries processed"
        )?;
        registry.register(Box::new(queries_total.clone()))?;

        let query_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_query_analytics_duration_seconds",
                "Query execution duration in seconds"
            ).buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0, 60.0])
        )?;
        registry.register(Box::new(query_duration.clone()))?;

        let optimized_queries = Counter::new(
            "duckhub_query_analytics_optimized_total",
            "Total number of optimized queries"
        )?;
        registry.register(Box::new(optimized_queries.clone()))?;

        let cache_hits = Counter::new(
            "duckhub_query_analytics_cache_hits_total",
            "Total number of cache hits"
        )?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = Counter::new(
            "duckhub_query_analytics_cache_misses_total",
            "Total number of cache misses"
        )?;
        registry.register(Box::new(cache_misses.clone()))?;

        let active_queries = Gauge::new(
            "duckhub_query_analytics_active_queries",
            "Number of currently active queries"
        )?;
        registry.register(Box::new(active_queries.clone()))?;

        Ok(Self {
            queries_total,
            query_duration,
            optimized_queries,
            cache_hits,
            cache_misses,
            active_queries,
        })
    }
}

/// 健康状态
#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
}

/// 查询结果
#[derive(Debug, Clone, Serialize)]
pub struct QueryResult {
    /// 查询ID
    pub query_id: String,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 返回行数
    pub row_count: usize,
    /// 是否使用了优化
    pub optimized: bool,
    /// 是否命中缓存
    pub cache_hit: bool,
    /// 执行计划
    pub execution_plan: Option<String>,
    /// 结果数据
    pub data: Vec<HashMap<String, serde_json::Value>>,
}

impl QueryAnalyticsService {
    /// 创建新的查询分析服务
    #[instrument(skip(engine, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        config: QueryAnalyticsConfig,
        registry: &Registry,
    ) -> Result<Self> {
        let metrics = QueryAnalyticsMetrics::new(registry)?;

        let optimizer = Arc::new(QueryOptimizer::new(Arc::clone(&engine), &config).await?);
        let analyzer = Arc::new(ComplexAnalyzer::new(Arc::clone(&engine)).await?);
        let window_processor = Arc::new(WindowFunctionProcessor::new(Arc::clone(&engine)).await?);
        let time_series_analyzer = Arc::new(TimeSeriesAnalyzer::new(Arc::clone(&engine)).await?);

        let service = Self {
            engine,
            optimizer,
            analyzer,
            window_processor,
            time_series_analyzer,
            config,
            metrics,
        };

        info!("查询分析服务初始化完成");
        Ok(service)
    }

    /// 执行查询
    #[instrument(skip(self, sql))]
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult> {
        let query_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        self.metrics.queries_total.inc();
        self.metrics.active_queries.inc();

        let result = self.execute_query_internal(sql, &query_id).await;

        self.metrics.active_queries.dec();
        let execution_time = start_time.elapsed();
        self.metrics.query_duration.observe(execution_time.as_secs_f64());

        result
    }

    /// 内部查询执行逻辑
    async fn execute_query_internal(&self, sql: &str, query_id: &str) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();

        // 查询优化
        let (optimized_sql, optimized) = if self.config.enable_optimization {
            match self.optimizer.optimize_query(sql).await {
                Ok(optimized) => {
                    self.metrics.optimized_queries.inc();
                    (optimized, true)
                }
                Err(_) => (sql.to_string(), false)
            }
        } else {
            (sql.to_string(), false)
        };

        // 执行查询
        let data = self.engine.query(&optimized_sql).await?;

        let execution_time = start_time.elapsed();

        Ok(QueryResult {
            query_id: query_id.to_string(),
            execution_time_ms: execution_time.as_millis() as u64,
            row_count: data.len(),
            optimized,
            cache_hit: false, // TODO: 实现缓存逻辑
            execution_plan: None, // TODO: 获取执行计划
            data,
        })
    }

    /// 执行复杂分析查询
    #[instrument(skip(self))]
    pub async fn execute_complex_analysis(&self, analysis_type: &str, params: &HashMap<String, serde_json::Value>) -> Result<QueryResult> {
        self.analyzer.execute_analysis(analysis_type, params).await
    }

    /// 执行窗口函数查询
    #[instrument(skip(self))]
    pub async fn execute_window_function(&self, sql: &str) -> Result<QueryResult> {
        self.window_processor.process_window_query(sql).await
    }

    /// 执行时间序列分析
    #[instrument(skip(self))]
    pub async fn execute_time_series_analysis(&self, table: &str, time_column: &str, value_column: &str) -> Result<QueryResult> {
        self.time_series_analyzer.analyze(table, time_column, value_column).await
    }

    /// 获取查询统计信息
    pub async fn get_query_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();

        stats.insert("total_queries".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.queries_total.get() as u64)));
        stats.insert("optimized_queries".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.optimized_queries.get() as u64)));
        stats.insert("cache_hits".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.cache_hits.get() as u64)));
        stats.insert("cache_misses".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.cache_misses.get() as u64)));
        stats.insert("active_queries".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.active_queries.get() as u64)));

        Ok(stats)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // 检查数据库连接
        match self.engine.check_connection().await {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(e) => {
                warn!("查询分析服务健康检查失败: {}", e);
                Ok(HealthStatus::Unhealthy)
            }
        }
    }
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
    async fn test_query_analytics_service_creation() {
        let engine = create_test_engine().await;
        let config = QueryAnalyticsConfig::default();
        let registry = Registry::new();

        let service = QueryAnalyticsService::new(engine, config, &registry).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_simple_query_execution() {
        let engine = create_test_engine().await;
        let config = QueryAnalyticsConfig::default();
        let registry = Registry::new();

        let service = QueryAnalyticsService::new(engine, config, &registry).await.unwrap();

        // 创建测试表
        let create_table_sql = "CREATE TABLE test_data (id INTEGER, value DOUBLE, name VARCHAR)";
        let _ = service.engine.execute(create_table_sql).await;

        // 插入测试数据
        let insert_sql = "INSERT INTO test_data VALUES (1, 10.5, 'test1'), (2, 20.3, 'test2'), (3, 15.7, 'test3')";
        let _ = service.engine.execute(insert_sql).await;

        // 执行查询
        let result = service.execute_query("SELECT COUNT(*) as count FROM test_data").await.unwrap();

        // 查询应该返回结果，执行时间是u64类型，总是>=0
        assert!(result.query_id.len() > 0); // 检查查询ID是否生成
    }

    #[tokio::test]
    async fn test_query_stats() {
        let engine = create_test_engine().await;
        let config = QueryAnalyticsConfig::default();
        let registry = Registry::new();

        let service = QueryAnalyticsService::new(engine, config, &registry).await.unwrap();
        let stats = service.get_query_stats().await.unwrap();

        assert!(stats.contains_key("total_queries"));
        assert!(stats.contains_key("optimized_queries"));
        assert!(stats.contains_key("cache_hits"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = create_test_engine().await;
        let config = QueryAnalyticsConfig::default();
        let registry = Registry::new();

        let service = QueryAnalyticsService::new(engine, config, &registry).await.unwrap();
        let health = service.health_check().await.unwrap();

        assert!(matches!(health, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_query_optimizer() {
        let engine = create_test_engine().await;
        let config = QueryAnalyticsConfig::default();

        let optimizer = QueryOptimizer::new(engine, &config).await.unwrap();

        // 测试常量折叠优化
        let sql = "SELECT 1 + 1 as result";
        let optimized = optimizer.optimize_query(sql).await.unwrap();
        assert!(optimized.contains("2"));

        // 测试优化建议
        let suggestions = optimizer.get_optimization_suggestions("SELECT * FROM table ORDER BY id");
        assert!(!suggestions.is_empty());
    }
}
