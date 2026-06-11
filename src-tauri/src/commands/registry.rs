//! 注册表相关命令

use std::path::PathBuf;

use crate::core::registry;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::models::registry_issue::{RegistryIssue, RegistryScope};

#[tauri::command]
pub fn scan_registry(scopes: Vec<RegistryScope>) -> Result<Vec<RegistryIssue>, AppError> {
    registry::scan(&scopes)
}

#[tauri::command]
pub fn backup_registry(issues: Vec<RegistryIssue>) -> Result<String, AppError> {
    let path: PathBuf = registry::backup(&issues)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn clean_registry(issues: Vec<RegistryIssue>) -> CleanSummary {
    registry::clean(&issues)
}
