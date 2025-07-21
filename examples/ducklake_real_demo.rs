//! DuckLake 真实功能演示
//! 
//! 展示 DuckHub 中真实的 DuckLake 功能实现

use duckhub_database::{DuckDBEngine, DuckLakeManager, Connection};
use duckhub_database::ducklake_real::{CreateSnapshotRequest, TimeTravelQueryRequest, TimeTravelTarget};
use duckhub_common::config::DatabaseConfig;
use std::sync::Arc;
use tokio;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🚀 DuckHub DuckLake 真实功能演示");
    println!("=====================================");
    
    // 1. 创建 DuckDB 引擎
    println!("\n📊 步骤 1: 创建 DuckDB 引擎");
    let config = DatabaseConfig {
        duckdb_path: ":memory:".to_string(),
        memory_limit: Some("1GB".to_string()),
        threads: Some(4),
        max_memory: Some("1GB".to_string()),
        temp_directory: None,
        extensions: vec!["parquet".to_string(), "json".to_string()],
        pool: duckhub_common::config::PoolConfig {
            min_connections: 1,
            max_connections: 10,
            connection_timeout: 30,
            idle_timeout: 300,
            max_lifetime: 3600,
        },
    };
    
    let engine = Arc::new(DuckDBEngine::new(config).await?);
    println!("✅ DuckDB 引擎创建成功");
    
    // 2. 检查 DuckLake 管理器
    println!("\n🦆 步骤 2: 检查 DuckLake 管理器");
    match engine.ducklake_manager() {
        Some(manager) => {
            println!("✅ DuckLake 管理器可用");
            
            // 3. 创建测试数据
            println!("\n📝 步骤 3: 创建测试数据");
            demo_create_test_data(&engine).await?;
            
            // 4. 演示快照功能
            println!("\n📸 步骤 4: 演示快照功能");
            demo_snapshot_operations(&engine).await?;
            
            // 5. 演示时间旅行查询
            println!("\n⏰ 步骤 5: 演示时间旅行查询");
            demo_time_travel_queries(&engine).await?;
            
        },
        None => {
            println!("⚠️  DuckLake 管理器不可用，但基本 DuckDB 功能正常");
            
            // 演示基本 DuckDB 功能
            demo_basic_duckdb_operations(&engine).await?;
        }
    }
    
    println!("\n🎉 演示完成！");
    println!("=====================================");
    
    Ok(())
}

