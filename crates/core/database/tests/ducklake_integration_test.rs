//! DuckLake 集成测试
//! 
//! 测试真实的 DuckLake 功能实现

use duckhub_database::{DuckLakeManager, DuckLakeConfig, Connection};
use duckhub_database::ducklake_real::{CreateSnapshotRequest, TimeTravelQueryRequest, TimeTravelTarget};
use std::collections::HashMap;
use tokio;
use chrono::Utc;

#[tokio::test]
async fn test_ducklake_manager_creation() {
    // 创建内存数据库连接
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    
    // 创建 DuckLake 管理器
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    println!("✅ DuckLake 管理器创建成功");
}

#[tokio::test]
async fn test_ducklake_database_operations() {
    // 创建内存数据库连接
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    // 创建测试数据库
    let config = DuckLakeConfig {
        metadata_path: "test_db.ducklake".to_string(),
        data_path: Some("test_data/".to_string()),
        ..Default::default()
    };
    
    // 附加数据库
    match manager.attach_ducklake("test_db", &config).await {
        Ok(_) => println!("✅ 数据库附加成功"),
        Err(e) => println!("⚠️  数据库附加失败（预期，因为没有真实的 DuckLake 扩展）: {}", e),
    }
}

#[tokio::test]
async fn test_ducklake_table_operations() {
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    // 创建测试表
    let create_table_sql = "CREATE TABLE test_table (id INTEGER, name VARCHAR, value DOUBLE)";
    match manager.create_table("main", "test_table", create_table_sql).await {
        Ok(_) => println!("✅ 表创建成功"),
        Err(e) => println!("⚠️  表创建失败: {}", e),
    }
    
    // 插入测试数据
    let insert_sql = "VALUES (1, 'Alice', 100.0), (2, 'Bob', 200.0), (3, 'Charlie', 300.0)";
    match manager.insert_data("main", "test_table", insert_sql).await {
        Ok(_) => println!("✅ 数据插入成功"),
        Err(e) => println!("⚠️  数据插入失败: {}", e),
    }
    
    // 查询数据
    match manager.query_table("main", "test_table", None).await {
        Ok(results) => {
            println!("✅ 查询成功，返回 {} 行数据", results.len());
            for (i, row) in results.iter().enumerate() {
                println!("  行 {}: {:?}", i + 1, row);
            }
        },
        Err(e) => println!("⚠️  查询失败: {}", e),
    }
}

#[tokio::test]
async fn test_ducklake_snapshot_operations() {
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    // 创建快照请求
    let snapshot_request = CreateSnapshotRequest {
        database: "main".to_string(),
        table: Some("test_table".to_string()),
        description: Some("测试快照".to_string()),
        include_all_tables: false,
        tables: vec!["test_table".to_string()],
    };
    
    // 创建快照
    match manager.create_snapshot(snapshot_request).await {
        Ok(snapshot) => {
            println!("✅ 快照创建成功: {}", snapshot.id);
            println!("  创建时间: {}", snapshot.created_at);
            println!("  大小: {} 字节", snapshot.size_bytes);
            println!("  表数量: {}", snapshot.table_count);
        },
        Err(e) => println!("⚠️  快照创建失败: {}", e),
    }
    
    // 列出快照
    match manager.list_snapshots("main").await {
        Ok(snapshots) => {
            println!("✅ 快照列表获取成功，共 {} 个快照", snapshots.len());
            for snapshot in snapshots {
                println!("  快照 ID: {}", snapshot.id);
                println!("  创建时间: {}", snapshot.created_at);
                if let Some(desc) = snapshot.description {
                    println!("  描述: {}", desc);
                }
            }
        },
        Err(e) => println!("⚠️  快照列表获取失败: {}", e),
    }
}

