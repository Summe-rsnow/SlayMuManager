use crate::app::state::SaveSyncPair;
use crate::domain::mod_entity::{InstalledMod, InstalledModState, ModToggleResult, SaveGuardInfo};
use crate::integrations::manifest::ModManifest;
use crate::utils::error::AppError;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 扫描
// ---------------------------------------------------------------------------

/// 扫描 mods/ 下所有已启用的 Mod（Manifest 无效的标记为 Broken）
pub fn scan_enabled_mods(game_root: &Path) -> Vec<InstalledMod> {
    let plugins_dir = game_root.join("mods");
    if !plugins_dir.exists() {
        return Vec::new();
    }
    scan_mods_in_dir(&plugins_dir, InstalledModState::Enabled)
}

/// 扫描 mods_disabled/ 下所有已禁用的 Mod
pub fn scan_disabled_mods(game_root: &Path) -> Vec<InstalledMod> {
    let disabled_dir = game_root.join("mods_disabled");
    if !disabled_dir.exists() {
        return Vec::new();
    }
    scan_mods_in_dir(&disabled_dir, InstalledModState::Disabled)
}

/// 在指定目录下扫描所有子文件夹，每个子文件夹当作一个 Mod
fn scan_mods_in_dir(dir: &Path, default_state: InstalledModState) -> Vec<InstalledMod> {
    let mut mods = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return mods,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 多文件名扫描 manifest
        let (manifest_path, manifest): (Option<PathBuf>, Option<ModManifest>) =
            ModManifest::find_in_dir(&path)
                .map(|(p, m)| (Some(p), Some(m)))
                .unwrap_or((None, None));

        // 严格验证：4 个必需字段缺一不可
        let is_valid = manifest.as_ref().map_or(false, |m: &ModManifest| m.is_valid());

        let id = manifest
            .as_ref()
            .and_then(|m: &ModManifest| m.id.clone())
            .filter(|s: &String| !s.trim().is_empty())
            .unwrap_or_else(|| format!("unknown:{}", folder_name));

        let name = manifest
            .as_ref()
            .and_then(|m: &ModManifest| m.name.clone())
            .filter(|s: &String| !s.trim().is_empty())
            .unwrap_or_else(|| folder_name.clone());

        let state: InstalledModState = if is_valid {
            default_state.clone()
        } else if manifest.is_some() {
            // 有 manifest 但不合格 → 标记为 Broken
            InstalledModState::Broken
        } else {
            default_state.clone()
        };

        mods.push(InstalledMod {
            id,
            name,
            version: manifest.as_ref().and_then(|m: &ModManifest| m.version.clone()),
            author: manifest.as_ref().and_then(|m: &ModManifest| m.author.clone()),
            folder_name,
            install_dir: path.to_string_lossy().to_string(),
            manifest_path: manifest_path.map(|p: PathBuf| p.to_string_lossy().to_string()),
            affects_gameplay: manifest
                .as_ref()
                .map(|m: &ModManifest| m.affects_gameplay)
                .unwrap_or(false),
            state,
        });
    }

    // 按名称排序
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    mods
}

// ---------------------------------------------------------------------------
// 启用 / 禁用
// ---------------------------------------------------------------------------

/// 启用 Mod：从 mods_disabled/ 移动到 mods/
pub fn enable_mod(
    game_root: &Path,
    mod_id: &str,
    sync_pairs: &[SaveSyncPair],
    backup_on_switch: bool,
) -> Result<ModToggleResult, AppError> {
    toggle_mod(game_root, mod_id, true, sync_pairs, backup_on_switch)
}

/// 禁用 Mod：从 mods/ 移动到 mods_disabled/
pub fn disable_mod(
    game_root: &Path,
    mod_id: &str,
    sync_pairs: &[SaveSyncPair],
    backup_on_switch: bool,
) -> Result<ModToggleResult, AppError> {
    toggle_mod(game_root, mod_id, false, sync_pairs, backup_on_switch)
}

