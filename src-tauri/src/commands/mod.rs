//! Tauri 命令处理器(#[tauri::command] 入口)

pub mod browser_cache;
pub mod large_files;
pub mod registry;
pub mod shared;
pub mod system_junk;

use tauri::Manager;

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
pub fn show_main_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| AppError::Other(format!("show: {}", e)))?;
        window
            .set_focus()
            .map_err(|e| AppError::Other(format!("set_focus: {}", e)))?;
    }
    Ok(())
}

/// 当前进程是否以管理员身份运行
#[tauri::command]
pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HANDLE;
        use windows::Win32::Security::{
            GetTokenInformation, TokenElevation, TOKEN_QUERY,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        unsafe {
            let mut token = HANDLE::default();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
                return false;
            }
            let mut elevation: u32 = 0;
            let mut ret_len: u32 = 0;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as _),
                std::mem::size_of::<u32>() as u32,
                &mut ret_len,
            );
            if ok.is_err() {
                return false;
            }
            elevation != 0
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// 以管理员身份重启当前应用,本进程退出
#[tauri::command]
pub fn relaunch_as_admin(app: tauri::AppHandle) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use crate::core::process_util;
        let exe = std::env::current_exe().map_err(|e| AppError::Other(e.to_string()))?;
        let exe_str = exe.to_string_lossy().to_string();
        // 用 PowerShell Start-Process -Verb RunAs 触发 UAC 提权
        let script = format!(
            "Start-Process -FilePath '{}' -Verb RunAs",
            exe_str.replace('\'', "''")
        );
        let output = process_util::run_capture("powershell", &["-NoProfile", "-Command", &script])
            .map_err(|e| AppError::Other(format!("spawn powershell: {}", e)))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Other(format!("elevate: {}", stderr.trim())));
        }
        // 退出当前(非提权)实例
        app.exit(0);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(AppError::Other("not supported on this platform".into()))
    }
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

