//! Windows 回收站接口与永久删除

use std::path::Path;

use crate::error::AppError;
use crate::models::junk_item::CleanSummary;

/// 删除结果
#[derive(Debug, Default, Clone)]
pub struct TrashResult {
    pub moved: u64,
    pub bytes: u64,
    pub errors: Vec<String>,
}

/// 送入回收站:逐文件使用 `cmd /c start /B "" /WAIT ... ` 或 PowerShell 调用
/// 这里为简化,使用 PowerShell 的 `Move-Item -LiteralPath X -Destination $RecycleBin` 形式
#[cfg(windows)]
pub fn send_to_trash(paths: &[String]) -> TrashResult {
    let mut result = TrashResult::default();
    for p in paths {
        if let Err(e) = move_to_recycle_bin(Path::new(p)) {
            result.errors.push(format!("{}: {}", p, e));
        } else {
            result.moved += 1;
        }
    }
    result
}

#[cfg(windows)]
fn move_to_recycle_bin(path: &Path) -> Result<(), AppError> {
    use std::process::Command;
    if !path.exists() {
        return Ok(()); // 已不存在视为成功
    }
    // 使用 PowerShell + Win32 Recycle API
    let path_str = path.to_string_lossy().to_string();
    let script = format!(
        "Add-Type -AssemblyName Microsoft.VisualBasic; \
         [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile('{}', 'OnlyErrorDialogs', 'SendToRecycleBin')",
        path_str.replace('\'', "''")
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| AppError::WindowsApi(format!("spawn powershell: {}", e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::WindowsApi(format!(
            "send to recycle bin: {}",
            stderr.trim()
        )));
    }
    Ok(())
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
        total_bytes: r.bytes,
        errors: r.errors,
    }
}

/// 清空回收站
pub fn empty_recycle_bin() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let script = "Clear-RecycleBin -Force -ErrorAction SilentlyContinue";
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
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
