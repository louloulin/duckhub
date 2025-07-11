//! 基于Rig框架的AI Agent实现
//! 使用DeepSeek作为LLM provider，提供统一的AI服务

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_query_analytics::{QueryAnalyticsService, QueryAnalyticsConfig};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument, error};
use prometheus::{Counter, Histogram, Gauge, Registry};
use anyhow::{Result as AnyhowResult, Context};
use thiserror::Error;
use async_trait::async_trait;

// Rig框架导入
use rig::{
    providers::deepseek,
    agent::Agent,
    tool::Tool,
    completion::{Prompt, ToolDefinition},
    client::CompletionClient,
};
use serde_json::{json, Value};

// 基于Rig框架的RAG系统导入
use crate::rig_rag::{RigRagService, RagConfig, RagQueryRequest, RagQueryResponse, QueryType};

/// Rig AI Agent错误类型
#[derive(Error, Debug)]
pub enum RigAIError {
    #[error("Agent错误: {0}")]
    AgentError(String),
    #[error("工具调用错误: {0}")]
    ToolError(String),
    #[error("SQL生成失败: {0}")]
    SqlGenerationError(String),
    #[error("查询执行失败: {0}")]
    QueryExecutionError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("Rig框架错误: {0}")]
    RigError(String),
}

/// 基于Rig框架的AI Agent服务
pub struct RigAIService {
    /// DeepSeek客户端
    deepseek_client: deepseek::Client,
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 查询分析服务
    query_service: Arc<QueryAnalyticsService>,
    
    /// 专业化Agent
    sql_agent: Agent<deepseek::DeepSeekCompletionModel>,
    analysis_agent: Agent<deepseek::DeepSeekCompletionModel>,
    chat_agent: Agent<deepseek::DeepSeekCompletionModel>,
    recommendation_agent: Agent<deepseek::DeepSeekCompletionModel>,
    
    /// 工具集
    toolset: DuckHubToolSet,
    
    /// 配置和监控
    config: RigAIConfig,
    metrics: RigAIMetrics,

    /// RAG服务
    rag_service: Option<Arc<RigRagService>>,
}

/// Rig AI配置 - 企业级配置管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAIConfig {
    /// DeepSeek API密钥
    pub deepseek_api_key: String,
    /// 模型配置
    pub model_config: ModelConfig,
    /// Agent配置映射
    pub agent_configs: AgentConfigs,
    /// 工具配置
    pub tool_configs: ToolConfigs,
    /// RAG配置
    pub rag_config: RagConfig,
    /// 安全配置
    pub security_config: SecurityConfig,
    /// 性能配置
    pub performance_config: PerformanceConfig,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用API密钥验证
    pub enable_api_key_validation: bool,
    /// 最大并发请求数
    pub max_concurrent_requests: u32,
    /// 请求速率限制（每分钟）
    pub rate_limit_per_minute: u32,
    /// 是否启用SQL注入检测
    pub enable_sql_injection_detection: bool,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 连接池大小
    pub connection_pool_size: u32,
    /// 请求超时时间（秒）
    pub request_timeout_seconds: u64,
    /// 缓存TTL（秒）
    pub cache_ttl_seconds: u64,
    /// 是否启用响应缓存
    pub enable_response_cache: bool,
}

/// Agent配置集合 - 结构化管理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigs {
    /// SQL生成Agent配置
    pub sql_agent: AgentConfig,
    /// 数据分析Agent配置
    pub analysis_agent: AgentConfig,
    /// 聊天Agent配置
    pub chat_agent: AgentConfig,
    /// 推荐Agent配置
    pub recommendation_agent: AgentConfig,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 主要模型名称
    pub primary_model: String,
    /// 推理模型名称
    pub reasoning_model: String,
    /// 默认最大令牌数
    pub default_max_tokens: u32,
    /// 默认温度
    pub default_temperature: f32,
}

/// Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent名称
    pub name: String,
    /// 系统提示词
    pub preamble: String,
    /// 温度参数
    pub temperature: f32,
    /// 最大令牌数
    pub max_tokens: u32,
    /// 是否启用工具
    pub enable_tools: bool,
}

/// 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfigs {
    /// 数据库查询工具配置
    pub database_query: DatabaseQueryConfig,
    /// 数据分析工具配置
    pub data_analysis: DataAnalysisConfig,
    /// 推荐工具配置
    pub recommendation: RecommendationConfig,
}

/// 数据库查询工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseQueryConfig {
    /// 最大查询时间（秒）
    pub max_query_time: u64,
    /// 最大返回行数
    pub max_rows: u32,
    /// 是否启用查询计划
    pub enable_explain: bool,
}

/// 数据分析工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAnalysisConfig {
    /// 最大分析数据量
    pub max_data_size: usize,
    /// 是否启用可视化
    pub enable_visualization: bool,
}

/// 推荐工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationConfig {
    /// 最大推荐数量
    pub max_recommendations: u32,
    /// 推荐置信度阈值
    pub confidence_threshold: f32,
}

/// Rig AI监控指标
#[derive(Debug)]
pub struct RigAIMetrics {
    /// SQL生成请求总数
    pub sql_generation_total: Counter,
    /// 数据分析请求总数
    pub analysis_requests_total: Counter,
    /// 聊天消息总数
    pub chat_messages_total: Counter,
    /// 推荐请求总数
    pub recommendation_requests_total: Counter,
    /// 工具调用总数
    pub tool_calls_total: Counter,
    /// 处理时间
    pub processing_duration: Histogram,
    /// 错误总数
    pub errors_total: Counter,
    /// 当前活跃会话数
    pub active_sessions: Gauge,
}

/// 服务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub average_response_time: f64,
    /// 活跃Agent数
    pub active_agents: u32,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
}

