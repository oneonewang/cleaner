//! 系统垃圾相关命令

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use uuid::Uuid;

use crate::commands::shared::{drop_cancel, is_cancelled, new_cancel};
use crate::core::system_junk;
use crate::error::AppError;
use crate::models::junk_item::{CleanItem, CleanSummary, JunkCategoryResult};

#[tauri::command]
pub fn list_system_junk_categories() -> Vec<JunkCategoryResult> {
    system_junk::system_junk_categories()
        .into_iter()
        .map(|c| JunkCategoryResult {
            id: c.id,
            name: c.name,
            description: c.description,
            total_bytes: 0,
            file_count: 0,
            files: vec![],
        })
        .collect()
}

#[tauri::command]
pub fn scan_system_junk(
    app: tauri::AppHandle,
    categories: Option<Vec<String>>,
) -> Result<serde_json::Value, AppError> {
    let scan_id = Uuid::new_v4().to_string();
    let cancel = new_cancel(&scan_id);
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();

    std::thread::spawn(move || {
        let scan_id_for_thread = scan_id_clone.clone();
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                crate::core::progress::emit_error(&app_clone, &scan_id_for_thread, &e.to_string());
                return;
            }
        };
        rt.block_on(async move {
            let res = system_junk::scan_all(
                &app_clone,
                &scan_id_for_thread,
                cancel.clone(),
                categories,
                |_, _| {},
            )
            .await;
            if let Err(e) = res {
                if matches!(e, AppError::Cancelled) {
                    crate::core::progress::emit_cancelled(&app_clone, &scan_id_for_thread);
                } else {
                    crate::core::progress::emit_error(&app_clone, &scan_id_for_thread, &e.to_string());
                }
            }
        });
        drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub fn clean_system_junk(items: Vec<CleanItem>, to_trash: bool) -> CleanSummary {
    system_junk::clean(&items, to_trash)
}
