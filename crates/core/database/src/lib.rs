//! DuckHub Database Core
//! 
//! This crate provides the core database functionality for DuckHub, including:
//! - DuckDB connection pool management
//! - Query execution and optimization
//! - Data lake integration (DuckDB Lake)
//! - Schema management
//! - Caching layer

pub mod duckdb;
mod mock_duckdb;
pub mod pool;
pub mod query;
pub mod lake;
pub mod ducklake;
pub mod schema;
pub mod cache;
pub mod metrics;
pub mod extensions;

// Re-export main components
pub use duckdb::{DuckDBEngine, DuckDBConnection};
pub use pool::{ConnectionPool, PoolManager};
pub use query::{QueryExecutor, QueryOptimizer, QueryCache};
pub use lake::{DataLakeManager, ObjectStorageProvider};
pub use ducklake::{DuckLakeManager, DuckLakeConfig, DuckLakeDatabase, DuckLakeSnapshot};
pub use schema::{SchemaManager};
pub use cache::{QueryCacheImpl, CacheManager};
pub use extensions::{ExtensionManager, DataLakeFeature, S3Config, AzureConfig};

use duckhub_common::prelude::*;
