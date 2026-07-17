use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsbDrive {
    pub id: String,
    pub device: String,
    pub name: String,
    pub vendor: Option<String>,
    pub size_bytes: u64,
    pub removable: bool,
    pub mount_points: Vec<String>,
    pub filesystem: Option<String>,
    pub partition_scheme: Option<String>,
    pub system: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsoInfo {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub slax_root_found: bool,
    pub edition: Option<String>,
    pub architecture: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    pub erase: bool,
    pub label: String,
    pub persistence_gb: u16,
    pub verify: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
    pub iso_path: String,
    pub drive_id: String,
    pub device: String,
    pub options: InstallOptions,
    pub confirmation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub phase: &'static str,
    pub percent: u8,
    pub title: String,
    pub detail: String,
}

