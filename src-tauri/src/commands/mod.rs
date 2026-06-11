//! Tauri 命令处理器(#[tauri::command] 入口)

pub mod browser_cache;
pub mod large_files;
pub mod registry;
pub mod system_junk;
pub mod shared;

use crate::core::CANCEL_REGISTRY;
use crate::error::AppError;

#[derive(serde::Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    #[serde(rename = "tauri_version")]
    pub tauri_version: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "oneonecleaner",
        version: env!("CARGO_PKG_VERSION"),
        tauri_version: tauri::VERSION,
    }
}

#[tauri::command]
pub fn cancel_scan(scan_id: String) -> Result<(), AppError> {
    CANCEL_REGISTRY.cancel(&scan_id);
    Ok(())
}

#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use crate::core::process_util;
        // 用 explorer.exe /select,"<path>" 在资源管理器中显示
        let arg = format!("/select,{}", path);
        let _ = process_util::spawn_detached("explorer", &[&arg]);
    }
    Ok(())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use crate::core::process_util;
        let _ = process_util::spawn_detached("cmd", &["/c", "start", "", &path]);
    }
    Ok(())
}
