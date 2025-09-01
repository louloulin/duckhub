//! Real-time Financial Data Processor
//! 
//! This module integrates all DuckLake components to provide a comprehensive
//! real-time financial data processing solution with millisecond-level performance.

use crate::ducklake_real::DuckLakeManager;
use crate::stream_processor::{StreamProcessor, StreamProcessorConfig, StreamRecord};
use crate::risk_engine::{RealTimeRiskEngine, RiskEngineConfig, PortfolioUpdate};
use crate::memory_optimization::{MemoryManager, MemoryPoolConfig};
use duckhub_common::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::{Stream, StreamExt};
use chrono::{DateTime, Utc, Timelike};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// Real-time financial processor that coordinates all components
#[derive(Debug)]
pub struct RealTimeFinancialProcessor {
    ducklake_manager: Arc<DuckLakeManager>,
    stream_processor: Arc<StreamProcessor>,
    risk_engine: Arc<RealTimeRiskEngine>,
    memory_manager: Arc<MemoryManager>,
    trade_analyzer: Arc<TradeAnalyzer>,
    market_data_handler: Arc<MarketDataHandler>,
    config: RealTimeProcessorConfig,
    metrics: Arc<RealTimeProcessorMetrics>,
}

/// Configuration for the real-time processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeProcessorConfig {
    pub max_processing_latency_ms: u64,
    pub batch_size: usize,
    pub enable_risk_monitoring: bool,
    pub enable_trade_analysis: bool,
    pub enable_market_data_processing: bool,
    pub memory_optimization_enabled: bool,
    pub auto_scaling_enabled: bool,
    pub performance_monitoring_interval_ms: u64,
}

impl Default for RealTimeProcessorConfig {
    fn default() -> Self {
        Self {
            max_processing_latency_ms: 100, // 100ms SLA
            batch_size: 1000,
            enable_risk_monitoring: true,
            enable_trade_analysis: true,
            enable_market_data_processing: true,
            memory_optimization_enabled: true,
            auto_scaling_enabled: true,
            performance_monitoring_interval_ms: 1000,
        }
    }
}

/// Financial data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinancialData {
    Trade(TradeData),
    MarketData(MarketDataSnapshot),
    Portfolio(PortfolioUpdate),
    RiskEvent(RiskEvent),
}

/// Trade data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub trade_id: String,
    pub symbol: String,
    pub side: TradeSide,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub account_id: String,
    pub order_type: OrderType,
    pub execution_venue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// Market data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSnapshot {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bid_price: f64,
    pub ask_price: f64,
    pub last_price: f64,
    pub volume: u64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
}

/// Risk event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvent {
    pub event_id: String,
    pub portfolio_id: String,
    pub event_type: RiskEventType,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskEventType {
    ThresholdBreach,
    AnomalyDetected,
    LiquidityIssue,
    ConcentrationRisk,
}

