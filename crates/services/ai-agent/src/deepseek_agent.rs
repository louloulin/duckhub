//! 基于DeepSeek的AI Agent实现
//! 直接使用DeepSeek API进行LLM交互，专为金融数据分析优化

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_query_analytics::{QueryAnalyticsService, QueryResult};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use prometheus::{Counter, Histogram, Registry};
use reqwest::Client;
use anyhow::{Result as AnyhowResult, Context};
use thiserror::Error;

/// DeepSeek AI Agent错误类型
#[derive(Error, Debug)]
pub enum DeepSeekAgentError {
    #[error("DeepSeek API错误: {0}")]
    DeepSeekError(String),
    #[error("SQL生成失败: {0}")]
    SqlGenerationError(String),
    #[error("查询执行失败: {0}")]
    QueryExecutionError(String),
    #[error("HTTP请求失败: {0}")]
    HttpError(String),
}

/// 基于DeepSeek的AI Agent服务
pub struct DeepSeekAIAgent {
    /// HTTP客户端
    http_client: Client,
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 查询分析服务
    query_service: Arc<QueryAnalyticsService>,
    /// 配置
    config: DeepSeekConfig,
    /// 监控指标
    metrics: DeepSeekMetrics,
}

/// DeepSeek配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    /// DeepSeek API密钥
    pub api_key: String,
    /// API基础URL
    pub api_base_url: String,
    /// 模型名称
    pub model_name: String,
    /// 最大令牌数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("DEEPSEEK_API_KEY")
                .unwrap_or_else(|_| "your-deepseek-api-key".to_string()),
            api_base_url: "https://api.deepseek.com".to_string(),
            model_name: "deepseek-chat".to_string(),
            max_tokens: 4000,
            temperature: 0.1, // 金融数据分析需要更低的温度
            timeout_seconds: 30,
        }
    }
}

/// DeepSeek监控指标
#[derive(Debug)]
pub struct DeepSeekMetrics {
    /// SQL生成请求总数
    pub sql_generation_total: Counter,
    /// 数据分析请求总数
    pub analysis_requests_total: Counter,
    /// 聊天消息总数
    pub chat_messages_total: Counter,
    /// API调用总数
    pub api_calls_total: Counter,
    /// 处理时间
    pub processing_duration: Histogram,
    /// 错误总数
    pub errors_total: Counter,
}

impl DeepSeekMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let sql_generation_total = Counter::new(
            "duckhub_deepseek_sql_generation_total",
            "Total number of SQL generation requests"
        )?;
        registry.register(Box::new(sql_generation_total.clone()))?;

        let analysis_requests_total = Counter::new(
            "duckhub_deepseek_analysis_requests_total",
            "Total number of data analysis requests"
        )?;
        registry.register(Box::new(analysis_requests_total.clone()))?;

        let chat_messages_total = Counter::new(
            "duckhub_deepseek_chat_messages_total",
            "Total number of chat messages processed"
        )?;
        registry.register(Box::new(chat_messages_total.clone()))?;

        let api_calls_total = Counter::new(
            "duckhub_deepseek_api_calls_total",
            "Total number of DeepSeek API calls"
        )?;
        registry.register(Box::new(api_calls_total.clone()))?;

        let processing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_deepseek_processing_duration_seconds",
                "DeepSeek AI processing duration in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
        )?;
        registry.register(Box::new(processing_duration.clone()))?;

        let errors_total = Counter::new(
            "duckhub_deepseek_errors_total",
            "Total number of errors encountered"
        )?;
        registry.register(Box::new(errors_total.clone()))?;

        Ok(Self {
            sql_generation_total,
            analysis_requests_total,
            chat_messages_total,
            api_calls_total,
            processing_duration,
            errors_total,
        })
    }
}

