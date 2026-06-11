//! 系统垃圾相关命令

use uuid::Uuid;

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
    let cancel = crate::commands::shared::new_cancel(&scan_id);
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        let res = system_junk::scan_all(
            &app_clone,
            &scan_id_clone,
            cancel.clone(),
            categories,
            |_, _| {},
        )
        .await;
        if let Err(e) = res {
            if matches!(e, AppError::Cancelled) {
                crate::core::progress::emit_cancelled(&app_clone, &scan_id_clone);
            } else {
                crate::core::progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
        }
        crate::commands::shared::drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub async fn clean_system_junk(
    items: Vec<CleanItem>,
    to_trash: bool,
) -> Result<CleanSummary, AppError> {
    // 在 tokio blocking 线程池跑,避免阻塞 Tauri 主循环
    let summary = tauri::async_runtime::spawn_blocking(move || {
        system_junk::clean(&items, to_trash)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))?;
    Ok(summary)
}
