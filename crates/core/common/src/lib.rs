//! DuckHub Common Library
//! 
//! This crate provides common utilities, types, and traits used across the DuckHub platform.

pub mod error;
pub mod types;
pub mod utils;
pub mod traits;
pub mod config;

// Re-export commonly used types
pub use error::{DuckHubError, Result};
pub use types::*;
pub use traits::*;

/// Common prelude for DuckHub crates
pub mod prelude {
    pub use crate::error::{DuckHubError, Result};
    pub use crate::types::*;
    pub use crate::traits::*;
    pub use async_trait::async_trait;
    pub use serde::{Deserialize, Serialize};
    pub use tracing::{debug, error, info, trace, warn};
    pub use uuid::Uuid;
    pub use chrono::{DateTime, Utc};
}
