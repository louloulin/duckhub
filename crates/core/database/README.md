# DuckHub Database - DuckLake Manager

## 概述

DuckHub Database 是一个基于 DuckDB 的高性能数据库管理系统，专为金融数据处理和分析而设计。核心组件是 **DuckLake Manager**，它提供了类似 Delta Lake 的功能，包括时间旅行查询、快照管理和 ACID 事务支持。

## 主要特性

### 🦆 DuckLake Manager
- **时间旅行查询**: 支持查询历史数据版本
- **快照管理**: 创建和管理数据库快照
- **ACID 事务**: 完整的事务支持
- **模式演进**: 支持表结构的动态变更
- **多数据库支持**: 可以附加和管理多个数据库

### 🚀 性能优化
- **内存池管理**: 高效的内存分配和回收
- **连接池**: 数据库连接的复用和管理
- **查询优化**: 智能查询计划优化
- **缓存系统**: 多层缓存提升查询性能

### 📊 实时数据处理
- **流处理器**: 高吞吐量的数据流处理
- **背压控制**: 自动调节处理速度
- **水印管理**: 处理乱序数据
- **容错机制**: 自动故障恢复

### 🔒 风险管理
- **实时风险引擎**: 实时计算风险指标
- **VaR 模型**: 风险价值计算
- **阈值监控**: 自动风险预警
- **合规检查**: 实时合规性验证

## 快速开始

### 安装依赖

```toml
[dependencies]
duckhub-database = "0.1.0"
duckhub-common = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基本使用

```rust
use duckhub_database::ducklake_real::DuckLakeManager;
use duckhub_database::real_duckdb::Connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建内存数据库连接
    let connection = Connection::open_in_memory().await?;
    
    // 创建 DuckLake Manager
    let manager = DuckLakeManager::new(connection).await?;
    
    // 创建表
    let schema = r#"(
        id INTEGER PRIMARY KEY,
        symbol VARCHAR NOT NULL,
        price DOUBLE NOT NULL,
        quantity INTEGER NOT NULL,
        timestamp TIMESTAMP NOT NULL
    )"#;
    
    manager.create_table("main", "trades", schema).await?;
    
    // 插入数据
    let insert_sql = r#"
        INSERT INTO trades (id, symbol, price, quantity, timestamp) VALUES
        (1, 'AAPL', 150.25, 100, '2024-01-01 10:00:00')
    "#;
    
    manager.execute_query("main", insert_sql).await?;
    
    // 查询数据
    let query_sql = "SELECT * FROM main.trades";
    let result = manager.execute_query("main", query_sql).await?;
    
    println!("查询结果: {:?}", result);
    
    Ok(())
}
```

## 运行示例

```bash
# 运行 DuckLake Manager 示例
cargo run --package duckhub-database --example ducklake_example

# 运行测试
cargo test --package duckhub-database --lib
```

## 架构组件

### 核心模块

1. **DuckLake Manager** (`ducklake_real.rs`)
   - 主要的数据库管理接口
   - 提供表创建、数据插入、查询等功能
   - 支持快照和时间旅行查询

2. **连接管理** (`real_duckdb.rs`)
   - DuckDB 连接的封装
   - 异步查询执行
   - 连接池管理

3. **流处理器** (`stream_processor.rs`)
   - 高性能数据流处理
   - 背压控制和水印管理
   - 容错和检查点机制

4. **风险引擎** (`risk_engine.rs`)
   - 实时风险计算
   - VaR 模型和阈值监控
   - 风险预警系统

5. **内存优化** (`memory_optimization.rs`)
   - 内存池和缓冲区管理
   - 对象池优化
   - 内存使用监控

### 支持模块

- **查询优化器** (`query.rs`): SQL 查询优化
- **缓存系统** (`cache.rs`): 多层缓存机制
- **指标收集** (`metrics.rs`): 性能监控和指标
- **数据湖** (`lake.rs`): 外部数据源集成
- **模式管理** (`schema.rs`): 数据库模式管理

## 测试覆盖

项目包含全面的测试套件：

- **单元测试**: 35个测试用例，覆盖所有核心功能
- **集成测试**: 测试组件间的交互
- **性能测试**: 验证系统性能指标
- **示例程序**: 演示实际使用场景

```bash
# 运行所有测试
cargo test --package duckhub-database

# 运行特定测试
cargo test --package duckhub-database test_ducklake_manager_creation
```

## 性能特性

- **高并发**: 支持数千个并发连接
- **低延迟**: 毫秒级查询响应
- **高吞吐**: 每秒处理数万条记录
- **内存效率**: 智能内存管理和回收
- **可扩展**: 支持水平和垂直扩展

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 LICENSE 文件。
