//! 自然语言处理模块

use duckhub_common::prelude::*;
use duckhub_query_analytics::QueryResult;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use regex::Regex;
use super::AIAgentConfig;

/// NLP处理器
pub struct NLPProcessor {
    /// 配置
    config: AIAgentConfig,
    /// 查询模式
    query_patterns: Vec<QueryPattern>,
    /// 表名映射
    table_mappings: HashMap<String, String>,
}

/// 查询模式
#[derive(Debug, Clone)]
pub struct QueryPattern {
    /// 模式名称
    pub name: String,
    /// 正则表达式
    pub regex: Regex,
    /// SQL模板
    pub sql_template: String,
    /// 置信度
    pub confidence: f32,
}

/// 解析后的查询
#[derive(Debug, Clone, Serialize)]
pub struct ParsedQuery {
    /// 查询类型
    pub query_type: QueryType,
    /// 表名
    pub table_name: Option<String>,
    /// 列名
    pub columns: Vec<String>,
    /// 条件
    pub conditions: Vec<Condition>,
    /// 聚合函数
    pub aggregations: Vec<Aggregation>,
    /// 排序
    pub order_by: Option<OrderBy>,
    /// 限制
    pub limit: Option<u32>,
    /// 置信度
    pub confidence: f32,
    /// 原始查询
    pub original_query: String,
}

/// 查询类型
#[derive(Debug, Clone, Serialize)]
pub enum QueryType {
    /// 选择查询
    Select,
    /// 聚合查询
    Aggregate,
    /// 统计查询
    Statistics,
    /// 趋势分析
    Trend,
    /// 比较查询
    Comparison,
    /// 未知
    Unknown,
}

/// 条件
#[derive(Debug, Clone, Serialize)]
pub struct Condition {
    /// 列名
    pub column: String,
    /// 操作符
    pub operator: String,
    /// 值
    pub value: String,
}

/// 聚合函数
#[derive(Debug, Clone, Serialize)]
pub struct Aggregation {
    /// 函数名
    pub function: String,
    /// 列名
    pub column: String,
    /// 别名
    pub alias: Option<String>,
}

/// 排序
#[derive(Debug, Clone, Serialize)]
pub struct OrderBy {
    /// 列名
    pub column: String,
    /// 方向
    pub direction: String,
}

impl NLPProcessor {
    /// 创建新的NLP处理器
    #[instrument(skip(config))]
    pub async fn new(config: &AIAgentConfig) -> Result<Self> {
        let query_patterns = Self::create_query_patterns()?;
        let table_mappings = Self::create_table_mappings();
        
        let processor = Self {
            config: config.clone(),
            query_patterns,
            table_mappings,
        };

        debug!("NLP处理器初始化完成");
        Ok(processor)
    }

