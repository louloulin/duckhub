//! DuckDB Lake demonstration
//! 
//! This example shows how to use DuckDB with data lake functionality,
//! including S3 access, Parquet files, and Delta Lake support.

use duckhub_common::prelude::*;
use duckhub_database::*;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("🦆 DuckDB Lake Demo Starting...\n");

    // Create database configuration with data lake extensions
    let config = DatabaseConfig {
        duckdb_path = ":memory:".to_string(),
        memory_limit: Some("2GB".to_string()),
        threads: Some(4),
        max_memory: None,
        temp_directory: None,
        extensions: vec![
            "httpfs".to_string(),
            "parquet".to_string(),
            "delta".to_string(),
            "json".to_string(),
        ],
        pool: PoolConfig::default(),
    };

    // Create DuckDB engine
    let engine = Arc::new(DuckDBEngine::new(config)?);
    
    println!("✅ DuckDB engine created with data lake extensions");

    // Test data lake connectivity
    println!("\n🔍 Testing data lake connectivity...");
    let connectivity_report = engine.test_data_lake_connectivity().await?;
    
    println!("📊 Data Lake Connectivity Report:");
    println!("  • S3 Access: {}", if connectivity_report.s3_available { "✅" } else { "❌" });
    println!("  • Parquet Support: {}", if connectivity_report.parquet_support { "✅" } else { "❌" });
    println!("  • Delta Lake Support: {}", if connectivity_report.delta_support { "✅" } else { "❌" });
    println!("  • JSON Support: {}", if connectivity_report.json_support { "✅" } else { "❌" });

    // Demo 1: Query public dataset from S3
    println!("\n📁 Demo 1: Querying public Parquet file from S3...");
    demo_s3_parquet_query(&engine).await?;

    // Demo 2: Create external table from data lake
    println!("\n🏗️  Demo 2: Creating external table from data lake...");
    demo_external_table(&engine).await?;

    // Demo 3: Work with JSON data
    println!("\n📄 Demo 3: Working with JSON data...");
    demo_json_processing(&engine).await?;

    // Demo 4: Delta Lake operations (if available)
    if connectivity_report.delta_support {
        println!("\n🔺 Demo 4: Delta Lake operations...");
        demo_delta_lake(&engine).await?;
    }

    // Demo 5: Performance comparison
    println!("\n⚡ Demo 5: Performance comparison...");
    demo_performance_comparison(&engine).await?;

    println!("\n🎉 DuckDB Lake Demo completed successfully!");
    Ok(())
}

async fn demo_s3_parquet_query(engine: &DuckDBEngine) -> Result<()> {
    // Query a public dataset (NYC taxi data sample)
    let query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                COUNT(*) as total_trips,
                AVG(trip_distance) as avg_distance,
                AVG(total_amount) as avg_fare
            FROM 's3://duckdb-md-dataset-121/part-00000-cc9a08d6-9c52-4d46-9e1b-7a8d8b0b0e1a-c000.snappy.parquet'
            LIMIT 1
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: Some(30),
    };

    match engine.execute_query(&query).await {
        Ok(result) => {
            println!("  ✅ Successfully queried S3 Parquet file");
            println!("  📊 Results: {} rows returned", result.row_count);
            if let Some(row) = result.rows.first() {
                println!("  🚕 Sample data: {:?}", row);
            }
        }
        Err(e) => {
            println!("  ⚠️  S3 query failed (this is expected without proper credentials): {}", e);
            println!("  💡 To enable S3 access, configure AWS credentials");
        }
    }

    Ok(())
}

async fn demo_external_table(engine: &DuckDBEngine) -> Result<()> {
    // Create a sample dataset first
    let create_sample = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE sample_financial_data AS 
            SELECT 
                row_number() OVER () as transaction_id,
                'ACME-' || (row_number() OVER ()) as account_id,
                random() * 10000 as amount,
                CASE 
                    WHEN random() > 0.5 THEN 'CREDIT' 
                    ELSE 'DEBIT' 
                END as transaction_type,
                current_date - INTERVAL (random() * 365) DAY as transaction_date
            FROM range(1000)
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&create_sample).await?;
    println!("  ✅ Created sample financial dataset (1000 transactions)");

    // Query the sample data
    let analysis_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                transaction_type,
                COUNT(*) as count,
                SUM(amount) as total_amount,
                AVG(amount) as avg_amount,
                MIN(transaction_date) as earliest_date,
                MAX(transaction_date) as latest_date
            FROM sample_financial_data 
            GROUP BY transaction_type
            ORDER BY transaction_type
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&analysis_query).await?;
    println!("  📊 Financial data analysis:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    Ok(())
}

