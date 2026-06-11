//! Windows 回收站接口与永久删除

use std::path::Path;

use crate::core::process_util;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;

/// 删除结果
#[derive(Debug, Default, Clone)]
pub struct TrashResult {
    pub moved: u64,
    pub errors: Vec<String>,
}

/// 送入回收站(批量 PowerShell 调用,避免逐个文件 spawn 进程)
#[cfg(windows)]
pub fn send_to_trash(paths: &[String]) -> TrashResult {
    let mut result = TrashResult::default();
    if paths.is_empty() {
        return result;
    }
    // 过滤出存在的路径
    let existing: Vec<&str> = paths
        .iter()
        .map(|s| s.as_str())
        .filter(|p| Path::new(p).exists())
        .collect();
    if existing.is_empty() {
        return result;
    }

    // 用单个 PowerShell 进程批量处理所有文件
    let array_literal = existing
        .iter()
        .map(|p| format!("'{}'", p.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "Add-Type -AssemblyName Microsoft.VisualBasic; \
         $paths = @({}); \
         $ok = 0; $errs = @(); \
         foreach ($p in $paths) {{ \
             try {{ [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($p, 'OnlyErrorDialogs', 'SendToRecycleBin'); $ok++ }} \
             catch {{ $errs += \"$p: $($_.Exception.Message)\" }} \
         }}; \
         Write-Output \"$ok|$($errs -join '||')\"",
        array_literal
    );

    match process_util::run_capture("powershell", &["-NoProfile", "-Command", &script]) {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout);
            let trimmed = s.trim();
            if let Some((ok_str, err_str)) = trimmed.split_once('|') {
                result.moved = ok_str.trim().parse().unwrap_or(0);
                if !err_str.is_empty() {
                    for e in err_str.split("||") {
                        if !e.is_empty() {
                            result.errors.push(e.to_string());
                        }
                    }
                }
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            for p in &existing {
                result.errors.push(format!("{}: {}", p, stderr.trim()));
            }
        }
        Err(e) => {
            for p in &existing {
                result.errors.push(format!("{}: {}", p, e));
            }
        }
    }
    result
}

/// 永久删除(不经过回收站)
pub fn delete_permanently(paths: &[String]) -> TrashResult {
    let mut result = TrashResult::default();
    for p in paths {
        let path = std::path::Path::new(p);
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

/// 包装:根据 to_trash 决定送回收站还是永久删除
pub fn remove_paths(paths: &[String], to_trash: bool) -> CleanSummary {
    let r = if to_trash {
        #[cfg(windows)]
        {
            send_to_trash(paths)
        }
        #[cfg(not(windows))]
        {
            delete_permanently(paths)
        }
    } else {
        delete_permanently(paths)
    };
    CleanSummary {
        total_files: r.moved,
        total_bytes: 0,
        errors: r.errors,
    }
}

/// 清空回收站
pub fn empty_recycle_bin() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        let script = "Clear-RecycleBin -Force -ErrorAction SilentlyContinue";
        let output = process_util::run_capture("powershell", &["-NoProfile", "-Command", script])
            .map_err(|e| AppError::WindowsApi(format!("spawn powershell: {}", e)))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::WindowsApi(format!(
                "empty recycle bin: {}",
                stderr.trim()
            )));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err(AppError::Other("not supported on this platform".into()))
    }
}
