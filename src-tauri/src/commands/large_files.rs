//! 大文件/旧文件相关命令

use std::sync::Arc;
use uuid::Uuid;

use crate::commands::shared::{drop_cancel, new_cancel};
use crate::core::large_files;
use crate::core::large_files::ScanParams;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::models::scan_result::LargeFile;

#[tauri::command]
pub fn scan_large_files(
    app: tauri::AppHandle,
    roots: Vec<String>,
    min_size: u64,
    older_than_days: u32,
) -> Result<serde_json::Value, AppError> {
    let scan_id = Uuid::new_v4().to_string();
    let cancel = new_cancel(&scan_id);
    let app_clone = app.clone();
    let scan_id_clone = scan_id.clone();
    let cancel_clone: Arc<std::sync::atomic::AtomicBool> = cancel.clone();

    std::thread::spawn(move || {
        let params = ScanParams {
            scan_id: scan_id_clone.clone(),
            roots,
            min_size,
            older_than_days,
            cancel: cancel_clone,
        };
        let res = large_files::scan(&app_clone, params, |_lf: &LargeFile| {});
        if let Err(e) = res {
            if matches!(e, AppError::Cancelled) {
                crate::core::progress::emit_cancelled(&app_clone, &scan_id_clone);
            } else {
                crate::core::progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
        }
        drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub fn delete_paths(paths: Vec<String>, to_trash: bool) -> CleanSummary {
    large_files::clean(&paths, to_trash)
}
