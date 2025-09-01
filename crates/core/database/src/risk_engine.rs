//! Real-time Risk Monitoring Engine
//! 
//! This module provides millisecond-level risk calculation and monitoring
//! capabilities for financial data processing.

use crate::ducklake_real::DuckLakeManager;
use crate::stream_processor::{StreamRecord, StreamProcessor};
use duckhub_common::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::{Stream, StreamExt};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};

/// Real-time risk engine
#[derive(Debug)]
pub struct RealTimeRiskEngine {
    ducklake_manager: Arc<DuckLakeManager>,
    risk_models: Arc<RwLock<HashMap<String, Arc<dyn RiskModel>>>>,
    threshold_monitor: Arc<ThresholdMonitor>,
    alert_system: Arc<AlertSystem>,
    metrics_collector: Arc<RiskMetricsCollector>,
    config: RiskEngineConfig,
}

/// Risk engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEngineConfig {
    pub max_processing_time_ms: u64,
    pub risk_calculation_interval_ms: u64,
    pub alert_cooldown_ms: u64,
    pub max_concurrent_calculations: usize,
    pub enable_real_time_alerts: bool,
    pub risk_aggregation_window_ms: u64,
}

impl Default for RiskEngineConfig {
    fn default() -> Self {
        Self {
            max_processing_time_ms: 100, // 100ms SLA
            risk_calculation_interval_ms: 1000,
            alert_cooldown_ms: 5000,
            max_concurrent_calculations: 50,
            enable_real_time_alerts: true,
            risk_aggregation_window_ms: 10000,
        }
    }
}

/// Portfolio update from stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioUpdate {
    pub portfolio_id: String,
    pub timestamp: DateTime<Utc>,
    pub positions: Vec<Position>,
    pub market_data: MarketData,
}

/// Position in portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub price: f64,
    pub market_value: f64,
    pub currency: String,
}

/// Market data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub timestamp: DateTime<Utc>,
    pub prices: HashMap<String, f64>,
    pub volatilities: HashMap<String, f64>,
    pub correlations: HashMap<String, HashMap<String, f64>>,
}

/// Risk metrics calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub portfolio_id: String,
    pub timestamp: DateTime<Utc>,
    pub var_95: f64,
    pub var_99: f64,
    pub expected_shortfall: f64,
    pub beta: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub concentration_risk: f64,
    pub liquidity_risk: f64,
    pub calculation_time_ms: u64,
}

/// Risk threshold violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub id: String,
    pub portfolio_id: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub severity: ViolationSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub id: String,
    pub portfolio_id: String,
    pub alert_type: AlertType,
    pub message: String,
    pub severity: ViolationSeverity,
    pub timestamp: DateTime<Utc>,
    pub auto_response_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    ThresholdViolation,
    AnomalyDetected,
    SystemError,
    MarketEvent,
}

/// Risk model trait
#[async_trait::async_trait]
pub trait RiskModel: Send + Sync + std::fmt::Debug {
    async fn calculate_risk(&self, portfolio: &PortfolioUpdate, market_data: &MarketData) -> Result<RiskMetrics>;
    fn model_name(&self) -> &str;
    fn supported_metrics(&self) -> Vec<String>;
}

/// Value at Risk (VaR) model
#[derive(Debug)]
pub struct VaRModel {
    confidence_levels: Vec<f64>,
    lookback_days: u32,
}

impl VaRModel {
    pub fn new() -> Self {
        Self {
            confidence_levels: vec![0.95, 0.99],
            lookback_days: 252, // 1 year of trading days
        }
    }
}

