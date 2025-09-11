//! 数据连接器模块 - 支持多种数据库和存储系统

use crate::config::*;
use crate::sources::DataRecord;
use duckhub_common::prelude::*;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 数据连接器特征
#[async_trait]
pub trait DataConnector {
    /// 连接器名称
    fn name(&self) -> &str;

    /// 连接器类型
    fn connector_type(&self) -> ConnectorType;

    /// 建立连接
    async fn connect(&mut self) -> Result<()>;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<()>;

    /// 写入数据
    async fn write_data(&self, records: Vec<DataRecord>) -> Result<WriteResult>;

    /// 读取数据
    async fn read_data(&self, query: &str) -> Result<Vec<DataRecord>>;

    /// 检查连接状态
    async fn health_check(&self) -> Result<bool>;

    /// 获取连接器统计信息
    async fn get_stats(&self) -> Result<ConnectorStats>;
}

/// 连接器类型
#[derive(Debug, Clone)]
pub enum ConnectorType {
    /// 数据库连接器
    Database(DatabaseType),
    /// 文件系统连接器
    FileSystem,
    /// 对象存储连接器
    ObjectStorage(StorageType),
    /// 消息队列连接器
    MessageQueue(QueueType),
}

/// 数据库类型
#[derive(Debug, Clone)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    Oracle,
    SQLServer,
    MongoDB,
    Redis,
}

/// 存储类型
#[derive(Debug, Clone)]
pub enum StorageType {
    S3,
    Azure,
    GCS,
    MinIO,
}

/// 队列类型
#[derive(Debug, Clone)]
pub enum QueueType {
    Kafka,
    RabbitMQ,
    Redis,
    NATS,
}

/// 写入结果
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// 成功写入的记录数
    pub records_written: u64,
    /// 失败的记录数
    pub records_failed: u64,
    /// 写入耗时（毫秒）
    pub duration_ms: u64,
    /// 错误信息
    pub errors: Vec<String>,
}

/// 连接器统计信息
#[derive(Debug, Clone)]
pub struct ConnectorStats {
    /// 连接状态
    pub connection_status: ConnectionStatus,
    /// 写入的记录总数
    pub total_records_written: u64,
    /// 读取的记录总数
    pub total_records_read: u64,
    /// 失败的操作数
    pub failed_operations: u64,
    /// 平均写入时间（毫秒）
    pub avg_write_time_ms: f64,
    /// 平均读取时间（毫秒）
    pub avg_read_time_ms: f64,
    /// 最后操作时间
    pub last_operation_at: Option<DateTime<Utc>>,
}

/// 连接状态
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

/// MySQL数据库连接器
pub struct MySQLConnector {
    name: String,
    config: ConnectionConfig,
    stats: ConnectorStats,
    is_connected: bool,
}

impl MySQLConnector {
    /// 创建新的MySQL连接器
    pub fn new(name: String, config: ConnectionConfig) -> Self {
        Self {
            name,
            config,
            stats: ConnectorStats {
                connection_status: ConnectionStatus::Disconnected,
                total_records_written: 0,
                total_records_read: 0,
                failed_operations: 0,
                avg_write_time_ms: 0.0,
                avg_read_time_ms: 0.0,
                last_operation_at: None,
            },
            is_connected: false,
        }
    }
}

