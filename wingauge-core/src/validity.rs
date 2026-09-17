//! 合理性校验：探测通过 ≠ 数据可信。
//!
//! W0 实测教训：sysinfo 温度返回 `-0.15°C` 且 `max == temp`，是"有值但错误"的典型。
//! 任何进 UI 的传感器读数都必须先过这里。

/// 读数连续 N 个样本完全不变而同期负载变化超过阈值 → 判 flatline 无效。
pub const FLATLINE_SAMPLES: usize = 300; // 1s 采样 × 5 分钟
pub const FLATLINE_CPU_DELTA: f32 = 30.0;

/// 温度合理区间（摄氏度）。通用通路实测会给出负值。
pub const TEMP_MIN_C: f32 = 5.0;
pub const TEMP_MAX_C: f32 = 110.0;

/// 温度是否可进 UI。
/// 规则（规划决策 2）：物理范围、max≠temp、以及 flatline 由调用方用 [`FlatlineGuard`] 判定。
pub fn temperature_is_plausible(c: f32, max_c: Option<f32>) -> bool {
    if !c.is_finite() || c < TEMP_MIN_C || c > TEMP_MAX_C {
        return false;
    }
    if let Some(max) = max_c {
        // W0：本机 sysinfo 的假值形态就是 max == temp
        if (max - c).abs() < f32::EPSILON {
            return false;
        }
        if max < c {
            return false;
        }
    }
    true
}

/// 连续值完全不变、但 CPU 负载有大幅变化 → 传感器可能挂死/假读。
pub struct FlatlineGuard {
    samples: Vec<f32>,
    cpu_samples: Vec<f32>,
}

impl Default for FlatlineGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl FlatlineGuard {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            cpu_samples: Vec::new(),
        }
    }

    /// 记录一次读数；返回 false 表示应判无效。
    pub fn push(&mut self, value: f32, cpu_usage: f32) -> bool {
        const CAP: usize = 600;
        if self.samples.len() >= CAP {
            self.samples.remove(0);
            self.cpu_samples.remove(0);
        }
        self.samples.push(value);
        self.cpu_samples.push(cpu_usage);
        !self.is_flatlined()
    }

    fn is_flatlined(&self) -> bool {
        if self.samples.len() < FLATLINE_SAMPLES {
            return false;
        }
        let first = self.samples[0];
        if self.samples.iter().any(|s| (*s - first).abs() > f32::EPSILON) {
            return false;
        }
        let cpu_min = self
            .cpu_samples
            .iter()
            .cloned()
            .fold(f32::INFINITY, f32::min);
        let cpu_max = self
            .cpu_samples
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        (cpu_max - cpu_min) > FLATLINE_CPU_DELTA
    }
}

/// 使用率类指标钳制到 0..100，丢弃 NaN/Inf。
pub fn clamp_pct(v: f32) -> Option<f32> {
    if !v.is_finite() {
        return None;
    }
    Some(v.clamp(0.0, 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_sysinfo_fake_temp() {
        // W0 实测假值
        assert!(!temperature_is_plausible(-0.15, Some(-0.15)));
        assert!(!temperature_is_plausible(0.0, None));
        assert!(!temperature_is_plausible(150.0, None));
    }

    #[test]
    fn accepts_realistic_temp() {
        assert!(temperature_is_plausible(51.0, Some(80.0)));
        assert!(temperature_is_plausible(73.0, None));
    }

    #[test]
    fn flatline_needs_window() {
        let mut g = FlatlineGuard::new();
        for i in 0..100 {
            assert!(g.push(50.0, i as f32));
        }
    }

    #[test]
    fn clamp_drops_nan() {
        assert!(clamp_pct(f32::NAN).is_none());
        assert_eq!(clamp_pct(120.0), Some(100.0));
    }
}
