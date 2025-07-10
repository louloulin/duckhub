//! 时间序列分析器模块

use duckhub_common::prelude::*;
use duckhub_database::DuckDBEngine;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use super::QueryResult;
use uuid::Uuid;

/// 时间序列分析器
pub struct TimeSeriesAnalyzer {
    /// 数据库引擎
    engine: Arc<DuckDBEngine>,
}

/// 时间序列分析类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSeriesAnalysisType {
    /// 趋势分析
    Trend,
    /// 季节性分析
    Seasonality,
    /// 异常检测
    AnomalyDetection,
    /// 预测
    Forecasting,
    /// 周期性分析
    Periodicity,
    /// 平稳性检验
    Stationarity,
}

/// 时间序列统计
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesStats {
    /// 数据点数量
    pub data_points: u64,
    /// 时间范围
    pub time_range: TimeRange,
    /// 基本统计
    pub basic_stats: BasicStats,
    /// 趋势信息
    pub trend_info: TrendInfo,
    /// 季节性信息
    pub seasonality_info: Option<SeasonalityInfo>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize)]
pub struct TimeRange {
    /// 开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 结束时间
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// 时间跨度（天）
    pub duration_days: i64,
}

/// 基本统计信息
#[derive(Debug, Clone, Serialize)]
pub struct BasicStats {
    /// 均值
    pub mean: f64,
    /// 标准差
    pub std_dev: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 中位数
    pub median: f64,
}

/// 趋势信息
#[derive(Debug, Clone, Serialize)]
pub struct TrendInfo {
    /// 趋势方向
    pub direction: String,
    /// 趋势强度
    pub strength: f64,
    /// 线性回归斜率
    pub slope: f64,
    /// R平方值
    pub r_squared: f64,
}

/// 季节性信息
#[derive(Debug, Clone, Serialize)]
pub struct SeasonalityInfo {
    /// 是否存在季节性
    pub has_seasonality: bool,
    /// 季节周期（天）
    pub period_days: Option<i32>,
    /// 季节性强度
    pub strength: f64,
}

impl TimeSeriesAnalyzer {
    /// 创建新的时间序列分析器
    #[instrument(skip(engine))]
    pub async fn new(engine: Arc<DuckDBEngine>) -> Result<Self> {
        let analyzer = Self { engine };
        debug!("时间序列分析器初始化完成");
        Ok(analyzer)
    }

    /// 执行时间序列分析
    #[instrument(skip(self))]
    pub async fn analyze(&self, table: &str, time_column: &str, value_column: &str) -> Result<QueryResult> {
        let query_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        // 获取时间序列统计信息
        let stats = self.get_time_series_stats(table, time_column, value_column).await?;

        // 转换为查询结果格式
        let mut result_data = Vec::new();
        let stats_json = serde_json::to_value(&stats)?;
        if let serde_json::Value::Object(map) = stats_json {
            result_data.push(map.into_iter().collect());
        }

        let execution_time = start_time.elapsed();

        Ok(QueryResult {
            query_id,
            execution_time_ms: execution_time.as_millis() as u64,
            row_count: result_data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("TimeSeriesAnalysis".to_string()),
            data: result_data,
        })
    }