    /// 创建查询模式
    fn create_query_patterns() -> Result<Vec<QueryPattern>> {
        let mut patterns = Vec::new();

        // 简单选择查询
        patterns.push(QueryPattern {
            name: "simple_select".to_string(),
            regex: Regex::new(r"(?i)显示|查看|获取|列出.*?(\w+)表.*?的.*?(\w+)")?,
            sql_template: "SELECT {columns} FROM {table}".to_string(),
            confidence: 0.8,
        });

        // 计数查询
        patterns.push(QueryPattern {
            name: "count_query".to_string(),
            regex: Regex::new(r"(?i)有多少|数量|总数.*?(\w+)")?,
            sql_template: "SELECT COUNT(*) as count FROM {table}".to_string(),
            confidence: 0.9,
        });

        // 求和查询
        patterns.push(QueryPattern {
            name: "sum_query".to_string(),
            regex: Regex::new(r"(?i)总和|合计|求和.*?(\w+).*?的.*?(\w+)")?,
            sql_template: "SELECT SUM({column}) as total FROM {table}".to_string(),
            confidence: 0.85,
        });

        // 平均值查询
        patterns.push(QueryPattern {
            name: "average_query".to_string(),
            regex: Regex::new(r"(?i)平均|均值|平均数.*?(\w+).*?的.*?(\w+)")?,
            sql_template: "SELECT AVG({column}) as average FROM {table}".to_string(),
            confidence: 0.85,
        });

        // 最大值查询
        patterns.push(QueryPattern {
            name: "max_query".to_string(),
            regex: Regex::new(r"(?i)最大|最高|最多.*?(\w+).*?的.*?(\w+)")?,
            sql_template: "SELECT MAX({column}) as maximum FROM {table}".to_string(),
            confidence: 0.85,
        });

        // 最小值查询
        patterns.push(QueryPattern {
            name: "min_query".to_string(),
            regex: Regex::new(r"(?i)最小|最低|最少.*?(\w+).*?的.*?(\w+)")?,
            sql_template: "SELECT MIN({column}) as minimum FROM {table}".to_string(),
            confidence: 0.85,
        });

        // 分组查询
        patterns.push(QueryPattern {
            name: "group_by_query".to_string(),
            regex: Regex::new(r"(?i)按.*?(\w+).*?分组.*?(\w+).*?的.*?(\w+)")?,
            sql_template: "SELECT {group_column}, {agg_function}({column}) FROM {table} GROUP BY {group_column}".to_string(),
            confidence: 0.8,
        });

        // 时间范围查询
        patterns.push(QueryPattern {
            name: "time_range_query".to_string(),
            regex: Regex::new(r"(?i)(\d{4}年|\d+月|\d+日|昨天|今天|本周|本月|本年).*?(\w+)")?,
            sql_template: "SELECT * FROM {table} WHERE {time_column} >= '{start_date}' AND {time_column} <= '{end_date}'".to_string(),
            confidence: 0.75,
        });

        Ok(patterns)
    }

    /// 创建表名映射
    fn create_table_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // 金融相关表名映射
        mappings.insert("交易".to_string(), "transactions".to_string());
        mappings.insert("订单".to_string(), "orders".to_string());
        mappings.insert("用户".to_string(), "users".to_string());
        mappings.insert("客户".to_string(), "customers".to_string());
        mappings.insert("产品".to_string(), "products".to_string());
        mappings.insert("账户".to_string(), "accounts".to_string());
        mappings.insert("资金".to_string(), "funds".to_string());
        mappings.insert("投资".to_string(), "investments".to_string());
        mappings.insert("股票".to_string(), "stocks".to_string());
        mappings.insert("债券".to_string(), "bonds".to_string());
        mappings.insert("基金".to_string(), "mutual_funds".to_string());
        mappings.insert("风险".to_string(), "risks".to_string());
        mappings.insert("收益".to_string(), "returns".to_string());
        mappings.insert("价格".to_string(), "prices".to_string());
        mappings.insert("市场".to_string(), "markets".to_string());
        