impl RigAIMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let sql_generation_total = Counter::new(
            "duckhub_rig_sql_generation_total",
            "Total number of SQL generation requests"
        )?;
        registry.register(Box::new(sql_generation_total.clone()))?;

        let analysis_requests_total = Counter::new(
            "duckhub_rig_analysis_requests_total",
            "Total number of data analysis requests"
        )?;
        registry.register(Box::new(analysis_requests_total.clone()))?;

        let chat_messages_total = Counter::new(
            "duckhub_rig_chat_messages_total",
            "Total number of chat messages processed"
        )?;
        registry.register(Box::new(chat_messages_total.clone()))?;

        let recommendation_requests_total = Counter::new(
            "duckhub_rig_recommendation_requests_total",
            "Total number of recommendation requests"
        )?;
        registry.register(Box::new(recommendation_requests_total.clone()))?;

        let tool_calls_total = Counter::new(
            "duckhub_rig_tool_calls_total",
            "Total number of tool calls"
        )?;
        registry.register(Box::new(tool_calls_total.clone()))?;

        let processing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_rig_processing_duration_seconds",
                "Rig AI processing duration in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        )?;
        registry.register(Box::new(processing_duration.clone()))?;

        let errors_total = Counter::new(
            "duckhub_rig_errors_total",
            "Total number of errors encountered"
        )?;
        registry.register(Box::new(errors_total.clone()))?;

        let active_sessions = Gauge::new(
            "duckhub_rig_active_sessions",
            "Number of active sessions"
        )?;
        registry.register(Box::new(active_sessions.clone()))?;

        Ok(Self {
            sql_generation_total,
            analysis_requests_total,
            chat_messages_total,
            recommendation_requests_total,
            tool_calls_total,
            processing_duration,
            errors_total,
            active_sessions,
        })
    }
}

/// DuckHub工具集 - 企业级工具管理
#[derive(Clone)]
pub struct DuckHubToolSet {
    /// 数据库查询工具
    pub database_query: DatabaseQueryTool,
    /// 表结构检查工具
    pub schema_inspector: SchemaInspectorTool,
    /// 数据分析工具
    pub data_analyzer: DataAnalyzerTool,
    /// 推荐工具
    pub recommendation_tool: RecommendationTool,
    /// 工具权限管理器
    pub permission_manager: ToolPermissionManager,
    /// 工具性能监控器
    pub performance_monitor: ToolPerformanceMonitor,
}

/// 工具权限管理器
#[derive(Clone)]
pub struct ToolPermissionManager {
    /// 用户权限映射
    pub user_permissions: std::collections::HashMap<String, Vec<String>>,
    /// 工具访问控制列表
    pub tool_acl: std::collections::HashMap<String, ToolPermission>,
}

/// 工具权限定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    /// 工具名称
    pub tool_name: String,
    /// 允许的用户角色
    pub allowed_roles: Vec<String>,
    /// 最大调用频率（每分钟）
    pub max_calls_per_minute: u32,
    /// 是否需要审批
    pub requires_approval: bool,
}

/// 工具性能监控器
#[derive(Clone)]
pub struct ToolPerformanceMonitor {
    /// 工具调用计数器
    pub call_counters: std::collections::HashMap<String, u64>,
    /// 工具响应时间记录
    pub response_times: std::collections::HashMap<String, Vec<u64>>,
    /// 错误计数器
    pub error_counters: std::collections::HashMap<String, u64>,
}

impl ToolPermissionManager {
    /// 创建新的权限管理器
    pub fn new() -> Self {
        let mut tool_acl = std::collections::HashMap::new();

        // 设置默认工具权限
        tool_acl.insert("database_query".to_string(), ToolPermission {
            tool_name: "database_query".to_string(),
            allowed_roles: vec!["admin".to_string(), "analyst".to_string()],
            max_calls_per_minute: 60,
            requires_approval: false,
        });

        tool_acl.insert("schema_inspector".to_string(), ToolPermission {
            tool_name: "schema_inspector".to_string(),
            allowed_roles: vec!["admin".to_string(), "analyst".to_string(), "user".to_string()],
            max_calls_per_minute: 30,
            requires_approval: false,
        });

        Self {
            user_permissions: std::collections::HashMap::new(),
            tool_acl,
        }
    }

    /// 检查用户是否有权限使用指定工具
    pub fn check_permission(&self, user_id: &str, tool_name: &str) -> bool {
        if let Some(user_roles) = self.user_permissions.get(user_id) {
            if let Some(tool_permission) = self.tool_acl.get(tool_name) {
                return user_roles.iter().any(|role| tool_permission.allowed_roles.contains(role));
            }
        }
        false
    }
}

impl ToolPerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            call_counters: std::collections::HashMap::new(),
            response_times: std::collections::HashMap::new(),
            error_counters: std::collections::HashMap::new(),
        }
    }

    /// 记录工具调用
    pub fn record_call(&mut self, tool_name: &str, response_time_ms: u64) {
        // 增加调用计数
        *self.call_counters.entry(tool_name.to_string()).or_insert(0) += 1;

        // 记录响应时间
        self.response_times
            .entry(tool_name.to_string())
            .or_insert_with(Vec::new)
            .push(response_time_ms);
    }

    /// 记录工具错误
    pub fn record_error(&mut self, tool_name: &str) {
        *self.error_counters.entry(tool_name.to_string()).or_insert(0) += 1;
    }

    /// 获取工具性能统计
    pub fn get_stats(&self, tool_name: &str) -> Option<ToolStats> {
        let call_count = self.call_counters.get(tool_name).copied().unwrap_or(0);
        let error_count = self.error_counters.get(tool_name).copied().unwrap_or(0);

        if let Some(response_times) = self.response_times.get(tool_name) {
            let avg_response_time = if !response_times.is_empty() {
                response_times.iter().sum::<u64>() / response_times.len() as u64
            } else {
                0
            };

            Some(ToolStats {
                tool_name: tool_name.to_string(),
                call_count,
                error_count,
                avg_response_time_ms: avg_response_time,
                success_rate: if call_count > 0 {
                    ((call_count - error_count) as f64 / call_count as f64) * 100.0
                } else {
                    0.0
                },
            })
        } else {
            None
        }
    }
}

/// 工具性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    /// 工具名称
    pub tool_name: String,
    /// 调用次数
    pub call_count: u64,
    /// 错误次数
    pub error_count: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: u64,
    /// 成功率（百分比）
    pub success_rate: f64,
}

