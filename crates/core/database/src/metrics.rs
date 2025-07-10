//! Database metrics collection and monitoring

use duckhub_common::prelude::*;
use prometheus::{Counter, Histogram, Gauge, Registry, Opts, HistogramOpts};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error};

/// Database metrics collector
pub struct DatabaseMetrics {
    registry: Registry,
    
    // Query metrics
    query_total: Counter,
    query_duration: Histogram,
    query_errors: Counter,
    query_cache_hits: Counter,
    query_cache_misses: Counter,
    
    // Connection metrics
    connections_active: Gauge,
    connections_idle: Gauge,
    connections_total: Counter,
    connection_errors: Counter,
    
    // Database metrics
    database_size: Gauge,
    table_count: Gauge,
    
    // Performance metrics
    rows_scanned: Counter,
    rows_returned: Counter,
    bytes_scanned: Counter,
    bytes_returned: Counter,
}

impl DatabaseMetrics {
    /// Create a new database metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Query metrics
        let query_total = Counter::with_opts(Opts::new(
            "duckhub_queries_total",
            "Total number of queries executed"
        ))?;
        
        let query_duration = Histogram::with_opts(HistogramOpts::new(
            "duckhub_query_duration_seconds",
            "Query execution duration in seconds"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]))?;
        
        let query_errors = Counter::with_opts(Opts::new(
            "duckhub_query_errors_total",
            "Total number of query errors"
        ))?;
        
        let query_cache_hits = Counter::with_opts(Opts::new(
            "duckhub_query_cache_hits_total",
            "Total number of query cache hits"
        ))?;
        
        let query_cache_misses = Counter::with_opts(Opts::new(
            "duckhub_query_cache_misses_total",
            "Total number of query cache misses"
        ))?;
        
        // Connection metrics
        let connections_active = Gauge::with_opts(Opts::new(
            "duckhub_connections_active",
            "Number of active database connections"
        ))?;
        
        let connections_idle = Gauge::with_opts(Opts::new(
            "duckhub_connections_idle",
            "Number of idle database connections"
        ))?;
        
        let connections_total = Counter::with_opts(Opts::new(
            "duckhub_connections_total",
            "Total number of database connections created"
        ))?;
        
        let connection_errors = Counter::with_opts(Opts::new(
            "duckhub_connection_errors_total",
            "Total number of connection errors"
        ))?;
        
        // Database metrics
        let database_size = Gauge::with_opts(Opts::new(
            "duckhub_database_size_bytes",
            "Database size in bytes"
        ))?;
        
        let table_count = Gauge::with_opts(Opts::new(
            "duckhub_table_count",
            "Number of tables in database"
        ))?;
        
        // Performance metrics
        let rows_scanned = Counter::with_opts(Opts::new(
            "duckhub_rows_scanned_total",
            "Total number of rows scanned"
        ))?;
        
        let rows_returned = Counter::with_opts(Opts::new(
            "duckhub_rows_returned_total",
            "Total number of rows returned"
        ))?;
        
        let bytes_scanned = Counter::with_opts(Opts::new(
            "duckhub_bytes_scanned_total",
            "Total number of bytes scanned"
        ))?;
        
        let bytes_returned = Counter::with_opts(Opts::new(
            "duckhub_bytes_returned_total",
            "Total number of bytes returned"
        ))?;
        
        // Register all metrics
        registry.register(Box::new(query_total.clone()))?;
        registry.register(Box::new(query_duration.clone()))?;
        registry.register(Box::new(query_errors.clone()))?;
        registry.register(Box::new(query_cache_hits.clone()))?;
        registry.register(Box::new(query_cache_misses.clone()))?;
        registry.register(Box::new(connections_active.clone()))?;
        registry.register(Box::new(connections_idle.clone()))?;
        registry.register(Box::new(connections_total.clone()))?;
        registry.register(Box::new(connection_errors.clone()))?;
        registry.register(Box::new(database_size.clone()))?;
        registry.register(Box::new(table_count.clone()))?;
        registry.register(Box::new(rows_scanned.clone()))?;
        registry.register(Box::new(rows_returned.clone()))?;
        registry.register(Box::new(bytes_scanned.clone()))?;
        registry.register(Box::new(bytes_returned.clone()))?;
        
        Ok(Self {
            registry,
            query_total,
            query_duration,
            query_errors,
            query_cache_hits,
            query_cache_misses,
            connections_active,
            connections_idle,
            connections_total,
            connection_errors,
            database_size,
            table_count,
            rows_scanned,
            rows_returned,
            bytes_scanned,
            bytes_returned,
        })
    }

    /// Record query execution
    pub fn record_query(&self, duration: Duration, result: &QueryResult) {
        self.query_total.inc();
        self.query_duration.observe(duration.as_secs_f64());
        self.rows_returned.inc_by(result.row_count as f64);
        
        if let Some(bytes) = result.metadata.bytes_returned {
            self.bytes_returned.inc_by(bytes as f64);
        }
        
        if let Some(bytes) = result.metadata.bytes_scanned {
            self.bytes_scanned.inc_by(bytes as f64);
        }
        
        debug!("Recorded query metrics: duration={}ms, rows={}", 
               duration.as_millis(), result.row_count);
    }

    /// Record query error
    pub fn record_query_error(&self) {
        self.query_errors.inc();
        debug!("Recorded query error");
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.query_cache_hits.inc();
        debug!("Recorded cache hit");
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.query_cache_misses.inc();
        debug!("Recorded cache miss");
    }

    /// Update connection metrics
    pub fn update_connections(&self, active: u32, idle: u32) {
        self.connections_active.set(active as f64);
        self.connections_idle.set(idle as f64);
        debug!("Updated connection metrics: active={}, idle={}", active, idle);
    }

    /// Record new connection
    pub fn record_connection(&self) {
        self.connections_total.inc();
        debug!("Recorded new connection");
    }

    /// Record connection error
    pub fn record_connection_error(&self) {
        self.connection_errors.inc();
        debug!("Recorded connection error");
    }

    /// Update database size
    pub fn update_database_size(&self, size_bytes: u64) {
        self.database_size.set(size_bytes as f64);
        debug!("Updated database size: {} bytes", size_bytes);
    }

    /// Update table count
    pub fn update_table_count(&self, count: u32) {
        self.table_count.set(count as f64);
        debug!("Updated table count: {}", count);
    }

    /// Get Prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get metrics as Prometheus format string
    pub fn gather(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        
        match encoder.encode_to_string(&metric_families) {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to encode metrics: {}", e);
                String::new()
            }
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            queries_total: self.query_total.get() as u64,
            query_errors_total: self.query_errors.get() as u64,
            cache_hits_total: self.query_cache_hits.get() as u64,
            cache_misses_total: self.query_cache_misses.get() as u64,
            connections_active: self.connections_active.get() as u32,
            connections_idle: self.connections_idle.get() as u32,
            connections_total: self.connections_total.get() as u64,
            connection_errors_total: self.connection_errors.get() as u64,
            database_size_bytes: self.database_size.get() as u64,
            table_count: self.table_count.get() as u32,
            rows_scanned_total: self.rows_scanned.get() as u64,
            rows_returned_total: self.rows_returned.get() as u64,
            bytes_scanned_total: self.bytes_scanned.get() as u64,
            bytes_returned_total: self.bytes_returned.get() as u64,
        }
    }
}