#[async_trait]
impl DataConnector for MySQLConnector {
    fn name(&self) -> &str {
        &self.name
    }

    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Database(DatabaseType::MySQL)
    }

    async fn connect(&mut self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }

        // 实现真实的MySQL连接逻辑
        let connection_string = format!(
            "mysql://{}:{}@{}:{}/{}",
            self.config.username.as_ref().unwrap_or(&"root".to_string()),
            self.config.password.as_ref().unwrap_or(&"".to_string()),
            self.config.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.port.unwrap_or(3306),
            self.config.database.as_ref().unwrap_or(&"test".to_string())
        );

        // 使用mysql_async库建立连接
        match mysql_async::Pool::new(&connection_string) {
            Ok(pool) => {
                // 测试连接
                let mut conn = pool.get_conn().await
                    .map_err(|e| DuckHubError::database(format!("MySQL连接失败: {}", e)))?;

                // 执行简单查询验证连接
                let _: Vec<mysql_async::Row> = conn.query("SELECT 1").await
                    .map_err(|e| DuckHubError::database(format!("MySQL连接验证失败: {}", e)))?;

                self.is_connected = true;
                self.stats.connection_status = ConnectionStatus::Connected;
                self.stats.last_connected_at = Some(Utc::now());

                info!("MySQL连接器 '{}' 连接成功", self.name);
                Ok(())
            }
            Err(e) => {
                self.stats.connection_status = ConnectionStatus::Failed;
                Err(DuckHubError::database(format!("MySQL连接池创建失败: {}", e)))
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.is_connected = false;
        self.stats.connection_status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn write_data(&self, records: Vec<DataRecord>) -> Result<WriteResult> {
        if !self.is_connected {
            return Err(DuckHubError::database("MySQL连接未建立"));
        }

        let start_time = Utc::now();
        
        // TODO: 实现真实的数据写入逻辑
        // 这里是Mock实现
        let records_count = records.len() as u64;
        
        // 模拟写入延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;

        Ok(WriteResult {
            records_written: records_count,
            records_failed: 0,
            duration_ms,
            errors: Vec::new(),
        })
    }

    async fn read_data(&self, query: &str) -> Result<Vec<DataRecord>> {
        if !self.is_connected {
            return Err(DuckHubError::database("MySQL连接未建立"));
        }

        // 实现真实的MySQL数据读取逻辑
        let connection_string = format!(
            "mysql://{}:{}@{}:{}/{}",
            self.config.username.as_ref().unwrap_or(&"root".to_string()),
            self.config.password.as_ref().unwrap_or(&"".to_string()),
            self.config.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.port.unwrap_or(3306),
            self.config.database.as_ref().unwrap_or(&"test".to_string())
        );

        let pool = mysql_async::Pool::new(&connection_string)
            .map_err(|e| DuckHubError::database(format!("MySQL连接池创建失败: {}", e)))?;

        let mut conn = pool.get_conn().await
            .map_err(|e| DuckHubError::database(format!("获取MySQL连接失败: {}", e)))?;

        // 执行查询
        let rows: Vec<mysql_async::Row> = conn.query(query).await
            .map_err(|e| DuckHubError::database(format!("MySQL查询执行失败: {}", e)))?;

        let mut records = Vec::new();

        for row in rows {
            // 将MySQL行转换为JSON对象
            let mut json_obj = serde_json::Map::new();

            // 获取列信息
            let columns = row.columns_ref();
            for (i, column) in columns.iter().enumerate() {
                let column_name = column.name_str();
                let value = match row.get_opt::<mysql_async::Value, usize>(i) {
                    Some(Ok(mysql_async::Value::NULL)) => serde_json::Value::Null,
                    Some(Ok(mysql_async::Value::Bytes(bytes))) => {
                        serde_json::Value::String(String::from_utf8_lossy(&bytes).to_string())
                    },
                    Some(Ok(mysql_async::Value::Int(i))) => serde_json::Value::Number(serde_json::Number::from(i)),
                    Some(Ok(mysql_async::Value::UInt(u))) => serde_json::Value::Number(serde_json::Number::from(u)),
                    Some(Ok(mysql_async::Value::Float(f))) => {
                        serde_json::Value::Number(serde_json::Number::from_f64(f as f64).unwrap_or(serde_json::Number::from(0)))
                    },
                    Some(Ok(mysql_async::Value::Double(d))) => {
                        serde_json::Value::Number(serde_json::Number::from_f64(d).unwrap_or(serde_json::Number::from(0)))
                    },
                    Some(Ok(mysql_async::Value::Date(year, month, day, hour, minute, second, _))) => {
                        let datetime = format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second);
                        serde_json::Value::String(datetime)
                    },
                    _ => serde_json::Value::Null,
                };

                json_obj.insert(column_name.to_string(), value);
            }

            let record = DataRecord::new(
                self.name.clone(),
                serde_json::Value::Object(json_obj)
            );
            records.push(record);
        }

        // 更新统计信息
        self.stats.records_processed += records.len() as u64;
        self.stats.last_processed_at = Some(Utc::now());

        info!("MySQL连接器 '{}' 成功读取 {} 条记录", self.name, records.len());
        Ok(records)
    }

    async fn health_check(&self) -> Result<bool> {
        // TODO: 实现真实的健康检查
        Ok(self.is_connected)
    }

    async fn get_stats(&self) -> Result<ConnectorStats> {
        Ok(self.stats.clone())
    }
}

