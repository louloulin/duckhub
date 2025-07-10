//! 系统监控模块

use crate::metrics::{SystemMetrics, NetworkIO, DiskIO, LoadAverage};
use duckhub_common::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt, ProcessExt};
use tracing::{debug, instrument};

/// 系统监控器
pub struct SystemMonitor {
    /// 系统信息
    system: System,
}

/// 系统状态
#[derive(Debug, Clone, Serialize)]
pub struct SystemStatus {
    /// 系统信息
    pub system_info: SystemInfo,
    /// CPU信息
    pub cpu_info: CpuInfo,
    /// 内存信息
    pub memory_info: MemoryInfo,
    /// 磁盘信息
    pub disk_info: Vec<DiskInfo>,
    /// 网络信息
    pub network_info: Vec<NetworkInfo>,
    /// 进程信息
    pub process_info: ProcessInfo,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 系统基本信息
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    /// 操作系统名称
    pub os_name: String,
    /// 操作系统版本
    pub os_version: String,
    /// 内核版本
    pub kernel_version: String,
    /// 主机名
    pub hostname: String,
    /// 启动时间
    pub boot_time: u64,
}

/// CPU信息
#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    /// CPU核心数
    pub core_count: usize,
    /// CPU使用率
    pub usage_percent: f32,
    /// CPU频率
    pub frequency: u64,
    /// CPU品牌
    pub brand: String,
}

/// 内存信息
#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    /// 总内存（字节）
    pub total_memory: u64,
    /// 已用内存（字节）
    pub used_memory: u64,
    /// 可用内存（字节）
    pub available_memory: u64,
    /// 内存使用率
    pub usage_percent: f64,
    /// 总交换空间（字节）
    pub total_swap: u64,
    /// 已用交换空间（字节）
    pub used_swap: u64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    /// 磁盘名称
    pub name: String,
    /// 挂载点
    pub mount_point: String,
    /// 文件系统类型
    pub file_system: String,
    /// 总空间（字节）
    pub total_space: u64,
    /// 可用空间（字节）
    pub available_space: u64,
    /// 使用率
    pub usage_percent: f64,
}

/// 网络信息
#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    /// 网络接口名称
    pub interface_name: String,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 发送包数
    pub packets_sent: u64,
    /// 错误数
    pub errors_on_received: u64,
    /// 错误数
    pub errors_on_transmitted: u64,
}

/// 进程信息
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    /// 总进程数
    pub total_processes: usize,
    /// 运行中的进程数
    pub running_processes: usize,
    /// 睡眠中的进程数
    pub sleeping_processes: usize,
    /// 僵尸进程数
    pub zombie_processes: usize,
}

impl SystemMonitor {
    /// 创建新的系统监控器
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self { system }
    }

    /// 收集系统指标
    #[instrument(skip(self))]
    pub async fn collect_metrics(&self) -> Result<SystemMetrics> {
        // 刷新系统信息
        let mut system = self.system.clone();
        system.refresh_all();

        // 计算CPU使用率
        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;

        // 计算内存使用率
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_usage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        // 计算磁盘使用率（取第一个磁盘）
        let disk_usage = system.disks().iter().next()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                if total > 0 {
                    ((total - available) as f64 / total as f64) * 100.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        // 收集网络IO统计
        let network_io = self.collect_network_io(&system);

        // 收集磁盘IO统计（简化实现）
        let disk_io = DiskIO {
            bytes_read: 0,
            bytes_written: 0,
            read_count: 0,
            write_count: 0,
        };

        // 进程数
        let process_count = system.processes().len() as u32;

        // 负载平均值（简化实现）
        let load_average = LoadAverage {
            one_minute: cpu_usage / 100.0,
            five_minutes: cpu_usage / 100.0,
            fifteen_minutes: cpu_usage / 100.0,
        };

        debug!("收集系统指标完成");

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_io,
            disk_io,
            process_count,
            load_average,
            timestamp: Utc::now(),
        })
    }

    /// 获取系统状态
    #[instrument(skip(self))]
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let mut system = self.system.clone();
        system.refresh_all();

        // 系统基本信息
        let system_info = SystemInfo {
            os_name: system.name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: system.os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: system.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: system.host_name().unwrap_or_else(|| "Unknown".to_string()),
            boot_time: system.boot_time(),
        };

        // CPU信息
        let cpu_info = CpuInfo {
            core_count: system.cpus().len(),
            usage_percent: system.global_cpu_info().cpu_usage(),
            frequency: system.global_cpu_info().frequency(),
            brand: system.global_cpu_info().brand().to_string(),
        };

        // 内存信息
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let memory_info = MemoryInfo {
            total_memory,
            used_memory,
            available_memory: total_memory - used_memory,
            usage_percent: if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            },
            total_swap: system.total_swap(),
            used_swap: system.used_swap(),
        };

        // 磁盘信息
        let disk_info: Vec<DiskInfo> = system.disks().iter().map(|disk| {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
                total_space,
                available_space,
                usage_percent: if total_space > 0 {
                    ((total_space - available_space) as f64 / total_space as f64) * 100.0
                } else {
                    0.0
                },
            }
        }).collect();

        // 网络信息
        let network_info: Vec<NetworkInfo> = system.networks().iter().map(|(interface_name, data)| {
            NetworkInfo {
                interface_name: interface_name.clone(),
                bytes_received: data.received(),
                bytes_sent: data.transmitted(),
                packets_received: data.packets_received(),
                packets_sent: data.packets_transmitted(),
                errors_on_received: data.errors_on_received(),
                errors_on_transmitted: data.errors_on_transmitted(),
            }
        }).collect();

        // 进程信息
        let processes = system.processes();
        let process_info = ProcessInfo {
            total_processes: processes.len(),
            running_processes: processes.values().filter(|p| p.status().to_string().contains("Running")).count(),
            sleeping_processes: processes.values().filter(|p| p.status().to_string().contains("Sleep")).count(),
            zombie_processes: processes.values().filter(|p| p.status().to_string().contains("Zombie")).count(),
        };

        Ok(SystemStatus {
            system_info,
            cpu_info,
            memory_info,
            disk_info,
            network_info,
            process_info,
            timestamp: Utc::now(),
        })
    }

    /// 收集网络IO统计
    fn collect_network_io(&self, system: &System) -> NetworkIO {
        let mut total_bytes_received = 0;
        let mut total_bytes_sent = 0;
        let mut total_packets_received = 0;
        let mut total_packets_sent = 0;

        for (_interface_name, data) in system.networks() {
            total_bytes_received += data.received();
            total_bytes_sent += data.transmitted();
            total_packets_received += data.packets_received();
            total_packets_sent += data.packets_transmitted();
        }

        NetworkIO {
            bytes_received: total_bytes_received,
            bytes_sent: total_bytes_sent,
            packets_received: total_packets_received,
            packets_sent: total_packets_sent,
        }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}
