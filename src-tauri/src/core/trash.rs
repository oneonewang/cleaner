//! 永久删除文件
//!
//! 清理操作直接删除,不走回收站。

use std::path::Path;

use crate::error::AppError;
use crate::models::junk_item::CleanSummary;

/// 删除结果
#[derive(Debug, Default, Clone)]
pub struct TrashResult {
    pub moved: u64,
    pub errors: Vec<String>,
}

/// 永久删除(直接删除,不走回收站)
pub fn delete_permanently(paths: &[String]) -> TrashResult {
    let mut result = TrashResult::default();
    if paths.is_empty() {
        return result;
    }
    let existing: Vec<String> = paths
        .iter()
        .filter(|p| Path::new(p).exists())
        .cloned()
        .collect();
    for p in existing {
        let path = Path::new(&p);
        let res = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        match res {
            Ok(()) => result.moved += 1,
            Err(e) => result.errors.push(format!("{}: {}", p, e)),
        }
    }
    result
}

/// 清理路径(直接删除)
pub fn remove_paths(paths: &[String]) -> CleanSummary {
    let r = delete_permanently(paths);
    CleanSummary {
        total_files: r.moved,
        total_bytes: 0,
        errors: r.errors,
    }
}

/// 清空回收站(用于"回收站"类别的清理)
pub fn empty_recycle_bin() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use crate::core::process_util;
        let script = "Clear-RecycleBin -Force -ErrorAction SilentlyContinue";
        let output = process_util::run_capture("powershell", &["-NoProfile", "-Command", script])
            .map_err(|e| AppError::WindowsApi(format!("spawn powershell: {}", e)))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(AppError::WindowsApi(format!(
                "empty recycle bin: {}",
                stderr.trim()
            )))
        }
    }
    #[cfg(not(windows))]
    {
        Err(AppError::Other("not supported on this platform".into()))
    }
}