/// Trade analyzer for real-time trade analysis
#[derive(Debug)]
pub struct TradeAnalyzer {
    config: TradeAnalyzerConfig,
    metrics: Arc<TradeAnalyzerMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalyzerConfig {
    pub enable_pattern_detection: bool,
    pub enable_anomaly_detection: bool,
    pub enable_compliance_check: bool,
    pub max_analysis_time_ms: u64,
}

impl Default for TradeAnalyzerConfig {
    fn default() -> Self {
        Self {
            enable_pattern_detection: true,
            enable_anomaly_detection: true,
            enable_compliance_check: true,
            max_analysis_time_ms: 50, // 50ms for trade analysis
        }
    }
}

/// Trade analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysisResult {
    pub trade_id: String,
    pub risk_score: f64,
    pub compliance_status: ComplianceStatus,
    pub market_impact: MarketImpact,
    pub execution_quality: ExecutionQuality,
    pub recommendations: Vec<TradeRecommendation>,
    pub analysis_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    Warning,
    Violation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpact {
    pub estimated_impact_bps: f64,
    pub liquidity_score: f64,
    pub timing_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQuality {
    pub slippage_bps: f64,
    pub fill_ratio: f64,
    pub speed_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecommendation {
    pub recommendation_type: String,
    pub message: String,
    pub priority: String,
}

/// Market data handler
#[derive(Debug)]
pub struct MarketDataHandler {
    config: MarketDataConfig,
    price_cache: Arc<RwLock<HashMap<String, MarketDataSnapshot>>>,
    metrics: Arc<MarketDataMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    pub cache_size: usize,
    pub cache_ttl_ms: u64,
    pub enable_real_time_updates: bool,
    pub max_processing_time_ms: u64,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            cache_size: 10000,
            cache_ttl_ms: 1000, // 1 second TTL
            enable_real_time_updates: true,
            max_processing_time_ms: 10, // 10ms for market data processing
        }
    }
}

/// Metrics for various components
#[derive(Debug)]
pub struct RealTimeProcessorMetrics {
    pub records_processed: prometheus::Counter,
    pub processing_latency: prometheus::Histogram,
    pub error_rate: prometheus::Gauge,
    pub throughput: prometheus::Gauge,
    pub memory_usage: prometheus::Gauge,
}

#[derive(Debug)]
pub struct TradeAnalyzerMetrics {
    pub trades_analyzed: prometheus::Counter,
    pub analysis_latency: prometheus::Histogram,
    pub compliance_violations: prometheus::Counter,
    pub risk_alerts: prometheus::Counter,
}

#[derive(Debug)]
pub struct MarketDataMetrics {
    pub market_updates: prometheus::Counter,
    pub cache_hits: prometheus::Counter,
    pub cache_misses: prometheus::Counter,
    pub update_latency: prometheus::Histogram,
}

impl Default for RealTimeProcessorMetrics {
    fn default() -> Self {
        Self {
            records_processed: prometheus::Counter::new("realtime_records_processed_total", "Total records processed").unwrap(),
            processing_latency: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("realtime_processing_latency_seconds", "Processing latency")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
            ).unwrap(),
            error_rate: prometheus::Gauge::new("realtime_error_rate", "Current error rate").unwrap(),
            throughput: prometheus::Gauge::new("realtime_throughput_per_second", "Current throughput per second").unwrap(),
            memory_usage: prometheus::Gauge::new("realtime_memory_usage_bytes", "Current memory usage in bytes").unwrap(),
        }
    }
}

impl Default for TradeAnalyzerMetrics {
    fn default() -> Self {
        Self {
            trades_analyzed: prometheus::Counter::new("trades_analyzed_total", "Total trades analyzed").unwrap(),
            analysis_latency: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("trade_analysis_latency_seconds", "Trade analysis latency")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1])
            ).unwrap(),
            compliance_violations: prometheus::Counter::new("compliance_violations_total", "Total compliance violations").unwrap(),
            risk_alerts: prometheus::Counter::new("trade_risk_alerts_total", "Total trade risk alerts").unwrap(),
        }
    }
}

impl Default for MarketDataMetrics {
    fn default() -> Self {
        Self {
            market_updates: prometheus::Counter::new("market_data_updates_total", "Total market data updates").unwrap(),
            cache_hits: prometheus::Counter::new("market_data_cache_hits_total", "Total cache hits").unwrap(),
            cache_misses: prometheus::Counter::new("market_data_cache_misses_total", "Total cache misses").unwrap(),
            update_latency: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("market_data_update_latency_seconds", "Market data update latency")
                    .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01])
            ).unwrap(),
        }
    }
}