    /// 生成趋势分析查询
    pub async fn generate_trend_analysis(&self, table: &str, time_column: &str, value_column: &str, period: &str) -> Result<QueryResult> {
        let sql = format!(
            r#"
            WITH trend_data AS (
                SELECT
                    DATE_TRUNC('{period}', {time_column}) as period,
                    AVG({value_column}) as avg_value,
                    COUNT(*) as data_points,
                    MIN({value_column}) as min_value,
                    MAX({value_column}) as max_value,
                    STDDEV({value_column}) as std_value
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                GROUP BY DATE_TRUNC('{period}', {time_column})
                ORDER BY period
            ),
            trend_calculation AS (
                SELECT
                    period,
                    avg_value,
                    data_points,
                    min_value,
                    max_value,
                    std_value,
                    LAG(avg_value) OVER (ORDER BY period) as prev_avg,
                    avg_value - LAG(avg_value) OVER (ORDER BY period) as period_change,
                    CASE
                        WHEN LAG(avg_value) OVER (ORDER BY period) > 0
                        THEN (avg_value - LAG(avg_value) OVER (ORDER BY period)) / LAG(avg_value) OVER (ORDER BY period) * 100
                        ELSE NULL
                    END as percent_change,
                    AVG(avg_value) OVER (ORDER BY period ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) as moving_avg_3,
                    ROW_NUMBER() OVER (ORDER BY period) as time_index
                FROM trend_data
            )
            SELECT
                period,
                avg_value,
                data_points,
                min_value,
                max_value,
                std_value,
                prev_avg,
                period_change,
                percent_change,
                moving_avg_3,
                CASE
                    WHEN period_change > 0 THEN 'Increasing'
                    WHEN period_change < 0 THEN 'Decreasing'
                    ELSE 'Stable'
                END as trend_direction
            FROM trend_calculation
            ORDER BY period
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column,
            period = period
        );

        let data = self.engine.query(&sql).await?;

        Ok(QueryResult {
            query_id: Uuid::new_v4().to_string(),
            execution_time_ms: 0,
            row_count: data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("TrendAnalysis".to_string()),
            data,
        })
    }

    /// 生成季节性分析查询
    pub async fn generate_seasonality_analysis(&self, table: &str, time_column: &str, value_column: &str) -> Result<QueryResult> {
        let sql = format!(
            r#"
            WITH seasonal_data AS (
                SELECT
                    EXTRACT(MONTH FROM {time_column}) as month,
                    EXTRACT(DOW FROM {time_column}) as day_of_week,
                    EXTRACT(HOUR FROM {time_column}) as hour,
                    AVG({value_column}) as avg_value,
                    COUNT(*) as data_points,
                    STDDEV({value_column}) as std_value
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                GROUP BY EXTRACT(MONTH FROM {time_column}), EXTRACT(DOW FROM {time_column}), EXTRACT(HOUR FROM {time_column})
            ),
            monthly_pattern AS (
                SELECT
                    month,
                    AVG(avg_value) as monthly_avg,
                    SUM(data_points) as monthly_points,
                    STDDEV(avg_value) as monthly_std
                FROM seasonal_data
                GROUP BY month
                ORDER BY month
            ),
            weekly_pattern AS (
                SELECT
                    day_of_week,
                    AVG(avg_value) as weekly_avg,
                    SUM(data_points) as weekly_points,
                    STDDEV(avg_value) as weekly_std
                FROM seasonal_data
                GROUP BY day_of_week
                ORDER BY day_of_week
            ),
            hourly_pattern AS (
                SELECT
                    hour,
                    AVG(avg_value) as hourly_avg,
                    SUM(data_points) as hourly_points,
                    STDDEV(avg_value) as hourly_std
                FROM seasonal_data
                GROUP BY hour
                ORDER BY hour
            )
            SELECT
                'monthly' as pattern_type,
                month as period_value,
                monthly_avg as avg_value,
                monthly_points as data_points,
                monthly_std as std_value
            FROM monthly_pattern
            UNION ALL
            SELECT
                'weekly' as pattern_type,
                day_of_week as period_value,
                weekly_avg as avg_value,
                weekly_points as data_points,
                weekly_std as std_value
            FROM weekly_pattern
            UNION ALL
            SELECT
                'hourly' as pattern_type,
                hour as period_value,
                hourly_avg as avg_value,
                hourly_points as data_points,
                hourly_std as std_value
            FROM hourly_pattern
            ORDER BY pattern_type, period_value
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column
        );

        let data = self.engine.query(&sql).await?;

        Ok(QueryResult {
            query_id: Uuid::new_v4().to_string(),
            execution_time_ms: 0,
            row_count: data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("SeasonalityAnalysis".to_string()),
            data,
        })
    }

