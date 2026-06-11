//! 路径解析与 Windows 常用目录定位

use std::path::{Path, PathBuf};

use crate::core::process_util;
use crate::error::AppError;

/// 展开环境变量 %FOO% 为实际路径
pub fn expand_env(path: &str) -> Result<PathBuf, AppError> {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        // 用 cmd.exe /c "echo %FOO%" 来展开环境变量(简单可靠)
        // CREATE_NO_WINDOW 避免黑色命令行窗口
        let output = process_util::run_capture("cmd", &["/C", &format!("echo {}", path)])
            .map_err(|e| AppError::Path(format!("expand_env: {}", e)))?;
        if !output.status.success() {
            return Err(AppError::Path(format!("expand_env failed: {}", path)));
        }
        let s = String::from_utf8_lossy(&output.stdout);
        let trimmed = s.trim().trim_matches('"').trim();
        let os = OsString::from(trimmed);
        Ok(PathBuf::from(os))
    }
    #[cfg(not(windows))]
    {
        Ok(PathBuf::from(path))
    }
}

/// 取得 Windows 系统盘的根路径(默认 C:\)
pub fn system_drive() -> PathBuf {
    #[cfg(windows)]
    {
        // %SystemDrive% 是标准环境变量,直接展开
        if let Ok(p) = expand_env("%SystemDrive%") {
            return p;
        }
        PathBuf::from("C:\\")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/")
    }
}

/// 探测目录是否存在
pub fn dir_exists(p: &Path) -> bool {
    p.is_dir()
}

/// 探测文件是否存在
pub fn file_exists(p: &Path) -> bool {
    p.is_file()
}

/// 通过 PowerShell SHGetKnownFolderPath 取得 known folder 路径
#[cfg(windows)]
pub fn known_folder_path(folder_id: i32) -> Option<PathBuf> {
    let script = format!(
        "$shell = New-Object -ComObject Shell.Application; \
         $folder = $shell.NameSpace({}); \
         if ($folder) {{ $folder.Self.Path }}",
        folder_id
    );
    let output = process_util::run_capture("powershell", &["-NoProfile", "-Command", &script]).ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout);
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

#[cfg(not(windows))]
pub fn known_folder_path(_folder_id: i32) -> Option<PathBuf> {
    None
}
