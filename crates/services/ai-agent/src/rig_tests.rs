//! Rig AI Agent测试

#[cfg(test)]
mod tests {
    use super::super::rig_agent::*;
    use duckhub_database::DuckDBEngine;
    use duckhub_query_analytics::QueryAnalyticsService;
    use prometheus::Registry;
    use std::sync::Arc;
    use tempfile::tempdir;
    use rig::tool::Tool;

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

    fn create_test_config() -> RigAIConfig {
        RigAIConfig::default()
    }

    #[tokio::test]
    async fn test_rig_agent_creation() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = create_test_config();
        let registry = Registry::new();

        let agent = RigAIService::new(engine, query_service, config, &registry).await;
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_database_query_tool() {
        let engine = create_test_engine().await;
        let query_service = create_test_query_service().await;
        let config = DatabaseQueryConfig::default();

        let tool = DatabaseQueryTool::new(engine, query_service, config);
        
        // 创建测试表
        let definition = tool.definition("".to_string()).await;
        assert_eq!(definition.name, "database_query");
        assert!(definition.description.contains("SQL"));
    }

    #[tokio::test]
    async fn test_schema_inspector_tool() {
        let engine = create_test_engine().await;
        let tool = SchemaInspectorTool::new(engine);
        
        let definition = tool.definition("".to_string()).await;
        assert_eq!(definition.name, "schema_inspector");
        assert!(definition.description.contains("表结构"));
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = RigAIConfig::default();
        
        // 验证模型配置
        assert!(config.model_config.primary_model.contains("deepseek"));
        
        // 验证Agent配置
        assert!(config.agent_configs.contains_key("sql_generation"));
        assert!(config.agent_configs.contains_key("data_analysis"));
        assert!(config.agent_configs.contains_key("chat"));
        assert!(config.agent_configs.contains_key("recommendation"));
        
        // 验证工具配置
        assert!(config.tool_configs.database_query.max_rows > 0);
        assert!(config.tool_configs.data_analysis.max_data_size > 0);
        assert!(config.tool_configs.recommendation.max_recommendations > 0);
    }

    #[tokio::test]
    async fn test_metrics_creation() {
        let registry = Registry::new();
        let metrics = RigAIMetrics::new(&registry);
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    async fn test_response_types() {
        // 测试响应类型的序列化
        let response_type = RigResponseType::SqlGeneration;
        let serialized = serde_json::to_string(&response_type).unwrap();
        assert!(serialized.contains("SqlGeneration"));

        let response_type = RigResponseType::DataAnalysis;
        let serialized = serde_json::to_string(&response_type).unwrap();
        assert!(serialized.contains("DataAnalysis"));

        let response_type = RigResponseType::ChatResponse;
        let serialized = serde_json::to_string(&response_type).unwrap();
        assert!(serialized.contains("ChatResponse"));

        let response_type = RigResponseType::Recommendation;
        let serialized = serde_json::to_string(&response_type).unwrap();
        assert!(serialized.contains("Recommendation"));

        let response_type = RigResponseType::Error;
        let serialized = serde_json::to_string(&response_type).unwrap();
        assert!(serialized.contains("Error"));
    }

    #[tokio::test]
    async fn test_rag_config_creation() {
        use crate::rig_rag::RagConfig;

        // 测试默认配置
        let config = RagConfig::default();
        assert_eq!(config.embedding_model, "text-embedding-ada-002");
        assert_eq!(config.completion_model, "gpt-4");
        assert_eq!(config.default_max_docs, 3);
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.context_window_size, 4000);
    }

    #[tokio::test]
    async fn test_rag_query_types() {
        use crate::rig_rag::{RagQueryRequest, QueryType};

        // 测试SQL生成查询
        let sql_request = RagQueryRequest {
            query: "SELECT * FROM users".to_string(),
            query_type: QueryType::SqlGeneration,
            max_docs: Some(5),
            enable_rag: true,
            context: None,
        };

        assert_eq!(sql_request.query_type, QueryType::SqlGeneration);
        assert_eq!(sql_request.max_docs, Some(5));
        assert!(sql_request.enable_rag);

        // 测试数据分析查询
        let analysis_request = RagQueryRequest {
            query: "分析用户行为趋势".to_string(),
            query_type: QueryType::DataAnalysis,
            max_docs: None,
            enable_rag: true,
            context: None,
        };

        assert_eq!(analysis_request.query_type, QueryType::DataAnalysis);
        assert_eq!(analysis_request.max_docs, None);
    }

    #[tokio::test]
    async fn test_financial_document_creation() {
        use crate::rig_rag::{FinancialDocument, DocumentType};
        use uuid::Uuid;

        let doc = FinancialDocument {
            id: Uuid::new_v4().to_string(),
            title: "测试文档".to_string(),
            content: "这是一个测试文档的内容".to_string(),
            doc_type: DocumentType::SqlPattern,
            created_at: chrono::Utc::now(),
            metadata: serde_json::json!({"category": "test"}),
        };

        assert_eq!(doc.title, "测试文档");
        assert_eq!(doc.content, "这是一个测试文档的内容");
        assert_eq!(doc.doc_type, DocumentType::SqlPattern);
        assert!(doc.metadata.is_object());
    }
}
