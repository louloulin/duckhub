//! 基于Rig框架的AI Agent实现
//! 使用DeepSeek作为LLM provider，提供统一的AI服务

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_query_analytics::{QueryAnalyticsService, QueryResult};
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
}

/// Rig AI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigAIConfig {
    /// DeepSeek API密钥
    pub deepseek_api_key: String,
    /// 模型配置
    pub model_config: ModelConfig,
    /// Agent配置
    pub agent_configs: HashMap<String, AgentConfig>,
    /// 工具配置
    pub tool_configs: ToolConfigs,
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

/// DuckHub工具集
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

    async fn definition(&self, _prompt: String) -> ToolDefinition {
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

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
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

    async fn definition(&self, _prompt: String) -> ToolDefinition {
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

    async fn call(&self, args: Self::Args) -> std::result::Result<Self::Output, Self::Error> {
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
        };

        info!("Rig AI服务初始化完成，使用DeepSeek模型: {}", model_name);
        Ok(service)
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
        }
    }

    /// 创建SQL生成Agent
    fn create_sql_agent(
        client: &deepseek::Client,
        config: &RigAIConfig,
        toolset: &DuckHubToolSet,
    ) -> Agent<deepseek::DeepSeekCompletionModel> {
        let sql_config = config.agent_configs.get("sql_generation")
            .expect("SQL生成Agent配置缺失");

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
        let analysis_config = config.agent_configs.get("data_analysis")
            .expect("数据分析Agent配置缺失");

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
        let chat_config = config.agent_configs.get("chat")
            .expect("聊天Agent配置缺失");

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
        let recommendation_config = config.agent_configs.get("recommendation")
            .expect("推荐Agent配置缺失");

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

    /// 从响应中提取SQL查询
    fn extract_sql_query(&self, response: &str) -> String {
        // 简单实现：查找SQL代码块
        if let Some(start) = response.find("```sql") {
            if let Some(end) = response[start..].find("```") {
                return response[start + 6..start + end].trim().to_string();
            }
        }

        // 如果没有找到SQL代码块，尝试查找SELECT语句
        if let Some(start) = response.to_uppercase().find("SELECT") {
            if let Some(end) = response[start..].find(";") {
                return response[start..start + end + 1].trim().to_string();
            }
            // 如果没有分号，返回整个后续内容
            return response[start..].trim().to_string();
        }

        // 如果都没找到，返回原始响应
        response.to_string()
    }
}

// ============================================================================
// 默认配置实现
// ============================================================================

impl Default for RigAIConfig {
    fn default() -> Self {
        let mut agent_configs = HashMap::new();

        // SQL生成Agent配置
        agent_configs.insert("sql_generation".to_string(), AgentConfig {
            name: "SQL生成Agent".to_string(),
            preamble: include_str!("prompts/sql_generation.txt").to_string(),
            temperature: 0.1, // 低温度确保准确性
            max_tokens: 2000,
            enable_tools: true,
        });

        // 数据分析Agent配置
        agent_configs.insert("data_analysis".to_string(), AgentConfig {
            name: "数据分析Agent".to_string(),
            preamble: include_str!("prompts/data_analysis.txt").to_string(),
            temperature: 0.3, // 中等温度平衡创造性和准确性
            max_tokens: 4000,
            enable_tools: true,
        });

        // 聊天Agent配置
        agent_configs.insert("chat".to_string(), AgentConfig {
            name: "聊天Agent".to_string(),
            preamble: include_str!("prompts/chat.txt").to_string(),
            temperature: 0.7, // 较高温度增加对话自然性
            max_tokens: 2000,
            enable_tools: true,
        });

        // 推荐Agent配置
        agent_configs.insert("recommendation".to_string(), AgentConfig {
            name: "推荐Agent".to_string(),
            preamble: include_str!("prompts/recommendation.txt").to_string(),
            temperature: 0.5, // 中等温度平衡准确性和多样性
            max_tokens: 3000,
            enable_tools: true,
        });

        Self {
            deepseek_api_key: std::env::var("DEEPSEEK_API_KEY")
                .unwrap_or_else(|_| "your-deepseek-api-key".to_string()),
            model_config: ModelConfig::default(),
            agent_configs,
            tool_configs: ToolConfigs::default(),
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            primary_model: deepseek::DEEPSEEK_CHAT.to_string(),
            reasoning_model: deepseek::DEEPSEEK_REASONER.to_string(),
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
