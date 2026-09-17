//! 实时采集层：sysinfo 承担 CPU/内存/磁盘/网络（W0 实测 0–25ms）。
//!
//! 设计决策（规划第五节）：
//! - 推送而非拉取：由 sampler 定时调本模块，前端不轮询
//! - 读不到 → None，UI 整卡消失，不填 0
//! - 温度/风扇走厂商 provider（W4b），本模块刻意不采温度（W0：通用通路是假数据）

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Disks, NetworkData, Networks, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL};

use crate::score::{self, CpuStressTracker, ScoreThresholds};
use crate::snapshot::{
    CpuSnapshot, DeviceSnapshot, DiskSnapshot, DriveSnapshot, MemorySnapshot, NetworkSnapshot,
    Snapshot,
};
use crate::validity;

/// 网络接口启发式黑名单：虚拟网卡/派生驱动名（W0：本机 11 个适配器）。
const NET_BLACKLIST: &[&str] = &[
    "vmware",
    "hyper-v",
    "vethernet",
    "openvpn",
    "tap-",
    "tun",
    "sangfor",
    "bluetooth",
    "wi-fi direct",
    "本地连接*",
    "native wifi",
    "loopback",
    "pseudo",
    "isatap",
    "teredo",
    "microsoft kernel debug",
];

/// 常驻采集器。内部持有 sysinfo 句柄与滚动状态，按 [`Collector::tick`] 产出快照。
pub struct Collector {
    system: System,
    disks: Disks,
    networks: Networks,
    net_prev: Option<(u64, u64, Instant)>,
    primary_net: Option<String>,
    cpu_stress: CpuStressTracker,
    cpu_peak_window: Vec<f32>,
    thresholds: ScoreThresholds,
    primed: bool,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        // 裁剪 RefreshKind：W0 实测 System::new_all() 冷启动 1.8s，绝不能在 UI 线程做
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(sysinfo::CpuRefreshKind::everything())
                .with_memory(sysinfo::MemoryRefreshKind::everything()),
        );
        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh(false);
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        Self {
            system,
            disks,
            networks,
            net_prev: None,
            primary_net: None,
            cpu_stress: CpuStressTracker::default(),
            cpu_peak_window: Vec::new(),
            thresholds: ScoreThresholds::default(),
            primed: false,
        }
    }

    /// 预热：多采一帧让 CPU 差分可用。应在后台线程调用。
    pub fn prime(&mut self) {
        self.system.refresh_cpu_all();
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL + std::time::Duration::from_millis(20));
        self.system.refresh_cpu_all();
        self.disks.refresh(false);
        self.networks.refresh(true);
        self.primed = true;
    }

    pub fn is_primed(&self) -> bool {
        self.primed
    }

    /// 采一帧完整快照。
    pub fn tick(&mut self) -> Snapshot {
        let now_ms = now_ms();

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh(false);
        self.networks.refresh(true);

        let device = self.read_device();
        let mut cpu = self.read_cpu();
        let memory = self.read_memory();
        let disk = self.read_disk();
        let network = self.read_network();

        if let Some(c) = &mut cpu {
            if self.cpu_peak_window.len() >= 60 {
                self.cpu_peak_window.remove(0);
            }
            self.cpu_peak_window.push(c.usage);
            let peak = self.cpu_peak_window.iter().cloned().fold(0.0f32, f32::max);
            c.peak = Some(peak);
        }

        if let Some(c) = &cpu {
            self.cpu_stress
                .observe(c.usage, now_ms, self.thresholds.cpu_usage_high_pct);
        }

        // 厂商温度/风扇：2.5s 缓存；非联想机器返回 None
        let thermal = crate::thermal::read_lenovo();
        if let (Some(cpu_snap), Some(th)) = (&mut cpu, &thermal) {
            cpu_snap.temp_c = th.cpu_temp_c;
        }

        let health = score::score(
            cpu.as_ref(),
            memory.as_ref(),
            disk.as_ref(),
            &self.cpu_stress,
            now_ms,
            &self.thresholds,
        );

        Snapshot {
            ts_ms: now_ms,
            device,
            health: Some(health),
            cpu,
            memory,
            network,
            disk,
            thermal,
        }
    }

    fn read_device(&self) -> Option<DeviceSnapshot> {
        let cpu_brand = self
            .system
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".into());

        Some(DeviceSnapshot {
            host: System::host_name().unwrap_or_else(|| "this-pc".into()),
            os: System::name().unwrap_or_else(|| "Windows".into()),
            os_version: System::os_version()
                .or_else(|| System::kernel_version())
                .unwrap_or_default(),
            cpu_brand,
            logical_cores: self.system.cpus().len(),
            total_memory_bytes: self.system.total_memory(),
            uptime_secs: System::uptime(),
        })
    }

    fn read_cpu(&self) -> Option<CpuSnapshot> {
        if self.system.cpus().is_empty() {
            return None;
        }
        let usage = validity::clamp_pct(self.system.global_cpu_usage())?;
        let per_core: Vec<f32> = self
            .system
            .cpus()
            .iter()
            .filter_map(|c| validity::clamp_pct(c.cpu_usage()))
            .collect();
        if per_core.is_empty() {
            return None;
        }
        Some(CpuSnapshot {
            usage,
            per_core,
            queue_length: None,
            peak: None,
            temp_c: None,
        })
    }

    fn read_memory(&self) -> Option<MemorySnapshot> {
        let total = self.system.total_memory();
        if total == 0 {
            return None;
        }
        let used = self.system.used_memory();
        let usage = validity::clamp_pct(used as f32 / total as f32 * 100.0)?;

        // sysinfo 在 Windows 上把 GlobalMemoryStatusEx 的 PageFile 字段映射到 swap。
        // W0：这对应"提交内存"语义（10.2GB），不是 pagefile.sys 实际占用（295MB）。
        // UI 文案写"已提交"，不写"交换"。
        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();
        let (committed, commit_limit) = if swap_total > 0 && swap_used > 0 {
            let charge = used.saturating_add(swap_used);
            let limit = total.saturating_add(swap_total).max(charge);
            (Some(charge), Some(limit))
        } else {
            (None, None)
        };

        Some(MemorySnapshot {
            usage,
            total_bytes: total,
            used_bytes: used,
            committed_bytes: committed,
            committed_limit_bytes: commit_limit,
        })
    }

    fn read_disk(&self) -> Option<DiskSnapshot> {
        let sys_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        let want = sys_drive.trim_end_matches('\\').to_ascii_lowercase();

        let disk = self.disks.iter().find(|d| {
            let m = d.mount_point().to_string_lossy();
            let m = m.trim_end_matches('\\').to_ascii_lowercase();
            m == want
        })?;

        let total = disk.total_space();
        if total == 0 {
            return None;
        }
        let available = disk.available_space();
        let used = total.saturating_sub(available);

        Some(DiskSnapshot {
            system_drive: DriveSnapshot {
                letter: format!("{}\\", sys_drive.trim_end_matches('\\')),
                total_bytes: total,
                used_bytes: used,
            },
        })
    }

    fn read_network(&mut self) -> Option<NetworkSnapshot> {
        // 已选主接口且仍在列表中则沿用，避免每帧重选导致数字跳动
        if let Some(p) = &self.primary_net {
            if !self.networks.iter().any(|(n, _)| n == p) {
                self.primary_net = None;
            }
        }
        if self.primary_net.is_none() {
            let mut best: Option<(String, u64)> = None;
            for (name, data) in self.networks.iter() {
                if is_blacklisted(name) {
                    continue;
                }
                let total = data.total_received().saturating_add(data.total_transmitted());
                match &best {
                    Some((_, t)) if *t >= total => {}
                    _ => best = Some((name.clone(), total)),
                }
            }
            self.primary_net = best.map(|(n, _)| n);
        }

        let name = self.primary_net.clone()?;
        let data: &NetworkData = self.networks.get(&name)?;
        let rx = data.total_received();
        let tx = data.total_transmitted();
        let now = Instant::now();
        let (down_bps, up_bps) = recompute_rate(&mut self.net_prev, rx, tx, now);

        Some(NetworkSnapshot {
            // v0.1：sysinfo 不暴露 ifIndex，先用 0 占位；W2.1 换 IP Helper
            if_index: 0,
            friendly_name: name,
            download_bps: down_bps,
            upload_bps: up_bps,
        })
    }
}