fn toggle_mod(
    game_root: &Path,
    mod_id: &str,
    enable: bool,
    sync_pairs: &[SaveSyncPair],
    backup_on_switch: bool,
) -> Result<ModToggleResult, AppError> {
    let plugins_dir = game_root.join("mods");
    let disabled_dir = game_root.join("mods_disabled");

    let (source_dir, target_dir, new_state) = if enable {
        (&disabled_dir, &plugins_dir, InstalledModState::Enabled)
    } else {
        (&plugins_dir, &disabled_dir, InstalledModState::Disabled)
    };

    // 在 rename 之前记录 mods/ 是否为空（用于 Save Guard 路径切换检测）
    let plugins_was_empty = std::fs::read_dir(&plugins_dir)
        .map(|mut rd| rd.next().is_none())
        .unwrap_or(true);

    // 查找 source 中的 mod 文件夹
    let mod_folder = find_mod_folder(source_dir, mod_id)?;

    // 检查目标是否已存在同名文件夹
    let folder_name = mod_folder.file_name().unwrap();
    let target_path = target_dir.join(folder_name);
    if target_path.exists() {
        return Err(AppError::Other(format!(
            "目标位置已存在同名文件夹: {}",
            target_path.display()
        )));
    }

    // 确保目标目录存在
    std::fs::create_dir_all(target_dir).map_err(AppError::Io)?;

    // 移动目录（同盘 rename 高效，跨盘回退到 copy+delete）
    if let Err(_e) = std::fs::rename(&mod_folder, &target_path) {
        super::save_service::copy_dir_recursive(&mod_folder, &target_path)?;
        std::fs::remove_dir_all(&mod_folder).map_err(AppError::Io)?;
    }

    // 重新读取移动后的 mod 信息（兼容多种 manifest 文件名）
    let manifest_found = ModManifest::find_in_dir(&target_path);
    let manifest = manifest_found.as_ref().map(|(_, m)| m);
    let final_folder_name = folder_name.to_string_lossy().to_string();

    let id = manifest
        .and_then(|m| m.id.clone())
        .unwrap_or_else(|| format!("unknown:{}", final_folder_name));

    let name = manifest
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| final_folder_name.clone());

    let mod_item = InstalledMod {
        id,
        name,
        version: manifest.and_then(|m| m.version.clone()),
        author: manifest.and_then(|m| m.author.clone()),
        folder_name: final_folder_name,
        install_dir: target_path.to_string_lossy().to_string(),
        manifest_path: manifest_found
            .as_ref()
            .map(|(p, _)| Some(p.to_string_lossy().to_string()))
            .unwrap_or(None),
        affects_gameplay: manifest
            .map(|m| m.affects_gameplay)
            .unwrap_or(false),
        state: new_state,
    };

    // Save Guard：检测 mods/ 空↔非空切换 + 自动同步存档
    let (path_switched, direction): (bool, Option<String>) = if enable {
        // 启用：rename 之前已检查 mods/ 是否为空
        if plugins_was_empty {
            (true, Some("vanilla_to_modded".to_string()))
        } else {
            (false, None)
        }
    } else {
        // 禁用：rename 之后 mods/ 是否变空（mod 已移出）
        let is_now_empty = std::fs::read_dir(&plugins_dir)
            .map(|mut rd| rd.next().is_none())
            .unwrap_or(true);
        if is_now_empty {
            (true, Some("modded_to_vanilla".to_string()))
        } else {
            (false, None)
        }
    };

    let had_pairs = !sync_pairs.is_empty() && backup_on_switch;
    let (saves_synced, backups_created) = if had_pairs && path_switched {
        match super::save_service::sync_saves(game_root, sync_pairs) {
            Ok(result) => (
                result.synced_count as u32,
                result.details.len() as u32,
            ),
            Err(_e) => (0, 0),
        }
    } else {
        (0, 0)
    };

    let save_guard = SaveGuardInfo {
        path_switched,
        direction,
        had_pairs,
        saves_synced,
        backups_created,
        error: None,
    };

    Ok(ModToggleResult { mod_item, save_guard })
}

// ---------------------------------------------------------------------------
// 卸载
// ---------------------------------------------------------------------------

/// 卸载 Mod：删除其文件夹。先在 plugins/ 找，再在 mods_disabled/ 找
pub fn uninstall_mod(game_root: &Path, mod_id: &str) -> Result<(), AppError> {
    let plugins_dir = game_root.join("mods");
    let disabled_dir = game_root.join("mods_disabled");

    let mod_folder = find_mod_folder(&plugins_dir, mod_id)
        .or_else(|_| find_mod_folder(&disabled_dir, mod_id))
        .map_err(|_| AppError::ModNotFound(mod_id.to_string()))?;

    std::fs::remove_dir_all(&mod_folder).map_err(AppError::Io)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------

/// 在 base_dir 下查找匹配 mod_id 的 Mod 文件夹
/// 匹配策略：先按 manifest.json 中的 id 匹配，再按文件夹名匹配
pub fn find_mod_folder(base_dir: &Path, mod_id: &str) -> Result<PathBuf, AppError> {
    let entries = std::fs::read_dir(base_dir).map_err(|_| {
        AppError::ModNotFound(mod_id.to_string())
    })?;

    for entry in entries {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 按 manifest 中的 id 匹配（兼容多种 manifest 文件名）
        if let Some((_, manifest)) = ModManifest::find_in_dir(&path) {
            if manifest.id.as_deref() == Some(mod_id) {
                return Ok(path);
            }
        }

        // 回退：按文件夹名匹配
        if path.file_name().and_then(|n| n.to_str()) == Some(mod_id) {
            return Ok(path);
        }
    }

    Err(AppError::ModNotFound(mod_id.to_string()))
}

/// 获取 Mod 文件夹下的文件列表（相对路径）
pub fn get_mod_files(game_root: &Path, mod_id: &str) -> Result<Vec<String>, AppError> {
    let plugins_dir = game_root.join("mods");
    let disabled_dir = game_root.join("mods_disabled");

    let mod_folder = find_mod_folder(&plugins_dir, mod_id)
        .or_else(|_| find_mod_folder(&disabled_dir, mod_id))
        .map_err(|_| AppError::ModNotFound(mod_id.to_string()))?;

    let mut files = Vec::new();
    collect_relative_paths_inner(&mod_folder, &mod_folder, &mut files);
    Ok(files)
}

fn collect_relative_paths_inner(base: &Path, dir: &Path, out: &mut Vec<String>) {
    collect_relative_paths_inner_depth(base, dir, out, 0)
}

fn collect_relative_paths_inner_depth(base: &Path, dir: &Path, out: &mut Vec<String>, depth: u32) {
    const MAX_DEPTH: u32 = 3;
    if depth > MAX_DEPTH {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(base) {
                let rel_str = rel.to_string_lossy().to_string();
                if !rel_str.is_empty() {
                    out.push(rel_str);
                }
            }
            if path.is_dir() {
                collect_relative_paths_inner_depth(base, &path, out, depth + 1);
            }
        }
    }
}
