//! 智能推荐引擎模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, instrument};
use super::AIAgentConfig;

/// 推荐引擎
pub struct RecommendationEngine {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
    /// 配置
    config: AIAgentConfig,
    /// 推荐规则
    rules: Vec<RecommendationRule>,
}

/// 推荐上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContext {
    /// 用户ID
    pub user_id: Option<String>,
    /// 当前查询
    pub current_query: Option<String>,
    /// 查询历史
    pub query_history: Vec<String>,
    /// 数据表信息
    pub table_info: HashMap<String, TableInfo>,
    /// 业务领域
    pub business_domain: Option<String>,
    /// 时间范围
    pub time_range: Option<TimeRange>,
}

/// 表信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    /// 表名
    pub table_name: String,
    /// 行数
    pub row_count: u64,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// 列名
    pub column_name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否为空
    pub nullable: bool,
    /// 唯一值数量
    pub distinct_count: Option<u64>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
}

/// 推荐
#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    /// 推荐ID
    pub id: String,
    /// 推荐类型
    pub recommendation_type: RecommendationType,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// SQL查询（如果适用）
    pub sql_query: Option<String>,
    /// 优先级
    pub priority: Priority,
    /// 置信度
    pub confidence: f32,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 推荐类型
#[derive(Debug, Clone, Serialize)]
pub enum RecommendationType {
    /// 查询优化
    QueryOptimization,
    /// 数据探索
    DataExploration,
    /// 异常检测
    AnomalyDetection,
    /// 趋势分析
    TrendAnalysis,
    /// 性能优化
    PerformanceOptimization,
    /// 数据质量
    DataQuality,
    /// 业务洞察
    BusinessInsight,
}

/// 优先级
#[derive(Debug, Clone, Serialize)]
pub enum Priority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 紧急
    Critical,
}

/// 推荐规则
#[derive(Debug, Clone)]
pub struct RecommendationRule {
    /// 规则名称
    pub name: String,
    /// 规则类型
    pub rule_type: RecommendationType,
    /// 触发条件
    pub trigger_condition: TriggerCondition,
    /// 推荐模板
    pub template: RecommendationTemplate,
    /// 优先级
    pub priority: Priority,
    /// 是否启用
    pub enabled: bool,
}

/// 触发条件
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    /// 查询包含特定关键词
    QueryContains(Vec<String>),
    /// 表行数超过阈值
    TableRowCountExceeds(u64),
    /// 查询执行时间超过阈值
    QueryTimeExceeds(u64),
    /// 数据更新频率
    DataUpdateFrequency(String),
    /// 总是触发
    Always,
}

/// 推荐模板
#[derive(Debug, Clone)]
pub struct RecommendationTemplate {
    /// 标题模板
    pub title: String,
    /// 描述模板
    pub description: String,
    /// SQL模板
    pub sql_template: Option<String>,
    /// 标签
    pub tags: Vec<String>,
}

impl RecommendationEngine {
    /// 创建新的推荐引擎
    #[instrument(skip(engine, config))]
    pub async fn new(engine: Arc<DuckDBEngine>, config: &AIAgentConfig) -> Result<Self> {
        let rules = Self::create_default_rules();
        
        let engine_instance = Self {
            engine,
            config: config.clone(),
            rules,
        };

        debug!("推荐引擎初始化完成");
        Ok(engine_instance)
    }