async fn demo_json_processing(engine: &DuckDBEngine) -> Result<()> {
    // Create JSON data
    let json_query = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE json_transactions AS
            SELECT 
                row_number() OVER () as id,
                {
                    'transaction_id': 'TXN-' || row_number() OVER (),
                    'amount': random() * 1000,
                    'currency': CASE WHEN random() > 0.7 THEN 'EUR' WHEN random() > 0.4 THEN 'GBP' ELSE 'USD' END,
                    'metadata': {
                        'source': 'mobile_app',
                        'location': CASE WHEN random() > 0.5 THEN 'US' ELSE 'UK' END
                    }
                } as transaction_json
            FROM range(100)
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&json_query).await?;
    println!("  ✅ Created JSON transaction data");

    // Query JSON data
    let json_analysis = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                transaction_json->>'$.currency' as currency,
                COUNT(*) as transaction_count,
                SUM(CAST(transaction_json->>'$.amount' AS DOUBLE)) as total_amount,
                transaction_json->'$.metadata'->>'$.location' as location
            FROM json_transactions 
            GROUP BY currency, location
            ORDER BY currency, location
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&json_analysis).await?;
    println!("  📊 JSON data analysis by currency and location:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    Ok(())
}

async fn demo_delta_lake(engine: &DuckDBEngine) -> Result<()> {
    println!("  🔺 Delta Lake functionality is available!");
    println!("  💡 Delta Lake allows for ACID transactions on data lakes");
    println!("  📝 Example operations would include:");
    println!("    • Reading Delta tables: SELECT * FROM delta_scan('path/to/delta/table')");
    println!("    • Time travel: SELECT * FROM delta_scan('path') VERSION AS OF 1");
    println!("    • Schema evolution and ACID guarantees");
    
    // Note: Actual Delta Lake operations would require a real Delta table
    // This is just demonstrating the capability
    
    Ok(())
}

async fn demo_performance_comparison(engine: &DuckDBEngine) -> Result<()> {
    // Create a larger dataset for performance testing
    let create_large_dataset = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE performance_test AS 
            SELECT 
                row_number() OVER () as id,
                'USER-' || (row_number() OVER () % 10000) as user_id,
                random() * 1000 as amount,
                current_date - INTERVAL (random() * 365) DAY as date,
                CASE 
                    WHEN random() > 0.8 THEN 'HIGH' 
                    WHEN random() > 0.5 THEN 'MEDIUM' 
                    ELSE 'LOW' 
                END as risk_category
            FROM range(100000)
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let start_time = std::time::Instant::now();
    engine.execute_query(&create_large_dataset).await?;
    let creation_time = start_time.elapsed();
    
    println!("  ✅ Created 100K row dataset in {:.2}ms", creation_time.as_millis());

    // Perform aggregation query
    let aggregation_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                risk_category,
                COUNT(*) as transaction_count,
                SUM(amount) as total_amount,
                AVG(amount) as avg_amount,
                MIN(amount) as min_amount,
                MAX(amount) as max_amount,
                STDDEV(amount) as amount_stddev
            FROM performance_test 
            GROUP BY risk_category
            ORDER BY total_amount DESC
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let start_time = std::time::Instant::now();
    let result = engine.execute_query(&aggregation_query).await?;
    let query_time = start_time.elapsed();

    println!("  ⚡ Aggregation query on 100K rows completed in {:.2}ms", query_time.as_millis());
    println!("  📊 Performance results:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    // Calculate throughput
    let throughput = 100_000.0 / query_time.as_secs_f64();
    println!("  🚀 Throughput: {:.0} rows/second", throughput);

    Ok(())
}