impl TradeAnalyzer {
    pub fn new(config: TradeAnalyzerConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(TradeAnalyzerMetrics::default()),
        }
    }

    /// Analyze a trade in real-time
    pub async fn analyze_trade(&self, trade: &TradeData) -> Result<TradeAnalysisResult> {
        let start_time = std::time::Instant::now();

        // Risk scoring
        let risk_score = self.calculate_risk_score(trade).await?;

        // Compliance check
        let compliance_status = self.check_compliance(trade).await?;

        // Market impact analysis
        let market_impact = self.analyze_market_impact(trade).await?;

        // Execution quality assessment
        let execution_quality = self.assess_execution_quality(trade).await?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(trade, risk_score, &compliance_status).await?;

        let analysis_time = start_time.elapsed().as_millis() as u64;

        // Update metrics
        self.metrics.trades_analyzed.inc();
        self.metrics.analysis_latency.observe(analysis_time as f64 / 1000.0);

        if matches!(compliance_status, ComplianceStatus::Violation) {
            self.metrics.compliance_violations.inc();
        }

        if risk_score > 0.8 {
            self.metrics.risk_alerts.inc();
        }

        Ok(TradeAnalysisResult {
            trade_id: trade.trade_id.clone(),
            risk_score,
            compliance_status,
            market_impact,
            execution_quality,
            recommendations,
            analysis_time_ms: analysis_time,
        })
    }

    async fn calculate_risk_score(&self, trade: &TradeData) -> Result<f64> {
        // Simplified risk scoring based on trade characteristics
        let mut score: f64 = 0.0;

        // Size-based risk
        let notional = trade.quantity * trade.price;
        if notional > 1_000_000.0 {
            score += 0.3;
        } else if notional > 100_000.0 {
            score += 0.1;
        }

        // Time-based risk (market hours vs after hours)
        let hour = trade.timestamp.hour();
        if hour < 9 || hour > 16 {
            score += 0.2; // After hours trading is riskier
        }

        // Order type risk
        match trade.order_type {
            OrderType::Market => score += 0.1,
            OrderType::Stop | OrderType::StopLimit => score += 0.2,
            _ => {}
        }

        Ok(score.min(1.0))
    }

    async fn check_compliance(&self, trade: &TradeData) -> Result<ComplianceStatus> {
        // Simplified compliance checks
        let notional = trade.quantity * trade.price;

        // Position size limits
        if notional > 10_000_000.0 {
            return Ok(ComplianceStatus::Violation);
        } else if notional > 5_000_000.0 {
            return Ok(ComplianceStatus::Warning);
        }

        // Time-based restrictions
        let hour = trade.timestamp.hour();
        if hour < 4 || hour > 20 {
            return Ok(ComplianceStatus::Warning);
        }

        Ok(ComplianceStatus::Compliant)
    }

    async fn analyze_market_impact(&self, trade: &TradeData) -> Result<MarketImpact> {
        // Simplified market impact analysis
        let notional = trade.quantity * trade.price;
        
        let estimated_impact_bps = if notional > 1_000_000.0 {
            5.0 // 5 basis points for large trades
        } else if notional > 100_000.0 {
            2.0 // 2 basis points for medium trades
        } else {
            0.5 // 0.5 basis points for small trades
        };

        Ok(MarketImpact {
            estimated_impact_bps,
            liquidity_score: 0.8, // Assume good liquidity
            timing_score: 0.9,    // Assume good timing
        })
    }

    async fn assess_execution_quality(&self, trade: &TradeData) -> Result<ExecutionQuality> {
        // Simplified execution quality assessment
        Ok(ExecutionQuality {
            slippage_bps: 1.0,  // 1 basis point slippage
            fill_ratio: 1.0,    // 100% fill
            speed_score: 0.95,  // 95% speed score
        })
    }

    async fn generate_recommendations(&self, trade: &TradeData, risk_score: f64, compliance_status: &ComplianceStatus) -> Result<Vec<TradeRecommendation>> {
        let mut recommendations = Vec::new();

        if risk_score > 0.7 {
            recommendations.push(TradeRecommendation {
                recommendation_type: "risk_management".to_string(),
                message: "Consider reducing position size due to high risk score".to_string(),
                priority: "high".to_string(),
            });
        }

        if matches!(compliance_status, ComplianceStatus::Warning | ComplianceStatus::Violation) {
            recommendations.push(TradeRecommendation {
                recommendation_type: "compliance".to_string(),
                message: "Review trade for compliance issues".to_string(),
                priority: "critical".to_string(),
            });
        }

        let notional = trade.quantity * trade.price;
        if notional > 1_000_000.0 {
            recommendations.push(TradeRecommendation {
                recommendation_type: "execution".to_string(),
                message: "Consider breaking large order into smaller chunks".to_string(),
                priority: "medium".to_string(),
            });
        }

        Ok(recommendations)
    }
}

