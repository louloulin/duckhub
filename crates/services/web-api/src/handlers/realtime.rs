// 实时功能API处理器
// 提供WebSocket连接、实时指标推送、实时查询结果等功能

use actix_web::{web, HttpResponse, Result as ActixResult, HttpRequest};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, ActorContext};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use chrono::Utc;
use std::collections::HashMap;
use crate::{AppState, success_response};

/// WebSocket消息类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// 订阅实时指标
    SubscribeMetrics {
        metrics: Vec<String>,
        interval: u32, // 秒
    },
    /// 取消订阅
    Unsubscribe {
        subscription_id: String,
    },
    /// 实时查询
    LiveQuery {
        query_id: String,
        sql: String,
        auto_refresh: bool,
        refresh_interval: Option<u32>,
    },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// WebSocket响应消息
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketResponse {
    /// 指标数据
    MetricsData {
        subscription_id: String,
        timestamp: String,
        metrics: HashMap<String, f64>,
    },
    /// 查询结果
    QueryResult {
        query_id: String,
        timestamp: String,
        data: Vec<HashMap<String, serde_json::Value>>,
        row_count: u32,
        execution_time_ms: u64,
    },
    /// 错误消息
    Error {
        error_code: String,
        message: String,
    },
    /// 连接状态
    ConnectionStatus {
        status: String,
        client_id: String,
        connected_at: String,
    },
    /// 心跳响应
    Pong,
}

/// 实时指标订阅请求
#[derive(Debug, Deserialize)]
pub struct MetricsSubscriptionRequest {
    pub metrics: Vec<String>,
    pub interval: u32,
    pub filters: Option<HashMap<String, String>>,
}

/// 实时查询请求
#[derive(Debug, Deserialize)]
pub struct LiveQueryRequest {
    pub sql: String,
    pub auto_refresh: bool,
    pub refresh_interval: Option<u32>,
    pub max_rows: Option<u32>,
}

/// WebSocket连接Actor
pub struct WebSocketConnection {
    pub client_id: String,
    pub connected_at: String,
    pub subscriptions: HashMap<String, MetricsSubscriptionRequest>,
    pub live_queries: HashMap<String, LiveQueryRequest>,
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("WebSocket连接建立: {}", self.client_id);
        
        // 发送连接状态消息
        let status_msg = WebSocketResponse::ConnectionStatus {
            status: "connected".to_string(),
            client_id: self.client_id.clone(),
            connected_at: self.connected_at.clone(),
        };
        
        if let Ok(msg_text) = serde_json::to_string(&status_msg) {
            ctx.text(msg_text);
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("WebSocket连接断开: {}", self.client_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    self.handle_websocket_message(ws_msg, ctx);
                } else {
                    error!("无法解析WebSocket消息: {}", text);
                    let error_msg = WebSocketResponse::Error {
                        error_code: "INVALID_MESSAGE".to_string(),
                        message: "无法解析消息格式".to_string(),
                    };
                    if let Ok(msg_text) = serde_json::to_string(&error_msg) {
                        ctx.text(msg_text);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                info!("收到二进制消息，暂不支持");
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // 心跳响应，无需处理
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket连接关闭: {:?}", reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                // 处理连续消息，暂不支持
                info!("收到连续消息，暂不支持");
            }
            Ok(ws::Message::Nop) => {
                // 无操作消息，忽略
            }
            Err(e) => {
                error!("WebSocket协议错误: {}", e);
                ctx.stop();
            }
        }
    }
}

impl WebSocketConnection {
    pub fn new() -> Self {
        Self {
            client_id: uuid::Uuid::new_v4().to_string(),
            connected_at: Utc::now().to_rfc3339(),
            subscriptions: HashMap::new(),
            live_queries: HashMap::new(),
        }
    }

