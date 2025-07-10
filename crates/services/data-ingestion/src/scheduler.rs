//! 任务调度器模块 - 管理数据采集任务的调度和执行

use duckhub_common::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, error, instrument};

/// 任务调度器
#[derive(Debug)]
pub struct TaskScheduler {
    /// 调度器配置
    config: SchedulerConfig,
    /// 运行中的任务
    running_tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
    /// 任务队列
    task_queue: Arc<RwLock<Vec<TaskDefinition>>>,
    /// 是否运行中
    is_running: Arc<RwLock<bool>>,
}

/// 调度器配置
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务超时时间（秒）
    pub task_timeout_secs: u64,
    /// 调度间隔（秒）
    pub schedule_interval_secs: u64,
    /// 失败重试次数
    pub max_retries: u32,
    /// 重试间隔（秒）
    pub retry_interval_secs: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout_secs: 300, // 5分钟
            schedule_interval_secs: 60, // 1分钟
            max_retries: 3,
            retry_interval_secs: 30,
        }
    }
}

/// 任务定义
#[derive(Debug, Clone)]
pub struct TaskDefinition {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 调度表达式（Cron格式）
    pub schedule: String,
    /// 数据源名称
    pub source_name: String,
    /// 处理器链
    pub processors: Vec<String>,
    /// 目标连接器
    pub target_connector: String,
    /// 任务配置
    pub config: HashMap<String, String>,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后执行时间
    pub last_executed_at: Option<DateTime<Utc>>,
}

/// 任务类型
#[derive(Debug, Clone)]
pub enum TaskType {
    /// 实时数据采集
    RealTimeIngestion,
    /// 批量数据导入
    BatchImport,
    /// 定时数据同步
    ScheduledSync,
    /// 数据质量检查
    DataQualityCheck,
    /// 数据清理
    DataCleanup,
}

/// 调度任务
#[derive(Debug)]
pub struct ScheduledTask {
    /// 任务定义
    pub definition: TaskDefinition,
    /// 任务状态
    pub status: TaskStatus,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 结束时间
    pub ended_at: Option<DateTime<Utc>>,
    /// 处理的记录数
    pub records_processed: u64,
    /// 失败的记录数
    pub records_failed: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 错误信息
    pub error_message: Option<String>,
    /// 任务句柄
    pub task_handle: Option<tokio::task::JoinHandle<()>>,
}

