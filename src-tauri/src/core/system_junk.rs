//! 系统垃圾扫描与清理

use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::core::paths::{dir_exists, expand_env, system_drive};
use crate::core::size::{dir_size, list_files};
use crate::core::trash;
use crate::error::AppError;
use crate::models::junk_item::{CleanItem, CleanSummary, JunkCategory, JunkCategoryResult};

/// 系统垃圾类别表(硬编码)
pub fn system_junk_categories() -> Vec<JunkCategory> {
    let drive = system_drive();
    let drive_str = drive.to_string_lossy().to_string();
    let system_drive_str = if drive_str.ends_with('\\') {
        drive_str.trim_end_matches('\\').to_string()
    } else {
        drive_str.clone()
    };

    vec![
        JunkCategory {
            id: "user_temp".into(),
            name: "User Temp Files".into(),
            description: Some("%TEMP% 下的临时文件".into()),
            paths: vec!["%TEMP%".into()],
        },
        JunkCategory {
            id: "windows_temp".into(),
            name: "Windows Temp Files".into(),
            description: Some("C:\\Windows\\Temp".into()),
            paths: vec![format!("{}\\Windows\\Temp", system_drive_str)],
        },
        JunkCategory {
            id: "prefetch".into(),
            name: "Prefetch".into(),
            description: Some("Windows 预读取文件".into()),
            paths: vec![format!("{}\\Windows\\Prefetch", system_drive_str)],
        },
        JunkCategory {
            id: "thumbnail_cache".into(),
            name: "Thumbnail Cache".into(),
            description: Some("资源管理器缩略图缓存".into()),
            paths: vec!["%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer".into()],
        },
        JunkCategory {
            id: "wer_reports".into(),
            name: "Windows Error Reports".into(),
            description: Some("Windows 错误报告".into()),
            paths: vec!["%LOCALAPPDATA%\\Microsoft\\Windows\\WER".into()],
        },
        JunkCategory {
            id: "delivery_opt".into(),
            name: "Delivery Optimization".into(),
            description: Some("Windows 更新传递优化缓存".into()),
            paths: vec![format!(
                "{}\\Windows\\SoftwareDistribution\\DeliveryOptimization",
                system_drive_str
            )],
        },
        JunkCategory {
            id: "dxcache".into(),
            name: "DirectX Shader Cache".into(),
            description: Some("D3D 着色器缓存".into()),
            paths: vec!["%LOCALAPPDATA%\\D3DSCache".into()],
        },
        JunkCategory {
            id: "recycle_bin".into(),
            name: "Recycle Bin".into(),
            description: Some("回收站".into()),
            paths: vec![],
        },
    ]
}

/// 异步执行:扫描所有类别
pub async fn scan_all<F>(
    app: &tauri::AppHandle,
    scan_id: &str,
    cancel: Arc<std::sync::atomic::AtomicBool>,
    only: Option<Vec<String>>,
    on_category: F,
) -> Result<Vec<JunkCategoryResult>, AppError>
where
    F: Fn(&str, &JunkCategoryResult) + Send + Sync + 'static,
{
    let categories = system_junk_categories();
    let only_set: Option<std::collections::HashSet<String>> =
        only.map(|v| v.into_iter().collect());

    let mut results: Vec<JunkCategoryResult> = Vec::new();
    let total = categories.len() as f32;
    let on_category = Arc::new(on_category);

    for (idx, cat) in categories.into_iter().enumerate() {
        if let Some(ref set) = only_set {
            if !set.contains(&cat.id) {
                continue;
            }
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }

        let percent = (idx as f32 / total) * 100.0;
        crate::core::progress::emit_progress(
            app,
            scan_id,
            percent,
            Some(cat.id.clone()),
            Some(cat.paths.first().cloned().unwrap_or_default()),
        );

        let result = if cat.id == "recycle_bin" {
            scan_recycle_bin()
        } else {
            scan_category(&cat).await
        };

        match result {
            Ok(r) => {
                crate::core::progress::emit_category_done(
                    app,
                    scan_id,
                    &r.id,
                    r.total_bytes,
                );
                on_category(&r.id, &r);
                results.push(r);
            }
            Err(e) => {
                crate::core::progress::emit_error(app, scan_id, &format!("{}: {}", cat.id, e));
                // 类别扫描失败时也加入占位结果,保证前端可见
                results.push(JunkCategoryResult {
                    id: cat.id,
                    name: cat.name,
                    description: cat.description,
                    total_bytes: 0,
                    file_count: 0,
                    files: vec![],
                });
            }
        }
    }

    let total_bytes = results.iter().map(|r| r.total_bytes).sum();
    crate::core::progress::emit_finished(app, scan_id, total_bytes);
    Ok(results)
}

