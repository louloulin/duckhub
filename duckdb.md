# DuckDB-rs 动态插件安装支持分析

## 📋 概述

基于对DuckDB官方文档和duckdb-rs crate的深入研究，本文档分析了DuckDB-rs对动态插件安装的支持情况。

## ✅ 核心结论

**DuckDB-rs 完全支持动态插件安装**，通过SQL命令在运行时安装和加载扩展插件。

## 🔧 技术实现

### 1. 基本机制

DuckDB-rs通过执行SQL命令来实现动态插件管理：

```rust
use duckdb::{Connection, Result};

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // 安装扩展（下载并安装）
    conn.execute("INSTALL httpfs", [])?;
    
    // 加载扩展到当前会话
    conn.execute("LOAD httpfs", [])?;
    
    // 现在可以使用httpfs功能
    conn.execute("SELECT * FROM 's3://bucket/file.parquet'", [])?;
    
    Ok(())
}
```

### 2. 支持的SQL命令

#### INSTALL 命令
- **功能**: 下载并安装扩展到本地
- **语法**: `INSTALL extension_name [FROM repository]`
- **示例**:
  ```sql
  INSTALL httpfs;                    -- 安装核心扩展
  INSTALL h3 FROM community;         -- 安装社区扩展
  INSTALL spatial FROM core;         -- 从特定仓库安装
  FORCE INSTALL httpfs;              -- 强制重新安装
  ```

#### LOAD 命令
- **功能**: 将已安装的扩展加载到当前会话
- **语法**: `LOAD extension_name`
- **示例**:
  ```sql
  LOAD httpfs;                       -- 加载httpfs扩展
  LOAD spatial;                      -- 加载spatial扩展
  ```

### 3. 扩展类型

#### 核心扩展 (Core Extensions)
- **httpfs**: HTTP/S3文件系统支持
- **parquet**: Parquet文件格式支持
- **json**: JSON数据处理
- **fts**: 全文搜索
- **spatial**: 空间数据支持

#### 社区扩展 (Community Extensions)
- **delta**: Delta Lake支持
- **h3**: H3地理索引
- **postgres_scanner**: PostgreSQL扫描器
- **sqlite_scanner**: SQLite扫描器

## 🚀 实际应用示例

### 示例1: S3数据访问

```rust
use duckdb::{Connection, Result};

async fn setup_s3_access() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // 1. 安装和加载httpfs扩展
    conn.execute("INSTALL httpfs", [])?;
    conn.execute("LOAD httpfs", [])?;
    
    // 2. 配置S3凭证
    conn.execute("SET s3_region='us-east-1'", [])?;
    conn.execute("SET s3_access_key_id='your-key'", [])?;
    conn.execute("SET s3_secret_access_key='your-secret'", [])?;
    
    // 3. 查询S3数据
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM 's3://bucket/data.parquet'")?;
    let result = stmt.query_row([], |row| {
        let count: i64 = row.get(0)?;
        Ok(count)
    })?;
    
    println!("Total rows: {}", result);
    Ok(())
}
```

### 示例2: Delta Lake支持

```rust
use duckdb::{Connection, Result};

async fn setup_delta_lake() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // 1. 安装Delta扩展
    conn.execute("INSTALL delta", [])?;
    conn.execute("LOAD delta", [])?;
    
    // 2. 查询Delta表
    conn.execute("SELECT * FROM delta_scan('s3://bucket/delta-table/')", [])?;
    
    // 3. 时间旅行查询
    conn.execute("SELECT * FROM delta_scan('s3://bucket/delta-table/', version => 1)", [])?;
    
    Ok(())
}
```

### 示例3: 多扩展组合使用

```rust
use duckdb::{Connection, Result};

async fn setup_comprehensive_extensions() -> Result<()> {
    let conn = Connection::open_in_memory()?;
    
    // 批量安装扩展
    let extensions = vec![
        "httpfs",      // S3/HTTP支持
        "parquet",     // Parquet格式
        "json",        // JSON处理
        "spatial",     // 空间数据
        "fts",         // 全文搜索
    ];
    
    for ext in extensions {
        // 安装扩展（如果失败可能已经安装）
        if let Err(e) = conn.execute(&format!("INSTALL {}", ext), []) {
            println!("Extension {} might already be installed: {}", ext, e);
        }
        
        // 加载扩展
        conn.execute(&format!("LOAD {}", ext), [])?;
        println!("Loaded extension: {}", ext);
    }
    
    // 现在可以使用所有扩展功能
    conn.execute(r#"
        SELECT 
            ST_Distance(ST_Point(0, 0), ST_Point(1, 1)) as distance,
            json_extract('{"name": "test"}', '$.name') as name
        FROM 's3://bucket/spatial_data.parquet'
    "#, [])?;
    
    Ok(())
}
```

## 🔍 高级特性

### 1. 扩展仓库管理

```rust
// 从不同仓库安装扩展
conn.execute("INSTALL h3 FROM community", [])?;
conn.execute("INSTALL custom_ext FROM 'https://custom-repo.com'", [])?;
```

### 2. 强制重新安装

```rust
// 强制重新下载和安装扩展
conn.execute("FORCE INSTALL httpfs", [])?;
```

