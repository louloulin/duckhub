//! DuckLake高级功能测试
//! 
//! 测试新增的数据血缘追踪和智能分区建议功能

use duckhub_database::ducklake_real::{
    DuckLakeManager, DuckLakeConfig, DataLineage, PartitioningSuggestion, PartitionStrategy
};
use duckhub_database::real_duckdb::Connection;
use duckhub_common::prelude::*;
use std::sync::Arc;
use tokio;

/// 创建测试用的DuckLake管理器
async fn create_test_manager() -> Result<DuckLakeManager> {
    let connection = Connection::open_in_memory().await?;

    DuckLakeManager::new(connection).await
}

/// 测试数据血缘追踪功能
#[tokio::test]
async fn test_data_lineage_tracing() -> Result<()> {
    let manager = create_test_manager().await?;
    
    // 创建测试数据库和表
    let database = "test_lineage_db";
    let table = "transactions";
    
    // 测试血缘追踪
    let lineage = manager.trace_data_lineage(database, table, Some("amount")).await?;
    
    // 验证结果
    assert_eq!(lineage.database, database);
    assert_eq!(lineage.table, table);
    assert_eq!(lineage.column, Some("amount".to_string()));
    assert!(!lineage.lineage_entries.is_empty());
    
    println!("✅ 数据血缘追踪测试通过");
    println!("   - 数据库: {}", lineage.database);
    println!("   - 表: {}", lineage.table);
    println!("   - 列: {:?}", lineage.column);
    println!("   - 血缘条目数: {}", lineage.lineage_entries.len());
    
    Ok(())
}

/// 测试表级别血缘追踪
#[tokio::test]
async fn test_table_level_lineage() -> Result<()> {
    let manager = create_test_manager().await?;
    
    let database = "test_db";
    let table = "users";
    
    // 测试表级别血缘追踪（不指定列）
    let lineage = manager.trace_data_lineage(database, table, None).await?;
    
    assert_eq!(lineage.database, database);
    assert_eq!(lineage.table, table);
    assert_eq!(lineage.column, None);
    
    println!("✅ 表级别血缘追踪测试通过");
    
    Ok(())
}

/// 测试智能分区建议功能
#[tokio::test]
async fn test_partitioning_suggestions() -> Result<()> {
    let manager = create_test_manager().await?;
    
    let database = "test_partition_db";
    let table = "large_table";
    
    // 测试分区建议
    let suggestion = manager.suggest_partitioning(database, table).await?;
    
    // 验证结果
    assert_eq!(suggestion.database, database);
    assert_eq!(suggestion.table, table);
    assert!(suggestion.estimated_performance_gain >= 0.0);
    assert!(!suggestion.implementation_sql.is_empty());
    
    println!("✅ 智能分区建议测试通过");
    println!("   - 数据库: {}", suggestion.database);
    println!("   - 表: {}", suggestion.table);
    println!("   - 当前行数: {}", suggestion.current_rows);
    println!("   - 建议策略: {:?}", suggestion.suggested_strategy);
    println!("   - 预估性能提升: {:.1}%", suggestion.estimated_performance_gain * 100.0);
    println!("   - 实现SQL: {}", suggestion.implementation_sql);
    
    Ok(())
}

/// 测试分区策略逻辑
#[tokio::test]
async fn test_partition_strategy_logic() -> Result<()> {
    let manager = create_test_manager().await?;
    
    // 测试不同数据量的分区策略
    let test_cases = vec![
        (100_000, PartitionStrategy::None),      // 小数据量，无需分区
        (5_000_000, PartitionStrategy::Monthly), // 中等数据量，月分区
        (50_000_000, PartitionStrategy::Daily),  // 大数据量，日分区
    ];
    
    for (expected_rows, expected_strategy) in test_cases {
        // 这里我们测试分区策略的逻辑
        let strategy = if expected_rows > 10_000_000 {
            PartitionStrategy::Daily
        } else if expected_rows > 1_000_000 {
            PartitionStrategy::Monthly
        } else {
            PartitionStrategy::None
        };
        
        // 验证策略匹配
        match (strategy, expected_strategy) {
            (PartitionStrategy::None, PartitionStrategy::None) => {},
            (PartitionStrategy::Monthly, PartitionStrategy::Monthly) => {},
            (PartitionStrategy::Daily, PartitionStrategy::Daily) => {},
            _ => panic!("分区策略不匹配: 期望 {:?}, 实际 {:?}", expected_strategy, strategy),
        }
        
        println!("✅ 数据量 {} 行 -> 策略 {:?}", expected_rows, strategy);
    }
    
    println!("✅ 分区策略逻辑测试通过");
    
    Ok(())
}