/// PostgreSQL数据库连接器
pub struct PostgreSQLConnector {
    name: String,
    config: ConnectionConfig,
    stats: ConnectorStats,
    is_connected: bool,
}

impl PostgreSQLConnector {
    /// 创建新的PostgreSQL连接器
    pub fn new(name: String, config: ConnectionConfig) -> Self {
        Self {
            name,
            config,
            stats: ConnectorStats {
                connection_status: ConnectionStatus::Disconnected,
                total_records_written: 0,
                total_records_read: 0,
                failed_operations: 0,
                avg_write_time_ms: 0.0,
                avg_read_time_ms: 0.0,
                last_operation_at: None,
            },
            is_connected: false,
        }
    }
}

#[async_trait]
impl DataConnector for PostgreSQLConnector {
    fn name(&self) -> &str {
        &self.name
    }

    fn connector_type(&self) -> ConnectorType {
        ConnectorType::Database(DatabaseType::PostgreSQL)
    }

    async fn connect(&mut self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }

        // 实现真实的PostgreSQL连接逻辑
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.config.username.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.password.as_ref().unwrap_or(&"".to_string()),
            self.config.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.port.unwrap_or(5432),
            self.config.database.as_ref().unwrap_or(&"postgres".to_string())
        );

        // 使用tokio-postgres建立连接
        match tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await {
            Ok((client, connection)) => {
                // 在后台运行连接
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("PostgreSQL连接错误: {}", e);
                    }
                });

                // 测试连接
                match client.query("SELECT 1", &[]).await {
                    Ok(_) => {
                        self.is_connected = true;
                        self.stats.connection_status = ConnectionStatus::Connected;
                        self.stats.last_connected_at = Some(Utc::now());

                        info!("PostgreSQL连接器 '{}' 连接成功", self.name);
                        Ok(())
                    }
                    Err(e) => {
                        self.stats.connection_status = ConnectionStatus::Failed;
                        Err(DuckHubError::database(format!("PostgreSQL连接验证失败: {}", e)))
                    }
                }
            }
            Err(e) => {
                self.stats.connection_status = ConnectionStatus::Failed;
                Err(DuckHubError::database(format!("PostgreSQL连接失败: {}", e)))
            }
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.is_connected = false;
        self.stats.connection_status = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn write_data(&self, records: Vec<DataRecord>) -> Result<WriteResult> {
        if !self.is_connected {
            return Err(DuckHubError::database("PostgreSQL连接未建立"));
        }

        let start_time = Utc::now();
        let records_count = records.len() as u64;
        
        // 模拟写入延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        
        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;

        Ok(WriteResult {
            records_written: records_count,
            records_failed: 0,
            duration_ms,
            errors: Vec::new(),
        })
    }

    async fn read_data(&self, query: &str) -> Result<Vec<DataRecord>> {
        if !self.is_connected {
            return Err(DuckHubError::database("PostgreSQL连接未建立"));
        }

        // 实现真实的PostgreSQL数据读取逻辑
        let connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.config.username.as_ref().unwrap_or(&"postgres".to_string()),
            self.config.password.as_ref().unwrap_or(&"".to_string()),
            self.config.host.as_ref().unwrap_or(&"localhost".to_string()),
            self.config.port.unwrap_or(5432),
            self.config.database.as_ref().unwrap_or(&"postgres".to_string())
        );

        let (client, connection) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls).await
            .map_err(|e| DuckHubError::database(format!("PostgreSQL连接失败: {}", e)))?;

        // 在后台运行连接
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL连接错误: {}", e);
            }
        });

        // 执行查询
        let rows = client.query(query, &[]).await
            .map_err(|e| DuckHubError::database(format!("PostgreSQL查询执行失败: {}", e)))?;

        let mut records = Vec::new();

        for row in rows {
            let mut json_obj = serde_json::Map::new();

            // 遍历所有列
            for (i, column) in row.columns().iter().enumerate() {
                let column_name = column.name();

                // 根据PostgreSQL类型转换为JSON值
                let value = match column.type_() {
                    &tokio_postgres::types::Type::BOOL => {
                        row.try_get::<_, Option<bool>>(i)
                            .unwrap_or(None)
                            .map(serde_json::Value::Bool)
                            .unwrap_or(serde_json::Value::Null)
                    },
                    &tokio_postgres::types::Type::INT2 | &tokio_postgres::types::Type::INT4 => {
                        row.try_get::<_, Option<i32>>(i)
                            .unwrap_or(None)
                            .map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
                            .unwrap_or(serde_json::Value::Null)
                    },
                    &tokio_postgres::types::Type::INT8 => {
                        row.try_get::<_, Option<i64>>(i)
                            .unwrap_or(None)
                            .map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
                            .unwrap_or(serde_json::Value::Null)
                    },
                    &tokio_postgres::types::Type::FLOAT4 | &tokio_postgres::types::Type::FLOAT8 => {
                        row.try_get::<_, Option<f64>>(i)
                            .unwrap_or(None)
                            .and_then(|v| serde_json::Number::from_f64(v))
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    },
                    &tokio_postgres::types::Type::TEXT | &tokio_postgres::types::Type::VARCHAR => {
                        row.try_get::<_, Option<String>>(i)
                            .unwrap_or(None)
                            .map(serde_json::Value::String)
                            .unwrap_or(serde_json::Value::Null)
                    },
                    &tokio_postgres::types::Type::TIMESTAMP | &tokio_postgres::types::Type::TIMESTAMPTZ => {
                        row.try_get::<_, Option<chrono::NaiveDateTime>>(i)
                            .unwrap_or(None)
                            .map(|dt| serde_json::Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string()))
                            .unwrap_or(serde_json::Value::Null)
                    },
                    _ => {
                        // 对于其他类型，尝试转换为字符串
                        row.try_get::<_, Option<String>>(i)
                            .unwrap_or(None)
                            .map(serde_json::Value::String)
                            .unwrap_or(serde_json::Value::Null)
                    }
                };

                json_obj.insert(column_name.to_string(), value);
            }

            let record = DataRecord::new(
                self.name.clone(),
                serde_json::Value::Object(json_obj)
            );
            records.push(record);
        }

        // 更新统计信息
        self.stats.records_processed += records.len() as u64;
        self.stats.last_processed_at = Some(Utc::now());

        info!("PostgreSQL连接器 '{}' 成功读取 {} 条记录", self.name, records.len());
        Ok(records)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_connected)
    }

    async fn get_stats(&self) -> Result<ConnectorStats> {
        Ok(self.stats.clone())
    }
}

