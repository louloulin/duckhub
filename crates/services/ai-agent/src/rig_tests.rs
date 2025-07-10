//! Rig AI Agent测试

#[cfg(test)]
mod tests {
    use super::super::rig_agent::*;
    use duckhub_database::DuckDBEngine;
    use duckhub_query_analytics::QueryAnalyticsService;
    use prometheus::Registry;
    use std::sync::Arc;
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
}
