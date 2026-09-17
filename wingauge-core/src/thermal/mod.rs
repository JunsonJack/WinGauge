//! 厂商温度/风扇 provider。
//!
//! 通用 ACPI 通路（sysinfo Components / MSAcpi_ThermalZoneTemperature）在 W0 实测为假数据，
//! 一律不采。这里只放厂商扩展位：联想 Legion 首发，其他机型探测失败则整段隐藏。

pub mod lenovo;

pub use lenovo::{read_snapshot as read_lenovo, ThermalSnapshot};
