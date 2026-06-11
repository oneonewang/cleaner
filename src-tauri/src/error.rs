//! 应用统一错误类型,可序列化到前端

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walk error: {0}")]
    Walk(String),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Windows API error: {0}")]
    WindowsApi(String),

    #[error("Scan cancelled")]
    Cancelled,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Other: {0}")]
    Other(String),
}

/// Tauri 要求命令的错误实现 Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        let kind = match self {
            AppError::Io(_) => "io",
            AppError::Walk(_) => "walk",
            AppError::Path(_) => "path",
            AppError::Registry(_) => "registry",
            AppError::WindowsApi(_) => "windows_api",
            AppError::Cancelled => "cancelled",
            AppError::NotFound(_) => "not_found",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::InvalidInput(_) => "invalid_input",
            AppError::Other(_) => "other",
        };
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("AppError", 2)?;
        st.serialize_field("kind", kind)?;
        st.serialize_field("message", &s)?;
        st.end()
    }
}

impl From<jwalk::Error> for AppError {
    fn from(e: jwalk::Error) -> Self {
        AppError::Walk(e.to_string())
    }
}

impl From<walkdir::Error> for AppError {
    fn from(e: walkdir::Error) -> Self {
        AppError::Walk(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Other(e.to_string())
    }
}
