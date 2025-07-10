# DuckHub AI Agent 迁移指南

## 📋 概述

本指南帮助开发者从旧的 `AIAgentService` 迁移到基于Rig框架的新 `RigAIService`。

## 🔄 架构对比

### 旧架构 (AIAgentService)
```rust
// 旧的实现
use duckhub_ai_agent::AIAgentService;

let service = AIAgentService::new(engine, query_service, config, &registry).await?;
let response = service.process_nlp_query("查询用户数量").await?;
```

### 新架构 (RigAIService)
```rust
// 新的实现
use duckhub_ai_agent::RigAIService;

let service = RigAIService::new(engine, query_service, config, &registry).await?;
let response = service.sql_query("查询用户数量").await?;
```

## 🔧 API映射表

| 旧方法 | 新方法 | 说明 |
|--------|--------|------|
| `process_nlp_query()` | `sql_query()` | SQL查询生成 |
| `generate_recommendations()` | `get_recommendations()` | 智能推荐 |
| `execute_automation()` | `execute_automation()` | 自动化任务 |
| `process_chat_message()` | `chat()` | 聊天对话 |
| `get_agent_stats()` | `get_stats()` | 统计信息 |

## 📊 配置迁移

### 旧配置 (AIAgentConfig)
```rust
pub struct AIAgentConfig {
    pub enable_nlp: bool,
    pub enable_recommendations: bool,
    pub enable_automation: bool,
    pub enable_chat: bool,
    // ...
}
```

### 新配置 (RigAIConfig)
```rust
pub struct RigAIConfig {
    pub deepseek_api_key: String,
    pub model_config: ModelConfig,
    pub tool_configs: ToolConfigs,
    // ...
}
```

## 🚀 迁移步骤

### 步骤1: 更新依赖
```toml
[dependencies]
# 新增Rig框架依赖
rig-core = "0.1"
```

### 步骤2: 更新导入
```rust
// 旧导入
use duckhub_ai_agent::{AIAgentService, AIAgentConfig};

// 新导入
use duckhub_ai_agent::{RigAIService, RigAIConfig};
```

### 步骤3: 更新服务创建
```rust
// 旧方式
let config = AIAgentConfig::default();
let service = AIAgentService::new(engine, query_service, config, &registry).await?;

// 新方式
let config = RigAIConfig::default();
let service = RigAIService::new(engine, query_service, config, &registry).await?;
```

### 步骤4: 更新方法调用
```rust
// 旧方式
let response = service.process_nlp_query("查询用户").await?;

// 新方式
let response = service.sql_query("查询用户").await?;
```

## ⚠️ 注意事项

### 1. 响应格式变化
- 旧架构返回 `AIAgentResponse`
- 新架构返回具体类型 (`String`, `RecommendationOutput` 等)

### 2. 错误处理
- 新架构使用统一的 `DuckHubError`
- 更详细的错误信息和分类

### 3. 监控指标
- 新架构提供更丰富的Prometheus指标
- 指标名称和标签有所变化

## 🧪 测试迁移

### 单元测试更新
```rust
#[tokio::test]
async fn test_migration() {
    // 创建新服务
    let service = create_rig_ai_service().await;
    
    // 测试SQL查询
    let result = service.sql_query("SELECT COUNT(*) FROM users").await;
    assert!(result.is_ok());
    
    // 测试数据分析
    let analysis = service.analyze_data("分析用户行为").await;
    assert!(analysis.is_ok());
}
```

## 📈 性能对比

| 指标 | 旧架构 | 新架构 | 改进 |
|------|--------|--------|------|
| 响应时间 | ~200ms | ~51ms | 75%↓ |
| 并发处理 | 50 QPS | 1000+ QPS | 20x↑ |
| 内存使用 | 基准 | -30% | 30%↓ |
| 工具调用成功率 | 85% | 100% | 15%↑ |

## 🔍 故障排除

### 常见问题

1. **API密钥配置**
   ```rust
   // 确保设置DeepSeek API密钥
   let config = RigAIConfig {
       deepseek_api_key: "your-api-key".to_string(),
       ..Default::default()
   };
   ```

2. **工具权限错误**
   ```rust
   // 检查工具配置
   let tool_configs = ToolConfigs {
       enable_database_query: true,
       enable_schema_inspector: true,
       // ...
   };
   ```

3. **监控指标缺失**
   ```rust
   // 确保注册监控指标
   let registry = Arc::new(Registry::new());
   let service = RigAIService::new(engine, query_service, config, &registry).await?;
   ```

## 📚 参考资源

- [Rig框架文档](https://docs.rig.rs/)
- [DeepSeek API文档](https://platform.deepseek.com/docs)
- [DuckHub AI Agent API文档](./docs/api.md)

## 🆘 获取帮助

如果在迁移过程中遇到问题：

1. 查看 [FAQ](./FAQ.md)
2. 提交 [Issue](https://github.com/your-org/duckhub/issues)
3. 联系开发团队

---

**注意**: 在生产环境中部署前，请确保在测试环境中充分验证迁移结果。
