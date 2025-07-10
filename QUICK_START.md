# DuckHub 快速启动指南

## 🚀 5分钟快速体验

### 前置要求
- Rust 1.70+ 
- Node.js 18+
- Git

### 1. 克隆和编译项目

```bash
# 克隆项目
git clone <repository-url>
cd duckhub

# 编译项目 (Release模式，性能最佳)
cargo build --release

# 运行测试确保一切正常
cargo test --workspace
```

### 2. 启动CLI工具体验

```bash
# 查看帮助
./target/release/duckhub --help

# 创建示例数据
./target/release/duckhub sample

# 执行简单查询
./target/release/duckhub query "SELECT * FROM users"

# 启动交互式模式
./target/release/duckhub interactive
```

### 3. 启动API服务

```bash
# 启动API网关 (默认端口8080)
./target/release/duckhub-api-gateway

# 测试API (新终端)
curl http://localhost:8080/api/health
```

### 4. 启动Web前端

```bash
# 进入前端目录
cd crates/web-frontend

# 安装依赖
npm install

# 启动开发服务器 (端口3000)
npm run dev
```

### 5. 访问Web界面

打开浏览器访问: http://localhost:3000

- 🏠 **仪表板**: 查看系统概览和实时指标
- 📊 **查询分析**: 执行SQL查询和性能分析  
- 🗃️ **数据探索**: 浏览表结构和数据内容
- 🤖 **AI助手**: 使用自然语言进行数据查询
- ⚙️ **设置**: 配置系统参数

## 📖 核心功能演示

### DuckLake数据湖功能

```bash
# 创建带版本控制的表
./target/release/duckhub query "
CREATE TABLE transactions_lake (
    id BIGINT,
    user_id BIGINT, 
    amount DECIMAL(10,2),
    created_at TIMESTAMP
) WITH (
    format = 'delta',
    versioning = true
)"

# 插入数据并创建版本
./target/release/duckhub query "
INSERT INTO transactions_lake VALUES 
(1, 1001, 1000.00, '2024-01-01 10:00:00'),
(2, 1002, 2000.00, '2024-01-01 11:00:00')"

# 时间旅行查询 - 查看历史版本
./target/release/duckhub query "
SELECT * FROM transactions_lake 
VERSION AS OF '2024-01-01 10:30:00'"
```

### AI智能查询

在Web界面的AI助手页面尝试：

```
"显示本月交易金额最大的10笔记录"
"分析用户的消费趋势"  
"检测异常交易模式"
"生成月度财务报告"
```

### 窗口函数分析

```sql
-- 计算移动平均
SELECT 
    user_id,
    amount,
    AVG(amount) OVER (
        PARTITION BY user_id 
        ORDER BY created_at 
        ROWS BETWEEN 2 PRECEDING AND CURRENT ROW
    ) as moving_avg
FROM transactions
ORDER BY user_id, created_at;

-- 排名分析
SELECT 
    user_id,
    amount,
    RANK() OVER (ORDER BY amount DESC) as rank,
    PERCENT_RANK() OVER (ORDER BY amount) as percentile
FROM transactions;
```

### 时间序列分析

```sql
-- 日交易量趋势
SELECT 
    DATE(created_at) as date,
    COUNT(*) as daily_count,
    SUM(amount) as daily_total,
    LAG(COUNT(*)) OVER (ORDER BY DATE(created_at)) as prev_day_count
FROM transactions 
GROUP BY DATE(created_at)
ORDER BY date;
```

## 🔧 配置说明

### 数据库配置

编辑 `config/database.toml`:

```toml
[database]
duckdb_path = "duckhub.db"
memory_limit = "2GB"
threads = 4
temp_directory = "/tmp/duckhub"

[ducklake]
enable_versioning = true
enable_time_travel = true
retention_days = 30
```

### 服务配置

编辑 `config/services.toml`:

```toml
[api_gateway]
host = "0.0.0.0"
port = 8080
cors_enabled = true

[ai_agent]
model_provider = "openai"
api_key = "your-api-key"
enable_nlp = true

[monitoring]
metrics_enabled = true
prometheus_port = 9090
```

### 认证配置

编辑 `config/auth.toml`:

```toml
[jwt]
secret_key = "your-secret-key"
expiration_hours = 24

[rbac]
enable_rbac = true
default_role = "viewer"
```

## 📊 监控和运维

### 健康检查

```bash
# API健康状态
curl http://localhost:8080/api/health

# 系统指标
curl http://localhost:8080/api/metrics

# 数据库状态
./target/release/duckhub info
```

### 性能监控

访问监控仪表板查看：
- 查询执行时间分布
- 系统资源使用情况  
- 缓存命中率统计
- 错误率和告警信息

### 日志查看

```bash
# 查看服务日志
tail -f logs/duckhub.log

# 查看查询日志
tail -f logs/queries.log

# 查看错误日志
tail -f logs/errors.log
```

## 🛠️ 开发和扩展

### 添加自定义函数

```rust
// 在 crates/database/src/functions/custom.rs
pub fn register_custom_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "financial_risk_score",
        1,
        FunctionFlags::SQLITE_UTF8,
        |ctx| {
            let amount: f64 = ctx.get(0)?;
            let risk_score = calculate_risk(amount);
            Ok(risk_score)
        },
    )?;
    Ok(())
}
```

### 添加新的API端点

```rust
// 在 crates/api-gateway/src/routes/custom.rs
#[get("/api/custom/analysis")]
pub async fn custom_analysis() -> impl Responder {
    // 自定义分析逻辑
    HttpResponse::Ok().json(analysis_result)
}
```

### 扩展AI功能

```rust
// 在 crates/services/ai-agent/src/processors/
pub struct CustomAnalyzer {
    // 自定义分析器实现
}

impl QueryProcessor for CustomAnalyzer {
    async fn process(&self, query: &str) -> Result<ProcessedQuery> {
        // 自定义处理逻辑
    }
}
```

## 🔍 故障排除

### 常见问题

1. **编译失败**
   ```bash
   # 更新Rust工具链
   rustup update
   
   # 清理并重新编译
   cargo clean && cargo build --release
   ```

2. **数据库连接失败**
   ```bash
   # 检查数据库文件权限
   ls -la duckhub.db
   
   # 重新创建数据库
   rm duckhub.db && ./target/release/duckhub sample
   ```

3. **前端启动失败**
   ```bash
   # 清理node_modules
   rm -rf node_modules package-lock.json
   npm install
   ```

4. **API服务无响应**
   ```bash
   # 检查端口占用
   lsof -i :8080
   
   # 查看服务日志
   ./target/release/duckhub-api-gateway --log-level debug
   ```

### 获取帮助

- 📖 查看完整文档: `docs/`
- 🐛 报告问题: GitHub Issues
- 💬 社区讨论: GitHub Discussions
- 📧 技术支持: support@duckhub.dev

## 🎉 下一步

恭喜！您已经成功启动了DuckHub金融数据平台。现在可以：

1. 📊 **探索数据**: 使用Web界面浏览和分析数据
2. 🤖 **体验AI**: 尝试自然语言查询功能
3. 🔧 **自定义配置**: 根据需求调整系统参数
4. 📈 **监控性能**: 查看实时系统指标
5. 🚀 **生产部署**: 参考部署文档进行生产环境配置

享受使用DuckHub带来的强大数据分析能力！ 🚀
