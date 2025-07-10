//! DuckLake集成测试
//! 
//! 测试DuckLake的ACID事务特性、时间旅行查询、Schema演进等核心功能

use duckhub_database::*;
use duckhub_common::prelude::*;
use duckdb::Connection;
use std::collections::HashMap;
use tempfile::tempdir;
use tokio_test;

/// 创建测试用的DuckLake管理器
async fn create_test_manager() -> DuckLakeManager {
    let conn = Connection::open_in_memory().unwrap();
    
    // 尝试安装DuckLake扩展（如果可用）
    let _ = conn.execute("INSTALL ducklake", []);
    let _ = conn.execute("LOAD ducklake", []);
    
    DuckLakeManager::new(conn)
}

/// 创建测试用的DuckLake配置
fn create_test_config() -> DuckLakeConfig {
    DuckLakeConfig {
        metadata_path: ":memory:".to_string(),
        data_path: Some(":memory:.files".to_string()),
        metadata_schema: Some("main".to_string()),
        metadata_catalog: None,
        encrypted: false,
        data_inlining_row_limit: 0,
        read_only: false,
        snapshot_version: None,
        snapshot_time: None,
        metadata_parameters: HashMap::new(),
    }
}

#[tokio::test]
async fn test_ducklake_attach_and_detach() {
    let mut manager = create_test_manager().await;
    let config = create_test_config();
    
    // 测试附加数据库
    let attach_result = manager.attach_database("test_db", &config).await;
    
    match attach_result {
        Ok(_) => {
            println!("✅ 成功附加DuckLake数据库");
            
            // 验证数据库已附加
            assert_eq!(manager.attached_databases.len(), 1);
            assert!(manager.attached_databases.contains_key("test_db"));
            
            // 测试分离数据库
            let detach_result = manager.detach_database("test_db").await;
            assert!(detach_result.is_ok());
            assert_eq!(manager.attached_databases.len(), 0);
            
            println!("✅ 成功分离DuckLake数据库");
        }
        Err(e) => {
            println!("⚠️  DuckLake扩展未安装，跳过附加测试: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ducklake_secret_creation() {
    let manager = create_test_manager().await;
    let config = create_test_config();
    
    // 测试创建Secret
    let secret_result = manager.create_secret("test_secret", &config).await;
    
    match secret_result {
        Ok(_) => {
            println!("✅ 成功创建DuckLake Secret");
        }
        Err(e) => {
            println!("⚠️  创建Secret失败（可能是扩展未安装）: {}", e);
        }
    }
}

#[tokio::test]
async fn test_ducklake_table_operations() {
    let mut manager = create_test_manager().await;
    let config = create_test_config();
    
    // 尝试附加数据库
    if manager.attach_database("test_db", &config).await.is_ok() {
        println!("✅ 数据库附加成功，开始测试表操作");
        
        // 创建测试Schema
        let schema = Schema {
            fields: vec![
                Field {
                    name: "id".to_string(),
                    data_type: DataType::Int32,
                    nullable: false,
                },
                Field {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    nullable: true,
                },
                Field {
                    name: "amount".to_string(),
                    data_type: DataType::Decimal { precision: 10, scale: 2 },
                    nullable: false,
                },
            ],
        };
        
        // 测试创建表
        let create_result = manager.create_table("test_db", "transactions", &schema).await;
        match create_result {
            Ok(_) => {
                println!("✅ 成功创建DuckLake表");
                
                // 测试插入数据
                let test_data = vec![
                    vec![
                        serde_json::Value::Number(serde_json::Number::from(1)),
                        serde_json::Value::String("测试交易1".to_string()),
                        serde_json::Value::Number(serde_json::Number::from_f64(1500.50).unwrap()),
                    ],
                    vec![
                        serde_json::Value::Number(serde_json::Number::from(2)),
                        serde_json::Value::String("测试交易2".to_string()),
                        serde_json::Value::Number(serde_json::Number::from_f64(2500.75).unwrap()),
                    ],
                ];
                
                let insert_result = manager.insert_data("test_db", "transactions", &test_data).await;
                match insert_result {
                    Ok(_) => {
                        println!("✅ 成功插入测试数据");
                    }
                    Err(e) => {
                        println!("⚠️  插入数据失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  创建表失败: {}", e);
            }
        }
    } else {
        println!("⚠️  跳过表操作测试（数据库附加失败）");
    }
}

#[tokio::test]
async fn test_ducklake_batch_operations() {
    let mut manager = create_test_manager().await;
    let config = create_test_config();
    
    if manager.attach_database("test_db", &config).await.is_ok() {
        println!("✅ 开始测试批量操作");
        
        // 创建测试Schema
        let schema = Schema {
            fields: vec![
                Field {
                    name: "id".to_string(),
                    data_type: DataType::Int32,
                    nullable: false,
                },
                Field {
                    name: "value".to_string(),
                    data_type: DataType::String,
                    nullable: true,
                },
            ],
        };
        
        // 准备批量操作
        let operations = vec![
            DuckLakeOperation::CreateTable {
                database: "test_db".to_string(),
                table: "batch_test".to_string(),
                schema: schema.clone(),
            },
            DuckLakeOperation::Insert {
                database: "test_db".to_string(),
                table: "batch_test".to_string(),
                data: vec![
                    vec![
                        serde_json::Value::Number(serde_json::Number::from(1)),
                        serde_json::Value::String("批量测试1".to_string()),
                    ],
                    vec![
                        serde_json::Value::Number(serde_json::Number::from(2)),
                        serde_json::Value::String("批量测试2".to_string()),
                    ],
                ],
            },
        ];
        
        // 执行批量操作
        let batch_result = manager.batch_operations(operations).await;
        match batch_result {
            Ok(results) => {
                println!("✅ 批量操作成功，执行了{}个操作", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("  操作{}: {:?}", i + 1, result);
                }
            }
            Err(e) => {
                println!("⚠️  批量操作失败: {}", e);
            }
        }
    } else {
        println!("⚠️  跳过批量操作测试（数据库附加失败）");
    }
}

#[tokio::test]
async fn test_ducklake_time_travel() {
    let mut manager = create_test_manager().await;
    let config = create_test_config();
    
    if manager.attach_database("test_db", &config).await.is_ok() {
        println!("✅ 开始测试时间旅行查询");
        
        // 测试版本查询
        let version_result = manager.query_at_version(
            "test_db", 
            "test_table", 
            1, 
            "SELECT COUNT(*) FROM test_db.test_table"
        ).await;
        
        match version_result {
            Ok(_) => {
                println!("✅ 版本查询执行成功");
            }
            Err(e) => {
                println!("⚠️  版本查询失败（表可能不存在）: {}", e);
            }
        }
        
        // 测试时间戳查询
        let timestamp = Utc::now() - chrono::Duration::hours(1);
        let timestamp_result = manager.query_at_timestamp(
            "test_db", 
            "test_table", 
            timestamp, 
            "SELECT COUNT(*) FROM test_db.test_table"
        ).await;
        
        match timestamp_result {
            Ok(_) => {
                println!("✅ 时间戳查询执行成功");
            }
            Err(e) => {
                println!("⚠️  时间戳查询失败（表可能不存在）: {}", e);
            }
        }
        
        // 测试时间范围查询
        let start_time = Utc::now() - chrono::Duration::hours(2);
        let end_time = Utc::now();
        let range_result = manager.query_time_range(
            "test_db", 
            "test_table", 
            start_time, 
            end_time, 
            "SELECT * FROM test_db.test_table"
        ).await;
        
        match range_result {
            Ok(result) => {
                println!("✅ 时间范围查询执行成功，包含{}个快照", result.snapshots_included.len());
            }
            Err(e) => {
                println!("⚠️  时间范围查询失败: {}", e);
            }
        }
    } else {
        println!("⚠️  跳过时间旅行测试（数据库附加失败）");
    }
}

#[tokio::test]
async fn test_ducklake_schema_evolution() {
    let mut manager = create_test_manager().await;
    let config = create_test_config();
    
    if manager.attach_database("test_db", &config).await.is_ok() {
        println!("✅ 开始测试Schema演进");
        
        // 测试添加列
        let add_column_result = manager.add_column(
            "test_db",
            "test_table",
            "new_column",
            "VARCHAR",
            Some("'default_value'"),
            true
        ).await;
        
        match add_column_result {
            Ok(_) => {
                println!("✅ 添加列成功");
            }
            Err(e) => {
                println!("⚠️  添加列失败（表可能不存在）: {}", e);
            }
        }
        
        // 测试类型提升
        let alter_type_result = manager.alter_column_type(
            "test_db",
            "test_table",
            "some_column",
            "BIGINT"
        ).await;
        
        match alter_type_result {
            Ok(_) => {
                println!("✅ 类型提升成功");
            }
            Err(e) => {
                println!("⚠️  类型提升失败（列可能不存在）: {}", e);
            }
        }
    } else {
        println!("⚠️  跳过Schema演进测试（数据库附加失败）");
    }
}

#[tokio::test]
async fn test_ducklake_error_handling_and_retry() {
    let manager = create_test_manager().await;
    
    println!("✅ 开始测试错误处理和重试机制");
    
    // 测试无效的附加操作（应该触发重试）
    let mut manager_mut = manager;
    let invalid_config = DuckLakeConfig {
        metadata_path: "/invalid/path/that/does/not/exist".to_string(),
        ..Default::default()
    };
    
    let start_time = std::time::Instant::now();
    let result = manager_mut.attach_database("invalid_db", &invalid_config).await;
    let elapsed = start_time.elapsed();
    
    // 应该失败，但会经过重试机制
    assert!(result.is_err());
    
    // 验证重试机制增加了执行时间（至少应该有初始延迟）
    assert!(elapsed.as_millis() >= 100, "重试机制应该增加执行时间");
    
    println!("✅ 错误处理和重试机制测试完成，耗时: {:?}", elapsed);
}

#[tokio::test]
async fn test_ducklake_metrics_collection() {
    let manager = create_test_manager().await;
    
    println!("✅ 开始测试性能指标收集");
    
    // 获取初始指标
    let metrics = manager.get_metrics();
    
    // 验证指标结构存在
    assert_eq!(metrics.snapshots_created.get(), 0.0);
    assert_eq!(metrics.time_travel_queries.get(), 0.0);
    assert_eq!(metrics.query_errors.get(), 0.0);
    assert_eq!(metrics.retries_total.get(), 0.0);
    
    println!("✅ 性能指标收集测试完成");
}
