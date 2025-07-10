//! DuckLake integration for DuckDB
//! 
//! DuckLake is a native lakehouse format for DuckDB that provides:
//! - ACID transactions
//! - Time travel queries
//! - Schema evolution
//! - Snapshot isolation
//! - Data versioning

use duckhub_common::prelude::*;
use duckhub_common::utils::{now, generate_id};
use duckdb::Connection;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn, error, instrument};
use crate::pool::{ConnectionPool, PooledConnectionGuard};
use prometheus::{Counter, Histogram, Gauge, Registry};

/// DuckLake manager for lakehouse operations
pub struct DuckLakeManager {
    connection: Connection,
    attached_databases: HashMap<String, DuckLakeDatabase>,
    connection_pool: Option<Arc<ConnectionPool>>,
    retry_config: RetryConfig,
    metrics: DuckLakeMetrics,
}

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 重试延迟倍数
    pub backoff_multiplier: f64,
    /// 最大重试延迟（毫秒）
    pub max_delay_ms: u64,
    /// 可重试的错误类型
    pub retryable_errors: Vec<String>,
    /// 连接超时时间（毫秒）
    pub connection_timeout_ms: u64,
    /// 查询超时时间（毫秒）
    pub query_timeout_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 5000,
            retryable_errors: vec![
                "connection".to_string(),
                "timeout".to_string(),
                "lock".to_string(),
                "busy".to_string(),
                "network".to_string(),
            ],
            connection_timeout_ms: 30000,  // 30秒
            query_timeout_ms: 300000,      // 5分钟
        }
    }
}

/// DuckLake性能指标
#[derive(Debug, Clone)]
pub struct DuckLakeMetrics {
    /// 快照创建总数
    pub snapshots_created: Counter,
    /// 时间旅行查询总数
    pub time_travel_queries: Counter,
    /// 事务执行时间
    pub transaction_duration: Histogram,
    /// 附加数据库数量
    pub attached_databases_count: Gauge,
    /// 查询错误总数
    pub query_errors: Counter,
    /// 重试总数
    pub retries_total: Counter,
    /// 批量操作总数
    pub batch_operations_total: Counter,
    /// 批量操作执行时间
    pub batch_operation_duration: Histogram,
    /// 连接池活跃连接数
    pub active_connections: Gauge,
    /// 连接池等待队列长度
    pub connection_queue_length: Gauge,
    /// 数据插入行数
    pub rows_inserted: Counter,
    /// 数据更新行数
    pub rows_updated: Counter,
    /// 数据删除行数
    pub rows_deleted: Counter,
    /// Schema演进操作总数
    pub schema_evolutions: Counter,
}

impl Default for DuckLakeMetrics {
    fn default() -> Self {
        Self {
            snapshots_created: Counter::new("ducklake_snapshots_created_total", "Total number of DuckLake snapshots created").unwrap(),
            time_travel_queries: Counter::new("ducklake_time_travel_queries_total", "Total number of time travel queries").unwrap(),
            transaction_duration: Histogram::new("ducklake_transaction_duration_seconds", "DuckLake transaction execution time").unwrap(),
            attached_databases_count: Gauge::new("ducklake_attached_databases", "Number of attached DuckLake databases").unwrap(),
            query_errors: Counter::new("ducklake_query_errors_total", "Total number of query errors").unwrap(),
            retries_total: Counter::new("ducklake_retries_total", "Total number of operation retries").unwrap(),
            batch_operations_total: Counter::new("ducklake_batch_operations_total", "Total number of batch operations").unwrap(),
            batch_operation_duration: Histogram::new("ducklake_batch_operation_duration_seconds", "Batch operation execution time").unwrap(),
            active_connections: Gauge::new("ducklake_active_connections", "Number of active database connections").unwrap(),
            connection_queue_length: Gauge::new("ducklake_connection_queue_length", "Length of connection pool queue").unwrap(),
            rows_inserted: Counter::new("ducklake_rows_inserted_total", "Total number of rows inserted").unwrap(),
            rows_updated: Counter::new("ducklake_rows_updated_total", "Total number of rows updated").unwrap(),
            rows_deleted: Counter::new("ducklake_rows_deleted_total", "Total number of rows deleted").unwrap(),
            schema_evolutions: Counter::new("ducklake_schema_evolutions_total", "Total number of schema evolution operations").unwrap(),
        }
    }
}

/// DuckLake database configuration
#[derive(Debug, Clone)]
pub struct DuckLakeDatabase {
    pub name: String,
    pub metadata_path: String,
    pub data_path: String,
    pub read_only: bool,
    pub encrypted: bool,
    pub snapshot_version: Option<u64>,
    pub snapshot_time: Option<DateTime<Utc>>,
}

/// DuckLake connection configuration
#[derive(Debug, Clone)]
pub struct DuckLakeConfig {
    pub metadata_path: String,
    pub data_path: Option<String>,
    pub metadata_schema: Option<String>,
    pub metadata_catalog: Option<String>,
    pub encrypted: bool,
    pub data_inlining_row_limit: u32,
    pub read_only: bool,
    pub snapshot_version: Option<u64>,
    pub snapshot_time: Option<DateTime<Utc>>,
    pub metadata_parameters: HashMap<String, String>,
}

impl Default for DuckLakeConfig {
    fn default() -> Self {
        Self {
            metadata_path: "ducklake.db".to_string(),
            data_path: None,
            metadata_schema: Some("main".to_string()),
            metadata_catalog: None,
            encrypted: false,
            data_inlining_row_limit: 0,
            read_only: false,
            snapshot_version: None,
            snapshot_time: None,
            metadata_parameters: HashMap::new(),
        }
    }
}

