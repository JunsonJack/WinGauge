//! 悬浮弹窗生命周期：托盘定位、失焦收起、钉住拖动、Esc 关闭。
//!
//! 关键点（规划第二节）：
//! - 无边框透明置顶、不占任务栏
//! - 失焦自动收起，但钉住态绕过（否则没法截图）
//! - 收起加 150ms 延迟并复查钉住态与聚焦态，避免"点托盘重新打开"与"失焦收起"竞争闪烁（风险 R5）
//! - 关闭时 hide() 而非 destroy()，保留 WebView 状态（风险 R6）

use std::sync::Mutex;

use tauri::{Event, Emitter, Listener, Manager, WebviewWindow};

use crate::PANEL_LABEL;

/// 钉住状态。前端通过图钉按钮发 panel/pin 事件切换。
#[derive(Default)]
pub struct PanelState {
    pinned: Mutex<bool>,
    last_pos: Mutex<Option<(i32, i32)>>,
}

impl PanelState {
    fn is_pinned(&self) -> bool {
        *self.pinned.lock().unwrap()
    }

    fn set_pinned(&self, v: bool) {
        *self.pinned.lock().unwrap() = v;
    }

    fn remember_pos(&self, x: i32, y: i32) {
        *self.last_pos.lock().unwrap() = Some((x, y));
    }

    fn recalled_pos(&self) -> Option<(i32, i32)> {
        *self.last_pos.lock().unwrap()
    }
}

pub fn setup(app: &tauri::App, window: WebviewWindow) -> tauri::Result<()> {
    // 记住上次钉住位置，重启后回到原位
    if let Some((x, y)) = app.state::<PanelState>().recalled_pos() {
        crate::move_to(&window, x, y)?;
    }

    // Tauri 2.11 的 listen 闭包只收一个 Event；window 引用从闭包外捕获
    {
        let w = window.clone();
        window.listen("panel/pin", move |event: Event| {
            let pinned = event.payload().trim_matches('"') == "true";
            w.state::<PanelState>().set_pinned(pinned);
            // 钉住 = 显式置顶一次，防止被后续窗口压下去
            let _ = w.set_always_on_top(pinned);
        });
    }

    // Esc 关闭（钉住态下 Esc 只是收起，不取消钉住）
    {
        let w = window.clone();
        window.listen("panel/escape", move |_event: Event| {
            let _ = w.hide();
        });
    }

    // 前端拖动：收到的增量位移，由 Rust 侧叠加到窗口当前物理位置。
    // 增量而非绝对坐标，避免多屏 DPI 混合下 screenX 与窗口坐标系换算失真
    let w = window.clone();
    window.listen("panel/drag-delta", move |event: Event| {
        if let Ok((dx, dy)) = serde_json::from_str::<(i32, i32)>(event.payload()) {
            let Ok(pos) = w.outer_position() else {
                return;
            };
            let mut x = pos.x + dx;
            let mut y = pos.y + dy;

            // 约束进工作区，不能拖出屏幕
            if let Ok(Some(monitor)) = w.current_monitor() {
                let area = monitor.work_area();
                let size = w.outer_size().unwrap_or_default();
                let max_x = area.position.x + area.size.width as i32 - size.width as i32;
                let max_y = area.position.y + area.size.height as i32 - size.height as i32;
                x = x.clamp(area.position.x, max_x);
                y = y.clamp(area.position.y, max_y);
            }

            let _ = crate::move_to(&w, x, y);
            w.state::<PanelState>().remember_pos(x, y);
        }
    });

    // 失焦收起：延迟 150ms 并复查钉住态与聚焦态（R5 竞争防护）
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if w.state::<PanelState>().is_pinned() {
                return;
            }
            let weak = w.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(150));
                // 延迟期间可能被重新聚焦或钉住，必须复查
                if weak.is_focused().unwrap_or(false) {
                    return;
                }
                if weak.state::<PanelState>().is_pinned() {
                    return;
                }
                let _ = weak.hide();
            });
        }
    });

    Ok(())
}

/// 托盘单击时调用：已显示则收起，否则按托盘 rect 定位并显示。
pub fn toggle_at_tray(app: &tauri::AppHandle, tray_rect: tauri::PhysicalRect<i32, u32>) {
    let Some(window) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        // 竞争防护：点托盘那一刻窗口仍可见 → 收起。失焦延迟回调会自查可见性，不会反复闪。
        let _ = window.hide();
        return;
    }

    let state = app.state::<PanelState>();
    if !state.is_pinned() {
        let (x, y) = crate::position_next_to_tray(&window, tray_rect);
        let _ = crate::move_to(&window, x, y);
    }
    let _ = window.show();
    let _ = window.set_focus();

    // 通知前端：弹窗打开了（W3 用来触发图表 resize / 滚动位置恢复）
    let _ = window.emit("panel://opened", ());
}
