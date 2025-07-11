//! 指标收集中间件

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use prometheus::{Counter, Histogram, IntCounter, Registry};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;

/// 指标收集中间件
pub struct MetricsMiddleware {
    registry: Arc<Registry>,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Registry::new()),
        }
    }

    pub fn with_registry(registry: Arc<Registry>) -> Self {
        Self { registry }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let metrics = WebApiMetrics::new(&self.registry);
        ok(MetricsMiddlewareService { service, metrics })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
    metrics: WebApiMetrics,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = std::time::Instant::now();
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let metrics = self.metrics.clone();
        
        // 增加请求计数
        metrics.requests_total.inc();
        metrics.requests_in_flight.inc();

        let fut = self.service.call(req);
        
        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();
            
            // 减少进行中的请求计数
            metrics.requests_in_flight.dec();
            
            match &result {
                Ok(response) => {
                    let status = response.status().as_u16().to_string();
                    
                    // 记录请求持续时间
                    metrics.request_duration
                        .with_label_values(&[&method, &path, &status])
                        .observe(duration.as_secs_f64());
                    
                    // 按状态码计数
                    if response.status().is_success() {
                        metrics.requests_success.inc();
                    } else if response.status().is_client_error() {
                        metrics.requests_client_error.inc();
                    } else if response.status().is_server_error() {
                        metrics.requests_server_error.inc();
                    }
                }
                Err(_) => {
                    // 记录错误
                    metrics.requests_server_error.inc();
                    metrics.request_duration
                        .with_label_values(&[&method, &path, "500"])
                        .observe(duration.as_secs_f64());
                }
            }
            
            result
        })
    }
}

/// Web API指标
#[derive(Clone)]
struct WebApiMetrics {
    /// 总请求数
    requests_total: IntCounter,
    /// 成功请求数
    requests_success: IntCounter,
    /// 客户端错误请求数
    requests_client_error: IntCounter,
    /// 服务器错误请求数
    requests_server_error: IntCounter,
    /// 进行中的请求数
    requests_in_flight: IntCounter,
    /// 请求持续时间直方图
    request_duration: Histogram,
}

impl WebApiMetrics {
    fn new(registry: &Registry) -> Self {
        let requests_total = IntCounter::new(
            "duckhub_api_requests_total",
            "API请求总数"
        ).unwrap();
        
        let requests_success = IntCounter::new(
            "duckhub_api_requests_success_total",
            "API成功请求总数"
        ).unwrap();
        
        let requests_client_error = IntCounter::new(
            "duckhub_api_requests_client_error_total",
            "API客户端错误请求总数"
        ).unwrap();
        
        let requests_server_error = IntCounter::new(
            "duckhub_api_requests_server_error_total",
            "API服务器错误请求总数"
        ).unwrap();
        
        let requests_in_flight = IntCounter::new(
            "duckhub_api_requests_in_flight",
            "进行中的API请求数"
        ).unwrap();
        
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_api_request_duration_seconds",
                "API请求持续时间（秒）"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0])
        ).unwrap();
        
        // 注册指标
        registry.register(Box::new(requests_total.clone())).unwrap();
        registry.register(Box::new(requests_success.clone())).unwrap();
        registry.register(Box::new(requests_client_error.clone())).unwrap();
        registry.register(Box::new(requests_server_error.clone())).unwrap();
        registry.register(Box::new(requests_in_flight.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        
        Self {
            requests_total,
            requests_success,
            requests_client_error,
            requests_server_error,
            requests_in_flight,
            request_duration,
        }
    }
}

/// 业务指标收集器
pub struct BusinessMetrics {
    /// 查询执行次数
    pub queries_executed: IntCounter,
    /// 查询执行时间
    pub query_duration: Histogram,
    /// 数据处理量
    pub data_processed_bytes: Counter,
    /// 活跃用户数
    pub active_users: IntCounter,
    /// 缓存命中次数
    pub cache_hits: IntCounter,
    /// 缓存未命中次数
    pub cache_misses: IntCounter,
}

impl BusinessMetrics {
    pub fn new(registry: &Registry) -> Self {
        let queries_executed = IntCounter::new(
            "duckhub_queries_executed_total",
            "执行的查询总数"
        ).unwrap();
        
        let query_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "duckhub_query_duration_seconds",
                "查询执行时间（秒）"
            ).buckets(vec![0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0])
        ).unwrap();
        
        let data_processed_bytes = Counter::new(
            "duckhub_data_processed_bytes_total",
            "处理的数据总量（字节）"
        ).unwrap();
        
        let active_users = IntCounter::new(
            "duckhub_active_users",
            "活跃用户数"
        ).unwrap();
        
        let cache_hits = IntCounter::new(
            "duckhub_cache_hits_total",
            "缓存命中总数"
        ).unwrap();
        
        let cache_misses = IntCounter::new(
            "duckhub_cache_misses_total",
            "缓存未命中总数"
        ).unwrap();
        
        // 注册指标
        registry.register(Box::new(queries_executed.clone())).unwrap();
        registry.register(Box::new(query_duration.clone())).unwrap();
        registry.register(Box::new(data_processed_bytes.clone())).unwrap();
        registry.register(Box::new(active_users.clone())).unwrap();
        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();
        
        Self {
            queries_executed,
            query_duration,
            data_processed_bytes,
            active_users,
            cache_hits,
            cache_misses,
        }
    }
    
    /// 记录查询执行
    pub fn record_query_execution(&self, duration: std::time::Duration, data_size: u64) {
        self.queries_executed.inc();
        self.query_duration.observe(duration.as_secs_f64());
        self.data_processed_bytes.inc_by(data_size as f64);
    }
    
    /// 记录缓存命中
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }
    
    /// 记录缓存未命中
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }
    
    /// 记录活跃用户
    pub fn record_active_user(&self) {
        self.active_users.inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus::Registry;

    #[test]
    fn test_business_metrics_creation() {
        let registry = Registry::new();
        let metrics = BusinessMetrics::new(&registry);
        
        // 测试指标记录
        metrics.record_query_execution(std::time::Duration::from_millis(100), 1024);
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.record_active_user();
        
        // 验证指标值
        assert_eq!(metrics.queries_executed.get(), 1);
        assert_eq!(metrics.cache_hits.get(), 1);
        assert_eq!(metrics.cache_misses.get(), 1);
        assert_eq!(metrics.active_users.get(), 1);
    }
}
