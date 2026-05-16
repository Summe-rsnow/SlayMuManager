use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::domain::task::ActivityLogEntry;
use crate::repositories::settings_repo;

// --- 全局状态 ---

#[derive(Debug)]
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub recent_activity: RwLock<Vec<ActivityLogEntry>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: RwLock::new(settings_repo::load_settings().unwrap_or_default()),
            recent_activity: RwLock::new(Vec::new()),
        }
    }
}

// --- 应用设置 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub game_root_dir: Option<String>,
    #[serde(default = "default_active_profile_name")]
    pub active_profile_name: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub save_auto_sync: bool,
    #[serde(default)]
    pub save_sync_pairs: Vec<SaveSyncPair>,
    pub nexus_api_key: Option<String>,
    #[serde(default)]
    pub nexus_is_premium: bool,
    pub nexus_user_name: Option<String>,
    pub proxy_url: Option<String>,
    #[serde(default = "default_auto_backup_keep_count")]
    pub auto_backup_keep_count: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            game_root_dir: None,
            active_profile_name: default_active_profile_name(),
            locale: default_locale(),
            save_auto_sync: false,
            save_sync_pairs: Vec::new(),
            nexus_api_key: None,
            nexus_is_premium: false,
            nexus_user_name: None,
            proxy_url: None,
            auto_backup_keep_count: default_auto_backup_keep_count(),
        }
    }
}

fn default_active_profile_name() -> String {
    "原版".to_string()
}

fn default_locale() -> String {
    "zh-CN".to_string()
}

fn default_auto_backup_keep_count() -> usize {
    5
}

// --- 存档同步配对 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncPair {
    pub vanilla_slot: u32,
    pub modded_slot: u32,
}