### 3. 扩展信息查询

```rust
// 查看已安装的扩展
let mut stmt = conn.prepare("SELECT * FROM duckdb_extensions()")?;
let rows = stmt.query_map([], |row| {
    let name: String = row.get(0)?;
    let loaded: bool = row.get(1)?;
    let installed: bool = row.get(2)?;
    Ok((name, loaded, installed))
})?;

for row in rows {
    let (name, loaded, installed) = row?;
    println!("Extension: {}, Loaded: {}, Installed: {}", name, loaded, installed);
}
```

## ⚠️ 注意事项和限制

### 1. 网络依赖
- 首次安装需要网络连接下载扩展
- 可以通过`FORCE INSTALL`强制重新下载
- 离线环境需要预先安装扩展

### 2. 平台兼容性
- 扩展需要与目标平台兼容
- 某些扩展可能不支持所有操作系统
- ARM64平台支持可能有限

### 3. 版本兼容性
- 扩展版本需要与DuckDB版本兼容
- 建议使用稳定版本的扩展

### 4. 错误处理

```rust
use duckdb::{Connection, Error};

fn install_extension_safely(conn: &Connection, ext_name: &str) -> Result<(), Error> {
    // 尝试安装扩展
    match conn.execute(&format!("INSTALL {}", ext_name), []) {
        Ok(_) => println!("Successfully installed {}", ext_name),
        Err(e) => {
            // 检查是否是"已安装"错误
            if e.to_string().contains("already exists") {
                println!("Extension {} already installed", ext_name);
            } else {
                return Err(e);
            }
        }
    }
    
    // 加载扩展
    conn.execute(&format!("LOAD {}", ext_name), [])?;
    println!("Successfully loaded {}", ext_name);
    
    Ok(())
}
```

## 🏗️ 最佳实践

### 1. 扩展管理策略

```rust
pub struct ExtensionManager {
    conn: Connection,
    installed_extensions: HashSet<String>,
}

impl ExtensionManager {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            installed_extensions: HashSet::new(),
        }
    }
    
    pub async fn ensure_extension(&mut self, name: &str) -> Result<(), Error> {
        if self.installed_extensions.contains(name) {
            return Ok(());
        }
        
        // 尝试安装
        if let Err(_) = self.conn.execute(&format!("INSTALL {}", name), []) {
            // 可能已经安装，继续加载
        }
        
        // 加载扩展
        self.conn.execute(&format!("LOAD {}", name), [])?;
        self.installed_extensions.insert(name.to_string());
        
        Ok(())
    }
    
    pub async fn setup_data_lake_extensions(&mut self) -> Result<(), Error> {
        let extensions = vec!["httpfs", "parquet", "delta", "json"];
        
        for ext in extensions {
            self.ensure_extension(ext).await?;
        }
        
        Ok(())
    }
}
```

### 2. 配置驱动的扩展管理

```rust
#[derive(Deserialize)]
struct ExtensionConfig {
    name: String,
    required: bool,
    repository: Option<String>,
}

async fn setup_extensions_from_config(
    conn: &Connection, 
    configs: Vec<ExtensionConfig>
) -> Result<(), Error> {
    for config in configs {
        let install_cmd = if let Some(repo) = config.repository {
            format!("INSTALL {} FROM {}", config.name, repo)
        } else {
            format!("INSTALL {}", config.name)
        };
        
        match conn.execute(&install_cmd, []) {
            Ok(_) => {},
            Err(e) if !config.required => {
                println!("Optional extension {} failed to install: {}", config.name, e);
                continue;
            },
            Err(e) => return Err(e),
        }
        
        conn.execute(&format!("LOAD {}", config.name), [])?;
    }
    
    Ok(())
}
```

## 📊 性能考虑

### 1. 扩展加载时间
- 首次安装: 1-5秒（取决于网络和扩展大小）
- 后续加载: <100ms
- 建议在应用启动时预加载常用扩展

### 2. 内存使用
- 每个扩展增加5-50MB内存使用
- 只加载必需的扩展以优化内存

### 3. 启动优化

```rust
// 并行安装扩展（如果支持）
async fn parallel_extension_setup(conn: &Connection) -> Result<(), Error> {
    let extensions = vec!["httpfs", "parquet", "json", "spatial"];
    
    // 串行安装（DuckDB连接不支持并发）
    for ext in extensions {
        conn.execute(&format!("INSTALL {}", ext), []).ok(); // 忽略错误
        conn.execute(&format!("LOAD {}", ext), [])?;
    }
    
    Ok(())
}
```

## 🎯 总结

DuckDB-rs **完全支持动态插件安装**，具备以下特点：

✅ **运行时安装**: 通过SQL命令动态安装扩展  
✅ **多仓库支持**: 支持核心、社区和自定义扩展仓库  
✅ **错误处理**: 提供完善的错误处理机制  
✅ **版本管理**: 支持强制重新安装和版本控制  
✅ **平台兼容**: 支持主流操作系统和架构  

这使得DuckDB-rs非常适合构建需要灵活扩展能力的数据平台，可以根据实际需求动态加载所需的功能模块。
