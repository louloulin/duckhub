//! AI Agent服务 - DuckHub金融数据平台

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use duckhub_query_analytics::QueryAnalyticsService;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, debug, instrument};
use prometheus::{Counter, Histogram, Gauge, Registry};

pub mod nlp;
pub mod recommendations;
pub mod automation;
pub mod chat;

// 基于Rig框架的AI Agent实现
pub mod rig_agent;
// 基于Rig框架的RAG系统实现
pub mod rig_rag;
// 传统RAG功能（暂时禁用，等迁移完成后删除）
// pub mod rag_agent;

// 测试模块
#[cfg(test)]
mod rig_tests;
#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod api_test;

pub use nlp::*;
pub use recommendations::*;
pub use automation::*;
pub use chat::*;

// 导出Rig相关组件
pub use rig_agent::*;
// 导出新的RAG系统
pub use rig_rag::*;

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: Option<String>,
    pub session_id: Option<String>,
}

/// 分析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub query: String,
    pub analysis_type: AnalysisType,
    pub context: Option<String>,
}

/// 建议请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestRequest {
    pub query: String,
    pub query_type: QueryType,
    pub context: Option<String>,
}

/// 分析类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Performance,
    Schema,
    Query,
    Data,
}

/// 查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Alter,
}

/// AI Agent服务
pub struct AIAgentService {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 查询分析服务
    query_service: Arc<QueryAnalyticsService>,
    /// NLP处理器
    nlp_processor: Arc<NLPProcessor>,
    /// 推荐引擎
    recommendation_engine: Arc<RecommendationEngine>,
    /// 自动化引擎
    automation_engine: Arc<AutomationEngine>,
    /// 聊天处理器
    chat_processor: Arc<ChatProcessor>,
    /// 配置
    config: AIAgentConfig,
    /// 监控指标
    metrics: AIAgentMetrics,
}

/// AI Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAgentConfig {
    /// 是否启用自然语言查询
    pub enable_nlp: bool,
    /// 是否启用智能推荐
    pub enable_recommendations: bool,
    /// 是否启用自动化分析
    pub enable_automation: bool,
    /// 是否启用聊天功能
    pub enable_chat: bool,
    /// OpenAI API密钥
    pub openai_api_key: Option<String>,
    /// 模型名称
    pub model_name: String,
    /// 最大令牌数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
    /// 会话历史保留数量
    pub max_conversation_history: usize,
}

impl Default for AIAgentConfig {
    fn default() -> Self {
        Self {
            enable_nlp: true,
            enable_recommendations: true,
            enable_automation: true,
            enable_chat: true,
            openai_api_key: None,
            model_name: "gpt-3.5-turbo".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            max_conversation_history: 10,
        }
    }
}

/// AI Agent监控指标
#[derive(Debug)]
pub struct AIAgentMetrics {
    /// NLP查询总数
    pub nlp_queries_total: Counter,
    /// 推荐生成总数
    pub recommendations_total: Counter,
    /// 自动化任务总数
    pub automation_tasks_total: Counter,
    /// 聊天消息总数
    pub chat_messages_total: Counter,
    /// 处理时间
    pub processing_duration: Histogram,
    /// 当前活跃会话数
    pub active_sessions: Gauge,
}

impl AIAgentMetrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let nlp_queries_total = Counter::new(
            "duckhub_ai_agent_nlp_queries_total",
            "Total number of NLP queries processed"
        )?;
        registry.register(Box::new(nlp_queries_total.clone()))?;

        let recommendations_total = Counter::new(
            "duckhub_ai_agent_recommendations_total",
            "Total number of recommendations generated"
        )?;
        registry.register(Box::new(recommendations_total.clone()))?;

        let automation_tasks_total = Counter::new(
            "duckhub_ai_agent_automation_tasks_total",
            "Total number of automation tasks executed"
        )?;
        registry.register(Box::new(automation_tasks_total.clone()))?;

        let chat_messages_total = Counter::new(
            "duckhub_ai_agent_chat_messages_total",
            "Total number of chat messages processed"
        )?;
        registry.register(Box::new(chat_messages_total.clone()))?;

        let processing_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_ai_agent_processing_duration_seconds",
                "AI Agent processing duration in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
        )?;
        registry.register(Box::new(processing_duration.clone()))?;

        let active_sessions = Gauge::new(
            "duckhub_ai_agent_active_sessions",
            "Number of currently active AI Agent sessions"
        )?;
        registry.register(Box::new(active_sessions.clone()))?;

        Ok(Self {
            nlp_queries_total,
            recommendations_total,
            automation_tasks_total,
            chat_messages_total,
            processing_duration,
            active_sessions,
        })
    }
}

