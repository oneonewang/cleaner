//! 大文件/旧文件扫描

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::SystemTime;

use jwalk::WalkDir;

use crate::core::progress;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::models::scan_result::LargeFile;

pub struct ScanParams {
    pub scan_id: String,
    pub roots: Vec<String>,
    pub min_size: u64,
    pub older_than_days: u32,
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
}

pub fn scan<F>(app: &tauri::AppHandle, params: ScanParams, on_found: F) -> Result<Vec<LargeFile>, AppError>
where
    F: Fn(&LargeFile) + Send + Sync + 'static,
{
    let ScanParams {
        scan_id,
        roots,
        min_size,
        older_than_days,
        cancel,
    } = params;

    let on_found = Arc::new(on_found);
    let mut results: Vec<LargeFile> = Vec::new();
    let mut total_bytes: u64 = 0;

    progress::emit_started(app, &scan_id, "large_files");

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let age_cutoff_secs = (older_than_days as u64).saturating_mul(86400);

    let total_roots = roots.len();
    for (idx, root) in roots.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            progress::emit_cancelled(app, &scan_id);
            return Err(AppError::Cancelled);
        }
        let root_pb = match crate::core::paths::expand_env(root) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !root_pb.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&root_pb)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if cancel.load(Ordering::Relaxed) {
                progress::emit_cancelled(app, &scan_id);
                return Err(AppError::Cancelled);
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = meta.len();
            if size < min_size {
                continue;
            }
            let modified = meta
                .modified()
                .ok()
                .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let accessed = meta
                .accessed()
                .ok()
                .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            if older_than_days > 0 && now.saturating_sub(accessed) < age_cutoff_secs {
                continue;
            }
            let path_str = entry.path().to_string_lossy().to_string();
            let lf = LargeFile {
                path: path_str.clone(),
                size,
                last_access: accessed,
                modified,
                is_dir: false,
            };
            on_found(&lf);
            // 流式 emit
            let _ = app.emit("large-file-found", &lf);
            results.push(lf);
            total_bytes = total_bytes.saturating_add(size);

            // 进度(每 100 个文件更新一次)
            if results.len() % 100 == 0 {
                let percent = ((idx as f32 + 0.5) / total_roots as f32) * 100.0;
                progress::emit_progress(
                    app,
                    &scan_id,
                    percent,
                    None,
                    Some(path_str),
                );
            }
        }
    }

    progress::emit_finished(app, &scan_id, total_bytes);
    Ok(results)
}

use tauri::Emitter;

/// 清理
pub fn clean(paths: &[String], to_trash: bool) -> CleanSummary {
    crate::core::trash::remove_paths(paths, to_trash)
}

#[allow(dead_code)]
pub fn is_path_excluded(p: &Path) -> bool {
    let s = p.to_string_lossy().to_string();
    // 排除明显的系统目录(整目录清理时)
    let excluded = [
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Program Files (x86)",
        "C:\\ProgramData",
    ];
    for e in excluded {
        if s.eq_ignore_ascii_case(e) {
            return true;
        }
    }
    false
}
