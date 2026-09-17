use tauri::{Emitter, Manager, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewWindow};

mod autostart;
mod panel;
mod sampler;
mod tray;
mod update;

pub const PANEL_LABEL: &str = "panel";

/// 弹窗相对托盘图标的偏移（物理像素）。托盘在右下角，弹窗出现在图标正上方。
const PANEL_OFFSET_X: f64 = 8.0;
const PANEL_OFFSET_Y: f64 = 8.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())
        // 单实例锁：开机自启 + 用户手动双开时只保留第一个实例（验收清单要求）
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // 第二个实例启动时，把已有实例的面板弹出来作为反馈
            if let Some(window) = app.get_webview_window(PANEL_LABEL) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(panel::PanelState::default())
        .invoke_handler(tauri::generate_handler![
            smoke_show_panel,
            get_autostart,
            set_autostart,
            app_version,
            quit_app,
            get_sampler_paused,
            set_sampler_paused,
            check_update,
            open_external
        ])
        .setup(|app| {
            let window = app
                .get_webview_window(PANEL_LABEL)
                .expect("panel window must be declared in tauri.conf.json");
            window.hide()?;

            tray::setup(app)?;
            panel::setup(app, window)?;
            autostart::ensure_default_off(app)?;
            sampler::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_autostart() -> bool {
    autostart::is_enabled().ok().flatten().is_some()
}

#[tauri::command]
fn set_autostart(enabled: bool) -> tauri::Result<()> {
    autostart::set_enabled(enabled)
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 手动检查 AtomGit 最新 Release
#[tauri::command]
async fn check_update() -> update::UpdateCheckResult {
    update::check_update(env!("CARGO_PKG_VERSION").to_string()).await
}

/// 用系统默认浏览器打开 https 链接（下载/Release 页）。
/// 走 explorer 而非 cmd start，少一层 shell，且配合 CREATE_NO_WINDOW 不闪窗。
#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("仅允许打开 http(s) 链接".into());
    }
    std::process::Command::new("explorer")
        .arg(trimmed)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn get_sampler_paused(app: tauri::AppHandle) -> bool {
    app.try_state::<sampler::SamplerHandle>()
        .map(|s| s.is_paused())
        .unwrap_or(false)
}

#[tauri::command]
fn set_sampler_paused(app: tauri::AppHandle, paused: bool) {
    if let Some(s) = app.try_state::<sampler::SamplerHandle>() {
        s.set_paused(paused);
        let _ = app.emit("tray://pause-toggled", paused);
    }
}

/// 冒烟测试命令：模拟"托盘图标在屏幕右下角"的点击，验证定位 + 弹出 + 收起链路。
/// 真实托盘事件走 tray::on_tray_icon_event，此处只在开发期自测用。
#[tauri::command]
fn smoke_show_panel(app: tauri::AppHandle) {
    // 假托盘 rect：屏幕右下角 32x32 物理像素（任务栏右端图标的典型位置）
    let tray_rect = PhysicalRect {
        position: PhysicalPosition::new(2520, 1528),
        size: PhysicalSize::new(32, 32),
    };
    panel::toggle_at_tray(&app, tray_rect);
}

/// 计算弹窗应出现的位置（物理像素）：托盘图标正上方，收敛进该显示器工作区。
pub fn position_next_to_tray(
    window: &WebviewWindow,
    tray_rect: PhysicalRect<i32, u32>,
) -> (i32, i32) {
    // 窗口尺寸：outer_size 在无边框窗口上 == 内尺寸，失败时退回配置里的 380x620
    let win_size = window.outer_size().unwrap_or(PhysicalSize::new(380, 620));
    let win_w = win_size.width as f64;
    let win_h = win_size.height as f64;

    let work_area = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())
        .map(|m| *m.work_area());

    // 托盘图标右边缘对齐弹窗右边，弹窗底边在图标顶上方 OFFSET 处
    let mut x = tray_rect.position.x as f64 + tray_rect.size.width as f64 - win_w - PANEL_OFFSET_X;
    let mut y = tray_rect.position.y as f64 - win_h - PANEL_OFFSET_Y;

    if let Some(area) = work_area {
        // 越界时回弹到工作区内，而不是顶到屏幕边缘
        let max_x = area.position.x as f64 + area.size.width as f64 - win_w;
        let max_y = area.position.y as f64 + area.size.height as f64 - win_h;
        x = x.min(max_x).max(area.position.x as f64);
        y = y.min(max_y).max(area.position.y as f64);
    }

    (x as i32, y as i32)
}

/// 供 panel 模块复用的定位入口（隐藏 PhysicalPosition 构造细节）
pub fn move_to(window: &WebviewWindow, x: i32, y: i32) -> tauri::Result<()> {
    window.set_position(PhysicalPosition::new(x, y))
}
