//! 浏览器缓存相关命令

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::Emitter;
use uuid::Uuid;

use crate::commands::shared::{drop_cancel, new_cancel};
use crate::core::browser_cache;
use crate::core::browser_cache::BrowserProfile;
use crate::core::progress;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;

#[tauri::command]
pub fn detect_browsers() -> Vec<BrowserProfile> {
    browser_cache::detect_browsers()
}

/// 启动扫描,返回 { scan_id }
/// 进度通过 `scan-progress` 事件推送,最终结果通过 `browser-cache-result` 事件推送
#[tauri::command]
pub fn scan_browser_cache(
    app: tauri::AppHandle,
    profile_ids: Vec<String>,
) -> Result<serde_json::Value, AppError> {
    let scan_id = Uuid::new_v4().to_string();
    let cancel = new_cancel(&scan_id);
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();
    let cancel_clone: Arc<std::sync::atomic::AtomicBool> = cancel.clone();

    tauri::async_runtime::spawn(async move {
        progress::emit_started(&app_clone, &scan_id_clone, "browser_cache");

        let app_for_block = app_clone.clone();
        let scan_id_for_block = scan_id_clone.clone();

        let res = tauri::async_runtime::spawn_blocking(move || {
            let mut profiles = browser_cache::detect_browsers();
            let wanted: HashSet<String> = profile_ids.into_iter().collect();
            let total = profiles.len().max(1) as f32;
            for (i, p) in profiles.iter_mut().enumerate() {
                if cancel_clone.load(Ordering::Relaxed) {
                    return Err(AppError::Cancelled);
                }
                if wanted.contains(&p.id) {
                    let percent = (i as f32 / total) * 100.0;
                    let current = p
                        .cache_paths
                        .first()
                        .cloned()
                        .unwrap_or_else(|| p.display_name.clone());
                    progress::emit_progress(
                        &app_for_block,
                        &scan_id_for_block,
                        percent,
                        Some(p.browser.as_str().to_string()),
                        Some(current),
                    );
                    browser_cache::scan_profile_size(p)?;
                }
            }
            Ok::<_, AppError>(
                profiles
                    .into_iter()
                    .filter(|p| wanted.contains(&p.id))
                    .collect::<Vec<_>>(),
            )
        })
        .await;

        match res {
            Ok(Ok(profiles)) => {
                let total_bytes: u64 = profiles.iter().map(|p| p.total_bytes).sum();
                progress::emit_finished(&app_clone, &scan_id_clone, total_bytes);
                // 单独 emit 一次结果,前端拿来更新 UI
                let _ = app_clone.emit("browser-cache-result", &profiles);
            }
            Ok(Err(AppError::Cancelled)) => {
                progress::emit_cancelled(&app_clone, &scan_id_clone);
            }
            Ok(Err(e)) => {
                progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
            Err(e) => {
                progress::emit_error(&app_clone, &scan_id_clone, &e.to_string());
            }
        }
        drop_cancel(&scan_id_clone);
    });

    Ok(serde_json::json!({ "scan_id": scan_id }))
}

#[tauri::command]
pub async fn clean_browser_cache(
    cache_paths: Vec<String>,
    to_trash: bool,
) -> Result<CleanSummary, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        browser_cache::clean(&cache_paths, to_trash)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))
}
