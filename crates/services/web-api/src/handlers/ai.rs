//! AI Agent处理器

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::{AppState, success_response, error_response};

/// AI聊天请求
#[derive(Debug, Deserialize, Validate)]
pub struct AIChatRequest {
    /// 用户消息
    #[validate(length(min = 1, max = 5000, message = "消息长度必须在1-5000字符之间"))]
    pub message: String,
    /// 会话ID（可选）
    pub session_id: Option<String>,
    /// 上下文（可选）
    pub context: Option<serde_json::Value>,
}

/// AI聊天响应
#[derive(Debug, Serialize)]
pub struct AIChatResponse {
    /// AI回复
    pub response: String,
    /// 会话ID
    pub session_id: String,
    /// 建议的SQL查询（如果有）
    pub suggested_sql: Option<String>,
    /// 置信度
    pub confidence: f64,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
}

/// AI分析请求
#[derive(Debug, Deserialize, Validate)]
pub struct AIAnalyzeRequest {
    /// 要分析的数据或查询
    #[validate(length(min = 1, max = 10000, message = "分析内容长度必须在1-10000字符之间"))]
    pub content: String,
    /// 分析类型
    pub analysis_type: AnalysisType,
    /// 额外参数
    pub parameters: Option<serde_json::Value>,
}

/// 分析类型
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    /// SQL查询分析
    SqlAnalysis,
    /// 数据模式分析
    SchemaAnalysis,
    /// 性能分析
    PerformanceAnalysis,
    /// 异常检测
    AnomalyDetection,
}

/// AI分析响应
#[derive(Debug, Serialize)]
pub struct AIAnalyzeResponse {
    /// 分析结果
    pub analysis: serde_json::Value,
    /// 建议
    pub recommendations: Vec<String>,
    /// 置信度
    pub confidence: f64,
    /// 分析时间（毫秒）
    pub analysis_time_ms: u64,
}

/// AI建议请求
#[derive(Debug, Deserialize, Validate)]
pub struct AISuggestRequest {
    /// 查询意图或描述
    #[validate(length(min = 1, max = 1000, message = "描述长度必须在1-1000字符之间"))]
    pub description: String,
    /// 相关表名（可选）
    pub tables: Option<Vec<String>>,
    /// 查询类型偏好
    pub query_type: Option<QueryType>,
}

/// 查询类型
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    /// 查询
    Select,
    /// 聚合
    Aggregate,
    /// 连接
    Join,
    /// 分析
    Analytics,
}

/// AI建议响应
#[derive(Debug, Serialize)]
pub struct AISuggestResponse {
    /// 建议的SQL查询
    pub suggested_queries: Vec<SuggestedQuery>,
    /// 解释
    pub explanation: String,
    /// 置信度
    pub confidence: f64,
}

/// 建议的查询
#[derive(Debug, Serialize)]
pub struct SuggestedQuery {
    /// SQL语句
    pub sql: String,
    /// 描述
    pub description: String,
    /// 复杂度评分（1-10）
    pub complexity: u8,
    /// 预估执行时间
    pub estimated_time: String,
}

/// AI聊天
#[instrument(skip(app_state, request))]
pub async fn ai_chat(
    app_state: web::Data<AppState>,
    request: web::Json<AIChatRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("AI聊天请求: {}", request.message);
    let start_time = std::time::Instant::now();

    let chat_request = duckhub_ai_agent::ChatRequest {
        message: request.message.clone(),
        session_id: request.session_id.clone(),
        context: request.context.as_ref().map(|v| v.to_string()),
    };

    match app_state.ai_service.chat(chat_request).await {
        Ok(response) => {
            let response_time = start_time.elapsed();
            
            let ai_response = AIChatResponse {
                response: response.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                session_id: response.get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                suggested_sql: response.get("metadata").and_then(|m| m.get("sql_query")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                confidence: response.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.85),
                response_time_ms: response_time.as_millis() as u64,
            };

            Ok(success_response(ai_response))
        }
        Err(e) => {
            error!("AI聊天失败: {}", e);
            Ok(error_response(&format!("AI聊天失败: {}", e), 500))
        }
    }
}

