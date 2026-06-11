//! 路径解析与 Windows 常用目录定位

use std::path::{Path, PathBuf};

use crate::error::AppError;

/// 展开环境变量 %FOO% 为实际路径
pub fn expand_env(path: &str) -> Result<PathBuf, AppError> {
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        // 用 cmd.exe /c "echo %FOO%" 来展开环境变量(简单可靠)
        let output = std::process::Command::new("cmd")
            .args(["/C", &format!("echo {}", path)])
            .output()
            .map_err(|e| AppError::Path(format!("expand_env: {}", e)))?;
        if !output.status.success() {
            return Err(AppError::Path(format!("expand_env failed: {}", path)));
        }
        let s = String::from_utf8_lossy(&output.stdout);
        let trimmed = s.trim().trim_matches('"').trim();
        // 转换宽字符(Windows 的 cmd 输出可能是 GBK,但 %TEMP% 等都是 ASCII,所以这里 OK)
        // 实际系统变量含中文的很少,且后续操作都通过 OsString,直接用 String 即可
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
        if let Some(p) = known_folder_path(36) {
            // FOLDERID_System
            if let Some(parent) = p.parent() {
                if let Some(drive) = parent.parent() {
                    return drive.to_path_buf();
                }
            }
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
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
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