async fn demo_create_test_data(engine: &Arc<DuckDBEngine>) -> Result<(), Box<dyn std::error::Error>> {
    // 创建金融数据表
    let create_table_sql = "
        CREATE TABLE financial_data (
            id INTEGER,
            symbol VARCHAR,
            price DOUBLE,
            volume BIGINT,
            timestamp TIMESTAMP,
            market VARCHAR
        )
    ";
    
    match engine.execute(create_table_sql).await {
        Ok(_) => println!("✅ 金融数据表创建成功"),
        Err(e) => println!("⚠️  表创建失败: {}", e),
    }
    
    // 插入测试数据
    let insert_sql = "
        INSERT INTO financial_data VALUES
        (1, 'AAPL', 150.25, 1000000, '2024-01-01 09:30:00', 'NASDAQ'),
        (2, 'GOOGL', 2800.50, 500000, '2024-01-01 09:30:00', 'NASDAQ'),
        (3, 'TSLA', 800.75, 750000, '2024-01-01 09:30:00', 'NASDAQ'),
        (4, 'MSFT', 300.00, 900000, '2024-01-01 09:30:00', 'NASDAQ'),
        (5, 'AMZN', 3200.25, 400000, '2024-01-01 09:30:00', 'NASDAQ')
    ";
    
    match engine.execute(insert_sql).await {
        Ok(rows) => println!("✅ 插入 {} 行测试数据", rows),
        Err(e) => println!("⚠️  数据插入失败: {}", e),
    }
    
    // 查询数据验证
    let query_sql = "SELECT COUNT(*) as total, AVG(price) as avg_price FROM financial_data";
    match engine.query(query_sql).await {
        Ok(results) => {
            if let Some(row) = results.first() {
                let total = row.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                let avg_price = row.get("avg_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                println!("✅ 数据验证: 总计 {} 条记录，平均价格 ${:.2}", total, avg_price);
            }
        },
        Err(e) => println!("⚠️  数据查询失败: {}", e),
    }
    
    Ok(())
}

async fn demo_snapshot_operations(engine: &Arc<DuckDBEngine>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_manager) = engine.ducklake_manager() {
        // 创建快照
        let snapshot_request = CreateSnapshotRequest {
            database: "main".to_string(),
            table: Some("financial_data".to_string()),
            description: Some("金融数据初始快照".to_string()),
            include_all_tables: false,
            tables: vec!["financial_data".to_string()],
        };
        
        match engine.create_ducklake_snapshot(snapshot_request).await {
            Ok(snapshot) => {
                println!("✅ 快照创建成功:");
                println!("   ID: {}", snapshot.id);
                println!("   时间: {}", snapshot.created_at);
                println!("   大小: {} 字节", snapshot.size_bytes);
                println!("   表数量: {}", snapshot.table_count);
                
                // 列出快照
                match engine.list_ducklake_snapshots("main").await {
                    Ok(snapshots) => {
                        println!("✅ 快照列表 ({} 个):", snapshots.len());
                        for (i, snap) in snapshots.iter().enumerate() {
                            println!("   {}. {} ({})", i + 1, snap.id, snap.created_at);
                        }
                    },
                    Err(e) => println!("⚠️  快照列表获取失败: {}", e),
                }
            },
            Err(e) => println!("⚠️  快照创建失败: {}", e),
        }
    }
    
    Ok(())
}

async fn demo_time_travel_queries(engine: &Arc<DuckDBEngine>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(_manager) = engine.ducklake_manager() {
        // 时间旅行查询
        let time_travel_request = TimeTravelQueryRequest {
            database: "main".to_string(),
            table: "financial_data".to_string(),
            target: TimeTravelTarget::Version(1),
            sql: Some("SELECT symbol, price FROM main.financial_data WHERE price > 1000".to_string()),
        };
        
        match engine.execute_ducklake_time_travel(time_travel_request).await {
            Ok(results) => {
                println!("✅ 时间旅行查询成功:");
                println!("   返回 {} 行数据", results.len());
                for (i, row) in results.iter().enumerate().take(3) {
                    println!("   {}. {:?}", i + 1, row);
                }
                if results.len() > 3 {
                    println!("   ... 还有 {} 行", results.len() - 3);
                }
            },
            Err(e) => println!("⚠️  时间旅行查询失败: {}", e),
        }
    }
    
    Ok(())
}

async fn demo_basic_duckdb_operations(engine: &Arc<DuckDBEngine>) -> Result<(), Box<dyn std::error::Error>> {
    println!("演示基本 DuckDB 功能:");
    
    // 创建表
    let create_sql = "CREATE TABLE demo_table (id INTEGER, name VARCHAR, value DOUBLE)";
    match engine.execute(create_sql).await {
        Ok(_) => println!("✅ 演示表创建成功"),
        Err(e) => println!("⚠️  表创建失败: {}", e),
    }
    
    // 插入数据
    let insert_sql = "INSERT INTO demo_table VALUES (1, 'Test', 123.45), (2, 'Demo', 678.90)";
    match engine.execute(insert_sql).await {
        Ok(rows) => println!("✅ 插入 {} 行数据", rows),
        Err(e) => println!("⚠️  数据插入失败: {}", e),
    }
    
    // 查询数据
    let query_sql = "SELECT * FROM demo_table";
    match engine.query(query_sql).await {
        Ok(results) => {
            println!("✅ 查询成功，返回 {} 行:", results.len());
            for (i, row) in results.iter().enumerate() {
                println!("   {}. {:?}", i + 1, row);
            }
        },
        Err(e) => println!("⚠️  查询失败: {}", e),
    }
    
    Ok(())
}
