use crate::domain::task::ActivityLogEntry;
use crate::repositories::settings_repo;
use crate::workflows::update_check::ModUpdateCheckCache;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

// --- 全局状态 ---

#[derive(Debug)]
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub recent_activity: RwLock<Vec<ActivityLogEntry>>,
    pub mod_updates_cache: RwLock<Option<ModUpdateCheckCache>>,
}

impl AppState {
    /// 向活动日志追加一条记录，超过 MAX_LOG_ENTRIES 时自动裁剪旧条目
    pub fn push_activity(&self, entry: ActivityLogEntry) {
        const MAX_LOG_ENTRIES: usize = 500;
        let mut log = self.recent_activity.write().unwrap();
        if log.len() >= MAX_LOG_ENTRIES {
            let excess = log.len() - MAX_LOG_ENTRIES + 1;
            log.drain(0..excess);
        }
        log.push(entry);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: RwLock::new(settings_repo::load_settings().unwrap_or_default()),
            recent_activity: RwLock::new(Vec::new()),
            mod_updates_cache: RwLock::new(
                crate::repositories::updates_cache_repo::load_updates_cache(),
            ),
        }
    }
}

// --- 应用设置 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)] // 缺失字段从 Default::default() 取值
pub struct AppSettings {
    pub game_root_dir: Option<String>,
    pub active_profile_name: String,
    pub locale: String,
    pub save_auto_sync: bool,
    pub save_sync_pairs: Vec<SaveSyncPair>,
    pub nexus_api_key: Option<String>,
    pub nexus_is_premium: bool,
    pub nexus_user_name: Option<String>,
    pub proxy_url: Option<String>,
    pub auto_backup_keep_count: usize,
    pub backup_on_path_switch: bool,
    pub theme_mode: String,
    pub theme_color: String,
    pub launch_mode: String,
    pub launch_check_cloud_save: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            game_root_dir: None,
            active_profile_name: "原版".to_string(),
            locale: "zh-CN".to_string(),
            save_auto_sync: false,
            save_sync_pairs: Vec::new(),
            nexus_api_key: None,
            nexus_is_premium: false,
            nexus_user_name: None,
            proxy_url: None,
            auto_backup_keep_count: 5,
            backup_on_path_switch: true,
            theme_mode: "system".to_string(),
            theme_color: "indigo".to_string(),
            launch_mode: "steam".to_string(),
            launch_check_cloud_save: true,
        }
    }
}

// --- 存档同步配对 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncPair {
    pub vanilla_slot: u32,
    pub modded_slot: u32,
}
