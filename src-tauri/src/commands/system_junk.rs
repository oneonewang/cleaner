//! 系统垃圾相关命令

use tauri::Emitter;
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
        // 用 channel 把结果从阻塞线程传回 async 任务
        let (tx, mut rx) = tauri::async_runtime::channel::<Result<Vec<JunkCategoryResult>, AppError>>(8);
        let app_for_block = app_clone.clone();
        let scan_id_for_block = scan_id_clone.clone();
        let cancel_for_block = cancel.clone();

        let scan_handle = tauri::async_runtime::spawn_blocking(move || {
            // 同步执行扫描,通过闭包回调逐个 category emit 进度
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.blocking_send(Err(AppError::Other(format!("rt: {}", e))));
                    return;
                }
            };
            let tx2 = tx.clone();
            let res = rt.block_on(async move {
                system_junk::scan_all(
                    &app_for_block,
                    &scan_id_for_block,
                    cancel_for_block,
                    categories,
                    move |_id, _cat| {
                        // 这里可以 emit 增量进度,目前用 category_done 已经够
                        let _ = tx2;
                    },
                )
                .await
            });
            let _ = tx.blocking_send(res);
        });

        let res = rx.recv().await;
        let _ = scan_handle.await;

        match res {
            Some(Ok(results)) => {
                let total_bytes: u64 = results.iter().map(|r| r.total_bytes).sum();
                crate::core::progress::emit_finished(&app_clone, &scan_id_clone, total_bytes);
                // 关键:把完整结果 emit 给前端
                let _ = app_clone.emit("system-junk-result", &results);
            }
            Some(Err(AppError::Cancelled)) => {
                crate::core::progress::emit_cancelled(&app_clone, &scan_id_clone);
            }
            Some(Err(e)) => {
                crate::core::progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
            None => {
                crate::core::progress::emit_error(
                    &app_clone,
                    &scan_id_clone,
                    "scan channel closed unexpectedly",
                );
            }
        }
        crate::commands::shared::drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub async fn clean_system_junk(
    items: Vec<CleanItem>,
) -> Result<CleanSummary, AppError> {
    let summary = tauri::async_runtime::spawn_blocking(move || {
        system_junk::clean(&items)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))?;
    Ok(summary)
}
