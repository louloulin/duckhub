use duckhub_database::ducklake_real::DuckLakeManager;
use duckhub_database::real_duckdb::Connection;
use duckhub_common::types::{Schema, Field, DataType};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦆 DuckLake Manager Example");
    println!("===========================");

    // Create an in-memory DuckDB connection
    let connection = Connection::open_in_memory().await?;
    println!("✅ Created in-memory DuckDB connection");

    // Create DuckLake Manager
    let manager = DuckLakeManager::new(connection).await?;
    println!("✅ Created DuckLake Manager");

    // Create a simple schema for financial data (for reference)
    let _schema = Schema {
        fields: vec![
            Field {
                name: "id".to_string(),
                data_type: DataType::Int32,
                nullable: false,
                default_value: None,
                description: Some("Trade ID".to_string()),
            },
            Field {
                name: "symbol".to_string(),
                data_type: DataType::String,
                nullable: false,
                default_value: None,
                description: Some("Stock symbol".to_string()),
            },
            Field {
                name: "price".to_string(),
                data_type: DataType::Float64,
                nullable: false,
                default_value: None,
                description: Some("Trade price".to_string()),
            },
            Field {
                name: "quantity".to_string(),
                data_type: DataType::Int32,
                nullable: false,
                default_value: None,
                description: Some("Trade quantity".to_string()),
            },
            Field {
                name: "timestamp".to_string(),
                data_type: DataType::Timestamp,
                nullable: false,
                default_value: None,
                description: Some("Trade timestamp".to_string()),
            },
        ],
        primary_key: Some(vec!["id".to_string()]),
        indexes: vec![],
    };

    // Create a table with schema definition
    let table_schema = r#"(
        id INTEGER PRIMARY KEY,
        symbol VARCHAR NOT NULL,
        price DOUBLE NOT NULL,
        quantity INTEGER NOT NULL,
        timestamp TIMESTAMP NOT NULL
    )"#;

    manager.create_table("main", "trades", table_schema).await?;
    println!("✅ Created trades table");

    // Insert some sample data
    let insert_sql = r#"
        INSERT INTO trades (id, symbol, price, quantity, timestamp) VALUES
        (1, 'AAPL', 150.25, 100, '2024-01-01 10:00:00'),
        (2, 'GOOGL', 2800.50, 50, '2024-01-01 10:01:00'),
        (3, 'MSFT', 380.75, 200, '2024-01-01 10:02:00'),
        (4, 'AAPL', 151.00, 150, '2024-01-01 10:03:00'),
        (5, 'TSLA', 250.30, 75, '2024-01-01 10:04:00')
    "#;

    manager.execute_query("main", insert_sql).await?;
    println!("✅ Inserted sample trade data");

    // Query the data
    let query_sql = "SELECT * FROM main.trades LIMIT 3";
    let query_result = manager.execute_query("main", query_sql).await?;
    println!("📊 Query Results:");
    println!("Found {} rows", query_result.rows.len());

    for (i, row) in query_result.rows.iter().take(3).enumerate() {
        println!("  Row {}: {:?}", i + 1, row);
    }

    // Perform some analytics
    let analytics_sql = r#"
        SELECT 
            symbol,
            COUNT(*) as trade_count,
            AVG(price) as avg_price,
            SUM(quantity) as total_quantity,
            MIN(timestamp) as first_trade,
            MAX(timestamp) as last_trade
        FROM trades 
        GROUP BY symbol 
        ORDER BY trade_count DESC
    "#;

    let analytics_result = manager.execute_query("main", analytics_sql).await?;
    println!("\n📈 Trade Analytics:");
    for row in analytics_result.rows {
        println!("  {:?}", row);
    }

    // Get metrics
    let metrics = manager.get_metrics();
    println!("\n📊 DuckLake Metrics:");
    println!("  Snapshots created: {}", metrics.snapshots_created.get());
    println!("  Time travel queries: {}", metrics.time_travel_queries.get());
    println!("  Attached databases: {}", metrics.attached_databases_count.get());
    println!("  Query errors: {}", metrics.query_errors.get());

    // Get attached databases
    let databases = manager.get_attached_databases().await;
    println!("\n🗄️  Attached Databases:");
    for db in databases {
        println!("  - {}: metadata={}, data={}, read_only={}",
                 db.name, db.metadata_path, db.data_path, db.read_only);
    }

    println!("\n🎉 DuckLake Manager example completed successfully!");

    Ok(())
}
