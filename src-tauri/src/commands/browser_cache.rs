//! 浏览器缓存相关命令

use crate::core::browser_cache;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::core::browser_cache::BrowserProfile;

#[tauri::command]
pub fn detect_browsers() -> Vec<BrowserProfile> {
    browser_cache::detect_browsers()
}

#[tauri::command]
pub async fn scan_browser_cache(
    profile_ids: Vec<String>,
) -> Result<Vec<BrowserProfile>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut profiles = browser_cache::detect_browsers();
        let wanted: std::collections::HashSet<String> = profile_ids.into_iter().collect();
        for p in profiles.iter_mut() {
            if wanted.contains(&p.id) {
                browser_cache::scan_profile_size(p)?;
            }
        }
        Ok::<_, AppError>(
            profiles
                .into_iter()
                .filter(|p| wanted.contains(&p.id))
                .collect(),
        )
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))?
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
