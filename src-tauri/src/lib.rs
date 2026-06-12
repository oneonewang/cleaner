//! 应用入口与 Tauri 初始化
//!
//! 本应用是 Tauri 2 + Vue 3 + Element Plus 的 Windows 系统清理器。
//! 主要功能在 `core` 与 `commands` 模块中实现,这里负责注册命令和启动。

mod commands;
mod core;
mod error;
mod models;

use crate::error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::show_main_window,
            commands::is_admin,
            commands::relaunch_as_admin,
            commands::system_junk::list_system_junk_categories,
            commands::system_junk::scan_system_junk,
            commands::system_junk::clean_system_junk,
            commands::browser_cache::detect_browsers,
            commands::browser_cache::scan_browser_cache,
            commands::browser_cache::clean_browser_cache,
            commands::large_files::scan_large_files,
            commands::large_files::delete_paths,
            commands::registry::scan_registry,
            commands::registry::backup_registry,
            commands::registry::clean_registry,
            commands::cancel_scan,
            commands::reveal_in_explorer,
            commands::open_path,
        ])
        .setup(|_app| {
            // 主窗口在配置中 visible: false,等待前端 Vue 挂载完成后通过
            // invoke('show_main_window') 主动通知显示,避免 WebView2 启动期间的白屏
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 供前端使用的统一错误类型别名
pub type AppResult<T> = std::result::Result<T, AppError>;