        mappings
    }

    /// 解析自然语言查询
    #[instrument(skip(self, query))]
    pub async fn parse_query(&self, query: &str) -> Result<ParsedQuery> {
        debug!("解析自然语言查询: {}", query);
        
        let mut best_match: Option<(QueryPattern, f32)> = None;
        
        // 尝试匹配查询模式
        for pattern in &self.query_patterns {
            if pattern.regex.is_match(query) {
                let confidence = pattern.confidence * self.calculate_context_confidence(query);
                if best_match.is_none() || confidence > best_match.as_ref().unwrap().1 {
                    best_match = Some((pattern.clone(), confidence));
                }
            }
        }

        if let Some((pattern, confidence)) = best_match {
            self.parse_with_pattern(query, &pattern, confidence).await
        } else {
            // 如果没有匹配的模式，尝试通用解析
            self.parse_generic_query(query).await
        }
    }

    /// 使用模式解析查询
    async fn parse_with_pattern(&self, query: &str, pattern: &QueryPattern, confidence: f32) -> Result<ParsedQuery> {
        let query_type = match pattern.name.as_str() {
            "simple_select" => QueryType::Select,
            "count_query" | "sum_query" | "average_query" | "max_query" | "min_query" => QueryType::Aggregate,
            "group_by_query" => QueryType::Statistics,
            "time_range_query" => QueryType::Trend,
            _ => QueryType::Unknown,
        };

        // 提取表名
        let table_name = self.extract_table_name(query);
        
        // 提取列名
        let columns = self.extract_columns(query);
        
        // 提取条件
        let conditions = self.extract_conditions(query);
        
        // 提取聚合函数
        let aggregations = self.extract_aggregations(query, &pattern.name);

        Ok(ParsedQuery {
            query_type,
            table_name,
            columns,
            conditions,
            aggregations,
            order_by: None,
            limit: None,
            confidence,
            original_query: query.to_string(),
        })
    }

    /// 通用查询解析
    async fn parse_generic_query(&self, query: &str) -> Result<ParsedQuery> {
        debug!("使用通用解析器处理查询");
        
        Ok(ParsedQuery {
            query_type: QueryType::Unknown,
            table_name: self.extract_table_name(query),
            columns: self.extract_columns(query),
            conditions: self.extract_conditions(query),
            aggregations: Vec::new(),
            order_by: None,
            limit: None,
            confidence: 0.3,
            original_query: query.to_string(),
        })
    }

    /// 提取表名
    fn extract_table_name(&self, query: &str) -> Option<String> {
        for (chinese_name, english_name) in &self.table_mappings {
            if query.contains(chinese_name) {
                return Some(english_name.clone());
            }
        }
        
        // 如果没有找到映射，尝试提取可能的表名
        let table_regex = Regex::new(r"(?i)(\w+)表").unwrap();
        if let Some(captures) = table_regex.captures(query) {
            if let Some(table_match) = captures.get(1) {
                return Some(table_match.as_str().to_string());
            }
        }
        
        None
    }

    /// 提取列名
    fn extract_columns(&self, query: &str) -> Vec<String> {
        let mut columns = Vec::new();
        
        // 常见的列名关键词
        let column_keywords = vec![
            "金额", "数量", "价格", "时间", "日期", "名称", "编号", "ID",
            "收益", "成本", "利润", "风险", "评级", "状态", "类型"
        ];
        
        for keyword in &column_keywords {
            if query.contains(keyword) {
                columns.push(keyword.to_string());
            }
        }
        
        if columns.is_empty() {
            columns.push("*".to_string());
        }
        
        columns
    }

    /// 提取条件
    fn extract_conditions(&self, query: &str) -> Vec<Condition> {
        let mut conditions = Vec::new();
        
        // 简单的条件提取
        let condition_patterns = vec![
            (r"大于(\d+)", ">"),
            (r"小于(\d+)", "<"),
            (r"等于(\d+)", "="),
            (r"超过(\d+)", ">"),
            (r"低于(\d+)", "<"),
        ];
        
        for (pattern, operator) in condition_patterns {
            let regex = Regex::new(pattern).unwrap();
            if let Some(captures) = regex.captures(query) {
                if let Some(value_match) = captures.get(1) {
                    conditions.push(Condition {
                        column: "value".to_string(), // 简化处理
                        operator: operator.to_string(),
                        value: value_match.as_str().to_string(),
                    });
                }
            }
        }
        
        conditions
    }

    /// 提取聚合函数
    fn extract_aggregations(&self, query: &str, pattern_name: &str) -> Vec<Aggregation> {
        let mut aggregations = Vec::new();
        
        match pattern_name {
            "count_query" => {
                aggregations.push(Aggregation {
                    function: "COUNT".to_string(),
                    column: "*".to_string(),
                    alias: Some("count".to_string()),
                });
            }
            "sum_query" => {
                aggregations.push(Aggregation {
                    function: "SUM".to_string(),
                    column: "amount".to_string(), // 简化处理
                    alias: Some("total".to_string()),
                });
            }
            "average_query" => {
                aggregations.push(Aggregation {
                    function: "AVG".to_string(),
                    column: "amount".to_string(),
                    alias: Some("average".to_string()),
                });
            }
            "max_query" => {
                aggregations.push(Aggregation {
                    function: "MAX".to_string(),
                    column: "amount".to_string(),
                    alias: Some("maximum".to_string()),
                });
            }
            "min_query" => {
                aggregations.push(Aggregation {
                    function: "MIN".to_string(),
                    column: "amount".to_string(),
                    alias: Some("minimum".to_string()),
                });
            }
            _ => {}
        }
        
        aggregations
    }

    /// 计算上下文置信度
    fn calculate_context_confidence(&self, query: &str) -> f32 {
        let mut confidence: f32 = 1.0;
        
        // 如果查询包含金融相关关键词，提高置信度
        let financial_keywords = vec![
            "交易", "投资", "收益", "风险", "资金", "账户", "股票", "债券", "基金"
        ];
        
        for keyword in &financial_keywords {
            if query.contains(keyword) {
                confidence += 0.1;
                break;
            }
        }
        
        // 如果查询很短，降低置信度
        if query.len() < 10 {
            confidence *= 0.8;
        }
        
        confidence.min(1.0)
    }

    /// 生成SQL查询
    #[instrument(skip(self, parsed_query))]
    pub async fn generate_sql(&self, parsed_query: &ParsedQuery) -> Result<String> {
        let default_table = "default_table".to_string();
        let table_name = parsed_query.table_name.as_ref()
            .unwrap_or(&default_table);
        
        let mut sql = String::new();
        
        match parsed_query.query_type {
            QueryType::Select => {
                let columns = if parsed_query.columns.is_empty() {
                    "*".to_string()
                } else {
                    parsed_query.columns.join(", ")
                };
                sql = format!("SELECT {} FROM {}", columns, table_name);
            }
            QueryType::Aggregate => {
                if !parsed_query.aggregations.is_empty() {
                    let agg = &parsed_query.aggregations[0];
                    sql = format!("SELECT {}({}) as {} FROM {}", 
                                agg.function, agg.column, 
                                agg.alias.as_ref().unwrap_or(&agg.function), 
                                table_name);
                }
            }
            _ => {
                sql = format!("SELECT * FROM {}", table_name);
            }
        }
        
        // 添加条件
        if !parsed_query.conditions.is_empty() {
            let conditions: Vec<String> = parsed_query.conditions.iter()
                .map(|c| format!("{} {} {}", c.column, c.operator, c.value))
                .collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }
        
        // 添加限制
        if let Some(limit) = parsed_query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        debug!("生成的SQL查询: {}", sql);
        Ok(sql)
    }

    /// 生成自然语言回复
    #[instrument(skip(self, parsed_query, query_result))]
    pub async fn generate_response(&self, parsed_query: &ParsedQuery, query_result: &QueryResult) -> Result<String> {
        let mut response = String::new();
        
        match parsed_query.query_type {
            QueryType::Aggregate => {
                if !query_result.data.is_empty() {
                    if let Some(first_row) = query_result.data.get(0) {
                        if let Some(value) = first_row.values().next() {
                            response = format!("查询结果：{}", value);
                        }
                    }
                }
            }
            QueryType::Select => {
                response = format!("查询返回了{}条记录", query_result.row_count);
                if query_result.row_count > 0 && query_result.row_count <= 5 {
                    response.push_str("，详细数据如下：");
                    // 这里可以添加格式化的数据展示
                }
            }
            _ => {
                response = format!("查询执行完成，返回{}条记录", query_result.row_count);
            }
        }
        
        if response.is_empty() {
            response = "查询执行完成，但没有找到匹配的数据。".to_string();
        }
        
        debug!("生成的自然语言回复: {}", response);
        Ok(response)
    }
}
