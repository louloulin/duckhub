//! Enhanced Stream Processing Engine for DuckLake
//! 
//! This module provides high-performance stream processing capabilities
//! built on top of DuckLake, supporting real-time financial data processing.

use crate::ducklake_real::{DuckLakeManager, DuckLakeConfig};
use crate::real_duckdb::Connection;
use duckhub_common::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc, Semaphore};
use tokio_stream::{Stream, StreamExt};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// Enhanced stream processor for DuckLake
#[derive(Debug)]
pub struct StreamProcessor {
    ducklake_manager: Arc<DuckLakeManager>,
    config: StreamProcessorConfig,
    backpressure_controller: Arc<BackpressureController>,
    fault_tolerance: Arc<FaultTolerance>,
    metrics: Arc<StreamProcessorMetrics>,
    processing_pipeline: Arc<ProcessingPipeline>,
    watermark_manager: Arc<WatermarkManager>,
}

/// Stream processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProcessorConfig {
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub max_concurrent_batches: usize,
    pub buffer_size: usize,
    pub enable_backpressure: bool,
    pub watermark_interval_ms: u64,
    pub late_data_tolerance_ms: u64,
    pub checkpoint_interval_ms: u64,
}

impl Default for StreamProcessorConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            batch_timeout_ms: 100,
            max_concurrent_batches: 10,
            buffer_size: 10000,
            enable_backpressure: true,
            watermark_interval_ms: 1000,
            late_data_tolerance_ms: 5000,
            checkpoint_interval_ms: 30000,
        }
    }
}

/// Data stream record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_time: DateTime<Utc>,
    pub data: serde_json::Value,
    pub partition_key: Option<String>,
    pub headers: HashMap<String, String>,
}

/// Processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub processed_count: usize,
    pub failed_count: usize,
    pub processing_time_ms: u64,
    pub watermark: Option<DateTime<Utc>>,
    pub errors: Vec<ProcessingError>,
}

/// Processing error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingError {
    pub record_id: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Backpressure controller
#[derive(Debug)]
pub struct BackpressureController {
    semaphore: Arc<Semaphore>,
    buffer_usage: Arc<RwLock<f64>>,
    config: StreamProcessorConfig,
}

impl BackpressureController {
    pub fn new(config: StreamProcessorConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_batches)),
            buffer_usage: Arc::new(RwLock::new(0.0)),
            config,
        }
    }

    /// Check if backpressure should be applied
    pub async fn should_apply_backpressure(&self) -> bool {
        if !self.config.enable_backpressure {
            return false;
        }

        let buffer_usage = *self.buffer_usage.read().await;
        buffer_usage > 0.8 // Apply backpressure when buffer is 80% full
    }

    /// Update buffer usage
    pub async fn update_buffer_usage(&self, current_size: usize) {
        let usage = current_size as f64 / self.config.buffer_size as f64;
        *self.buffer_usage.write().await = usage;
    }

    /// Acquire processing permit
    pub async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit> {
        self.semaphore.acquire().await
            .map_err(|e| DuckHubError::database(format!("Failed to acquire processing permit: {}", e)))
    }
}

/// Fault tolerance manager
#[derive(Debug)]
pub struct FaultTolerance {
    retry_config: RetryConfig,
    dead_letter_queue: Arc<Mutex<VecDeque<StreamRecord>>>,
    checkpoint_manager: Arc<CheckpointManager>,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl FaultTolerance {
    pub fn new(retry_config: RetryConfig) -> Self {
        Self {
            retry_config,
            dead_letter_queue: Arc::new(Mutex::new(VecDeque::new())),
            checkpoint_manager: Arc::new(CheckpointManager::new()),
        }
    }

    /// Retry processing with exponential backoff
    pub async fn retry_with_backoff<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut delay = self.retry_config.initial_delay_ms;
        
        for attempt in 0..=self.retry_config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt == self.retry_config.max_retries => return Err(e),
                Err(e) => {
                    warn!("Operation failed (attempt {}): {}", attempt + 1, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    delay = (delay as f64 * self.retry_config.backoff_multiplier) as u64;
                    delay = delay.min(self.retry_config.max_delay_ms);
                }
            }
        }
        
        unreachable!()
    }

    /// Add record to dead letter queue
    pub async fn add_to_dead_letter_queue(&self, record: StreamRecord) {
        let mut dlq = self.dead_letter_queue.lock().await;
        dlq.push_back(record);
        
        // Limit DLQ size
        if dlq.len() > 1000 {
            dlq.pop_front();
        }
    }

    /// Get dead letter queue size
    pub async fn dead_letter_queue_size(&self) -> usize {
        self.dead_letter_queue.lock().await.len()
    }
}

