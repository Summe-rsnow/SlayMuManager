use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlot {
    pub steam_user_id: String,
    pub kind: SaveKind,
    pub slot_index: u32,
    pub path: String,
    pub has_data: bool,
    pub has_current_run: bool,
    pub file_count: u32,
    pub last_modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveKind {
    Vanilla,
    Modded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSlotRef {
    pub steam_user_id: String,
    pub kind: SaveKind,
    pub slot_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTransferPreview {
    pub source: SaveSlotRef,
    pub target: SaveSlotRef,
    pub source_has_data: bool,
    pub target_has_data: bool,
    pub backup_will_be_created: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackupEntry {
    pub id: String,
    pub steam_user_id: String,
    pub kind: SaveKind,
    pub slot_index: u32,
    pub backup_path: String,
    pub created_at: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncResult {
    pub synced_count: u32,
    pub details: Vec<SyncDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDetail {
    pub slot_index: u32,
    pub direction: SyncDirection,
    pub backup_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    VanillaToModded,
    ModdedToVanilla,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveStatus {
    pub is_available: bool,
    pub cloud_path: Option<String>,
    pub local_path: Option<String>,
    pub has_mismatch: bool,
    pub local_only_count: u32,
    pub cloud_only_count: u32,
    pub different_count: u32,
    pub local_file_count: u32,
    pub cloud_file_count: u32,
    pub local_applied_to_cloud: bool,
    pub cloud_applied_to_local: bool,
    pub diagnostic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSaveDiffEntry {
    pub relative_path: String,
    pub kind: DiffKind,
    pub local_exists: bool,
    pub cloud_exists: bool,
    pub local_size: Option<u64>,
    pub cloud_size: Option<u64>,
    pub local_sha: Option<String>,
    pub cloud_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    InSync,
    Different,
    LocalOnly,
    CloudOnly,
}
