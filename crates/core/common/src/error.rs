//! Error types and handling for DuckHub


use thiserror::Error;

/// Main error type for DuckHub operations
#[derive(Error, Debug)]
pub enum DuckHubError {
    /// Database related errors
    #[error("Database error: {message}")]
    Database { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// IO errors
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Serialization errors
    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// Network/HTTP errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Authentication/Authorization errors
    #[error("Auth error: {message}")]
    Auth { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Cache errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    /// Plugin errors
    #[error("Plugin error: {message}")]
    Plugin { message: String },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// Not found errors
    #[error("Not found: {resource}")]
    NotFound { resource: String },

    /// Already exists errors
    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },

    /// Permission denied errors
    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Service unavailable
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
}

impl DuckHubError {
    /// Create a database error
    pub fn database<S: Into<String>>(message: S) -> Self {
        Self::Database {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create an auth error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create a plugin error
    pub fn plugin<S: Into<String>>(message: S) -> Self {
        Self::Plugin {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create an already exists error
    pub fn already_exists<S: Into<String>>(resource: S) -> Self {
        Self::AlreadyExists {
            resource: resource.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(action: S) -> Self {
        Self::PermissionDenied {
            action: action.into(),
        }
    }

    /// Create a service unavailable error
    pub fn service_unavailable<S: Into<String>>(service: S) -> Self {
        Self::ServiceUnavailable {
            service: service.into(),
        }
    }

    /// Create an IO error
    pub fn io<E: Into<std::io::Error>>(error: E) -> Self {
        Self::Io {
            source: error.into(),
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. } | Self::ServiceUnavailable { .. } | Self::RateLimitExceeded
        )
    }

    /// Get error category for metrics/logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::Database { .. } => "database",
            Self::Config { .. } => "config",
            Self::Io { .. } => "io",
            Self::Serialization { .. } => "serialization",
            Self::Network { .. } => "network",
            Self::Auth { .. } => "auth",
            Self::Validation { .. } => "validation",
            Self::Cache { .. } => "cache",
            Self::Plugin { .. } => "plugin",
            Self::Internal { .. } => "internal",
            Self::NotFound { .. } => "not_found",
            Self::AlreadyExists { .. } => "already_exists",
            Self::PermissionDenied { .. } => "permission_denied",
            Self::RateLimitExceeded => "rate_limit",
            Self::ServiceUnavailable { .. } => "service_unavailable",
        }
    }
}

/// Result type alias for DuckHub operations
pub type Result<T> = std::result::Result<T, DuckHubError>;

/// Convert from DuckDB errors
// Temporarily disabled due to arrow-arith conflicts
// impl From<::duckdb::Error> for DuckHubError {
//     fn from(err: ::duckdb::Error) -> Self {
//         Self::Database {
//             message: err.to_string(),
//         }
//     }
// }

/// Convert from Redis errors
impl From<::redis::RedisError> for DuckHubError {
    fn from(err: ::redis::RedisError) -> Self {
        Self::Cache {
            message: err.to_string(),
        }
    }
}

/// Convert from Prometheus errors
impl From<::prometheus::Error> for DuckHubError {
    fn from(err: ::prometheus::Error) -> Self {
        Self::Internal {
            message: format!("Prometheus error: {}", err),
        }
    }
}
