//! Utility functions for DuckHub

use crate::{DuckHubError, Result};
use chrono::{DateTime, Utc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Generate a new UUID v4
pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Convert SystemTime to DateTime<Utc>
pub fn system_time_to_datetime(time: SystemTime) -> Result<DateTime<Utc>> {
    let duration = time.duration_since(UNIX_EPOCH)
        .map_err(|e| DuckHubError::internal(format!("Invalid system time: {}", e)))?;
    
    let datetime = DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
        .ok_or_else(|| DuckHubError::internal("Failed to convert timestamp"))?;
    
    Ok(datetime)
}

/// Convert Duration to milliseconds
pub fn duration_to_ms(duration: Duration) -> u64 {
    duration.as_millis() as u64
}

/// Validate SQL query (basic validation)
pub fn validate_sql(sql: &str) -> Result<()> {
    let sql = sql.trim().to_lowercase();
    
    if sql.is_empty() {
        return Err(DuckHubError::validation("SQL query cannot be empty"));
    }
    
    // Check for dangerous operations in production
    let dangerous_keywords = ["drop", "delete", "truncate", "alter"];
    for keyword in dangerous_keywords {
        if sql.starts_with(keyword) {
            return Err(DuckHubError::validation(format!(
                "Dangerous SQL operation '{}' is not allowed", keyword
            )));
        }
    }
    
    Ok(())
}

/// Sanitize table/column names
pub fn sanitize_identifier(name: &str) -> Result<String> {
    if name.is_empty() {
        return Err(DuckHubError::validation("Identifier cannot be empty"));
    }
    
    // Check for valid identifier characters
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(DuckHubError::validation(
            "Identifier can only contain alphanumeric characters and underscores"
        ));
    }
    
    // Check if starts with letter or underscore
    if !name.chars().next().unwrap().is_alphabetic() && !name.starts_with('_') {
        return Err(DuckHubError::validation(
            "Identifier must start with a letter or underscore"
        ));
    }
    
    Ok(name.to_lowercase())
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration to human readable string
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let milliseconds = duration.subsec_millis();
    
    if total_seconds == 0 {
        return format!("{}ms", milliseconds);
    }
    
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    let mut parts = Vec::new();
    
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        if milliseconds > 0 {
            parts.push(format!("{}.{:03}s", seconds, milliseconds));
        } else {
            parts.push(format!("{}s", seconds));
        }
    }
    
    parts.join(" ")
}

/// Calculate hash of a string (for caching keys)
pub fn calculate_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Retry logic with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);
                
                if attempt < max_retries {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, Duration::from_secs(60));
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| DuckHubError::internal("Retry failed without error")))
}

/// Parse connection string into components
pub fn parse_connection_string(conn_str: &str) -> Result<ConnectionComponents> {
    let url = url::Url::parse(conn_str)
        .map_err(|e| DuckHubError::config(format!("Invalid connection string: {}", e)))?;
    
    Ok(ConnectionComponents {
        scheme: url.scheme().to_string(),
        host: url.host_str().unwrap_or("localhost").to_string(),
        port: url.port(),
        database: url.path().trim_start_matches('/').to_string(),
        username: if url.username().is_empty() { None } else { Some(url.username().to_string()) },
        password: url.password().map(|p| p.to_string()),
        query_params: url.query_pairs().into_owned().collect(),
    })
}

/// Connection string components
#[derive(Debug, Clone)]
pub struct ConnectionComponents {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub query_params: std::collections::HashMap<String, String>,
}

/// Environment variable helpers
pub fn get_env_var(key: &str) -> Result<String> {
    std::env::var(key)
        .map_err(|_| DuckHubError::config(format!("Environment variable '{}' not found", key)))
}

pub fn get_env_var_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// JSON utilities
pub fn to_json_string<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|e| DuckHubError::internal(format!("JSON serialization failed: {}", e)))
}

pub fn from_json_string<T: serde::de::DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json)
        .map_err(|e| DuckHubError::internal(format!("JSON deserialization failed: {}", e)))
}

/// File utilities
pub async fn read_file_to_string(path: &std::path::Path) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| DuckHubError::io(e))
}

pub async fn write_string_to_file(path: &std::path::Path, content: &str) -> Result<()> {
    tokio::fs::write(path, content)
        .await
        .map_err(|e| DuckHubError::io(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }
    
    #[test]
    fn test_sanitize_identifier() {
        assert!(sanitize_identifier("valid_name").is_ok());
        assert!(sanitize_identifier("_valid").is_ok());
        assert!(sanitize_identifier("123invalid").is_err());
        assert!(sanitize_identifier("invalid-name").is_err());
        assert!(sanitize_identifier("").is_err());
    }
    
    #[test]
    fn test_validate_sql() {
        assert!(validate_sql("SELECT * FROM table").is_ok());
        assert!(validate_sql("drop table users").is_err());
        assert!(validate_sql("").is_err());
    }
}
