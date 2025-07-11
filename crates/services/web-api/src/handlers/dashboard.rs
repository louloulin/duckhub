// 仪表板API处理器
// 提供系统指标、查询趋势、性能数据等仪表板功能

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use chrono::{DateTime, Utc, Duration, Timelike};
use std::collections::HashMap;
use crate::{AppState, success_response, error_response};

/// 仪表板核心指标
#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_queries_today: u64,
    pub avg_response_time_ms: f64,
    pub active_connections: u32,
    pub cache_hit_rate: f64,
    pub system_cpu_usage: f64,
    pub system_memory_usage: f64,
    pub disk_usage: f64,
    pub error_rate: f64,
    pub total_tables: u32,
    pub total_rows: u64,
    pub data_size_gb: f64,
    pub active_users: u32,
    pub last_updated: String,
}

/// 查询趋势数据点
#[derive(Debug, Serialize)]
pub struct QueryTrendPoint {
    pub timestamp: String,
    pub query_count: u64,
    pub avg_response_time: f64,
    pub error_count: u64,
    pub cache_hits: u64,
}

/// 查询趋势响应
#[derive(Debug, Serialize)]
pub struct QueryTrendsResponse {
    pub time_range: String,
    pub data_points: Vec<QueryTrendPoint>,
    pub summary: QueryTrendSummary,
}

/// 查询趋势汇总
#[derive(Debug, Serialize)]
pub struct QueryTrendSummary {
    pub total_queries: u64,
    pub avg_response_time: f64,
    pub peak_qps: u64,
    pub peak_time: String,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
}

/// 系统健康状态
#[derive(Debug, Serialize)]
pub struct SystemHealthDashboard {
    pub overall_status: String, // "healthy", "warning", "critical"
    pub components: Vec<ComponentHealth>,
    pub alerts: Vec<SystemAlert>,
    pub uptime_seconds: u64,
    pub last_restart: String,
    pub version: String,
}

/// 组件健康状态
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String, // "healthy", "warning", "critical"
    pub message: String,
    pub last_check: String,
    pub response_time_ms: Option<f64>,
}

/// 系统告警
#[derive(Debug, Serialize)]
pub struct SystemAlert {
    pub id: String,
    pub level: String, // "info", "warning", "error", "critical"
    pub title: String,
    pub message: String,
    pub timestamp: String,
    pub resolved: bool,
}

/// 时间范围查询参数
#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    pub range: Option<String>, // "1h", "6h", "24h", "7d", "30d"
}

/// 获取仪表板核心指标
#[instrument(skip(app_state))]
pub async fn get_dashboard_metrics(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取仪表板核心指标");
    
    // 模拟实时指标数据
    let metrics = DashboardMetrics {
        total_queries_today: 15_847,
        avg_response_time_ms: 85.3,
        active_connections: 28,
        cache_hit_rate: 87.5,
        system_cpu_usage: 42.8,
        system_memory_usage: 68.2,
        disk_usage: 45.6,
        error_rate: 0.12,
        total_tables: 12,
        total_rows: 6_875_000,
        data_size_gb: 2.8,
        active_users: 156,
        last_updated: Utc::now().to_rfc3339(),
    };
    
    info!("成功获取仪表板指标，今日查询数: {}", metrics.total_queries_today);
    Ok(success_response(metrics))
}

/// 获取查询趋势数据
#[instrument(skip(app_state))]
pub async fn get_query_trends(
    app_state: web::Data<AppState>,
    query: web::Query<TimeRangeQuery>
) -> ActixResult<HttpResponse> {
    let time_range = query.range.as_deref().unwrap_or("24h");
    info!("获取查询趋势数据，时间范围: {}", time_range);
    
    // 根据时间范围生成不同的数据点
    let (data_points, interval_minutes) = match time_range {
        "1h" => (generate_trend_data(12, 5), 5),   // 12个点，每5分钟
        "6h" => (generate_trend_data(24, 15), 15), // 24个点，每15分钟
        "24h" => (generate_trend_data(24, 60), 60), // 24个点，每小时
        "7d" => (generate_trend_data(28, 360), 360), // 28个点，每6小时
        "30d" => (generate_trend_data(30, 1440), 1440), // 30个点，每天
        _ => (generate_trend_data(24, 60), 60), // 默认24小时
    };
    
    // 计算汇总统计
    let total_queries: u64 = data_points.iter().map(|p| p.query_count).sum();
    let avg_response_time = data_points.iter().map(|p| p.avg_response_time).sum::<f64>() / data_points.len() as f64;
    let total_errors: u64 = data_points.iter().map(|p| p.error_count).sum();
    let total_cache_hits: u64 = data_points.iter().map(|p| p.cache_hits).sum();
    
    let peak_point = data_points.iter().max_by_key(|p| p.query_count).unwrap();
    
    let summary = QueryTrendSummary {
        total_queries,
        avg_response_time,
        peak_qps: peak_point.query_count,
        peak_time: peak_point.timestamp.clone(),
        error_rate: if total_queries > 0 { (total_errors as f64 / total_queries as f64) * 100.0 } else { 0.0 },
        cache_hit_rate: if total_queries > 0 { (total_cache_hits as f64 / total_queries as f64) * 100.0 } else { 0.0 },
    };
    
    let response = QueryTrendsResponse {
        time_range: time_range.to_string(),
        data_points,
        summary,
    };
    
    info!("成功获取查询趋势数据，包含 {} 个数据点", response.data_points.len());
    Ok(success_response(response))
}