    /// 创建默认推荐规则
    fn create_default_rules() -> Vec<RecommendationRule> {
        vec![
            // 查询优化推荐
            RecommendationRule {
                name: "select_star_optimization".to_string(),
                rule_type: RecommendationType::QueryOptimization,
                trigger_condition: TriggerCondition::QueryContains(vec!["SELECT *".to_string()]),
                template: RecommendationTemplate {
                    title: "查询优化建议".to_string(),
                    description: "建议明确指定需要的列名，避免使用SELECT *，这样可以提高查询性能并减少网络传输。".to_string(),
                    sql_template: Some("SELECT column1, column2, column3 FROM {table}".to_string()),
                    tags: vec!["性能优化".to_string(), "最佳实践".to_string()],
                },
                priority: Priority::Medium,
                enabled: true,
            },
            
            // 数据探索推荐
            RecommendationRule {
                name: "data_exploration".to_string(),
                rule_type: RecommendationType::DataExploration,
                trigger_condition: TriggerCondition::Always,
                template: RecommendationTemplate {
                    title: "数据探索建议".to_string(),
                    description: "建议先了解数据的基本统计信息，包括行数、列数、数据类型等。".to_string(),
                    sql_template: Some("SELECT COUNT(*) as row_count, COUNT(DISTINCT {column}) as distinct_values FROM {table}".to_string()),
                    tags: vec!["数据探索".to_string(), "统计分析".to_string()],
                },
                priority: Priority::Low,
                enabled: true,
            },
            
            // 异常检测推荐
            RecommendationRule {
                name: "anomaly_detection".to_string(),
                rule_type: RecommendationType::AnomalyDetection,
                trigger_condition: TriggerCondition::QueryContains(vec!["异常".to_string(), "异常值".to_string()]),
                template: RecommendationTemplate {
                    title: "异常检测分析".to_string(),
                    description: "建议使用统计方法检测数据中的异常值，如Z-score或IQR方法。".to_string(),
                    sql_template: Some("SELECT * FROM {table} WHERE ABS({column} - (SELECT AVG({column}) FROM {table})) > 2 * (SELECT STDDEV({column}) FROM {table})".to_string()),
                    tags: vec!["异常检测".to_string(), "数据质量".to_string()],
                },
                priority: Priority::High,
                enabled: true,
            },
            
            // 趋势分析推荐
            RecommendationRule {
                name: "trend_analysis".to_string(),
                rule_type: RecommendationType::TrendAnalysis,
                trigger_condition: TriggerCondition::QueryContains(vec!["趋势".to_string(), "变化".to_string(), "时间".to_string()]),
                template: RecommendationTemplate {
                    title: "趋势分析建议".to_string(),
                    description: "建议使用时间序列分析来识别数据的趋势和季节性模式。".to_string(),
                    sql_template: Some("SELECT DATE_TRUNC('month', {time_column}) as month, AVG({value_column}) as avg_value FROM {table} GROUP BY month ORDER BY month".to_string()),
                    tags: vec!["趋势分析".to_string(), "时间序列".to_string()],
                },
                priority: Priority::Medium,
                enabled: true,
            },
            
            // 性能优化推荐
            RecommendationRule {
                name: "performance_optimization".to_string(),
                rule_type: RecommendationType::PerformanceOptimization,
                trigger_condition: TriggerCondition::QueryTimeExceeds(1000), // 1秒
                template: RecommendationTemplate {
                    title: "性能优化建议".to_string(),
                    description: "查询执行时间较长，建议添加适当的索引或优化查询条件。".to_string(),
                    sql_template: Some("CREATE INDEX idx_{table}_{column} ON {table}({column})".to_string()),
                    tags: vec!["性能优化".to_string(), "索引".to_string()],
                },
                priority: Priority::High,
                enabled: true,
            },
            
            // 数据质量推荐
            RecommendationRule {
                name: "data_quality_check".to_string(),
                rule_type: RecommendationType::DataQuality,
                trigger_condition: TriggerCondition::Always,
                template: RecommendationTemplate {
                    title: "数据质量检查".to_string(),
                    description: "建议定期检查数据的完整性，包括空值、重复值等。".to_string(),
                    sql_template: Some("SELECT COUNT(*) as total_rows, COUNT(*) - COUNT({column}) as null_count FROM {table}".to_string()),
                    tags: vec!["数据质量".to_string(), "数据清洗".to_string()],
                },
                priority: Priority::Medium,
                enabled: true,
            },
        ]
    }

