//! 注册表扫描、备份、清理

use std::path::PathBuf;
use std::time::SystemTime;

use uuid::Uuid;

use crate::error::AppError;
use crate::models::junk_item::CleanSummary;
use crate::models::registry_issue::{Hive, RegistryIssue, RegistryScope, RiskLevel};

/// winreg 0.52 中的 HKEY 常量位于 crate 根 re-export
#[cfg(windows)]
type WinHkey = winreg::HKEY;
#[cfg(windows)]
const HKEY_LOCAL_MACHINE: WinHkey = winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(windows)]
const HKEY_CURRENT_USER: WinHkey = winreg::enums::HKEY_CURRENT_USER;
#[cfg(windows)]
const HKEY_CLASSES_ROOT: WinHkey = winreg::enums::HKEY_CLASSES_ROOT;
#[cfg(windows)]
const KEY_ALL_ACCESS: u32 = winreg::enums::KEY_ALL_ACCESS;

/// 系统白名单:这些 Run 项即使指向不存在文件也不要清理
const RUN_WHITELIST: &[&str] = &[
    "SecurityHealth",
    "RtkAudUService",
    "OneDrive",
    "iTunesHelper",
    "Spotify",
    "Discord",
    "Steam",
    "EpicGamesLauncher",
];

fn new_id(scope: RegistryScope, hive: Hive, key: &str, value: Option<&str>) -> String {
    let raw = format!("{:?}::{:?}::{}::{:?}", scope, hive, key, value);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, raw.as_bytes()).to_string()
}

/// 扫描单个 hive + key 下的所有字符串值,检查指向的文件是否存在
#[cfg(windows)]
fn scan_run_like(
    hive: Hive,
    key_path: &str,
    scope: RegistryScope,
    out: &mut Vec<RegistryIssue>,
) {
    use winreg::RegKey;

    let hkey = match hive {
        Hive::HKLM => HKEY_LOCAL_MACHINE,
        Hive::HKCU => HKEY_CURRENT_USER,
    };
    let root = RegKey::predef(hkey);
    let key = match root.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return,
    };
    for (name, value) in key.enum_values().flatten() {
        let value_str = value.to_string();
        if value_str.is_empty() {
            continue;
        }
        if let Some(path) = extract_path(&value_str) {
            if !std::path::Path::new(&path).exists() {
                let whitelisted = RUN_WHITELIST
                    .iter()
                    .any(|w| name.eq_ignore_ascii_case(w));
                out.push(RegistryIssue {
                    id: new_id(scope, hive, key_path, Some(&name)),
                    scope,
                    hive,
                    key_path: key_path.to_string(),
                    value_name: Some(name.clone()),
                    value_data: Some(value_str),
                    description: format!("文件不存在: {}", path),
                    risk: if whitelisted {
                        RiskLevel::Low
                    } else {
                        RiskLevel::Medium
                    },
                    whitelisted,
                });
            }
        }
    }
}

/// 从注册表值中提取第一条可能的文件路径
fn extract_path(value: &str) -> Option<String> {
    if value.starts_with("http") || value.starts_with("https") {
        return None;
    }
    if let Some(idx) = value.find(".exe") {
        let path = &value[..idx + 4];
        let trimmed = path.trim_matches('"');
        return Some(trimmed.to_string());
    }
    for ext in &[".lnk", ".bat", ".cmd"] {
        if let Some(idx) = value.find(ext) {
            let path = &value[..idx + ext.len()];
            let trimmed = path.trim_matches('"');
            return Some(trimmed.to_string());
        }
    }
    None
}

// `Hive::to_winreg` 保留为可选便捷方法
impl Hive {
    #[cfg(windows)]
    pub fn to_winreg(&self) -> winreg::HKEY {
        match self {
            Hive::HKLM => HKEY_LOCAL_MACHINE,
            Hive::HKCU => HKEY_CURRENT_USER,
        }
    }
}

pub fn scan(scopes: &[RegistryScope]) -> Result<Vec<RegistryIssue>, AppError> {
    let mut out = Vec::new();
    for s in scopes {
        match s {
            RegistryScope::Run => {
                scan_run_like(
                    Hive::HKLM,
                    r"Software\Microsoft\Windows\CurrentVersion\Run",
                    *s,
                    &mut out,
                );
                scan_run_like(
                    Hive::HKCU,
                    r"Software\Microsoft\Windows\CurrentVersion\Run",
                    *s,
                    &mut out,
                );
            }
            RegistryScope::RunOnce => {
                scan_run_like(
                    Hive::HKLM,
                    r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
                    *s,
                    &mut out,
                );
                scan_run_like(
                    Hive::HKCU,
                    r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
                    *s,
                    &mut out,
                );
            }
            RegistryScope::Uninstall => {
                scan_uninstall(&mut out);
            }
            RegistryScope::Com => {
                scan_com_broken(&mut out);
            }
        }
    }
    Ok(out)
}