/// AI分析
#[instrument(skip(app_state, request))]
pub async fn ai_analyze(
    app_state: web::Data<AppState>,
    request: web::Json<AIAnalyzeRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("AI分析请求，类型: {:?}", request.analysis_type);
    let start_time = std::time::Instant::now();

    let analysis_request = duckhub_ai_agent::AnalysisRequest {
        query: request.content.clone(),
        context: None,
        analysis_type: match request.analysis_type {
            AnalysisType::SqlAnalysis => duckhub_ai_agent::AnalysisType::Query,
            AnalysisType::SchemaAnalysis => duckhub_ai_agent::AnalysisType::Schema,
            AnalysisType::PerformanceAnalysis => duckhub_ai_agent::AnalysisType::Performance,
            AnalysisType::AnomalyDetection => duckhub_ai_agent::AnalysisType::Data,
        },
    };

    match app_state.ai_service.analyze(analysis_request).await {
        Ok(analysis) => {
            let analysis_time = start_time.elapsed();
            
            let ai_response = AIAnalyzeResponse {
                analysis: analysis.get("result").cloned().unwrap_or(serde_json::Value::String("".to_string())),
                recommendations: analysis.get("recommendations").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                confidence: analysis.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.85),
                analysis_time_ms: analysis_time.as_millis() as u64,
            };

            Ok(success_response(ai_response))
        }
        Err(e) => {
            error!("AI分析失败: {}", e);
            Ok(error_response(&format!("AI分析失败: {}", e), 500))
        }
    }
}