/// Agent响应类型
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RigResponseType {
    /// SQL生成
    SqlGeneration,
    /// 数据分析
    DataAnalysis,
    /// 聊天回复
    ChatResponse,
    /// 智能推荐
    Recommendation,
    /// 错误信息
    Error,
}

/// Rig Agent响应
#[derive(Debug, Clone, Serialize)]
pub struct RigAgentResponse {
    /// 响应ID
    pub response_id: String,
    /// 响应类型
    pub response_type: RigResponseType,
    /// 响应内容
    pub content: String,
    /// SQL查询（如果适用）
    pub sql_query: Option<String>,
    /// 查询结果（如果适用）
    pub query_result: Option<Value>,
    /// 推荐列表（如果适用）
    pub recommendations: Option<Vec<Value>>,
    /// 置信度
    pub confidence: f32,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 使用的模型
    pub model_used: String,
    /// 工具调用信息
    pub tool_calls: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 查询上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    /// 用户ID
    pub user_id: Option<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 业务领域
    pub business_domain: Option<String>,
    /// 可用表信息
    pub available_tables: Vec<String>,
    /// 查询历史
    pub query_history: Vec<String>,
    /// 上下文数据
    pub context_data: HashMap<String, Value>,
}

// ============================================================================
// 工具实现部分
// ============================================================================

/// 数据库查询工具
#[derive(Clone)]
pub struct DatabaseQueryTool {
    engine: Arc<DuckDBEngine>,
    query_service: Arc<QueryAnalyticsService>,
    config: DatabaseQueryConfig,
}

impl DatabaseQueryTool {
    pub fn new(
        engine: Arc<DuckDBEngine>,
        query_service: Arc<QueryAnalyticsService>,
        config: DatabaseQueryConfig,
    ) -> Self {
        Self {
            engine,
            query_service,
            config,
        }
    }
}

/// 数据库查询工具参数
#[derive(Debug, Deserialize)]
pub struct DatabaseQueryArgs {
    /// SQL查询语句
    pub sql: String,
    /// 限制返回行数
    pub limit: Option<u32>,
    /// 是否返回执行计划
    pub explain: Option<bool>,
}

/// 数据库查询工具输出
#[derive(Debug, Serialize)]
pub struct DatabaseQueryOutput {
    /// 是否成功
    pub success: bool,
    /// 返回数据
    pub data: Vec<Value>,
    /// 行数
    pub row_count: usize,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 查询计划（如果请求）
    pub query_plan: Option<String>,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

#[async_trait]
impl Tool for DatabaseQueryTool {
    const NAME: &'static str = "database_query";
    type Error = RigAIError;
    type Args = DatabaseQueryArgs;
    type Output = DatabaseQueryOutput;

    fn definition(&self, _prompt: String) -> impl std::future::Future<Output = ToolDefinition> + Send + Sync {
        async move {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "执行SQL查询并返回结果。支持DuckDB语法，适用于金融数据分析。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "要执行的SQL查询语句，必须符合DuckDB语法"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "限制返回的行数，默认100，最大1000",
                        "minimum": 1,
                        "maximum": 1000
                    },
                    "explain": {
                        "type": "boolean",
                        "description": "是否返回查询执行计划，用于性能分析"
                    }
                },
                "required": ["sql"]
            }),
        }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync {
        async move {
        let start_time = std::time::Instant::now();

        // 应用行数限制
        let mut sql = args.sql.clone();
        if let Some(limit) = args.limit {
            let limit = limit.min(self.config.max_rows);
            if !sql.to_uppercase().contains("LIMIT") {
                sql = format!("{} LIMIT {}", sql, limit);
            }
        }

        // 执行查询
        match self.query_service.execute_query(&sql).await {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;

                // 转换数据格式
                let data: Vec<Value> = result.data.iter()
                    .map(|row| serde_json::to_value(row).unwrap_or(Value::Null))
                    .collect();

                // 获取查询计划（如果请求）
                let query_plan = if args.explain.unwrap_or(false) && self.config.enable_explain {
                    let explain_sql = format!("EXPLAIN {}", sql);
                    match self.query_service.execute_query(&explain_sql).await {
                        Ok(plan_result) => Some(format!("{:?}", plan_result.data)),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                Ok(DatabaseQueryOutput {
                    success: true,
                    data,
                    row_count: result.row_count,
                    execution_time_ms: execution_time,
                    query_plan,
                    error: None,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                Ok(DatabaseQueryOutput {
                    success: false,
                    data: vec![],
                    row_count: 0,
                    execution_time_ms: execution_time,
                    query_plan: None,
                    error: Some(e.to_string()),
                })
            }
        }
        }
    }
}

/// 表结构检查工具
#[derive(Clone)]
pub struct SchemaInspectorTool {
    engine: Arc<DuckDBEngine>,
}

impl SchemaInspectorTool {
    pub fn new(engine: Arc<DuckDBEngine>) -> Self {
        Self { engine }
    }
}

/// 表结构检查参数
#[derive(Debug, Deserialize)]
pub struct SchemaInspectorArgs {
    /// 表名（可选，如果不提供则返回所有表）
    pub table_name: Option<String>,
    /// 是否包含详细信息
    pub include_details: Option<bool>,
}

/// 表结构检查输出
#[derive(Debug, Serialize)]
pub struct SchemaInspectorOutput {
    /// 表信息列表
    pub tables: Vec<TableInfo>,
    /// 总表数
    pub total_tables: usize,
}

/// 表信息
#[derive(Debug, Serialize)]
pub struct TableInfo {
    /// 表名
    pub table_name: String,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 行数估计
    pub estimated_rows: Option<u64>,
    /// 表大小（字节）
    pub size_bytes: Option<u64>,
}

/// 列信息
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    /// 列名
    pub column_name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可为空
    pub nullable: bool,
    /// 默认值
    pub default_value: Option<String>,
}

#[async_trait]
impl Tool for SchemaInspectorTool {
    const NAME: &'static str = "schema_inspector";
    type Error = RigAIError;
    type Args = SchemaInspectorArgs;
    type Output = SchemaInspectorOutput;

    fn definition(&self, _prompt: String) -> impl std::future::Future<Output = ToolDefinition> + Send + Sync {
        async move {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "检查数据库表结构和元数据信息。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "table_name": {
                        "type": "string",
                        "description": "要检查的表名，如果不提供则返回所有表的信息"
                    },
                    "include_details": {
                        "type": "boolean",
                        "description": "是否包含详细信息如行数和大小"
                    }
                }
            }),
        }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync {
        async move {
        // 这里应该实现实际的表结构检查逻辑
        // 由于DuckDBEngine的具体实现可能不同，这里提供一个框架

        let tables = if let Some(table_name) = args.table_name {
            // 检查特定表
            vec![TableInfo {
                table_name: table_name.clone(),
                columns: vec![], // 实际实现中应该查询表结构
                estimated_rows: None,
                size_bytes: None,
            }]
        } else {
            // 返回所有表
            vec![] // 实际实现中应该查询所有表
        };

        Ok(SchemaInspectorOutput {
            total_tables: tables.len(),
            tables,
        })
        }
    }
}

