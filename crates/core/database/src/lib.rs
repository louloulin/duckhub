//! DuckHub Database Core
//! 
//! This crate provides the core database functionality for DuckHub, including:
//! - DuckDB connection pool management
//! - Query execution and optimization
//! - Data lake integration (DuckDB Lake)
//! - Schema management
//! - Caching layer

pub mod duckdb;
mod real_duckdb;
pub mod pool;
pub mod query;
pub mod lake;
// pub mod ducklake;  // Temporarily disabled due to compilation issues
pub mod ducklake_simple;
pub mod schema;
pub mod cache;
pub mod metrics;
// pub mod extensions;  // Temporarily disabled due to compilation issues

// Re-export main components
pub use duckdb::{DuckDBEngine, QueryResult};
pub use pool::{ConnectionPool, PoolManager};
pub use query::{QueryExecutor, QueryOptimizer, QueryCache};
pub use lake::{DataLakeManager, ObjectStorageProvider};
pub use ducklake_simple::{DuckLakeManager, DuckLakeConfig, DuckLakeDatabase};
pub use schema::{SchemaManager};
pub use cache::{QueryCacheImpl, CacheManager};
// pub use extensions::{ExtensionManager, DataLakeFeature, S3Config, AzureConfig};  // Temporarily disabled

use duckhub_common::prelude::*;
