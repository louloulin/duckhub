//! 输入验证和SQL注入防护中间件
//! 提供全面的输入验证、SQL注入检测和XSS防护

use duckhub_common::prelude::*;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn, error, instrument};
use once_cell::sync::Lazy;

/// 输入验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidationConfig {
    /// 是否启用SQL注入检测
    pub enable_sql_injection_detection: bool,
    /// 是否启用XSS检测
    pub enable_xss_detection: bool,
    /// 是否启用路径遍历检测
    pub enable_path_traversal_detection: bool,
    /// 最大请求体大小 (字节)
    pub max_body_size: usize,
    /// 最大字符串长度
    pub max_string_length: usize,
    /// 允许的文件扩展名
    pub allowed_file_extensions: Vec<String>,
    /// 严格模式 (更严格的检测)
    pub strict_mode: bool,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        Self {
            enable_sql_injection_detection: true,
            enable_xss_detection: true,
            enable_path_traversal_detection: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
            max_string_length: 10000,
            allowed_file_extensions: vec![
                "csv".to_string(), "json".to_string(), "txt".to_string(),
                "xlsx".to_string(), "parquet".to_string(),
            ],
            strict_mode: false,
        }
    }
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 是否通过验证
    pub is_valid: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 威胁类型
    pub threat_type: Option<ThreatType>,
    /// 风险级别
    pub risk_level: RiskLevel,
}

/// 威胁类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    /// SQL注入
    SqlInjection,
    /// XSS攻击
    XssAttack,
    /// 路径遍历
    PathTraversal,
    /// 恶意文件上传
    MaliciousFileUpload,
    /// 过大请求
    OversizedRequest,
    /// 无效字符
    InvalidCharacters,
}

/// 风险级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中等风险
    Medium,
    /// 高风险
    High,
    /// 严重风险
    Critical,
}

/// SQL注入检测模式
static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 经典SQL注入模式
        Regex::new(r"(?i)(\bUNION\b.*\bSELECT\b)").unwrap(),
        Regex::new(r"(?i)(\bSELECT\b.*\bFROM\b.*\bWHERE\b.*\bOR\b.*=)").unwrap(),
        Regex::new(r"(?i)(\bINSERT\b.*\bINTO\b.*\bVALUES\b)").unwrap(),
        Regex::new(r"(?i)(\bUPDATE\b.*\bSET\b.*\bWHERE\b)").unwrap(),
        Regex::new(r"(?i)(\bDELETE\b.*\bFROM\b.*\bWHERE\b)").unwrap(),
        Regex::new(r"(?i)(\bDROP\b.*\bTABLE\b)").unwrap(),
        Regex::new(r"(?i)(\bALTER\b.*\bTABLE\b)").unwrap(),
        Regex::new(r"(?i)(\bCREATE\b.*\bTABLE\b)").unwrap(),
        
        // 注释和字符串逃逸
        Regex::new(r"(--|#|/\*|\*/|')").unwrap(),
        Regex::new(r"(?i)(\bOR\b.*\b1\s*=\s*1\b)").unwrap(),
        Regex::new(r"(?i)(\bAND\b.*\b1\s*=\s*1\b)").unwrap(),
        
        // 函数调用
        Regex::new(r"(?i)(\bEXEC\b|\bEXECUTE\b|\bSP_\w+)").unwrap(),
        Regex::new(r"(?i)(\bXP_\w+|\bSYS\w+)").unwrap(),
        
        // 时间延迟攻击
        Regex::new(r"(?i)(\bWAITFOR\b.*\bDELAY\b|\bSLEEP\b\s*\()").unwrap(),
        Regex::new(r"(?i)(\bBENCHMARK\b\s*\()").unwrap(),
    ]
});

/// XSS检测模式
static XSS_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // 脚本标签
        Regex::new(r"(?i)(<script[^>]*>.*?</script>)").unwrap(),
        Regex::new(r"(?i)(<script[^>]*>)").unwrap(),
        
        // 事件处理器
        Regex::new(r"(?i)(on\w+\s*=)").unwrap(),
        
        // JavaScript协议
        Regex::new(r"(?i)(javascript\s*:)").unwrap(),
        
        // 数据协议
        Regex::new(r"(?i)(data\s*:\s*text/html)").unwrap(),
        
        // 其他危险标签
        Regex::new(r"(?i)(<iframe[^>]*>)").unwrap(),
        Regex::new(r"(?i)(<object[^>]*>)").unwrap(),
        Regex::new(r"(?i)(<embed[^>]*>)").unwrap(),
        Regex::new(r"(?i)(<link[^>]*>)").unwrap(),
        Regex::new(r"(?i)(<meta[^>]*>)").unwrap(),
    ]
});