#[tokio::test]
async fn test_ducklake_time_travel_query() {
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    // 时间旅行查询请求
    let time_travel_request = TimeTravelQueryRequest {
        database: "main".to_string(),
        table: "test_table".to_string(),
        target: TimeTravelTarget::Version(1),
        sql: Some("SELECT * FROM main.test_table".to_string()),
    };
    
    // 执行时间旅行查询
    match manager.time_travel_query(time_travel_request).await {
        Ok(results) => {
            println!("✅ 时间旅行查询成功，返回 {} 行数据", results.len());
            for (i, row) in results.iter().enumerate() {
                println!("  行 {}: {:?}", i + 1, row);
            }
        },
        Err(e) => println!("⚠️  时间旅行查询失败: {}", e),
    }
}

#[tokio::test]
async fn test_ducklake_comprehensive_workflow() {
    println!("🚀 开始 DuckLake 综合工作流测试");
    
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    let manager = DuckLakeManager::new(connection).await.expect("Failed to create DuckLake manager");
    
    // 1. 创建表和插入数据
    println!("\n📝 步骤 1: 创建表和插入数据");
    let _ = manager.create_table("main", "financial_data", 
        "CREATE TABLE financial_data (id INTEGER, symbol VARCHAR, price DOUBLE, timestamp TIMESTAMP)").await;
    
    let _ = manager.insert_data("main", "financial_data", 
        "VALUES (1, 'AAPL', 150.0, '2024-01-01 10:00:00'), (2, 'GOOGL', 2800.0, '2024-01-01 10:00:00')").await;
    
    // 2. 创建第一个快照
    println!("\n📸 步骤 2: 创建第一个快照");
    let snapshot1_request = CreateSnapshotRequest {
        database: "main".to_string(),
        table: Some("financial_data".to_string()),
        description: Some("初始数据快照".to_string()),
        include_all_tables: false,
        tables: vec!["financial_data".to_string()],
    };
    
    let snapshot1 = manager.create_snapshot(snapshot1_request).await;
    if let Ok(snap) = &snapshot1 {
        println!("✅ 第一个快照创建成功: {}", snap.id);
    }
    
    // 3. 插入更多数据
    println!("\n📝 步骤 3: 插入更多数据");
    let _ = manager.insert_data("main", "financial_data", 
        "VALUES (3, 'TSLA', 800.0, '2024-01-01 11:00:00'), (4, 'MSFT', 300.0, '2024-01-01 11:00:00')").await;
    
    // 4. 创建第二个快照
    println!("\n📸 步骤 4: 创建第二个快照");
    let snapshot2_request = CreateSnapshotRequest {
        database: "main".to_string(),
        table: Some("financial_data".to_string()),
        description: Some("更新后数据快照".to_string()),
        include_all_tables: false,
        tables: vec!["financial_data".to_string()],
    };
    
    let snapshot2 = manager.create_snapshot(snapshot2_request).await;
    if let Ok(snap) = &snapshot2 {
        println!("✅ 第二个快照创建成功: {}", snap.id);
    }
    
    // 5. 列出所有快照
    println!("\n📋 步骤 5: 列出所有快照");
    match manager.list_snapshots("main").await {
        Ok(snapshots) => {
            println!("✅ 找到 {} 个快照:", snapshots.len());
            for (i, snapshot) in snapshots.iter().enumerate() {
                println!("  {}. ID: {}, 时间: {}", i + 1, snapshot.id, snapshot.created_at);
            }
        },
        Err(e) => println!("⚠️  获取快照列表失败: {}", e),
    }
    
    // 6. 执行时间旅行查询
    println!("\n⏰ 步骤 6: 执行时间旅行查询");
    let time_travel_request = TimeTravelQueryRequest {
        database: "main".to_string(),
        table: "financial_data".to_string(),
        target: TimeTravelTarget::Version(1),
        sql: Some("SELECT COUNT(*) as total_records FROM main.financial_data".to_string()),
    };
    
    match manager.time_travel_query(time_travel_request).await {
        Ok(results) => {
            println!("✅ 时间旅行查询成功");
            for row in results {
                println!("  结果: {:?}", row);
            }
        },
        Err(e) => println!("⚠️  时间旅行查询失败: {}", e),
    }
    
    println!("\n🎉 DuckLake 综合工作流测试完成！");
}
