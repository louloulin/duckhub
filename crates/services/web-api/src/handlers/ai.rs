//! AI Agent处理器

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use validator::Validate;
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
        context: request.context.clone(),
    };

    match app_state.ai_service.chat(chat_request).await {
        Ok(response) => {
            let response_time = start_time.elapsed();
            
            let ai_response = AIChatResponse {
                response: response.message,
                session_id: response.session_id,
                suggested_sql: response.suggested_sql,
                confidence: response.confidence,
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
        content: request.content.clone(),
        analysis_type: match request.analysis_type {
            AnalysisType::SqlAnalysis => duckhub_ai_agent::AnalysisType::SqlAnalysis,
            AnalysisType::SchemaAnalysis => duckhub_ai_agent::AnalysisType::SchemaAnalysis,
            AnalysisType::PerformanceAnalysis => duckhub_ai_agent::AnalysisType::PerformanceAnalysis,
            AnalysisType::AnomalyDetection => duckhub_ai_agent::AnalysisType::AnomalyDetection,
        },
        parameters: request.parameters.clone(),
    };

    match app_state.ai_service.analyze(analysis_request).await {
        Ok(analysis) => {
            let analysis_time = start_time.elapsed();
            
            let ai_response = AIAnalyzeResponse {
                analysis: analysis.result,
                recommendations: analysis.recommendations,
                confidence: analysis.confidence,
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
        description: request.description.clone(),
        tables: request.tables.clone().unwrap_or_default(),
        query_type: request.query_type.as_ref().map(|qt| match qt {
            QueryType::Select => duckhub_ai_agent::QueryType::Select,
            QueryType::Aggregate => duckhub_ai_agent::QueryType::Aggregate,
            QueryType::Join => duckhub_ai_agent::QueryType::Join,
            QueryType::Analytics => duckhub_ai_agent::QueryType::Analytics,
        }),
    };

    match app_state.ai_service.suggest(suggest_request).await {
        Ok(suggestions) => {
            let suggested_queries: Vec<SuggestedQuery> = suggestions.queries.into_iter().map(|q| {
                SuggestedQuery {
                    sql: q.sql,
                    description: q.description,
                    complexity: q.complexity,
                    estimated_time: q.estimated_time,
                }
            }).collect();

            let ai_response = AISuggestResponse {
                suggested_queries,
                explanation: suggestions.explanation,
                confidence: suggestions.confidence,
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
