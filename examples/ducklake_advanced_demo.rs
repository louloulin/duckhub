//! DuckLake Advanced Features Demo
//! 
//! This example demonstrates advanced DuckLake features including:
//! - ACID transactions
//! - Time travel queries
//! - Schema evolution
//! - Snapshot management
//! - Multi-database operations

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

    println!("🦆 DuckLake Advanced Features Demo Starting...\n");

    // Create database configuration with DuckLake support
    let config = DatabaseConfig {
        duckdb_path: ":memory:".to_string(),
        memory_limit: Some("2GB".to_string()),
        threads: Some(4),
        max_memory: None,
        temp_directory: None,
        extensions: vec![
            "ducklake".to_string(),  // Native DuckLake support
            "httpfs".to_string(),    // For cloud storage
            "parquet".to_string(),   // For Parquet files
            "json".to_string(),      // For JSON data
        ],
        pool: PoolConfig::default(),
    };

    // Create DuckDB engine
    let engine = Arc::new(DuckDBEngine::new(config)?);
    
    println!("✅ DuckDB engine created with DuckLake support");

    // Enable DuckLake features
    engine.enable_data_lake_features(vec![
        DataLakeFeature::DuckLake,
        DataLakeFeature::S3Access,
        DataLakeFeature::ParquetFiles,
    ]).await?;

    println!("✅ DuckLake features enabled");

    // Demo 1: Basic DuckLake database operations
    println!("\n📁 Demo 1: Basic DuckLake Operations...");
    demo_basic_ducklake_operations(&engine).await?;

    // Demo 2: ACID Transactions
    println!("\n🔒 Demo 2: ACID Transactions...");
    demo_acid_transactions(&engine).await?;

    // Demo 3: Time Travel Queries
    println!("\n⏰ Demo 3: Time Travel Queries...");
    demo_time_travel(&engine).await?;

    // Demo 4: Schema Evolution
    println!("\n🔄 Demo 4: Schema Evolution...");
    demo_schema_evolution(&engine).await?;

    // Demo 5: Multi-Database Operations
    println!("\n🗄️  Demo 5: Multi-Database Operations...");
    demo_multi_database(&engine).await?;

    // Demo 6: Performance Comparison
    println!("\n⚡ Demo 6: Performance Comparison...");
    demo_performance_comparison(&engine).await?;

    println!("\n🎉 DuckLake Advanced Demo completed successfully!");
    Ok(())
}

async fn demo_basic_ducklake_operations(engine: &DuckDBEngine) -> Result<()> {
    // Create DuckLake secret for easier connection management
    let query = Query {
        id: generate_id(),
        sql: r#"
            CREATE SECRET financial_lake (
                TYPE DUCKLAKE,
                METADATA_PATH 'financial_data.ducklake',
                DATA_PATH 'financial_data.files'
            )
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&query).await?;
    println!("  ✅ Created DuckLake secret for financial data");

    // Attach DuckLake database
    let attach_query = Query {
        id: generate_id(),
        sql: "ATTACH 'ducklake:financial_data.ducklake' AS financial_db".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&attach_query).await?;
    println!("  ✅ Attached DuckLake database: financial_db");

    // Create a table in DuckLake
    let create_table_query = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE financial_db.transactions (
                transaction_id VARCHAR PRIMARY KEY,
                account_id VARCHAR NOT NULL,
                amount DECIMAL(15,2) NOT NULL,
                transaction_type VARCHAR NOT NULL,
                transaction_date DATE NOT NULL,
                description VARCHAR,
                metadata JSON,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&create_table_query).await?;
    println!("  ✅ Created transactions table in DuckLake");

    // Insert sample data
    let insert_query = Query {
        id: generate_id(),
        sql: r#"
            INSERT INTO financial_db.transactions 
            (transaction_id, account_id, amount, transaction_type, transaction_date, description, metadata)
            VALUES 
            ('TXN-001', 'ACC-001', 1500.00, 'CREDIT', '2024-01-15', 'Salary deposit', '{"source": "payroll", "category": "income"}'),
            ('TXN-002', 'ACC-001', -250.00, 'DEBIT', '2024-01-16', 'Grocery shopping', '{"merchant": "SuperMart", "category": "food"}'),
            ('TXN-003', 'ACC-002', 5000.00, 'CREDIT', '2024-01-17', 'Investment return', '{"portfolio": "growth", "category": "investment"}'),
            ('TXN-004', 'ACC-001', -75.50, 'DEBIT', '2024-01-18', 'Utility bill', '{"utility": "electricity", "category": "bills"}'),
            ('TXN-005', 'ACC-002', -1200.00, 'DEBIT', '2024-01-19', 'Rent payment', '{"property": "main_residence", "category": "housing"}')
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&insert_query).await?;
    println!("  ✅ Inserted sample transaction data");

    // Query the data
    let select_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                account_id,
                COUNT(*) as transaction_count,
                SUM(amount) as total_amount,
                AVG(amount) as avg_amount
            FROM financial_db.transactions 
            GROUP BY account_id
            ORDER BY account_id
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&select_query).await?;
    println!("  📊 Account summary:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    Ok(())
}

