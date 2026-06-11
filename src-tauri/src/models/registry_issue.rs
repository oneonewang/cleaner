use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum RegistryScope {
    Run,
    RunOnce,
    Uninstall,
    Com,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Hive {
    HKLM,
    HKCU,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIssue {
    pub id: String,
    pub scope: RegistryScope,
    pub hive: Hive,
    #[serde(rename = "key_path")]
    pub key_path: String,
    #[serde(rename = "value_name")]
    pub value_name: Option<String>,
    #[serde(rename = "value_data")]
    pub value_data: Option<String>,
    pub description: String,
    pub risk: RiskLevel,
    pub whitelisted: bool,
}
