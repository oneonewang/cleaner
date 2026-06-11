//! 目录大小计算(并行)

use std::path::Path;

use jwalk::WalkDir;

use crate::error::AppError;

/// 计算目录中所有常规文件的总大小(字节)
pub fn dir_size(root: &Path) -> Result<u64, AppError> {
    if !root.exists() {
        return Ok(0);
    }
    let mut total: u64 = 0;
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(meta) = entry.metadata().ok() {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

/// 列出目录内所有文件(返回 path, size, is_dir),最大 max_count 条
pub fn list_files(
    root: &Path,
    max_count: usize,
) -> Result<Vec<(String, u64, bool)>, AppError> {
    let mut out = Vec::with_capacity(max_count.min(1024));
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if out.len() >= max_count {
            break;
        }
        let is_file = entry.file_type().is_file();
        if let Some(meta) = entry.metadata().ok() {
            let path_str = entry.path().to_string_lossy().to_string();
            out.push((path_str, meta.len(), !is_file));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn dir_size_counts_files() {
        let dir = tempdir().unwrap();
        let p1 = dir.path().join("a.txt");
        let p2 = dir.path().join("b.txt");
        fs::write(&p1, vec![0u8; 100]).unwrap();
        fs::write(&p2, vec![0u8; 250]).unwrap();
        let total = dir_size(dir.path()).unwrap();
        assert_eq!(total, 350);
    }
}
