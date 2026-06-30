use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 文件哈希存储：mod_id → { relative_file_path → sha1_hex }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModHashesStore {
    pub hashes: HashMap<String, HashMap<String, String>>,
}

fn data_dir() -> PathBuf {
    let base = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("slaymumanager")
}

fn hashes_path() -> PathBuf {
    data_dir().join("mod_hashes.json")
}

/// 加载所有 mod 的文件哈希
pub fn load_mod_hashes() -> ModHashesStore {
    let path = hashes_path();
    if !path.exists() {
        return ModHashesStore::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ModHashesStore::default(),
    }
}

/// 保存 mod 文件哈希
pub fn save_mod_hashes(store: &ModHashesStore) -> Result<(), AppError> {
    let path = hashes_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// 为指定 mod 添加/更新文件哈希
pub fn set_mod_hashes(mod_id: &str, file_hashes: HashMap<String, String>) -> Result<(), AppError> {
    let mut store = load_mod_hashes();
    store.hashes.insert(mod_id.to_string(), file_hashes);
    save_mod_hashes(&store)
}

/// 删除指定 mod 的文件哈希
pub fn remove_mod_hashes(mod_id: &str) -> Result<(), AppError> {
    let mut store = load_mod_hashes();
    store.hashes.remove(mod_id);
    save_mod_hashes(&store)
}