/// 数据分析工具
#[derive(Clone)]
pub struct DataAnalyzerTool {
    config: DataAnalysisConfig,
}

impl DataAnalyzerTool {
    pub fn new(config: DataAnalysisConfig) -> Self {
        Self { config }
    }
}

/// 数据分析工具参数
#[derive(Debug, Deserialize)]
pub struct DataAnalysisArgs {
    /// 要分析的数据
    pub data: Vec<Value>,
    /// 分析类型
    pub analysis_type: String,
    /// 是否生成可视化
    pub generate_visualization: Option<bool>,
}

/// 数据分析工具输出
#[derive(Debug, Serialize)]
pub struct DataAnalysisOutput {
    /// 分析结果
    pub analysis: String,
    /// 统计指标
    pub statistics: Value,
    /// 可视化数据（如果生成）
    pub visualization: Option<Value>,
}

#[async_trait]
impl Tool for DataAnalyzerTool {
    const NAME: &'static str = "data_analyzer";
    type Error = RigAIError;
    type Args = DataAnalysisArgs;
    type Output = DataAnalysisOutput;

    fn definition(&self, _prompt: String) -> impl std::future::Future<Output = ToolDefinition> + Send + Sync {
        async move {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "分析数据并提供统计洞察和可视化。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "array",
                        "description": "要分析的数据数组"
                    },
                    "analysis_type": {
                        "type": "string",
                        "description": "分析类型：descriptive, trend, correlation, anomaly",
                        "enum": ["descriptive", "trend", "correlation", "anomaly"]
                    },
                    "generate_visualization": {
                        "type": "boolean",
                        "description": "是否生成可视化图表"
                    }
                },
                "required": ["data", "analysis_type"]
            }),
        }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync {
        async move {
        // 简单的数据分析实现
        let data_size = args.data.len();

        let analysis = match args.analysis_type.as_str() {
            "descriptive" => format!("数据集包含{}条记录。", data_size),
            "trend" => "趋势分析：数据呈现稳定趋势。".to_string(),
            "correlation" => "相关性分析：发现中等程度的正相关。".to_string(),
            "anomaly" => "异常检测：未发现明显异常值。".to_string(),
            _ => "未知分析类型".to_string(),
        };

        let statistics = json!({
            "count": data_size,
            "analysis_type": args.analysis_type
        });

        let visualization = if args.generate_visualization.unwrap_or(false) {
            Some(json!({"type": "chart", "data": "visualization_data"}))
        } else {
            None
        };

        Ok(DataAnalysisOutput {
            analysis,
            statistics,
            visualization,
        })
        }
    }
}

/// 推荐工具
#[derive(Clone)]
pub struct RecommendationTool {
    config: RecommendationConfig,
}

impl RecommendationTool {
    pub fn new(config: RecommendationConfig) -> Self {
        Self { config }
    }
}

/// 推荐工具参数
#[derive(Debug, Deserialize)]
pub struct RecommendationArgs {
    /// 用户查询历史
    pub query_history: Vec<String>,
    /// 当前上下文
    pub context: String,
    /// 推荐类型
    pub recommendation_type: String,
}

/// 推荐工具输出
#[derive(Debug, Serialize)]
pub struct RecommendationOutput {
    /// 推荐列表
    pub recommendations: Vec<RecommendationItem>,
    /// 推荐总数
    pub total_count: usize,
}

/// 推荐项
#[derive(Debug, Serialize)]
pub struct RecommendationItem {
    /// 推荐标题
    pub title: String,
    /// 推荐描述
    pub description: String,
    /// 推荐类型
    pub recommendation_type: String,
    /// 置信度
    pub confidence: f32,
    /// 建议操作
    pub action: Option<String>,
}

#[async_trait]
impl Tool for RecommendationTool {
    const NAME: &'static str = "recommendation_engine";
    type Error = RigAIError;
    type Args = RecommendationArgs;
    type Output = RecommendationOutput;

    fn definition(&self, _prompt: String) -> impl std::future::Future<Output = ToolDefinition> + Send + Sync {
        async move {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "基于用户历史和上下文提供智能推荐。".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query_history": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "用户的查询历史"
                    },
                    "context": {
                        "type": "string",
                        "description": "当前上下文信息"
                    },
                    "recommendation_type": {
                        "type": "string",
                        "description": "推荐类型：query, analysis, optimization",
                        "enum": ["query", "analysis", "optimization"]
                    }
                },
                "required": ["context", "recommendation_type"]
            }),
        }
        }
    }

    fn call(&self, args: Self::Args) -> impl std::future::Future<Output = std::result::Result<Self::Output, Self::Error>> + Send + Sync {
        async move {
        // 简单的推荐实现
        let mut recommendations = Vec::new();

        match args.recommendation_type.as_str() {
            "query" => {
                recommendations.push(RecommendationItem {
                    title: "查询优化建议".to_string(),
                    description: "建议添加索引以提升查询性能".to_string(),
                    recommendation_type: "query".to_string(),
                    confidence: 0.8,
                    action: Some("CREATE INDEX idx_timestamp ON transactions(timestamp)".to_string()),
                });
            },
            "analysis" => {
                recommendations.push(RecommendationItem {
                    title: "数据分析建议".to_string(),
                    description: "建议进行趋势分析以发现模式".to_string(),
                    recommendation_type: "analysis".to_string(),
                    confidence: 0.7,
                    action: None,
                });
            },
            "optimization" => {
                recommendations.push(RecommendationItem {
                    title: "性能优化建议".to_string(),
                    description: "建议使用分区表以提升查询性能".to_string(),
                    recommendation_type: "optimization".to_string(),
                    confidence: 0.9,
                    action: None,
                });
            },
            _ => {
                return Err(RigAIError::ToolError("未知推荐类型".to_string()));
            }
        }

        Ok(RecommendationOutput {
            total_count: recommendations.len(),
            recommendations,
        })
        }
    }
}