    /// 生成推荐
    #[instrument(skip(self, context))]
    pub async fn generate_recommendations(&self, context: &RecommendationContext) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            
            if self.should_trigger_rule(rule, context).await? {
                let recommendation = self.create_recommendation_from_rule(rule, context).await?;
                recommendations.push(recommendation);
            }
        }
        
        // 按优先级排序
        recommendations.sort_by(|a, b| {
            match (&a.priority, &b.priority) {
                (Priority::Critical, _) => std::cmp::Ordering::Less,
                (_, Priority::Critical) => std::cmp::Ordering::Greater,
                (Priority::High, Priority::Low | Priority::Medium) => std::cmp::Ordering::Less,
                (Priority::Low | Priority::Medium, Priority::High) => std::cmp::Ordering::Greater,
                (Priority::Medium, Priority::Low) => std::cmp::Ordering::Less,
                (Priority::Low, Priority::Medium) => std::cmp::Ordering::Greater,
                _ => a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal).reverse(),
            }
        });
        
        debug!("生成了{}条推荐", recommendations.len());
        Ok(recommendations)
    }

    /// 判断是否应该触发规则
    async fn should_trigger_rule(&self, rule: &RecommendationRule, context: &RecommendationContext) -> Result<bool> {
        match &rule.trigger_condition {
            TriggerCondition::QueryContains(keywords) => {
                if let Some(query) = &context.current_query {
                    for keyword in keywords {
                        if query.to_uppercase().contains(&keyword.to_uppercase()) {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            }
            TriggerCondition::TableRowCountExceeds(threshold) => {
                for table_info in context.table_info.values() {
                    if table_info.row_count > *threshold {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TriggerCondition::QueryTimeExceeds(_threshold) => {
                // 这里需要查询执行时间信息，简化处理
                Ok(false)
            }
            TriggerCondition::DataUpdateFrequency(_frequency) => {
                // 这里需要检查数据更新频率，简化处理
                Ok(false)
            }
            TriggerCondition::Always => Ok(true),
        }
    }

    /// 从规则创建推荐
    async fn create_recommendation_from_rule(&self, rule: &RecommendationRule, context: &RecommendationContext) -> Result<Recommendation> {
        let mut title = rule.template.title.clone();
        let mut description = rule.template.description.clone();
        let mut sql_query = rule.template.sql_template.clone();
        
        // 替换模板变量
        if let Some(table_name) = context.table_info.keys().next() {
            title = title.replace("{table}", table_name);
            description = description.replace("{table}", table_name);
            if let Some(ref sql) = sql_query {
                sql_query = Some(sql.replace("{table}", table_name));
                
                // 替换列名
                if let Some(table_info) = context.table_info.get(table_name) {
                    if let Some(first_column) = table_info.columns.first() {
                        sql_query = sql_query.map(|s| s.replace("{column}", &first_column.column_name));
                    }
                }
            }
        }
        
        // 计算置信度
        let confidence = self.calculate_recommendation_confidence(rule, context);
        
        Ok(Recommendation {
            id: uuid::Uuid::new_v4().to_string(),
            recommendation_type: rule.rule_type.clone(),
            title,
            description,
            sql_query,
            priority: rule.priority.clone(),
            confidence,
            tags: rule.template.tags.clone(),
            created_at: Utc::now(),
        })
    }

    /// 计算推荐置信度
    fn calculate_recommendation_confidence(&self, rule: &RecommendationRule, context: &RecommendationContext) -> f32 {
        let mut confidence: f32 = 0.7; // 基础置信度
        
        // 根据查询历史调整置信度
        if !context.query_history.is_empty() {
            confidence += 0.1;
        }
        
        // 根据表信息调整置信度
        if !context.table_info.is_empty() {
            confidence += 0.1;
        }
        
        // 根据业务领域调整置信度
        if context.business_domain.is_some() {
            confidence += 0.1;
        }
        
        confidence.min(1.0)
    }

    /// 获取个性化推荐
    pub async fn get_personalized_recommendations(&self, user_id: &str, limit: usize) -> Result<Vec<Recommendation>> {
        // 简化实现：基于用户历史生成个性化推荐
        let context = RecommendationContext {
            user_id: Some(user_id.to_string()),
            current_query: None,
            query_history: Vec::new(),
            table_info: HashMap::new(),
            business_domain: Some("finance".to_string()),
            time_range: None,
        };
        
        let mut recommendations = self.generate_recommendations(&context).await?;
        recommendations.truncate(limit);
        
        Ok(recommendations)
    }

    /// 获取基于内容的推荐
    pub async fn get_content_based_recommendations(&self, query: &str) -> Result<Vec<Recommendation>> {
        let context = RecommendationContext {
            user_id: None,
            current_query: Some(query.to_string()),
            query_history: Vec::new(),
            table_info: HashMap::new(),
            business_domain: None,
            time_range: None,
        };
        
        self.generate_recommendations(&context).await
    }
}