#[async_trait::async_trait]
impl RiskModel for VaRModel {
    async fn calculate_risk(&self, portfolio: &PortfolioUpdate, market_data: &MarketData) -> Result<RiskMetrics> {
        let start_time = std::time::Instant::now();
        
        // Calculate portfolio value
        let portfolio_value: f64 = portfolio.positions.iter().map(|p| p.market_value).sum();
        
        // Calculate VaR using historical simulation method
        let var_95 = self.calculate_historical_var(portfolio, market_data, 0.95).await?;
        let var_99 = self.calculate_historical_var(portfolio, market_data, 0.99).await?;
        
        // Calculate Expected Shortfall (CVaR)
        let expected_shortfall = self.calculate_expected_shortfall(portfolio, market_data, 0.95).await?;
        
        // Calculate other risk metrics
        let beta = self.calculate_portfolio_beta(portfolio, market_data).await?;
        let sharpe_ratio = self.calculate_sharpe_ratio(portfolio, market_data).await?;
        let max_drawdown = self.calculate_max_drawdown(portfolio).await?;
        let concentration_risk = self.calculate_concentration_risk(portfolio).await?;
        let liquidity_risk = self.calculate_liquidity_risk(portfolio, market_data).await?;
        
        let calculation_time = start_time.elapsed().as_millis() as u64;
        
        Ok(RiskMetrics {
            portfolio_id: portfolio.portfolio_id.clone(),
            timestamp: Utc::now(),
            var_95,
            var_99,
            expected_shortfall,
            beta,
            sharpe_ratio,
            max_drawdown,
            concentration_risk,
            liquidity_risk,
            calculation_time_ms: calculation_time,
        })
    }

    fn model_name(&self) -> &str {
        "VaR_Model"
    }

    fn supported_metrics(&self) -> Vec<String> {
        vec![
            "var_95".to_string(),
            "var_99".to_string(),
            "expected_shortfall".to_string(),
            "beta".to_string(),
            "sharpe_ratio".to_string(),
            "max_drawdown".to_string(),
            "concentration_risk".to_string(),
            "liquidity_risk".to_string(),
        ]
    }
}

impl VaRModel {
    async fn calculate_historical_var(&self, portfolio: &PortfolioUpdate, _market_data: &MarketData, confidence: f64) -> Result<f64> {
        // Simplified VaR calculation
        // In a real implementation, this would use historical price data
        let portfolio_value: f64 = portfolio.positions.iter().map(|p| p.market_value).sum();
        let volatility = 0.02; // Assume 2% daily volatility
        
        // Normal distribution approximation
        let z_score = match confidence {
            0.95 => 1.645,
            0.99 => 2.326,
            _ => 1.645,
        };
        
        Ok(portfolio_value * volatility * z_score)
    }

    async fn calculate_expected_shortfall(&self, portfolio: &PortfolioUpdate, market_data: &MarketData, confidence: f64) -> Result<f64> {
        let var = self.calculate_historical_var(portfolio, market_data, confidence).await?;
        // ES is typically 1.2-1.5 times VaR for normal distributions
        Ok(var * 1.3)
    }

    async fn calculate_portfolio_beta(&self, portfolio: &PortfolioUpdate, _market_data: &MarketData) -> Result<f64> {
        // Simplified beta calculation
        // Weight-average of individual stock betas
        let mut weighted_beta = 0.0;
        let total_value: f64 = portfolio.positions.iter().map(|p| p.market_value).sum();
        
        for position in &portfolio.positions {
            let weight = position.market_value / total_value;
            let beta = 1.0; // Assume beta of 1.0 for simplicity
            weighted_beta += weight * beta;
        }
        
        Ok(weighted_beta)
    }

    async fn calculate_sharpe_ratio(&self, _portfolio: &PortfolioUpdate, _market_data: &MarketData) -> Result<f64> {
        // Simplified Sharpe ratio calculation
        let expected_return = 0.08; // 8% annual return
        let risk_free_rate = 0.02; // 2% risk-free rate
        let volatility = 0.15; // 15% volatility
        
        Ok((expected_return - risk_free_rate) / volatility)
    }

    async fn calculate_max_drawdown(&self, _portfolio: &PortfolioUpdate) -> Result<f64> {
        // Simplified max drawdown calculation
        // In a real implementation, this would analyze historical portfolio values
        Ok(0.15) // Assume 15% max drawdown
    }

