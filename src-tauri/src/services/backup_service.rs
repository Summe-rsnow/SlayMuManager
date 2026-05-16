use crate::domain::save::{
    CloudSaveDiffEntry, CloudSaveStatus, DiffKind, SaveKind,
};
use crate::integrations::steam;
use crate::services::save_service;
use crate::utils::error::AppError;
use crate::utils::hash;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 云存档状态
// ---------------------------------------------------------------------------

pub fn get_cloud_save_status(game_root: &Path) -> Result<CloudSaveStatus, AppError> {
    let mut diagnostic = String::new();

    // 尝试获取 Steam 用户：先注册表活跃用户，再扫描 userdata
    let account_ids = steam::list_steam_users();
    let account_id = match account_ids.first() {
        Some(id) => id.clone(),
        None => {
            return Ok(CloudSaveStatus {
                is_available: false,
                cloud_path: None,
                local_path: None,
                has_mismatch: false,
                local_only_count: 0, cloud_only_count: 0, different_count: 0,
                local_file_count: 0, cloud_file_count: 0,
                local_applied_to_cloud: false, cloud_applied_to_local: false,
                diagnostic: "未找到 Steam 用户（请确认 Steam 已安装并登录）".to_string(),
            });
        }
    };

    // 云存档路径
    let cloud_path = steam::get_cloud_save_dir(&account_id);

    // 本地存档：AccountID → SteamID64
    let steam_id64 = account_id.parse::<u64>().ok()
        .map(|id| steam::account_id_to_steam_id64(id).to_string())
        .unwrap_or_else(|| account_id.clone());

    let local_path = {
        let vanilla_root = save_service::vanilla_saves_root();
        let p = vanilla_root.join(&steam_id64);
        if p.exists() { Some(p) } else { None }
    };

    // 诊断信息
    if cloud_path.is_none() {
        diagnostic.push_str("Steam 云存档目录不存在（游戏可能尚未同步）；");
    }
    if local_path.is_none() {
        diagnostic.push_str("本地存档目录不存在（可能尚未开始游戏）；");
    }

    let is_available = cloud_path.is_some() && local_path.is_some();

    if !is_available {
        return Ok(CloudSaveStatus {
            is_available: false,
            cloud_path: cloud_path.map(|p| p.to_string_lossy().to_string()),
            local_path: local_path.map(|p| p.to_string_lossy().to_string()),
            has_mismatch: false,
            local_only_count: 0, cloud_only_count: 0, different_count: 0,
            local_file_count: 0, cloud_file_count: 0,
            local_applied_to_cloud: false, cloud_applied_to_local: false,
            diagnostic: if diagnostic.is_empty() { "未知原因".to_string() } else { diagnostic },
        });
    }

    let cloud_dir = cloud_path.as_ref().unwrap();
    let local_dir = local_path.as_ref().unwrap();

    let cloud_files = list_files_relative(cloud_dir);
    let local_files = list_files_relative(local_dir);

    let mut local_only = 0u32;
    let mut cloud_only = 0u32;
    let mut different = 0u32;

    for (rel, _) in &local_files {
        if cloud_files.contains_key(rel) {
            let lh = hash_file_opt(&local_dir.join(rel));
            let ch = hash_file_opt(&cloud_dir.join(rel));
            if lh != ch {
                different += 1;
            }
        } else {
            local_only += 1;
        }
    }

    for rel in cloud_files.keys() {
        if !local_files.contains_key(rel) {
            cloud_only += 1;
        }
    }

    // 所有计数为 0 则没有不一致
    let has_mismatch = (local_only > 0 || cloud_only > 0 || different > 0)
        && (local_only + cloud_only + different > 0);

    if has_mismatch {
        diagnostic = format!("{} 本地独有, {} 云端独有, {} 内容不同", local_only, cloud_only, different);
    } else {
        diagnostic = "本地与云端完全同步".to_string();
    }

    Ok(CloudSaveStatus {
        is_available: true,
        cloud_path: Some(cloud_dir.to_string_lossy().to_string()),
        local_path: Some(local_dir.to_string_lossy().to_string()),
        has_mismatch,
        local_only_count: local_only,
        cloud_only_count: cloud_only,
        different_count: different,
        local_file_count: local_files.len() as u32,
        cloud_file_count: cloud_files.len() as u32,
        local_applied_to_cloud: false,
        cloud_applied_to_local: false,
        diagnostic,
    })
}

// ---------------------------------------------------------------------------
// 差异条目
// ---------------------------------------------------------------------------

pub fn list_cloud_save_diff_entries(game_root: &Path) -> Result<Vec<CloudSaveDiffEntry>, AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Ok(Vec::new());
    }

    let cloud_dir = Path::new(status.cloud_path.as_ref().unwrap());
    let local_dir = Path::new(status.local_path.as_ref().unwrap());

    let cloud_files = list_files_relative(cloud_dir);
    let local_files = list_files_relative(local_dir);

    let mut entries = Vec::new();
    let mut all_paths: Vec<String> = local_files.keys().cloned().collect();
    for p in cloud_files.keys() {
        if !all_paths.contains(p) {
            all_paths.push(p.clone());
        }
    }
    all_paths.sort();

    for rel in all_paths {
        let local_exists = local_files.contains_key(&rel);
        let cloud_exists = cloud_files.contains_key(&rel);

        let (local_size, cloud_size) = if local_exists && cloud_exists {
            (local_files[&rel], cloud_files[&rel])
        } else if local_exists {
            (local_files[&rel], 0)
        } else {
            (0, cloud_files[&rel])
        };

        let local_sha = if local_exists {
            hash_file_opt(&local_dir.join(&rel))
        } else {
            None
        };
        let cloud_sha = if cloud_exists {
            hash_file_opt(&cloud_dir.join(&rel))
        } else {
            None
        };

        let kind = match (local_exists, cloud_exists) {
            (true, true) if local_sha == cloud_sha => DiffKind::InSync,
            (true, true) => DiffKind::Different,
            (true, false) => DiffKind::LocalOnly,
            (false, true) => DiffKind::CloudOnly,
            _ => DiffKind::InSync,
        };

        entries.push(CloudSaveDiffEntry {
            relative_path: rel,
            kind,
            local_exists,
            cloud_exists,
            local_size: if local_exists { Some(local_size) } else { None },
            cloud_size: if cloud_exists { Some(cloud_size) } else { None },
            local_sha,
            cloud_sha,
        });
    }

    Ok(entries)
}

