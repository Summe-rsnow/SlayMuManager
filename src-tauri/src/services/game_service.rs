use crate::integrations::steam;
use std::path::{Path, PathBuf};

/// 检查目录下是否存在 SlayTheSpire2.exe（用于游戏路径检测）
pub fn contains_game_executable(root: &Path) -> bool {
    root.join("SlayTheSpire2.exe").is_file()
}

/// 检查目录下是否存在 mods/（用于 Mod 管理功能）
pub fn has_mods_dir(root: &Path) -> bool {
    root.join("mods").is_dir()
}

/// 自动发现游戏安装路径
/// 三关检测：Steam 注册表 → Steam 库扫描 → 硬编码路径兜底
/// 返回 (路径, 来源描述)
pub fn auto_detect_game_install() -> Vec<(PathBuf, String)> {
    let mut results: Vec<(PathBuf, String)> = Vec::new();

    // 第1关：Steam 发现（默认路径 + 库扫描）
    for (path, source) in steam::find_game_install() {
        if contains_game_executable(&path) && !results.iter().any(|(p, _)| p == &path) {
            results.push((path, source));
        }
    }

    // 第2关：硬编码常见路径兜底（C-H 盘）
    let drives = ["C", "D", "E", "F", "G", "H"];
    let templates = [
        "{drive}:\\Program Files (x86)\\Steam\\steamapps\\common\\Slay the Spire 2",
        "{drive}:\\Program Files\\Steam\\steamapps\\common\\Slay the Spire 2",
        "{drive}:\\SteamLibrary\\steamapps\\common\\Slay the Spire 2",
        "{drive}:\\Games\\SteamLibrary\\steamapps\\common\\Slay the Spire 2",
        "{drive}:\\Steam\\steamapps\\common\\Slay the Spire 2",
        "{drive}:\\Games\\Slay the Spire 2",
        "{drive}:\\Slay the Spire 2",
    ];

    for drive in &drives {
        for template in &templates {
            let candidate = PathBuf::from(template.replace("{drive}:", &format!("{}:", drive)));
            if contains_game_executable(&candidate) && !results.iter().any(|(p, _)| p == &candidate) {
                results.push((candidate, "Common path".to_string()));
            }
        }
    }

    results
}

/// 获取游戏可执行文件路径
pub fn game_exe_path(root: &Path) -> PathBuf {
    root.join("SlayTheSpire2.exe")
}

/// 获取 Mod 启用目录
pub fn mods_dir(root: &Path) -> PathBuf {
    root.join("mods")
}

/// 获取 Mod 禁用目录
pub fn disabled_dir(root: &Path) -> PathBuf {
    root.join("mods_disabled")
}

/// 获取模组版存档目录
pub fn modded_saves_dir(root: &Path) -> PathBuf {
    root.join("BepInEx").join("Saves")
}

/// 获取存档备份目录
pub fn save_backups_dir(root: &Path) -> PathBuf {
    root.join("BepInEx").join("spm_backups")
}
