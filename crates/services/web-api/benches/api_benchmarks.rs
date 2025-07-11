//! Web API性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use duckhub_web_api::{WebApiConfig, utils::*};
use std::time::Duration;

/// 测试数字格式化性能
fn bench_format_number(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_number");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| format_number(black_box(size)));
        });
    }
    
    group.finish();
}

/// 测试字节格式化性能
fn bench_format_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("format_bytes");
    
    for size in [1024, 1024*1024, 1024*1024*1024].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| format_bytes(black_box(size)));
        });
    }
    
    group.finish();
}

/// 测试SQL清理性能
fn bench_sanitize_sql(c: &mut Criterion) {
    let sql_samples = vec![
        "SELECT * FROM users WHERE id = 1",
        "SELECT u.*, COUNT(*) FROM users u -- comment\nJOIN orders o ON u.id = o.user_id GROUP BY u.id",
        "SELECT /* multi\nline\ncomment */ * FROM complex_table WHERE condition = 'value'",
        "WITH cte AS (SELECT * FROM table1) SELECT * FROM cte JOIN table2 ON cte.id = table2.ref_id",
    ];
    
    let mut group = c.benchmark_group("sanitize_sql");
    
    for (i, sql) in sql_samples.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), sql, |b, sql| {
            b.iter(|| sanitize_sql(black_box(sql)));
        });
    }
    
    group.finish();
}

/// 测试查询复杂度计算性能
fn bench_calculate_query_complexity(c: &mut Criterion) {
    let queries = vec![
        "SELECT * FROM users",
        "SELECT u.*, COUNT(*) FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.id",
        "WITH RECURSIVE cte AS (SELECT id, parent_id FROM categories WHERE parent_id IS NULL UNION ALL SELECT c.id, c.parent_id FROM categories c JOIN cte ON c.parent_id = cte.id) SELECT * FROM cte",
        "SELECT user_id, SUM(amount) OVER (PARTITION BY user_id ORDER BY created_at ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_total FROM transactions",
    ];
    
    let mut group = c.benchmark_group("calculate_query_complexity");
    
    for (i, query) in queries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), query, |b, query| {
            b.iter(|| calculate_query_complexity(black_box(query)));
        });
    }
    
    group.finish();
}

/// 测试表名提取性能
fn bench_extract_table_names(c: &mut Criterion) {
    let queries = vec![
        "SELECT * FROM users",
        "SELECT * FROM users u JOIN orders o ON u.id = o.user_id",
        "INSERT INTO logs SELECT * FROM temp_logs WHERE created_at > '2023-01-01'",
        "UPDATE users SET last_login = NOW() WHERE id IN (SELECT user_id FROM sessions WHERE active = true)",
    ];
    
    let mut group = c.benchmark_group("extract_table_names");
    
    for (i, query) in queries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), query, |b, query| {
            b.iter(|| extract_table_names(black_box(query)));
        });
    }
    
    group.finish();
}

/// 测试缓存键生成性能
fn bench_generate_cache_key(c: &mut Criterion) {
    let params_sets = vec![
        vec!["user", "123"],
        vec!["query", "SELECT * FROM users", "limit=10"],
        vec!["complex", "param1", "param2", "param3", "param4", "param5"],
    ];
    
    let mut group = c.benchmark_group("generate_cache_key");
    
    for (i, params) in params_sets.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), params, |b, params| {
            b.iter(|| generate_cache_key(black_box("prefix"), black_box(params)));
        });
    }
    
    group.finish();
}

/// 测试密码验证性能
fn bench_validate_password_strength(c: &mut Criterion) {
    let passwords = vec![
        "weak",
        "StrongPassword123!",
        "VeryLongPasswordWithManyCharacters123!@#",
        "P@ssw0rd",
    ];
    
    let mut group = c.benchmark_group("validate_password_strength");
    
    for (i, password) in passwords.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), password, |b, password| {
            b.iter(|| validate_password_strength(black_box(password)));
        });
    }
    
    group.finish();
}