// ---------------------------------------------------------------------------
// 单侧复制
// ---------------------------------------------------------------------------

pub fn copy_cloud_save_diff_side(
    game_root: &Path,
    relative_path: &str,
    side: &str,
) -> Result<(), AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Err(AppError::Other("云存档不可用".to_string()));
    }

    let cloud_dir = Path::new(status.cloud_path.as_ref().unwrap());
    let local_dir = Path::new(status.local_path.as_ref().unwrap());

    match side {
        "local_to_cloud" => {
            let src = local_dir.join(relative_path);
            let dst = cloud_dir.join(relative_path);
            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent).map_err(AppError::Io)?;
                }
                std::fs::copy(&src, &dst).map_err(AppError::Io)?;
            }
        }
        "cloud_to_local" => {
            let src = cloud_dir.join(relative_path);
            let dst = local_dir.join(relative_path);
            if src.exists() {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent).map_err(AppError::Io)?;
                }
                std::fs::copy(&src, &dst).map_err(AppError::Io)?;
            }
        }
        _ => return Err(AppError::Other(format!("无效的复制方向: {}", side))),
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 全量上传/下载
// ---------------------------------------------------------------------------

pub fn ascend_to_cloud_full(game_root: &Path) -> Result<(), AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Err(AppError::Other("云存档不可用".to_string()));
    }

    let cloud_dir = Path::new(status.cloud_path.as_ref().unwrap());
    let local_dir = Path::new(status.local_path.as_ref().unwrap());

    // 备份云端
    create_cloud_artifact(cloud_dir, "ascend")?;

    // 全量复制本地 → 云端
    let local_files = list_files_relative(local_dir);
    for (rel, _) in &local_files {
        let src = local_dir.join(rel);
        let dst = cloud_dir.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        std::fs::copy(&src, &dst).map_err(AppError::Io)?;
    }

    Ok(())
}

pub fn descend_from_cloud_full(game_root: &Path) -> Result<(), AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Err(AppError::Other("云存档不可用".to_string()));
    }

    let cloud_dir = Path::new(status.cloud_path.as_ref().unwrap());
    let local_dir = Path::new(status.local_path.as_ref().unwrap());

    // 备份本地
    create_cloud_artifact(local_dir, "descend")?;

    // 全量复制云端 → 本地
    let cloud_files = list_files_relative(cloud_dir);
    for (rel, _) in &cloud_files {
        let src = cloud_dir.join(rel);
        let dst = local_dir.join(rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        std::fs::copy(&src, &dst).map_err(AppError::Io)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 云操作备份制品（存储在应用数据目录，避免污染存档目录）
// ---------------------------------------------------------------------------

fn artifacts_dir() -> PathBuf {
    let base = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.slay.mumanager")
        .join("cloud_artifacts");
    std::fs::create_dir_all(&base).ok();
    base
}

pub fn get_backup_artifact_status(game_root: &Path) -> Result<bool, AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Ok(false);
    }
    // 检查制品目录是否有内容
    let dir = artifacts_dir();
    Ok(dir.exists() && std::fs::read_dir(&dir).map_or(false, |mut r| r.next().is_some()))
}

pub fn cleanup_backup_artifacts(game_root: &Path) -> Result<(), AppError> {
    let status = get_cloud_save_status(game_root)?;
    if !status.is_available {
        return Ok(());
    }
    let dir = artifacts_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(AppError::Io)?;
    }
    Ok(())
}

fn create_cloud_artifact(dir: &Path, operation: &str) -> Result<(), AppError> {
    if !dir.exists() {
        return Ok(());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let artifact_name = format!(
        "{}_{}_{}",
        operation,
        dir.file_name().unwrap_or_default().to_string_lossy(),
        timestamp,
    );
    let artifact_dir = artifacts_dir().join(&artifact_name);

    if artifact_dir.exists() {
        std::fs::remove_dir_all(&artifact_dir).map_err(AppError::Io)?;
    }
    std::fs::create_dir_all(&artifact_dir).map_err(AppError::Io)?;

    save_service::copy_dir_recursive(dir, &artifact_dir)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------

fn list_files_relative(dir: &Path) -> std::collections::HashMap<String, u64> {
    let mut map = std::collections::HashMap::new();
    collect_files(dir, dir, &mut map);
    map
}

fn collect_files(base: &Path, current: &Path, out: &mut std::collections::HashMap<String, u64>) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base) {
                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    out.insert(rel.to_string_lossy().to_string(), size);
                }
            } else if path.is_dir() {
                collect_files(base, &path, out);
            }
        }
    }
}

fn hash_file_opt(path: &Path) -> Option<String> {
    hash::sha1_file(path).ok()
}