impl MarketDataHandler {
    pub fn new(config: MarketDataConfig) -> Self {
        Self {
            config,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(MarketDataMetrics::default()),
        }
    }

    /// Process market data update
    pub async fn process_market_data(&self, data: MarketDataSnapshot) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Update cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(data.symbol.clone(), data.clone());

            // Limit cache size
            if cache.len() > self.config.cache_size {
                // Remove oldest entries (simplified)
                let keys_to_remove: Vec<String> = cache.keys().take(cache.len() - self.config.cache_size).cloned().collect();
                for key in keys_to_remove {
                    cache.remove(&key);
                }
            }
        }

        // Update metrics
        self.metrics.market_updates.inc();
        let processing_time = start_time.elapsed().as_secs_f64();
        self.metrics.update_latency.observe(processing_time);

        debug!("Processed market data for {} in {:.3}ms", data.symbol, processing_time * 1000.0);
        Ok(())
    }

    /// Get latest market data for symbol
    pub async fn get_market_data(&self, symbol: &str) -> Option<MarketDataSnapshot> {
        let cache = self.price_cache.read().await;
        if let Some(data) = cache.get(symbol) {
            self.metrics.cache_hits.inc();
            Some(data.clone())
        } else {
            self.metrics.cache_misses.inc();
            None
        }
    }

    /// Get all cached market data
    pub async fn get_all_market_data(&self) -> HashMap<String, MarketDataSnapshot> {
        let cache = self.price_cache.read().await;
        cache.clone()
    }
}

impl RealTimeFinancialProcessor {
    /// Create a new real-time financial processor
    pub async fn new(
        ducklake_manager: Arc<DuckLakeManager>,
        config: RealTimeProcessorConfig,
    ) -> Result<Self> {
        // Create stream processor
        let stream_config = StreamProcessorConfig {
            max_batch_size: config.batch_size,
            batch_timeout_ms: config.max_processing_latency_ms,
            ..Default::default()
        };
        let stream_processor = Arc::new(StreamProcessor::new(ducklake_manager.clone(), stream_config).await?);

        // Create risk engine
        let risk_config = RiskEngineConfig {
            max_processing_time_ms: config.max_processing_latency_ms,
            ..Default::default()
        };
        let risk_engine = Arc::new(RealTimeRiskEngine::new(ducklake_manager.clone(), risk_config).await?);

        // Create memory manager
        let memory_config = MemoryPoolConfig::default();
        let memory_manager = Arc::new(MemoryManager::new(memory_config).await
            .map_err(|e| DuckHubError::database(format!("Failed to create memory manager: {}", e)))?);

        // Create trade analyzer
        let trade_analyzer = Arc::new(TradeAnalyzer::new(TradeAnalyzerConfig::default()));

        // Create market data handler
        let market_data_handler = Arc::new(MarketDataHandler::new(MarketDataConfig::default()));

        let metrics = Arc::new(RealTimeProcessorMetrics::default());

        Ok(Self {
            ducklake_manager,
            stream_processor,
            risk_engine,
            memory_manager,
            trade_analyzer,
            market_data_handler,
            config,
            metrics,
        })
    }

    /// Process a stream of financial data
    pub async fn process_financial_stream<S>(&self, mut stream: S) -> Result<()>
    where
        S: Stream<Item = FinancialData> + Send + Unpin,
    {
        info!("Starting real-time financial data processing");

        while let Some(data) = stream.next().await {
            let start_time = std::time::Instant::now();

            match self.process_financial_data(data).await {
                Ok(_) => {
                    self.metrics.records_processed.inc();
                }
                Err(e) => {
                    error!("Failed to process financial data: {}", e);
                    // Update error rate metric
                    let current_error_rate = self.metrics.error_rate.get();
                    self.metrics.error_rate.set(current_error_rate + 0.01);
                }
            }

            let processing_time = start_time.elapsed();
            self.metrics.processing_latency.observe(processing_time.as_secs_f64());

            // Check SLA compliance
            if processing_time.as_millis() > self.config.max_processing_latency_ms as u128 {
                warn!("Processing exceeded SLA: {}ms > {}ms", 
                      processing_time.as_millis(), self.config.max_processing_latency_ms);
            }
        }

        info!("Real-time financial data processing completed");
        Ok(())
    }