/// DeepSeek AI响应
#[derive(Debug, Clone, Serialize)]
pub struct DeepSeekResponse {
    /// 响应ID
    pub response_id: String,
    /// 响应类型
    pub response_type: DeepSeekResponseType,
    /// 响应内容
    pub content: String,
    /// SQL查询（如果适用）
    pub sql_query: Option<String>,
    /// 查询结果（如果适用）
    pub query_result: Option<serde_json::Value>,
    /// 置信度
    pub confidence: f32,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 使用的模型
    pub model_used: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 响应类型
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DeepSeekResponseType {
    /// SQL生成
    SqlGeneration,
    /// 数据分析
    DataAnalysis,
    /// 聊天回复
    ChatResponse,
    /// 错误信息
    Error,
}

/// DeepSeek API请求结构
#[derive(Debug, Clone, Serialize)]
struct DeepSeekApiRequest {
    model: String,
    messages: Vec<DeepSeekMessage>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

/// DeepSeek消息
#[derive(Debug, Clone, Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

/// DeepSeek API响应结构
#[derive(Debug, Clone, Deserialize)]
struct DeepSeekApiResponse {
    id: String,
    choices: Vec<DeepSeekChoice>,
}

/// DeepSeek选择
#[derive(Debug, Clone, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekResponseMessage,
}

/// DeepSeek响应消息
#[derive(Debug, Clone, Deserialize)]
struct DeepSeekResponseMessage {
    content: String,
}

/// 查询上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    /// 用户ID
    pub user_id: Option<String>,
    /// 业务领域
    pub business_domain: Option<String>,
    /// 可用表信息
    pub available_tables: Vec<String>,
    /// 查询历史
    pub query_history: Vec<String>,
}

