//! 一次采样的快照。Rust 侧 1s 定时采样，批量 emit 给前端；前端不做轮询 IPC。

use serde::{Deserialize, Serialize};

/// 面板用到的全部指标。任何一项读不到就是 None —— UI 隐藏对应区域，不填 0。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub ts_ms: u64,
    pub device: Option<DeviceSnapshot>,
    pub health: Option<HealthSnapshot>,
    pub cpu: Option<CpuSnapshot>,
    pub memory: Option<MemorySnapshot>,
    pub network: Option<NetworkSnapshot>,
    pub disk: Option<DiskSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    /// 主机名
    pub host: String,
    /// 操作系统名，如 Windows 11 Pro
    pub os: String,
    /// 内核/构建号
    pub os_version: String,
    /// CPU 品牌字符串
    pub cpu_brand: String,
    /// 逻辑核数
    pub logical_cores: usize,
    /// 物理内存总量（字节）
    pub total_memory_bytes: u64,
    /// 开机至今秒数
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthBand {
    Excellent,
    Good,
    Watch,
    Critical,
}

impl HealthBand {
    pub fn label_zh(self) -> &'static str {
        match self {
            Self::Excellent => "很好",
            Self::Good => "良好",
            Self::Watch => "需注意",
            Self::Critical => "异常",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthIssue {
    /// 指标名，如 cpu / memory
    pub metric: String,
    /// 人话原因
    pub reason: String,
    pub points: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSnapshot {
    /// 0–100，从 100 起扣
    pub score: u32,
    pub band: HealthBand,
    /// 汇总一句话
    pub summary: String,
    /// 可点开看的扣分明细
    pub issues: Vec<HealthIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuSnapshot {
    /// 总使用率 0..100
    pub usage: f32,
    /// 每核使用率，长度 = 逻辑核数，直接喂柱状图
    pub per_core: Vec<f32>,
    /// PDH System\Processor Queue Length（参考图"负载 4.2/11核"的 Windows 替代项）
    pub queue_length: Option<u32>,
    /// 短时峰值（采样器内维护滑动窗口）
    pub peak: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    pub usage: f32,
    pub total_bytes: u64,
    pub used_bytes: u64,
    /// 已提交 / 提交上限。注意：Windows 的"交换"有两套语义（提交内存 vs pagefile 实际占用，
    /// W0 实测 10.2GB vs 295MB），UI 文案统一用"已提交"，不出现"交换"字样
    pub committed_bytes: Option<u64>,
    pub committed_limit_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    /// 主接口 ifIndex —— 唯一可靠键，名字只用于展示（W0 实测 11 个适配器名称失真）。
    /// v0.1 用 sysinfo 友好名启发式选路；IP Helper ifIndex 在后续版本替换。
    pub if_index: u32,
    /// 接口友好名，仅展示
    pub friendly_name: String,
    pub download_bps: u64,
    pub upload_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSnapshot {
    /// 系统盘
    pub system_drive: DriveSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveSnapshot {
    pub letter: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

impl DriveSnapshot {
    pub fn usage(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f32 / self.total_bytes as f32) * 100.0
    }
}