impl Default for DatabaseMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics")
    }
}

impl MetricsCollector for DatabaseMetrics {
    fn increment_counter(&self, name: &str, _labels: &[(&str, &str)]) {
        match name {
            "queries_total" => self.query_total.inc(),
            "query_errors_total" => self.query_errors.inc(),
            "cache_hits_total" => self.query_cache_hits.inc(),
            "cache_misses_total" => self.query_cache_misses.inc(),
            "connections_total" => self.connections_total.inc(),
            "connection_errors_total" => self.connection_errors.inc(),
            _ => debug!("Unknown counter: {}", name),
        }
    }

    fn record_histogram(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        match name {
            "query_duration_seconds" => self.query_duration.observe(value),
            _ => debug!("Unknown histogram: {}", name),
        }
    }

    fn set_gauge(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        match name {
            "connections_active" => self.connections_active.set(value),
            "connections_idle" => self.connections_idle.set(value),
            "database_size_bytes" => self.database_size.set(value),
            "table_count" => self.table_count.set(value),
            _ => debug!("Unknown gauge: {}", name),
        }
    }

    fn record_timing(&self, name: &str, duration_ms: f64, labels: &[(&str, &str)]) {
        self.record_histogram(name, duration_ms / 1000.0, labels);
    }
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub queries_total: u64,
    pub query_errors_total: u64,
    pub cache_hits_total: u64,
    pub cache_misses_total: u64,
    pub connections_active: u32,
    pub connections_idle: u32,
    pub connections_total: u64,
    pub connection_errors_total: u64,
    pub database_size_bytes: u64,
    pub table_count: u32,
    pub rows_scanned_total: u64,
    pub rows_returned_total: u64,
    pub bytes_scanned_total: u64,
    pub bytes_returned_total: u64,
}