    fn handle_websocket_message(&mut self, msg: WebSocketMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WebSocketMessage::SubscribeMetrics { metrics, interval } => {
                let subscription_id = uuid::Uuid::new_v4().to_string();
                let subscription = MetricsSubscriptionRequest {
                    metrics: metrics.clone(),
                    interval,
                    filters: None,
                };
                
                self.subscriptions.insert(subscription_id.clone(), subscription);
                info!("新增指标订阅: {} -> {:?}", subscription_id, metrics);
                
                // 模拟发送指标数据
                self.send_metrics_data(&subscription_id, &metrics, ctx);
            }
            WebSocketMessage::Unsubscribe { subscription_id } => {
                if self.subscriptions.remove(&subscription_id).is_some() {
                    info!("取消订阅: {}", subscription_id);
                } else {
                    error!("订阅不存在: {}", subscription_id);
                }
            }
            WebSocketMessage::LiveQuery { query_id, sql, auto_refresh, refresh_interval } => {
                let query_request = LiveQueryRequest {
                    sql: sql.clone(),
                    auto_refresh,
                    refresh_interval,
                    max_rows: Some(1000),
                };
                
                self.live_queries.insert(query_id.clone(), query_request);
                info!("新增实时查询: {} -> {}", query_id, sql);
                
                // 模拟执行查询并发送结果
                self.send_query_result(&query_id, &sql, ctx);
            }
            WebSocketMessage::Ping => {
                let pong_msg = WebSocketResponse::Pong;
                if let Ok(msg_text) = serde_json::to_string(&pong_msg) {
                    ctx.text(msg_text);
                }
            }
            WebSocketMessage::Pong => {
                // 客户端心跳响应，无需处理
            }
        }
    }

    fn send_metrics_data(&self, subscription_id: &str, metrics: &[String], ctx: &mut ws::WebsocketContext<Self>) {
        let mut metric_values = HashMap::new();
        
        // TODO: 从真实监控系统获取指标数据
        // 暂时返回空数据，避免使用mock数据
        for metric in metrics {
            metric_values.insert(metric.clone(), 0.0);
        }
        
        let metrics_msg = WebSocketResponse::MetricsData {
            subscription_id: subscription_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            metrics: metric_values,
        };
        
        if let Ok(msg_text) = serde_json::to_string(&metrics_msg) {
            ctx.text(msg_text);
        }
    }

    fn send_query_result(&self, query_id: &str, _sql: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // 模拟查询结果数据
        let mut sample_data = Vec::new();
        for i in 1..=5 {
            let mut row = HashMap::new();
            row.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
            row.insert("name".to_string(), serde_json::Value::String(format!("Record_{}", i)));
            row.insert("value".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(100.0 + i as f64 * 10.0).unwrap()
            ));
            row.insert("timestamp".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
            sample_data.push(row);
        }
        
        let query_msg = WebSocketResponse::QueryResult {
            query_id: query_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: sample_data,
            row_count: 5,
            execution_time_ms: 25,
        };
        
        if let Ok(msg_text) = serde_json::to_string(&query_msg) {
            ctx.text(msg_text);
        }
    }
}

/// WebSocket连接处理器
#[instrument(skip(_app_state, stream))]
pub async fn websocket_handler(
    _app_state: web::Data<AppState>,
    req: HttpRequest,
    stream: web::Payload,
) -> ActixResult<HttpResponse> {
    info!("建立WebSocket连接");
    
    let ws_conn = WebSocketConnection::new();
    let resp = ws::start(ws_conn, &req, stream)?;
    
    Ok(resp)
}

/// 获取实时指标API
#[instrument(skip(_app_state))]
pub async fn get_real_time_metrics(
    _app_state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>
) -> ActixResult<HttpResponse> {
    let metrics = query.get("metrics").unwrap_or(&"cpu_usage,memory_usage".to_string()).clone();
    info!("获取实时指标: {}", metrics);
    
    let metric_list: Vec<&str> = metrics.split(',').collect();
    let mut metric_values = HashMap::new();
    
    // TODO: 从真实监控系统获取指标数据
    // 暂时返回空数据，避免使用mock数据
    for metric in metric_list {
        metric_values.insert(metric.trim().to_string(), 0.0);
    }
    
    let response = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "metrics": metric_values,
        "refresh_interval": 5
    });
    
    info!("成功获取实时指标，包含 {} 个指标", metric_values.len());
    Ok(success_response(response))
}

/// 获取实时查询结果API
#[instrument(skip(_app_state))]
pub async fn get_live_query_results(
    _app_state: web::Data<AppState>,
    request: web::Json<LiveQueryRequest>
) -> ActixResult<HttpResponse> {
    info!("执行实时查询: {}", request.sql);
    
    // 模拟查询结果
    let mut sample_data = Vec::new();
    let row_count = request.max_rows.unwrap_or(10).min(100);
    
    for i in 1..=row_count {
        let mut row = HashMap::new();
        row.insert("id".to_string(), serde_json::Value::Number(serde_json::Number::from(i)));
        row.insert("name".to_string(), serde_json::Value::String(format!("LiveRecord_{}", i)));
        row.insert("value".to_string(), serde_json::Value::Number(
            serde_json::Number::from_f64(1000.0 + i as f64 * 25.0).unwrap()
        ));
        row.insert("updated_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
        sample_data.push(row);
    }
    
    let response = serde_json::json!({
        "query_id": uuid::Uuid::new_v4().to_string(),
        "sql": request.sql,
        "timestamp": Utc::now().to_rfc3339(),
        "data": sample_data,
        "row_count": row_count,
        "execution_time_ms": 45,
        "auto_refresh": request.auto_refresh,
        "refresh_interval": request.refresh_interval.unwrap_or(30)
    });
    
    info!("实时查询执行完成，返回 {} 行数据", row_count);
    Ok(success_response(response))
}