/// 路径遍历检测模式
static PATH_TRAVERSAL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(\.\./|\.\.\\)").unwrap(),
        Regex::new(r"(%2e%2e%2f|%2e%2e%5c)").unwrap(),
        Regex::new(r"(%252e%252e%252f|%252e%252e%255c)").unwrap(),
        Regex::new(r"(\.\.%2f|\.\.%5c)").unwrap(),
    ]
});

/// 输入验证器
pub struct InputValidator {
    config: InputValidationConfig,
}

impl InputValidator {
    /// 创建新的输入验证器
    pub fn new(config: InputValidationConfig) -> Self {
        Self { config }
    }

    /// 验证请求
    #[instrument(skip(self, body))]
    pub async fn validate_request(&self, headers: &HeaderMap, path: &str, body: &[u8]) -> ValidationResult {
        let mut errors = Vec::new();
        let mut threat_type = None;
        let mut risk_level = RiskLevel::Low;

        // 1. 检查请求体大小
        if body.len() > self.config.max_body_size {
            errors.push(format!("请求体过大: {} bytes (最大: {} bytes)", body.len(), self.config.max_body_size));
            threat_type = Some(ThreatType::OversizedRequest);
            risk_level = RiskLevel::Medium;
        }

        // 2. 检查路径遍历
        if self.config.enable_path_traversal_detection {
            if let Some(traversal_result) = self.detect_path_traversal(path) {
                errors.push(traversal_result);
                threat_type = Some(ThreatType::PathTraversal);
                risk_level = RiskLevel::High;
            }
        }

        // 3. 检查请求体内容
        if !body.is_empty() {
            if let Ok(body_str) = std::str::from_utf8(body) {
                // SQL注入检测
                if self.config.enable_sql_injection_detection {
                    if let Some(sql_result) = self.detect_sql_injection(body_str) {
                        errors.push(sql_result);
                        threat_type = Some(ThreatType::SqlInjection);
                        risk_level = RiskLevel::Critical;
                    }
                }

                // XSS检测
                if self.config.enable_xss_detection {
                    if let Some(xss_result) = self.detect_xss(body_str) {
                        errors.push(xss_result);
                        threat_type = Some(ThreatType::XssAttack);
                        risk_level = RiskLevel::High;
                    }
                }

                // 字符串长度检查
                if body_str.len() > self.config.max_string_length {
                    errors.push(format!("字符串过长: {} 字符 (最大: {} 字符)", 
                               body_str.len(), self.config.max_string_length));
                    risk_level = RiskLevel::Medium;
                }

                // 无效字符检查
                if let Some(invalid_char_result) = self.detect_invalid_characters(body_str) {
                    errors.push(invalid_char_result);
                    threat_type = Some(ThreatType::InvalidCharacters);
                    risk_level = RiskLevel::Low;
                }
            }
        }

        // 4. 检查文件上传
        if path.contains("/upload") || path.contains("/file") {
            if let Some(file_result) = self.validate_file_upload(headers, body) {
                errors.push(file_result);
                threat_type = Some(ThreatType::MaliciousFileUpload);
                risk_level = RiskLevel::High;
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            threat_type,
            risk_level,
        }
    }

    /// 检测SQL注入
    fn detect_sql_injection(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();
        
        for (i, pattern) in SQL_INJECTION_PATTERNS.iter().enumerate() {
            if pattern.is_match(&input_lower) {
                return Some(format!("检测到SQL注入模式 #{}: {}", i + 1, 
                           pattern.as_str().chars().take(50).collect::<String>()));
            }
        }

        // 严格模式下的额外检查
        if self.config.strict_mode {
            // 检查多个SQL关键字组合
            let sql_keywords = ["select", "insert", "update", "delete", "drop", "alter", "create"];
            let keyword_count = sql_keywords.iter()
                .filter(|&keyword| input_lower.contains(keyword))
                .count();
            
            if keyword_count >= 2 {
                return Some("检测到多个SQL关键字组合，疑似SQL注入".to_string());
            }
        }

        None
    }

    /// 检测XSS攻击
    fn detect_xss(&self, input: &str) -> Option<String> {
        for (i, pattern) in XSS_PATTERNS.iter().enumerate() {
            if pattern.is_match(input) {
                return Some(format!("检测到XSS攻击模式 #{}: {}", i + 1,
                           pattern.as_str().chars().take(50).collect::<String>()));
            }
        }

        // 检查HTML实体编码的恶意内容
        let decoded = html_escape::decode_html_entities(input);
        if decoded != input {
            for pattern in XSS_PATTERNS.iter() {
                if pattern.is_match(&decoded) {
                    return Some("检测到HTML实体编码的XSS攻击".to_string());
                }
            }
        }

        None
    }

    /// 检测路径遍历
    fn detect_path_traversal(&self, path: &str) -> Option<String> {
        for (i, pattern) in PATH_TRAVERSAL_PATTERNS.iter().enumerate() {
            if pattern.is_match(path) {
                return Some(format!("检测到路径遍历攻击模式 #{}", i + 1));
            }
        }
        None
    }

    /// 检测无效字符
    fn detect_invalid_characters(&self, input: &str) -> Option<String> {
        // 检查控制字符 (除了常见的空白字符)
        for ch in input.chars() {
            if ch.is_control() && !matches!(ch, '\t' | '\n' | '\r') {
                return Some(format!("检测到无效控制字符: U+{:04X}", ch as u32));
            }
        }

        // 检查可疑的Unicode字符
        if self.config.strict_mode {
            for ch in input.chars() {
                // 检查零宽字符
                if matches!(ch, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}') {
                    return Some("检测到可疑的零宽字符".to_string());
                }
            }
        }

        None
    }

