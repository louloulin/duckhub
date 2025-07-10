//! Configuration management for DuckHub

use crate::{ConfigProvider, DuckHubError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub features: FeatureFlags,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub max_connections: usize,
    pub max_connection_rate: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            keep_alive: 75,
            client_timeout: 5000,
            client_shutdown: 5000,
            max_connections: 25000,
            max_connection_rate: 256,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub duckdb_path: String,
    pub memory_limit: Option<String>,
    pub threads: Option<usize>,
    pub max_memory: Option<String>,
    pub temp_directory: Option<String>,
    pub extensions: Vec<String>,
    pub pool: PoolConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub pool_size: u32,
    pub connection_timeout: u64,
    pub default_ttl: u64,
    pub max_key_size: usize,
    pub max_value_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            connection_timeout: 5,
            default_ttl: 3600,
            max_key_size: 1024,
            max_value_size: 1024 * 1024, // 1MB
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_path: String,
    pub temp_path: String,
    pub max_file_size: u64,
    pub compression: CompressionConfig,
    pub object_storage: Option<ObjectStorageConfig>,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub level: i32,
}

/// Object storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    pub provider: String,
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub prefix: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
    pub structured: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
            file_path: None,
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            max_files: Some(10),
            structured: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub password_min_length: usize,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub tls: Option<TlsConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u32,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: Option<String>,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub ai_agent: bool,
    pub plugins: bool,
    pub real_time: bool,
    pub caching: bool,
    pub metrics: bool,
    pub tracing: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            ai_agent: false,
            plugins: false,
            real_time: true,
            caching: true,
            metrics: true,
            tracing: true,
        }
    }
}

/// Configuration provider implementation
pub struct ConfigManager {
    config: AppConfig,
    values: HashMap<String, Value>,
}

impl ConfigManager {
    /// Load configuration from file and environment
    pub fn load() -> Result<Self> {
        let mut config_builder = config::Config::builder();
        
        // Load from default configuration
        config_builder = config_builder.add_source(
            config::File::from_str(include_str!("../config/default.toml"), config::FileFormat::Toml)
        );
        
        // Load from environment-specific config if exists
        if let Ok(env) = std::env::var("DUCKHUB_ENV") {
            let config_file = format!("config/{}.toml", env);
            if std::path::Path::new(&config_file).exists() {
                config_builder = config_builder.add_source(config::File::with_name(&config_file));
            }
        }
        
        // Override with environment variables
        config_builder = config_builder.add_source(
            config::Environment::with_prefix("DUCKHUB")
                .separator("_")
                .try_parsing(true)
        );
        
        let settings = config_builder.build()
            .map_err(|e| DuckHubError::config(format!("Failed to load configuration: {}", e)))?;
        
        let config: AppConfig = settings.try_deserialize()
            .map_err(|e| DuckHubError::config(format!("Failed to deserialize configuration: {}", e)))?;
        
        let values = settings.try_deserialize::<HashMap<String, Value>>()
            .map_err(|e| DuckHubError::config(format!("Failed to get configuration values: {}", e)))?;
        
        Ok(Self { config, values })
    }
    
    /// Get the full application configuration
    pub fn app_config(&self) -> &AppConfig {
        &self.config
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.config.server.port == 0 {
            return Err(DuckHubError::config("Server port cannot be 0"));
        }
        
        // Validate database configuration
        if self.config.database.duckdb_path.is_empty() {
            return Err(DuckHubError::config("DuckDB path cannot be empty"));
        }
        
        // Validate cache configuration
        if self.config.cache.pool_size == 0 {
            return Err(DuckHubError::config("Cache pool size cannot be 0"));
        }
        
        // Validate security configuration
        if self.config.security.jwt_secret.len() < 32 {
            return Err(DuckHubError::config("JWT secret must be at least 32 characters"));
        }
        
        Ok(())
    }
}

impl ConfigProvider for ConfigManager {
    fn get<T>(&self, key: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.values.get(key)
            .ok_or_else(|| DuckHubError::config(format!("Configuration key '{}' not found", key)))
            .and_then(|value| {
                serde_json::from_value(value.clone())
                    .map_err(|e| DuckHubError::config(format!("Failed to deserialize config value: {}", e)))
            })
    }
    
    fn get_or_default<T>(&self, key: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.get(key).unwrap_or(default)
    }
    
    fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
    
    fn get_all(&self) -> Result<HashMap<String, Value>> {
        Ok(self.values.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_configs() {
        let server_config = ServerConfig::default();
        assert_eq!(server_config.port, 8080);
        assert_eq!(server_config.host, "0.0.0.0");
        
        let pool_config = PoolConfig::default();
        assert_eq!(pool_config.max_connections, 10);
        
        let logging_config = LoggingConfig::default();
        assert_eq!(logging_config.level, "info");
        
        let features = FeatureFlags::default();
        assert!(features.caching);
        assert!(features.metrics);
    }
}