async fn scan_category(cat: &JunkCategory) -> Result<JunkCategoryResult, AppError> {
    let mut result = JunkCategoryResult {
        id: cat.id.clone(),
        name: cat.name.clone(),
        description: cat.description.clone(),
        total_bytes: 0,
        file_count: 0,
        files: vec![],
    };
    for p in &cat.paths {
        let path = expand_env(p)?;
        if !dir_exists(&path) {
            continue;
        }
        let size = dir_size(&path).unwrap_or(0);
        // 列出文件(最多 200 个示例)
        let files = list_files(&path, 200).unwrap_or_default();
        let file_count: u64 = files
            .iter()
            .filter(|(_, _, is_dir)| !*is_dir)
            .count() as u64;
        result.total_bytes = result.total_bytes.saturating_add(size);
        result.file_count = result.file_count.saturating_add(file_count);
        for (path, size, is_dir) in files {
            result.files.push(crate::models::junk_item::JunkFile {
                path,
                size,
                is_dir,
            });
        }
    }
    Ok(result)
}

#[cfg(windows)]
fn scan_recycle_bin() -> Result<JunkCategoryResult, AppError> {
    use crate::core::process_util;
    // 通过 PowerShell 读取回收站大小
    let script = r#"
        $shell = New-Object -ComObject Shell.Application
        $folder = $shell.NameSpace(0xa)  # ssfBITBUCKET = 10
        $size = 0
        $count = 0
        if ($folder.Items().Count -gt 0) {
            foreach ($item in $folder.Items()) {
                $size += $item.Size
                $count += 1
            }
        }
        Write-Output "$count|$size"
    "#;
    let output = process_util::run_capture("powershell", &["-NoProfile", "-Command", script])
        .map_err(|e| AppError::WindowsApi(format!("spawn powershell: {}", e)))?;
    let s = String::from_utf8_lossy(&output.stdout);
    let trimmed = s.trim();
    let (count, size) = if let Some((c, sz)) = trimmed.split_once('|') {
        (
            c.trim().parse::<u64>().unwrap_or(0),
            sz.trim().parse::<u64>().unwrap_or(0),
        )
    } else {
        (0, 0)
    };
    Ok(JunkCategoryResult {
        id: "recycle_bin".into(),
        name: "Recycle Bin".into(),
        description: Some("回收站中的文件".into()),
        total_bytes: size,
        file_count: count,
        files: vec![],
    })
}

#[cfg(not(windows))]
fn scan_recycle_bin() -> Result<JunkCategoryResult, AppError> {
    Ok(JunkCategoryResult {
        id: "recycle_bin".into(),
        name: "Recycle Bin".into(),
        description: Some("回收站".into()),
        total_bytes: 0,
        file_count: 0,
        files: vec![],
    })
}

/// 清理
pub fn clean(items: &[CleanItem], to_trash: bool) -> CleanSummary {
    let mut summary = CleanSummary::default();
    for it in items {
        if it.category == "recycle_bin" {
            #[cfg(windows)]
            {
                if let Err(e) = trash::empty_recycle_bin() {
                    summary.errors.push(format!("recycle_bin: {}", e));
                } else {
                    // 估算
                }
            }
            continue;
        }
        let r = trash::remove_paths(&it.paths, to_trash);
        summary.total_files = summary.total_files.saturating_add(r.total_files);
        summary.total_bytes = summary.total_bytes.saturating_add(r.total_bytes);
        summary.errors.extend(r.errors);
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_includes_common_ones() {
        let cats = system_junk_categories();
        let ids: Vec<&str> = cats.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"user_temp"));
        assert!(ids.contains(&"recycle_bin"));
    }
}