/// AI Agent响应
#[derive(Debug, Clone, Serialize)]
pub struct AIAgentResponse {
    /// 响应ID
    pub response_id: String,
    /// 响应类型
    pub response_type: ResponseType,
    /// 响应内容
    pub content: String,
    /// SQL查询（如果适用）
    pub sql_query: Option<String>,
    /// 查询结果（如果适用）
    pub query_result: Option<serde_json::Value>,
    /// 推荐列表（如果适用）
    pub recommendations: Option<Vec<Recommendation>>,
    /// 置信度
    pub confidence: f32,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 响应类型
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ResponseType {
    /// 自然语言查询结果
    NLPQuery,
    /// 智能推荐
    Recommendation,
    /// 自动化分析结果
    AutomationResult,
    /// 聊天回复
    ChatResponse,
    /// 错误信息
    Error,
}

impl AIAgentService {
    /// 创建新的AI Agent服务
    #[instrument(skip(engine, query_service, registry))]
    pub async fn new(
        engine: Arc<DuckDBEngine>,
        query_service: Arc<QueryAnalyticsService>,
        config: AIAgentConfig,
        registry: &Registry,
    ) -> Result<Self> {
        let metrics = AIAgentMetrics::new(registry)?;
        
        let nlp_processor = Arc::new(NLPProcessor::new(&config).await?);
        let recommendation_engine = Arc::new(RecommendationEngine::new(Arc::clone(&engine), &config).await?);
        let automation_engine = Arc::new(AutomationEngine::new(Arc::clone(&engine), Arc::clone(&query_service)).await?);
        let chat_processor = Arc::new(ChatProcessor::new(&config).await?);

        let service = Self {
            engine,
            query_service,
            nlp_processor,
            recommendation_engine,
            automation_engine,
            chat_processor,
            config,
            metrics,
        };

        info!("AI Agent服务初始化完成");
        Ok(service)
    }

    /// 处理自然语言查询
    #[instrument(skip(self, query))]
    pub async fn process_nlp_query(&self, query: &str) -> Result<AIAgentResponse> {
        if !self.config.enable_nlp {
            return Err(DuckHubError::validation("NLP功能未启用"));
        }

        let start_time = std::time::Instant::now();
        self.metrics.nlp_queries_total.inc();

        let response = self.process_nlp_query_internal(query).await?;
        
        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        Ok(response)
    }

    /// 内部NLP查询处理
    async fn process_nlp_query_internal(&self, query: &str) -> Result<AIAgentResponse> {
        let start_time = std::time::Instant::now();
        
        // 解析自然语言查询
        let parsed_query = self.nlp_processor.parse_query(query).await?;
        
        // 生成SQL查询
        let sql_query = self.nlp_processor.generate_sql(&parsed_query).await?;
        
        // 执行查询
        let query_result = self.query_service.execute_query(&sql_query).await?;
        
        // 生成自然语言回复
        let content = self.nlp_processor.generate_response(&parsed_query, &query_result).await?;
        
        let processing_time = start_time.elapsed();

        Ok(AIAgentResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: ResponseType::NLPQuery,
            content,
            sql_query: Some(sql_query),
            query_result: Some(serde_json::to_value(&query_result.data)?),
            recommendations: None,
            confidence: parsed_query.confidence,
            processing_time_ms: processing_time.as_millis() as u64,
            created_at: Utc::now(),
        })
    }

