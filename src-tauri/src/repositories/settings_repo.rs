use crate::app::state::AppSettings;
use std::path::PathBuf;

/// 设置文件路径：$APPDATA/com.slay.mumanager/settings.json
fn settings_path() -> PathBuf {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.slay.mumanager");
    std::fs::create_dir_all(&base).ok();
    base.join("settings.json")
}

pub fn load_settings() -> Option<AppSettings> {
    let path = settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}