/// 任务状态
#[derive(Debug, Clone)]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
    /// 重试中
    Retrying,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动调度器
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(DuckHubError::validation("调度器已在运行中"));
        }
        *is_running = true;
        drop(is_running);

        info!("启动任务调度器");

        // 启动调度循环
        let scheduler = Arc::new(self.clone());
        tokio::spawn(async move {
            scheduler.schedule_loop().await;
        });

        Ok(())
    }

    /// 停止调度器
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        info!("停止任务调度器");

        // 取消所有运行中的任务
        let mut running_tasks = self.running_tasks.write().await;
        for (task_id, task) in running_tasks.iter_mut() {
            if let Some(handle) = &task.task_handle {
                handle.abort();
                info!("取消任务: {}", task_id);
            }
            task.status = TaskStatus::Cancelled;
        }
        running_tasks.clear();

        Ok(())
    }

    /// 添加任务定义
    #[instrument(skip(self))]
    pub async fn add_task(&self, mut task_def: TaskDefinition) -> Result<()> {
        task_def.id = Uuid::new_v4().to_string();
        task_def.created_at = Utc::now();

        let mut task_queue = self.task_queue.write().await;
        task_queue.push(task_def.clone());
        
        info!("添加任务定义: {} ({})", task_def.name, task_def.id);
        Ok(())
    }

    /// 移除任务定义
    #[instrument(skip(self))]
    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        let mut task_queue = self.task_queue.write().await;
        task_queue.retain(|task| task.id != task_id);
        
        // 如果任务正在运行，也要停止它
        let mut running_tasks = self.running_tasks.write().await;
        if let Some(task) = running_tasks.get_mut(task_id) {
            if let Some(handle) = &task.task_handle {
                handle.abort();
            }
            task.status = TaskStatus::Cancelled;
            running_tasks.remove(task_id);
        }

        info!("移除任务: {}", task_id);
        Ok(())
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let running_tasks = self.running_tasks.read().await;
        running_tasks.get(task_id).map(|task| task.status.clone())
    }

    /// 获取所有任务统计
    pub async fn get_task_stats(&self) -> TaskStats {
        let running_tasks = self.running_tasks.read().await;
        let task_queue = self.task_queue.read().await;

        let mut stats = TaskStats {
            total_tasks: task_queue.len() as u64,
            running_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            pending_tasks: 0,
            total_records_processed: 0,
            total_records_failed: 0,
        };

        for task in running_tasks.values() {
            match task.status {
                TaskStatus::Running => stats.running_tasks += 1,
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed => stats.failed_tasks += 1,
                TaskStatus::Pending => stats.pending_tasks += 1,
                _ => {}
            }
            stats.total_records_processed += task.records_processed;
            stats.total_records_failed += task.records_failed;
        }

        stats
    }

    /// 调度循环
    async fn schedule_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.schedule_interval_secs)
        );

        loop {
            interval.tick().await;

            let is_running = *self.is_running.read().await;
            if !is_running {
                break;
            }

            // 检查需要执行的任务
            if let Err(e) = self.check_and_execute_tasks().await {
                error!("检查任务时发生错误: {}", e);
            }

            // 清理已完成的任务
            self.cleanup_completed_tasks().await;
        }
    }

    /// 检查并执行任务
    async fn check_and_execute_tasks(&self) -> Result<()> {
        let task_queue = self.task_queue.read().await;
        let running_tasks_count = self.running_tasks.read().await.len();

        // 检查是否达到最大并发数
        if running_tasks_count >= self.config.max_concurrent_tasks {
            debug!("达到最大并发任务数限制: {}", self.config.max_concurrent_tasks);
            return Ok(());
        }

        // 查找需要执行的任务
        for task_def in task_queue.iter() {
            if !task_def.enabled {
                continue;
            }

            // 简化的调度逻辑 - 实际应该解析Cron表达式
            let should_execute = self.should_execute_task(task_def).await;
            if should_execute {
                self.execute_task(task_def.clone()).await?;
            }
        }

        Ok(())
    }

    /// 判断任务是否应该执行
    async fn should_execute_task(&self, task_def: &TaskDefinition) -> bool {
        // 简化的调度逻辑
        // 实际应该使用Cron表达式解析库
        match task_def.last_executed_at {
            None => true, // 从未执行过
            Some(last_time) => {
                let now = Utc::now();
                let elapsed = now.signed_duration_since(last_time);
                elapsed.num_seconds() >= 60 // 简单的1分钟间隔
            }
        }
    }

    /// 执行任务
    async fn execute_task(&self, task_def: TaskDefinition) -> Result<()> {
        let task_id = task_def.id.clone();
        
        // 创建调度任务
        let scheduled_task = ScheduledTask {
            definition: task_def.clone(),
            status: TaskStatus::Running,
            started_at: Utc::now(),
            ended_at: None,
            records_processed: 0,
            records_failed: 0,
            retry_count: 0,
            error_message: None,
            task_handle: None,
        };

        // 添加到运行任务列表
        {
            let mut running_tasks = self.running_tasks.write().await;
            running_tasks.insert(task_id.clone(), scheduled_task);
        }

        info!("开始执行任务: {} ({})", task_def.name, task_id);

        // 启动任务执行
        let running_tasks_ref = Arc::clone(&self.running_tasks);
        let task_id_clone = task_id.clone();
        let handle = tokio::spawn(async move {
            // TODO: 实现真实的任务执行逻辑
            // 这里是Mock实现
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // 更新任务状态
            let mut running_tasks = running_tasks_ref.write().await;
            if let Some(task) = running_tasks.get_mut(&task_id_clone) {
                task.status = TaskStatus::Completed;
                task.ended_at = Some(Utc::now());
                task.records_processed = 100; // Mock数据
            }
        });

        // 更新任务句柄
        {
            let mut running_tasks = self.running_tasks.write().await;
            if let Some(task) = running_tasks.get_mut(&task_id) {
                task.task_handle = Some(handle);
            }
        }

        Ok(())
    }

    /// 清理已完成的任务
    async fn cleanup_completed_tasks(&self) {
        let mut running_tasks = self.running_tasks.write().await;
        let mut completed_tasks = Vec::new();

        for (task_id, task) in running_tasks.iter() {
            match task.status {
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    completed_tasks.push(task_id.clone());
                }
                _ => {}
            }
        }

        for task_id in completed_tasks {
            if let Some(task) = running_tasks.remove(&task_id) {
                info!("清理已完成任务: {} (状态: {:?})", task_id, task.status);
            }
        }
    }
}

// 为了支持clone，需要手动实现
impl Clone for TaskScheduler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            running_tasks: Arc::clone(&self.running_tasks),
            task_queue: Arc::clone(&self.task_queue),
            is_running: Arc::clone(&self.is_running),
        }
    }
}

/// 任务统计信息
#[derive(Debug, Clone)]
pub struct TaskStats {
    /// 总任务数
    pub total_tasks: u64,
    /// 运行中的任务数
    pub running_tasks: u64,
    /// 已完成的任务数
    pub completed_tasks: u64,
    /// 失败的任务数
    pub failed_tasks: u64,
    /// 等待中的任务数
    pub pending_tasks: u64,
    /// 处理的记录总数
    pub total_records_processed: u64,
    /// 失败的记录总数
    pub total_records_failed: u64,
}
