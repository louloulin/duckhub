//! 数据采集服务配置模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 数据采集服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// 服务名称
    pub service_name: String,
    /// 监听端口
    pub port: u16,
    /// 工作线程数
    pub worker_threads: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 批处理超时时间（秒）
    pub batch_timeout_secs: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 数据源配置
    pub sources: HashMap<String, DataSourceConfig>,
    /// 处理器配置
    pub processors: HashMap<String, ProcessorConfig>,
    /// 监控配置
    pub monitoring: MonitoringConfig,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            service_name: "duckhub-data-ingestion".to_string(),
            port: 8080,
            worker_threads: 4,
            batch_size: 1000,
            batch_timeout_secs: 30,
            max_retries: 3,
            retry_interval_ms: 1000,
            sources: HashMap::new(),
            processors: HashMap::new(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

/// 数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// 数据源类型
    pub source_type: DataSourceType,
    /// 连接配置
    pub connection: ConnectionConfig,
    /// 是否启用
    pub enabled: bool,
    /// 轮询间隔（秒）
    pub poll_interval_secs: Option<u64>,
    /// 数据格式
    pub data_format: DataFormat,
    /// 质量检查配置
    pub quality_checks: QualityCheckConfig,
}

// 使用common模块中的DataSourceType
pub use duckhub_common::types::DataSourceType;

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// 连接URL或地址
    pub url: String,
    /// 认证信息
    pub auth: Option<AuthConfig>,
    /// 连接超时时间（秒）
    pub timeout_secs: u64,
    /// 最大连接数
    pub max_connections: u32,
    /// 其他配置参数
    pub parameters: HashMap<String, String>,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// 认证类型
    pub auth_type: AuthType,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// API密钥
    pub api_key: Option<String>,
    /// Token
    pub token: Option<String>,
}

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// 无认证
    None,
    /// 基础认证
    Basic,
    /// API密钥
    ApiKey,
    /// Bearer Token
    Bearer,
    /// OAuth2
    OAuth2,
}

/// 数据格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    /// JSON格式
    Json,
    /// CSV格式
    Csv,
    /// Parquet格式
    Parquet,
    /// Avro格式
    Avro,
    /// 原始文本
    Text,
}

/// 数据质量检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckConfig {
    /// 是否启用质量检查
    pub enabled: bool,
    /// 必填字段
    pub required_fields: Vec<String>,
    /// 数据类型验证
    pub type_validation: bool,
    /// 范围检查
    pub range_checks: HashMap<String, RangeCheck>,
    /// 自定义验证规则
    pub custom_rules: Vec<String>,
}

impl Default for QualityCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            required_fields: Vec::new(),
            type_validation: true,
            range_checks: HashMap::new(),
            custom_rules: Vec::new(),
        }
    }
}

/// 范围检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeCheck {
    /// 最小值
    pub min: Option<f64>,
    /// 最大值
    pub max: Option<f64>,
}

/// 处理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// 处理器类型
    pub processor_type: ProcessorType,
    /// 是否启用
    pub enabled: bool,
    /// 处理顺序
    pub order: u32,
    /// 配置参数
    pub parameters: HashMap<String, String>,
}

/// 处理器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorType {
    /// 数据清洗
    DataCleaning,
    /// 数据转换
    DataTransformation,
    /// 数据验证
    DataValidation,
    /// 数据丰富
    DataEnrichment,
    /// 数据过滤
    DataFiltering,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 是否启用监控
    pub enabled: bool,
    /// Prometheus指标端口
    pub metrics_port: u16,
    /// 健康检查端点
    pub health_check_path: String,
    /// 日志级别
    pub log_level: String,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_port: 9090,
            health_check_path: "/health".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl IngestionConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.service_name.is_empty() {
            return Err("服务名称不能为空".to_string());
        }

        if self.port == 0 {
            return Err("端口号必须大于0".to_string());
        }

        if self.worker_threads == 0 {
            return Err("工作线程数必须大于0".to_string());
        }

        if self.batch_size == 0 {
            return Err("批处理大小必须大于0".to_string());
        }

        // 验证数据源配置
        for (name, source_config) in &self.sources {
            if source_config.connection.url.is_empty() {
                return Err(format!("数据源'{}'的连接URL不能为空", name));
            }
        }

        Ok(())
    }

    /// 获取超时时间
    pub fn get_batch_timeout(&self) -> Duration {
        Duration::from_secs(self.batch_timeout_secs)
    }

    /// 获取重试间隔
    pub fn get_retry_interval(&self) -> Duration {
        Duration::from_millis(self.retry_interval_ms)
    }
}
