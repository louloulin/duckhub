//! DuckLake Real-time Financial Processing Demo
//! 
//! This example demonstrates the enhanced DuckLake capabilities including:
//! - Real-time stream processing
//! - Risk monitoring
//! - Memory optimization
//! - Financial data analysis

use duckhub_database::{
    DuckLakeManager, DuckLakeConfig,
    RealTimeFinancialProcessor, RealTimeProcessorConfig,
    FinancialData, TradeData, TradeSide, OrderType,
    MemoryManager, MemoryPoolConfig,
};
use duckhub_database::real_duckdb::Connection;
use tokio_stream::{self as stream, StreamExt};
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();

    info!("🚀 Starting DuckLake Real-time Financial Processing Demo");

    // 1. Create DuckDB connection
    let connection = Connection::open(":memory:").await?;
    info!("✅ Created DuckDB connection");

    // 2. Create DuckLake manager
    let ducklake_manager = Arc::new(DuckLakeManager::new(connection).await?);
    info!("✅ Created DuckLake manager");

    // 3. Initialize database schema
    setup_database_schema(&ducklake_manager).await?;
    info!("✅ Set up database schema");

    // 4. Create real-time financial processor
    let processor_config = RealTimeProcessorConfig {
        max_processing_latency_ms: 100,
        batch_size: 1000,
        enable_risk_monitoring: true,
        enable_trade_analysis: true,
        enable_market_data_processing: true,
        memory_optimization_enabled: true,
        auto_scaling_enabled: true,
        performance_monitoring_interval_ms: 1000,
    };

    let processor = RealTimeFinancialProcessor::new(
        ducklake_manager.clone(),
        processor_config,
    ).await?;
    info!("✅ Created real-time financial processor");

    // 5. Generate sample financial data stream
    let financial_stream = generate_sample_financial_data();
    info!("✅ Generated sample financial data stream");

    // 6. Process the stream
    info!("🔄 Starting real-time processing...");
    let start_time = std::time::Instant::now();

    processor.process_financial_stream(financial_stream).await?;

    let processing_time = start_time.elapsed();
    info!("✅ Completed processing in {:.2}ms", processing_time.as_millis());

    // 7. Display processing statistics
    display_processing_stats(&processor).await?;

    // 8. Query processed data
    query_processed_data(&ducklake_manager).await?;

    // 9. Demonstrate memory optimization
    demonstrate_memory_optimization().await?;

    info!("🎉 Demo completed successfully!");
    Ok(())
}

/// Set up database schema for the demo
async fn setup_database_schema(ducklake_manager: &DuckLakeManager) -> Result<(), Box<dyn std::error::Error>> {
    // Create trades table
    let create_trades_sql = "
        CREATE TABLE IF NOT EXISTS trades (
            trade_id VARCHAR PRIMARY KEY,
            symbol VARCHAR NOT NULL,
            side VARCHAR NOT NULL,
            quantity DOUBLE NOT NULL,
            price DOUBLE NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            account_id VARCHAR NOT NULL,
            order_type VARCHAR NOT NULL,
            execution_venue VARCHAR NOT NULL
        )
    ";
    ducklake_manager.execute_query("default", create_trades_sql).await?;

    // Create trade analysis table
    let create_analysis_sql = "
        CREATE TABLE IF NOT EXISTS trade_analysis (
            trade_id VARCHAR PRIMARY KEY,
            risk_score DOUBLE NOT NULL,
            compliance_status VARCHAR NOT NULL,
            analysis_time_ms BIGINT NOT NULL
        )
    ";
    ducklake_manager.execute_query("default", create_analysis_sql).await?;

    // Create market data table
    let create_market_data_sql = "
        CREATE TABLE IF NOT EXISTS market_data (
            symbol VARCHAR NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            bid_price DOUBLE NOT NULL,
            ask_price DOUBLE NOT NULL,
            last_price DOUBLE NOT NULL,
            volume BIGINT NOT NULL,
            high DOUBLE NOT NULL,
            low DOUBLE NOT NULL,
            open DOUBLE NOT NULL,
            close DOUBLE NOT NULL
        )
    ";
    ducklake_manager.execute_query("default", create_market_data_sql).await?;

    // Create risk metrics table
    let create_risk_sql = "
        CREATE TABLE IF NOT EXISTS risk_metrics (
            portfolio_id VARCHAR NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            var_95 DOUBLE NOT NULL,
            var_99 DOUBLE NOT NULL,
            expected_shortfall DOUBLE NOT NULL,
            beta DOUBLE NOT NULL,
            sharpe_ratio DOUBLE NOT NULL,
            max_drawdown DOUBLE NOT NULL,
            concentration_risk DOUBLE NOT NULL,
            liquidity_risk DOUBLE NOT NULL,
            calculation_time_ms BIGINT NOT NULL
        )
    ";
    ducklake_manager.execute_query("default", create_risk_sql).await?;

    info!("📊 Created database tables: trades, trade_analysis, market_data, risk_metrics");
    Ok(())
}