#[cfg(windows)]
fn scan_uninstall(out: &mut Vec<RegistryIssue>) {
    use winreg::RegKey;
    let paths = [
        (HKEY_LOCAL_MACHINE, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];
    for (hkey, key_path) in paths {
        let root = RegKey::predef(hkey);
        let key = match root.open_subkey(key_path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub_name in key.enum_keys().flatten() {
            let sub_path = format!("{}\\{}", key_path, sub_name);
            let sub_key = match root.open_subkey(&sub_path) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let display_name: String = sub_key.get_value("DisplayName").unwrap_or_default();
            let install_location: String = sub_key
                .get_value("InstallLocation")
                .unwrap_or_default();
            let uninstall_string: String = sub_key
                .get_value("UninstallString")
                .unwrap_or_default();
            let check_path = if !install_location.is_empty() {
                install_location.clone()
            } else if !uninstall_string.is_empty() {
                extract_path(&uninstall_string).unwrap_or(uninstall_string.clone())
            } else {
                String::new()
            };
            if check_path.is_empty() {
                continue;
            }
            let p = std::path::Path::new(&check_path);
            if !p.exists() {
                let hive = if hkey == HKEY_LOCAL_MACHINE { Hive::HKLM } else { Hive::HKCU };
                out.push(RegistryIssue {
                    id: new_id(RegistryScope::Uninstall, hive, &sub_path, None),
                    scope: RegistryScope::Uninstall,
                    hive,
                    key_path: sub_path,
                    value_name: None,
                    value_data: Some(display_name),
                    description: format!("安装目录不存在: {}", check_path),
                    risk: RiskLevel::High,
                    whitelisted: false,
                });
            }
        }
    }
}

#[cfg(not(windows))]
fn scan_uninstall(_out: &mut Vec<RegistryIssue>) {}

#[cfg(windows)]
fn scan_com_broken(out: &mut Vec<RegistryIssue>) {
    use winreg::RegKey;
    let roots = [
        (HKEY_CLASSES_ROOT, Hive::HKLM),
        (HKEY_CURRENT_USER, Hive::HKCU),
        (HKEY_LOCAL_MACHINE, Hive::HKLM),
    ];
    for (hkey, hive) in roots {
        let root = RegKey::predef(hkey);
        let clsid_key = match root.open_subkey(r"CLSID") {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub_name in clsid_key.enum_keys().flatten() {
            let sub_path = format!(r"CLSID\{}", sub_name);
            for sub_key_name in &["InprocServer32", "LocalServer32"] {
                let full = format!("{}\\{}", sub_path, sub_key_name);
                let k = match root.open_subkey(&full) {
                    Ok(k) => k,
                    Err(_) => continue,
                };
                let default_value: String = k.get_value("").unwrap_or_default();
                if default_value.is_empty() {
                    continue;
                }
                if let Some(path) = extract_path(&default_value) {
                    if !std::path::Path::new(&path).exists() {
                        out.push(RegistryIssue {
                            id: new_id(RegistryScope::Com, hive, &full, None),
                            scope: RegistryScope::Com,
                            hive,
                            key_path: full,
                            value_name: None,
                            value_data: Some(default_value),
                            description: format!("COM 组件文件不存在: {}", path),
                            risk: RiskLevel::High,
                            whitelisted: false,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
fn scan_com_broken(_out: &mut Vec<RegistryIssue>) {}

/// 备份:把指定 issue 的 key/value 导出为 .reg 文件
pub fn backup(issues: &[RegistryIssue]) -> Result<PathBuf, AppError> {
    use std::fmt::Write;
    let backup_root = dirs::data_dir()
        .ok_or_else(|| AppError::Path("no data_dir".into()))?
        .join("oneonecleaner")
        .join("backups");
    std::fs::create_dir_all(&backup_root)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = backup_root.join(format!("registry-backup-{}.reg", now));
    let mut s = String::new();
    s.push_str("Windows Registry Editor Version 5.00\r\n\r\n");
    s.push_str("; oneonecleaner registry backup\r\n");
    s.push_str("; Generated by oneonecleaner\r\n\r\n");
    for issue in issues {
        let hive_str = match issue.hive {
            Hive::HKLM => "HKEY_LOCAL_MACHINE",
            Hive::HKCU => "HKEY_CURRENT_USER",
        };
        let _ = writeln!(s, "[{}\\{}]", hive_str, issue.key_path);
        if let (Some(name), Some(data)) = (&issue.value_name, &issue.value_data) {
            let _ = writeln!(s, "\"{}\"=\"{}\"", name, data.escape_default());
        } else {
            // 整 key 备份:不做删除
        }
        s.push_str("\r\n");
    }
    std::fs::write(&path, s)?;
    Ok(path)
}

/// 清理
#[cfg(windows)]
pub fn clean(issues: &[RegistryIssue]) -> CleanSummary {
    use winreg::RegKey;
    let mut summary = CleanSummary::default();
    for issue in issues {
        let hkey = match issue.hive {
            Hive::HKLM => HKEY_LOCAL_MACHINE,
            Hive::HKCU => HKEY_CURRENT_USER,
        };
        let root = RegKey::predef(hkey);
        let key = match root.open_subkey_with_flags(&issue.key_path, KEY_ALL_ACCESS) {
            Ok(k) => k,
            Err(e) => {
                summary.errors.push(format!("{}: open key: {}", issue.key_path, e));
                continue;
            }
        };
        let result = match &issue.value_name {
            Some(name) => key.delete_value(name),
            None => root.delete_subkey_all(&issue.key_path),
        };
        match result {
            Ok(()) => summary.total_files = summary.total_files.saturating_add(1),
            Err(e) => summary.errors.push(format!("{}: {}", issue.key_path, e)),
        }
    }
    summary
}

#[cfg(not(windows))]
pub fn clean(_issues: &[RegistryIssue]) -> CleanSummary {
    let mut s = CleanSummary::default();
    s.errors.push("registry not supported on this platform".into());
    s
}
