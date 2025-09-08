//! DuckHub Database Core
//! 
//! This crate provides the core database functionality for DuckHub, including:
//! - DuckDB connection pool management
//! - Query execution and optimization
//! - Data lake integration (DuckDB Lake)
//! - Schema management
//! - Caching layer

pub mod duckdb;
pub mod real_duckdb;
pub mod ducklake_real;  // New real DuckLake implementation
pub mod stream_processor;  // Enhanced stream processing engine
pub mod risk_engine;  // Real-time risk monitoring engine
pub mod memory_optimization;  // Memory pool and object pool optimization
pub mod realtime_financial_processor;  // Integrated real-time financial processor
pub mod pool;
pub mod query;
pub mod lake;
// 已移除 ducklake 模拟实现，使用 ducklake_real 真实实现
// pub mod ducklake_simple;  // 简化版本已被真实实现替代
pub mod schema;
pub mod cache;
pub mod metrics;
// pub mod extensions;  // Temporarily disabled due to compilation issues

// Re-export main components
pub use duckdb::{DuckDBEngine, QueryResult};
pub use pool::{ConnectionPool, PoolManager};
pub use query::{QueryExecutor, QueryOptimizer, QueryCache};
pub use lake::{DataLakeManager, ObjectStorageProvider};
// 已移除 ducklake_simple 导出，统一使用 ducklake_real
pub use ducklake_real::{DuckLakeManager, DuckLakeConfig, DuckLakeDatabase, DuckLakeMetrics};
pub use stream_processor::{StreamProcessor, StreamProcessorConfig, StreamRecord, ProcessingResult};
pub use risk_engine::{RealTimeRiskEngine, RiskEngineConfig, PortfolioUpdate, RiskMetrics, RiskAlert};
pub use memory_optimization::{MemoryPool, MemoryManager, ObjectPool, BufferPool, StringPool};
pub use realtime_financial_processor::{RealTimeFinancialProcessor, RealTimeProcessorConfig, FinancialData, TradeData};
pub use real_duckdb::{Connection, ConnectionConfig};
pub use schema::{SchemaManager};
pub use cache::{QueryCacheImpl, CacheManager};
// pub use extensions::{ExtensionManager, DataLakeFeature, S3Config, AzureConfig};  // Temporarily disabled

use duckhub_common::prelude::*;

#[cfg(test)]
mod tests;