/// Checkpoint manager for fault recovery
#[derive(Debug)]
pub struct CheckpointManager {
    checkpoints: Arc<RwLock<HashMap<String, Checkpoint>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub stream_id: String,
    pub offset: u64,
    pub timestamp: DateTime<Utc>,
    pub watermark: Option<DateTime<Utc>>,
}

impl CheckpointManager {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Save checkpoint
    pub async fn save_checkpoint(&self, checkpoint: Checkpoint) {
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.insert(checkpoint.stream_id.clone(), checkpoint);
    }

    /// Get checkpoint
    pub async fn get_checkpoint(&self, stream_id: &str) -> Option<Checkpoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints.get(stream_id).cloned()
    }
}

/// Processing pipeline
#[derive(Debug)]
pub struct ProcessingPipeline {
    stages: Vec<Arc<dyn ProcessingStage>>,
}

#[async_trait::async_trait]
pub trait ProcessingStage: Send + Sync + std::fmt::Debug {
    async fn process(&self, records: Vec<StreamRecord>) -> Result<Vec<StreamRecord>>;
    fn name(&self) -> &str;
}

impl ProcessingPipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
        }
    }

    pub fn add_stage(&mut self, stage: Arc<dyn ProcessingStage>) {
        self.stages.push(stage);
    }

    /// Process records through all stages
    pub async fn process(&self, mut records: Vec<StreamRecord>) -> Result<Vec<StreamRecord>> {
        for stage in &self.stages {
            debug!("Processing {} records through stage: {}", records.len(), stage.name());
            records = stage.process(records).await?;
        }
        Ok(records)
    }
}

/// Watermark manager for event time processing
#[derive(Debug)]
pub struct WatermarkManager {
    current_watermark: Arc<RwLock<Option<DateTime<Utc>>>>,
    config: StreamProcessorConfig,
}

impl WatermarkManager {
    pub fn new(config: StreamProcessorConfig) -> Self {
        Self {
            current_watermark: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Update watermark based on event times
    pub async fn update_watermark(&self, event_times: &[DateTime<Utc>]) {
        if event_times.is_empty() {
            return;
        }

        let min_event_time = event_times.iter().min().unwrap();
        let new_watermark = *min_event_time - chrono::Duration::milliseconds(self.config.late_data_tolerance_ms as i64);

        let mut current = self.current_watermark.write().await;
        match *current {
            Some(current_watermark) if new_watermark > current_watermark => {
                *current = Some(new_watermark);
                debug!("Updated watermark to: {}", new_watermark);
            }
            None => {
                *current = Some(new_watermark);
                debug!("Set initial watermark to: {}", new_watermark);
            }
            _ => {} // Don't move watermark backwards
        }
    }

    /// Get current watermark
    pub async fn get_watermark(&self) -> Option<DateTime<Utc>> {
        *self.current_watermark.read().await
    }

    /// Check if record is late
    pub async fn is_late_record(&self, event_time: DateTime<Utc>) -> bool {
        if let Some(watermark) = self.get_watermark().await {
            event_time < watermark
        } else {
            false
        }
    }
}

/// Stream processor metrics
#[derive(Debug)]
pub struct StreamProcessorMetrics {
    pub records_processed: prometheus::Counter,
    pub records_failed: prometheus::Counter,
    pub processing_latency: prometheus::Histogram,
    pub buffer_usage: prometheus::Gauge,
    pub backpressure_events: prometheus::Counter,
    pub watermark_lag: prometheus::Gauge,
}

impl Default for StreamProcessorMetrics {
    fn default() -> Self {
        Self {
            records_processed: prometheus::Counter::new("stream_records_processed_total", "Total number of stream records processed").unwrap(),
            records_failed: prometheus::Counter::new("stream_records_failed_total", "Total number of stream records failed").unwrap(),
            processing_latency: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("stream_processing_latency_seconds", "Stream processing latency")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
            ).unwrap(),
            buffer_usage: prometheus::Gauge::new("stream_buffer_usage_ratio", "Stream buffer usage ratio").unwrap(),
            backpressure_events: prometheus::Counter::new("stream_backpressure_events_total", "Total number of backpressure events").unwrap(),
            watermark_lag: prometheus::Gauge::new("stream_watermark_lag_seconds", "Stream watermark lag in seconds").unwrap(),
        }
    }
}

impl StreamProcessor {
    /// Create a new stream processor
    pub async fn new(ducklake_manager: Arc<DuckLakeManager>, config: StreamProcessorConfig) -> Result<Self> {
        let backpressure_controller = Arc::new(BackpressureController::new(config.clone()));
        let fault_tolerance = Arc::new(FaultTolerance::new(RetryConfig::default()));
        let metrics = Arc::new(StreamProcessorMetrics::default());
        let processing_pipeline = Arc::new(ProcessingPipeline::new());
        let watermark_manager = Arc::new(WatermarkManager::new(config.clone()));

        Ok(Self {
            ducklake_manager,
            config,
            backpressure_controller,
            fault_tolerance,
            metrics,
            processing_pipeline,
            watermark_manager,
        })
    }

