use serde::{Deserialize, Serialize};

/// 单个待清理的文件或目录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkFile {
    pub path: String,
    pub size: u64,
    #[serde(rename = "is_dir")]
    pub is_dir: bool,
}

/// 系统垃圾类别元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub paths: Vec<String>,
}

/// 类别扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkCategoryResult {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "total_bytes")]
    pub total_bytes: u64,
    #[serde(rename = "file_count")]
    pub file_count: u64,
    pub files: Vec<JunkFile>,
}

/// 待清理项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanItem {
    pub category: String,
    pub paths: Vec<String>,
}

/// 清理结果摘要
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanSummary {
    #[serde(rename = "total_files")]
    pub total_files: u64,
    #[serde(rename = "total_bytes")]
    pub total_bytes: u64,
    pub errors: Vec<String>,
}