    async fn calculate_concentration_risk(&self, portfolio: &PortfolioUpdate) -> Result<f64> {
        let total_value: f64 = portfolio.positions.iter().map(|p| p.market_value).sum();
        
        // Calculate Herfindahl-Hirschman Index (HHI)
        let mut hhi = 0.0;
        for position in &portfolio.positions {
            let weight = position.market_value / total_value;
            hhi += weight * weight;
        }
        
        Ok(hhi)
    }

    async fn calculate_liquidity_risk(&self, portfolio: &PortfolioUpdate, _market_data: &MarketData) -> Result<f64> {
        // Simplified liquidity risk calculation
        // In practice, this would consider bid-ask spreads, trading volumes, etc.
        let mut liquidity_score = 0.0;
        let total_value: f64 = portfolio.positions.iter().map(|p| p.market_value).sum();
        
        for position in &portfolio.positions {
            let weight = position.market_value / total_value;
            let liquidity = 0.95; // Assume 95% liquidity for simplicity
            liquidity_score += weight * liquidity;
        }
        
        Ok(1.0 - liquidity_score) // Higher score means higher risk
    }
}

/// Threshold monitor
#[derive(Debug)]
pub struct ThresholdMonitor {
    thresholds: Arc<RwLock<HashMap<String, ThresholdConfig>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub metric_name: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub enabled: bool,
}

impl ThresholdMonitor {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        
        // Default thresholds
        thresholds.insert("var_95".to_string(), ThresholdConfig {
            metric_name: "var_95".to_string(),
            warning_threshold: 1000000.0, // $1M
            critical_threshold: 5000000.0, // $5M
            enabled: true,
        });
        
        thresholds.insert("concentration_risk".to_string(), ThresholdConfig {
            metric_name: "concentration_risk".to_string(),
            warning_threshold: 0.3, // 30%
            critical_threshold: 0.5, // 50%
            enabled: true,
        });
        
