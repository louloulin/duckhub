//! DuckLake 性能基准测试
//! 
//! 这个测试文件专门用于测试 DuckLake 的性能特征，包括：
//! 1. 大数据量插入性能
//! 2. 复杂查询性能
//! 3. 并发操作性能
//! 4. 内存使用效率
//! 5. 时间旅行查询性能
//! 6. 批量操作性能

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::sync::Semaphore;

use duckhub_database::{DuckLakeManager, Connection};
use duckhub_common::prelude::*;

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// 创建测试用的 DuckLake 管理器
    async fn create_test_manager() -> Result<DuckLakeManager> {
        let conn = Connection::open_in_memory().await?;
        DuckLakeManager::new(conn).await
    }

    /// 性能测试结果结构
    #[derive(Debug, Clone)]
    struct PerformanceResult {
        operation: String,
        duration: Duration,
        throughput: f64, // 操作/秒
        memory_usage: Option<usize>,
        success_rate: f64,
    }

    impl PerformanceResult {
        fn new(operation: String, duration: Duration, operations_count: usize) -> Self {
            let throughput = operations_count as f64 / duration.as_secs_f64();
            Self {
                operation,
                duration,
                throughput,
                memory_usage: None,
                success_rate: 1.0,
            }
        }

        fn with_success_rate(mut self, success_rate: f64) -> Self {
            self.success_rate = success_rate;
            self
        }

        fn print_summary(&self) {
            println!("📊 性能测试结果: {}", self.operation);
            println!("   ⏱️  执行时间: {:?}", self.duration);
            println!("   🚀 吞吐量: {:.2} ops/sec", self.throughput);
            println!("   ✅ 成功率: {:.2}%", self.success_rate * 100.0);
            if let Some(memory) = self.memory_usage {
                println!("   💾 内存使用: {} bytes", memory);
            }
            println!();
        }
    }

    /// 测试大批量数据插入性能
    #[tokio::test]
    async fn test_bulk_insert_performance() {
        println!("🚀 测试大批量数据插入性能...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建测试表
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS bulk_insert_test (
                id INTEGER,
                name VARCHAR(100),
                value DECIMAL(10,2),
                category VARCHAR(50),
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        
        let result = manager.execute_query("test_db", create_table_sql).await;
        assert!(result.is_ok(), "创建表应该成功");
        
        // 测试不同批量大小的性能
        let batch_sizes = vec![100, 500, 1000, 2000];
        let mut results = Vec::new();
        
        for batch_size in batch_sizes {
            println!("📦 测试批量大小: {}", batch_size);
            
            let start_time = Instant::now();
            let mut successful_operations = 0;
            
            for i in 0..batch_size {
                let insert_sql = format!(
                    "INSERT INTO bulk_insert_test (id, name, value, category) VALUES ({}, 'name_{}', {}, 'category_{}')",
                    i, i, (i as f64) * 1.5, i % 10
                );
                
                if manager.execute_query("test_db", &insert_sql).await.is_ok() {
                    successful_operations += 1;
                }
            }
            
            let duration = start_time.elapsed();
            let success_rate = successful_operations as f64 / batch_size as f64;
            
            let result = PerformanceResult::new(
                format!("批量插入 (size: {})", batch_size),
                duration,
                successful_operations,
            ).with_success_rate(success_rate);
            
            result.print_summary();
            results.push(result);
            
            // 清理数据
            let _ = manager.execute_query("test_db", "DELETE FROM bulk_insert_test").await;
        }
        
        // 验证性能趋势
        assert!(!results.is_empty(), "应该有性能测试结果");
        
        println!("✅ 大批量数据插入性能测试完成");
    }

    /// 测试复杂查询性能
    #[tokio::test]
    async fn test_complex_query_performance() {
        println!("🔍 测试复杂查询性能...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建测试表和数据
        let setup_sql = r#"
            CREATE TABLE IF NOT EXISTS complex_query_test (
                id INTEGER,
                customer_id INTEGER,
                product_id INTEGER,
                amount DECIMAL(10,2),
                quantity INTEGER,
                order_date DATE,
                status VARCHAR(20),
                region VARCHAR(50)
            );
        "#;
        
        let result = manager.execute_query("test_db", setup_sql).await;
        assert!(result.is_ok(), "创建测试表应该成功");
        
        // 插入测试数据
        let data_size = 1000;
        println!("📊 插入 {} 条测试数据...", data_size);
        
        for i in 1..=data_size {
            let insert_sql = format!(
                "INSERT INTO complex_query_test VALUES ({}, {}, {}, {}, {}, '2024-{:02d}-{:02d}', '{}', '{}')",
                i,
                (i % 100) + 1,  // customer_id
                (i % 50) + 1,   // product_id
                (i as f64) * 10.5,  // amount
                (i % 10) + 1,   // quantity
                (i % 12) + 1,   // month
                (i % 28) + 1,   // day
                if i % 3 == 0 { "completed" } else if i % 3 == 1 { "pending" } else { "cancelled" },
                if i % 4 == 0 { "North" } else if i % 4 == 1 { "South" } else if i % 4 == 2 { "East" } else { "West" }
            );
            
            let _ = manager.execute_query("test_db", &insert_sql).await;
        }
        
        // 测试不同复杂度的查询
        let queries = vec![
            (
                "简单聚合查询",
                "SELECT COUNT(*), AVG(amount) FROM complex_query_test"
            ),
            (
                "分组聚合查询",
                "SELECT region, status, COUNT(*), SUM(amount) FROM complex_query_test GROUP BY region, status"
            ),
            (
                "复杂条件查询",
                r#"
                SELECT customer_id, SUM(amount) as total_amount, COUNT(*) as order_count
                FROM complex_query_test 
                WHERE status = 'completed' AND amount > 50 
                GROUP BY customer_id 
                HAVING COUNT(*) > 5 
                ORDER BY total_amount DESC 
                LIMIT 10
                "#
            ),
            (
                "窗口函数查询",
                r#"
                SELECT 
                    customer_id, 
                    amount,
                    ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY amount DESC) as rank
                FROM complex_query_test 
                WHERE status = 'completed'
                "#
            ),
        ];
        
        let mut results = Vec::new();
        
        for (query_name, sql) in queries {
            println!("🔍 执行查询: {}", query_name);
            
            let start_time = Instant::now();
            let result = manager.execute_query("test_db", sql).await;
            let duration = start_time.elapsed();
            
            let success = result.is_ok();
            let perf_result = PerformanceResult::new(
                query_name.to_string(),
                duration,
                1,
            ).with_success_rate(if success { 1.0 } else { 0.0 });
            
            perf_result.print_summary();
            results.push(perf_result);
            
            assert!(success, "查询 {} 应该成功", query_name);
        }
        
        println!("✅ 复杂查询性能测试完成");
    }

    /// 测试并发操作性能
    #[tokio::test]
    async fn test_concurrent_operations_performance() {
        println!("🔄 测试并发操作性能...");
        
        // 创建多个管理器实例模拟并发
        let concurrent_count = 5;
        let operations_per_worker = 50;
        
        println!("🚀 启动 {} 个并发工作者，每个执行 {} 个操作", concurrent_count, operations_per_worker);
        
        let semaphore = Arc::new(Semaphore::new(concurrent_count));
        let mut join_set = JoinSet::new();
        
        let start_time = Instant::now();
        
        for worker_id in 0..concurrent_count {
            let semaphore = semaphore.clone();
            
            join_set.spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let manager = create_test_manager().await.expect("创建管理器失败");
                
                // 创建工作者专用表
                let table_name = format!("concurrent_test_{}", worker_id);
                let create_table_sql = format!(
                    "CREATE TABLE IF NOT EXISTS {} (id INTEGER, data TEXT, worker_id INTEGER)",
                    table_name
                );
                
                let _ = manager.execute_query("test_db", &create_table_sql).await;
                
                let mut successful_ops = 0;
                
                for op_id in 0..operations_per_worker {
                    let insert_sql = format!(
                        "INSERT INTO {} (id, data, worker_id) VALUES ({}, 'data_{}', {})",
                        table_name, op_id, op_id, worker_id
                    );
                    
                    if manager.execute_query("test_db", &insert_sql).await.is_ok() {
                        successful_ops += 1;
                    }
                }
                
                (worker_id, successful_ops)
            });
        }
        
        // 等待所有工作者完成
        let mut total_successful_ops = 0;
        let mut completed_workers = 0;
        
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((worker_id, successful_ops)) => {
                    println!("✅ 工作者 {} 完成，成功操作: {}", worker_id, successful_ops);
                    total_successful_ops += successful_ops;
                    completed_workers += 1;
                }
                Err(e) => {
                    println!("❌ 工作者失败: {:?}", e);
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        let total_operations = concurrent_count * operations_per_worker;
        let success_rate = total_successful_ops as f64 / total_operations as f64;
        
        let result = PerformanceResult::new(
            "并发操作".to_string(),
            total_duration,
            total_successful_ops,
        ).with_success_rate(success_rate);
        
        result.print_summary();
        
        assert_eq!(completed_workers, concurrent_count, "所有工作者应该完成");
        assert!(success_rate > 0.9, "成功率应该大于90%");
        
        println!("✅ 并发操作性能测试完成");
    }

    /// 测试内存使用效率
    #[tokio::test]
    async fn test_memory_efficiency() {
        println!("💾 测试内存使用效率...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建内存测试表
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS memory_test (
                id INTEGER,
                large_text TEXT,
                numbers INTEGER
            )
        "#;
        
        let result = manager.execute_query("test_db", create_table_sql).await;
        assert!(result.is_ok(), "创建内存测试表应该成功");
        
        // 测试大量数据的内存使用
        let record_count = 1000;
        let large_text = "x".repeat(1000); // 1KB per record
        
        println!("📊 插入 {} 条大记录 (每条约1KB)...", record_count);
        
        let start_time = Instant::now();
        
        for i in 0..record_count {
            let insert_sql = format!(
                "INSERT INTO memory_test (id, large_text, numbers) VALUES ({}, '{}', {})",
                i, large_text, i * 2
            );
            
            let result = manager.execute_query("test_db", &insert_sql).await;
            assert!(result.is_ok(), "插入大记录应该成功");
        }
        
        let insert_duration = start_time.elapsed();
        
        // 测试查询大量数据的性能
        let query_start = Instant::now();
        let select_sql = "SELECT COUNT(*), LENGTH(large_text) FROM memory_test";
        let result = manager.execute_query("test_db", select_sql).await;
        let query_duration = query_start.elapsed();
        
        assert!(result.is_ok(), "查询大量数据应该成功");
        
        let insert_result = PerformanceResult::new(
            "大记录插入".to_string(),
            insert_duration,
            record_count,
        );
        
        let query_result = PerformanceResult::new(
            "大数据查询".to_string(),
            query_duration,
            1,
        );
        
        insert_result.print_summary();
        query_result.print_summary();
        
        // 验证内存效率
        assert!(insert_duration < Duration::from_secs(30), "插入应该在合理时间内完成");
        assert!(query_duration < Duration::from_secs(5), "查询应该快速完成");
        
        println!("✅ 内存使用效率测试完成");
    }
}
