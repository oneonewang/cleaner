use serde::{Deserialize, Serialize};

/// 扫描进度事件(后端 → 前端)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressEvent {
    Started {
        scan_id: String,
        scan_type: String,
    },
    Progress {
        scan_id: String,
        percent: f32,
        category: Option<String>,
        current_path: Option<String>,
    },
    CategoryDone {
        scan_id: String,
        category: String,
        bytes: u64,
    },
    ItemFound {
        scan_id: String,
        category: String,
        path: String,
        bytes: u64,
    },
    Finished {
        scan_id: String,
        total_bytes: u64,
    },
    Cancelled {
        scan_id: String,
    },
    Error {
        scan_id: String,
        message: String,
    },
}

/// 大文件结果(用于流式事件)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFile {
    pub path: String,
    pub size: u64,
    #[serde(rename = "last_access")]
    pub last_access: u64,
    pub modified: u64,
    #[serde(rename = "is_dir")]
    pub is_dir: bool,
}
