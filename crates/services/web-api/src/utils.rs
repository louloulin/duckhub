//! 工具函数

use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 格式化数字，添加千分位分隔符
pub fn format_number(num: u64) -> String {
    let num_str = num.to_string();
    let chars: Vec<char> = num_str.chars().collect();
    let mut result = String::new();
    
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    
    result
}

/// 格式化字节大小
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 格式化持续时间
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    
    if total_seconds < 60 {
        format!("{}秒", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        if seconds == 0 {
            format!("{}分钟", minutes)
        } else {
            format!("{}分{}秒", minutes, seconds)
        }
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        if minutes == 0 {
            format!("{}小时", hours)
        } else {
            format!("{}小时{}分钟", hours, minutes)
        }
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        if hours == 0 {
            format!("{}天", days)
        } else {
            format!("{}天{}小时", days, hours)
        }
    }
}

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 验证邮箱格式
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    email_regex.is_match(email)
}

/// 验证密码强度
pub fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("密码长度至少8位".to_string());
    }
    
    if password.len() > 128 {
        return Err("密码长度不能超过128位".to_string());
    }
    
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
    
    let mut strength_score = 0;
    if has_lowercase { strength_score += 1; }
    if has_uppercase { strength_score += 1; }
    if has_digit { strength_score += 1; }
    if has_special { strength_score += 1; }
    
    if strength_score < 3 {
        return Err("密码必须包含大写字母、小写字母、数字和特殊字符中的至少3种".to_string());
    }
    
    Ok(())
}

/// 清理SQL查询（移除注释和多余空格）
pub fn sanitize_sql(sql: &str) -> String {
    // 移除单行注释
    let mut result = String::new();
    let mut in_string = false;
    let mut string_char = '\0';
    let mut chars = sql.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '\'' | '"' => {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                result.push(ch);
            }
            '-' if !in_string => {
                if let Some(&'-') = chars.peek() {
                    // 跳过单行注释
                    chars.next(); // 跳过第二个 '-'
                    while let Some(ch) = chars.next() {
                        if ch == '\n' {
                            result.push(ch);
                            break;
                        }
                    }
                } else {
                    result.push(ch);
                }
            }
            '/' if !in_string => {
                if let Some(&'*') = chars.peek() {
                    // 跳过多行注释
                    chars.next(); // 跳过 '*'
                    let mut found_end = false;
                    while let Some(ch) = chars.next() {
                        if ch == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next(); // 跳过 '/'
                                found_end = true;
                                break;
                            }
                        }
                    }
                    if !found_end {
                        // 注释没有正确结束，这可能是一个错误
                        result.push('/');
                        result.push('*');
                    }
                } else {
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }
    
    // 清理多余的空格
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 计算SQL查询的复杂度评分
pub fn calculate_query_complexity(sql: &str) -> u8 {
    let sql_upper = sql.to_uppercase();
    let mut complexity = 1;
    
    // 基础关键字
    if sql_upper.contains("SELECT") { complexity += 1; }
    if sql_upper.contains("FROM") { complexity += 1; }
    
    // 连接操作
    let join_count = sql_upper.matches("JOIN").count();
    complexity += join_count * 2;
    
    // 子查询
    let subquery_count = sql_upper.matches("(SELECT").count();
    complexity += subquery_count * 3;
    
    // 聚合函数
    let agg_functions = ["COUNT", "SUM", "AVG", "MAX", "MIN", "GROUP_CONCAT"];
    for func in &agg_functions {
        complexity += sql_upper.matches(func).count();
    }
    
    // 窗口函数
    if sql_upper.contains("OVER") { complexity += 3; }
    
    // 排序和分组
    if sql_upper.contains("ORDER BY") { complexity += 1; }
    if sql_upper.contains("GROUP BY") { complexity += 2; }
    if sql_upper.contains("HAVING") { complexity += 2; }
    
    // CTE (Common Table Expressions)
    if sql_upper.contains("WITH") { complexity += 2; }
    
    // 限制复杂度评分在1-10之间
    std::cmp::min(complexity, 10) as u8
}

/// 提取SQL查询中的表名
pub fn extract_table_names(sql: &str) -> Vec<String> {
    let sql_upper = sql.to_uppercase();
    let mut tables = Vec::new();
    
    // 简单的表名提取（实际应用中可能需要更复杂的SQL解析）
    let words: Vec<&str> = sql_upper.split_whitespace().collect();
    let mut i = 0;
    
    while i < words.len() {
        if words[i] == "FROM" || words[i] == "JOIN" || words[i] == "UPDATE" || words[i] == "INTO" {
            if i + 1 < words.len() {
                let table_name = words[i + 1].trim_end_matches(',').trim_end_matches(';');
                if !table_name.is_empty() && !table_name.starts_with('(') {
                    tables.push(table_name.to_lowercase());
                }
            }
        }
        i += 1;
    }
    
    tables.sort();
    tables.dedup();
    tables
}

/// 生成缓存键
pub fn generate_cache_key(prefix: &str, params: &[&str]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    
    prefix.hash(&mut hasher);
    for param in params {
        param.hash(&mut hasher);
    }
    
    format!("{}:{:x}", prefix, hasher.finish())
}

/// 转换查询参数
pub fn convert_query_params(params: &HashMap<String, Value>) -> HashMap<String, String> {
    params.iter().map(|(k, v)| {
        let value_str = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => serde_json::to_string(v).unwrap_or_default(),
        };
        (k.clone(), value_str)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@domain.com"));
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(validate_password_strength("Password123!").is_ok());
        assert!(validate_password_strength("weak").is_err());
        assert!(validate_password_strength("NoNumbers!").is_err());
    }

    #[test]
    fn test_sanitize_sql() {
        let sql = "SELECT * FROM users -- this is a comment\nWHERE id = 1";
        let sanitized = sanitize_sql(sql);
        assert_eq!(sanitized, "SELECT * FROM users WHERE id = 1");
    }

    #[test]
    fn test_calculate_query_complexity() {
        let simple_query = "SELECT * FROM users";
        assert_eq!(calculate_query_complexity(simple_query), 3);
        
        let complex_query = "SELECT u.*, COUNT(*) FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.id";
        assert!(calculate_query_complexity(complex_query) > 5);
    }

    #[test]
    fn test_extract_table_names() {
        let sql = "SELECT * FROM users u JOIN orders o ON u.id = o.user_id";
        let tables = extract_table_names(sql);
        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"orders".to_string()));
    }
}
