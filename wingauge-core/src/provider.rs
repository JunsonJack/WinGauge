//! Provider 抽象：每个采集源自己声明"在这台机器上能不能读"。
//!
//! W0 实测教训：探测通过 ≠ 数据可信（sysinfo 温度返回 -0.15°C 的假有效值），
//! 因此能力探测之后还必须过 [`crate::validity`] 合理性校验，二者缺一卡片都不得渲染。

use serde::{Deserialize, Serialize};

/// provider 在当前机器上的可用性。UI 按此决定卡片是渲染还是整卡消失。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// 可读且通过合理性校验
    Available,
    /// 能读但不可全信（如温度角色未交叉验证），降级展示
    Degraded,
    /// 读不到或读数无效 → 卡片整卡隐藏，不显示 0 / N/A 占位
    Unavailable,
}

impl Default for Capability {
    fn default() -> Self {
        Self::Unavailable
    }
}

/// 厂商 provider（联想 EC 温度/风扇等）首次运行须扫描 ID 区间并落盘，
/// 不硬编码 SensorID / FanID（W0 第四轮：硬编码 3/4 的语义无文档）。
pub trait Provider: Send + Sync {
    /// provider 标识，用于配置文件与日志
    fn id(&self) -> &'static str;

    /// 启动时探测一次：能不能在这台机器上读
    fn probe(&self) -> Capability;
}

/// CPU / 内存 / 网络 / 磁盘 —— 通用 sysinfo provider。
/// v0.1 四项均默认 Available（W0 实测 0ms 级可读）；温度/风扇不在其中。
pub struct SysinfoProvider;

impl Provider for SysinfoProvider {
    fn id(&self) -> &'static str {
        "sysinfo"
    }

    fn probe(&self) -> Capability {
        Capability::Available
    }
}

/// 温度：通用通路实测为假数据，永远 Unavailable。
/// 厂商通路（联想 LENOVO_FAN_METHOD）在 W4b 接入，命中前保持不可用。
pub struct GenericThermalProvider;

impl Provider for GenericThermalProvider {
    fn id(&self) -> &'static str {
        "thermal/generic"
    }

    fn probe(&self) -> Capability {
        Capability::Unavailable
    }
}
