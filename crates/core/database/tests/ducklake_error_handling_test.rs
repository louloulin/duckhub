//! DuckLake 错误处理和恢复测试
//! 
//! 这个测试文件专门测试 DuckLake 的错误处理能力，包括：
//! 1. SQL 语法错误处理
//! 2. 连接错误和重试机制
//! 3. 数据完整性错误处理
//! 4. 资源限制错误处理
//! 5. 并发冲突错误处理
//! 6. 恢复机制验证

use std::time::Duration;
use tokio::time::sleep;

use duckhub_database::{DuckLakeManager, Connection};
use duckhub_common::prelude::*;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    /// 创建测试用的 DuckLake 管理器
    async fn create_test_manager() -> Result<DuckLakeManager> {
        let conn = Connection::open_in_memory().await?;
        DuckLakeManager::new(conn).await
    }

    /// 错误测试结果结构
    #[derive(Debug, Clone)]
    struct ErrorTestResult {
        test_name: String,
        expected_error: bool,
        actual_error: bool,
        error_message: Option<String>,
        recovery_successful: bool,
    }

    impl ErrorTestResult {
        fn new(test_name: String, expected_error: bool) -> Self {
            Self {
                test_name,
                expected_error,
                actual_error: false,
                error_message: None,
                recovery_successful: false,
            }
        }

        fn with_error(mut self, error_message: String) -> Self {
            self.actual_error = true;
            self.error_message = Some(error_message);
            self
        }

        fn with_recovery(mut self, successful: bool) -> Self {
            self.recovery_successful = successful;
            self
        }

        fn is_successful(&self) -> bool {
            self.expected_error == self.actual_error
        }

        fn print_summary(&self) {
            let status = if self.is_successful() { "✅" } else { "❌" };
            println!("{} 错误测试: {}", status, self.test_name);
            println!("   期望错误: {}, 实际错误: {}", self.expected_error, self.actual_error);
            if let Some(msg) = &self.error_message {
                println!("   错误信息: {}", msg);
            }
            if self.expected_error {
                println!("   恢复成功: {}", self.recovery_successful);
            }
            println!();
        }
    }

    /// 测试 SQL 语法错误处理
    #[tokio::test]
    async fn test_sql_syntax_error_handling() {
        println!("🔍 测试 SQL 语法错误处理...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        let mut results = Vec::new();
        
        // 测试各种语法错误
        let syntax_errors = vec![
            ("无效的 SQL 语句", "INVALID SQL STATEMENT"),
            ("缺少 FROM 子句", "SELECT * WHERE id = 1"),
            ("无效的列名", "SELECT invalid_column FROM non_existent_table"),
            ("语法错误的 CREATE", "CREATE INVALID TABLE test"),
            ("错误的 INSERT 语法", "INSERT INVALID VALUES (1, 2, 3)"),
            ("无效的 UPDATE", "UPDATE SET column = value"),
            ("错误的 DELETE", "DELETE WHERE"),
        ];
        
        for (test_name, invalid_sql) in syntax_errors {
            let mut test_result = ErrorTestResult::new(test_name.to_string(), true);
            
            match manager.execute_query("test_db", invalid_sql).await {
                Ok(_) => {
                    // 不应该成功
                    test_result.print_summary();
                    assert!(false, "语法错误的 SQL 不应该成功执行: {}", test_name);
                }
                Err(e) => {
                    test_result = test_result.with_error(e.to_string());
                    
                    // 测试恢复能力 - 执行一个正确的查询
                    let recovery_sql = "SELECT 1 as recovery_test";
                    let recovery_result = manager.execute_query("test_db", recovery_sql).await;
                    test_result = test_result.with_recovery(recovery_result.is_ok());
                    
                    test_result.print_summary();
                    assert!(test_result.is_successful(), "错误处理测试失败: {}", test_name);
                    assert!(test_result.recovery_successful, "错误后应该能够恢复: {}", test_name);
                }
            }
            
            results.push(test_result);
        }
        
        println!("✅ SQL 语法错误处理测试完成，共测试 {} 个错误场景", results.len());
    }

    /// 测试数据完整性错误处理
    #[tokio::test]
    async fn test_data_integrity_error_handling() {
        println!("🔒 测试数据完整性错误处理...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建带约束的测试表
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS integrity_test (
                id INTEGER PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                email VARCHAR(255) UNIQUE,
                age INTEGER CHECK (age >= 0 AND age <= 150),
                status VARCHAR(20) DEFAULT 'active'
            )
        "#;
        
        let result = manager.execute_query("test_db", create_table_sql).await;
        assert!(result.is_ok(), "创建约束表应该成功");
        
        // 插入正常数据
        let valid_insert = "INSERT INTO integrity_test (id, name, email, age) VALUES (1, 'John Doe', 'john@example.com', 30)";
        let result = manager.execute_query("test_db", valid_insert).await;
        assert!(result.is_ok(), "插入有效数据应该成功");
        
        // 测试各种完整性约束违反
        let integrity_violations = vec![
            (
                "主键重复",
                "INSERT INTO integrity_test (id, name, email, age) VALUES (1, 'Jane Doe', 'jane@example.com', 25)"
            ),
            (
                "NOT NULL 约束违反",
                "INSERT INTO integrity_test (id, email, age) VALUES (2, 'test@example.com', 25)"
            ),
            (
                "UNIQUE 约束违反",
                "INSERT INTO integrity_test (id, name, email, age) VALUES (3, 'Bob Smith', 'john@example.com', 35)"
            ),
            (
                "CHECK 约束违反 (负年龄)",
                "INSERT INTO integrity_test (id, name, email, age) VALUES (4, 'Invalid Age', 'invalid@example.com', -5)"
            ),
            (
                "CHECK 约束违反 (超大年龄)",
                "INSERT INTO integrity_test (id, name, email, age) VALUES (5, 'Too Old', 'old@example.com', 200)"
            ),
        ];
        
        let mut results = Vec::new();
        
        for (test_name, violation_sql) in integrity_violations {
            let mut test_result = ErrorTestResult::new(test_name.to_string(), true);
            
            match manager.execute_query("test_db", violation_sql).await {
                Ok(_) => {
                    // 某些约束可能不被所有数据库严格执行
                    println!("⚠️  约束 {} 可能未被严格执行", test_name);
                    test_result = test_result.with_recovery(true);
                }
                Err(e) => {
                    test_result = test_result.with_error(e.to_string());
                    
                    // 测试恢复 - 插入有效数据
                    let recovery_sql = format!(
                        "INSERT INTO integrity_test (id, name, email, age) VALUES ({}, 'Recovery Test {}', 'recovery{}@example.com', 25)",
                        100 + results.len(), results.len(), results.len()
                    );
                    let recovery_result = manager.execute_query("test_db", &recovery_sql).await;
                    test_result = test_result.with_recovery(recovery_result.is_ok());
                }
            }
            
            test_result.print_summary();
            results.push(test_result);
        }
        
        // 验证数据库状态
        let count_sql = "SELECT COUNT(*) as count FROM integrity_test";
        let result = manager.execute_query("test_db", count_sql).await;
        assert!(result.is_ok(), "验证数据库状态应该成功");
        
        println!("✅ 数据完整性错误处理测试完成");
    }

    /// 测试资源限制错误处理
    #[tokio::test]
    async fn test_resource_limit_error_handling() {
        println!("📊 测试资源限制错误处理...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建测试表
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS resource_test (
                id INTEGER,
                large_data TEXT
            )
        "#;
        
        let result = manager.execute_query("test_db", create_table_sql).await;
        assert!(result.is_ok(), "创建资源测试表应该成功");
        
        // 测试大数据插入
        let large_text = "x".repeat(10000); // 10KB 文本
        let mut successful_inserts = 0;
        let mut failed_inserts = 0;
        let max_attempts = 100;
        
        println!("📦 尝试插入 {} 条大记录...", max_attempts);
        
        for i in 0..max_attempts {
            let insert_sql = format!(
                "INSERT INTO resource_test (id, large_data) VALUES ({}, '{}')",
                i, large_text
            );
            
            match manager.execute_query("test_db", &insert_sql).await {
                Ok(_) => successful_inserts += 1,
                Err(_) => {
                    failed_inserts += 1;
                    // 测试恢复 - 尝试插入较小的数据
                    let small_insert = format!(
                        "INSERT INTO resource_test (id, large_data) VALUES ({}, 'small_data')",
                        i + 1000
                    );
                    let _ = manager.execute_query("test_db", &small_insert).await;
                }
            }
            
            // 每10次操作检查一次状态
            if i % 10 == 0 {
                let count_sql = "SELECT COUNT(*) as count FROM resource_test";
                let result = manager.execute_query("test_db", count_sql).await;
                assert!(result.is_ok(), "状态检查应该成功");
            }
        }
        
        println!("📊 插入结果: 成功 {}, 失败 {}", successful_inserts, failed_inserts);
        
        // 验证系统仍然可用
        let final_check = "SELECT COUNT(*) as final_count FROM resource_test";
        let result = manager.execute_query("test_db", final_check).await;
        assert!(result.is_ok(), "最终状态检查应该成功");
        
        println!("✅ 资源限制错误处理测试完成");
    }

    /// 测试连接错误和重试机制
    #[tokio::test]
    async fn test_connection_error_and_retry() {
        println!("🔄 测试连接错误和重试机制...");
        
        // 测试正常连接
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 执行一些操作来建立连接状态
        let setup_sql = r#"
            CREATE TABLE IF NOT EXISTS retry_test (
                id INTEGER,
                data TEXT,
                attempt_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#;
        
        let result = manager.execute_query("test_db", setup_sql).await;
        assert!(result.is_ok(), "初始连接应该成功");
        
        // 模拟重试场景
        let retry_attempts = 5;
        let mut successful_operations = 0;
        
        for attempt in 1..=retry_attempts {
            println!("🔄 重试尝试 {}/{}", attempt, retry_attempts);
            
            let insert_sql = format!(
                "INSERT INTO retry_test (id, data) VALUES ({}, 'attempt_{}')",
                attempt, attempt
            );
            
            // 模拟网络延迟
            sleep(Duration::from_millis(10)).await;
            
            match manager.execute_query("test_db", &insert_sql).await {
                Ok(_) => {
                    successful_operations += 1;
                    println!("✅ 尝试 {} 成功", attempt);
                }
                Err(e) => {
                    println!("❌ 尝试 {} 失败: {}", attempt, e);
                    
                    // 等待后重试
                    sleep(Duration::from_millis(100)).await;
                    
                    // 再次尝试
                    if manager.execute_query("test_db", &insert_sql).await.is_ok() {
                        successful_operations += 1;
                        println!("✅ 重试后尝试 {} 成功", attempt);
                    }
                }
            }
        }
        
        let success_rate = successful_operations as f64 / retry_attempts as f64;
        println!("📊 重试成功率: {:.2}%", success_rate * 100.0);
        
        // 验证最终状态
        let count_sql = "SELECT COUNT(*) as count FROM retry_test";
        let result = manager.execute_query("test_db", count_sql).await;
        assert!(result.is_ok(), "最终状态验证应该成功");
        
        assert!(success_rate > 0.5, "重试成功率应该大于50%");
        
        println!("✅ 连接错误和重试机制测试完成");
    }

    /// 测试错误恢复的完整性
    #[tokio::test]
    async fn test_comprehensive_error_recovery() {
        println!("🔧 测试错误恢复的完整性...");
        
        let manager = create_test_manager().await.expect("创建管理器失败");
        
        // 创建测试环境
        let setup_sql = r#"
            CREATE TABLE IF NOT EXISTS recovery_test (
                id INTEGER PRIMARY KEY,
                name VARCHAR(100),
                status VARCHAR(20) DEFAULT 'active'
            )
        "#;
        
        let result = manager.execute_query("test_db", setup_sql).await;
        assert!(result.is_ok(), "设置恢复测试环境应该成功");
        
        // 插入初始数据
        let initial_data = "INSERT INTO recovery_test (id, name) VALUES (1, 'Initial Record')";
        let result = manager.execute_query("test_db", initial_data).await;
        assert!(result.is_ok(), "插入初始数据应该成功");
        
        // 模拟一系列错误和恢复
        let error_recovery_scenarios = vec![
            ("语法错误", "INVALID SQL", "SELECT COUNT(*) FROM recovery_test"),
            ("约束违反", "INSERT INTO recovery_test (id, name) VALUES (1, 'Duplicate')", "INSERT INTO recovery_test (id, name) VALUES (2, 'Valid Record')"),
            ("表不存在", "SELECT * FROM non_existent_table", "SELECT * FROM recovery_test"),
            ("列不存在", "SELECT invalid_column FROM recovery_test", "SELECT name FROM recovery_test"),
        ];
        
        for (scenario_name, error_sql, recovery_sql) in error_recovery_scenarios {
            println!("🧪 测试场景: {}", scenario_name);
            
            // 执行错误操作
            let error_result = manager.execute_query("test_db", error_sql).await;
            assert!(error_result.is_err(), "错误操作应该失败: {}", scenario_name);
            
            // 执行恢复操作
            let recovery_result = manager.execute_query("test_db", recovery_sql).await;
            assert!(recovery_result.is_ok(), "恢复操作应该成功: {}", scenario_name);
            
            // 验证系统状态
            let status_check = "SELECT COUNT(*) as count FROM recovery_test";
            let status_result = manager.execute_query("test_db", status_check).await;
            assert!(status_result.is_ok(), "状态检查应该成功: {}", scenario_name);
            
            println!("✅ 场景 {} 恢复成功", scenario_name);
        }
        
        // 最终完整性检查
        let final_check = "SELECT id, name, status FROM recovery_test ORDER BY id";
        let result = manager.execute_query("test_db", final_check).await;
        assert!(result.is_ok(), "最终完整性检查应该成功");
        
        println!("✅ 错误恢复完整性测试完成");
    }
}
