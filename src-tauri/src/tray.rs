//! 托盘：左键单击在图标正上方弹出，右键出系统菜单。
//!
//! Tauri 2 的 TrayIconEvent::Click 带 rect，可直接拿到图标物理坐标 —— 这是
//! 选 Tauri 而非 Electron 的关键理由（规划第五节）。

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Emitter, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, Position, Rect, Size,
};

use crate::panel;

const MENU_PAUSE: &str = "menu_pause";
const MENU_SETTINGS: &str = "menu_settings";
const MENU_QUIT: &str = "menu_quit";

pub fn setup(app: &App) -> tauri::Result<()> {
    // 全程只在这里创建托盘；不要在 tauri.conf.json 再声明 trayIcon，
    // 否则 Tauri 2 会叠出第二个图标（已踩过）。
    if app.tray_by_id("main").is_some() {
        log::warn!("tray id=main already exists, skip rebuild");
        return Ok(());
    }

    let pause = MenuItem::with_id(app, MENU_PAUSE, "暂停采样", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, MENU_SETTINGS, "设置…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出 WinGauge", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&pause, &settings, &quit])?;

    let tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("WinGauge")
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_QUIT => {
                app.exit(0);
            }
            MENU_PAUSE => {
                if let Some(sampler) = app.try_state::<crate::sampler::SamplerHandle>() {
                    let next = !sampler.is_paused();
                    sampler.set_paused(next);
                    let _ = app.emit("tray://pause-toggled", next);
                    log::info!("sampler paused={next}");
                }
            }
            MENU_SETTINGS => {
                let _ = app.emit("tray://open-settings", ());
            }
            _ => {}
        })
        // 只在左键"按下"那一刻响应；Click 在释放时还会再来一次，不拦截会双触发
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                rect,
                ..
            } = event
            else {
                return;
            };
            if let Some(physical) = rect_to_physical(rect) {
                panel::toggle_at_tray(tray.app_handle(), physical);
            }
        });

    tray.build(app)?;
    Ok(())
}

/// Rect（Position/Size 枚举）→ PhysicalRect。托盘坐标本来就是物理像素，
/// 逻辑分支只是兜底；拿不到物理值时返回 None，本次点击不响应。
fn rect_to_physical(rect: Rect) -> Option<PhysicalRect<i32, u32>> {
    let (x, y) = match rect.position {
        Position::Physical(p) => (p.x, p.y),
        Position::Logical(_) => return None,
    };
    let (w, h) = match rect.size {
        Size::Physical(s) => (s.width, s.height),
        Size::Logical(_) => return None,
    };
    Some(PhysicalRect {
        position: PhysicalPosition::new(x, y),
        size: PhysicalSize::new(w, h),
    })
}
