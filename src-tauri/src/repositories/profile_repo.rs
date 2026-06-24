use crate::domain::profile::ModProfile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileStore {
    profiles: Vec<ModProfile>,
}

fn profiles_path() -> PathBuf {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("slaymumanager");
    std::fs::create_dir_all(&base).ok();
    base.join("profiles.json")
}

/// 旧版内置原版预设的固定 ID（用于迁移清理）
const LEGACY_BUILTIN_VANILLA_ID: &str = "__builtin__vanilla";

pub fn load_profiles() -> Vec<ModProfile> {
    let path = profiles_path();

    let profiles = if path.exists() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        let store: ProfileStore = serde_json::from_str(&content).unwrap_or(ProfileStore {
            profiles: Vec::new(),
        });
        store.profiles
    } else {
        Vec::new()
    };

    // 迁移：移除旧版内置原版预设（已替换为原版启动开关）
    let before = profiles.len();
    let filtered: Vec<ModProfile> = profiles
        .into_iter()
        .filter(|p| p.id != LEGACY_BUILTIN_VANILLA_ID)
        .collect();

    if filtered.len() != before {
        // 持久化清理结果
        let _ = save_profiles(&filtered);
    }

    filtered
}

pub fn save_profiles(profiles: &[ModProfile]) -> Result<(), String> {
    let path = profiles_path();
    let store = ProfileStore {
        profiles: profiles.to_vec(),
    };
    let content = serde_json::to_string_pretty(&store).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}