/// 获取系统健康状态
#[instrument(skip(app_state))]
pub async fn get_system_health_dashboard(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取系统健康状态");
    
    let now = Utc::now();
    
    // 模拟组件健康状态
    let components = vec![
        ComponentHealth {
            name: "DuckDB数据库".to_string(),
            status: "healthy".to_string(),
            message: "数据库连接正常，查询响应良好".to_string(),
            last_check: now.to_rfc3339(),
            response_time_ms: Some(12.5),
        },
        ComponentHealth {
            name: "缓存系统".to_string(),
            status: "healthy".to_string(),
            message: "缓存命中率87.5%，性能良好".to_string(),
            last_check: now.to_rfc3339(),
            response_time_ms: Some(2.1),
        },
        ComponentHealth {
            name: "AI Agent服务".to_string(),
            status: "healthy".to_string(),
            message: "AI服务响应正常".to_string(),
            last_check: now.to_rfc3339(),
            response_time_ms: Some(156.8),
        },
        ComponentHealth {
            name: "监控系统".to_string(),
            status: "warning".to_string(),
            message: "磁盘使用率较高(85%)，建议清理".to_string(),
            last_check: now.to_rfc3339(),
            response_time_ms: Some(8.3),
        },
        ComponentHealth {
            name: "数据采集服务".to_string(),
            status: "healthy".to_string(),
            message: "数据采集正常，处理延迟低".to_string(),
            last_check: now.to_rfc3339(),
            response_time_ms: Some(45.2),
        },
    ];
    
    // 模拟系统告警
    let alerts = vec![
        SystemAlert {
            id: "alert-001".to_string(),
            level: "warning".to_string(),
            title: "磁盘空间不足".to_string(),
            message: "系统磁盘使用率已达到85%，建议清理日志文件".to_string(),
            timestamp: (now - Duration::hours(2)).to_rfc3339(),
            resolved: false,
        },
        SystemAlert {
            id: "alert-002".to_string(),
            level: "info".to_string(),
            title: "缓存清理完成".to_string(),
            message: "定期缓存清理任务已完成，释放了1.2GB空间".to_string(),
            timestamp: (now - Duration::hours(6)).to_rfc3339(),
            resolved: true,
        },
    ];
    
    // 确定整体状态
    let overall_status = if components.iter().any(|c| c.status == "critical") {
        "critical"
    } else if components.iter().any(|c| c.status == "warning") {
        "warning"
    } else {
        "healthy"
    };
    
    let health = SystemHealthDashboard {
        overall_status: overall_status.to_string(),
        components,
        alerts,
        uptime_seconds: 2_847_392, // 约33天
        last_restart: "2024-12-08T10:23:57Z".to_string(),
        version: "1.0.0".to_string(),
    };
    
    info!("成功获取系统健康状态，整体状态: {}", health.overall_status);
    Ok(success_response(health))
}

/// 生成趋势数据的辅助函数
fn generate_trend_data(points: usize, interval_minutes: i64) -> Vec<QueryTrendPoint> {
    let mut data_points = Vec::new();
    let now = Utc::now();
    
    for i in 0..points {
        let timestamp = now - Duration::minutes(interval_minutes * (points - i - 1) as i64);
        
        // 模拟查询量波动（工作时间更高）
        let hour = timestamp.hour();
        let base_queries = if hour >= 9 && hour <= 17 { 800 } else { 200 };
        let query_count = base_queries + (i * 50) % 300;
        
        // 模拟响应时间波动
        let avg_response_time = 50.0 + (i as f64 * 10.0) % 100.0;
        
        // 模拟错误和缓存命中
        let error_count = query_count / 100; // 1%错误率
        let cache_hits = (query_count as f64 * 0.85) as u64; // 85%缓存命中率
        
        data_points.push(QueryTrendPoint {
            timestamp: timestamp.to_rfc3339(),
            query_count: query_count as u64,
            avg_response_time,
            error_count: error_count as u64,
            cache_hits,
        });
    }
    
    data_points
}
