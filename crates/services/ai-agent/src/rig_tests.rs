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
        assert_eq!(config.agent_configs.sql_agent.name, "SQL专家");
        assert_eq!(config.agent_configs.analysis_agent.name, "数据分析师");
        assert_eq!(config.agent_configs.chat_agent.name, "智能助手");
        assert_eq!(config.agent_configs.recommendation_agent.name, "推荐专家");
        
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

    #[tokio::test]
    async fn test_tool_permission_manager() {
        use crate::rig_agent::ToolPermissionManager;

        let mut manager = ToolPermissionManager::new();

        // 测试默认权限设置
        assert!(manager.tool_acl.contains_key("database_query"));
        assert!(manager.tool_acl.contains_key("schema_inspector"));

        // 添加用户权限
        manager.user_permissions.insert(
            "admin_user".to_string(),
            vec!["admin".to_string()]
        );
        manager.user_permissions.insert(
            "analyst_user".to_string(),
            vec!["analyst".to_string()]
        );
        manager.user_permissions.insert(
            "regular_user".to_string(),
            vec!["user".to_string()]
        );

        // 测试权限检查
        assert!(manager.check_permission("admin_user", "database_query"));
        assert!(manager.check_permission("analyst_user", "database_query"));
        assert!(!manager.check_permission("regular_user", "database_query"));

        assert!(manager.check_permission("admin_user", "schema_inspector"));
        assert!(manager.check_permission("analyst_user", "schema_inspector"));
        assert!(manager.check_permission("regular_user", "schema_inspector"));
    }

    #[tokio::test]
    async fn test_tool_performance_monitor() {
        use crate::rig_agent::ToolPerformanceMonitor;

        let mut monitor = ToolPerformanceMonitor::new();

        // 记录工具调用
        monitor.record_call("database_query", 150);
        monitor.record_call("database_query", 200);
        monitor.record_call("database_query", 100);

        // 记录错误
        monitor.record_error("database_query");

        // 获取统计信息
        let stats = monitor.get_stats("database_query").unwrap();
        assert_eq!(stats.tool_name, "database_query");
        assert_eq!(stats.call_count, 3);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.avg_response_time_ms, 150); // (150+200+100)/3 = 150
        assert!((stats.success_rate - 66.67).abs() < 0.1); // 2/3 * 100 ≈ 66.67%
    }

    #[tokio::test]
    async fn test_security_config() {
        use crate::rig_agent::SecurityConfig;

        let config = SecurityConfig::default();

        // 验证默认安全配置
        assert!(config.enable_api_key_validation);
        assert_eq!(config.max_concurrent_requests, 100);
        assert_eq!(config.rate_limit_per_minute, 1000);
        assert!(config.enable_sql_injection_detection);
    }

    #[tokio::test]
    async fn test_performance_config() {
        use crate::rig_agent::PerformanceConfig;

        let config = PerformanceConfig::default();

        // 验证默认性能配置
        assert_eq!(config.connection_pool_size, 10);
        assert_eq!(config.request_timeout_seconds, 30);
        assert_eq!(config.cache_ttl_seconds, 300); // 5分钟
        assert!(config.enable_response_cache);
    }

    #[tokio::test]
    async fn test_agent_configs_structure() {
        use crate::rig_agent::AgentConfigs;

        let configs = AgentConfigs::default();

        // 验证所有Agent配置都存在
        assert_eq!(configs.sql_agent.name, "SQL专家");
        assert_eq!(configs.analysis_agent.name, "数据分析师");
        assert_eq!(configs.chat_agent.name, "智能助手");
        assert_eq!(configs.recommendation_agent.name, "推荐专家");

        // 验证温度设置合理
        assert_eq!(configs.sql_agent.temperature, 0.1); // 低温度确保准确性
        assert_eq!(configs.analysis_agent.temperature, 0.3); // 中等温度
        assert_eq!(configs.chat_agent.temperature, 0.7); // 高温度增加自然性
        assert_eq!(configs.recommendation_agent.temperature, 0.5); // 平衡温度

        // 验证所有Agent都启用了工具
        assert!(configs.sql_agent.enable_tools);
        assert!(configs.analysis_agent.enable_tools);
        assert!(configs.chat_agent.enable_tools);
        assert!(configs.recommendation_agent.enable_tools);
    }
}