/// 文件系统连接器
pub struct FileSystemConnector {
    name: String,
    config: ConnectionConfig,
    stats: ConnectorStats,
    base_path: String,
}

impl FileSystemConnector {
    /// 创建新的文件系统连接器
    pub fn new(name: String, config: ConnectionConfig) -> Self {
        let base_path = config.parameters.get("base_path")
            .unwrap_or(&"/tmp/duckhub".to_string())
            .clone();

        Self {
            name,
            config,
            stats: ConnectorStats {
                connection_status: ConnectionStatus::Connected, // 文件系统总是可用
                total_records_written: 0,
                total_records_read: 0,
                failed_operations: 0,
                avg_write_time_ms: 0.0,
                avg_read_time_ms: 0.0,
                last_operation_at: None,
            },
            base_path,
        }
    }
}

#[async_trait]
impl DataConnector for FileSystemConnector {
    fn name(&self) -> &str {
        &self.name
    }

    fn connector_type(&self) -> ConnectorType {
        ConnectorType::FileSystem
    }

    async fn connect(&mut self) -> Result<()> {
        // 确保基础目录存在
        tokio::fs::create_dir_all(&self.base_path).await
            .map_err(|e| DuckHubError::io(e))?;
        
        self.stats.connection_status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        // 文件系统连接器不需要断开连接
        Ok(())
    }