impl MetricsSnapshot {
    /// Calculate cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_cache_requests = self.cache_hits_total + self.cache_misses_total;
        if total_cache_requests == 0 {
            0.0
        } else {
            self.cache_hits_total as f64 / total_cache_requests as f64
        }
    }

    /// Calculate error rate
    pub fn query_error_rate(&self) -> f64 {
        if self.queries_total == 0 {
            0.0
        } else {
            self.query_errors_total as f64 / self.queries_total as f64
        }
    }

    /// Calculate connection error rate
    pub fn connection_error_rate(&self) -> f64 {
        if self.connections_total == 0 {
            0.0
        } else {
            self.connection_errors_total as f64 / self.connections_total as f64
        }
    }

    /// Get total connections
    pub fn total_connections(&self) -> u32 {
        self.connections_active + self.connections_idle
    }
}

/// Metrics reporter for periodic reporting
pub struct MetricsReporter {
    metrics: Arc<DatabaseMetrics>,
    interval: Duration,
}

impl MetricsReporter {
    /// Create a new metrics reporter
    pub fn new(metrics: Arc<DatabaseMetrics>, interval: Duration) -> Self {
        Self { metrics, interval }
    }

    /// Start periodic reporting
    pub fn start(&self) {
        let metrics = self.metrics.clone();
        let interval = self.interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                let snapshot = metrics.snapshot();
                Self::report_metrics(&snapshot);
            }
        });
    }

    /// Report metrics snapshot
    fn report_metrics(snapshot: &MetricsSnapshot) {
        info!(
            "Database Metrics - Queries: {}, Errors: {}, Cache Hit Ratio: {:.2}%, Connections: {}",
            snapshot.queries_total,
            snapshot.query_errors_total,
            snapshot.cache_hit_ratio() * 100.0,
            snapshot.total_connections()
        );
        
        debug!(
            "Detailed Metrics - DB Size: {} bytes, Tables: {}, Rows Scanned: {}, Rows Returned: {}",
            snapshot.database_size_bytes,
            snapshot.table_count,
            snapshot.rows_scanned_total,
            snapshot.rows_returned_total
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use duckhub_common::utils::{generate_id, now};

    #[test]
    fn test_metrics_creation() {
        let metrics = DatabaseMetrics::new().unwrap();
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.queries_total, 0);
        assert_eq!(snapshot.connections_active, 0);
    }

    #[test]
    fn test_query_metrics() {
        let metrics = DatabaseMetrics::new().unwrap();
        
        let result = QueryResult {
            query_id: generate_id(),
            columns: vec!["col1".to_string()],
            rows: vec![vec![serde_json::Value::String("test".to_string())]],
            row_count: 1,
            execution_time_ms: 100,
            metadata: QueryMetadata {
                bytes_scanned: Some(1024),
                bytes_returned: Some(512),
                cache_hit: false,
                execution_plan: None,
            },
        };
        
        metrics.record_query(Duration::from_millis(100), &result);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.queries_total, 1);
        assert_eq!(snapshot.rows_returned_total, 1);
        assert_eq!(snapshot.bytes_scanned_total, 1024);
        assert_eq!(snapshot.bytes_returned_total, 512);
    }

    #[test]
    fn test_connection_metrics() {
        let metrics = DatabaseMetrics::new().unwrap();
        
        metrics.record_connection();
        metrics.update_connections(5, 3);
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.connections_total, 1);
        assert_eq!(snapshot.connections_active, 5);
        assert_eq!(snapshot.connections_idle, 3);
        assert_eq!(snapshot.total_connections(), 8);
    }

    #[test]
    fn test_metrics_snapshot_calculations() {
        let snapshot = MetricsSnapshot {
            queries_total: 100,
            query_errors_total: 5,
            cache_hits_total: 80,
            cache_misses_total: 20,
            connections_active: 5,
            connections_idle: 3,
            connections_total: 50,
            connection_errors_total: 2,
            database_size_bytes: 1024 * 1024,
            table_count: 10,
            rows_scanned_total: 10000,
            rows_returned_total: 5000,
            bytes_scanned_total: 1024 * 1024,
            bytes_returned_total: 512 * 1024,
        };
        
        assert_eq!(snapshot.cache_hit_ratio(), 0.8);
        assert_eq!(snapshot.query_error_rate(), 0.05);
        assert_eq!(snapshot.connection_error_rate(), 0.04);
        assert_eq!(snapshot.total_connections(), 8);
    }
}