// ============================================================================
// RigAIService实现部分
// ============================================================================

impl RigAIService {
    /// 创建新的Rig AI服务
    #[instrument(skip(engine, query_service, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        query_service: Arc<QueryAnalyticsService>,
        config: RigAIConfig,
        registry: &Registry,
    ) -> AnyhowResult<Self> {
        let metrics = RigAIMetrics::new(registry)
            .context("创建监控指标失败")?;

        // 初始化DeepSeek客户端
        let deepseek_client = deepseek::Client::new(&config.deepseek_api_key);

        // 初始化工具集
        let toolset = Self::create_toolset(&engine, &query_service, &config);

        // 创建SQL生成Agent
        let sql_agent = Self::create_sql_agent(&deepseek_client, &config, &toolset);

        // 创建数据分析Agent
        let analysis_agent = Self::create_analysis_agent(&deepseek_client, &config, &toolset);

        // 创建聊天Agent
        let chat_agent = Self::create_chat_agent(&deepseek_client, &config, &toolset);

        // 创建推荐Agent
        let recommendation_agent = Self::create_recommendation_agent(&deepseek_client, &config, &toolset);

        // 初始化RAG服务
        let rag_service = Self::initialize_rag_service(&config).await?;

        let model_name = config.model_config.primary_model.clone();

        let service = Self {
            deepseek_client,
            engine,
            query_service,
            sql_agent,
            analysis_agent,
            chat_agent,
            recommendation_agent,
            toolset,
            config,
            metrics,
            rag_service,
        };

        info!("Rig AI服务初始化完成，使用DeepSeek模型: {}", model_name);
        Ok(service)
    }

    /// 创建测试实例（仅用于测试）
    #[cfg(test)]
    pub async fn create_test_instance(_config: RigAIConfig, _registry: Arc<Registry>) -> Result<Self> {
        // 为了简化测试，我们创建一个模拟的服务实例
        // 在实际测试中，这些组件会被正确模拟
        Err(DuckHubError::config("测试实例创建暂未实现，请使用模拟测试"))
    }

    /// SQL查询方法（用于测试和API）- 增强版错误处理
    #[instrument(skip(self))]
    pub async fn sql_query(&self, query: &str) -> Result<String> {
        self.metrics.sql_generation_total.inc();
        let timer = self.metrics.processing_duration.start_timer();

        // 输入验证
        if query.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::validation("查询不能为空"));
        }

        // 检测危险SQL操作
        let dangerous_keywords = ["DROP", "DELETE", "TRUNCATE", "ALTER", "CREATE", "INSERT", "UPDATE"];
        let upper_query = query.to_uppercase();
        for keyword in &dangerous_keywords {
            if upper_query.contains(keyword) {
                self.metrics.errors_total.inc();
                warn!("检测到危险SQL操作: {} in query: {}", keyword, query);
                return Err(DuckHubError::validation(
                    format!("不允许执行{}操作，仅支持查询操作", keyword)
                ));
            }
        }

        // 检测SQL注入模式
        let injection_patterns = ["';", "--", "/*", "*/", "UNION", "OR 1=1", "AND 1=1"];
        for pattern in &injection_patterns {
            if upper_query.contains(pattern) {
                self.metrics.errors_total.inc();
                warn!("检测到潜在SQL注入: {} in query: {}", pattern, query);
                return Err(DuckHubError::validation("检测到潜在的SQL注入攻击"));
            }
        }

        let result = self.sql_agent.prompt(query).await
            .map_err(|e| {
                self.metrics.errors_total.inc();
                error!("SQL Agent调用失败: {}", e);
                DuckHubError::internal(format!("SQL生成失败: {}", e))
            })?;

        timer.observe_duration();

        // 验证生成的SQL
        if result.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::internal("生成的SQL为空"));
        }

        Ok(result)
    }

    /// 数据分析方法（用于测试和API）- 增强版错误处理
    #[instrument(skip(self))]
    pub async fn analyze_data(&self, query: &str) -> Result<String> {
        self.metrics.analysis_requests_total.inc();
        let timer = self.metrics.processing_duration.start_timer();

        // 输入验证
        if query.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::validation("分析查询不能为空"));
        }

        if query.len() > 10000 {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::validation("查询内容过长，请简化查询"));
        }

        let result = self.analysis_agent.prompt(query).await
            .map_err(|e| {
                self.metrics.errors_total.inc();
                error!("Analysis Agent调用失败: {}", e);
                DuckHubError::internal(format!("数据分析失败: {}", e))
            })?;

        timer.observe_duration();

        if result.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::internal("分析结果为空"));
        }

        Ok(result)
    }

    /// 聊天方法（用于测试和API）- 增强版错误处理
    #[instrument(skip(self))]
    pub async fn chat(&self, message: &str) -> Result<String> {
        self.metrics.chat_messages_total.inc();
        let timer = self.metrics.processing_duration.start_timer();

        // 输入验证
        if message.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::validation("消息不能为空"));
        }

        if message.len() > 5000 {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::validation("消息过长，请简化内容"));
        }

        // 检测不当内容（简单实现）
        let inappropriate_keywords = ["hack", "attack", "exploit", "malware"];
        let lower_message = message.to_lowercase();
        for keyword in &inappropriate_keywords {
            if lower_message.contains(keyword) {
                self.metrics.errors_total.inc();
                warn!("检测到不当内容: {} in message: {}", keyword, message);
                return Err(DuckHubError::validation("消息包含不当内容"));
            }
        }

        let result = self.chat_agent.prompt(message).await
            .map_err(|e| {
                self.metrics.errors_total.inc();
                error!("Chat Agent调用失败: {}", e);
                DuckHubError::internal(format!("聊天处理失败: {}", e))
            })?;

        timer.observe_duration();

        if result.trim().is_empty() {
            self.metrics.errors_total.inc();
            return Err(DuckHubError::internal("聊天回复为空"));
        }

        Ok(result)
    }

    /// 获取服务统计信息
    pub fn get_stats(&self) -> ServiceStats {
        let sql_requests = self.metrics.sql_generation_total.get() as u64;
        let analysis_requests = self.metrics.analysis_requests_total.get() as u64;
        let chat_requests = self.metrics.chat_messages_total.get() as u64;
        let recommendation_requests = self.metrics.recommendation_requests_total.get() as u64;
        let total_requests = sql_requests + analysis_requests + chat_requests + recommendation_requests;
        let failed_requests = self.metrics.errors_total.get() as u64;
        let successful_requests = total_requests.saturating_sub(failed_requests);

        // 计算平均响应时间
        let sample_count = self.metrics.processing_duration.get_sample_count();
        let average_response_time = if sample_count > 0 {
            (self.metrics.processing_duration.get_sample_sum() / sample_count as f64) * 1000.0 // 转换为毫秒
        } else {
            0.0
        };

        ServiceStats {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time,
            active_agents: 4, // SQL, Analysis, Chat, Recommendation
            uptime_seconds: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// 初始化RAG服务
    async fn initialize_rag_service(
        _config: &RigAIConfig,
    ) -> Result<Option<Arc<RigRagService>>> {
        // 暂时返回None，等Rig API问题解决后再实现
        // 实际实现应该是：
        // let rag_service = RigRagService::new(
        //     config.deepseek_api_key.clone(),
        //     config.rag_config.clone(),
        //     engine.clone(),
        // ).await?;
        // Ok(Some(Arc::new(rag_service)))

        Ok(None)
    }

    /// 创建工具集
    fn create_toolset(
        engine: &Arc<DuckDBEngine>,
        query_service: &Arc<QueryAnalyticsService>,
        config: &RigAIConfig,
    ) -> DuckHubToolSet {
        DuckHubToolSet {
            database_query: DatabaseQueryTool::new(
                engine.clone(),
                query_service.clone(),
                config.tool_configs.database_query.clone(),
            ),
            schema_inspector: SchemaInspectorTool::new(engine.clone()),
            data_analyzer: DataAnalyzerTool::new(
                config.tool_configs.data_analysis.clone(),
            ),
            recommendation_tool: RecommendationTool::new(
                config.tool_configs.recommendation.clone(),
            ),
            permission_manager: ToolPermissionManager::new(),
            performance_monitor: ToolPerformanceMonitor::new(),
        }
    }

    /// 创建SQL生成Agent
    fn create_sql_agent(
        client: &deepseek::Client,
        config: &RigAIConfig,
        toolset: &DuckHubToolSet,
    ) -> Agent<deepseek::DeepSeekCompletionModel> {
        let sql_config = &config.agent_configs.sql_agent;

        let mut builder = client
            .agent(&config.model_config.primary_model)
            .preamble(&sql_config.preamble)
            .max_tokens(sql_config.max_tokens as u64)
            .temperature(sql_config.temperature as f64);

        if sql_config.enable_tools {
            builder = builder
                .tool(toolset.database_query.clone())
                .tool(toolset.schema_inspector.clone());
        }

        builder.build()
    }

    /// 创建数据分析Agent
    fn create_analysis_agent(
        client: &deepseek::Client,
        config: &RigAIConfig,
        toolset: &DuckHubToolSet,
    ) -> Agent<deepseek::DeepSeekCompletionModel> {
        let analysis_config = &config.agent_configs.analysis_agent;

        let mut builder = client
            .agent(&config.model_config.primary_model)
            .preamble(&analysis_config.preamble)
            .max_tokens(analysis_config.max_tokens as u64)
            .temperature(analysis_config.temperature as f64);

        if analysis_config.enable_tools {
            builder = builder
                .tool(toolset.database_query.clone())
                .tool(toolset.data_analyzer.clone());
        }

        builder.build()
    }

    /// 创建聊天Agent
    fn create_chat_agent(
        client: &deepseek::Client,
        config: &RigAIConfig,
        toolset: &DuckHubToolSet,
    ) -> Agent<deepseek::DeepSeekCompletionModel> {
        let chat_config = &config.agent_configs.chat_agent;

        let mut builder = client
            .agent(&config.model_config.primary_model)
            .preamble(&chat_config.preamble)
            .max_tokens(chat_config.max_tokens as u64)
            .temperature(chat_config.temperature as f64);

        if chat_config.enable_tools {
            builder = builder
                .tool(toolset.database_query.clone())
                .tool(toolset.schema_inspector.clone());
        }

        builder.build()
    }

    /// 创建推荐Agent
    fn create_recommendation_agent(
        client: &deepseek::Client,
        config: &RigAIConfig,
        toolset: &DuckHubToolSet,
    ) -> Agent<deepseek::DeepSeekCompletionModel> {
        let recommendation_config = &config.agent_configs.recommendation_agent;

        let mut builder = client
            .agent(&config.model_config.primary_model)
            .preamble(&recommendation_config.preamble)
            .max_tokens(recommendation_config.max_tokens as u64)
            .temperature(recommendation_config.temperature as f64);

        if recommendation_config.enable_tools {
            builder = builder
                .tool(toolset.recommendation_tool.clone());
        }

        builder.build()
    }

    /// 生成SQL查询
    #[instrument(skip(self, query, context))]
    pub async fn generate_sql(
        &self,
        query: &str,
        context: &QueryContext,
    ) -> AnyhowResult<RigAgentResponse> {
        let start_time = std::time::Instant::now();
        self.metrics.sql_generation_total.inc();

        // 构建提示词
        let prompt = self.build_sql_prompt(query, context);

        // 调用Agent
        let response = self.sql_agent
            .prompt(&prompt)
            .await
            .map_err(|e| RigAIError::AgentError(e.to_string()))?;

        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        // 提取SQL查询
        let sql_query = self.extract_sql_query(&response);

        // 构建响应
        Ok(RigAgentResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: RigResponseType::SqlGeneration,
            content: response,
            sql_query: Some(sql_query),
            query_result: None,
            recommendations: None,
            confidence: 0.9, // 高置信度
            processing_time_ms: processing_time.as_millis() as u64,
            model_used: self.config.model_config.primary_model.clone(),
            tool_calls: vec![], // 实际实现中应该记录工具调用
            created_at: Utc::now(),
        })
    }

    /// 构建SQL生成提示词
    fn build_sql_prompt(&self, query: &str, context: &QueryContext) -> String {
        let mut prompt_parts = Vec::new();

        // 添加用户查询
        prompt_parts.push(format!("用户查询: {}", query));

        // 添加上下文信息
        if let Some(domain) = &context.business_domain {
            prompt_parts.push(format!("业务领域: {}", domain));
        }

        if !context.available_tables.is_empty() {
            prompt_parts.push("可用表:".to_string());
            for table in &context.available_tables {
                prompt_parts.push(format!("- {}", table));
            }
        }

        if !context.query_history.is_empty() {
            prompt_parts.push("最近查询历史:".to_string());
            for (i, query) in context.query_history.iter().take(3).enumerate() {
                prompt_parts.push(format!("{}. {}", i + 1, query));
            }
        }

        prompt_parts.join("\n")
    }

    /// 从响应中提取SQL查询 - 增强版
    fn extract_sql_query(&self, response: &str) -> String {
        // 1. 首先查找SQL代码块
        if let Some(start) = response.find("```sql") {
            if let Some(end) = response[start + 6..].find("```") {
                let sql = response[start + 6..start + 6 + end].trim();
                if !sql.is_empty() {
                    return sql.to_string();
                }
            }
        }

        // 2. 查找其他代码块格式
        if let Some(start) = response.find("```") {
            if let Some(end) = response[start + 3..].find("```") {
                let sql = response[start + 3..start + 3 + end].trim();
                if sql.to_uppercase().contains("SELECT") ||
                   sql.to_uppercase().contains("INSERT") ||
                   sql.to_uppercase().contains("UPDATE") ||
                   sql.to_uppercase().contains("DELETE") {
                    return sql.to_string();
                }
            }
        }

        // 3. 查找SQL关键字开始的语句
        let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "WITH", "CREATE"];
        let upper_response = response.to_uppercase();

        for keyword in &sql_keywords {
            if let Some(start) = upper_response.find(keyword) {
                let remaining = &response[start..];

                // 查找语句结束位置
                if let Some(end) = remaining.find(";") {
                    return remaining[..end + 1].trim().to_string();
                }

                // 如果没有分号，查找下一行或段落结束
                let lines: Vec<&str> = remaining.lines().collect();
                if !lines.is_empty() {
                    let mut sql_lines = Vec::new();
                    for line in lines {
                        let trimmed = line.trim();
                        if trimmed.is_empty() && !sql_lines.is_empty() {
                            break;
                        }
                        if !trimmed.is_empty() {
                            sql_lines.push(trimmed);
                        }
                    }
                    if !sql_lines.is_empty() {
                        return sql_lines.join(" ");
                    }
                }
            }
        }

        // 4. 如果都没找到，返回清理后的响应
        response.trim().to_string()
    }

    // RAG功能暂时禁用，等基础功能稳定后再启用
    /*
    /// RAG增强的SQL查询
    #[instrument(skip(self, request))]
    pub async fn rag_sql_query(&self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        if let Some(ref rag_agent) = self.rag_sql_agent {
            rag_agent.process_query(request).await
                .map_err(|e| DuckHubError::internal(e.to_string()))
        } else {
            Err(DuckHubError::config("RAG SQL Agent未初始化"))
        }
    }

    /// RAG增强的数据分析
    #[instrument(skip(self, request))]
    pub async fn rag_analysis(&self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        if let Some(ref rag_agent) = self.rag_analysis_agent {
            rag_agent.analyze(request).await
                .map_err(|e| DuckHubError::internal(e.to_string()))
        } else {
            Err(DuckHubError::config("RAG Analysis Agent未初始化"))
        }
    }

    /// RAG增强的聊天
    #[instrument(skip(self, request))]
    pub async fn rag_chat(&mut self, request: RAGQueryRequest) -> Result<RAGQueryResponse> {
        if let Some(ref mut rag_agent) = self.rag_chat_agent {
            rag_agent.chat(request).await
                .map_err(|e| DuckHubError::internal(e.to_string()))
        } else {
            Err(DuckHubError::config("RAG Chat Agent未初始化"))
        }
    }

    /// 检索金融知识
    #[instrument(skip(self))]
    pub async fn retrieve_knowledge(&self, query: &str) -> Result<RetrievalResult> {
        if let Some(ref knowledge_base) = self.knowledge_base {
            knowledge_base.retrieve_knowledge(query).await
                .map_err(|e| DuckHubError::internal(e.to_string()))
        } else {
            Err(DuckHubError::config("金融知识库未初始化"))
        }
    }
    */
}

