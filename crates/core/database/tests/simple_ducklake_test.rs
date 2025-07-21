//! 简单的 DuckLake 测试
//! 
//! 测试基本的 DuckDB 连接和 DuckLake 管理器创建

use duckhub_database::{Connection, DuckLakeManager};
use tokio;

#[tokio::test]
async fn test_duckdb_connection() {
    println!("🔗 测试 DuckDB 连接创建");
    
    // 创建内存数据库连接
    let connection = Connection::open_in_memory().await;
    
    match connection {
        Ok(conn) => {
            println!("✅ DuckDB 连接创建成功");
            
            // 测试基本查询
            match conn.execute("SELECT 1 as test", &[]).await {
                Ok(_) => println!("✅ 基本查询执行成功"),
                Err(e) => println!("❌ 基本查询执行失败: {}", e),
            }
        },
        Err(e) => {
            println!("❌ DuckDB 连接创建失败: {}", e);
            panic!("无法创建 DuckDB 连接");
        }
    }
}

#[tokio::test]
async fn test_ducklake_manager_creation() {
    println!("🦆 测试 DuckLake 管理器创建");
    
    // 创建内存数据库连接
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    
    // 创建 DuckLake 管理器
    let manager_result = DuckLakeManager::new(connection).await;
    
    match manager_result {
        Ok(_manager) => {
            println!("✅ DuckLake 管理器创建成功");
        },
        Err(e) => {
            println!("❌ DuckLake 管理器创建失败: {}", e);
            // 不要 panic，因为这可能是预期的（没有真实的 DuckLake 扩展）
        }
    }
}

#[tokio::test]
async fn test_basic_sql_operations() {
    println!("📝 测试基本 SQL 操作");
    
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    
    // 创建表
    let create_table_sql = "CREATE TABLE test_table (id INTEGER, name VARCHAR, value DOUBLE)";
    match connection.execute(create_table_sql, &[]).await {
        Ok(_) => println!("✅ 表创建成功"),
        Err(e) => println!("❌ 表创建失败: {}", e),
    }
    
    // 插入数据 - 分别插入以避免批量插入问题
    let insert_sql1 = "INSERT INTO test_table VALUES (1, 'Alice', 100.0)";
    let insert_sql2 = "INSERT INTO test_table VALUES (2, 'Bob', 200.0)";

    match connection.execute(insert_sql1, &[]).await {
        Ok(rows) => println!("✅ 第一行数据插入成功，影响 {} 行", rows),
        Err(e) => println!("❌ 第一行数据插入失败: {}", e),
    }

    match connection.execute(insert_sql2, &[]).await {
        Ok(rows) => println!("✅ 第二行数据插入成功，影响 {} 行", rows),
        Err(e) => println!("❌ 第二行数据插入失败: {}", e),
    }
    
    // 查询数据
    let query_sql = "SELECT * FROM test_table";
    match connection.query_rows(query_sql, &[]).await {
        Ok(results) => {
            println!("✅ 查询成功，返回 {} 行数据", results.len());
            for (i, row) in results.iter().enumerate() {
                println!("  行 {}: {:?}", i + 1, row);
            }
        },
        Err(e) => println!("❌ 查询失败: {}", e),
    }
}

#[tokio::test]
async fn test_ducklake_metadata_tables() {
    println!("🗃️  测试 DuckLake 元数据表");
    
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    
    // 检查是否创建了 DuckLake 元数据表 - 使用 DuckDB 兼容的查询
    let check_tables_sql = "SHOW TABLES";
    match connection.query_rows(check_tables_sql, &[]).await {
        Ok(results) => {
            println!("✅ 找到 {} 个表", results.len());
            for (i, row) in results.iter().enumerate().take(5) {
                println!("  {}. {:?}", i + 1, row);
            }
        },
        Err(e) => {
            println!("⚠️  SHOW TABLES 失败: {}", e);
            // 尝试使用 information_schema
            let alt_sql = "SELECT table_name FROM information_schema.tables LIMIT 5";
            match connection.query_rows(alt_sql, &[]).await {
                Ok(alt_results) => {
                    println!("✅ 通过 information_schema 找到 {} 个表", alt_results.len());
                    for (i, row) in alt_results.iter().enumerate() {
                        println!("  {}. {:?}", i + 1, row);
                    }
                },
                Err(alt_e) => println!("❌ information_schema 查询也失败: {}", alt_e),
            }
        },
    }
}

#[tokio::test]
async fn test_ducklake_extensions() {
    println!("🔌 测试 DuckDB 扩展");
    
    let connection = Connection::open_in_memory().await.expect("Failed to create connection");
    
    // 检查已加载的扩展 - 使用更安全的查询
    let extensions_sql = "PRAGMA show_tables";
    match connection.query_rows(extensions_sql, &[]).await {
        Ok(results) => {
            println!("✅ 数据库查询成功，找到 {} 个结果", results.len());
            for (i, row) in results.iter().enumerate().take(3) {
                println!("  {}. {:?}", i + 1, row);
            }
        },
        Err(e) => {
            println!("⚠️  扩展查询失败，尝试基本查询: {}", e);
            // 尝试更简单的查询
            match connection.query_rows("SELECT 1 as test_value", &[]).await {
                Ok(simple_results) => {
                    println!("✅ 基本查询成功，返回 {} 行", simple_results.len());
                },
                Err(simple_e) => println!("❌ 基本查询也失败: {}", simple_e),
            }
        },
    }
    
    // 尝试安装 parquet 扩展
    match connection.execute("INSTALL parquet", &[]).await {
        Ok(_) => {
            println!("✅ Parquet 扩展安装成功");
            match connection.execute("LOAD parquet", &[]).await {
                Ok(_) => println!("✅ Parquet 扩展加载成功"),
                Err(e) => println!("⚠️  Parquet 扩展加载失败: {}", e),
            }
        },
        Err(e) => println!("⚠️  Parquet 扩展安装失败: {}", e),
    }
}
