use crate::workflows::update_check::ModUpdateCheckCache;
use std::path::PathBuf;

/// 缓存文件路径：{data_dir}/slaymumanager/mod_updates_cache.json
fn cache_path() -> PathBuf {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("slaymumanager");
    std::fs::create_dir_all(&base).ok();
    base.join("mod_updates_cache.json")
}

pub fn load_updates_cache() -> Option<ModUpdateCheckCache> {
    let path = cache_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_updates_cache(cache: &ModUpdateCheckCache) -> Result<(), String> {
    let path = cache_path();
    let content =
        serde_json::to_string_pretty(cache).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

pub fn clear_updates_cache() {
    let path = cache_path();
    let _ = std::fs::remove_file(&path);
}