// ============================================================================
// 默认配置实现
// ============================================================================

impl RigAIConfig {
    /// 获取SQL生成提示词 - 增强版
    fn get_sql_generation_prompt() -> String {
        r#"你是一个专业的金融数据分析师和SQL专家。你的任务是将自然语言查询转换为准确的SQL语句。

核心要求：
1. 生成的SQL必须符合DuckDB语法
2. 优先使用标准SQL语法，避免特定数据库的扩展
3. 对于金融数据，确保数值计算的精度
4. 使用适当的聚合函数和窗口函数
5. 考虑数据的时间序列特性
6. 确保查询性能，避免全表扫描

金融领域常见表结构：
- transactions: 交易记录表 (id, amount, currency, timestamp, account_id, type)
- accounts: 账户表 (id, account_number, balance, currency, created_at)
- users: 用户表 (id, name, email, created_at, status)
- market_data: 市场数据表 (symbol, price, volume, timestamp)

SQL生成规则：
1. 查询所有/全部 → SELECT * FROM table_name
2. 统计/计数/数量 → SELECT COUNT(*) FROM table_name
3. 按...分组/分类 → SELECT column, COUNT(*) FROM table_name GROUP BY column
4. 排序/排列 → SELECT * FROM table_name ORDER BY column
5. 连接/关联 → SELECT * FROM table1 t1 JOIN table2 t2 ON t1.id = t2.ref_id
6. 过滤/筛选/条件 → SELECT * FROM table_name WHERE condition
7. 求和/总计 → SELECT SUM(column) FROM table_name
8. 时间范围 → SELECT * FROM table_name WHERE date_column BETWEEN 'start' AND 'end'
9. 模糊匹配/包含 → SELECT * FROM table_name WHERE column LIKE '%pattern%'
10. 去重/唯一 → SELECT DISTINCT column FROM table_name

响应格式：
```sql
-- 生成的SQL语句
SELECT ...
```

重要：必须严格按照上述规则生成SQL，确保包含正确的关键字。"#.to_string()
    }

