# DuckHub - 金融数据平台核心底座

基于DuckDB+DuckDB Lake构建的现代化金融数据平台核心底座，使用Rust实现高性能、高可靠性的数据处理能力。

## 🚀 核心特性

### 📊 DuckDB核心引擎
- **高性能OLAP**: 基于DuckDB的列式存储和向量化执行
- **连接池管理**: 智能连接池，支持连接复用和自动扩缩容
- **查询优化**: 内置查询优化器，支持谓词下推、投影下推等优化策略
- **事务支持**: 完整的ACID事务支持

### 🏞️ 数据湖集成 (DuckDB Lake)
- **多格式支持**: Parquet、CSV、JSON、ORC、Avro等格式
- **对象存储**: 支持S3、Azure Blob、Google Cloud Storage
- **外部表**: 直接查询数据湖文件，无需数据移动
- **分区支持**: 智能分区策略，提升查询性能

### ⚡ 性能优化
- **智能缓存**: 多层缓存策略，支持Redis和内存缓存
- **并发处理**: 基于Tokio的异步处理，支持高并发
- **查询缓存**: 自动缓存查询结果，提升重复查询性能
- **连接复用**: 连接池管理，减少连接开销

### 📈 监控与指标
- **Prometheus集成**: 完整的指标收集和监控
- **性能追踪**: 查询执行时间、资源使用情况追踪
- **健康检查**: 实时健康状态监控
- **审计日志**: 完整的操作审计和日志记录

## 🏗️ 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                   应用层 (CLI/API)                          │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   查询执行层                                │
├─────────────────┬─────────────────┬─────────────────────────┤
│   查询优化器     │   缓存管理       │      指标收集            │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   连接池管理                                │
├─────────────────┬─────────────────┬─────────────────────────┤
│   连接复用       │   健康检查       │      负载均衡            │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   DuckDB引擎                               │
├─────────────────┬─────────────────┬─────────────────────────┤
│   DuckDB核心     │   Schema管理     │      数据湖集成          │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   存储层                                    │
├─────────────────┬─────────────────┬─────────────────────────┤
│   本地存储       │   对象存储       │      缓存存储            │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## 🛠️ 快速开始

### 环境要求

- Rust 1.75+
- DuckDB (通过bundled feature自动包含)
- Redis (可选，用于缓存)

### 安装和构建

```bash
# 克隆项目
git clone https://github.com/your-org/duckhub.git
cd duckhub

# 构建项目
cargo build --release

# 运行测试
cargo test

# 构建CLI工具
cargo build --release --bin duckhub
```

### 基础使用

#### 1. 使用CLI工具

```bash
# 查看帮助
./target/release/duckhub --help

# 执行SQL查询
./target/release/duckhub query "SELECT 1 as test"

# 查看数据库信息
./target/release/duckhub info

# 列出所有表
./target/release/duckhub schema list

# 创建表
./target/release/duckhub query "CREATE TABLE users (id INTEGER, name VARCHAR)"

# 插入数据
./target/release/duckhub query "INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')"

# 查询数据
./target/release/duckhub query "SELECT * FROM users" --format table
```

#### 2. 数据湖操作

```bash
# 创建外部表（从Parquet文件）
./target/release/duckhub lake create-table sales_data /path/to/sales.parquet --format parquet

# 直接查询数据湖文件
./target/release/duckhub lake query /path/to/data.parquet "SELECT COUNT(*) FROM table" --format parquet
```

#### 3. 性能测试

```bash
# 运行基准测试
./target/release/duckhub benchmark --count 1000 --concurrency 10
```

### 编程接口使用

```rust
use duckhub_common::prelude::*;
use duckhub_database::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建数据库配置
    let config = DatabaseConfig {
        duckdb_path: "data/my_database.db".to_string(),
        memory_limit: Some("4GB".to_string()),
        threads: Some(4),
        extensions: vec!["httpfs".to_string(), "parquet".to_string()],
        pool: PoolConfig::default(),
        ..Default::default()
    };

    // 创建数据库引擎
    let engine = Arc::new(DuckDBEngine::new(config)?);

    // 创建缓存（可选）
    let cache = Arc::new(QueryCacheImpl::new(
        "redis://localhost:6379",
        "duckhub".to_string(),
        3600,
    ).await?);

    // 创建查询执行器
    let executor = QueryExecutor::new(engine.clone(), Some(cache));

    // 执行查询
    let query = Query {
        id: generate_id(),
        sql: "SELECT COUNT(*) FROM my_table".to_string(),
        parameters: HashMap::new(),
        user_id: None,
        created_at: now(),
        timeout_seconds: None,
    };

    let result = executor.execute(&query).await?;
    println!("查询结果: {} 行", result.row_count);

    Ok(())
}
```

