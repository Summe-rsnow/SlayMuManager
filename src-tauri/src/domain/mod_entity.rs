use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub folder_name: String,
    pub install_dir: String,
    pub manifest_path: Option<String>,
    #[serde(default)]
    pub affects_gameplay: bool,
    pub state: InstalledModState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstalledModState {
    Enabled,
    Disabled,
    UpdateAvailable,
    Conflict,
    Broken,
    Unknown,
}

// --- 切换保护 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGuardInfo {
    pub path_switched: bool,
    pub direction: Option<String>,
    pub had_pairs: bool,
    pub saves_synced: u32,
    pub backups_created: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModToggleResult {
    pub mod_item: InstalledMod,
    pub save_guard: SaveGuardInfo,
}

// --- 批量导入 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveredModStatus {
    Ready,
    Conflict,
    UnsupportedFormat,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveredModSourceType {
    Folder,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredMod {
    pub mod_id: String,
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub folder_name: String,
    pub status: DiscoveredModStatus,
    pub conflicts: Vec<String>,
    pub status_message: Option<String>,
    pub source_archive: Option<String>,
    pub source_type: DiscoveredModSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportPreview {
    pub total_targets_scanned: u32,
    pub discovered_mods: Vec<DiscoveredMod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchInstallResult {
    pub success_count: u32,
    pub failure_count: u32,
    pub results: Vec<BatchInstallItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchInstallItem {
    pub mod_id: String,
    pub name: String,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    Skip,
    Replace,
    Rename,
}