/// 网速差分。间隔不足 150ms 时不更新速率（并保留旧 prev），避免除零/抖动。
fn recompute_rate(
    prev: &mut Option<(u64, u64, Instant)>,
    rx: u64,
    tx: u64,
    now: Instant,
) -> (u64, u64) {
    match prev.take() {
        None => {
            *prev = Some((rx, tx, now));
            (0, 0)
        }
        Some((prx, ptx, pt)) => {
            let dt = pt.elapsed().as_secs_f64();
            if dt < 0.15 {
                *prev = Some((prx, ptx, pt));
                (0, 0)
            } else {
                let down = (rx.saturating_sub(prx) as f64 / dt).round() as u64;
                let up = (tx.saturating_sub(ptx) as f64 / dt).round() as u64;
                *prev = Some((rx, tx, now));
                (down, up)
            }
        }
    }
}

fn is_blacklisted(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    NET_BLACKLIST.iter().any(|b| lower.contains(b))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blacklist_filters_virtual() {
        assert!(is_blacklisted("VMware Network Adapter VMnet8"));
        assert!(is_blacklisted("vEthernet (WSL)"));
        assert!(is_blacklisted("以太网 3-Native WiFi Filter Driver-0000"));
        assert!(!is_blacklisted("以太网 3"));
        assert!(!is_blacklisted("WLAN"));
    }

    #[test]
    fn rate_needs_two_samples() {
        let mut prev = None;
        let t0 = Instant::now();
        assert_eq!(recompute_rate(&mut prev, 0, 0, t0), (0, 0));
        assert_eq!(recompute_rate(&mut prev, 100, 50, t0), (0, 0));
    }
}