    /// 获取时间序列统计信息
    async fn get_time_series_stats(&self, table: &str, time_column: &str, value_column: &str) -> Result<TimeSeriesStats> {
        // 获取基本统计信息
        let basic_stats_sql = format!(
            r#"
            SELECT 
                COUNT(*) as data_points,
                MIN({time_column}) as start_time,
                MAX({time_column}) as end_time,
                AVG({value_column}) as mean_value,
                STDDEV({value_column}) as std_value,
                MIN({value_column}) as min_value,
                MAX({value_column}) as max_value,
                MEDIAN({value_column}) as median_value
            FROM {table}
            WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column
        );

        let basic_result = self.engine.query(&basic_stats_sql).await?;
        let basic_row = basic_result.get(0)
            .ok_or_else(|| DuckHubError::NotFound { resource: "基本统计信息".to_string() })?;

        let data_points = basic_row.get("data_points").and_then(|v| v.as_u64()).unwrap_or(0);
        let start_time_str = basic_row.get("start_time").and_then(|v| v.as_str()).unwrap_or("");
        let end_time_str = basic_row.get("end_time").and_then(|v| v.as_str()).unwrap_or("");
        
        let start_time = chrono::DateTime::parse_from_rfc3339(start_time_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        let end_time = chrono::DateTime::parse_from_rfc3339(end_time_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let duration_days = (end_time - start_time).num_days();

        let time_range = TimeRange {
            start_time,
            end_time,
            duration_days,
        };

        let basic_stats = BasicStats {
            mean: basic_row.get("mean_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            std_dev: basic_row.get("std_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            min: basic_row.get("min_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            max: basic_row.get("max_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
            median: basic_row.get("median_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
        };

        // 获取趋势信息
        let trend_info = self.analyze_trend(table, time_column, value_column).await?;

        // 分析季节性（如果数据足够）
        let seasonality_info = if duration_days > 30 {
            Some(self.analyze_seasonality(table, time_column, value_column).await?)
        } else {
            None
        };

        Ok(TimeSeriesStats {
            data_points,
            time_range,
            basic_stats,
            trend_info,
            seasonality_info,
        })
    }

    /// 分析趋势
    async fn analyze_trend(&self, table: &str, time_column: &str, value_column: &str) -> Result<TrendInfo> {
        let trend_sql = format!(
            r#"
            WITH trend_data AS (
                SELECT 
                    {time_column},
                    {value_column},
                    ROW_NUMBER() OVER (ORDER BY {time_column}) as x
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                ORDER BY {time_column}
            ),
            trend_stats AS (
                SELECT 
                    COUNT(*) as n,
                    SUM(x) as sum_x,
                    SUM({value_column}) as sum_y,
                    SUM(x * {value_column}) as sum_xy,
                    SUM(x * x) as sum_x2,
                    SUM({value_column} * {value_column}) as sum_y2
                FROM trend_data
            )
            SELECT 
                (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x) as slope,
                (n * sum_xy - sum_x * sum_y) / SQRT((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)) as correlation
            FROM trend_stats
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column
        );

        let trend_result = self.engine.query(&trend_sql).await?;
        let trend_row = trend_result.get(0)
            .ok_or_else(|| DuckHubError::NotFound { resource: "趋势信息".to_string() })?;

        let slope = trend_row.get("slope").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let correlation = trend_row.get("correlation").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let direction = if slope > 0.01 {
            "上升".to_string()
        } else if slope < -0.01 {
            "下降".to_string()
        } else {
            "平稳".to_string()
        };

        let strength = correlation.abs();
        let r_squared = correlation * correlation;

        Ok(TrendInfo {
            direction,
            strength,
            slope,
            r_squared,
        })
    }

    /// 分析季节性
    async fn analyze_seasonality(&self, table: &str, time_column: &str, value_column: &str) -> Result<SeasonalityInfo> {
        // 简化的季节性分析：检查周、月的模式
        let seasonality_sql = format!(
            r#"
            WITH daily_avg AS (
                SELECT 
                    EXTRACT(DOW FROM {time_column}) as day_of_week,
                    AVG({value_column}) as avg_value
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                GROUP BY EXTRACT(DOW FROM {time_column})
            ),
            weekly_variance AS (
                SELECT 
                    STDDEV(avg_value) / AVG(avg_value) as weekly_cv
                FROM daily_avg
            ),
            monthly_avg AS (
                SELECT 
                    EXTRACT(DAY FROM {time_column}) as day_of_month,
                    AVG({value_column}) as avg_value
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                GROUP BY EXTRACT(DAY FROM {time_column})
            ),
            monthly_variance AS (
                SELECT 
                    STDDEV(avg_value) / AVG(avg_value) as monthly_cv
                FROM monthly_avg
            )
            SELECT 
                weekly_cv,
                monthly_cv,
                CASE 
                    WHEN weekly_cv > 0.1 THEN true
                    WHEN monthly_cv > 0.1 THEN true
                    ELSE false
                END as has_seasonality
            FROM weekly_variance, monthly_variance
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column
        );

        let seasonality_result = self.engine.query(&seasonality_sql).await?;
        let seasonality_row = seasonality_result.get(0)
            .ok_or_else(|| DuckHubError::NotFound { resource: "季节性信息".to_string() })?;

        let weekly_cv = seasonality_row.get("weekly_cv").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let monthly_cv = seasonality_row.get("monthly_cv").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let has_seasonality = seasonality_row.get("has_seasonality").and_then(|v| v.as_bool()).unwrap_or(false);

        let (period_days, strength) = if weekly_cv > monthly_cv {
            (Some(7), weekly_cv)
        } else {
            (Some(30), monthly_cv)
        };

        Ok(SeasonalityInfo {
            has_seasonality,
            period_days: if has_seasonality { period_days } else { None },
            strength,
        })
    }

    /// 异常检测
    pub async fn detect_anomalies(&self, table: &str, time_column: &str, value_column: &str, threshold: f64) -> Result<QueryResult> {
        let query_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let sql = format!(
            r#"
            WITH time_series AS (
                SELECT 
                    {time_column},
                    {value_column},
                    AVG({value_column}) OVER (
                        ORDER BY {time_column} 
                        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
                    ) as moving_avg,
                    STDDEV({value_column}) OVER (
                        ORDER BY {time_column} 
                        ROWS BETWEEN 6 PRECEDING AND CURRENT ROW
                    ) as moving_std
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                ORDER BY {time_column}
            ),
            anomalies AS (
                SELECT 
                    *,
                    ABS({value_column} - moving_avg) / NULLIF(moving_std, 0) as z_score,
                    CASE 
                        WHEN ABS({value_column} - moving_avg) / NULLIF(moving_std, 0) > {threshold} THEN true
                        ELSE false
                    END as is_anomaly
                FROM time_series
                WHERE moving_std IS NOT NULL AND moving_std > 0
            )
            SELECT 
                {time_column},
                {value_column},
                moving_avg,
                moving_std,
                z_score,
                is_anomaly,
                CASE 
                    WHEN {value_column} > moving_avg + {threshold} * moving_std THEN '异常高值'
                    WHEN {value_column} < moving_avg - {threshold} * moving_std THEN '异常低值'
                    ELSE '正常'
                END as anomaly_type
            FROM anomalies
            WHERE is_anomaly = true
            ORDER BY {time_column}
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column,
            threshold = threshold
        );

        let data = self.engine.query(&sql).await?;
        let execution_time = start_time.elapsed();

        Ok(QueryResult {
            query_id,
            execution_time_ms: execution_time.as_millis() as u64,
            row_count: data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("TimeSeriesAnomalyDetection".to_string()),
            data,
        })
    }

    /// 简单预测（线性外推）
    pub async fn simple_forecast(&self, table: &str, time_column: &str, value_column: &str, periods: i32) -> Result<QueryResult> {
        let query_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let sql = format!(
            r#"
            WITH trend_data AS (
                SELECT 
                    {time_column},
                    {value_column},
                    ROW_NUMBER() OVER (ORDER BY {time_column}) as x
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                ORDER BY {time_column}
            ),
            trend_params AS (
                SELECT 
                    COUNT(*) as n,
                    SUM(x) as sum_x,
                    SUM({value_column}) as sum_y,
                    SUM(x * {value_column}) as sum_xy,
                    SUM(x * x) as sum_x2,
                    MAX(x) as max_x,
                    MAX({time_column}) as last_time
                FROM trend_data
            ),
            regression AS (
                SELECT 
                    (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x) as slope,
                    (sum_y - ((n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)) * sum_x) / n as intercept,
                    max_x,
                    last_time
                FROM trend_params
            ),
            forecast_periods AS (
                SELECT generate_series(1, {periods}) as period_ahead
            )
            SELECT 
                last_time + INTERVAL '1 day' * period_ahead as forecast_time,
                intercept + slope * (max_x + period_ahead) as forecast_value,
                period_ahead
            FROM regression, forecast_periods
            ORDER BY period_ahead
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column,
            periods = periods
        );

        let data = self.engine.query(&sql).await?;
        let execution_time = start_time.elapsed();

        Ok(QueryResult {
            query_id,
            execution_time_ms: execution_time.as_millis() as u64,
            row_count: data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("TimeSeriesForecast".to_string()),
            data,
        })
    }

    /// 计算时间序列分解
    pub async fn decompose_time_series(&self, table: &str, time_column: &str, value_column: &str, period: i32) -> Result<QueryResult> {
        let query_id = Uuid::new_v4().to_string();
        let start_time = std::time::Instant::now();

        let sql = format!(
            r#"
            WITH time_series AS (
                SELECT 
                    {time_column},
                    {value_column},
                    ROW_NUMBER() OVER (ORDER BY {time_column}) as x
                FROM {table}
                WHERE {time_column} IS NOT NULL AND {value_column} IS NOT NULL
                ORDER BY {time_column}
            ),
            trend_component AS (
                SELECT 
                    *,
                    AVG({value_column}) OVER (
                        ORDER BY {time_column} 
                        ROWS BETWEEN {half_period} PRECEDING AND {half_period} FOLLOWING
                    ) as trend
                FROM time_series
            ),
            seasonal_component AS (
                SELECT 
                    *,
                    {value_column} - trend as detrended,
                    x % {period} as seasonal_index
                FROM trend_component
            ),
            seasonal_avg AS (
                SELECT 
                    seasonal_index,
                    AVG(detrended) as seasonal_factor
                FROM seasonal_component
                WHERE trend IS NOT NULL
                GROUP BY seasonal_index
            )
            SELECT 
                sc.{time_column},
                sc.{value_column} as original,
                sc.trend,
                COALESCE(sa.seasonal_factor, 0) as seasonal,
                sc.{value_column} - sc.trend - COALESCE(sa.seasonal_factor, 0) as residual
            FROM seasonal_component sc
            LEFT JOIN seasonal_avg sa ON sc.seasonal_index = sa.seasonal_index
            WHERE sc.trend IS NOT NULL
            ORDER BY sc.{time_column}
            "#,
            table = table,
            time_column = time_column,
            value_column = value_column,
            period = period,
            half_period = period / 2
        );

        let data = self.engine.query(&sql).await?;
        let execution_time = start_time.elapsed();

        Ok(QueryResult {
            query_id,
            execution_time_ms: execution_time.as_millis() as u64,
            row_count: data.len(),
            optimized: false,
            cache_hit: false,
            execution_plan: Some("TimeSeriesDecomposition".to_string()),
            data,
        })
    }
}