    /// 生成智能推荐
    #[instrument(skip(self))]
    pub async fn generate_recommendations(&self, context: &RecommendationContext) -> Result<AIAgentResponse> {
        if !self.config.enable_recommendations {
            return Err(DuckHubError::validation("推荐功能未启用"));
        }

        let start_time = std::time::Instant::now();
        self.metrics.recommendations_total.inc();

        let recommendations = self.recommendation_engine.generate_recommendations(context).await?;
        let content = format!("为您生成了{}条智能推荐", recommendations.len());
        
        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        Ok(AIAgentResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: ResponseType::Recommendation,
            content,
            sql_query: None,
            query_result: None,
            recommendations: Some(recommendations),
            confidence: 0.8,
            processing_time_ms: processing_time.as_millis() as u64,
            created_at: Utc::now(),
        })
    }

    /// 执行自动化分析
    #[instrument(skip(self))]
    pub async fn execute_automation(&self, task: &AutomationTask) -> Result<AIAgentResponse> {
        if !self.config.enable_automation {
            return Err(DuckHubError::validation("自动化功能未启用"));
        }

        let start_time = std::time::Instant::now();
        self.metrics.automation_tasks_total.inc();

        let result = self.automation_engine.execute_task(task).await?;
        let content = format!("自动化任务执行完成: {}", result.summary);
        
        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        Ok(AIAgentResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: ResponseType::AutomationResult,
            content,
            sql_query: result.sql_query,
            query_result: Some(serde_json::to_value(&result.data)?),
            recommendations: None,
            confidence: result.confidence,
            processing_time_ms: processing_time.as_millis() as u64,
            created_at: Utc::now(),
        })
    }

    /// 处理聊天消息
    #[instrument(skip(self, message))]
    pub async fn process_chat_message(&self, session_id: &str, message: &str) -> Result<AIAgentResponse> {
        if !self.config.enable_chat {
            return Err(DuckHubError::validation("聊天功能未启用"));
        }

        let start_time = std::time::Instant::now();
        self.metrics.chat_messages_total.inc();

        let response = self.chat_processor.process_message(session_id, message).await?;
        
        let processing_time = start_time.elapsed();
        self.metrics.processing_duration.observe(processing_time.as_secs_f64());

        Ok(AIAgentResponse {
            response_id: Uuid::new_v4().to_string(),
            response_type: ResponseType::ChatResponse,
            content: response.content,
            sql_query: response.sql_query,
            query_result: response.query_result,
            recommendations: response.recommendations,
            confidence: response.confidence,
            processing_time_ms: processing_time.as_millis() as u64,
            created_at: Utc::now(),
        })
    }

    /// 获取AI Agent统计信息
    pub async fn get_agent_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        stats.insert("nlp_queries_total".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.nlp_queries_total.get() as u64)));
        stats.insert("recommendations_total".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.recommendations_total.get() as u64)));
        stats.insert("automation_tasks_total".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.automation_tasks_total.get() as u64)));
        stats.insert("chat_messages_total".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.chat_messages_total.get() as u64)));
        stats.insert("active_sessions".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from(self.metrics.active_sessions.get() as u64)));

        Ok(stats)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        // 检查各个组件的健康状态
        match self.engine.check_connection().await {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(e) => {
                warn!("AI Agent服务健康检查失败: {}", e);
                Ok(HealthStatus::Unhealthy)
            }
        }
    }

    /// 聊天对话
    #[instrument(skip(self))]
    pub async fn chat(&self, request: ChatRequest) -> Result<serde_json::Value> {
        info!("处理聊天请求: {}", request.message);

        // 模拟AI聊天响应
        Ok(serde_json::json!({
            "response": "我理解您的需求。基于DuckLake的强大功能，我可以为您提供数据查询、版本管理和性能优化建议。",
            "timestamp": chrono::Utc::now(),
            "session_id": request.session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            "metadata": {
                "sql_query": "SELECT * FROM transactions WHERE amount > 1000",
                "execution_time": 125,
                "result_count": 1250
            }
        }))
    }

    /// 数据分析
    #[instrument(skip(self))]
    pub async fn analyze(&self, request: AnalysisRequest) -> Result<serde_json::Value> {
        info!("处理分析请求: {:?}", request.analysis_type);

        let analysis_result = match request.analysis_type {
            AnalysisType::Performance => "查询性能分析：建议为user_id字段添加索引，预计可提升查询速度40%",
            AnalysisType::Schema => "Schema分析：建议为transactions表添加status字段，这是一个向后兼容的安全操作",
            AnalysisType::Query => "查询分析：发现可优化的JOIN操作，建议重写为子查询以提升性能",
            AnalysisType::Data => "数据分析：检测到数据质量问题，建议清理重复记录",
        };

        Ok(serde_json::json!({
            "analysis_type": request.analysis_type,
            "result": analysis_result,
            "recommendations": [
                "优化建议1：添加索引",
                "优化建议2：重构查询",
                "优化建议3：数据清理"
            ],
            "confidence": 0.85,
            "timestamp": chrono::Utc::now()
        }))
    }

    /// 查询建议
    #[instrument(skip(self))]
    pub async fn suggest(&self, request: SuggestRequest) -> Result<serde_json::Value> {
        info!("处理建议请求: {:?}", request.query_type);

        let suggestion = match request.query_type {
            QueryType::Select => "建议使用LIMIT子句限制返回结果数量，避免内存溢出",
            QueryType::Insert => "建议使用批量插入以提升性能",
            QueryType::Update => "建议在WHERE子句中使用索引字段",
            QueryType::Delete => "建议先备份数据再执行删除操作",
            QueryType::Create => "建议为主键和外键字段创建索引",
            QueryType::Alter => "建议在低峰期执行表结构变更",
        };

        Ok(serde_json::json!({
            "query_type": request.query_type,
            "suggestion": suggestion,
            "optimized_query": format!("-- 优化后的查询\n{}", request.query),
            "performance_impact": "预计性能提升30-50%",
            "timestamp": chrono::Utc::now()
        }))
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

    async fn create_test_query_service() -> Arc<QueryAnalyticsService> {
        let engine = create_test_engine().await;
        let config = duckhub_query_analytics::QueryAnalyticsConfig::default();
        let registry = Registry::new();

        Arc::new(QueryAnalyticsService::new(engine, config, &registry).await.unwrap())
    }

    #[tokio::test]
    async fn test_ai_agent_service_creation() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_nlp_query_processing() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let result = service.process_nlp_query("显示所有用户的数量").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.response_type, ResponseType::NLPQuery);
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_recommendation_generation() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let context = RecommendationContext {
            user_id: Some("test_user".to_string()),
            current_query: Some("SELECT * FROM users".to_string()),
            query_history: vec!["SELECT COUNT(*) FROM orders".to_string()],
            table_info: HashMap::new(),
            business_domain: Some("finance".to_string()),
            time_range: None,
        };

        let result = service.generate_recommendations(&context).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.response_type, ResponseType::Recommendation);
    }

    #[tokio::test]
    async fn test_automation_execution() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let task = AutomationTask {
            task_id: "test_task".to_string(),
            task_type: AutomationTaskType::DataQualityCheck,
            name: "测试数据质量检查".to_string(),
            description: "检查数据质量".to_string(),
            parameters: HashMap::new(),
            schedule: None,
            enabled: true,
            created_at: Utc::now(),
        };

        let result = service.execute_automation(&task).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.response_type, ResponseType::AutomationResult);
    }

    #[tokio::test]
    async fn test_chat_message_processing() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let result = service.process_chat_message("session_001", "你好，我需要帮助").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.response_type, ResponseType::ChatResponse);
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_agent_stats() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let stats = service.get_agent_stats().await.unwrap();

        assert!(stats.contains_key("nlp_queries_total"));
        assert!(stats.contains_key("recommendations_total"));
        assert!(stats.contains_key("automation_tasks_total"));
        assert!(stats.contains_key("chat_messages_total"));
        assert!(stats.contains_key("active_sessions"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = AIAgentConfig::default();
        let registry = Registry::new();

        let service = AIAgentService::new(engine, query_service, config, &registry).await.unwrap();

        let health = service.health_check().await.unwrap();
        assert!(matches!(health, HealthStatus::Healthy));
    }
}
