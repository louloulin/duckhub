//! AI服务测试

use duckhub_ai_agent::{AIAgentService, ChatRequest, AnalysisRequest, SuggestRequest, AnalysisType, QueryType};
use duckhub_common::prelude::*;
use tokio;

#[tokio::test]
async fn test_ai_chat() {
    // 创建AI服务实例
    let ai_service = create_test_ai_service().await;

    let chat_request = ChatRequest {
        message: "帮我分析一下数据库性能".to_string(),
        context: Some("financial_data数据库".to_string()),
        session_id: Some("test_session_123".to_string()),
    };

    // 测试聊天功能
    let response = ai_service.chat(chat_request).await.expect("AI聊天失败");

    // 验证响应格式
    assert!(response.get("response").is_some(), "响应应该包含response字段");
    assert!(response.get("session_id").is_some(), "响应应该包含session_id字段");
    assert!(response.get("timestamp").is_some(), "响应应该包含timestamp字段");

    let response_text = response.get("response").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!response_text.is_empty(), "响应内容不应该为空");
}

#[tokio::test]
async fn test_ai_analysis() {
    let ai_service = create_test_ai_service().await;

    // 测试性能分析
    let analysis_request = AnalysisRequest {
        query: "SELECT * FROM transactions WHERE amount > 1000".to_string(),
        analysis_type: AnalysisType::Performance,
        context: Some("优化查询性能".to_string()),
    };

    let response = ai_service.analyze(analysis_request).await.expect("AI分析失败");

    // 验证分析响应
    assert!(response.get("analysis_type").is_some(), "响应应该包含analysis_type字段");
    assert!(response.get("result").is_some(), "响应应该包含result字段");
    assert!(response.get("recommendations").is_some(), "响应应该包含recommendations字段");
    assert!(response.get("confidence").is_some(), "响应应该包含confidence字段");

    let confidence = response.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
    assert!(confidence > 0.0 && confidence <= 1.0, "置信度应该在0-1之间");
}

#[tokio::test]
async fn test_ai_schema_analysis() {
    let ai_service = create_test_ai_service().await;

    let schema_analysis = AnalysisRequest {
        query: "ALTER TABLE transactions ADD COLUMN status VARCHAR(20)".to_string(),
        analysis_type: AnalysisType::Schema,
        context: Some("添加状态字段".to_string()),
    };

    let response = ai_service.analyze(schema_analysis).await.expect("Schema分析失败");

    let result = response.get("result").and_then(|v| v.as_str()).unwrap_or("");
    assert!(result.contains("Schema"), "Schema分析结果应该包含Schema相关内容");
}

#[tokio::test]
async fn test_ai_suggestions() {
    let ai_service = create_test_ai_service().await;

    let suggest_request = SuggestRequest {
        query: "查询所有高价值交易".to_string(),
        query_type: QueryType::Select,
        context: Some("transactions表包含amount字段".to_string()),
    };

    let response = ai_service.suggest(suggest_request).await.expect("AI建议失败");

    // 验证建议响应
    assert!(response.get("query_type").is_some(), "响应应该包含query_type字段");
    assert!(response.get("suggestion").is_some(), "响应应该包含suggestion字段");
    assert!(response.get("optimized_query").is_some(), "响应应该包含optimized_query字段");
    assert!(response.get("performance_impact").is_some(), "响应应该包含performance_impact字段");

    let optimized_query = response.get("optimized_query").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!optimized_query.is_empty(), "优化查询不应该为空");
}

#[tokio::test]
async fn test_ai_different_query_types() {
    let ai_service = create_test_ai_service().await;

    let query_types = vec![
        QueryType::Select,
        QueryType::Insert,
        QueryType::Update,
        QueryType::Delete,
        QueryType::Create,
        QueryType::Alter,
    ];

    for query_type in query_types {
        let suggest_request = SuggestRequest {
            query: format!("测试{:?}类型查询", query_type),
            query_type: query_type.clone(),
            context: None,
        };

        let response = ai_service.suggest(suggest_request).await
            .expect(&format!("AI建议失败，查询类型: {:?}", query_type));

        assert!(response.get("suggestion").is_some(), 
                "每种查询类型都应该有建议，当前类型: {:?}", query_type);
    }
}

#[tokio::test]
async fn test_ai_analysis_types() {
    let ai_service = create_test_ai_service().await;

    let analysis_types = vec![
        AnalysisType::Performance,
        AnalysisType::Schema,
        AnalysisType::Query,
        AnalysisType::Data,
    ];

    for analysis_type in analysis_types {
        let analysis_request = AnalysisRequest {
            query: format!("测试{:?}类型分析", analysis_type),
            analysis_type: analysis_type.clone(),
            context: None,
        };

        let response = ai_service.analyze(analysis_request).await
            .expect(&format!("AI分析失败，分析类型: {:?}", analysis_type));

        assert!(response.get("result").is_some(), 
                "每种分析类型都应该有结果，当前类型: {:?}", analysis_type);
    }
}

#[tokio::test]
async fn test_ai_service_health() {
    let ai_service = create_test_ai_service().await;

    // 测试健康检查
    let health = ai_service.health_check().await.expect("AI服务健康检查失败");
    
    // 验证健康状态
    match health {
        duckhub_ai_agent::HealthStatus::Healthy => {
            // 健康状态正常
        },
        _ => panic!("AI服务应该是健康的"),
    }
}

#[tokio::test]
async fn test_ai_service_stats() {
    let ai_service = create_test_ai_service().await;

    // 先进行一些操作
    let chat_request = ChatRequest {
        message: "测试统计".to_string(),
        context: None,
        session_id: None,
    };
    
    let _ = ai_service.chat(chat_request).await.expect("AI聊天失败");

    // 获取统计信息
    let stats = ai_service.get_stats().await.expect("获取AI服务统计失败");

    // 验证统计信息
    assert!(stats.contains_key("chat_messages_total"), "统计应该包含聊天消息总数");
    assert!(stats.contains_key("active_sessions"), "统计应该包含活跃会话数");
}

// 辅助函数：创建测试用的AI服务
async fn create_test_ai_service() -> AIAgentService {
    // 这里应该创建一个测试配置的AI服务
    // 由于实际的AI服务可能需要复杂的初始化，这里使用模拟配置
    
    // 注意：这个函数需要根据实际的AIAgentService构造函数来实现
    // 目前先返回一个占位符，实际测试时需要根据具体实现调整
    
    // 创建测试配置
    let config = duckhub_ai_agent::AIConfig {
        model_name: "test-model".to_string(),
        api_key: "test-key".to_string(),
        max_tokens: 1000,
        temperature: 0.7,
        timeout_seconds: 30,
    };

    AIAgentService::new(config).await.expect("创建AI服务失败")
}
