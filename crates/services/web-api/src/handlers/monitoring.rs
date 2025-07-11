//! 监控处理器 - 为前端Dashboard提供监控数据

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use tracing::{info, error, instrument};
use crate::{AppState, success_response, error_response};

/// Dashboard数据响应
#[derive(Debug, Serialize)]
pub struct DashboardData {
    /// 核心指标
    pub metrics: CoreMetrics,
    /// 查询趋势
    pub query_trends: Vec<QueryTrendPoint>,
    /// 系统健康状态
    pub system_health: SystemHealth,
    /// 最近活动
    pub recent_activities: Vec<Activity>,
    /// 性能统计
    pub performance_stats: PerformanceStats,
}

/// 核心指标
#[derive(Debug, Serialize)]
pub struct CoreMetrics {
    /// 总查询数
    pub total_queries: u64,
    /// 活跃连接数
    pub active_connections: u32,
    /// 数据处理量（MB）
    pub data_processed_mb: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
}

/// 查询趋势点
#[derive(Debug, Serialize)]
pub struct QueryTrendPoint {
    /// 时间戳
    pub timestamp: String,
    /// 查询数量
    pub value: u32,
}

/// 系统健康状态
#[derive(Debug, Serialize)]
pub struct SystemHealth {
    /// 整体状态
    pub overall_status: String,
    /// 组件状态
    pub components: Vec<ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub name: String,
    /// 状态
    pub status: String,
    /// 使用率百分比
    pub usage_percentage: f64,
    /// 响应时间（毫秒）
    pub response_time_ms: Option<f64>,
}

/// 活动记录
#[derive(Debug, Serialize)]
pub struct Activity {
    /// 活动ID
    pub id: String,
    /// 活动类型
    pub activity_type: String,
    /// 描述
    pub description: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 严重程度
    pub severity: String,
}

/// 性能统计
#[derive(Debug, Serialize)]
pub struct PerformanceStats {
    /// 查询性能分布
    pub query_performance_distribution: Vec<PerformanceDistribution>,
    /// 资源使用情况
    pub resource_usage: ResourceUsage,
}

/// 性能分布
#[derive(Debug, Serialize)]
pub struct PerformanceDistribution {
    /// 时间范围
    pub time_range: String,
    /// 查询数量
    pub count: u32,
    /// 百分比
    pub percentage: f64,
}

/// 资源使用情况
#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    /// CPU使用率
    pub cpu_usage: f64,
    /// 内存使用率
    pub memory_usage: f64,
    /// 磁盘使用率
    pub disk_usage: f64,
    /// 网络IO
    pub network_io: NetworkIO,
}

/// 网络IO
#[derive(Debug, Serialize)]
pub struct NetworkIO {
    /// 入站流量（MB/s）
    pub inbound_mbps: f64,
    /// 出站流量（MB/s）
    pub outbound_mbps: f64,
}

/// 获取Dashboard数据
#[instrument(skip(app_state))]
pub async fn get_dashboard_data(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取Dashboard数据");

    // 获取核心指标
    let metrics = match get_core_metrics(&app_state).await {
        Ok(m) => m,
        Err(e) => {
            error!("获取核心指标失败: {}", e);
            return Ok(error_response("获取核心指标失败", 500));
        }
    };

    // 获取查询趋势
    let query_trends = match get_query_trends(&app_state).await {
        Ok(t) => t,
        Err(e) => {
            error!("获取查询趋势失败: {}", e);
            Vec::new()
        }
    };

    // 获取系统健康状态
    let system_health = match get_system_health(&app_state).await {
        Ok(h) => h,
        Err(e) => {
            error!("获取系统健康状态失败: {}", e);
            SystemHealth {
                overall_status: "unknown".to_string(),
                components: Vec::new(),
            }
        }
    };

    // 获取最近活动
    let recent_activities = match get_recent_activities(&app_state).await {
        Ok(a) => a,
        Err(e) => {
            error!("获取最近活动失败: {}", e);
            Vec::new()
        }
    };

    // 获取性能统计
    let performance_stats = match get_performance_stats(&app_state).await {
        Ok(p) => p,
        Err(e) => {
            error!("获取性能统计失败: {}", e);
            PerformanceStats {
                query_performance_distribution: Vec::new(),
                resource_usage: ResourceUsage {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    disk_usage: 0.0,
                    network_io: NetworkIO {
                        inbound_mbps: 0.0,
                        outbound_mbps: 0.0,
                    },
                },
            }
        }
    };

    let dashboard_data = DashboardData {
        metrics,
        query_trends,
        system_health,
        recent_activities,
        performance_stats,
    };

    Ok(success_response(dashboard_data))
}

