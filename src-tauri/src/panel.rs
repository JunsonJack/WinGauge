//! 悬浮弹窗生命周期：托盘定位、失焦收起、钉住拖动、Esc 关闭、胶囊模式。
//!
//! 关键点（规划第二节）：
//! - 无边框透明置顶、不占任务栏
//! - 失焦自动收起，但钉住态与胶囊态绕过（胶囊是桌面小部件，不收起）
//! - 收起加 150ms 延迟并复查钉住态与聚焦态，避免"点托盘重新打开"与"失焦收起"竞争闪烁（风险 R5）
//! - 关闭时 hide() 而非 destroy()，保留 WebView 状态（风险 R6）

use std::sync::Mutex;

use tauri::{Event, Emitter, Listener, Manager, WebviewWindow};

use crate::PANEL_LABEL;

/// 完整面板 / 胶囊逻辑尺寸（与前端 LogicalSize 常量保持一致，便于文档与后续 Rust 侧 resize）
#[allow(dead_code)]
pub const PANEL_W: u32 = 380;
#[allow(dead_code)]
pub const PANEL_H: u32 = 620;
#[allow(dead_code)]
pub const CAPSULE_W: u32 = 320;
#[allow(dead_code)]
pub const CAPSULE_H: u32 = 48;

/// 钉住 / 胶囊状态。前端通过事件切换。
#[derive(Default)]
pub struct PanelState {
    pinned: Mutex<bool>,
    capsule: Mutex<bool>,
    last_pos: Mutex<Option<(i32, i32)>>,
}

impl PanelState {
    fn is_pinned(&self) -> bool {
        *self.pinned.lock().unwrap()
    }

    fn set_pinned(&self, v: bool) {
        *self.pinned.lock().unwrap() = v;
    }

    pub fn is_capsule(&self) -> bool {
        *self.capsule.lock().unwrap()
    }

    fn set_capsule(&self, v: bool) {
        *self.capsule.lock().unwrap() = v;
    }

    fn remember_pos(&self, x: i32, y: i32) {
        *self.last_pos.lock().unwrap() = Some((x, y));
    }

    fn recalled_pos(&self) -> Option<(i32, i32)> {
        *self.last_pos.lock().unwrap()
    }

    /// 胶囊或钉住 = 失焦不收起
    fn should_keep_on_blur(&self) -> bool {
        self.is_pinned() || self.is_capsule()
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

    // 胶囊模式：尺寸由前端 setSize，这里只记状态并保证置顶
    {
        let w = window.clone();
        window.listen("panel/capsule", move |event: Event| {
            let capsule = event.payload().trim_matches('"') == "true";
            w.state::<PanelState>().set_capsule(capsule);
            // 胶囊常驻桌面，始终置顶
            let _ = w.set_always_on_top(capsule || w.state::<PanelState>().is_pinned());
        });
    }

    // Esc 关闭（钉住/胶囊态下 Esc 只是收起，不取消状态）
    {
        let w = window.clone();
        window.listen("panel/escape", move |_event: Event| {
            let _ = w.hide();
        });
    }

    // 位置记忆挂在 Moved 事件（原生 startDragging 跨屏后仍能记住）
    {
        let w = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Moved(pos) = event {
                if w.state::<PanelState>().is_pinned() || w.state::<PanelState>().is_capsule() {
                    w.state::<PanelState>().remember_pos(pos.x, pos.y);
                }
            }
        });
    }

    // 失焦收起：延迟 150ms 并复查钉住态/胶囊态与聚焦态（R5 竞争防护）
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if w.state::<PanelState>().should_keep_on_blur() {
                return;
            }
            let weak = w.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(150));
                // 延迟期间可能被重新聚焦或钉住/切胶囊，必须复查
                if weak.is_focused().unwrap_or(false) {
                    return;
                }
                if weak.state::<PanelState>().should_keep_on_blur() {
                    return;
                }
                let _ = weak.hide();
            });
        }
    });

    Ok(())
}

/// 托盘单击时调用：已显示则收起，否则按托盘 rect 定位并显示完整面板。
/// 若当前是胶囊，点托盘会展开成完整面板（而不是隐藏胶囊）。
pub fn toggle_at_tray(app: &tauri::AppHandle, tray_rect: tauri::PhysicalRect<i32, u32>) {
    let Some(window) = app.get_webview_window(PANEL_LABEL) else {
        return;
    };

    let state = app.state::<PanelState>();

    if window.is_visible().unwrap_or(false) {
        if state.is_capsule() {
            // 胶囊可见：展开完整面板
            let _ = window.emit("panel://expand-from-capsule", ());
            let _ = window.show();
            let _ = window.set_focus();
            return;
        }
        // 面板可见：收起。失焦延迟回调会自查可见性，不会反复闪。
        let _ = window.hide();
        return;
    }

    // 从胶囊恢复时也走展开事件，保证尺寸正确
    if state.is_capsule() {
        let _ = window.emit("panel://expand-from-capsule", ());
    }

    if !state.is_pinned() && !state.is_capsule() {
        let (x, y) = crate::position_next_to_tray(&window, tray_rect);
        let _ = crate::move_to(&window, x, y);
    }
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit("panel://opened", ());
}