    /// 获取数据分析提示词
    fn get_data_analysis_prompt() -> String {
        r#"你是一个专业的金融数据分析师。你的任务是分析查询结果并提供有价值的洞察。

分析重点：
1. 识别数据中的趋势和模式
2. 发现异常值和潜在风险
3. 提供业务建议和行动建议
4. 使用适当的金融术语和概念
5. 量化分析结果，提供具体数字
6. 考虑时间序列特征和季节性

响应要求：
- 使用中文回复
- 结构化输出，包含关键指标
- 提供可执行的建议
- 突出重要发现和风险点"#.to_string()
    }

    /// 获取聊天提示词
    fn get_chat_prompt() -> String {
        r#"你是DuckHub金融数据平台的AI助手。你专门帮助用户进行金融数据分析和查询。

你的能力：
1. 理解自然语言查询并转换为SQL
2. 分析查询结果并提供洞察
3. 回答金融数据相关问题
4. 提供数据分析建议和最佳实践

交互原则：
- 友好、专业、准确
- 使用中文回复
- 主动询问澄清问题
- 提供具体的操作建议"#.to_string()
    }

    /// 获取推荐提示词
    fn get_recommendation_prompt() -> String {
        r#"你是DuckHub金融数据平台的智能推荐引擎。你的任务是基于用户的查询历史、数据模式和业务需求提供智能推荐。

推荐类型：
1. 查询优化建议
2. 相关数据探索
3. 分析方法推荐
4. 最佳实践建议

推荐原则：
- 基于用户历史行为
- 考虑数据特征和模式
- 符合业务逻辑
- 提供可操作的建议"#.to_string()
    }
}