async fn demo_acid_transactions(engine: &DuckDBEngine) -> Result<()> {
    println!("  🔒 Demonstrating ACID transaction properties...");

    // Begin transaction
    let begin_query = Query {
        id: generate_id(),
        sql: "BEGIN TRANSACTION".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&begin_query).await?;
    println!("  ✅ Started transaction");

    // Insert multiple related records
    let batch_insert = Query {
        id: generate_id(),
        sql: r#"
            INSERT INTO financial_db.transactions 
            (transaction_id, account_id, amount, transaction_type, transaction_date, description, metadata)
            VALUES 
            ('TXN-006', 'ACC-001', -500.00, 'DEBIT', '2024-01-20', 'Transfer out', '{"transfer_to": "ACC-002", "batch_id": "BATCH-001"}'),
            ('TXN-007', 'ACC-002', 500.00, 'CREDIT', '2024-01-20', 'Transfer in', '{"transfer_from": "ACC-001", "batch_id": "BATCH-001"}')
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&batch_insert).await?;
    println!("  ✅ Inserted transfer transactions");

    // Verify data consistency within transaction
    let verify_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                SUM(CASE WHEN metadata->>'$.batch_id' = 'BATCH-001' THEN amount ELSE 0 END) as batch_total
            FROM financial_db.transactions
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&verify_query).await?;
    println!("  📊 Batch total (should be 0 for balanced transfer): {:?}", result.rows[0]);

    // Commit transaction
    let commit_query = Query {
        id: generate_id(),
        sql: "COMMIT".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&commit_query).await?;
    println!("  ✅ Transaction committed successfully");

    Ok(())
}