## 📊 性能特性

### 查询性能
- **向量化执行**: DuckDB的SIMD优化，单核性能优异
- **列式存储**: 高效的数据压缩和查询性能
- **并行处理**: 多线程并行查询执行
- **智能索引**: 自动创建和使用索引

### 缓存性能
- **查询结果缓存**: 自动缓存查询结果，提升重复查询性能
- **元数据缓存**: 缓存表结构和统计信息
- **连接池**: 连接复用，减少连接建立开销
- **智能预热**: 基于访问模式的缓存预热

### 扩展性
- **水平扩展**: 支持多实例部署
- **垂直扩展**: 充分利用多核CPU和大内存
- **存储扩展**: 支持本地存储和云存储
- **插件扩展**: 预留WASM插件接口

## 🔧 配置说明

### 数据库配置

```toml
[database]
duckdb_path = "data/duckhub.db"  # 数据库文件路径
memory_limit = "4GB"             # 内存限制
threads = 4                      # 线程数
extensions = ["httpfs", "parquet"] # 扩展插件

[database.pool]
min_connections = 1              # 最小连接数
max_connections = 10             # 最大连接数
connection_timeout = 30          # 连接超时(秒)
idle_timeout = 600               # 空闲超时(秒)
max_lifetime = 3600              # 连接最大生命周期(秒)
```

### 缓存配置

```toml
[cache]
redis_url = "redis://localhost:6379"  # Redis连接URL
pool_size = 10                         # 连接池大小
default_ttl = 3600                     # 默认TTL(秒)
max_key_size = 1024                    # 最大键长度
max_value_size = 1048576               # 最大值大小(字节)
```

## 📈 监控指标

### 查询指标
- `duckhub_queries_total`: 总查询数
- `duckhub_query_duration_seconds`: 查询执行时间
- `duckhub_query_errors_total`: 查询错误数
- `duckhub_query_cache_hits_total`: 缓存命中数

### 连接指标
- `duckhub_connections_active`: 活跃连接数
- `duckhub_connections_idle`: 空闲连接数
- `duckhub_connection_errors_total`: 连接错误数

### 数据库指标
- `duckhub_database_size_bytes`: 数据库大小
- `duckhub_table_count`: 表数量
- `duckhub_rows_scanned_total`: 扫描行数
- `duckhub_rows_returned_total`: 返回行数

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration

# 运行特定测试
cargo test test_duckdb_engine

# 运行性能测试
cargo test --release -- --ignored
```

## 🚀 部署

### Docker部署

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/duckhub /usr/local/bin/
EXPOSE 8080
CMD ["duckhub"]
```

### Kubernetes部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: duckhub
spec:
  replicas: 3
  selector:
    matchLabels:
      app: duckhub
  template:
    metadata:
      labels:
        app: duckhub
    spec:
      containers:
      - name: duckhub
        image: duckhub:latest
        ports:
        - containerPort: 8080
        env:
        - name: DUCKHUB_DATABASE__DUCKDB_PATH
          value: "/data/duckhub.db"
        volumeMounts:
        - name: data
          mountPath: /data
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: duckhub-data
```

## 🤝 贡献

欢迎贡献代码！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细信息。

## 📄 许可证

本项目采用 MIT 或 Apache-2.0 双重许可证。详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。

## 🔗 相关链接

- [DuckDB官网](https://duckdb.org/)
- [Rust官网](https://www.rust-lang.org/)
- [Tokio异步运行时](https://tokio.rs/)
- [Actix Web框架](https://actix.rs/)

---

**DuckHub** - 为金融数据分析而生的高性能数据平台核心底座 🚀