    /// Process individual financial data item
    async fn process_financial_data(&self, data: FinancialData) -> Result<()> {
        match data {
            FinancialData::Trade(trade_data) => {
                self.process_trade_data(trade_data).await
            }
            FinancialData::MarketData(market_data) => {
                self.process_market_data(market_data).await
            }
            FinancialData::Portfolio(portfolio_update) => {
                self.process_portfolio_update(portfolio_update).await
            }
            FinancialData::RiskEvent(risk_event) => {
                self.process_risk_event(risk_event).await
            }
        }
    }

    /// Process trade data
    async fn process_trade_data(&self, trade: TradeData) -> Result<()> {
        if self.config.enable_trade_analysis {
            // Analyze trade
            let analysis_result = self.trade_analyzer.analyze_trade(&trade).await?;
            
            // Store analysis result
            self.store_trade_analysis(&analysis_result).await?;
            
            debug!("Processed trade {} with risk score {:.2}", 
                   trade.trade_id, analysis_result.risk_score);
        }

        // Store trade data
        self.store_trade_data(&trade).await?;
        Ok(())
    }

    /// Process market data
    async fn process_market_data(&self, market_data: MarketDataSnapshot) -> Result<()> {
        if self.config.enable_market_data_processing {
            // Update market data cache
            self.market_data_handler.process_market_data(market_data.clone()).await?;
        }

        // Store market data
        self.store_market_data(&market_data).await?;
        Ok(())
    }

    /// Process portfolio update
    async fn process_portfolio_update(&self, portfolio_update: PortfolioUpdate) -> Result<()> {
        if self.config.enable_risk_monitoring {
            // This would trigger risk monitoring, but we need to convert to a stream
            // For now, just store the portfolio update
            debug!("Processing portfolio update for {}", portfolio_update.portfolio_id);
        }

        // Store portfolio update
        self.store_portfolio_update(&portfolio_update).await?;
        Ok(())
    }

    /// Process risk event
    async fn process_risk_event(&self, risk_event: RiskEvent) -> Result<()> {
        // Store risk event
        self.store_risk_event(&risk_event).await?;
        
        warn!("Risk event processed: {:?} - {}", risk_event.event_type, risk_event.message);
        Ok(())
    }

