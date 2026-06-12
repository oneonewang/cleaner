//! 路径解析与 Windows 常用目录定位

use std::path::{Path, PathBuf};

use crate::core::process_util;
use crate::error::AppError;

/// 展开环境变量 %FOO% 为实际路径
pub fn expand_env(path: &str) -> Result<PathBuf, AppError> {
    #[cfg(windows)]
    {
        // 用 cmd.exe /c "echo %FOO%" 来展开环境变量(简单可靠)
        // CREATE_NO_WINDOW 避免黑色命令行窗口
        let output = process_util::run_capture("cmd", &["/C", &format!("echo {}", path)])
            .map_err(|e| AppError::Path(format!("expand_env: {}", e)))?;
        if !output.status.success() {
            return Err(AppError::Path(format!("expand_env failed: {}", path)));
        }
        let s = String::from_utf8_lossy(&output.stdout);
        let trimmed = s.trim().trim_matches('"').trim();
        Ok(PathBuf::from(trimmed))
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