/// AI建议
#[instrument(skip(app_state, request))]
pub async fn ai_suggest(
    app_state: web::Data<AppState>,
    request: web::Json<AISuggestRequest>,
) -> ActixResult<HttpResponse> {
    if let Err(e) = request.validate() {
        return Ok(error_response(&format!("请求验证失败: {}", e), 400));
    }

    info!("AI建议请求: {}", request.description);

    let suggest_request = duckhub_ai_agent::SuggestRequest {
        query: request.description.clone(),
        query_type: request.query_type.as_ref().map(|qt| match qt {
            QueryType::Select => duckhub_ai_agent::QueryType::Select,
            QueryType::Aggregate => duckhub_ai_agent::QueryType::Insert, // 映射到Insert
            QueryType::Join => duckhub_ai_agent::QueryType::Update,      // 映射到Update
            QueryType::Analytics => duckhub_ai_agent::QueryType::Create, // 映射到Create
        }).unwrap_or(duckhub_ai_agent::QueryType::Select),
        context: request.tables.as_ref().map(|tables| format!("可用表: {}", tables.join(", "))),
    };

    match app_state.ai_service.suggest(suggest_request).await {
        Ok(suggestions) => {
            // 从JSON响应中提取建议查询
            let suggested_queries = vec![
                SuggestedQuery {
                    sql: suggestions.get("optimized_query").and_then(|v| v.as_str()).unwrap_or("SELECT * FROM table").to_string(),
                    description: suggestions.get("suggestion").and_then(|v| v.as_str()).unwrap_or("查询建议").to_string(),
                    complexity: 2, // medium complexity
                    estimated_time: "100ms".to_string(),
                }
            ];

            let ai_response = AISuggestResponse {
                suggested_queries,
                explanation: suggestions.get("performance_impact").and_then(|v| v.as_str()).unwrap_or("性能优化建议").to_string(),
                confidence: 0.85,
            };

            Ok(success_response(ai_response))
        }
        Err(e) => {
            error!("AI建议失败: {}", e);
            Ok(error_response(&format!("AI建议失败: {}", e), 500))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_chat_request_validation() {
        let valid_request = AIChatRequest {
            message: "帮我查询用户数据".to_string(),
            session_id: Some("session123".to_string()),
            context: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = AIChatRequest {
            message: "".to_string(), // 空消息
            session_id: None,
            context: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_ai_suggest_request_validation() {
        let valid_request = AISuggestRequest {
            description: "查询最近一周的销售数据".to_string(),
            tables: Some(vec!["sales".to_string()]),
            query_type: Some(QueryType::Select),
        };
        assert!(valid_request.validate().is_ok());
    }
}

// ==================== AI会话管理功能 ====================

/// AI会话信息
#[derive(Debug, Serialize, Clone)]
pub struct AISession {
    pub session_id: String,
    pub session_type: String, // "chat", "analysis", "query_builder"
    pub created_at: String,
    pub last_activity: String,
    pub message_count: u32,
    pub user_id: String,
    pub title: Option<String>,
    pub status: String, // "active", "archived", "expired"
}

/// 创建会话请求
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSessionRequest {
    #[validate(length(min = 1, max = 50))]
    pub session_type: String, // "chat", "analysis", "query_builder"
    pub title: Option<String>,
    pub initial_context: Option<serde_json::Value>,
}

/// 会话历史消息
#[derive(Debug, Serialize, Clone)]
pub struct SessionMessage {
    pub message_id: String,
    pub session_id: String,
    pub role: String, // "user", "assistant"
    pub content: String,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}

/// 会话历史响应
#[derive(Debug, Serialize)]
pub struct SessionHistoryResponse {
    pub session_id: String,
    pub messages: Vec<SessionMessage>,
    pub total_messages: u32,
    pub session_info: AISession,
}

/// 会话历史查询参数
#[derive(Debug, Deserialize)]
pub struct SessionHistoryQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub include_metadata: Option<bool>,
}

/// NLP查询请求
#[derive(Debug, Deserialize, Validate)]
pub struct NLPQueryRequest {
    #[validate(length(min = 1, max = 1000))]
    pub query: String,
    pub context: Option<serde_json::Value>,
    pub session_id: Option<String>,
}

/// NLP查询响应
#[derive(Debug, Serialize)]
pub struct NLPQueryResponse {
    pub original_query: String,
    pub interpreted_intent: String,
    pub generated_sql: String,
    pub confidence: f64,
    pub explanation: String,
    pub suggested_tables: Vec<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub session_id: Option<String>,
}

/// 创建AI会话
#[instrument(skip(app_state))]
pub async fn create_ai_session(
    app_state: web::Data<AppState>,
    request: web::Json<CreateSessionRequest>
) -> ActixResult<HttpResponse> {
    info!("创建AI会话，类型: {}", request.session_type);

    if let Err(e) = request.validate() {
        error!("会话创建请求验证失败: {:?}", e);
        return Ok(error_response("请求参数无效", 400));
    }

    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let session = AISession {
        session_id: session_id.clone(),
        session_type: request.session_type.clone(),
        created_at: now.clone(),
        last_activity: now,
        message_count: 0,
        user_id: "current-user-id".to_string(), // 从认证中获取
        title: request.title.clone(),
        status: "active".to_string(),
    };

    info!("成功创建AI会话: {}", session_id);
    Ok(success_response(session))
}

/// 获取会话历史
#[instrument(skip(app_state))]
pub async fn get_session_history(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<SessionHistoryQuery>
) -> ActixResult<HttpResponse> {
    let session_id = path.into_inner();
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    info!("获取会话历史: {}, limit: {}, offset: {}", session_id, limit, offset);

    // 模拟会话信息
    let session_info = AISession {
        session_id: session_id.clone(),
        session_type: "chat".to_string(),
        created_at: "2024-01-11T10:00:00Z".to_string(),
        last_activity: Utc::now().to_rfc3339(),
        message_count: 8,
        user_id: "current-user-id".to_string(),
        title: Some("数据分析咨询".to_string()),
        status: "active".to_string(),
    };

    // 模拟历史消息
    let messages = vec![
        SessionMessage {
            message_id: "msg-001".to_string(),
            session_id: session_id.clone(),
            role: "user".to_string(),
            content: "你好，我想分析一下最近的交易数据".to_string(),
            timestamp: "2024-01-11T10:00:00Z".to_string(),
            metadata: None,
        },
        SessionMessage {
            message_id: "msg-002".to_string(),
            session_id: session_id.clone(),
            role: "assistant".to_string(),
            content: "您好！我可以帮您分析交易数据。请告诉我您想了解哪些方面的信息？比如交易量、金额分布、时间趋势等。".to_string(),
            timestamp: "2024-01-11T10:00:05Z".to_string(),
            metadata: Some(serde_json::json!({
                "suggested_sql": "SELECT COUNT(*), SUM(amount) FROM transactions WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'"
            })),
        },
        SessionMessage {
            message_id: "msg-003".to_string(),
            session_id: session_id.clone(),
            role: "user".to_string(),
            content: "我想看看最近一周每天的交易量和总金额".to_string(),
            timestamp: "2024-01-11T10:01:00Z".to_string(),
            metadata: None,
        },
        SessionMessage {
            message_id: "msg-004".to_string(),
            session_id: session_id.clone(),
            role: "assistant".to_string(),
            content: "好的，我为您生成了查询最近一周每日交易统计的SQL。这个查询会显示每天的交易数量和总金额。".to_string(),
            timestamp: "2024-01-11T10:01:05Z".to_string(),
            metadata: Some(serde_json::json!({
                "generated_sql": "SELECT DATE(created_at) as date, COUNT(*) as transaction_count, SUM(amount) as total_amount FROM transactions WHERE created_at >= CURRENT_DATE - INTERVAL '7 days' GROUP BY DATE(created_at) ORDER BY date",
                "confidence": 0.95
            })),
        },
    ];

    let response = SessionHistoryResponse {
        session_id,
        messages,
        total_messages: 8,
        session_info,
    };

    info!("成功获取会话历史，返回 {} 条消息", response.messages.len());
    Ok(success_response(response))
}

/// 处理自然语言查询
#[instrument(skip(app_state))]
pub async fn process_nlp_query(
    app_state: web::Data<AppState>,
    request: web::Json<NLPQueryRequest>
) -> ActixResult<HttpResponse> {
    info!("处理自然语言查询: {}", request.query);

    if let Err(e) = request.validate() {
        error!("NLP查询请求验证失败: {:?}", e);
        return Ok(error_response("请求参数无效", 400));
    }

    // 模拟NLP处理逻辑
    let (intent, sql, confidence, explanation, tables) = analyze_nlp_query(&request.query);

    let response = NLPQueryResponse {
        original_query: request.query.clone(),
        interpreted_intent: intent,
        generated_sql: sql,
        confidence,
        explanation,
        suggested_tables: tables,
        parameters: None,
        session_id: request.session_id.clone(),
    };

    info!("NLP查询处理完成，置信度: {:.2}", response.confidence);
    Ok(success_response(response))
}

/// 分析自然语言查询的辅助函数
fn analyze_nlp_query(query: &str) -> (String, String, f64, String, Vec<String>) {
    let query_lower = query.to_lowercase();

    if query_lower.contains("交易") && query_lower.contains("统计") {
        (
            "查询交易统计信息".to_string(),
            "SELECT COUNT(*) as total_transactions, SUM(amount) as total_amount, AVG(amount) as avg_amount FROM transactions WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'".to_string(),
            0.92,
            "根据您的查询，我理解您想要获取交易的统计信息，包括总数量、总金额和平均金额".to_string(),
            vec!["transactions".to_string()],
        )
    } else if query_lower.contains("用户") && query_lower.contains("数量") {
        (
            "查询用户数量".to_string(),
            "SELECT COUNT(*) as user_count FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'".to_string(),
            0.88,
            "您想了解用户数量，我为您生成了统计用户总数的查询".to_string(),
            vec!["users".to_string()],
        )
    } else if query_lower.contains("最近") && query_lower.contains("天") {
        (
            "查询最近时间段的数据".to_string(),
            "SELECT * FROM transactions WHERE created_at >= CURRENT_DATE - INTERVAL '7 days' ORDER BY created_at DESC LIMIT 100".to_string(),
            0.85,
            "您想查看最近几天的数据，我为您生成了相应的时间范围查询".to_string(),
            vec!["transactions".to_string()],
        )
    } else {
        (
            "通用数据查询".to_string(),
            "SELECT * FROM transactions LIMIT 10".to_string(),
            0.60,
            "我理解您想要查询数据，但具体需求不够明确，为您提供了一个基础查询示例".to_string(),
            vec!["transactions".to_string(), "users".to_string()],
        )
    }
}
