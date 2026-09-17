//! 联想 EC 温度/风扇（W0 第四轮实测通路）。
//!
//! - `root\wmi\LENOVO_FAN_METHOD.Fan_GetCurrentSensorTemperature(SensorID)`
//! - `root\wmi\LENOVO_FAN_METHOD.Fan_GetCurrentFanSpeed(FanID)`
//!
//! 通用 ACPI 是假数据，这里才采真读数。单次 PowerShell 完成扫描/读取，避免逐 ID 起进程。

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// 读数刷新间隔（WMI 有成本，面板 2.5s 足够）
const REFRESH_INTERVAL: Duration = Duration::from_millis(2500);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalSnapshot {
    pub cpu_temp_c: Option<f32>,
    pub gpu_temp_c: Option<f32>,
    pub other_sensors: Vec<OtherSensor>,
    pub fans: Vec<FanReading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherSensor {
    pub id: u8,
    pub temp_c: f32,
    /// UI 显示名：宁可写「传感器 N」也不乱贴 CPU/GPU
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanReading {
    pub id: u8,
    pub rpm: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorRole {
    Cpu,
    Gpu,
    Unknown,
}

#[derive(Debug, Clone)]
struct DiscoveredSensor {
    id: u8,
    role: SensorRole,
}

#[derive(Default)]
struct Cache {
    scanned: bool,
    available: bool,
    sensors: Vec<DiscoveredSensor>,
    fans: Vec<u8>,
    snapshot: Option<ThermalSnapshot>,
    last_read: Option<Instant>,
    /// 扫描/读取失败次数，连续失败后降频放弃
    failures: u32,
}

static CACHE: OnceLock<Arc<Mutex<Cache>>> = OnceLock::new();

fn cache() -> Arc<Mutex<Cache>> {
    CACHE
        .get_or_init(|| Arc::new(Mutex::new(Cache::default())))
        .clone()
}

/// W0 角色先验：3 随负载变（CPU 侧）、4 与 nvidia-smi 一致（GPU）。
/// 其他 ID 标 Unknown，UI 显示「传感器 N」。
fn classify_role(id: u8) -> SensorRole {
    match id {
        3 => SensorRole::Cpu,
        4 => SensorRole::Gpu,
        _ => SensorRole::Unknown,
    }
}

fn plausible_temp(t: f32) -> bool {
    t.is_finite() && (5.0..=110.0).contains(&t)
}

/// 扫描脚本：一次输出 `T,<id>,<celsius>` / `F,<id>,<rpm>` 行。
/// W0 实测：成功时 ReturnValue=True，读数在 CurrentSensorTemperature / CurrentFanSpeed。
fn scan_script() -> &'static str {
    r#"$ErrorActionPreference='SilentlyContinue'
$inst = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD
if (-not $inst) { exit 0 }
foreach ($id in 0..12) {
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentSensorTemperature -Arguments @{SensorID=[byte]$id}
  if ($r -and $r.CurrentSensorTemperature -ne $null) {
    $t = [int]$r.CurrentSensorTemperature
    if ($t -ge 5 -and $t -le 110) { "T,$id,$t" }
  }
}
foreach ($id in 0..3) {
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentFanSpeed -Arguments @{FanID=[byte]$id}
  if ($r -and $r.CurrentFanSpeed -ne $null) {
    $rpm = [int]$r.CurrentFanSpeed
    if ($rpm -ge 0 -and $rpm -le 20000) { "F,$id,$rpm" }
  }
}
"#
}

/// 读已知 ID 脚本
fn read_script(sensor_ids: &[u8], fan_ids: &[u8]) -> String {
    let sensors: Vec<String> = sensor_ids.iter().map(|i| i.to_string()).collect();
    let fans: Vec<String> = fan_ids.iter().map(|i| i.to_string()).collect();
    format!(
        r#"$ErrorActionPreference='SilentlyContinue'
$inst = Get-CimInstance -Namespace root\wmi -ClassName LENOVO_FAN_METHOD
if (-not $inst) {{ exit 0 }}
foreach ($id in @({sensors})) {{
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentSensorTemperature -Arguments @{{SensorID=[byte]$id}}
  if ($r -and $r.CurrentSensorTemperature -ne $null) {{
    "T,$id,$([int]$r.CurrentSensorTemperature)"
  }}
}}
foreach ($id in @({fans})) {{
  $r = Invoke-CimMethod -InputObject $inst -MethodName Fan_GetCurrentFanSpeed -Arguments @{{FanID=[byte]$id}}
  if ($r -and $r.CurrentFanSpeed -ne $null) {{
    "F,$id,$([int]$r.CurrentFanSpeed)"
  }}
}}
"#,
        sensors = sensors.join(","),
        fans = fans.join(",")
    )
}

fn run_ps(script: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;
    // 隐藏控制台：否则每次采温度都会闪出 powershell 黑框
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !out.status.success() && out.stdout.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

fn parse_lines(s: &str) -> (Vec<(u8, f32)>, Vec<(u8, u32)>) {
    let mut temps = Vec::new();
    let mut fans = Vec::new();
    for line in s.lines() {
        let parts: Vec<&str> = line.trim().split(',').collect();
        if parts.len() != 3 {
            continue;
        }
        let Ok(id) = parts[1].parse::<u8>() else {
            continue;
        };
        match parts[0] {
            "T" => {
                if let Ok(t) = parts[2].parse::<f32>() {
                    if plausible_temp(t) {
                        temps.push((id, t));
                    }
                }
            }
            "F" => {
                if let Ok(rpm) = parts[2].parse::<u32>() {
                    if rpm <= 20000 {
                        fans.push((id, rpm));
                    }
                }
            }
            _ => {}
        }
    }
    (temps, fans)
}

/// 首次扫描（幂等）。返回是否本机可用。
pub fn ensure_scanned() -> bool {
    let c = cache();
    {
        let g = c.lock().unwrap();
        if g.scanned {
            return g.available;
        }
    }

    let Some(out) = run_ps(scan_script()) else {
        let mut g = c.lock().unwrap();
        g.scanned = true;
        g.available = false;
        return false;
    };

    let (temps, fans) = parse_lines(&out);
    let mut g = c.lock().unwrap();
    g.scanned = true;
    g.available = !temps.is_empty() || !fans.is_empty();
    g.sensors = temps
        .iter()
        .map(|(id, _)| DiscoveredSensor {
            id: *id,
            role: classify_role(*id),
        })
        .collect();
    g.fans = fans.iter().map(|(id, _)| *id).collect();
    for (id, t) in &temps {
        tracing::info!(target: "thermal_lenovo", "discovered sensor {id} = {t}°C role={:?}", classify_role(*id));
    }
    for (id, rpm) in &fans {
        tracing::info!(target: "thermal_lenovo", "discovered fan {id} = {rpm} RPM");
    }
    g.available
}

/// 读一帧（2.5s 缓存）。非联想或扫描失败 → None（UI 隐藏温度/风扇）。
pub fn read_snapshot() -> Option<ThermalSnapshot> {
    if !ensure_scanned() {
        return None;
    }

    let c = cache();
    {
        let g = c.lock().unwrap();
        if g.failures >= 5 {
            return g.snapshot.clone();
        }
        if let (Some(last), Some(snap)) = (g.last_read, g.snapshot.clone()) {
            if last.elapsed() < REFRESH_INTERVAL {
                return Some(snap);
            }
        }
        // 扫描完成但无传感器
        if g.sensors.is_empty() && g.fans.is_empty() {
            return None;
        }
    }

    let (sensors, fans) = {
        let g = c.lock().unwrap();
        (g.sensors.clone(), g.fans.clone())
    };
    let sensor_ids: Vec<u8> = sensors.iter().map(|s| s.id).collect();
    let fan_ids: Vec<u8> = fans.clone();

    let Some(out) = run_ps(&read_script(&sensor_ids, &fan_ids)) else {
        let mut g = c.lock().unwrap();
        g.failures += 1;
        g.last_read = Some(Instant::now());
        return g.snapshot.clone();
    };

    let (temps, fan_rpms) = parse_lines(&out);
    let mut cpu_temp = None;
    let mut gpu_temp = None;
    let mut other = Vec::new();

    for (id, t) in &temps {
        match classify_role(*id) {
            SensorRole::Cpu => cpu_temp = Some(*t),
            SensorRole::Gpu => gpu_temp = Some(*t),
            SensorRole::Unknown => other.push(OtherSensor {
                id: *id,
                temp_c: *t,
                label: format!("传感器 {id}"),
            }),
        }
    }

    let snap = ThermalSnapshot {
        cpu_temp_c: cpu_temp,
        gpu_temp_c: gpu_temp,
        other_sensors: other,
        fans: fan_rpms
            .into_iter()
            .map(|(id, rpm)| FanReading { id, rpm })
            .collect(),
    };

    let mut g = c.lock().unwrap();
    g.failures = 0;
    g.snapshot = Some(snap.clone());
    g.last_read = Some(Instant::now());
    Some(snap)
}

pub fn scan_report() -> String {
    let ok = ensure_scanned();
    let c = cache();
    let g = c.lock().unwrap();
    format!(
        "available={} sensors={:?} fans={:?} last={:?}",
        ok,
        g.sensors
            .iter()
            .map(|s| (s.id, s.role))
            .collect::<Vec<_>>(),
        g.fans,
        g.snapshot
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scan_lines() {
        let (t, f) = parse_lines("T,3,73\nT,4,51\nF,0,2500\nT,9,999\n");
        assert_eq!(t.len(), 2);
        assert_eq!(f.len(), 1);
        assert_eq!(t[0].0, 3);
        // 999 不合理被丢掉
        assert!(!t.iter().any(|(id, _)| *id == 9));
    }

    #[test]
    fn classify_w0() {
        assert_eq!(classify_role(3), SensorRole::Cpu);
        assert_eq!(classify_role(4), SensorRole::Gpu);
        assert_eq!(classify_role(8), SensorRole::Unknown);
    }

    #[test]
    fn plausible() {
        assert!(!plausible_temp(-0.15));
        assert!(plausible_temp(51.0));
        assert!(!plausible_temp(150.0));
    }
}