async fn demo_time_travel(engine: &DuckDBEngine) -> Result<()> {
    println!("  ⏰ Demonstrating time travel capabilities...");

    // Get current snapshot information
    let snapshot_query = Query {
        id: generate_id(),
        sql: "SELECT * FROM financial_db.snapshots() ORDER BY snapshot_id DESC LIMIT 3".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    match engine.execute_query(&snapshot_query).await {
        Ok(result) => {
            println!("  📸 Recent snapshots:");
            for row in &result.rows {
                println!("    • Snapshot: {:?}", row);
            }
        }
        Err(_) => {
            println!("  ⚠️  Snapshot information not available (this is expected in memory database)");
        }
    }

    // Demonstrate time travel query syntax (would work with persistent DuckLake)
    println!("  💡 Time travel query examples:");
    println!("    • Query at specific version: SELECT * FROM financial_db.transactions AT (VERSION => 1)");
    println!("    • Query at timestamp: SELECT * FROM financial_db.transactions AT (TIMESTAMP => '2024-01-20 10:00:00')");
    println!("    • Query between versions: SELECT * FROM financial_db.transactions FOR SYSTEM_VERSION BETWEEN 1 AND 3");

    Ok(())
}

async fn demo_schema_evolution(engine: &DuckDBEngine) -> Result<()> {
    println!("  🔄 Demonstrating schema evolution...");

    // Add a new column to existing table
    let alter_query = Query {
        id: generate_id(),
        sql: r#"
            ALTER TABLE financial_db.transactions 
            ADD COLUMN risk_score INTEGER DEFAULT 0
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&alter_query).await?;
    println!("  ✅ Added risk_score column to transactions table");

    // Update some records with risk scores
    let update_query = Query {
        id: generate_id(),
        sql: r#"
            UPDATE financial_db.transactions 
            SET risk_score = CASE 
                WHEN ABS(amount) > 1000 THEN 3
                WHEN ABS(amount) > 500 THEN 2
                ELSE 1
            END
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&update_query).await?;
    println!("  ✅ Updated risk scores based on transaction amounts");

    // Query with new column
    let enhanced_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                transaction_type,
                AVG(ABS(amount)) as avg_amount,
                AVG(risk_score) as avg_risk_score,
                COUNT(*) as count
            FROM financial_db.transactions 
            GROUP BY transaction_type
            ORDER BY avg_risk_score DESC
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&enhanced_query).await?;
    println!("  📊 Risk analysis by transaction type:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    Ok(())
}

async fn demo_multi_database(engine: &DuckDBEngine) -> Result<()> {
    println!("  🗄️  Demonstrating multi-database operations...");

    // Create a second DuckLake database for reference data
    let create_ref_db = Query {
        id: generate_id(),
        sql: "ATTACH 'ducklake:reference_data.ducklake' AS ref_db".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&create_ref_db).await?;
    println!("  ✅ Attached reference database");

    // Create reference tables
    let create_accounts_table = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE ref_db.accounts (
                account_id VARCHAR PRIMARY KEY,
                account_type VARCHAR NOT NULL,
                account_name VARCHAR NOT NULL,
                opening_date DATE NOT NULL,
                status VARCHAR DEFAULT 'ACTIVE'
            )
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&create_accounts_table).await?;

    // Insert reference data
    let insert_accounts = Query {
        id: generate_id(),
        sql: r#"
            INSERT INTO ref_db.accounts VALUES 
            ('ACC-001', 'CHECKING', 'Primary Checking', '2023-01-01', 'ACTIVE'),
            ('ACC-002', 'SAVINGS', 'High Yield Savings', '2023-06-15', 'ACTIVE'),
            ('ACC-003', 'INVESTMENT', 'Growth Portfolio', '2023-03-10', 'ACTIVE')
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&insert_accounts).await?;
    println!("  ✅ Created and populated accounts reference table");

    // Cross-database query
    let cross_db_query = Query {
        id: generate_id(),
        sql: r#"
            SELECT 
                a.account_id,
                a.account_type,
                a.account_name,
                COUNT(t.transaction_id) as transaction_count,
                COALESCE(SUM(t.amount), 0) as total_amount,
                COALESCE(AVG(t.risk_score), 0) as avg_risk_score
            FROM ref_db.accounts a
            LEFT JOIN financial_db.transactions t ON a.account_id = t.account_id
            GROUP BY a.account_id, a.account_type, a.account_name
            ORDER BY total_amount DESC
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = engine.execute_query(&cross_db_query).await?;
    println!("  📊 Cross-database account analysis:");
    for row in &result.rows {
        println!("    • {:?}", row);
    }

    Ok(())
}

async fn demo_performance_comparison(engine: &DuckDBEngine) -> Result<()> {
    println!("  ⚡ Performance comparison: DuckLake vs regular tables...");

    // Create a regular table for comparison
    let create_regular_table = Query {
        id: generate_id(),
        sql: r#"
            CREATE TABLE regular_transactions AS 
            SELECT * FROM financial_db.transactions
        "#.to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    engine.execute_query(&create_regular_table).await?;

    // Performance test query
    let test_query = r#"
        SELECT 
            account_id,
            transaction_type,
            COUNT(*) as count,
            SUM(amount) as total,
            AVG(amount) as average
        FROM {} 
        GROUP BY account_id, transaction_type
        ORDER BY total DESC
    "#;

    // Test DuckLake table
    let ducklake_start = std::time::Instant::now();
    let ducklake_query = Query {
        id: generate_id(),
        sql: test_query.replace("{}", "financial_db.transactions"),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };
    engine.execute_query(&ducklake_query).await?;
    let ducklake_time = ducklake_start.elapsed();

    // Test regular table
    let regular_start = std::time::Instant::now();
    let regular_query = Query {
        id: generate_id(),
        sql: test_query.replace("{}", "regular_transactions"),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };
    engine.execute_query(&regular_query).await?;
    let regular_time = regular_start.elapsed();

    println!("  📊 Performance Results:");
    println!("    • DuckLake table: {:.2}ms", ducklake_time.as_millis());
    println!("    • Regular table: {:.2}ms", regular_time.as_millis());
    println!("    • DuckLake provides ACID guarantees, time travel, and schema evolution");
    println!("    • Regular tables are optimized for pure query performance");

    Ok(())
}