    /// Process a stream of records
    pub async fn process_stream<S>(&self, mut stream: S) -> Result<()>
    where
        S: Stream<Item = StreamRecord> + Send + Unpin,
    {
        info!("Starting stream processing");

        let mut buffer = Vec::with_capacity(self.config.buffer_size);
        let mut last_batch_time = std::time::Instant::now();

        while let Some(record) = stream.next().await {
            // Check for late records
            if self.watermark_manager.is_late_record(record.event_time).await {
                warn!("Received late record: {}", record.id);
                self.fault_tolerance.add_to_dead_letter_queue(record).await;
                continue;
            }

            buffer.push(record);

            // Check if we should process the batch
            let should_process = buffer.len() >= self.config.max_batch_size
                || last_batch_time.elapsed().as_millis() >= self.config.batch_timeout_ms as u128;

            if should_process && !buffer.is_empty() {
                // Apply backpressure if needed
                if self.backpressure_controller.should_apply_backpressure().await {
                    self.metrics.backpressure_events.inc();
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }

                // Process the batch
                let batch = std::mem::take(&mut buffer);
                self.process_batch(batch).await?;
                last_batch_time = std::time::Instant::now();
            }

            // Update buffer usage metrics
            self.backpressure_controller.update_buffer_usage(buffer.len()).await;
            self.metrics.buffer_usage.set(buffer.len() as f64 / self.config.buffer_size as f64);
        }

        // Process remaining records
        if !buffer.is_empty() {
            self.process_batch(buffer).await?;
        }

        info!("Stream processing completed");
        Ok(())
    }

    /// Process a batch of records
    async fn process_batch(&self, records: Vec<StreamRecord>) -> Result<ProcessingResult> {
        let start_time = std::time::Instant::now();
        let batch_size = records.len();

        debug!("Processing batch of {} records", batch_size);

        // Acquire processing permit
        let _permit = self.backpressure_controller.acquire_permit().await?;

        let mut processed_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        // Extract event times for watermark update
        let event_times: Vec<DateTime<Utc>> = records.iter().map(|r| r.event_time).collect();

        // Process records through pipeline with fault tolerance
        let result = self.fault_tolerance.retry_with_backoff(|| async {
            self.processing_pipeline.process(records.clone()).await
        }).await;

        match result {
            Ok(processed_records) => {
                // Store processed records in DuckLake
                if let Err(e) = self.store_records_in_ducklake(&processed_records).await {
                    error!("Failed to store records in DuckLake: {}", e);
                    failed_count = batch_size;
                    errors.push(ProcessingError {
                        record_id: "batch".to_string(),
                        error_type: "storage_error".to_string(),
                        message: e.to_string(),
                        timestamp: Utc::now(),
                    });
                } else {
                    processed_count = processed_records.len();
                }
            }
            Err(e) => {
                error!("Failed to process batch: {}", e);
                failed_count = batch_size;
                
                // Add all records to dead letter queue
                for record in records {
                    self.fault_tolerance.add_to_dead_letter_queue(record).await;
                }

                errors.push(ProcessingError {
                    record_id: "batch".to_string(),
                    error_type: "processing_error".to_string(),
                    message: e.to_string(),
                    timestamp: Utc::now(),
                });
            }
        }

        // Update watermark
        self.watermark_manager.update_watermark(&event_times).await;
        let watermark = self.watermark_manager.get_watermark().await;

        // Update metrics
        self.metrics.records_processed.inc_by(processed_count as f64);
        self.metrics.records_failed.inc_by(failed_count as f64);
        
        let processing_time = start_time.elapsed();
        self.metrics.processing_latency.observe(processing_time.as_secs_f64());

        if let Some(wm) = watermark {
            let lag = Utc::now().signed_duration_since(wm).num_seconds() as f64;
            self.metrics.watermark_lag.set(lag);
        }

        let result = ProcessingResult {
            processed_count,
            failed_count,
            processing_time_ms: processing_time.as_millis() as u64,
            watermark,
            errors,
        };

        debug!("Batch processing completed: {:?}", result);
        Ok(result)
    }