    async fn write_data(&self, records: Vec<DataRecord>) -> Result<WriteResult> {
        let start_time = Utc::now();
        let mut records_written = 0;
        let mut errors = Vec::new();

        for record in records {
            let file_path = format!("{}/{}.json", self.base_path, record.id);
            let content = serde_json::to_string_pretty(&record)
                .map_err(|e| DuckHubError::validation(e.to_string()))?;

            match tokio::fs::write(&file_path, content).await {
                Ok(_) => records_written += 1,
                Err(e) => errors.push(format!("写入文件{}失败: {}", file_path, e)),
            }
        }

        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;

        Ok(WriteResult {
            records_written,
            records_failed: errors.len() as u64,
            duration_ms,
            errors,
        })
    }

    async fn read_data(&self, pattern: &str) -> Result<Vec<DataRecord>> {
        // 实现真实的文件模式匹配读取
        let mut records = Vec::new();

        // 构建完整路径
        let full_pattern = if pattern.starts_with('/') {
            pattern.to_string()
        } else {
            format!("{}/{}", self.base_path, pattern)
        };

        // 使用glob模式匹配文件
        let paths = glob::glob(&full_pattern)
            .map_err(|e| DuckHubError::validation(format!("无效的文件模式: {}", e)))?;

        for path_result in paths {
            match path_result {
                Ok(path) => {
                    if path.is_file() {
                        match self.read_file(&path).await {
                            Ok(file_records) => records.extend(file_records),
                            Err(e) => {
                                warn!("读取文件 {:?} 失败: {}", path, e);
                                continue;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("文件路径错误: {}", e);
                    continue;
                }
            }
        }

        // 更新统计信息
        self.stats.records_processed += records.len() as u64;
        self.stats.last_processed_at = Some(Utc::now());

        info!("文件系统连接器 '{}' 成功读取 {} 条记录", self.name, records.len());
        Ok(records)
    }

    async fn health_check(&self) -> Result<bool> {
        // 检查基础目录是否可访问
        match tokio::fs::metadata(&self.base_path).await {
            Ok(metadata) => Ok(metadata.is_dir()),
            Err(_) => Ok(false),
        }
    }

    async fn get_stats(&self) -> Result<ConnectorStats> {
        Ok(self.stats.clone())
    }
}

impl FileSystemConnector {
    /// 读取单个文件
    async fn read_file(&self, path: &std::path::Path) -> Result<Vec<DataRecord>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "csv" => self.read_csv_file(path).await,
            "json" => self.read_json_file(path).await,
            "jsonl" | "ndjson" => self.read_jsonl_file(path).await,
            _ => self.read_text_file(path).await,
        }
    }

    /// 读取CSV文件
    async fn read_csv_file(&self, path: &std::path::Path) -> Result<Vec<DataRecord>> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| DuckHubError::internal(format!("读取CSV文件失败: {}", e)))?;

        let mut reader = csv::Reader::from_reader(content.as_bytes());
        let headers = reader.headers()
            .map_err(|e| DuckHubError::validation(format!("CSV头部解析失败: {}", e)))?
            .clone();

        let mut records = Vec::new();

        for result in reader.records() {
            match result {
                Ok(record) => {
                    let mut json_obj = serde_json::Map::new();

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            // 尝试解析为数字，否则作为字符串
                            let value = if let Ok(num) = field.parse::<f64>() {
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(num).unwrap_or(serde_json::Number::from(0))
                                )
                            } else if field.is_empty() {
                                serde_json::Value::Null
                            } else {
                                serde_json::Value::String(field.to_string())
                            };

                            json_obj.insert(header.to_string(), value);
                        }
                    }