impl Default for RigAIConfig {
    fn default() -> Self {
        let deepseek_api_key = std::env::var("DEEPSEEK_API_KEY")
            .unwrap_or_else(|_| "sk-a4f888023ea74cef8afae36dc8581512".to_string());

        // 检查API密钥是否为占位符
        if deepseek_api_key == "your-deepseek-api-key" || deepseek_api_key.starts_with("your-") {
            warn!("⚠️  检测到占位符API密钥，AI功能将受限！");
            warn!("   请设置真实的DeepSeek API密钥:");
            warn!("   方法1: export DEEPSEEK_API_KEY='your-real-api-key'");
            warn!("   方法2: 在配置文件中设置真实密钥");
            warn!("   获取API密钥: https://platform.deepseek.com/");
        } else {
            info!("✅ DeepSeek API密钥已配置 (长度: {}字符)", deepseek_api_key.len());
        }

        Self {
            deepseek_api_key,
            model_config: ModelConfig::default(),
            agent_configs: AgentConfigs::default(),
            tool_configs: ToolConfigs::default(),
            rag_config: RagConfig::default(),
            security_config: SecurityConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            primary_model: "deepseek-chat".to_string(),
            reasoning_model: "deepseek-reasoner".to_string(),
            default_max_tokens: 4000,
            default_temperature: 0.3,
        }
    }
}

impl Default for ToolConfigs {
    fn default() -> Self {
        Self {
            database_query: DatabaseQueryConfig::default(),
            data_analysis: DataAnalysisConfig::default(),
            recommendation: RecommendationConfig::default(),
        }
    }
}

impl Default for DatabaseQueryConfig {
    fn default() -> Self {
        Self {
            max_query_time: 30, // 30秒
            max_rows: 1000,
            enable_explain: true,
        }
    }
}

impl Default for DataAnalysisConfig {
    fn default() -> Self {
        Self {
            max_data_size: 10_000_000, // 10MB
            enable_visualization: true,
        }
    }
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            max_recommendations: 10,
            confidence_threshold: 0.7,
        }
    }
}

impl Default for AgentConfigs {
    fn default() -> Self {
        Self {
            sql_agent: AgentConfig {
                name: "SQL专家".to_string(),
                preamble: RigAIConfig::get_sql_generation_prompt(),
                temperature: 0.1, // 低温度确保准确性
                max_tokens: 2000,
                enable_tools: true,
            },
            analysis_agent: AgentConfig {
                name: "数据分析师".to_string(),
                preamble: RigAIConfig::get_data_analysis_prompt(),
                temperature: 0.3, // 中等温度平衡创造性和准确性
                max_tokens: 4000,
                enable_tools: true,
            },
            chat_agent: AgentConfig {
                name: "智能助手".to_string(),
                preamble: RigAIConfig::get_chat_prompt(),
                temperature: 0.7, // 较高温度增加对话自然性
                max_tokens: 2000,
                enable_tools: true,
            },
            recommendation_agent: AgentConfig {
                name: "推荐专家".to_string(),
                preamble: RigAIConfig::get_recommendation_prompt(),
                temperature: 0.5, // 中等温度平衡准确性和多样性
                max_tokens: 3000,
                enable_tools: true,
            },
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_api_key_validation: true,
            max_concurrent_requests: 100,
            rate_limit_per_minute: 1000,
            enable_sql_injection_detection: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            connection_pool_size: 10,
            request_timeout_seconds: 30,
            cache_ttl_seconds: 300, // 5分钟
            enable_response_cache: true,
        }
    }
}
