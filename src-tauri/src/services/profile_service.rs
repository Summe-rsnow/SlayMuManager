use crate::domain::profile::{ApplyProfileResult, ModProfile};
use crate::repositories::profile_repo;
use crate::services::mod_service;
use crate::services::save_service;
use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

pub fn list_profiles() -> Vec<ModProfile> {
    profile_repo::load_profiles()
}

pub fn create_profile(
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
) -> Result<ModProfile, AppError> {
    let mut profiles = profile_repo::load_profiles();

    let now = chrono::Utc::now().to_rfc3339();
    let profile = ModProfile {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description,
        mod_ids,
        created_at: now.clone(),
        updated_at: now,
        builtin: false,
    };

    profiles.push(profile.clone());
    profile_repo::save_profiles(&profiles).map_err(|e| AppError::Other(e))?;

    Ok(profile)
}

pub fn update_profile(
    id: String,
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
) -> Result<ModProfile, AppError> {
    let mut profiles = profile_repo::load_profiles();

    let idx = profiles
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| AppError::Other(format!("预设不存在: {}", id)))?;

    // 内置预设不可编辑
    if profiles[idx].builtin {
        return Err(AppError::Other("内置预设不可编辑".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let updated = ModProfile {
        id,
        name,
        description,
        mod_ids,
        created_at: profiles[idx].created_at.clone(),
        updated_at: now,
        builtin: false,
    };

    profiles[idx] = updated.clone();
    profile_repo::save_profiles(&profiles).map_err(|e| AppError::Other(e))?;

    Ok(updated)
}

pub fn delete_profile(id: &str, locale: &str) -> Result<(), AppError> {
    let mut profiles = profile_repo::load_profiles();

    let idx = profiles
        .iter()
        .position(|p| p.id == id)
        .ok_or_else(|| AppError::Other(format!("预设不存在: {}", id)))?;

    profiles.remove(idx);
    profile_repo::save_profiles(&profiles).map_err(|e| AppError::Other(e))?;

    // 如果删除后为空，创建一个默认预设
    if profiles.is_empty() {
        let default = profile_repo::create_default_profile(locale);
        profile_repo::save_profiles(&[default.clone()]).map_err(|e| AppError::Other(e))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 应用预设
// ---------------------------------------------------------------------------

pub fn apply_profile(
    id: &str,
    game_root: &Path,
    sync_pairs: &[crate::app::state::SaveSyncPair],
    backup_on_switch: bool,
) -> Result<ApplyProfileResult, AppError> {
    let profiles = profile_repo::load_profiles();
    let profile = profiles
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or_else(|| AppError::Other(format!("预设不存在: {}", id)))?;

    let enabled = mod_service::scan_enabled_mods(game_root);
    let disabled = mod_service::scan_disabled_mods(game_root);

    let enabled_ids: Vec<String> = enabled.iter().map(|m| m.id.clone()).collect();
    let disabled_ids: Vec<String> = disabled.iter().map(|m| m.id.clone()).collect();

    let mut enabled_list = Vec::new();
    let mut disabled_list = Vec::new();
    let mut missing_list = Vec::new();

    for mod_id in &profile.mod_ids {
        if enabled_ids.contains(mod_id) {
            // 已经在启用状态，不需要动
            enabled_list.push(mod_id.clone());
        } else if disabled_ids.contains(mod_id) {
            // 需要启用
            mod_service::enable_mod(game_root, mod_id, sync_pairs, backup_on_switch)?;
            enabled_list.push(mod_id.clone());
        } else {
            // 未安装
            missing_list.push(mod_id.clone());
        }
    }

    // 禁用不在预设中的已启用 Mod（跳过创意工坊 Mod）
    for m in &enabled {
        if m.source == "workshop" { continue; }
        if !profile.mod_ids.contains(&m.id) {
            mod_service::disable_mod(game_root, &m.id, sync_pairs, backup_on_switch)?;
            disabled_list.push(m.id.clone());
        }
    }

    // 更新 active_profile_name
    let mut settings = crate::repositories::settings_repo::load_settings()
        .unwrap_or_default();
    settings.active_profile_name = profile.name.clone();
    let _ = crate::repositories::settings_repo::save_settings(&settings);

    Ok(ApplyProfileResult {
        profile,
        enabled_mod_ids: enabled_list,
        disabled_mod_ids: disabled_list,
        missing_mod_ids: missing_list,
    })
}

// ---------------------------------------------------------------------------
// 整合包格式 (.spm)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub format: String,
    pub profile: BundleProfileInfo,
    pub mods: Vec<BundleModInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleProfileInfo {
    pub name: String,
    pub description: Option<String>,
    pub mod_ids: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleModInfo {
    pub mod_id: String,
    pub name: String,
    pub version: Option<String>,
    pub folder_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlePreview {
    pub manifest: BundleManifest,
    pub conflicts: Vec<BundleConflict>,
    pub missing_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleConflict {
    pub mod_id: String,
    pub name: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// 导出整合包
// ---------------------------------------------------------------------------

pub fn export_bundle(profile_id: &str, output_path: &str, game_root: &Path) -> Result<String, AppError> {
    let profiles = profile_repo::load_profiles();
    let profile = profiles
        .iter()
        .find(|p| p.id == profile_id)
        .cloned()
        .ok_or_else(|| AppError::Other(format!("预设不存在: {}", profile_id)))?;

    let plugins_dir = game_root.join("mods");
    let temp_dir = std::env::temp_dir()
        .join("slaymumanager")
        .join("bundle")
        .join(uuid::Uuid::new_v4().to_string());

    std::fs::create_dir_all(&temp_dir).map_err(AppError::Io)?;

    let mut bundle_mods = Vec::new();

    // 复制每个 Mod 目录
    for mod_id in &profile.mod_ids {
        match mod_service::find_mod_folder(&plugins_dir, mod_id) {
            Ok(mod_folder) => {
                let folder_name = mod_folder
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let dest = temp_dir.join(&folder_name);
                save_service::copy_dir_recursive(&mod_folder, &dest)?;

                let manifest = crate::integrations::manifest::ModManifest::find_in_dir(&mod_folder)
                    .map(|(_, m)| m);
                bundle_mods.push(BundleModInfo {
                    mod_id: mod_id.clone(),
                    name: manifest
                        .as_ref()
                        .and_then(|m| m.name.clone())
                        .unwrap_or_else(|| folder_name.clone()),
                    version: manifest.as_ref().and_then(|m| m.version.clone()),
                    folder_name,
                });
            }
            Err(_) => {
                // Mod 未安装，不包含在整合包中但仍记录
            }
        }
    }

    // 生成 .spm 文件
    let manifest = BundleManifest {
        format: "spm-1".to_string(),
        profile: BundleProfileInfo {
            name: profile.name.clone(),
            description: profile.description.clone(),
            mod_ids: profile.mod_ids.clone(),
            created_at: profile.created_at.clone(),
        },
        mods: bundle_mods,
    };

    let spm_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| AppError::Other(format!("序列化失败: {}", e)))?;
    std::fs::write(temp_dir.join("bundle.spm"), &spm_json).map_err(AppError::Io)?;

    // 打包为 7z（默认 LZMA，后台执行不卡 UI）
    sevenz_rust::compress_to_path(&temp_dir, Path::new(output_path))
        .map_err(|e| AppError::Other(format!("7z 打包失败: {}", e)))?;

    // 清理临时目录
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(output_path.to_string())
}

// ---------------------------------------------------------------------------
// 导入整合包
// ---------------------------------------------------------------------------

pub fn preview_bundle(bundle_path: &str, game_root: &Path) -> Result<BundlePreview, AppError> {
    // 复用 install_archive_workflow 的统一解压逻辑
    let temp_dir = crate::workflows::install_archive_workflow::extract_archive(Path::new(bundle_path))
        .map_err(|_| AppError::Other("解压整合包失败".to_string()))?;

    // 读取 bundle.spm
    let spm_path = temp_dir.join("bundle.spm");
    let spm_content = std::fs::read_to_string(&spm_path)
        .map_err(|_| AppError::Other("整合包缺少 bundle.spm 文件".to_string()))?;
    let manifest: BundleManifest = serde_json::from_str(&spm_content)
        .map_err(|e| AppError::Other(format!("bundle.spm 解析失败: {}", e)))?;

    // 冲突检测
    let installed = mod_service::scan_enabled_mods(game_root);
    let disabled = mod_service::scan_disabled_mods(game_root);

    let mut conflicts = Vec::new();
    let mut missing_ids = Vec::new();

    for mod_id in &manifest.profile.mod_ids {
        // 检查是否在已安装 Mod 中
        let installed_exists = installed.iter().any(|m| &m.id == mod_id);
        let disabled_exists = disabled.iter().any(|m| &m.id == mod_id);
        // 检查 bundle 中是否有该 mod 的文件
        let in_bundle = manifest.mods.iter().any(|m| &m.mod_id == mod_id);

        if installed_exists || disabled_exists {
            conflicts.push(BundleConflict {
                mod_id: mod_id.clone(),
                name: manifest
                    .mods
                    .iter()
                    .find(|m| &m.mod_id == mod_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| mod_id.clone()),
                reason: "已安装此 Mod，默认跳过".to_string(),
            });
        }

        if !in_bundle {
            missing_ids.push(mod_id.clone());
        }
    }

    // 清理
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(BundlePreview {
        manifest,
        conflicts,
        missing_ids,
    })
}

pub fn import_bundle(
    bundle_path: &str,
    game_root: &Path,
    should_apply: bool,
    resolutions: &[(String, String)], // (modId, "skip"|"replace")
    sync_pairs: &[crate::app::state::SaveSyncPair],
    backup_on_switch: bool,
) -> Result<ApplyProfileResult, AppError> {
    // 复用 install_archive_workflow 的统一解压逻辑
    let temp_dir = crate::workflows::install_archive_workflow::extract_archive(Path::new(bundle_path))
        .map_err(|_| AppError::Other("解压整合包失败".to_string()))?;

    // 读取 manifest
    let spm_path = temp_dir.join("bundle.spm");
    let spm_content = std::fs::read_to_string(&spm_path)
        .map_err(|_| AppError::Other("整合包缺少 bundle.spm 文件".to_string()))?;
    let manifest: BundleManifest = serde_json::from_str(&spm_content)
        .map_err(|e| AppError::Other(format!("bundle.spm 解析失败: {}", e)))?;

    // 构建冲突解析映射
    let skip_ids: Vec<&str> = resolutions
        .iter()
        .filter(|(_, r)| r == "skip")
        .map(|(id, _)| id.as_str())
        .collect();

    let plugins_dir = game_root.join("mods");
    std::fs::create_dir_all(&plugins_dir).map_err(AppError::Io)?;

    // 安装 Mod
    for bundle_mod in &manifest.mods {
        if skip_ids.contains(&bundle_mod.mod_id.as_str()) {
            continue;
        }
        let source = temp_dir.join(&bundle_mod.folder_name);
        if source.is_dir() {
            let dest = plugins_dir.join(&bundle_mod.folder_name);
            if dest.exists() {
                std::fs::remove_dir_all(&dest).map_err(AppError::Io)?;
            }
            save_service::copy_dir_recursive(&source, &dest)?;
        }
    }

    // 保存预设（如果需要）
    if should_apply && !manifest.profile.mod_ids.is_empty() {
        let mut profiles = profile_repo::load_profiles();
        let existing = profiles.iter().position(|p| p.name == manifest.profile.name);
        let now = chrono::Utc::now().to_rfc3339();
        let new_profile = ModProfile {
            id: uuid::Uuid::new_v4().to_string(),
            name: manifest.profile.name.clone(),
            description: manifest.profile.description.clone(),
            mod_ids: manifest.profile.mod_ids.clone(),
            created_at: now.clone(),
            updated_at: now,
            builtin: false,
        };

        if let Some(idx) = existing {
            profiles[idx] = new_profile.clone();
        } else {
            profiles.push(new_profile.clone());
        }
        profile_repo::save_profiles(&profiles).map_err(|e| AppError::Other(e))?;
        return apply_profile(&new_profile.id, game_root, sync_pairs, backup_on_switch);
    }

    // 返回结果
    let enabled = mod_service::scan_enabled_mods(game_root);
    Ok(ApplyProfileResult {
        profile: ModProfile {
            id: String::new(),
            name: manifest.profile.name,
            description: manifest.profile.description,
            mod_ids: manifest.profile.mod_ids.clone(),
            created_at: manifest.profile.created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
            builtin: false,
        },
        enabled_mod_ids: enabled.iter().map(|m| m.id.clone()).collect(),
        disabled_mod_ids: Vec::new(),
        missing_mod_ids: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------