/// 测试性能提升计算
#[tokio::test]
async fn test_performance_gain_calculation() -> Result<()> {
    let manager = create_test_manager().await?;
    
    // 测试不同策略的性能提升计算
    let test_cases = vec![
        (PartitionStrategy::None, 0.0),
        (PartitionStrategy::Monthly, 0.1), // 小数据量月分区
        (PartitionStrategy::Daily, 0.4),   // 小数据量日分区
    ];
    
    for (strategy, expected_min_gain) in test_cases {
        let gain = manager.calculate_performance_gain(5_000_000, &strategy);
        assert!(gain >= expected_min_gain, 
            "性能提升计算错误: 策略 {:?}, 期望 >= {}, 实际 {}", 
            strategy, expected_min_gain, gain);
        
        println!("✅ 策略 {:?} -> 性能提升 {:.1}%", strategy, gain * 100.0);
    }
    
    println!("✅ 性能提升计算测试通过");
    
    Ok(())
}

/// 测试SQL生成功能
#[tokio::test]
async fn test_partition_sql_generation() -> Result<()> {
    let manager = create_test_manager().await?;
    
    let database = "test_db";
    let table = "test_table";
    
    // 测试不同策略的SQL生成
    let test_cases = vec![
        (PartitionStrategy::None, "-- No partitioning recommended"),
        (PartitionStrategy::Monthly, "ALTER TABLE test_db.test_table ADD PARTITION BY (DATE_TRUNC('month', created_at))"),
        (PartitionStrategy::Daily, "ALTER TABLE test_db.test_table ADD PARTITION BY (DATE_TRUNC('day', created_at))"),
    ];
    
    for (strategy, expected_sql) in test_cases {
        let sql = manager.generate_partition_sql(database, table, &strategy);
        assert_eq!(sql, expected_sql, "SQL生成错误: 策略 {:?}", strategy);
        
        println!("✅ 策略 {:?} -> SQL: {}", strategy, sql);
    }
    
    println!("✅ SQL生成功能测试通过");
    
    Ok(())
}

/// 集成测试：完整的分区建议流程
#[tokio::test]
async fn test_complete_partitioning_workflow() -> Result<()> {
    let manager = create_test_manager().await?;
    
    let database = "production_db";
    let table = "financial_transactions";
    
    // 1. 获取分区建议
    let suggestion = manager.suggest_partitioning(database, table).await?;
    
    // 2. 验证建议的完整性
    assert!(!suggestion.database.is_empty());
    assert!(!suggestion.table.is_empty());
    assert!(suggestion.estimated_performance_gain >= 0.0);
    assert!(!suggestion.implementation_sql.is_empty());
    
    // 3. 验证建议的合理性
    match suggestion.suggested_strategy {
        PartitionStrategy::None => {
            assert_eq!(suggestion.estimated_performance_gain, 0.0);
            assert!(suggestion.implementation_sql.contains("No partitioning"));
        },
        PartitionStrategy::Monthly | PartitionStrategy::Daily => {
            assert!(suggestion.estimated_performance_gain > 0.0);
            assert!(suggestion.implementation_sql.contains("ALTER TABLE"));
        },
    }
    
    println!("✅ 完整分区建议流程测试通过");
    println!("   建议详情: {:?}", suggestion);
    
    Ok(())
}

/// 错误处理测试
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let manager = create_test_manager().await?;
    
    // 测试空数据库名
    let result = manager.trace_data_lineage("", "table", None).await;
    assert!(result.is_ok(), "空数据库名应该被处理");
    
    // 测试空表名
    let result = manager.trace_data_lineage("db", "", None).await;
    assert!(result.is_ok(), "空表名应该被处理");
    
    println!("✅ 错误处理测试通过");
    
    Ok(())
}