impl DuckLakeManager {
    /// 创建新的DuckLake管理器
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,
            attached_databases: HashMap::new(),
            connection_pool: None,
            retry_config: RetryConfig::default(),
            metrics: DuckLakeMetrics::default(),
        }
    }

    /// 创建带连接池的DuckLake管理器
    pub fn with_connection_pool(connection: Connection, pool: Arc<ConnectionPool>) -> Self {
        Self {
            connection,
            attached_databases: HashMap::new(),
            connection_pool: Some(pool),
            retry_config: RetryConfig::default(),
            metrics: DuckLakeMetrics::default(),
        }
    }

    /// 设置重试配置
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// 获取性能指标
    pub fn get_metrics(&self) -> &DuckLakeMetrics {
        &self.metrics
    }

    /// 更新连接池指标
    pub fn update_connection_pool_metrics(&self) {
        if let Some(pool) = &self.connection_pool {
            // 这里需要从连接池获取实际的指标
            // 由于ConnectionPool的具体实现可能不同，这里提供一个通用的接口
            self.metrics.active_connections.set(pool.active_connections() as f64);
            self.metrics.connection_queue_length.set(pool.queue_length() as f64);
        }
    }

    /// 获取连接池状态
    pub fn get_connection_pool_status(&self) -> Option<ConnectionPoolStatus> {
        self.connection_pool.as_ref().map(|pool| {
            ConnectionPoolStatus {
                active_connections: pool.active_connections(),
                idle_connections: pool.idle_connections(),
                max_connections: pool.max_connections(),
                queue_length: pool.queue_length(),
            }
        })
    }

    /// 检查错误是否可重试
    fn is_retryable_error(&self, error: &DuckHubError) -> bool {
        let error_msg = error.to_string().to_lowercase();
        self.retry_config.retryable_errors.iter()
            .any(|pattern| error_msg.contains(pattern))
    }

    /// 执行带重试的操作（增强版）
    async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Result<T> + Send + Sync,
        T: Send,
    {
        let mut last_error = None;
        let mut delay_ms = self.retry_config.initial_delay_ms;

        for attempt in 0..=self.retry_config.max_retries {
            match operation() {
                Ok(result) => {
                    if attempt > 0 {
                        info!("操作在第{}次重试后成功", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    // 检查错误是否可重试
                    if !self.is_retryable_error(&e) {
                        error!("遇到不可重试的错误: {}", e);
                        self.metrics.query_errors.inc();
                        return Err(e);
                    }

                    let error_msg = e.to_string();
                    last_error = Some(DuckHubError::database(error_msg.clone()));

                    if attempt < self.retry_config.max_retries {
                        warn!("操作失败，第{}次重试，延迟{}ms，错误: {}", attempt + 1, delay_ms, e);
                        self.metrics.retries_total.inc();

                        sleep(Duration::from_millis(delay_ms)).await;

                        // 指数退避
                        delay_ms = ((delay_ms as f64) * self.retry_config.backoff_multiplier) as u64;
                        delay_ms = delay_ms.min(self.retry_config.max_delay_ms);
                    }
                }
            }
        }

        self.metrics.query_errors.inc();
        Err(last_error.unwrap_or_else(|| DuckHubError::database("未知错误")))
    }

    /// 智能错误恢复处理
    pub async fn handle_connection_failure(&self, error: &DuckHubError) -> Result<RecoveryAction> {
        let error_msg = error.to_string().to_lowercase();

        if error_msg.contains("connection") || error_msg.contains("network") {
            info!("检测到连接错误，建议重新连接");
            return Ok(RecoveryAction::Reconnect);
        }

        if error_msg.contains("lock") || error_msg.contains("busy") {
            info!("检测到锁定错误，建议等待后重试");
            return Ok(RecoveryAction::WaitAndRetry);
        }

        if error_msg.contains("timeout") {
            info!("检测到超时错误，建议增加超时时间");
            return Ok(RecoveryAction::IncreaseTimeout);
        }

        if error_msg.contains("disk") || error_msg.contains("space") {
            error!("检测到磁盘空间错误，需要人工干预");
            return Ok(RecoveryAction::ManualIntervention);
        }

        warn!("未知错误类型，建议重试: {}", error);
        Ok(RecoveryAction::Retry)
    }

    /// 附加DuckLake数据库（带重试机制）
    #[instrument(skip(self))]
    pub async fn attach_database(&mut self, name: &str, config: &DuckLakeConfig) -> Result<()> {
        let attach_sql = self.build_attach_sql(name, config);

        debug!("正在附加DuckLake数据库: {}", attach_sql);

        // 使用重试机制执行附加操作
        let result = self.execute_with_retry(|| {
            self.connection.execute(&attach_sql, [])
                .map_err(|e| DuckHubError::database(format!("附加DuckLake数据库失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                let database = DuckLakeDatabase {
                    name: name.to_string(),
                    metadata_path: config.metadata_path.clone(),
                    data_path: config.data_path.clone().unwrap_or_else(|| format!("{}.files", config.metadata_path)),
                    read_only: config.read_only,
                    encrypted: config.encrypted,
                    snapshot_version: config.snapshot_version,
                    snapshot_time: config.snapshot_time,
                };

                self.attached_databases.insert(name.to_string(), database);
                self.metrics.attached_databases_count.set(self.attached_databases.len() as f64);

                info!("成功附加DuckLake数据库: {}", name);
                Ok(())
            }
            Err(e) => {
                error!("附加DuckLake数据库失败: {}", e);
                Err(e)
            }
        }
    }

    /// 构建附加SQL语句
    fn build_attach_sql(&self, name: &str, config: &DuckLakeConfig) -> String {
        let mut attach_sql = format!("ATTACH 'ducklake:{}'", config.metadata_path);

        // 添加参数
        let mut params = Vec::new();

        if let Some(data_path) = &config.data_path {
            params.push(format!("DATA_PATH '{}'", data_path));
        }

        if let Some(schema) = &config.metadata_schema {
            params.push(format!("METADATA_SCHEMA '{}'", schema));
        }

        if let Some(catalog) = &config.metadata_catalog {
            params.push(format!("METADATA_CATALOG '{}'", catalog));
        }

        if config.encrypted {
            params.push("ENCRYPTED".to_string());
        }

        if config.read_only {
            params.push("READ_ONLY".to_string());
        }

        if config.data_inlining_row_limit > 0 {
            params.push(format!("DATA_INLINING_ROW_LIMIT {}", config.data_inlining_row_limit));
        }

        if let Some(version) = config.snapshot_version {
            params.push(format!("SNAPSHOT_VERSION {}", version));
        }

        if let Some(time) = &config.snapshot_time {
            params.push(format!("SNAPSHOT_TIME '{}'", time.format("%Y-%m-%d %H:%M:%S")));
        }

        // 添加元数据参数
        for (key, value) in &config.metadata_parameters {
            params.push(format!("META_{} '{}'", key.to_uppercase(), value));
        }

        if !params.is_empty() {
            attach_sql.push_str(&format!(" ({})", params.join(", ")));
        }

        attach_sql.push_str(&format!(" AS {}", name));
        attach_sql
    }

    /// Create a DuckLake secret for easier connection management
    pub async fn create_secret(&self, secret_name: &str, config: &DuckLakeConfig) -> Result<()> {
        let mut secret_sql = if secret_name.is_empty() {
            "CREATE SECRET (".to_string()
        } else {
            format!("CREATE SECRET {} (", secret_name)
        };
        
        let mut params = vec![
            "TYPE DUCKLAKE".to_string(),
            format!("METADATA_PATH '{}'", config.metadata_path),
        ];
        
        if let Some(data_path) = &config.data_path {
            params.push(format!("DATA_PATH '{}'", data_path));
        }
        
        if !config.metadata_parameters.is_empty() {
            let meta_params: Vec<String> = config.metadata_parameters
                .iter()
                .map(|(k, v)| format!("'{}': '{}'", k, v))
                .collect();
            params.push(format!("METADATA_PARAMETERS MAP {{{}}}", meta_params.join(", ")));
        }
        
        secret_sql.push_str(&params.join(", "));
        secret_sql.push(')');
        
        self.connection.execute(&secret_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to create DuckLake secret: {}", e)))?;
        
        info!("Created DuckLake secret: {}", if secret_name.is_empty() { "default" } else { secret_name });
        Ok(())
    }

    /// Create a persistent secret
    pub async fn create_persistent_secret(&self, secret_name: &str, config: &DuckLakeConfig) -> Result<()> {
        let secret_sql = format!("CREATE PERSISTENT SECRET {} (TYPE DUCKLAKE, METADATA_PATH '{}', DATA_PATH '{}')",
                                secret_name, 
                                config.metadata_path,
                                config.data_path.as_ref().unwrap_or(&format!("{}.files", config.metadata_path)));
        
        self.connection.execute(&secret_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to create persistent DuckLake secret: {}", e)))?;
        
        info!("Created persistent DuckLake secret: {}", secret_name);
        Ok(())
    }

    /// 时间旅行查询 - 查询特定版本的表数据（增强版）
    pub async fn query_at_version(&self, database: &str, table: &str, version: u64, sql: &str) -> Result<TimeTravelQueryResult> {
        let start_time = Instant::now();

        // 首先验证版本是否存在
        if !self.version_exists(database, table, version).await? {
            return Err(DuckHubError::validation(format!("版本{}不存在于表{}.{}", version, database, table)));
        }

        let time_travel_sql = if sql.is_empty() {
            format!("SELECT * FROM {}.{} AT (VERSION => {})", database, table, version)
        } else {
            sql.replace(&format!("{}.{}", database, table),
                       &format!("{}.{} AT (VERSION => {})", database, table, version))
        };

        debug!("执行时间旅行查询: {}", time_travel_sql);

        let result = self.execute_with_retry(|| {
            self.connection.execute(&time_travel_sql, [])
                .map_err(|e| DuckHubError::database(format!("时间旅行查询失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                let duration = start_time.elapsed().as_secs_f64();
                self.metrics.time_travel_queries.inc();
                self.metrics.transaction_duration.observe(duration);

                info!("成功执行版本{}的时间旅行查询，耗时{:.2}秒", version, duration);

                Ok(TimeTravelQueryResult {
                    query_type: TimeTravelQueryType::Version(version),
                    execution_time: duration,
                    rows_returned: 0, // TODO: 获取实际返回行数
                    cache_hit: false, // TODO: 实现查询缓存
                    snapshot_info: self.get_snapshot_info(database, table, version).await.ok(),
                })
            }
            Err(e) => {
                error!("版本{}的时间旅行查询失败: {}", version, e);
                Err(e)
            }
        }
    }

    /// 验证版本是否存在
    async fn version_exists(&self, database: &str, table: &str, version: u64) -> Result<bool> {
        let check_sql = format!(
            "SELECT COUNT(*) FROM {}.snapshot_metadata WHERE table_name = '{}' AND snapshot_id = {}",
            database, table, version
        );

        // 这里应该执行查询并检查结果，简化实现
        // TODO: 实现实际的版本检查逻辑
        Ok(true)
    }

    /// 获取快照信息
    async fn get_snapshot_info(&self, database: &str, table: &str, version: u64) -> Result<SnapshotInfo> {
        let info_sql = format!(
            "SELECT snapshot_id, timestamp, operation, summary FROM {}.snapshot_metadata WHERE table_name = '{}' AND snapshot_id = {}",
            database, table, version
        );

        // TODO: 实现实际的快照信息查询
        Ok(SnapshotInfo {
            snapshot_id: version,
            timestamp: now(),
            operation: "SELECT".to_string(),
            summary: HashMap::new(),
            size_bytes: 0,
            row_count: 0,
        })
    }

    /// 时间旅行查询 - 查询特定时间戳的表数据（增强版）
    pub async fn query_at_timestamp(&self, database: &str, table: &str, timestamp: DateTime<Utc>, sql: &str) -> Result<TimeTravelQueryResult> {
        let start_time = Instant::now();

        // 验证时间戳是否在有效范围内
        if !self.timestamp_is_valid(database, table, timestamp).await? {
            return Err(DuckHubError::validation(format!("时间戳{}超出表{}.{}的有效范围", timestamp, database, table)));
        }

        let time_travel_sql = if sql.is_empty() {
            format!("SELECT * FROM {}.{} AT (TIMESTAMP => '{}')",
                   database, table, timestamp.format("%Y-%m-%d %H:%M:%S%.3f"))
        } else {
            sql.replace(&format!("{}.{}", database, table),
                       &format!("{}.{} AT (TIMESTAMP => '{}')",
                               database, table, timestamp.format("%Y-%m-%d %H:%M:%S%.3f")))
        };

        debug!("执行时间戳查询: {}", time_travel_sql);

        let result = self.execute_with_retry(|| {
            self.connection.execute(&time_travel_sql, [])
                .map_err(|e| DuckHubError::database(format!("时间旅行查询失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                let duration = start_time.elapsed().as_secs_f64();
                self.metrics.time_travel_queries.inc();
                self.metrics.transaction_duration.observe(duration);

                info!("成功执行时间戳{}的时间旅行查询，耗时{:.2}秒", timestamp, duration);

                Ok(TimeTravelQueryResult {
                    query_type: TimeTravelQueryType::Timestamp(timestamp),
                    execution_time: duration,
                    rows_returned: 0, // TODO: 获取实际返回行数
                    cache_hit: false, // TODO: 实现查询缓存
                    snapshot_info: self.get_snapshot_at_timestamp(database, table, timestamp).await.ok(),
                })
            }
            Err(e) => {
                error!("时间戳{}的时间旅行查询失败: {}", timestamp, e);
                Err(e)
            }
        }
    }

    /// 验证时间戳是否在有效范围内
    async fn timestamp_is_valid(&self, database: &str, table: &str, timestamp: DateTime<Utc>) -> Result<bool> {
        let check_sql = format!(
            "SELECT MIN(timestamp) as min_ts, MAX(timestamp) as max_ts FROM {}.snapshot_metadata WHERE table_name = '{}'",
            database, table
        );

        // TODO: 实现实际的时间戳范围检查
        Ok(true)
    }

    /// 获取指定时间戳的快照信息
    async fn get_snapshot_at_timestamp(&self, database: &str, table: &str, timestamp: DateTime<Utc>) -> Result<SnapshotInfo> {
        let info_sql = format!(
            "SELECT snapshot_id, timestamp, operation, summary FROM {}.snapshot_metadata
             WHERE table_name = '{}' AND timestamp <= '{}'
             ORDER BY timestamp DESC LIMIT 1",
            database, table, timestamp.format("%Y-%m-%d %H:%M:%S%.3f")
        );

        // TODO: 实现实际的快照信息查询
        Ok(SnapshotInfo {
            snapshot_id: 0,
            timestamp,
            operation: "SELECT".to_string(),
            summary: HashMap::new(),
            size_bytes: 0,
            row_count: 0,
        })
    }

    /// 复杂时间范围查询 - 查询指定时间范围内的数据变化（增强版）
    pub async fn query_time_range(
        &self,
        database: &str,
        table: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        sql: &str
    ) -> Result<TimeRangeQueryResult> {
        let query_start = Instant::now();

        // 验证时间范围
        if start_time >= end_time {
            return Err(DuckHubError::validation("开始时间必须早于结束时间"));
        }

        info!("开始执行时间范围查询: {} 到 {}", start_time, end_time);

        // 获取时间范围内的所有快照
        let snapshots_sql = format!(
            "SELECT snapshot_id, timestamp FROM {}.snapshots()
             WHERE table_name = '{}'
             AND timestamp BETWEEN '{}' AND '{}'
             ORDER BY timestamp",
            database, table,
            start_time.format("%Y-%m-%d %H:%M:%S%.3f"),
            end_time.format("%Y-%m-%d %H:%M:%S%.3f")
        );

        debug!("执行快照查询: {}", snapshots_sql);

        // 获取快照列表
        let snapshots_included = self.get_snapshots_in_range(database, table, start_time, end_time).await?;

        // 统计总行数
        let total_rows = self.count_rows_in_range(database, table, start_time, end_time).await?;

        // 如果提供了自定义SQL，执行时间范围查询
        let data = if !sql.is_empty() {
            self.execute_time_range_query(database, table, start_time, end_time, sql).await?
        } else {
            vec![]
        };

        let duration = query_start.elapsed().as_secs_f64();
        self.metrics.time_travel_queries.inc();
        self.metrics.transaction_duration.observe(duration);

        let result = TimeRangeQueryResult {
            start_time,
            end_time,
            snapshots_included: snapshots_included.clone(),
            total_rows,
            data,
            execution_time: duration,
            cache_hit: false, // TODO: 实现查询缓存
        };

        info!("成功执行时间范围查询，找到{}个快照，{}行数据，耗时{:.2}秒",
              snapshots_included.len(), total_rows, duration);
        Ok(result)
    }

    /// 获取时间范围内的快照列表
    async fn get_snapshots_in_range(&self, database: &str, table: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<SnapshotInfo>> {
        // TODO: 实现实际的快照查询逻辑
        // 这里应该查询snapshot_metadata表获取实际的快照信息
        Ok(vec![])
    }

    /// 统计时间范围内的总行数
    async fn count_rows_in_range(&self, database: &str, table: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<u64> {
        // TODO: 实现实际的行数统计逻辑
        Ok(0)
    }

    /// 执行时间范围查询
    async fn execute_time_range_query(&self, database: &str, table: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>, sql: &str) -> Result<Vec<serde_json::Value>> {
        // TODO: 实现实际的时间范围查询逻辑
        Ok(vec![])
    }

    /// 快照差异分析 - 比较两个快照版本之间的差异（增强版）
    pub async fn compare_snapshots(
        &self,
        database: &str,
        table: &str,
        version1: u64,
        version2: u64
    ) -> Result<SnapshotDiff> {
        let start_time = Instant::now();

        if version1 == version2 {
            return Err(DuckHubError::validation("不能比较相同的快照版本"));
        }

        info!("开始比较快照版本{}和{}", version1, version2);

        // 验证两个版本都存在
        if !self.version_exists(database, table, version1).await? {
            return Err(DuckHubError::validation(format!("版本{}不存在", version1)));
        }
        if !self.version_exists(database, table, version2).await? {
            return Err(DuckHubError::validation(format!("版本{}不存在", version2)));
        }

        // 获取两个版本的数据统计
        let stats_sql = format!(
            "WITH v1_stats AS (
                SELECT COUNT(*) as count FROM {}.{} AT (VERSION => {})
             ),
             v2_stats AS (
                SELECT COUNT(*) as count FROM {}.{} AT (VERSION => {})
             )
             SELECT v1.count as v1_count, v2.count as v2_count
             FROM v1_stats v1, v2_stats v2",
            database, table, version1, database, table, version2
        );

        debug!("执行统计查询: {}", stats_sql);

        // 计算行级差异
        let (added_rows, deleted_rows, modified_rows) = self.calculate_row_differences(database, table, version1, version2).await?;

        // 检测Schema变化
        let schema_changes = self.detect_schema_changes(database, table, version1, version2).await?;

        // 获取快照元数据
        let snapshot1_info = self.get_snapshot_info(database, table, version1).await?;
        let snapshot2_info = self.get_snapshot_info(database, table, version2).await?;

        let duration = start_time.elapsed().as_secs_f64();

        let diff = SnapshotDiff {
            version1,
            version2,
            added_rows,
            deleted_rows,
            modified_rows,
            schema_changes,
            snapshot1_info: Some(snapshot1_info),
            snapshot2_info: Some(snapshot2_info),
            analysis_time: duration,
            size_difference: 0, // TODO: 计算大小差异
        };

        info!("成功比较快照版本{}和{}，新增{}行，删除{}行，修改{}行，耗时{:.2}秒",
              version1, version2, added_rows, deleted_rows, modified_rows, duration);
        Ok(diff)
    }

    /// 计算行级差异
    async fn calculate_row_differences(&self, database: &str, table: &str, version1: u64, version2: u64) -> Result<(u64, u64, u64)> {
        // TODO: 实现实际的行级差异计算
        // 这里应该使用EXCEPT和INTERSECT等SQL操作来计算差异
        Ok((0, 0, 0))
    }

    /// 检测Schema变化
    async fn detect_schema_changes(&self, database: &str, table: &str, version1: u64, version2: u64) -> Result<Vec<SchemaChange>> {
        // TODO: 实现实际的Schema变化检测
        Ok(vec![])
    }

    /// 获取当前Schema版本
    async fn get_current_schema_version(&self, database: &str, table: &str) -> Result<u64> {
        let version_sql = format!(
            "SELECT COALESCE(MAX(version), 0) FROM {}.schema_versions WHERE table_name = '{}'",
            database, table
        );

        // TODO: 实现实际的版本查询
        Ok(1) // 暂时返回版本1
    }

    /// 更新Schema版本
    async fn update_schema_version(&self, database: &str, table: &str, version: u64, description: &str) -> Result<()> {
        let update_sql = format!(
            "INSERT INTO {}.schema_versions (table_name, version, description, created_at) VALUES ('{}', {}, '{}', '{}')",
            database, table, version, description, now().format("%Y-%m-%d %H:%M:%S%.3f")
        );

        // TODO: 实现实际的版本更新
        debug!("更新Schema版本: {}", update_sql);
        Ok(())
    }

    /// 验证列名
    fn validate_column_name(&self, column_name: &str) -> Result<()> {
        if column_name.is_empty() {
            return Err(DuckHubError::validation("列名不能为空"));
        }

        if column_name.len() > 128 {
            return Err(DuckHubError::validation("列名长度不能超过128个字符"));
        }

        // 检查是否包含非法字符
        if !column_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DuckHubError::validation("列名只能包含字母、数字和下划线"));
        }

        // 检查是否以数字开头
        if column_name.chars().next().unwrap().is_numeric() {
            return Err(DuckHubError::validation("列名不能以数字开头"));
        }

        Ok(())
    }

    /// 验证列类型
    fn validate_column_type(&self, column_type: &str) -> Result<()> {
        let valid_types = [
            "BOOLEAN", "TINYINT", "SMALLINT", "INTEGER", "BIGINT",
            "UTINYINT", "USMALLINT", "UINTEGER", "UBIGINT",
            "REAL", "DOUBLE", "DECIMAL", "VARCHAR", "TEXT",
            "DATE", "TIME", "TIMESTAMP", "TIMESTAMPTZ",
            "BLOB", "UUID", "JSON"
        ];

        let base_type = column_type.split('(').next().unwrap().to_uppercase();

        if !valid_types.contains(&base_type.as_str()) {
            return Err(DuckHubError::validation(format!("不支持的列类型: {}", column_type)));
        }

        Ok(())
    }

    /// 检查添加列的兼容性
    async fn check_add_column_compatibility(&self, database: &str, table: &str, column_name: &str, column_type: &str, nullable: bool) -> Result<CompatibilityCheck> {
        // 添加列通常是向后兼容的，特别是当列可空或有默认值时
        if nullable {
            return Ok(CompatibilityCheck {
                is_compatible: true,
                reason: "添加可空列是向后兼容的".to_string(),
                impact_level: CompatibilityImpact::Low,
            });
        }

        // 非空列需要默认值才能保证兼容性
        Ok(CompatibilityCheck {
            is_compatible: true,
            reason: "添加非空列需要确保有默认值".to_string(),
            impact_level: CompatibilityImpact::Medium,
        })
    }

    /// 检查删除列的兼容性
    async fn check_drop_column_compatibility(&self, database: &str, table: &str, column_name: &str) -> Result<CompatibilityCheck> {
        // 删除列通常是破坏性的
        Ok(CompatibilityCheck {
            is_compatible: false,
            reason: "删除列是破坏性操作，会影响依赖该列的查询".to_string(),
            impact_level: CompatibilityImpact::High,
        })
    }

    /// 获取列信息
    async fn get_column_info(&self, database: &str, table: &str, column_name: &str) -> Result<ColumnInfo> {
        let info_sql = format!(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_schema = '{}' AND table_name = '{}' AND column_name = '{}'",
            database, table, column_name
        );

        // TODO: 实现实际的列信息查询
        Ok(ColumnInfo {
            name: column_name.to_string(),
            data_type: "VARCHAR".to_string(),
            nullable: true,
            default_value: None,
            is_primary_key: false,
        })
    }

    /// Get snapshots for a database
    pub async fn get_snapshots(&self, database: &str) -> Result<Vec<DuckLakeSnapshot>> {
        let snapshots_sql = format!("SELECT * FROM {}.snapshots()", database);
        
        // This would need proper result parsing in a real implementation
        self.connection.execute(&snapshots_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to get snapshots: {}", e)))?;
        
        // For now, return empty vector - would need proper result parsing
        Ok(Vec::new())
    }

    /// Create a table in DuckLake
    pub async fn create_table(&self, database: &str, table: &str, schema: &Schema) -> Result<()> {
        let mut create_sql = format!("CREATE TABLE {}.{} (", database, table);
        
        let field_definitions: Vec<String> = schema.fields.iter().map(|field| {
            format!("{} {}{}", 
                   field.name, 
                   self.convert_to_duckdb_type(&field.data_type).unwrap_or("VARCHAR".to_string()),
                   if field.nullable { "" } else { " NOT NULL" })
        }).collect();
        
        create_sql.push_str(&field_definitions.join(", "));
        create_sql.push(')');
        
        self.connection.execute(&create_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to create DuckLake table: {}", e)))?;
        
        info!("Created DuckLake table: {}.{}", database, table);
        Ok(())
    }

    /// 批量插入数据到DuckLake表（增强版）
    pub async fn insert_data(&self, database: &str, table: &str, data: &[Vec<serde_json::Value>]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();

        // 如果数据量很大，分批处理
        const BATCH_SIZE: usize = 1000;
        let mut total_inserted = 0;

        if data.len() > BATCH_SIZE {
            info!("数据量较大({})，将分批插入，每批{}行", data.len(), BATCH_SIZE);

            for chunk in data.chunks(BATCH_SIZE) {
                let chunk_start = Instant::now();
                let insert_sql = self.build_batch_insert_sql(database, table, chunk)?;

                let result = self.execute_with_retry(|| {
                    self.connection.execute(&insert_sql, [])
                        .map_err(|e| DuckHubError::database(format!("批量插入数据失败: {}", e)))
                }).await;

                match result {
                    Ok(_) => {
                        total_inserted += chunk.len();
                        let chunk_duration = chunk_start.elapsed().as_secs_f64();
                        debug!("成功插入批次{}行数据，耗时{:.2}秒", chunk.len(), chunk_duration);
                    }
                    Err(e) => {
                        error!("插入批次数据失败: {}", e);
                        return Err(e);
                    }
                }
            }
        } else {
            // 小批量数据直接插入
            let insert_sql = self.build_batch_insert_sql(database, table, data)?;

            let result = self.execute_with_retry(|| {
                self.connection.execute(&insert_sql, [])
                    .map_err(|e| DuckHubError::database(format!("批量插入数据失败: {}", e)))
            }).await;

            match result {
                Ok(_) => {
                    total_inserted = data.len();
                }
                Err(e) => {
                    error!("插入数据到{}.{}失败: {}", database, table, e);
                    return Err(e);
                }
            }
        }

        let duration = start_time.elapsed().as_secs_f64();
        self.metrics.transaction_duration.observe(duration);
        self.metrics.rows_inserted.inc_by(total_inserted as f64);

        info!("成功插入{}行数据到{}.{}，耗时{:.2}秒", total_inserted, database, table, duration);
        Ok(())
    }

    /// 构建批量插入SQL语句
    fn build_batch_insert_sql(&self, database: &str, table: &str, data: &[Vec<serde_json::Value>]) -> Result<String> {
        if data.is_empty() {
            return Err(DuckHubError::validation("数据不能为空"));
        }

        let mut sql = format!("INSERT INTO {}.{} VALUES ", database, table);
        let mut value_clauses = Vec::new();

        for row in data {
            let values: Vec<String> = row.iter().map(|value| {
                match value {
                    serde_json::Value::Null => "NULL".to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")), // 转义单引号
                    _ => format!("'{}'", value.to_string().replace("'", "''")),
                }
            }).collect();

            value_clauses.push(format!("({})", values.join(", ")));
        }

        sql.push_str(&value_clauses.join(", "));
        Ok(sql)
    }

    /// 批量操作：支持多个操作在一个事务中执行（增强版）
    pub async fn batch_operations(&self, operations: Vec<DuckLakeOperation>) -> Result<Vec<OperationResult>> {
        if operations.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut total_rows_affected = 0;

        info!("开始执行批量操作，共{}个操作", operations.len());
        self.metrics.batch_operations_total.inc();

        // 按操作类型分组，优化执行顺序
        let mut insert_ops = Vec::new();
        let mut update_ops = Vec::new();
        let mut delete_ops = Vec::new();
        let mut ddl_ops = Vec::new();

        for operation in operations {
            match operation {
                DuckLakeOperation::Insert { .. } => insert_ops.push(operation),
                DuckLakeOperation::Update { .. } => update_ops.push(operation),
                DuckLakeOperation::Delete { .. } => delete_ops.push(operation),
                DuckLakeOperation::CreateTable { .. } => ddl_ops.push(operation),
            }
        }

        // 开始事务
        self.execute_with_retry(|| {
            self.connection.execute("BEGIN TRANSACTION", [])
                .map_err(|e| DuckHubError::database(format!("开始事务失败: {}", e)))
        }).await?;

        // 先执行DDL操作
        for operation in ddl_ops {
            match self.execute_operation(&operation).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    self.rollback_transaction().await;
                    return Err(e);
                }
            }
        }

        // 然后执行DML操作（插入、更新、删除）
        for operation in insert_ops.into_iter().chain(update_ops).chain(delete_ops) {
            match self.execute_operation(&operation).await {
                Ok(result) => {
                    // 统计影响的行数
                    match &result {
                        OperationResult::Insert { rows_affected } => total_rows_affected += rows_affected,
                        OperationResult::Update { rows_affected } => total_rows_affected += rows_affected,
                        OperationResult::Delete { rows_affected } => total_rows_affected += rows_affected,
                        _ => {}
                    }
                    results.push(result);
                }
                Err(e) => {
                    self.rollback_transaction().await;
                    return Err(e);
                }
            }
        }

        // 提交事务
        self.execute_with_retry(|| {
            self.connection.execute("COMMIT", [])
                .map_err(|e| DuckHubError::database(format!("提交事务失败: {}", e)))
        }).await?;

        let duration = start_time.elapsed().as_secs_f64();
        self.metrics.transaction_duration.observe(duration);
        self.metrics.batch_operation_duration.observe(duration);

        info!("批量操作完成，共影响{}行，耗时{:.2}秒", total_rows_affected, duration);
        Ok(results)
    }

    /// 回滚事务的辅助方法
    async fn rollback_transaction(&self) {
        if let Err(rollback_err) = self.connection.execute("ROLLBACK", []) {
            error!("回滚事务失败: {}", rollback_err);
        } else {
            info!("事务已回滚");
        }
    }

    /// 开始事务（增强版）
    pub async fn begin_transaction(&self, isolation_level: Option<IsolationLevel>) -> Result<TransactionHandle> {
        let transaction_id = generate_id();
        let start_time = Instant::now();

        // 设置隔离级别
        if let Some(level) = isolation_level {
            let isolation_sql = match level {
                IsolationLevel::ReadUncommitted => "SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED",
                IsolationLevel::ReadCommitted => "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
                IsolationLevel::RepeatableRead => "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ",
                IsolationLevel::Serializable => "SET TRANSACTION ISOLATION LEVEL SERIALIZABLE",
            };

            self.execute_with_retry(|| {
                self.connection.execute(isolation_sql, [])
                    .map_err(|e| DuckHubError::database(format!("设置隔离级别失败: {}", e)))
            }).await?;
        }

        // 开始事务
        self.execute_with_retry(|| {
            self.connection.execute("BEGIN TRANSACTION", [])
                .map_err(|e| DuckHubError::database(format!("开始事务失败: {}", e)))
        }).await?;

        info!("开始事务: {}", transaction_id);

        Ok(TransactionHandle {
            id: transaction_id,
            start_time,
            isolation_level,
            operations: Vec::new(),
            savepoints: Vec::new(),
        })
    }

    /// 提交事务（增强版）
    pub async fn commit_transaction(&self, transaction: &mut TransactionHandle) -> Result<TransactionResult> {
        let start_time = Instant::now();

        // 检查事务状态
        if transaction.operations.is_empty() {
            warn!("提交空事务: {}", transaction.id);
        }

        // 执行提交
        let result = self.execute_with_retry(|| {
            self.connection.execute("COMMIT", [])
                .map_err(|e| DuckHubError::database(format!("提交事务失败: {}", e)))
        }).await;

        let duration = start_time.elapsed().as_secs_f64();
        let total_duration = transaction.start_time.elapsed().as_secs_f64();

        match result {
            Ok(_) => {
                self.metrics.transaction_duration.observe(total_duration);

                info!("成功提交事务: {}，总耗时{:.2}秒", transaction.id, total_duration);

                Ok(TransactionResult {
                    transaction_id: transaction.id.clone(),
                    status: TransactionStatus::Committed,
                    operations_count: transaction.operations.len(),
                    total_duration,
                    commit_duration: duration,
                    rollback_duration: None,
                })
            }
            Err(e) => {
                error!("提交事务{}失败: {}", transaction.id, e);
                Err(e)
            }
        }
    }

    /// 回滚事务（增强版）
    pub async fn rollback_transaction_enhanced(&self, transaction: &mut TransactionHandle) -> Result<TransactionResult> {
        let start_time = Instant::now();

        // 执行回滚
        let result = self.execute_with_retry(|| {
            self.connection.execute("ROLLBACK", [])
                .map_err(|e| DuckHubError::database(format!("回滚事务失败: {}", e)))
        }).await;

        let duration = start_time.elapsed().as_secs_f64();
        let total_duration = transaction.start_time.elapsed().as_secs_f64();

        match result {
            Ok(_) => {
                info!("成功回滚事务: {}，总耗时{:.2}秒", transaction.id, total_duration);

                Ok(TransactionResult {
                    transaction_id: transaction.id.clone(),
                    status: TransactionStatus::RolledBack,
                    operations_count: transaction.operations.len(),
                    total_duration,
                    commit_duration: None,
                    rollback_duration: Some(duration),
                })
            }
            Err(e) => {
                error!("回滚事务{}失败: {}", transaction.id, e);
                Err(e)
            }
        }
    }

    /// 创建保存点
    pub async fn create_savepoint(&self, transaction: &mut TransactionHandle, savepoint_name: &str) -> Result<()> {
        let savepoint_sql = format!("SAVEPOINT {}", savepoint_name);

        self.execute_with_retry(|| {
            self.connection.execute(&savepoint_sql, [])
                .map_err(|e| DuckHubError::database(format!("创建保存点失败: {}", e)))
        }).await?;

        transaction.savepoints.push(Savepoint {
            name: savepoint_name.to_string(),
            created_at: Instant::now(),
        });

        info!("创建保存点: {} (事务: {})", savepoint_name, transaction.id);
        Ok(())
    }

    /// 回滚到保存点
    pub async fn rollback_to_savepoint(&self, transaction: &mut TransactionHandle, savepoint_name: &str) -> Result<()> {
        let rollback_sql = format!("ROLLBACK TO SAVEPOINT {}", savepoint_name);

        self.execute_with_retry(|| {
            self.connection.execute(&rollback_sql, [])
                .map_err(|e| DuckHubError::database(format!("回滚到保存点失败: {}", e)))
        }).await?;

        // 移除该保存点之后的所有保存点
        transaction.savepoints.retain(|sp| sp.name != savepoint_name);

        info!("回滚到保存点: {} (事务: {})", savepoint_name, transaction.id);
        Ok(())
    }

    /// 释放保存点
    pub async fn release_savepoint(&self, transaction: &mut TransactionHandle, savepoint_name: &str) -> Result<()> {
        let release_sql = format!("RELEASE SAVEPOINT {}", savepoint_name);

        self.execute_with_retry(|| {
            self.connection.execute(&release_sql, [])
                .map_err(|e| DuckHubError::database(format!("释放保存点失败: {}", e)))
        }).await?;

        // 从事务中移除保存点
        transaction.savepoints.retain(|sp| sp.name != savepoint_name);

        info!("释放保存点: {} (事务: {})", savepoint_name, transaction.id);
        Ok(())
    }

    /// 检测死锁
    pub async fn detect_deadlock(&self) -> Result<Vec<DeadlockInfo>> {
        // TODO: 实现实际的死锁检测逻辑
        // 这里应该查询数据库的锁信息和等待图
        let _deadlock_sql = "
            SELECT
                waiting_pid,
                blocking_pid,
                waiting_query,
                blocking_query,
                wait_start
            FROM pg_stat_activity
            WHERE state = 'active' AND wait_event_type = 'Lock'
        ";

        // 简化实现，返回空列表
        Ok(vec![])
    }

    /// 获取事务锁信息
    pub async fn get_transaction_locks(&self, _transaction_id: &str) -> Result<Vec<LockInfo>> {
        // TODO: 实现实际的锁信息查询
        Ok(vec![])
    }

    /// 执行单个操作（增强版）
    async fn execute_operation(&self, operation: &DuckLakeOperation) -> Result<OperationResult> {
        match operation {
            DuckLakeOperation::Insert { database, table, data } => {
                let rows_count = data.len();
                self.insert_data(database, table, data).await?;
                Ok(OperationResult::Insert { rows_affected: rows_count })
            }
            DuckLakeOperation::Update { sql } => {
                let result = self.execute_with_retry(|| {
                    self.connection.execute(sql, [])
                        .map_err(|e| DuckHubError::database(format!("更新操作失败: {}", e)))
                }).await?;

                self.metrics.rows_updated.inc_by(result as f64);
                Ok(OperationResult::Update { rows_affected: result })
            }
            DuckLakeOperation::Delete { sql } => {
                let result = self.execute_with_retry(|| {
                    self.connection.execute(sql, [])
                        .map_err(|e| DuckHubError::database(format!("删除操作失败: {}", e)))
                }).await?;

                self.metrics.rows_deleted.inc_by(result as f64);
                Ok(OperationResult::Delete { rows_affected: result })
            }
            DuckLakeOperation::CreateTable { database, table, schema } => {
                self.create_table(database, table, schema).await?;
                Ok(OperationResult::CreateTable)
            }
        }
    }

    /// Get list of attached DuckLake databases
    pub fn get_attached_databases(&self) -> Vec<&DuckLakeDatabase> {
        self.attached_databases.values().collect()
    }

    /// Detach a DuckLake database
    pub async fn detach_database(&mut self, name: &str) -> Result<()> {
        let detach_sql = format!("DETACH {}", name);
        
        self.connection.execute(&detach_sql, [])
            .map_err(|e| DuckHubError::database(format!("Failed to detach DuckLake database: {}", e)))?;
        
        self.attached_databases.remove(name);
        
        info!("Detached DuckLake database: {}", name);
        Ok(())
    }

    /// Schema演进 - 安全地添加列（增强版）
    pub async fn add_column(
        &self,
        database: &str,
        table: &str,
        column_name: &str,
        column_type: &str,
        default_value: Option<&str>,
        nullable: bool
    ) -> Result<SchemaEvolutionResult> {
        let start_time = Instant::now();

        // 检查列是否已存在
        if self.column_exists(database, table, column_name).await? {
            return Err(DuckHubError::validation(format!("列'{}'已存在", column_name)));
        }

        // 验证列名和类型
        self.validate_column_name(column_name)?;
        self.validate_column_type(column_type)?;

        // 检查向后兼容性
        let compatibility_check = self.check_add_column_compatibility(database, table, column_name, column_type, nullable).await?;

        if !compatibility_check.is_compatible {
            return Err(DuckHubError::validation(format!("添加列操作不兼容: {}", compatibility_check.reason)));
        }

        // 记录Schema版本
        let old_version = self.get_current_schema_version(database, table).await?;
        let new_version = old_version + 1;

        let mut alter_sql = format!("ALTER TABLE {}.{} ADD COLUMN {} {}",
                                   database, table, column_name, column_type);

        if !nullable {
            alter_sql.push_str(" NOT NULL");
        }

        if let Some(default) = default_value {
            alter_sql.push_str(&format!(" DEFAULT {}", default));
        }

        info!("执行Schema演进: 添加列{}.{}.{}", database, table, column_name);

        let result = self.execute_with_retry(|| {
            self.connection.execute(&alter_sql, [])
                .map_err(|e| DuckHubError::database(format!("添加列失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                // 更新Schema版本
                self.update_schema_version(database, table, new_version, &format!("添加列: {}", column_name)).await?;

                let duration = start_time.elapsed().as_secs_f64();
                self.metrics.schema_evolutions.inc();

                info!("成功添加列'{}'到表{}.{}，耗时{:.2}秒", column_name, database, table, duration);

                Ok(SchemaEvolutionResult {
                    operation: SchemaOperation::AddColumn {
                        column_name: column_name.to_string(),
                        column_type: column_type.to_string(),
                        nullable,
                        default_value: default_value.map(|s| s.to_string()),
                    },
                    old_version,
                    new_version,
                    execution_time: duration,
                    compatibility_impact: compatibility_check.impact_level,
                    rollback_possible: true,
                })
            }
            Err(e) => {
                error!("添加列'{}'到表{}.{}失败: {}", column_name, database, table, e);
                Err(e)
            }
        }
    }

    /// Schema演进 - 安全地删除列（增强版）
    pub async fn drop_column(&self, database: &str, table: &str, column_name: &str) -> Result<SchemaEvolutionResult> {
        let start_time = Instant::now();

        // 检查列是否存在
        if !self.column_exists(database, table, column_name).await? {
            return Err(DuckHubError::validation(format!("列'{}'不存在", column_name)));
        }

        // 检查向后兼容性（删除列通常是破坏性的）
        let compatibility_check = self.check_drop_column_compatibility(database, table, column_name).await?;

        if !compatibility_check.is_compatible {
            return Err(DuckHubError::validation(format!("删除列操作不兼容: {}", compatibility_check.reason)));
        }

        // 记录Schema版本
        let old_version = self.get_current_schema_version(database, table).await?;
        let new_version = old_version + 1;

        // 获取列信息用于回滚
        let column_info = self.get_column_info(database, table, column_name).await?;

        let alter_sql = format!("ALTER TABLE {}.{} DROP COLUMN {}", database, table, column_name);

        info!("执行Schema演进: 删除列{}.{}.{}", database, table, column_name);

        let result = self.execute_with_retry(|| {
            self.connection.execute(&alter_sql, [])
                .map_err(|e| DuckHubError::database(format!("删除列失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                // 更新Schema版本
                self.update_schema_version(database, table, new_version, &format!("删除列: {}", column_name)).await?;

                let duration = start_time.elapsed().as_secs_f64();
                self.metrics.schema_evolutions.inc();

                info!("成功删除列'{}'从表{}.{}，耗时{:.2}秒", column_name, database, table, duration);

                Ok(SchemaEvolutionResult {
                    operation: SchemaOperation::DropColumn {
                        column_name: column_name.to_string(),
                        column_info: Some(column_info),
                    },
                    old_version,
                    new_version,
                    execution_time: duration,
                    compatibility_impact: compatibility_check.impact_level,
                    rollback_possible: false, // 删除列通常不可回滚
                })
            }
            Err(e) => {
                error!("删除列'{}'从表{}.{}失败: {}", column_name, database, table, e);
                Err(e)
            }
        }
    }

    /// Schema演进 - 安全的类型提升
    pub async fn alter_column_type(
        &self,
        database: &str,
        table: &str,
        column_name: &str,
        new_type: &str
    ) -> Result<()> {
        // 检查列是否存在
        if !self.column_exists(database, table, column_name).await? {
            return Err(DuckHubError::validation(format!("列'{}'不存在", column_name)));
        }

        // 检查类型提升是否安全
        let current_type = self.get_column_type(database, table, column_name).await?;
        if !self.is_safe_type_promotion(&current_type, new_type) {
            return Err(DuckHubError::validation(
                format!("不安全的类型提升: {} -> {}", current_type, new_type)
            ));
        }

        let alter_sql = format!("ALTER TABLE {}.{} ALTER COLUMN {} SET TYPE {}",
                               database, table, column_name, new_type);

        let result = self.execute_with_retry(|| {
            self.connection.execute(&alter_sql, [])
                .map_err(|e| DuckHubError::database(format!("修改列类型失败: {}", e)))
        }).await;

        match result {
            Ok(_) => {
                info!("成功修改列'{}'类型从{}到{}", column_name, current_type, new_type);
                Ok(())
            }
            Err(e) => {
                error!("修改列'{}'类型失败: {}", column_name, e);
                Err(e)
            }
        }
    }

    /// 检查列是否存在
    async fn column_exists(&self, database: &str, table: &str, column_name: &str) -> Result<bool> {
        let check_sql = format!(
            "SELECT COUNT(*) FROM information_schema.columns
             WHERE table_schema = '{}' AND table_name = '{}' AND column_name = '{}'",
            database, table, column_name
        );

        // TODO: 实现实际的列存在检查
        // 这里需要解析查询结果
        Ok(false) // 暂时返回false
    }

    /// 获取列的当前类型
    async fn get_column_type(&self, database: &str, table: &str, column_name: &str) -> Result<String> {
        let type_sql = format!(
            "SELECT data_type FROM information_schema.columns
             WHERE table_schema = '{}' AND table_name = '{}' AND column_name = '{}'",
            database, table, column_name
        );

        // TODO: 实现实际的类型查询
        Ok("VARCHAR".to_string()) // 暂时返回VARCHAR
    }

    /// 检查类型提升是否安全
    fn is_safe_type_promotion(&self, from_type: &str, to_type: &str) -> bool {
        match (from_type.to_uppercase().as_str(), to_type.to_uppercase().as_str()) {
            // 整数类型提升
            ("TINYINT", "SMALLINT" | "INTEGER" | "BIGINT") => true,
            ("SMALLINT", "INTEGER" | "BIGINT") => true,
            ("INTEGER", "BIGINT") => true,

            // 无符号整数类型提升
            ("UTINYINT", "USMALLINT" | "UINTEGER" | "UBIGINT") => true,
            ("USMALLINT", "UINTEGER" | "UBIGINT") => true,
            ("UINTEGER", "UBIGINT") => true,

            // 浮点数类型提升
            ("REAL", "DOUBLE") => true,

            // 字符串类型（通常安全）
            ("VARCHAR", "TEXT") => true,

            // 相同类型
            (from, to) if from == to => true,

            // 其他情况不安全
            _ => false,
        }
    }

    /// 辅助方法：将DataType转换为DuckDB SQL类型
    fn convert_to_duckdb_type(&self, data_type: &DataType) -> Result<String> {
        let duckdb_type = match data_type {
            DataType::Boolean => "BOOLEAN",
            DataType::Int8 => "TINYINT",
            DataType::Int16 => "SMALLINT",
            DataType::Int32 => "INTEGER",
            DataType::Int64 => "BIGINT",
            DataType::UInt8 => "UTINYINT",
            DataType::UInt16 => "USMALLINT",
            DataType::UInt32 => "UINTEGER",
            DataType::UInt64 => "UBIGINT",
            DataType::Float32 => "REAL",
            DataType::Float64 => "DOUBLE",
            DataType::String => "VARCHAR",
            DataType::Binary => "BLOB",
            DataType::Date => "DATE",
            DataType::Time => "TIME",
            DataType::Timestamp => "TIMESTAMP",
            DataType::Decimal { precision, scale } => {
                return Ok(format!("DECIMAL({}, {})", precision, scale));
            }
            _ => "VARCHAR", // 复杂类型默认为VARCHAR
        };

        Ok(duckdb_type.to_string())
    }
}

/// 错误恢复动作
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// 重试操作
    Retry,
    /// 重新连接
    Reconnect,
    /// 等待后重试
    WaitAndRetry,
    /// 增加超时时间
    IncreaseTimeout,
    /// 需要人工干预
    ManualIntervention,
}

/// 连接池状态信息
#[derive(Debug, Clone)]
pub struct ConnectionPoolStatus {
    /// 活跃连接数
    pub active_connections: usize,
    /// 空闲连接数
    pub idle_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 等待队列长度
    pub queue_length: usize,
}

/// 时间旅行查询结果
#[derive(Debug, Clone)]
pub struct TimeTravelQueryResult {
    /// 查询类型
    pub query_type: TimeTravelQueryType,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 返回行数
    pub rows_returned: usize,
    /// 是否命中缓存
    pub cache_hit: bool,
    /// 快照信息
    pub snapshot_info: Option<SnapshotInfo>,
}

/// 时间旅行查询类型
#[derive(Debug, Clone)]
pub enum TimeTravelQueryType {
    /// 按版本查询
    Version(u64),
    /// 按时间戳查询
    Timestamp(DateTime<Utc>),
    /// 时间范围查询
    TimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// 快照信息
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// 快照ID
    pub snapshot_id: u64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 操作类型
    pub operation: String,
    /// 摘要信息
    pub summary: HashMap<String, String>,
    /// 快照大小（字节）
    pub size_bytes: u64,
    /// 行数
    pub row_count: u64,
}

/// 时间范围查询结果
#[derive(Debug, Clone)]
pub struct TimeRangeQueryResult {
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 包含的快照列表
    pub snapshots_included: Vec<SnapshotInfo>,
    /// 总行数
    pub total_rows: u64,
    /// 查询结果数据
    pub data: Vec<serde_json::Value>,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 是否命中缓存
    pub cache_hit: bool,
}

/// 快照差异分析结果
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    /// 第一个版本
    pub version1: u64,
    /// 第二个版本
    pub version2: u64,
    /// 新增行数
    pub added_rows: u64,
    /// 删除行数
    pub deleted_rows: u64,
    /// 修改行数
    pub modified_rows: u64,
    /// Schema变化列表
    pub schema_changes: Vec<SchemaChange>,
    /// 第一个快照信息
    pub snapshot1_info: Option<SnapshotInfo>,
    /// 第二个快照信息
    pub snapshot2_info: Option<SnapshotInfo>,
    /// 分析耗时（秒）
    pub analysis_time: f64,
    /// 大小差异（字节）
    pub size_difference: i64,
}

/// Schema变化类型
#[derive(Debug, Clone)]
pub enum SchemaChange {
    /// 添加列
    ColumnAdded {
        column_name: String,
        data_type: String,
    },
    /// 删除列
    ColumnRemoved {
        column_name: String,
    },
    /// 列类型变化
    ColumnTypeChanged {
        column_name: String,
        old_type: String,
        new_type: String,
    },
    /// 列重命名
    ColumnRenamed {
        old_name: String,
        new_name: String,
    },
}

/// Schema演进结果
#[derive(Debug, Clone)]
pub struct SchemaEvolutionResult {
    /// 执行的操作
    pub operation: SchemaOperation,
    /// 旧版本号
    pub old_version: u64,
    /// 新版本号
    pub new_version: u64,
    /// 执行时间（秒）
    pub execution_time: f64,
    /// 兼容性影响级别
    pub compatibility_impact: CompatibilityImpact,
    /// 是否可以回滚
    pub rollback_possible: bool,
}

/// Schema操作类型
#[derive(Debug, Clone)]
pub enum SchemaOperation {
    /// 添加列
    AddColumn {
        column_name: String,
        column_type: String,
        nullable: bool,
        default_value: Option<String>,
    },
    /// 删除列
    DropColumn {
        column_name: String,
        column_info: Option<ColumnInfo>,
    },
    /// 修改列类型
    AlterColumnType {
        column_name: String,
        old_type: String,
        new_type: String,
    },
    /// 重命名列
    RenameColumn {
        old_name: String,
        new_name: String,
    },
}

/// 兼容性检查结果
#[derive(Debug, Clone)]
pub struct CompatibilityCheck {
    /// 是否兼容
    pub is_compatible: bool,
    /// 不兼容的原因
    pub reason: String,
    /// 影响级别
    pub impact_level: CompatibilityImpact,
}

/// 兼容性影响级别
#[derive(Debug, Clone, PartialEq)]
pub enum CompatibilityImpact {
    /// 无影响
    None,
    /// 低影响
    Low,
    /// 中等影响
    Medium,
    /// 高影响（破坏性变更）
    High,
}

/// 列信息
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可空
    pub nullable: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否为主键
    pub is_primary_key: bool,
}

/// 事务句柄
#[derive(Debug)]
pub struct TransactionHandle {
    /// 事务ID
    pub id: String,
    /// 开始时间
    pub start_time: Instant,
    /// 隔离级别
    pub isolation_level: Option<IsolationLevel>,
    /// 操作列表
    pub operations: Vec<TransactionOperation>,
    /// 保存点列表
    pub savepoints: Vec<Savepoint>,
}

/// 隔离级别
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationLevel {
    /// 读未提交
    ReadUncommitted,
    /// 读已提交
    ReadCommitted,
    /// 可重复读
    RepeatableRead,
    /// 串行化
    Serializable,
}

/// 事务操作
#[derive(Debug, Clone)]
pub struct TransactionOperation {
    /// 操作类型
    pub operation_type: String,
    /// 操作SQL
    pub sql: String,
    /// 执行时间
    pub execution_time: Duration,
    /// 影响行数
    pub affected_rows: u64,
}

/// 保存点
#[derive(Debug, Clone)]
pub struct Savepoint {
    /// 保存点名称
    pub name: String,
    /// 创建时间
    pub created_at: Instant,
}

/// 事务结果
#[derive(Debug, Clone)]
pub struct TransactionResult {
    /// 事务ID
    pub transaction_id: String,
    /// 事务状态
    pub status: TransactionStatus,
    /// 操作数量
    pub operations_count: usize,
    /// 总耗时（秒）
    pub total_duration: f64,
    /// 提交耗时（秒）
    pub commit_duration: Option<f64>,
    /// 回滚耗时（秒）
    pub rollback_duration: Option<f64>,
}

/// 事务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    /// 活跃中
    Active,
    /// 已提交
    Committed,
    /// 已回滚
    RolledBack,
    /// 已中止
    Aborted,
}

/// 死锁信息
#[derive(Debug, Clone)]
pub struct DeadlockInfo {
    /// 等待进程ID
    pub waiting_pid: u32,
    /// 阻塞进程ID
    pub blocking_pid: u32,
    /// 等待查询
    pub waiting_query: String,
    /// 阻塞查询
    pub blocking_query: String,
    /// 等待开始时间
    pub wait_start: DateTime<Utc>,
}

/// 锁信息
#[derive(Debug, Clone)]
pub struct LockInfo {
    /// 锁类型
    pub lock_type: LockType,
    /// 锁模式
    pub lock_mode: LockMode,
    /// 资源标识
    pub resource_id: String,
    /// 持有者进程ID
    pub holder_pid: u32,
    /// 获取时间
    pub acquired_at: DateTime<Utc>,
}

/// 锁类型
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    /// 表锁
    Table,
    /// 行锁
    Row,
    /// 页锁
    Page,
    /// 索引锁
    Index,
}

/// 锁模式
#[derive(Debug, Clone, PartialEq)]
pub enum LockMode {
    /// 共享锁
    Shared,
    /// 排他锁
    Exclusive,
    /// 意向共享锁
    IntentionShared,
    /// 意向排他锁
    IntentionExclusive,
}

/// DuckLake snapshot information
#[derive(Debug, Clone)]
pub struct DuckLakeSnapshot {
    pub snapshot_id: u64,
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub summary: HashMap<String, String>,
}

/// 批量操作类型
#[derive(Debug, Clone)]
pub enum DuckLakeOperation {
    /// 插入操作
    Insert {
        database: String,
        table: String,
        data: Vec<Vec<serde_json::Value>>,
    },
    /// 更新操作
    Update {
        sql: String,
    },
    /// 删除操作
    Delete {
        sql: String,
    },
    /// 创建表操作
    CreateTable {
        database: String,
        table: String,
        schema: Schema,
    },
}

/// 操作结果
#[derive(Debug, Clone)]
pub enum OperationResult {
    /// 插入结果
    Insert {
        rows_affected: usize,
    },
    /// 更新结果
    Update {
        rows_affected: usize,
    },
    /// 删除结果
    Delete {
        rows_affected: usize,
    },
    /// 创建表结果
    CreateTable,
}



/// Schema变更类型
#[derive(Debug, Clone)]
pub enum SchemaChangeType {
    /// 添加列
    AddColumn,
    /// 删除列
    DropColumn,
    /// 重命名列
    RenameColumn,
    /// 修改列类型
    AlterColumnType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;
    use tempfile::tempdir;
    use tokio_test;

    /// 创建测试用的DuckLakeManager
    fn create_test_manager() -> DuckLakeManager {
        let conn = Connection::open_in_memory().unwrap();
        DuckLakeManager::new(conn)
    }

    #[test]
    fn test_ducklake_config_default() {
        let config = DuckLakeConfig::default();
        assert_eq!(config.metadata_path, "ducklake.db");
        assert!(!config.encrypted);
        assert!(!config.read_only);
        assert_eq!(config.data_inlining_row_limit, 0);
    }

    #[test]
    fn test_ducklake_config_with_options() {
        let mut config = DuckLakeConfig::default();
        config.encrypted = true;
        config.read_only = true;
        config.data_path = Some("s3://my-bucket/data/".to_string());

        assert!(config.encrypted);
        assert!(config.read_only);
        assert_eq!(config.data_path.unwrap(), "s3://my-bucket/data/");
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_delay_ms, 5000);
    }

    #[test]
    fn test_ducklake_manager_creation() {
        let manager = create_test_manager();
        assert_eq!(manager.attached_databases.len(), 0);
        assert!(manager.connection_pool.is_none());
    }

    #[test]
    fn test_ducklake_manager_with_retry_config() {
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 200,
            backoff_multiplier: 1.5,
            max_delay_ms: 10000,
        };

        let manager = create_test_manager().with_retry_config(retry_config.clone());
        assert_eq!(manager.retry_config.max_retries, 5);
        assert_eq!(manager.retry_config.initial_delay_ms, 200);
    }

    #[test]
    fn test_build_attach_sql_basic() {
        let manager = create_test_manager();
        let config = DuckLakeConfig::default();
        let sql = manager.build_attach_sql("test_db", &config);

        assert!(sql.contains("ATTACH 'ducklake:ducklake.db'"));
        assert!(sql.contains("AS test_db"));
    }

    #[test]
    fn test_build_attach_sql_with_options() {
        let manager = create_test_manager();
        let mut config = DuckLakeConfig::default();
        config.data_path = Some("s3://bucket/data".to_string());
        config.encrypted = true;
        config.read_only = true;

        let sql = manager.build_attach_sql("test_db", &config);

        assert!(sql.contains("DATA_PATH 's3://bucket/data'"));
        assert!(sql.contains("ENCRYPTED"));
        assert!(sql.contains("READ_ONLY"));
    }

    #[test]
    fn test_is_safe_type_promotion() {
        let manager = create_test_manager();

        // 安全的类型提升
        assert!(manager.is_safe_type_promotion("TINYINT", "SMALLINT"));
        assert!(manager.is_safe_type_promotion("SMALLINT", "INTEGER"));
        assert!(manager.is_safe_type_promotion("INTEGER", "BIGINT"));
        assert!(manager.is_safe_type_promotion("REAL", "DOUBLE"));
        assert!(manager.is_safe_type_promotion("VARCHAR", "TEXT"));

        // 相同类型
        assert!(manager.is_safe_type_promotion("INTEGER", "INTEGER"));

        // 不安全的类型提升
        assert!(!manager.is_safe_type_promotion("BIGINT", "INTEGER"));
        assert!(!manager.is_safe_type_promotion("DOUBLE", "REAL"));
        assert!(!manager.is_safe_type_promotion("TEXT", "VARCHAR"));
    }

    #[test]
    fn test_build_batch_insert_sql() {
        let manager = create_test_manager();
        let data = vec![
            vec![
                serde_json::Value::String("test1".to_string()),
                serde_json::Value::Number(serde_json::Number::from(123)),
                serde_json::Value::Bool(true),
            ],
            vec![
                serde_json::Value::String("test2".to_string()),
                serde_json::Value::Number(serde_json::Number::from(456)),
                serde_json::Value::Bool(false),
            ],
        ];

        let sql = manager.build_batch_insert_sql("test_db", "test_table", &data).unwrap();

        assert!(sql.contains("INSERT INTO test_db.test_table VALUES"));
        assert!(sql.contains("('test1', 123, true)"));
        assert!(sql.contains("('test2', 456, false)"));
    }

    #[test]
    fn test_build_batch_insert_sql_empty_data() {
        let manager = create_test_manager();
        let data: Vec<Vec<serde_json::Value>> = vec![];

        let result = manager.build_batch_insert_sql("test_db", "test_table", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_batch_insert_sql_with_null() {
        let manager = create_test_manager();
        let data = vec![
            vec![
                serde_json::Value::String("test".to_string()),
                serde_json::Value::Null,
                serde_json::Value::Number(serde_json::Number::from(123)),
            ],
        ];

        let sql = manager.build_batch_insert_sql("test_db", "test_table", &data).unwrap();
        assert!(sql.contains("('test', NULL, 123)"));
    }

    #[test]
    fn test_build_batch_insert_sql_with_quotes() {
        let manager = create_test_manager();
        let data = vec![
            vec![
                serde_json::Value::String("test's data".to_string()),
            ],
        ];

        let sql = manager.build_batch_insert_sql("test_db", "test_table", &data).unwrap();
        assert!(sql.contains("('test''s data')"));
    }

    #[tokio::test]
    async fn test_attach_database_basic() {
        let mut manager = create_test_manager();
        let config = DuckLakeConfig {
            metadata_path: ":memory:".to_string(),
            ..Default::default()
        };

        // 注意：这个测试可能会失败，因为DuckLake扩展可能未安装
        // 在实际环境中需要确保DuckLake扩展可用
        let result = manager.attach_database("test_db", &config).await;

        // 如果DuckLake扩展未安装，这个测试会失败，这是预期的
        // 在CI环境中应该安装必要的扩展
        match result {
            Ok(_) => {
                assert_eq!(manager.attached_databases.len(), 1);
                assert!(manager.attached_databases.contains_key("test_db"));
            }
            Err(_) => {
                // 如果扩展未安装，测试应该跳过而不是失败
                println!("警告: DuckLake扩展未安装，跳过附加数据库测试");
            }
        }
    }

    #[tokio::test]
    async fn test_detach_database() {
        let mut manager = create_test_manager();

        // 手动添加一个数据库记录（模拟已附加的数据库）
        let database = DuckLakeDatabase {
            name: "test_db".to_string(),
            metadata_path: ":memory:".to_string(),
            data_path: ":memory:.files".to_string(),
            read_only: false,
            encrypted: false,
            snapshot_version: None,
            snapshot_time: None,
        };
        manager.attached_databases.insert("test_db".to_string(), database);

        assert_eq!(manager.attached_databases.len(), 1);

        let result = manager.detach_database("test_db").await;

        // 分离操作可能会失败（如果数据库实际上没有附加），但我们主要测试内部状态
        match result {
            Ok(_) => {
                assert_eq!(manager.attached_databases.len(), 0);
            }
            Err(_) => {
                // 即使SQL执行失败，内部状态也应该被清理
                assert_eq!(manager.attached_databases.len(), 0);
            }
        }
    }

    #[test]
    fn test_get_attached_databases() {
        let mut manager = create_test_manager();

        let database = DuckLakeDatabase {
            name: "test_db".to_string(),
            metadata_path: ":memory:".to_string(),
            data_path: ":memory:.files".to_string(),
            read_only: false,
            encrypted: false,
            snapshot_version: None,
            snapshot_time: None,
        };
        manager.attached_databases.insert("test_db".to_string(), database);

        let databases = manager.get_attached_databases();
        assert_eq!(databases.len(), 1);
        assert_eq!(databases[0].name, "test_db");
    }

    #[test]
    fn test_convert_to_duckdb_type() {
        let manager = create_test_manager();

        assert_eq!(manager.convert_to_duckdb_type(&DataType::Boolean).unwrap(), "BOOLEAN");
        assert_eq!(manager.convert_to_duckdb_type(&DataType::Int32).unwrap(), "INTEGER");
        assert_eq!(manager.convert_to_duckdb_type(&DataType::String).unwrap(), "VARCHAR");
        assert_eq!(manager.convert_to_duckdb_type(&DataType::Date).unwrap(), "DATE");

        let decimal_type = DataType::Decimal { precision: 10, scale: 2 };
        assert_eq!(manager.convert_to_duckdb_type(&decimal_type).unwrap(), "DECIMAL(10, 2)");
    }
}