impl DeepSeekAIAgent {
    /// 创建新的DeepSeek AI Agent
    #[instrument(skip(engine, query_service, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        query_service: Arc<QueryAnalyticsService>,
        config: DeepSeekConfig,
        registry: &Registry,
    ) -> AnyhowResult<Self> {
        let metrics = DeepSeekMetrics::new(registry)
            .context("创建监控指标失败")?;
        
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .context("创建HTTP客户端失败")?;

        let agent = Self {
            http_client,
            engine,
            query_service,
            config,
            metrics,
        };

        info!("DeepSeek AI Agent初始化完成，使用模型: {}", agent.config.model_name);
        Ok(agent)
    }

    /// 调用DeepSeek API
    async fn call_deepseek_api(&self, messages: Vec<DeepSeekMessage>) -> AnyhowResult<String> {
        self.metrics.api_calls_total.inc();

        let request = DeepSeekApiRequest {
            model: self.config.model_name.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: false,
        };

        let url = format!("{}/v1/chat/completions", self.config.api_base_url);
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("发送DeepSeek API请求失败")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            return Err(DeepSeekAgentError::DeepSeekError(format!("API调用失败: {}", error_text)).into());
        }

        let deepseek_response: DeepSeekApiResponse = response
            .json()
            .await
            .context("解析DeepSeek API响应失败")?;

        if let Some(choice) = deepseek_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(DeepSeekAgentError::DeepSeekError("API响应中没有内容".to_string()).into())
        }
    }

    /// 生成SQL查询
    #[instrument(skip(self, query, context))]
    pub async fn generate_sql(
        &self,
        query: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();
        self.metrics.sql_generation_total.inc();

        let response = self.generate_sql_internal(query, context).await;

        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                self.metrics.errors_total.inc();
                warn!("SQL生成失败: {}", e);
                Ok(DeepSeekResponse {
                    response_id: Uuid::new_v4().to_string(),
                    response_type: DeepSeekResponseType::Error,
                    content: format!("SQL生成失败: {}", e),
                    sql_query: None,
                    query_result: None,
                    confidence: 0.0,
                    processing_time_ms: processing_time.as_millis() as u64,
                    model_used: self.config.model_name.clone(),
                    created_at: Utc::now(),
                })
            }
        }
    }

    /// 内部SQL生成实现
    async fn generate_sql_internal(
        &self,
        query: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();

        // 构建SQL生成提示词
        let system_prompt = self.get_sql_generation_prompt();
        let context_info = self.build_context_info(context);
        let user_prompt = format!(
            "上下文信息：\n{}\n\n用户查询：{}\n\n请生成对应的SQL语句：",
            context_info, query
        );

        let messages = vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        // 调用DeepSeek生成SQL
        let sql_result = self.call_deepseek_api(messages).await?;
        let sql_query = self.clean_sql_response(&sql_result);

        let processing_time = start_time.elapsed();

        Ok(DeepSeekResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: DeepSeekResponseType::SqlGeneration,
            content: format!("已生成SQL查询：\n```sql\n{}\n```", sql_query),
            sql_query: Some(sql_query),
            query_result: None,
            confidence: 0.9, // DeepSeek在SQL生成方面表现优秀
            processing_time_ms: processing_time.as_millis() as u64,
            model_used: self.config.model_name.clone(),
            created_at: Utc::now(),
        })
    }

    /// 执行查询并分析结果
    #[instrument(skip(self, sql_query, context))]
    pub async fn execute_and_analyze(
        &self,
        sql_query: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();
        self.metrics.analysis_requests_total.inc();

        let response = self.execute_and_analyze_internal(sql_query, context).await;

        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                self.metrics.errors_total.inc();
                warn!("查询执行和分析失败: {}", e);
                Ok(DeepSeekResponse {
                    response_id: Uuid::new_v4().to_string(),
                    response_type: DeepSeekResponseType::Error,
                    content: format!("查询执行和分析失败: {}", e),
                    sql_query: Some(sql_query.to_string()),
                    query_result: None,
                    confidence: 0.0,
                    processing_time_ms: processing_time.as_millis() as u64,
                    model_used: self.config.model_name.clone(),
                    created_at: Utc::now(),
                })
            }
        }
    }

    /// 内部查询执行和分析实现
    async fn execute_and_analyze_internal(
        &self,
        sql_query: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();

        // 执行SQL查询
        let query_result = self.query_service
            .execute_query(sql_query)
            .await
            .map_err(|e| DeepSeekAgentError::QueryExecutionError(e.to_string()))?;

        // 准备分析提示词
        let system_prompt = self.get_data_analysis_prompt();
        let analysis_prompt = self.build_analysis_prompt(sql_query, &query_result, context);

        let messages = vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: analysis_prompt,
            },
        ];

        // 调用DeepSeek进行数据分析
        let analysis_result = self.call_deepseek_api(messages).await?;

        let processing_time = start_time.elapsed();

        Ok(DeepSeekResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: DeepSeekResponseType::DataAnalysis,
            content: analysis_result,
            sql_query: Some(sql_query.to_string()),
            query_result: Some(serde_json::to_value(&query_result.data)?),
            confidence: 0.85,
            processing_time_ms: processing_time.as_millis() as u64,
            model_used: self.config.model_name.clone(),
            created_at: Utc::now(),
        })
    }

    /// 处理聊天消息
    #[instrument(skip(self, message, context))]
    pub async fn process_chat(
        &self,
        session_id: &str,
        message: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();
        self.metrics.chat_messages_total.inc();

        let response = self.process_chat_internal(session_id, message, context).await;

        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        match response {
            Ok(resp) => Ok(resp),
            Err(e) => {
                self.metrics.errors_total.inc();
                warn!("聊天处理失败: {}", e);
                Ok(DeepSeekResponse {
                    response_id: Uuid::new_v4().to_string(),
                    response_type: DeepSeekResponseType::Error,
                    content: format!("抱歉，处理您的消息时遇到了问题: {}", e),
                    sql_query: None,
                    query_result: None,
                    confidence: 0.0,
                    processing_time_ms: processing_time.as_millis() as u64,
                    model_used: self.config.model_name.clone(),
                    created_at: Utc::now(),
                })
            }
        }
    }

    /// 内部聊天处理实现
    async fn process_chat_internal(
        &self,
        session_id: &str,
        message: &str,
        context: &QueryContext,
    ) -> AnyhowResult<DeepSeekResponse> {
        let start_time = std::time::Instant::now();

        // 构建聊天提示词
        let system_prompt = self.get_chat_prompt();
        let context_info = self.build_context_info(context);
        let user_prompt = format!(
            "会话ID: {}\n上下文信息：\n{}\n\n用户消息：{}\n\n请提供有帮助的回复：",
            session_id, context_info, message
        );

        let messages = vec![
            DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            DeepSeekMessage {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        // 调用DeepSeek进行聊天
        let chat_result = self.call_deepseek_api(messages).await?;

        let processing_time = start_time.elapsed();

        Ok(DeepSeekResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: DeepSeekResponseType::ChatResponse,
            content: chat_result,
            sql_query: None,
            query_result: None,
            confidence: 0.8,
            processing_time_ms: processing_time.as_millis() as u64,
            model_used: self.config.model_name.clone(),
            created_at: Utc::now(),
        })
    }

    /// 获取SQL生成提示词
    fn get_sql_generation_prompt(&self) -> String {
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

响应格式：
只返回SQL语句，不要包含任何解释或markdown格式。
如果查询不明确，返回最合理的解释。"#.to_string()
    }

    /// 获取数据分析提示词
    fn get_data_analysis_prompt(&self) -> String {
        r#"你是一个专业的金融数据分析师。你的任务是分析查询结果并提供有价值的洞察。

分析重点：
1. 识别数据中的趋势和模式
2. 发现异常值和潜在风险
3. 提供业务建议和行动建议
4. 使用适当的金融术语和概念
5. 量化分析结果，提供具体数字
6. 考虑时间序列特征和季节性

分析框架：
- 描述性统计：均值、中位数、标准差等
- 趋势分析：增长率、变化幅度
- 风险评估：波动性、异常检测
- 比较分析：同比、环比、基准对比
- 预测建议：基于历史数据的趋势预测

响应要求：
- 使用中文回复
- 结构化输出，包含关键指标
- 提供可执行的建议
- 突出重要发现和风险点"#.to_string()
    }

    /// 获取聊天提示词
    fn get_chat_prompt(&self) -> String {
        r#"你是DuckHub金融数据平台的AI助手。你专门帮助用户进行金融数据分析和查询。

你的能力：
1. 理解自然语言查询并转换为SQL
2. 分析查询结果并提供洞察
3. 回答金融数据相关问题
4. 提供数据分析建议和最佳实践
5. 解释复杂的金融概念

交互原则：
- 友好、专业、准确
- 使用中文回复
- 主动询问澄清问题
- 提供具体的操作建议
- 解释技术概念时要通俗易懂
- 对于敏感的金融数据要谨慎处理

如果用户的问题不清楚，主动询问更多细节以提供更准确的帮助。"#.to_string()
    }

    /// 构建上下文信息
    fn build_context_info(&self, context: &QueryContext) -> String {
        let mut context_parts = Vec::new();

        if let Some(domain) = &context.business_domain {
            context_parts.push(format!("业务领域: {}", domain));
        }

        if !context.available_tables.is_empty() {
            context_parts.push("可用表:".to_string());
            for table in &context.available_tables {
                context_parts.push(format!("- {}", table));
            }
        }

        if !context.query_history.is_empty() {
            context_parts.push("最近查询历史:".to_string());
            for (i, query) in context.query_history.iter().take(3).enumerate() {
                context_parts.push(format!("{}. {}", i + 1, query));
            }
        }

        context_parts.join("\n")
    }

    /// 构建分析提示词
    fn build_analysis_prompt(
        &self,
        sql_query: &str,
        query_result: &QueryResult,
        context: &QueryContext,
    ) -> String {
        let context_info = self.build_context_info(context);

        format!(
            r#"请分析以下查询结果：

执行的SQL查询：
```sql
{}
```

查询结果统计：
- 返回行数: {}
- 执行时间: {}ms
- 查询ID: {}

上下文信息：
{}

查询结果数据（前10行）：
{}

请提供详细的数据分析，包括：
1. 数据概览和关键指标
2. 趋势和模式识别
3. 异常值检测
4. 业务洞察和建议
5. 风险评估（如适用）

请用中文回复，并提供具体的数值和百分比。"#,
            sql_query,
            query_result.row_count,
            query_result.execution_time_ms,
            query_result.query_id,
            context_info,
            self.format_query_result_preview(&query_result.data.iter().map(|row| serde_json::to_value(row).unwrap_or(serde_json::Value::Null)).collect::<Vec<_>>())
        )
    }

    /// 格式化查询结果预览
    fn format_query_result_preview(&self, data: &[serde_json::Value]) -> String {
        if data.is_empty() {
            return "无数据".to_string();
        }

        let preview_data: Vec<&serde_json::Value> = data.iter().take(10).collect();
        serde_json::to_string_pretty(&preview_data)
            .unwrap_or_else(|_| "数据格式化失败".to_string())
    }

    /// 清理SQL响应
    fn clean_sql_response(&self, response: &str) -> String {
        // 移除markdown格式和多余的空白
        response
            .trim()
            .replace("```sql", "")
            .replace("```", "")
            .trim()
            .to_string()
    }

    /// 获取服务统计信息
    pub async fn get_stats(&self) -> AnyhowResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();

        stats.insert("sql_generation_total".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.sql_generation_total.get() as u64)));
        stats.insert("analysis_requests_total".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.analysis_requests_total.get() as u64)));
        stats.insert("chat_messages_total".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.chat_messages_total.get() as u64)));
        stats.insert("api_calls_total".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.api_calls_total.get() as u64)));
        stats.insert("errors_total".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.errors_total.get() as u64)));
        stats.insert("model_name".to_string(),
                    serde_json::Value::String(self.config.model_name.clone()));

        Ok(stats)
    }

    /// 健康检查
    pub async fn health_check(&self) -> AnyhowResult<bool> {
        // 检查数据库连接
        match self.engine.check_connection().await {
            Ok(_) => {
                debug!("DeepSeek AI Agent健康检查通过");
                Ok(true)
            },
            Err(e) => {
                warn!("DeepSeek AI Agent健康检查失败: {}", e);
                Ok(false)
            }
        }
    }
}
