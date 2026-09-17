//! WinGauge 核心库：provider 抽象、采样、合理性校验、健康度评分。
//! 设计原则：不依赖 Tauri，可被 CLI / 其他 shell 复用。

pub mod collector;
pub mod provider;
pub mod score;
pub mod snapshot;
pub mod validity;

pub use collector::Collector;
pub use snapshot::{HealthBand, HealthSnapshot, Snapshot};

/// 库版本，与 workspace package version 同步
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
