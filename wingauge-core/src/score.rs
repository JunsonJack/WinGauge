//! 健康度评分：从 100 起扣，取最差若干项之和。每项扣分可在 UI 点开看原因。
//!
//! 阈值见产品规划第四节；v0.1 硬编码默认值，v0.1+ 进 config.toml。

use crate::snapshot::{
    CpuSnapshot, HealthBand, HealthIssue, HealthSnapshot, MemorySnapshot, DiskSnapshot,
};

/// 默认阈值（可配置）。
#[derive(Debug, Clone)]
pub struct ScoreThresholds {
    pub cpu_usage_high_pct: f32,
    pub cpu_sustain_secs: u64,
    pub mem_watch_pct: f32,
    pub mem_critical_pct: f32,
    pub disk_min_free_pct: f32,
    pub commit_watch_pct: f32,
}

impl Default for ScoreThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_high_pct: 90.0,
            cpu_sustain_secs: 60,
            mem_watch_pct: 85.0,
            mem_critical_pct: 95.0,
            disk_min_free_pct: 10.0,
            commit_watch_pct: 90.0,
        }
    }
}

/// 高 CPU 持续时间追踪（简单滑动窗口）。
#[derive(Debug, Default)]
pub struct CpuStressTracker {
    over_since_ms: Option<u64>,
}

impl CpuStressTracker {
    pub fn observe(&mut self, usage: f32, now_ms: u64, high_pct: f32) {
        if usage > high_pct {
            if self.over_since_ms.is_none() {
                self.over_since_ms = Some(now_ms);
            }
        } else {
            self.over_since_ms = None;
        }
    }

    pub fn sustained_ms(&self, now_ms: u64) -> u64 {
        self.over_since_ms
            .map(|t| now_ms.saturating_sub(t))
            .unwrap_or(0)
    }
}

pub fn score(
    cpu: Option<&CpuSnapshot>,
    memory: Option<&MemorySnapshot>,
    disk: Option<&DiskSnapshot>,
    tracker: &CpuStressTracker,
    now_ms: u64,
    th: &ScoreThresholds,
) -> HealthSnapshot {
    let mut score: i32 = 100;
    let mut issues = Vec::new();

    if let Some(_cpu) = cpu {
        let sustained = tracker.sustained_ms(now_ms);
        if sustained >= th.cpu_sustain_secs * 1000 {
            let pts = 15;
            score -= pts;
            issues.push(HealthIssue {
                metric: "cpu".into(),
                reason: format!(
                    "CPU 使用率持续 {:.0}s 高于 {:.0}%",
                    sustained / 1000,
                    th.cpu_usage_high_pct
                ),
                points: pts as u32,
            });
        }
    }

    if let Some(mem) = memory {
        if mem.usage > th.mem_critical_pct {
            let pts = 30;
            score -= pts;
            issues.push(HealthIssue {
                metric: "memory".into(),
                reason: format!("内存占用 {:.0}%，接近上限", mem.usage),
                points: pts as u32,
            });
        } else if mem.usage > th.mem_watch_pct {
            let pts = 20;
            score -= pts;
            issues.push(HealthIssue {
                metric: "memory".into(),
                reason: format!("内存占用 {:.0}%，偏高", mem.usage),
                points: pts as u32,
            });
        }

        if let (Some(committed), Some(limit)) =
            (mem.committed_bytes, mem.committed_limit_bytes)
        {
            if limit > 0 {
                let pct = committed as f32 / limit as f32 * 100.0;
                if pct > th.commit_watch_pct {
                    let pts = 15;
                    score -= pts;
                    issues.push(HealthIssue {
                        metric: "commit".into(),
                        reason: format!("已提交内存 {:.0}%，接近提交上限", pct),
                        points: pts as u32,
                    });
                }
            }
        }
    }

    if let Some(disk) = disk {
        let free_pct = 100.0 - disk.system_drive.usage();
        if free_pct < th.disk_min_free_pct {
            let pts = 20;
            score -= pts;
            issues.push(HealthIssue {
                metric: "disk".into(),
                reason: format!(
                    "系统盘 {} 剩余 {:.0}%，空间紧张",
                    disk.system_drive.letter, free_pct
                ),
                points: pts as u32,
            });
        }
    }

    let score = score.clamp(0, 100) as u32;
    let band = band_of(score);
    let summary = if issues.is_empty() {
        "各项指标正常".to_string()
    } else {
        format!("{} 项需关注", issues.len())
    };

    HealthSnapshot {
        score,
        band,
        summary,
        issues,
    }
}

pub fn band_of(score: u32) -> HealthBand {
    match score {
        90..=100 => HealthBand::Excellent,
        70..=89 => HealthBand::Good,
        50..=69 => HealthBand::Watch,
        _ => HealthBand::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{DriveSnapshot, DiskSnapshot, MemorySnapshot, CpuSnapshot};

    fn mem(usage: f32) -> MemorySnapshot {
        MemorySnapshot {
            usage,
            total_bytes: 16 << 30,
            used_bytes: ((16 << 30) as f32 * usage / 100.0) as u64,
            committed_bytes: None,
            committed_limit_bytes: None,
        }
    }

    #[test]
    fn perfect_when_idle() {
        let tracker = CpuStressTracker::default();
        let cpu = CpuSnapshot {
            usage: 10.0,
            per_core: vec![10.0],
            queue_length: None,
            peak: None,
        };
        let h = score(
            Some(&cpu),
            Some(&mem(40.0)),
            Some(&DiskSnapshot {
                system_drive: DriveSnapshot {
                    letter: "C:".into(),
                    total_bytes: 100,
                    used_bytes: 50,
                },
            }),
            &tracker,
            0,
            &ScoreThresholds::default(),
        );
        assert_eq!(h.score, 100);
        assert_eq!(h.band, HealthBand::Excellent);
        assert!(h.issues.is_empty());
    }

    #[test]
    fn memory_critical() {
        let tracker = CpuStressTracker::default();
        let h = score(None, Some(&mem(97.0)), None, &tracker, 0, &ScoreThresholds::default());
        assert_eq!(h.score, 70);
        assert_eq!(h.band, HealthBand::Good);
        assert_eq!(h.issues[0].metric, "memory");
    }
}
