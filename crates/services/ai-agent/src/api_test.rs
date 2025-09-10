//! DeepSeek API真实测试
//! 验证API密钥有效性和基本功能

#[cfg(test)]
mod api_tests {
    use super::super::rig_agent::*;
    use std::env;
    use rig::client::CompletionClient;
    use rig::completion::Prompt;

    /// 测试真实的DeepSeek API调用
    #[tokio::test]
    #[ignore] // 默认忽略，需要真实API密钥时手动运行
    async fn test_real_deepseek_api() {
        // 检查API密钥是否已设置
        if env::var("DEEPSEEK_API_KEY").is_err() {
            println!("⚠️  请设置DEEPSEEK_API_KEY环境变量后运行此测试");
            return;
        }
        
        // 创建配置
        let config = RigAIConfig::default();
        
        // 验证API密钥已正确设置
        assert!(!config.deepseek_api_key.is_empty(), "API密钥不应为空");
        println!("✅ API密钥配置正确");
        
        // 创建DeepSeek客户端
        use rig::providers::deepseek;
        let client = deepseek::Client::new(&config.deepseek_api_key);
        println!("✅ DeepSeek客户端创建成功");

        // 创建一个简单的Agent进行测试
        let agent = client
            .agent(&config.model_config.primary_model)
            .preamble("你是一个专业的SQL查询生成专家。请根据用户的自然语言描述生成对应的SQL查询。")
            .build();
        
        // 测试SQL生成
        let test_query = "查询所有用户的数量";
        println!("🔍 测试查询: {}", test_query);
        
        match agent.prompt(test_query).await {
            Ok(response) => {
                println!("✅ API调用成功!");
                println!("📝 生成的SQL: {}", response);
                
                // 验证响应包含SQL关键词
                let response_upper = response.to_uppercase();
                assert!(
                    response_upper.contains("SELECT") || 
                    response_upper.contains("COUNT"),
                    "生成的响应应该包含SQL关键词"
                );
                
                println!("🎉 SQL生成准确率测试通过!");
            }
            Err(e) => {
                panic!("❌ API调用失败: {}", e);
            }
        }
    }
    
    /// 测试多种查询类型的SQL生成准确率
    #[tokio::test]
    #[ignore] // 需要真实API密钥
    async fn test_sql_generation_accuracy_real() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            println!("⚠️  请设置DEEPSEEK_API_KEY环境变量后运行此测试");
            return;
        }
        
        let config = RigAIConfig::default();
        use rig::providers::deepseek;
        let client = deepseek::Client::new(&config.deepseek_api_key);

        let agent = client
            .agent(&config.model_config.primary_model)
            .preamble("你是一个专业的SQL查询生成专家。请根据用户的自然语言描述生成对应的SQL查询。只返回SQL语句，不要其他解释。")
            .build();
        
        let test_cases = vec![
            ("查询所有用户", "SELECT"),
            ("统计用户数量", "COUNT"),
            ("按年龄分组", "GROUP BY"),
            ("排序结果", "ORDER BY"),
            ("过滤条件", "WHERE"),
            ("聚合函数", "SUM"),
            ("去重查询", "DISTINCT"),
        ];
        
        let mut accurate_queries = 0;
        let total_queries = test_cases.len();
        
        for (description, expected_keyword) in test_cases {
            println!("🔍 测试: {}", description);
            
            match agent.prompt(description).await {
                Ok(sql) => {
                    let sql_upper = sql.to_uppercase();
                    if sql_upper.contains(expected_keyword) {
                        accurate_queries += 1;
                        println!("✅ 准确: '{}' -> 包含 '{}'", description, expected_keyword);
                    } else {
                        println!("❌ 不准确: '{}' -> 缺少 '{}'", description, expected_keyword);
                        println!("   生成的SQL: {}", sql);
                    }
                }
                Err(e) => {
                    println!("❌ API调用失败: '{}' -> {}", description, e);
                }
            }
            
            // 避免API限流
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        let accuracy_rate = (accurate_queries as f64 / total_queries as f64) * 100.0;
        
        println!("📊 真实SQL生成准确率测试结果:");
        println!("   - 总查询数: {}", total_queries);
        println!("   - 准确查询数: {}", accurate_queries);
        println!("   - 准确率: {:.1}%", accuracy_rate);
        
        // 验证准确率达到目标
        assert!(accuracy_rate >= 80.0, 
            "真实API的SQL生成准确率 {:.1}% 应该 >= 80%", accuracy_rate);
        
        if accuracy_rate >= 95.0 {
            println!("🎉 优秀! 准确率达到95%以上!");
        }
    }
    
    /// 测试响应时间
    #[tokio::test]
    #[ignore] // 需要真实API密钥
    async fn test_real_response_time() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            println!("⚠️  请设置DEEPSEEK_API_KEY环境变量后运行此测试");
            return;
        }
        
        let config = RigAIConfig::default();
        use rig::providers::deepseek;
        let client = deepseek::Client::new(&config.deepseek_api_key);

        let agent = client
            .agent(&config.model_config.primary_model)
            .preamble("你是一个AI助手，请简洁回答问题。")
            .build();
        
        let start = std::time::Instant::now();
        
        match agent.prompt("你好").await {
            Ok(response) => {
                let duration = start.elapsed();
                println!("✅ 响应时间: {}ms", duration.as_millis());
                println!("📝 响应内容: {}", response);
                
                // 验证响应时间在合理范围内（真实API可能较慢）
                assert!(duration.as_secs() < 30, "响应时间应该在30秒内");
                
                if duration.as_secs() < 2 {
                    println!("🚀 优秀! 响应时间 < 2秒");
                }
            }
            Err(e) => {
                panic!("❌ API调用失败: {}", e);
            }
        }
    }
}
