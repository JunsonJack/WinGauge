//! 1s 采样循环：后台线程跑 Collector，emit `snapshot://tick` 给前端。
//!
//! 推送而非拉取（规划决策 5）：前端绝不做轮询 IPC。
//! 弹窗隐藏时不停采样（决策 6）：重开瞬间有数据，趋势线不断。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::{Emitter, Manager};
use wingauge_core::{Collector, Snapshot};

use crate::PANEL_LABEL;

pub const TICK_EVENT: &str = "snapshot://tick";

/// 托盘"暂停采样"共享标志
#[derive(Clone)]
pub struct SamplerHandle {
    paused: Arc<AtomicBool>,
}

impl SamplerHandle {
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_paused(&self, v: bool) {
        self.paused.store(v, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

impl Default for SamplerHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// 在 setup 里调用。会启动一个后台线程，永不 join。
pub fn start(app: tauri::AppHandle) {
    let handle = SamplerHandle::new();
    app.manage(handle.clone());

    std::thread::Builder::new()
        .name("wingauge-sampler".into())
        .spawn(move || {
            let mut collector = Collector::new();
            collector.prime();

            loop {
                if handle.is_paused() {
                    std::thread::sleep(Duration::from_millis(200));
                    continue;
                }

                let snap: Snapshot = collector.tick();
                let _ = app.emit(TICK_EVENT, &snap);

                // 弹窗不可见时降到 2s 一帧，省 CPU；隐藏时仍在采（决策 6）
                let interval = match app.get_webview_window(PANEL_LABEL) {
                    Some(w) if w.is_visible().unwrap_or(false) => Duration::from_secs(1),
                    Some(_) => Duration::from_secs(2),
                    None => Duration::from_secs(2),
                };
                std::thread::sleep(interval);
            }
        })
        .expect("spawn sampler thread");
}