/// 测试邮箱验证性能
fn bench_is_valid_email(c: &mut Criterion) {
    let emails = vec![
        "test@example.com",
        "user.name+tag@domain.co.uk",
        "invalid.email",
        "very.long.email.address.with.many.dots@very.long.domain.name.com",
    ];
    
    let mut group = c.benchmark_group("is_valid_email");
    
    for (i, email) in emails.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), email, |b, email| {
            b.iter(|| is_valid_email(black_box(email)));
        });
    }
    
    group.finish();
}

/// 测试配置验证性能
fn bench_config_validation(c: &mut Criterion) {
    c.bench_function("config_validation", |b| {
        let config = WebApiConfig::default();
        b.iter(|| config.validate());
    });
}

/// 测试持续时间格式化性能
fn bench_format_duration(c: &mut Criterion) {
    let durations = vec![
        Duration::from_secs(30),
        Duration::from_secs(90),
        Duration::from_secs(3661),
        Duration::from_secs(86461),
    ];
    
    let mut group = c.benchmark_group("format_duration");
    
    for (i, duration) in durations.iter().enumerate() {
        group.bench_with_input(BenchmarkId::from_parameter(i), duration, |b, duration| {
            b.iter(|| format_duration(black_box(*duration)));
        });
    }
    
    group.finish();
}

/// 综合性能测试
fn bench_comprehensive_workflow(c: &mut Criterion) {
    c.bench_function("comprehensive_workflow", |b| {
        b.iter(|| {
            let sql = black_box("SELECT u.*, COUNT(*) FROM users u -- comment\nJOIN orders o ON u.id = o.user_id GROUP BY u.id");
            let sanitized = sanitize_sql(sql);
            let complexity = calculate_query_complexity(&sanitized);
            let tables = extract_table_names(&sanitized);
            let cache_key = generate_cache_key("query", &[&sanitized]);
            
            // 模拟一些计算
            (sanitized, complexity, tables, cache_key)
        });
    });
}

criterion_group!(
    benches,
    bench_format_number,
    bench_format_bytes,
    bench_sanitize_sql,
    bench_calculate_query_complexity,
    bench_extract_table_names,
    bench_generate_cache_key,
    bench_validate_password_strength,
    bench_is_valid_email,
    bench_config_validation,
    bench_format_duration,
    bench_comprehensive_workflow
);

criterion_main!(benches);

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_format_number_performance() {
        let start = Instant::now();
        for _ in 0..1000 {
            format_number(1234567890);
        }
        let duration = start.elapsed();
        
        // 1000次格式化应该在10ms内完成
        assert!(duration.as_millis() < 10);
    }

    #[test]
    fn test_sql_sanitization_performance() {
        let sql = "SELECT u.*, COUNT(*) FROM users u -- comment\nJOIN orders o ON u.id = o.user_id GROUP BY u.id";
        let start = Instant::now();
        
        for _ in 0..100 {
            sanitize_sql(sql);
        }
        
        let duration = start.elapsed();
        
        // 100次SQL清理应该在5ms内完成
        assert!(duration.as_millis() < 5);
    }

    #[test]
    fn test_query_complexity_performance() {
        let query = "WITH RECURSIVE cte AS (SELECT id, parent_id FROM categories WHERE parent_id IS NULL UNION ALL SELECT c.id, c.parent_id FROM categories c JOIN cte ON c.parent_id = cte.id) SELECT * FROM cte";
        let start = Instant::now();
        
        for _ in 0..1000 {
            calculate_query_complexity(query);
        }
        
        let duration = start.elapsed();
        
        // 1000次复杂度计算应该在10ms内完成
        assert!(duration.as_millis() < 10);
    }

    #[test]
    fn test_email_validation_performance() {
        let email = "very.long.email.address.with.many.dots@very.long.domain.name.com";
        let start = Instant::now();
        
        for _ in 0..1000 {
            is_valid_email(email);
        }
        
        let duration = start.elapsed();
        
        // 1000次邮箱验证应该在50ms内完成
        assert!(duration.as_millis() < 50);
    }
}