/// 获取核心指标
async fn get_core_metrics(app_state: &AppState) -> Result<CoreMetrics> {
    // 从监控服务获取指标
    let monitoring_metrics = app_state.monitoring_service.get_metrics().await?;
    
    Ok(CoreMetrics {
        total_queries: monitoring_metrics.total_queries,
        active_connections: monitoring_metrics.active_connections,
        data_processed_mb: monitoring_metrics.data_processed_bytes as f64 / 1024.0 / 1024.0,
        avg_response_time_ms: monitoring_metrics.avg_response_time_ms,
    })
}

/// 获取查询趋势
async fn get_query_trends(app_state: &AppState) -> Result<Vec<QueryTrendPoint>> {
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(24);
    
    let trends = app_state.analytics_service
        .get_query_trends(start_time, end_time, Duration::hours(1))
        .await?;
    
    Ok(trends.into_iter().map(|trend| QueryTrendPoint {
        timestamp: trend.timestamp.format("%H:%M").to_string(),
        value: trend.count,
    }).collect())
}

/// 获取系统健康状态
async fn get_system_health(app_state: &AppState) -> Result<SystemHealth> {
    let health_status = app_state.monitoring_service.get_health_status().await?;
    
    let components = vec![
        ComponentHealth {
            name: "CPU".to_string(),
            status: if health_status.cpu_usage < 80.0 { "healthy" } else { "warning" }.to_string(),
            usage_percentage: health_status.cpu_usage,
            response_time_ms: None,
        },
        ComponentHealth {
            name: "内存".to_string(),
            status: if health_status.memory_usage < 80.0 { "healthy" } else { "warning" }.to_string(),
            usage_percentage: health_status.memory_usage,
            response_time_ms: None,
        },
        ComponentHealth {
            name: "磁盘".to_string(),
            status: if health_status.disk_usage < 80.0 { "healthy" } else { "warning" }.to_string(),
            usage_percentage: health_status.disk_usage,
            response_time_ms: None,
        },
        ComponentHealth {
            name: "数据库".to_string(),
            status: "healthy".to_string(),
            usage_percentage: 45.0,
            response_time_ms: Some(5.2),
        },
    ];

    let overall_status = if components.iter().all(|c| c.status == "healthy") {
        "healthy"
    } else if components.iter().any(|c| c.status == "critical") {
        "critical"
    } else {
        "warning"
    };

    Ok(SystemHealth {
        overall_status: overall_status.to_string(),
        components,
    })
}

/// 获取最近活动
async fn get_recent_activities(app_state: &AppState) -> Result<Vec<Activity>> {
    let activities = app_state.monitoring_service.get_recent_activities(10).await?;
    
    Ok(activities.into_iter().map(|activity| Activity {
        id: activity.id.to_string(),
        activity_type: activity.activity_type,
        description: activity.description,
        timestamp: activity.timestamp,
        severity: activity.severity,
    }).collect())
}

/// 获取性能统计
async fn get_performance_stats(app_state: &AppState) -> Result<PerformanceStats> {
    let perf_data = app_state.analytics_service.get_performance_statistics().await?;
    
    let query_performance_distribution = vec![
        PerformanceDistribution {
            time_range: "< 100ms".to_string(),
            count: perf_data.fast_queries,
            percentage: (perf_data.fast_queries as f64 / perf_data.total_queries as f64) * 100.0,
        },
        PerformanceDistribution {
            time_range: "100ms - 1s".to_string(),
            count: perf_data.medium_queries,
            percentage: (perf_data.medium_queries as f64 / perf_data.total_queries as f64) * 100.0,
        },
        PerformanceDistribution {
            time_range: "> 1s".to_string(),
            count: perf_data.slow_queries,
            percentage: (perf_data.slow_queries as f64 / perf_data.total_queries as f64) * 100.0,
        },
    ];

    let resource_usage = ResourceUsage {
        cpu_usage: perf_data.cpu_usage,
        memory_usage: perf_data.memory_usage,
        disk_usage: perf_data.disk_usage,
        network_io: NetworkIO {
            inbound_mbps: perf_data.network_in_mbps,
            outbound_mbps: perf_data.network_out_mbps,
        },
    };

    Ok(PerformanceStats {
        query_performance_distribution,
        resource_usage,
    })
}

/// 获取告警信息
#[instrument(skip(app_state))]
pub async fn get_alerts(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取告警信息");

    match app_state.monitoring_service.get_active_alerts().await {
        Ok(alerts) => Ok(success_response(alerts)),
        Err(e) => {
            error!("获取告警信息失败: {}", e);
            Ok(error_response("获取告警信息失败", 500))
        }
    }
}

/// 获取性能指标
#[instrument(skip(app_state))]
pub async fn get_performance_metrics(app_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    info!("获取性能指标");

    match app_state.monitoring_service.get_performance_metrics().await {
        Ok(metrics) => Ok(success_response(metrics)),
        Err(e) => {
            error!("获取性能指标失败: {}", e);
            Ok(error_response("获取性能指标失败", 500))
        }
    }
}
