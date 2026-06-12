//! 大文件/旧文件相关命令

use std::sync::Arc;
use uuid::Uuid;

use crate::commands::shared::new_cancel;
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

    tauri::async_runtime::spawn(async move {
        let scan_id_for_task = scan_id_clone.clone();
        let app_for_task = app_clone.clone();
        let res = tauri::async_runtime::spawn_blocking(move || {
            let app_in_block = app_for_task.clone();
            let params = ScanParams {
                scan_id: scan_id_for_task.clone(),
                roots,
                min_size,
                older_than_days,
                cancel: cancel_clone,
            };
            large_files::scan(&app_in_block, params, |_lf: &LargeFile| {})
        })
        .await;

        match res {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                if matches!(e, AppError::Cancelled) {
                    crate::core::progress::emit_cancelled(&app_clone, &scan_id_clone);
                } else {
                    crate::core::progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
                }
            }
            Err(e) => {
                crate::core::progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
        }
        crate::commands::shared::drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub async fn delete_paths(paths: Vec<String>) -> Result<CleanSummary, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        large_files::clean(&paths)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))
}
