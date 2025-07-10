//! DuckLake性能基准测试
//! 
//! 测试DuckLake在金融数据处理场景下的性能表现

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use duckhub_database::*;
use duckhub_common::prelude::*;
use duckdb::Connection;
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// 创建基准测试用的DuckLake管理器
fn create_benchmark_manager() -> DuckLakeManager {
    let conn = Connection::open_in_memory().unwrap();
    
    // 尝试安装必要的扩展
    let _ = conn.execute("INSTALL ducklake", []);
    let _ = conn.execute("LOAD ducklake", []);
    let _ = conn.execute("INSTALL httpfs", []);
    let _ = conn.execute("LOAD httpfs", []);
    
    DuckLakeManager::new(conn)
}

/// 创建基准测试用的配置
fn create_benchmark_config() -> DuckLakeConfig {
    DuckLakeConfig {
        metadata_path: ":memory:".to_string(),
        data_path: Some(":memory:.files".to_string()),
        metadata_schema: Some("main".to_string()),
        metadata_catalog: None,
        encrypted: false,
        data_inlining_row_limit: 1000, // 启用数据内联以提高小数据集性能
        read_only: false,
        snapshot_version: None,
        snapshot_time: None,
        metadata_parameters: HashMap::new(),
    }
}

/// 生成测试用的金融交易数据
fn generate_financial_data(num_records: usize) -> Vec<Vec<serde_json::Value>> {
    let mut data = Vec::with_capacity(num_records);
    
    for i in 0..num_records {
        let transaction = vec![
            serde_json::Value::String(format!("TXN-{:06}", i)),
            serde_json::Value::String(format!("ACC-{:03}", i % 100)),
            serde_json::Value::Number(serde_json::Number::from_f64(
                (i as f64 * 123.45) % 10000.0
            ).unwrap()),
            serde_json::Value::String(if i % 2 == 0 { "CREDIT" } else { "DEBIT" }.to_string()),
            serde_json::Value::String("2024-01-15".to_string()),
            serde_json::Value::String(format!("交易描述 {}", i)),
        ];
        data.push(transaction);
    }
    
    data
}

/// 基准测试：数据库附加操作
fn bench_database_attach(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("ducklake_attach_database", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                // 基准测试附加操作
                let result = manager.attach_database("bench_db", &config).await;
                
                // 忽略错误（可能是扩展未安装）
                black_box(result);
            });
        });
    });
}

/// 基准测试：批量数据插入
fn bench_batch_insert(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("ducklake_batch_insert");
    
    // 测试不同数据量的插入性能
    for size in [100, 1000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("records", size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let mut manager = create_benchmark_manager();
                    let config = create_benchmark_config();
                    
                    // 尝试附加数据库
                    if manager.attach_database("bench_db", &config).await.is_ok() {
                        // 创建测试表
                        let schema = Schema {
                            fields: vec![
                                Field {
                                    name: "transaction_id".to_string(),
                                    data_type: DataType::String,
                                    nullable: false,
                                },
                                Field {
                                    name: "account_id".to_string(),
                                    data_type: DataType::String,
                                    nullable: false,
                                },
                                Field {
                                    name: "amount".to_string(),
                                    data_type: DataType::Float64,
                                    nullable: false,
                                },
                                Field {
                                    name: "transaction_type".to_string(),
                                    data_type: DataType::String,
                                    nullable: false,
                                },
                                Field {
                                    name: "transaction_date".to_string(),
                                    data_type: DataType::String,
                                    nullable: false,
                                },
                                Field {
                                    name: "description".to_string(),
                                    data_type: DataType::String,
                                    nullable: true,
                                },
                            ],
                        };
                        
                        let _ = manager.create_table("bench_db", "transactions", &schema).await;
                        
                        // 生成测试数据
                        let data = generate_financial_data(size);
                        
                        // 基准测试批量插入
                        let result = manager.insert_data("bench_db", "transactions", &data).await;
                        black_box(result);
                    }
                });
            });
        });
    }
    
    group.finish();
}

/// 基准测试：批量操作
fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("ducklake_batch_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let schema = Schema {
                        fields: vec![
                            Field {
                                name: "id".to_string(),
                                data_type: DataType::Int32,
                                nullable: false,
                            },
                            Field {
                                name: "value".to_string(),
                                data_type: DataType::String,
                                nullable: true,
                            },
                        ],
                    };
                    
                    // 创建批量操作
                    let operations = vec![
                        DuckLakeOperation::CreateTable {
                            database: "bench_db".to_string(),
                            table: "batch_test".to_string(),
                            schema: schema.clone(),
                        },
                        DuckLakeOperation::Insert {
                            database: "bench_db".to_string(),
                            table: "batch_test".to_string(),
                            data: generate_financial_data(1000),
                        },
                    ];
                    
                    // 基准测试批量操作
                    let result = manager.batch_operations(operations).await;
                    black_box(result);
                }
            });
        });
    });
}

/// 基准测试：时间旅行查询
fn bench_time_travel_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("ducklake_time_travel");
    
    // 基准测试版本查询
    group.bench_function("query_at_version", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let result = manager.query_at_version(
                        "bench_db",
                        "test_table",
                        1,
                        "SELECT COUNT(*) FROM bench_db.test_table"
                    ).await;
                    black_box(result);
                }
            });
        });
    });
    
    // 基准测试时间戳查询
    group.bench_function("query_at_timestamp", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let timestamp = Utc::now() - chrono::Duration::hours(1);
                    let result = manager.query_at_timestamp(
                        "bench_db",
                        "test_table",
                        timestamp,
                        "SELECT COUNT(*) FROM bench_db.test_table"
                    ).await;
                    black_box(result);
                }
            });
        });
    });
    
    // 基准测试时间范围查询
    group.bench_function("query_time_range", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let start_time = Utc::now() - chrono::Duration::hours(2);
                    let end_time = Utc::now();
                    let result = manager.query_time_range(
                        "bench_db",
                        "test_table",
                        start_time,
                        end_time,
                        "SELECT * FROM bench_db.test_table"
                    ).await;
                    black_box(result);
                }
            });
        });
    });
    
    group.finish();
}

/// 基准测试：Schema演进操作
fn bench_schema_evolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("ducklake_schema_evolution");
    
    // 基准测试添加列
    group.bench_function("add_column", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let result = manager.add_column(
                        "bench_db",
                        "test_table",
                        "new_column",
                        "VARCHAR",
                        Some("'default'"),
                        true
                    ).await;
                    black_box(result);
                }
            });
        });
    });
    
    // 基准测试类型提升
    group.bench_function("alter_column_type", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut manager = create_benchmark_manager();
                let config = create_benchmark_config();
                
                if manager.attach_database("bench_db", &config).await.is_ok() {
                    let result = manager.alter_column_type(
                        "bench_db",
                        "test_table",
                        "some_column",
                        "BIGINT"
                    ).await;
                    black_box(result);
                }
            });
        });
    });
    
    group.finish();
}

/// 基准测试：重试机制性能影响
fn bench_retry_mechanism(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("ducklake_retry_mechanism", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = create_benchmark_manager();
                
                // 测试重试机制的性能开销
                let result = manager.execute_with_retry(|| {
                    // 模拟一个总是成功的操作
                    Ok::<(), DuckHubError>(())
                }).await;
                
                black_box(result);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_database_attach,
    bench_batch_insert,
    bench_batch_operations,
    bench_time_travel_queries,
    bench_schema_evolution,
    bench_retry_mechanism
);

criterion_main!(benches);