    /// Store trade data in DuckLake
    async fn store_trade_data(&self, trade: &TradeData) -> Result<()> {
        let sql = format!(
            "INSERT INTO trades (trade_id, symbol, side, quantity, price, timestamp, account_id, order_type, execution_venue) VALUES ('{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}')",
            trade.trade_id,
            trade.symbol,
            serde_json::to_string(&trade.side).unwrap_or_default().trim_matches('"'),
            trade.quantity,
            trade.price,
            trade.timestamp.to_rfc3339(),
            trade.account_id,
            serde_json::to_string(&trade.order_type).unwrap_or_default().trim_matches('"'),
            trade.execution_venue
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        Ok(())
    }

    /// Store trade analysis result
    async fn store_trade_analysis(&self, analysis: &TradeAnalysisResult) -> Result<()> {
        let sql = format!(
            "INSERT INTO trade_analysis (trade_id, risk_score, compliance_status, analysis_time_ms) VALUES ('{}', {}, '{}', {})",
            analysis.trade_id,
            analysis.risk_score,
            serde_json::to_string(&analysis.compliance_status).unwrap_or_default().trim_matches('"'),
            analysis.analysis_time_ms
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        Ok(())
    }

    /// Store market data
    async fn store_market_data(&self, market_data: &MarketDataSnapshot) -> Result<()> {
        let sql = format!(
            "INSERT INTO market_data (symbol, timestamp, bid_price, ask_price, last_price, volume, high, low, open, close) VALUES ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {})",
            market_data.symbol,
            market_data.timestamp.to_rfc3339(),
            market_data.bid_price,
            market_data.ask_price,
            market_data.last_price,
            market_data.volume,
            market_data.high,
            market_data.low,
            market_data.open,
            market_data.close
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        Ok(())
    }

    /// Store portfolio update
    async fn store_portfolio_update(&self, portfolio: &PortfolioUpdate) -> Result<()> {
        let positions_json = serde_json::to_string(&portfolio.positions)
            .map_err(|e| DuckHubError::database(format!("Failed to serialize positions: {}", e)))?;

        let sql = format!(
            "INSERT INTO portfolio_updates (portfolio_id, timestamp, positions) VALUES ('{}', '{}', '{}')",
            portfolio.portfolio_id,
            portfolio.timestamp.to_rfc3339(),
            positions_json.replace("'", "''")
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        Ok(())
    }

    /// Store risk event
    async fn store_risk_event(&self, risk_event: &RiskEvent) -> Result<()> {
        let sql = format!(
            "INSERT INTO risk_events (event_id, portfolio_id, event_type, severity, message, timestamp) VALUES ('{}', '{}', '{}', '{}', '{}', '{}')",
            risk_event.event_id,
            risk_event.portfolio_id,
            serde_json::to_string(&risk_event.event_type).unwrap_or_default().trim_matches('"'),
            risk_event.severity,
            risk_event.message.replace("'", "''"),
            risk_event.timestamp.to_rfc3339()
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        Ok(())
    }

    /// Get processing metrics
    pub fn get_metrics(&self) -> Arc<RealTimeProcessorMetrics> {
        self.metrics.clone()
    }

    /// Get current throughput
    pub async fn get_current_throughput(&self) -> f64 {
        self.metrics.throughput.get()
    }

    /// Get memory usage statistics
    pub async fn get_memory_stats(&self) -> Result<String> {
        let stats = self.memory_manager.get_memory_stats().await;
        Ok(serde_json::to_string_pretty(&stats)
            .map_err(|e| DuckHubError::database(format!("Failed to serialize memory stats: {}", e)))?)
    }

    /// Perform garbage collection
    pub async fn garbage_collect(&self) -> Result<usize> {
        self.memory_manager.garbage_collect_all().await
            .map_err(|e| DuckHubError::database(format!("Garbage collection failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::iter;

    #[tokio::test]
    async fn test_trade_analyzer() {
        let analyzer = TradeAnalyzer::new(TradeAnalyzerConfig::default());
        
        let trade = TradeData {
            trade_id: "test_trade_1".to_string(),
            symbol: "AAPL".to_string(),
            side: TradeSide::Buy,
            quantity: 100.0,
            price: 150.0,
            timestamp: Utc::now(),
            account_id: "test_account".to_string(),
            order_type: OrderType::Market,
            execution_venue: "NASDAQ".to_string(),
        };

        let result = analyzer.analyze_trade(&trade).await.unwrap();
        
        assert_eq!(result.trade_id, "test_trade_1");
        assert!(result.risk_score >= 0.0 && result.risk_score <= 1.0);
        assert!(result.analysis_time_ms < 100); // Should be fast
    }

    #[tokio::test]
    async fn test_market_data_handler() {
        let handler = MarketDataHandler::new(MarketDataConfig::default());
        
        let market_data = MarketDataSnapshot {
            symbol: "AAPL".to_string(),
            timestamp: Utc::now(),
            bid_price: 149.50,
            ask_price: 150.50,
            last_price: 150.00,
            volume: 1000000,
            high: 151.00,
            low: 149.00,
            open: 149.75,
            close: 150.00,
        };

        // Process market data
        handler.process_market_data(market_data.clone()).await.unwrap();
        
        // Retrieve market data
        let retrieved = handler.get_market_data("AAPL").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().last_price, 150.00);
    }
}