                    // 添加文件元数据
                    json_obj.insert("_file_path".to_string(),
                        serde_json::Value::String(path.to_string_lossy().to_string()));

                    let data_record = DataRecord::new(
                        self.name.clone(),
                        serde_json::Value::Object(json_obj)
                    );
                    records.push(data_record);
                }
                Err(e) => {
                    warn!("CSV记录解析失败: {}", e);
                    continue;
                }
            }
        }

        Ok(records)
    }

    /// 读取JSON文件
    async fn read_json_file(&self, path: &std::path::Path) -> Result<Vec<DataRecord>> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| DuckHubError::internal(format!("读取JSON文件失败: {}", e)))?;

        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| DuckHubError::validation(format!("JSON解析失败: {}", e)))?;

        let mut records = Vec::new();

        match json_value {
            serde_json::Value::Array(array) => {
                for item in array {
                    let mut obj = match item {
                        serde_json::Value::Object(obj) => obj,
                        other => {
                            let mut new_obj = serde_json::Map::new();
                            new_obj.insert("value".to_string(), other);
                            new_obj
                        }
                    };

                    // 添加文件元数据
                    obj.insert("_file_path".to_string(),
                        serde_json::Value::String(path.to_string_lossy().to_string()));

                    let record = DataRecord::new(
                        self.name.clone(),
                        serde_json::Value::Object(obj)
                    );
                    records.push(record);
                }
            }
            serde_json::Value::Object(mut obj) => {
                // 添加文件元数据
                obj.insert("_file_path".to_string(),
                    serde_json::Value::String(path.to_string_lossy().to_string()));

                let record = DataRecord::new(
                    self.name.clone(),
                    serde_json::Value::Object(obj)
                );
                records.push(record);
            }
            other => {
                let mut obj = serde_json::Map::new();
                obj.insert("value".to_string(), other);
                obj.insert("_file_path".to_string(),
                    serde_json::Value::String(path.to_string_lossy().to_string()));

                let record = DataRecord::new(
                    self.name.clone(),
                    serde_json::Value::Object(obj)
                );
                records.push(record);
            }
        }

        Ok(records)
    }

    /// 读取JSONL文件
    async fn read_jsonl_file(&self, path: &std::path::Path) -> Result<Vec<DataRecord>> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| DuckHubError::internal(format!("读取JSONL文件失败: {}", e)))?;

        let mut records = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json_value) => {
                    let mut obj = match json_value {
                        serde_json::Value::Object(obj) => obj,
                        other => {
                            let mut new_obj = serde_json::Map::new();
                            new_obj.insert("value".to_string(), other);
                            new_obj
                        }
                    };

                    // 添加文件元数据
                    obj.insert("_file_path".to_string(),
                        serde_json::Value::String(path.to_string_lossy().to_string()));
                    obj.insert("_line_number".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(line_num + 1)));

                    let record = DataRecord::new(
                        self.name.clone(),
                        serde_json::Value::Object(obj)
                    );
                    records.push(record);
                }
                Err(e) => {
                    warn!("JSONL第{}行解析失败: {}", line_num + 1, e);
                    continue;
                }
            }
        }

        Ok(records)
    }

    /// 读取文本文件
    async fn read_text_file(&self, path: &std::path::Path) -> Result<Vec<DataRecord>> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| DuckHubError::internal(format!("读取文本文件失败: {}", e)))?;

        let metadata = tokio::fs::metadata(path).await
            .map_err(|e| DuckHubError::internal(format!("获取文件元数据失败: {}", e)))?;

        let mut obj = serde_json::Map::new();
        obj.insert("content".to_string(), serde_json::Value::String(content));
        obj.insert("_file_path".to_string(),
            serde_json::Value::String(path.to_string_lossy().to_string()));
        obj.insert("_file_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(metadata.len())));
        obj.insert("_modified_at".to_string(),
            serde_json::Value::String(
                metadata.modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string()
            ));

        let record = DataRecord::new(
            self.name.clone(),
            serde_json::Value::Object(obj)
        );

        Ok(vec![record])
    }
}
