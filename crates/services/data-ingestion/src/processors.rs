//! 数据处理器模块 - 数据清洗、转换和验证

use crate::config::*;
use crate::sources::DataRecord;
use duckhub_common::prelude::*;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use validator::Validate;

/// 数据处理结果
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// 是否成功
    pub success: bool,
    /// 处理后的数据
    pub data: Option<DataRecord>,
    /// 错误信息
    pub error: Option<String>,
    /// 处理统计
    pub stats: ProcessingStats,
}

/// 处理统计信息
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// 处理开始时间
    pub start_time: DateTime<Utc>,
    /// 处理结束时间
    pub end_time: DateTime<Utc>,
    /// 处理耗时（毫秒）
    pub duration_ms: u64,
    /// 处理的字段数
    pub fields_processed: u32,
    /// 应用的规则数
    pub rules_applied: u32,
}

/// 数据处理器特征
#[async_trait]
pub trait DataProcessor {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 处理器类型
    fn processor_type(&self) -> ProcessorType;

    /// 处理数据记录
    async fn process(&self, record: DataRecord) -> Result<ProcessingResult>;

    /// 验证处理器配置
    fn validate_config(&self) -> Result<()>;

    /// 获取处理器统计信息
    async fn get_stats(&self) -> Result<ProcessorStats>;
}

/// 处理器统计信息
#[derive(Debug, Clone)]
pub struct ProcessorStats {
    /// 处理的记录总数
    pub records_processed: u64,
    /// 成功处理的记录数
    pub records_success: u64,
    /// 失败的记录数
    pub records_failed: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
    /// 最后处理时间
    pub last_processed_at: Option<DateTime<Utc>>,
}

/// 数据清洗处理器
pub struct DataCleaningProcessor {
    name: String,
    config: ProcessorConfig,
    stats: ProcessorStats,
    cleaning_rules: Vec<CleaningRule>,
}

/// 清洗规则
#[derive(Debug, Clone)]
pub enum CleaningRule {
    /// 移除空值
    RemoveNulls,
    /// 去除空白字符
    TrimWhitespace,
    /// 标准化大小写
    NormalizeCase(CaseType),
    /// 移除重复记录
    RemoveDuplicates,
    /// 数据类型转换
    TypeConversion(String, DataType),
}

/// 大小写类型
#[derive(Debug, Clone)]
pub enum CaseType {
    Upper,
    Lower,
    Title,
}

/// 数据类型
#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
}

impl DataCleaningProcessor {
    /// 创建新的数据清洗处理器
    pub fn new(name: String, config: ProcessorConfig) -> Self {
        let cleaning_rules = vec![
            CleaningRule::RemoveNulls,
            CleaningRule::TrimWhitespace,
            CleaningRule::NormalizeCase(CaseType::Lower),
        ];

        Self {
            name,
            config,
            stats: ProcessorStats {
                records_processed: 0,
                records_success: 0,
                records_failed: 0,
                avg_processing_time_ms: 0.0,
                last_processed_at: None,
            },
            cleaning_rules,
        }
    }

    /// 应用清洗规则
    fn apply_cleaning_rules(&self, mut data: Value) -> Value {
        for rule in &self.cleaning_rules {
            data = self.apply_rule(data, rule);
        }
        data
    }

    /// 应用单个清洗规则
    fn apply_rule(&self, data: Value, rule: &CleaningRule) -> Value {
        match rule {
            CleaningRule::RemoveNulls => self.remove_nulls(data),
            CleaningRule::TrimWhitespace => self.trim_whitespace(data),
            CleaningRule::NormalizeCase(case_type) => self.normalize_case(data, case_type),
            _ => data, // 其他规则的实现
        }
    }

    /// 移除空值
    fn remove_nulls(&self, data: Value) -> Value {
        match data {
            Value::Object(mut map) => {
                map.retain(|_, v| !v.is_null());
                Value::Object(map)
            }
            _ => data,
        }
    }

    /// 去除空白字符
    fn trim_whitespace(&self, data: Value) -> Value {
        match data {
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    let new_value = match v {
                        Value::String(s) => Value::String(s.trim().to_string()),
                        _ => v,
                    };
                    new_map.insert(k, new_value);
                }
                Value::Object(new_map)
            }
            Value::String(s) => Value::String(s.trim().to_string()),
            _ => data,
        }
    }

    /// 标准化大小写
    fn normalize_case(&self, data: Value, case_type: &CaseType) -> Value {
        match data {
            Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    let new_value = match v {
                        Value::String(s) => {
                            let normalized = match case_type {
                                CaseType::Upper => s.to_uppercase(),
                                CaseType::Lower => s.to_lowercase(),
                                CaseType::Title => s.chars()
                                    .enumerate()
                                    .map(|(i, c)| if i == 0 { c.to_uppercase().collect::<String>() } else { c.to_lowercase().collect::<String>() })
                                    .collect::<String>(),
                            };
                            Value::String(normalized)
                        }
                        _ => v,
                    };
                    new_map.insert(k, new_value);
                }
                Value::Object(new_map)
            }
            Value::String(s) => {
                let normalized = match case_type {
                    CaseType::Upper => s.to_uppercase(),
                    CaseType::Lower => s.to_lowercase(),
                    CaseType::Title => s.chars()
                        .enumerate()
                        .map(|(i, c)| if i == 0 { c.to_uppercase().collect::<String>() } else { c.to_lowercase().collect::<String>() })
                        .collect::<String>(),
                };
                Value::String(normalized)
            }
            _ => data,
        }
    }
}

