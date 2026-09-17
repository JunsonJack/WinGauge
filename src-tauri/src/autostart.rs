//! 开机自启：写 HKCU\...\Run，设置页开关，默认关闭。
//!
//! 用注册表而非启动文件夹，可被任务管理器"启动"页统一管理，用户可自查可自关。

use tauri::App;
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;

const RUN_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "WinGauge";

/// 首次启动时确保自启键指向有效可执行文件；若自启键残留但 exe 已不存在则清理。
/// 已存在且有效则不动 —— 用户在任务管理器里的设置必须被尊重。
pub fn ensure_default_off(_app: &App) -> tauri::Result<()> {
    // W4 设置页接入前的守卫：自启键残留但 exe 已移动/删除时清理，避免死键
    let Ok(Some(exe)) = is_enabled() else {
        return Ok(());
    };
    if std::path::Path::new(&exe).exists() {
        return Ok(());
    }
    set_enabled(false)?;
    Ok(())
}

/// 返回自启命令行（去掉首尾引号）
pub fn is_enabled() -> tauri::Result<Option<String>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(RUN_PATH)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("打开 HKCU Run 失败: {e}")))?;
    Ok(key
        .get_value::<String, _>(VALUE_NAME)
        .ok()
        .map(|v| v.trim_matches('"').to_string()))
}

pub fn set_enabled(enabled: bool) -> tauri::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(RUN_PATH, KEY_WRITE)
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("打开 HKCU Run(写) 失败: {e}")))?;
    if enabled {
        // 引号包路径，含空格时不会被 Windows 拆参
        key.set_value(
            VALUE_NAME,
            &format!("\"{}\"", std::env::current_exe()?.display()),
        )
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("写自启键失败: {e}")))?;
    } else {
        // 键不存在时删除会报错，吞掉
        let _ = key.delete_value(VALUE_NAME);
    }
    Ok(())
}