/// Generate sample financial data for demonstration
fn generate_sample_financial_data() -> impl tokio_stream::Stream<Item = FinancialData> {
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"];
    let mut trades = Vec::new();

    // Generate 1000 sample trades
    for i in 0..1000 {
        let symbol = symbols[i % symbols.len()];
        let trade = FinancialData::Trade(TradeData {
            trade_id: format!("TRADE_{:06}", i),
            symbol: symbol.to_string(),
            side: if i % 2 == 0 { TradeSide::Buy } else { TradeSide::Sell },
            quantity: 100.0 + (i as f64 * 10.0),
            price: 100.0 + (i as f64 * 0.1),
            timestamp: Utc::now(),
            account_id: format!("ACCOUNT_{}", i % 10),
            order_type: match i % 4 {
                0 => OrderType::Market,
                1 => OrderType::Limit,
                2 => OrderType::Stop,
                _ => OrderType::StopLimit,
            },
            execution_venue: "NASDAQ".to_string(),
        });
        trades.push(trade);
    }

    stream::iter(trades)
}

/// Display processing statistics
async fn display_processing_stats(processor: &RealTimeFinancialProcessor) -> Result<(), Box<dyn std::error::Error>> {
    info!("📈 Processing Statistics:");
    
    let throughput = processor.get_current_throughput().await;
    info!("   • Current throughput: {:.2} records/second", throughput);

    let memory_stats = processor.get_memory_stats().await?;
    info!("   • Memory statistics: {}", memory_stats);

    Ok(())
}

/// Query processed data to verify results
async fn query_processed_data(ducklake_manager: &DuckLakeManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Querying processed data:");

    // Count trades
    let trade_count_result = ducklake_manager.execute_query("default", "SELECT COUNT(*) as count FROM trades").await?;
    info!("   • Total trades processed: Query executed successfully");

    // Count trade analyses
    let analysis_count_result = ducklake_manager.execute_query("default", "SELECT COUNT(*) as count FROM trade_analysis").await?;
    info!("   • Total trade analyses: Query executed successfully");

    // Sample trade data
    let sample_trades_result = ducklake_manager.execute_query("default", "SELECT trade_id, symbol, side, quantity, price FROM trades LIMIT 5").await?;
    info!("   • Sample trades: Query executed successfully");

    // Risk score statistics
    let risk_stats_result = ducklake_manager.execute_query("default", "SELECT AVG(risk_score) as avg_risk, MAX(risk_score) as max_risk FROM trade_analysis").await?;
    info!("   • Risk statistics: Query executed successfully");

    Ok(())
}

/// Demonstrate memory optimization features
async fn demonstrate_memory_optimization() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧠 Demonstrating memory optimization:");

    // Create memory manager
    let memory_config = MemoryPoolConfig {
        initial_pool_size: 100,
        max_pool_size: 1000,
        chunk_size: 4096,
        enable_monitoring: true,
        gc_threshold: 0.8,
        preallocation_enabled: true,
    };

    let memory_manager = MemoryManager::new(memory_config).await?;
    info!("   • Created memory manager with optimized configuration");

    // Demonstrate buffer pool usage
    let buffer_pool = memory_manager.buffer_pool();
    let mut buffers = Vec::new();

    // Allocate some buffers
    for i in 0..10 {
        let mut buffer = buffer_pool.get().await;
        buffer.extend_from_slice(format!("Sample data {}", i).as_bytes());
        buffers.push(buffer);
    }
    info!("   • Allocated 10 buffers from pool");

    // Drop buffers (they return to pool automatically)
    drop(buffers);
    info!("   • Released buffers back to pool");

    // Check pool statistics
    let pool_size = buffer_pool.size().await;
    info!("   • Buffer pool size after release: {}", pool_size);

    // Demonstrate garbage collection
    let freed_chunks = memory_manager.garbage_collect_all().await?;
    info!("   • Garbage collection freed {} chunks", freed_chunks);

    // Get comprehensive memory statistics
    let memory_stats = memory_manager.get_memory_stats().await;
    info!("   • Memory pool usage: {:.1}%", memory_stats.memory_pool.usage_ratio * 100.0);
    info!("   • Total memory chunks: {}", memory_stats.memory_pool.total_chunks);

    Ok(())
}

/// Performance benchmark
#[allow(dead_code)]
async fn run_performance_benchmark(processor: &RealTimeFinancialProcessor) -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ Running performance benchmark:");

    let benchmark_sizes = vec![1000, 5000, 10000, 50000];

    for size in benchmark_sizes {
        let start_time = std::time::Instant::now();
        
        // Generate benchmark data
        let benchmark_stream = stream::iter((0..size).map(|i| {
            FinancialData::Trade(TradeData {
                trade_id: format!("BENCH_{:06}", i),
                symbol: "BENCHMARK".to_string(),
                side: TradeSide::Buy,
                quantity: 100.0,
                price: 100.0,
                timestamp: Utc::now(),
                account_id: "BENCH_ACCOUNT".to_string(),
                order_type: OrderType::Market,
                execution_venue: "BENCHMARK".to_string(),
            })
        }));

        // Process benchmark data
        processor.process_financial_stream(benchmark_stream).await?;

        let processing_time = start_time.elapsed();
        let throughput = size as f64 / processing_time.as_secs_f64();

        info!("   • {} records: {:.2}ms ({:.0} records/sec)", 
              size, processing_time.as_millis(), throughput);
    }

    Ok(())
}