    /// Store processed records in DuckLake
    async fn store_records_in_ducklake(&self, records: &[StreamRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        // Convert records to SQL INSERT statements
        let table_name = "stream_data";
        let mut values = Vec::new();

        for record in records {
            let data_json = serde_json::to_string(&record.data)
                .map_err(|e| DuckHubError::database(format!("Failed to serialize record data: {}", e)))?;
            
            values.push(format!(
                "('{}', '{}', '{}', '{}', '{}')",
                record.id,
                record.timestamp.to_rfc3339(),
                record.event_time.to_rfc3339(),
                data_json.replace("'", "''"),
                record.partition_key.as_deref().unwrap_or("")
            ));
        }

        let sql = format!(
            "INSERT INTO {} (id, timestamp, event_time, data, partition_key) VALUES {}",
            table_name,
            values.join(", ")
        );

        // Execute through DuckLake manager
        self.ducklake_manager.execute_query("default", &sql).await?;

        debug!("Stored {} records in DuckLake", records.len());
        Ok(())
    }

    /// Get processing metrics
    pub fn get_metrics(&self) -> Arc<StreamProcessorMetrics> {
        self.metrics.clone()
    }

    /// Get current watermark
    pub async fn get_current_watermark(&self) -> Option<DateTime<Utc>> {
        self.watermark_manager.get_watermark().await
    }

    /// Get dead letter queue size
    pub async fn get_dead_letter_queue_size(&self) -> usize {
        self.fault_tolerance.dead_letter_queue_size().await
    }
}

/// Default processing stages

/// Validation stage
#[derive(Debug)]
pub struct ValidationStage {
    name: String,
}

impl ValidationStage {
    pub fn new() -> Self {
        Self {
            name: "validation".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ProcessingStage for ValidationStage {
    async fn process(&self, records: Vec<StreamRecord>) -> Result<Vec<StreamRecord>> {
        let mut valid_records = Vec::new();
        
        for record in records {
            // Basic validation
            if record.id.is_empty() {
                warn!("Skipping record with empty ID");
                continue;
            }
            
            if record.data.is_null() {
                warn!("Skipping record {} with null data", record.id);
                continue;
            }
            
            valid_records.push(record);
        }
        
        debug!("Validation stage processed {} records", valid_records.len());
        Ok(valid_records)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Enrichment stage
#[derive(Debug)]
pub struct EnrichmentStage {
    name: String,
}

impl EnrichmentStage {
    pub fn new() -> Self {
        Self {
            name: "enrichment".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ProcessingStage for EnrichmentStage {
    async fn process(&self, mut records: Vec<StreamRecord>) -> Result<Vec<StreamRecord>> {
        for record in &mut records {
            // Add processing metadata
            if let Some(data_obj) = record.data.as_object_mut() {
                data_obj.insert("processed_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
                data_obj.insert("processor_id".to_string(), serde_json::Value::String("ducklake-stream-processor".to_string()));
            }
        }
        
        debug!("Enrichment stage processed {} records", records.len());
        Ok(records)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::iter;

    #[tokio::test]
    async fn test_stream_processor_creation() {
        // This test would require a real DuckLake manager
        // For now, we'll just test the configuration
        let config = StreamProcessorConfig::default();
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.batch_timeout_ms, 100);
    }

    #[tokio::test]
    async fn test_backpressure_controller() {
        let config = StreamProcessorConfig::default();
        let controller = BackpressureController::new(config);
        
        // Initially no backpressure
        assert!(!controller.should_apply_backpressure().await);
        
        // Update buffer usage to trigger backpressure
        controller.update_buffer_usage(9000).await; // 90% of 10000
        assert!(controller.should_apply_backpressure().await);
    }

    #[tokio::test]
    async fn test_watermark_manager() {
        let config = StreamProcessorConfig::default();
        let manager = WatermarkManager::new(config);
        
        let now = Utc::now();
        let event_times = vec![now, now - chrono::Duration::seconds(1)];
        
        manager.update_watermark(&event_times).await;
        let watermark = manager.get_watermark().await;
        
        assert!(watermark.is_some());
        assert!(watermark.unwrap() < now);
    }

    #[tokio::test]
    async fn test_processing_pipeline() {
        let mut pipeline = ProcessingPipeline::new();
        pipeline.add_stage(Arc::new(ValidationStage::new()));
        pipeline.add_stage(Arc::new(EnrichmentStage::new()));
        
        let records = vec![
            StreamRecord {
                id: "test1".to_string(),
                timestamp: Utc::now(),
                event_time: Utc::now(),
                data: serde_json::json!({"value": 42}),
                partition_key: None,
                headers: HashMap::new(),
            }
        ];
        
        let result = pipeline.process(records).await.unwrap();
        assert_eq!(result.len(), 1);
        
        // Check that enrichment was applied
        let processed_data = &result[0].data;
        assert!(processed_data.get("processed_at").is_some());
        assert!(processed_data.get("processor_id").is_some());
    }
}