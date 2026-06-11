//! 注册表相关命令

use std::path::PathBuf;

use crate::core::registry;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::models::registry_issue::{RegistryIssue, RegistryScope};

#[tauri::command]
pub async fn scan_registry(scopes: Vec<RegistryScope>) -> Result<Vec<RegistryIssue>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        registry::scan(&scopes)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))?
}

#[tauri::command]
pub async fn backup_registry(issues: Vec<RegistryIssue>) -> Result<String, AppError> {
    let path: PathBuf = tauri::async_runtime::spawn_blocking(move || {
        registry::backup(&issues)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))??;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn clean_registry(issues: Vec<RegistryIssue>) -> Result<CleanSummary, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        registry::clean(&issues)
    })
    .await
    .map_err(|e| AppError::Other(format!("join error: {}", e)))
}