        Self {
            thresholds: Arc::new(RwLock::new(thresholds)),
        }
    }

    pub async fn check_thresholds(&self, risk_metrics: &RiskMetrics) -> Result<Vec<ThresholdViolation>> {
        let thresholds = self.thresholds.read().await;
        let mut violations = Vec::new();
        
        // Check VaR 95%
        if let Some(config) = thresholds.get("var_95") {
            if config.enabled {
                if risk_metrics.var_95 > config.critical_threshold {
                    violations.push(ThresholdViolation {
                        id: Uuid::new_v4().to_string(),
                        portfolio_id: risk_metrics.portfolio_id.clone(),
                        metric_name: "var_95".to_string(),
                        current_value: risk_metrics.var_95,
                        threshold_value: config.critical_threshold,
                        severity: ViolationSeverity::Critical,
                        timestamp: Utc::now(),
                    });
                } else if risk_metrics.var_95 > config.warning_threshold {
                    violations.push(ThresholdViolation {
                        id: Uuid::new_v4().to_string(),
                        portfolio_id: risk_metrics.portfolio_id.clone(),
                        metric_name: "var_95".to_string(),
                        current_value: risk_metrics.var_95,
                        threshold_value: config.warning_threshold,
                        severity: ViolationSeverity::Medium,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        // Check concentration risk
        if let Some(config) = thresholds.get("concentration_risk") {
            if config.enabled {
                if risk_metrics.concentration_risk > config.critical_threshold {
                    violations.push(ThresholdViolation {
                        id: Uuid::new_v4().to_string(),
                        portfolio_id: risk_metrics.portfolio_id.clone(),
                        metric_name: "concentration_risk".to_string(),
                        current_value: risk_metrics.concentration_risk,
                        threshold_value: config.critical_threshold,
                        severity: ViolationSeverity::Critical,
                        timestamp: Utc::now(),
                    });
                } else if risk_metrics.concentration_risk > config.warning_threshold {
                    violations.push(ThresholdViolation {
                        id: Uuid::new_v4().to_string(),
                        portfolio_id: risk_metrics.portfolio_id.clone(),
                        metric_name: "concentration_risk".to_string(),
                        current_value: risk_metrics.concentration_risk,
                        threshold_value: config.warning_threshold,
                        severity: ViolationSeverity::Medium,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        Ok(violations)
    }

    pub async fn update_threshold(&self, metric_name: String, config: ThresholdConfig) {
        let mut thresholds = self.thresholds.write().await;
        thresholds.insert(metric_name, config);
    }
}

/// Alert system
#[derive(Debug)]
pub struct AlertSystem {
    alert_sender: mpsc::UnboundedSender<RiskAlert>,
    auto_response: Arc<AutoResponse>,
    config: RiskEngineConfig,
}

impl AlertSystem {
    pub fn new(config: RiskEngineConfig) -> (Self, mpsc::UnboundedReceiver<RiskAlert>) {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        let auto_response = Arc::new(AutoResponse::new());
        
        (Self {
            alert_sender,
            auto_response,
            config,
        }, alert_receiver)
    }

    pub async fn trigger_alerts(&self, violations: &[ThresholdViolation]) -> Result<()> {
        for violation in violations {
            let alert = RiskAlert {
                id: Uuid::new_v4().to_string(),
                portfolio_id: violation.portfolio_id.clone(),
                alert_type: AlertType::ThresholdViolation,
                message: format!(
                    "Risk threshold violated: {} = {:.2} exceeds threshold {:.2}",
                    violation.metric_name, violation.current_value, violation.threshold_value
                ),
                severity: violation.severity.clone(),
                timestamp: Utc::now(),
                auto_response_triggered: false,
            };

            // Send alert
            if let Err(e) = self.alert_sender.send(alert.clone()) {
                error!("Failed to send risk alert: {}", e);
            }

            // Trigger auto-response for critical violations
            if matches!(violation.severity, ViolationSeverity::Critical) {
                if let Err(e) = self.auto_response.execute_response(violation).await {
                    error!("Failed to execute auto-response: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// Automatic response system
#[derive(Debug)]
pub struct AutoResponse {
    response_actions: HashMap<String, ResponseAction>,
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    ReducePosition { symbol: String, percentage: f64 },
    HedgePosition { symbol: String, hedge_ratio: f64 },
    NotifyRiskManager { urgency: String },
    FreezeTrading { duration_minutes: u32 },
}

impl AutoResponse {
    pub fn new() -> Self {
        let mut response_actions = HashMap::new();
        
        // Default response actions
        response_actions.insert("var_95".to_string(), ResponseAction::NotifyRiskManager {
            urgency: "high".to_string(),
        });
        
        response_actions.insert("concentration_risk".to_string(), ResponseAction::ReducePosition {
            symbol: "largest_position".to_string(),
            percentage: 0.1, // Reduce by 10%
        });
        
        Self { response_actions }
    }

    pub async fn execute_response(&self, violation: &ThresholdViolation) -> Result<()> {
        if let Some(action) = self.response_actions.get(&violation.metric_name) {
            match action {
                ResponseAction::ReducePosition { symbol, percentage } => {
                    info!("Auto-response: Reducing position {} by {}%", symbol, percentage * 100.0);
                    // In a real implementation, this would interface with trading systems
                }
                ResponseAction::HedgePosition { symbol, hedge_ratio } => {
                    info!("Auto-response: Hedging position {} with ratio {}", symbol, hedge_ratio);
                }
                ResponseAction::NotifyRiskManager { urgency } => {
                    info!("Auto-response: Notifying risk manager with urgency: {}", urgency);
                }
                ResponseAction::FreezeTrading { duration_minutes } => {
                    info!("Auto-response: Freezing trading for {} minutes", duration_minutes);
                }
            }
        }

        Ok(())
    }
}

/// Risk metrics collector
#[derive(Debug)]
pub struct RiskMetricsCollector {
    pub risk_calculations: prometheus::Counter,
    pub calculation_latency: prometheus::Histogram,
    pub threshold_violations: prometheus::Counter,
    pub alerts_triggered: prometheus::Counter,
}

impl Default for RiskMetricsCollector {
    fn default() -> Self {
        Self {
            risk_calculations: prometheus::Counter::new("risk_calculations_total", "Total number of risk calculations").unwrap(),
            calculation_latency: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("risk_calculation_latency_seconds", "Risk calculation latency")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
            ).unwrap(),
            threshold_violations: prometheus::Counter::new("risk_threshold_violations_total", "Total number of threshold violations").unwrap(),
            alerts_triggered: prometheus::Counter::new("risk_alerts_triggered_total", "Total number of risk alerts triggered").unwrap(),
        }
    }
}

impl RealTimeRiskEngine {
    /// Create a new real-time risk engine
    pub async fn new(ducklake_manager: Arc<DuckLakeManager>, config: RiskEngineConfig) -> Result<Self> {
        let risk_models = Arc::new(RwLock::new(HashMap::new()));
        let threshold_monitor = Arc::new(ThresholdMonitor::new());
        let (alert_system, _alert_receiver) = AlertSystem::new(config.clone());
        let alert_system = Arc::new(alert_system);
        let metrics_collector = Arc::new(RiskMetricsCollector::default());

        // Add default VaR model
        {
            let mut models = risk_models.write().await;
            models.insert("var_model".to_string(), Arc::new(VaRModel::new()) as Arc<dyn RiskModel>);
        }

        Ok(Self {
            ducklake_manager,
            risk_models,
            threshold_monitor,
            alert_system,
            metrics_collector,
            config,
        })
    }

    /// Monitor portfolio stream for real-time risk calculation
    pub async fn monitor_portfolio_stream<S>(&self, mut portfolio_stream: S) -> Result<()>
    where
        S: Stream<Item = PortfolioUpdate> + Send + Unpin,
    {
        info!("Starting real-time risk monitoring");

        while let Some(update) = portfolio_stream.next().await {
            let start_time = std::time::Instant::now();

            // Calculate risk metrics
            let risk_metrics = self.calculate_risk_metrics(&update).await?;

            // Check thresholds
            let threshold_violations = self.threshold_monitor
                .check_thresholds(&risk_metrics).await?;

            // Trigger alerts if needed
            if !threshold_violations.is_empty() {
                self.alert_system.trigger_alerts(&threshold_violations).await?;
                self.metrics_collector.threshold_violations.inc_by(threshold_violations.len() as f64);
                self.metrics_collector.alerts_triggered.inc_by(threshold_violations.len() as f64);
            }

            // Store risk metrics in DuckLake
            self.store_risk_metrics(&risk_metrics).await?;

            // Update metrics
            let processing_time = start_time.elapsed();
            self.metrics_collector.risk_calculations.inc();
            self.metrics_collector.calculation_latency.observe(processing_time.as_secs_f64());

            // Ensure we meet the 100ms SLA
            if processing_time.as_millis() > self.config.max_processing_time_ms as u128 {
                warn!("Risk processing exceeded SLA: {}ms > {}ms", 
                      processing_time.as_millis(), self.config.max_processing_time_ms);
            }

            debug!("Processed portfolio {} in {}ms", update.portfolio_id, processing_time.as_millis());
        }

        info!("Real-time risk monitoring completed");
        Ok(())
    }

    /// Calculate risk metrics for a portfolio update
    async fn calculate_risk_metrics(&self, portfolio_update: &PortfolioUpdate) -> Result<RiskMetrics> {
        let models = self.risk_models.read().await;
        
        // Use the VaR model for calculation
        if let Some(var_model) = models.get("var_model") {
            var_model.calculate_risk(portfolio_update, &portfolio_update.market_data).await
        } else {
            Err(DuckHubError::database("No risk model available".to_string()))
        }
    }

    /// Store risk metrics in DuckLake
    async fn store_risk_metrics(&self, risk_metrics: &RiskMetrics) -> Result<()> {
        let sql = format!(
            "INSERT INTO risk_metrics (portfolio_id, timestamp, var_95, var_99, expected_shortfall, beta, sharpe_ratio, max_drawdown, concentration_risk, liquidity_risk, calculation_time_ms) VALUES ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {})",
            risk_metrics.portfolio_id,
            risk_metrics.timestamp.to_rfc3339(),
            risk_metrics.var_95,
            risk_metrics.var_99,
            risk_metrics.expected_shortfall,
            risk_metrics.beta,
            risk_metrics.sharpe_ratio,
            risk_metrics.max_drawdown,
            risk_metrics.concentration_risk,
            risk_metrics.liquidity_risk,
            risk_metrics.calculation_time_ms
        );

        self.ducklake_manager.execute_query("default", &sql).await?;
        debug!("Stored risk metrics for portfolio: {}", risk_metrics.portfolio_id);
        Ok(())
    }

    /// Get current risk metrics for a portfolio
    pub async fn get_current_risk_metrics(&self, portfolio_id: &str) -> Result<Option<RiskMetrics>> {
        let sql = format!(
            "SELECT * FROM risk_metrics WHERE portfolio_id = '{}' ORDER BY timestamp DESC LIMIT 1",
            portfolio_id
        );

        let result = self.ducklake_manager.execute_query("default", &sql).await?;
        
        // Convert query result to RiskMetrics
        // This is a simplified implementation
        Ok(None) // TODO: Implement proper result parsing
    }

    /// Add a new risk model
    pub async fn add_risk_model(&self, name: String, model: Arc<dyn RiskModel>) {
        let mut models = self.risk_models.write().await;
        models.insert(name, model);
    }

    /// Get metrics collector
    pub fn get_metrics(&self) -> Arc<RiskMetricsCollector> {
        self.metrics_collector.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_var_model_calculation() {
        let model = VaRModel::new();
        
        let portfolio = PortfolioUpdate {
            portfolio_id: "test_portfolio".to_string(),
            timestamp: Utc::now(),
            positions: vec![
                Position {
                    symbol: "AAPL".to_string(),
                    quantity: 100.0,
                    price: 150.0,
                    market_value: 15000.0,
                    currency: "USD".to_string(),
                },
                Position {
                    symbol: "GOOGL".to_string(),
                    quantity: 50.0,
                    price: 2500.0,
                    market_value: 125000.0,
                    currency: "USD".to_string(),
                },
            ],
            market_data: MarketData {
                timestamp: Utc::now(),
                prices: HashMap::new(),
                volatilities: HashMap::new(),
                correlations: HashMap::new(),
            },
        };

        let risk_metrics = model.calculate_risk(&portfolio, &portfolio.market_data).await.unwrap();
        
        assert!(risk_metrics.var_95 > 0.0);
        assert!(risk_metrics.var_99 > risk_metrics.var_95);
        assert!(risk_metrics.expected_shortfall > risk_metrics.var_95);
        assert!(risk_metrics.calculation_time_ms < 100); // Should be fast
    }

    #[tokio::test]
    async fn test_threshold_monitor() {
        let monitor = ThresholdMonitor::new();
        
        let risk_metrics = RiskMetrics {
            portfolio_id: "test".to_string(),
            timestamp: Utc::now(),
            var_95: 2000000.0, // $2M - should trigger warning
            var_99: 3000000.0,
            expected_shortfall: 2500000.0,
            beta: 1.2,
            sharpe_ratio: 0.8,
            max_drawdown: 0.15,
            concentration_risk: 0.4, // 40% - should trigger warning
            liquidity_risk: 0.05,
            calculation_time_ms: 50,
        };

        let violations = monitor.check_thresholds(&risk_metrics).await.unwrap();
        assert_eq!(violations.len(), 2); // VaR and concentration risk violations
    }
}