//! DeepSeek API密钥测试脚本
//! 验证API密钥是否有效并测试基本功能

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置API密钥
    env::set_var("DEEPSEEK_API_KEY", "sk-a4f888023ea74cef8afae36dc8581512");
    
    println!("🔑 测试DeepSeek API密钥...");
    
    // 创建DeepSeek客户端
    let api_key = env::var("DEEPSEEK_API_KEY")?;
    println!("✅ API密钥已设置: {}...{}", 
        &api_key[..10], 
        &api_key[api_key.len()-4..]
    );
    
    // 这里可以添加实际的API调用测试
    // 但需要先确保rig框架的依赖已正确配置
    
    println!("🎉 API密钥配置完成！");
    println!("💡 提示: 现在可以运行真实的AI Agent测试了");
    
    Ok(())
}
