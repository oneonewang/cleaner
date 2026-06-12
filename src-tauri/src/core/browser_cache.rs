//! 浏览器缓存检测与清理

use std::path::PathBuf;

use crate::core::paths::{dir_exists, expand_env};
use crate::core::size::dir_size;
use crate::error::AppError;
use crate::models::junk_item::CleanSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BrowserKind {
    Chrome,
    Edge,
    Brave,
    Opera,
    Firefox,
}

impl BrowserKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrowserKind::Chrome => "Chrome",
            BrowserKind::Edge => "Edge",
            BrowserKind::Brave => "Brave",
            BrowserKind::Opera => "Opera",
            BrowserKind::Firefox => "Firefox",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserProfile {
    pub id: String,
    pub browser: BrowserKind,
    #[serde(rename = "profile_name")]
    pub profile_name: String,
    #[serde(rename = "display_name")]
    pub display_name: String,
    #[serde(rename = "cache_paths")]
    pub cache_paths: Vec<String>,
    #[serde(rename = "total_bytes")]
    pub total_bytes: u64,
}

fn chromium_browser_profiles(
    browser: BrowserKind,
    user_data_root: &str,
    cache_subdirs: &[&str],
) -> Vec<BrowserProfile> {
    let mut out = Vec::new();
    let root = match expand_env(user_data_root) {
        Ok(p) => p,
        Err(_) => return out,
    };
    if !dir_exists(&root) {
        return out;
    }
    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // 跳过 "System Profile", "Guest Profile" 等
        if name == "System Profile" || name == "Guest Profile" {
            continue;
        }
        // 默认 profile 名是 "Default",其他为 "Profile N"
        let profile_label = if name == "Default" {
            "Default".to_string()
        } else if name.starts_with("Profile ") {
            name.clone()
        } else {
            // 不认识的子目录(可能是 Crashpad 等),跳过
            continue;
        };
        let mut cache_paths = Vec::new();
        for sub in cache_subdirs {
            let p = path.join(sub);
            if dir_exists(&p) {
                cache_paths.push(p.to_string_lossy().to_string());
            }
        }
        if cache_paths.is_empty() {
            continue;
        }
        let id = format!("{}::{}::{}", browser.as_str(), user_data_root, name);
        out.push(BrowserProfile {
            id,
            browser,
            profile_name: name.clone(),
            display_name: if name == "Default" {
                "Default".to_string()
            } else {
                name
            },
            cache_paths,
            total_bytes: 0,
        });
    }
    out
}

fn firefox_profiles() -> Vec<BrowserProfile> {
    let mut out = Vec::new();
    let appdata = match expand_env("%APPDATA%\\Mozilla\\Firefox\\Profiles") {
        Ok(p) => p,
        Err(_) => return out,
    };
    if !dir_exists(&appdata) {
        return out;
    }
    let entries = match std::fs::read_dir(&appdata) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let cache2 = path.join("cache2");
        if !dir_exists(&cache2) {
            continue;
        }
        let id = format!("Firefox::{}", name);
        out.push(BrowserProfile {
            id,
            browser: BrowserKind::Firefox,
            profile_name: name.clone(),
            display_name: name,
            cache_paths: vec![cache2.to_string_lossy().to_string()],
            total_bytes: 0,
        });
    }
    out
}

/// 检测本机已安装的浏览器
pub fn detect_browsers() -> Vec<BrowserProfile> {
    let cache_subdirs = ["Cache", "Code Cache", "GPUCache", "Service Worker\\CacheStorage"];
    let cache_subdirs_vec: Vec<&str> = cache_subdirs.to_vec();

    let mut out = Vec::new();
    out.extend(chromium_browser_profiles(
        BrowserKind::Chrome,
        "%LOCALAPPDATA%\\Google\\Chrome\\User Data",
        &cache_subdirs_vec,
    ));
    out.extend(chromium_browser_profiles(
        BrowserKind::Edge,
        "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data",
        &cache_subdirs_vec,
    ));
    out.extend(chromium_browser_profiles(
        BrowserKind::Brave,
        "%LOCALAPPDATA%\\BraveSoftware\\Brave-Browser\\User Data",
        &cache_subdirs_vec,
    ));
    out.extend(chromium_browser_profiles(
        BrowserKind::Opera,
        "%LOCALAPPDATA%\\Opera Software\\Opera Stable",
        &cache_subdirs_vec,
    ));
    out.extend(firefox_profiles());
    out
}

/// 计算 profile 缓存总大小
pub fn scan_profile_size(profile: &mut BrowserProfile) -> Result<(), AppError> {
    let mut total: u64 = 0;
    for p in &profile.cache_paths {
        let path = PathBuf::from(p);
        if dir_exists(&path) {
            total = total.saturating_add(dir_size(&path).unwrap_or(0));
        }
    }
    profile.total_bytes = total;
    Ok(())
}

/// 清理浏览器缓存
pub fn clean(cache_paths: &[String]) -> CleanSummary {
    crate::core::trash::remove_paths(cache_paths)
}
