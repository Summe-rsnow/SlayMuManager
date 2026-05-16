use crate::domain::profile::ModProfile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 内置"原版"预设的固定 ID（不可修改/删除）
const BUILTIN_VANILLA_ID: &str = "__builtin__vanilla";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileStore {
    profiles: Vec<ModProfile>,
}

fn profiles_path() -> PathBuf {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.slay.mumanager");
    std::fs::create_dir_all(&base).ok();
    base.join("profiles.json")
}

/// 创建内置"原版"预设（不加载任何 Mod）
fn builtin_vanilla_profile() -> ModProfile {
    ModProfile {
        id: BUILTIN_VANILLA_ID.to_string(),
        name: "原版".to_string(),
        description: Some("不使用任何模组，纯净原版游戏".to_string()),
        mod_ids: vec![],
        created_at: "2025-01-01T00:00:00+08:00".to_string(),
        updated_at: "2025-01-01T00:00:00+08:00".to_string(),
        builtin: true,
    }
}

pub fn load_profiles() -> Vec<ModProfile> {
    let path = profiles_path();

    let mut profiles = if path.exists() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return vec![builtin_vanilla_profile()],
        };
        let store: ProfileStore = serde_json::from_str(&content).unwrap_or(ProfileStore {
            profiles: Vec::new(),
        });
        store.profiles
    } else {
        Vec::new()
    };

    // 确保内置"原版"预设始终存在且排在最前面
    if !profiles.iter().any(|p| p.id == BUILTIN_VANILLA_ID) {
        let vanilla = builtin_vanilla_profile();
        profiles.insert(0, vanilla);
        // 首次注入时持久化
        let _ = save_profiles(&profiles);
    } else {
        // 移到最前面（可能被用户编辑过顺序）
        if let Some(pos) = profiles.iter().position(|p| p.id == BUILTIN_VANILLA_ID) {
            if pos != 0 {
                let vanilla = profiles.remove(pos);
                profiles.insert(0, vanilla);
            }
        }
    }

    profiles
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
