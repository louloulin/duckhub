//! 中间件模块

pub mod auth;
pub mod metrics;
pub mod rate_limit;
pub mod cors;

pub use auth::AuthMiddleware;
pub use metrics::MetricsMiddleware;
pub use rate_limit::RateLimitMiddleware;
pub use cors::CorsMiddleware;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
    HttpMessage,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// 请求ID中间件
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdMiddlewareService { service })
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // 生成请求ID
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // 将请求ID添加到请求扩展中
        req.extensions_mut().insert(request_id.clone());
        
        // 添加到响应头
        let fut = self.service.call(req);
        
        Box::pin(async move {
            let mut res = fut.await?;
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                actix_web::http::header::HeaderValue::from_str(&request_id).unwrap(),
            );
            Ok(res)
        })
    }
}

/// 日志中间件
pub struct LoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddlewareService { service })
    }
}

pub struct LoggingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddlewareService<S>
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
        let method = req.method().clone();
        let path = req.path().to_string();
        let request_id = req.extensions().get::<String>().cloned().unwrap_or_default();
        
        let fut = self.service.call(req);
        
        Box::pin(async move {
            let res = fut.await?;
            let duration = start_time.elapsed();
            let status = res.status();
            
            tracing::info!(
                request_id = %request_id,
                method = %method,
                path = %path,
                status = %status.as_u16(),
                duration_ms = %duration.as_millis(),
                "HTTP请求完成"
            );
            
            Ok(res)
        })
    }
}

/// 错误处理中间件
pub struct ErrorHandlingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ErrorHandlingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ErrorHandlingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorHandlingMiddlewareService { service })
    }
}

pub struct ErrorHandlingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorHandlingMiddlewareService<S>
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
        let fut = self.service.call(req);
        
        Box::pin(async move {
            match fut.await {
                Ok(res) => Ok(res),
                Err(err) => {
                    tracing::error!("请求处理错误: {}", err);
                    
                    // 这里可以根据错误类型返回不同的响应
                    // 目前简单地传播错误
                    Err(err)
                }
            }
        })
    }
}