#[async_trait]
impl DataProcessor for DataCleaningProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::DataCleaning
    }

    async fn process(&self, mut record: DataRecord) -> Result<ProcessingResult> {
        let start_time = Utc::now();
        
        // 应用清洗规则
        let cleaned_data = self.apply_cleaning_rules(record.data.clone());
        record.data = cleaned_data;
        
        // 添加处理元数据
        record.metadata.insert("processed_by".to_string(), self.name.clone());
        record.metadata.insert("processor_type".to_string(), "data_cleaning".to_string());
        
        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;

        Ok(ProcessingResult {
            success: true,
            data: Some(record),
            error: None,
            stats: ProcessingStats {
                start_time,
                end_time,
                duration_ms,
                fields_processed: 1, // 简化统计
                rules_applied: self.cleaning_rules.len() as u32,
            },
        })
    }

    fn validate_config(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(DuckHubError::validation("处理器名称不能为空"));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<ProcessorStats> {
        Ok(self.stats.clone())
    }
}

/// 数据验证处理器
pub struct DataValidationProcessor {
    name: String,
    config: ProcessorConfig,
    stats: ProcessorStats,
    validation_rules: Vec<ValidationRule>,
}

/// 验证规则
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// 必填字段检查
    RequiredField(String),
    /// 数据类型检查
    TypeCheck(String, DataType),
    /// 范围检查
    RangeCheck(String, f64, f64),
    /// 正则表达式检查
    RegexCheck(String, String),
    /// 自定义验证函数
    CustomValidation(String),
}

impl DataValidationProcessor {
    /// 创建新的数据验证处理器
    pub fn new(name: String, config: ProcessorConfig) -> Self {
        let validation_rules = vec![
            ValidationRule::RequiredField("id".to_string()),
            ValidationRule::RequiredField("timestamp".to_string()),
        ];

        Self {
            name,
            config,
            stats: ProcessorStats {
                records_processed: 0,
                records_success: 0,
                records_failed: 0,
                avg_processing_time_ms: 0.0,
                last_processed_at: None,
            },
            validation_rules,
        }
    }

    /// 验证数据记录
    fn validate_record(&self, record: &DataRecord) -> Result<()> {
        for rule in &self.validation_rules {
            self.apply_validation_rule(record, rule)?;
        }
        Ok(())
    }

    /// 应用验证规则
    fn apply_validation_rule(&self, record: &DataRecord, rule: &ValidationRule) -> Result<()> {
        match rule {
            ValidationRule::RequiredField(field) => {
                if !record.data.get(field).is_some() {
                    return Err(DuckHubError::validation(format!("必填字段'{}'缺失", field)));
                }
            }
            ValidationRule::TypeCheck(field, expected_type) => {
                if let Some(value) = record.data.get(field) {
                    if !self.check_type(value, expected_type) {
                        return Err(DuckHubError::validation(format!("字段'{}'类型不匹配", field)));
                    }
                }
            }
            ValidationRule::RangeCheck(field, min, max) => {
                if let Some(value) = record.data.get(field) {
                    if let Some(num) = value.as_f64() {
                        if num < *min || num > *max {
                            return Err(DuckHubError::validation(format!("字段'{}'值超出范围", field)));
                        }
                    }
                }
            }
            _ => {} // 其他验证规则的实现
        }
        Ok(())
    }

    /// 检查数据类型
    fn check_type(&self, value: &Value, expected_type: &DataType) -> bool {
        match expected_type {
            DataType::String => value.is_string(),
            DataType::Integer => value.is_i64(),
            DataType::Float => value.is_f64(),
            DataType::Boolean => value.is_boolean(),
            DataType::DateTime => {
                // 简化的日期时间检查
                value.is_string() && value.as_str().unwrap_or("").contains("T")
            }
        }
    }
}

#[async_trait]
impl DataProcessor for DataValidationProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::DataValidation
    }

    async fn process(&self, mut record: DataRecord) -> Result<ProcessingResult> {
        let start_time = Utc::now();
        
        // 验证数据记录
        match self.validate_record(&record) {
            Ok(_) => {
                // 添加验证元数据
                record.metadata.insert("validated_by".to_string(), self.name.clone());
                record.metadata.insert("validation_status".to_string(), "passed".to_string());
                
                let end_time = Utc::now();
                let duration_ms = (end_time - start_time).num_milliseconds() as u64;

                Ok(ProcessingResult {
                    success: true,
                    data: Some(record),
                    error: None,
                    stats: ProcessingStats {
                        start_time,
                        end_time,
                        duration_ms,
                        fields_processed: 1,
                        rules_applied: self.validation_rules.len() as u32,
                    },
                })
            }
            Err(e) => {
                let end_time = Utc::now();
                let duration_ms = (end_time - start_time).num_milliseconds() as u64;

                Ok(ProcessingResult {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    stats: ProcessingStats {
                        start_time,
                        end_time,
                        duration_ms,
                        fields_processed: 0,
                        rules_applied: self.validation_rules.len() as u32,
                    },
                })
            }
        }
    }

    fn validate_config(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(DuckHubError::validation("处理器名称不能为空"));
        }
        Ok(())
    }

    async fn get_stats(&self) -> Result<ProcessorStats> {
        Ok(self.stats.clone())
    }
}