    /// 验证文件上传
    fn validate_file_upload(&self, headers: &HeaderMap, body: &[u8]) -> Option<String> {
        // 检查Content-Type
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                // 检查是否为允许的MIME类型
                let allowed_types = [
                    "text/csv", "application/json", "text/plain",
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "application/octet-stream", // for parquet
                ];
                
                if !allowed_types.iter().any(|&t| ct_str.contains(t)) {
                    return Some(format!("不允许的文件类型: {}", ct_str));
                }
            }
        }

        // 检查文件头魔数
        if body.len() >= 4 {
            let magic = &body[0..4];
            
            // 检查可执行文件魔数
            let executable_magics = [
                &[0x4D, 0x5A], // PE executable
                &[0x7F, 0x45, 0x4C, 0x46], // ELF
                &[0xCA, 0xFE, 0xBA, 0xBE], // Java class
                &[0xFE, 0xED, 0xFA], // Mach-O
            ];
            
            for exe_magic in &executable_magics {
                if magic.starts_with(exe_magic) {
                    return Some("检测到可执行文件，禁止上传".to_string());
                }
            }
        }

        None
    }
}

/// 输入验证中间件函数
#[instrument(skip(validator, request, next))]
pub async fn input_validation_middleware(
    State(validator): State<Arc<InputValidator>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers().clone();
    let path = request.uri().path().to_string();
    
    // 读取请求体
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("读取请求体失败: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 执行验证
    let validation_result = validator.validate_request(&headers, &path, &body_bytes).await;

    if !validation_result.is_valid {
        warn!("输入验证失败: path={}, errors={:?}, threat={:?}, risk={:?}", 
              path, validation_result.errors, validation_result.threat_type, validation_result.risk_level);
        
        // 根据风险级别返回不同的状态码
        let status_code = match validation_result.risk_level {
            RiskLevel::Critical => StatusCode::FORBIDDEN,
            RiskLevel::High => StatusCode::BAD_REQUEST,
            RiskLevel::Medium => StatusCode::UNPROCESSABLE_ENTITY,
            RiskLevel::Low => StatusCode::BAD_REQUEST,
        };
        
        return Err(status_code);
    }

    // 重建请求
    let request = Request::from_parts(parts, Body::from(body_bytes));
    
    // 继续处理请求
    let response = next.run(request).await;
    Ok(response)
}
