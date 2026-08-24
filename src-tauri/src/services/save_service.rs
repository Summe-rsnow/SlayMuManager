use crate::app::state::SaveSyncPair;
use crate::domain::save::{
    SaveBackupEntry, SaveKind, SaveSlot, SaveSlotRef, SaveSyncResult, SaveTransferPreview,
    SyncDetail, SyncDirection,
};
use crate::repositories::settings_repo;
use crate::utils::error::AppError;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 路径工具
// ---------------------------------------------------------------------------

/// 原版存档根目录（跨平台）
/// Windows: %APPDATA%/SlayTheSpire2/steam/
/// macOS:   ~/Library/Application Support/SlayTheSpire2/steam/
/// Linux:   ~/.local/share/SlayTheSpire2/steam/
pub fn vanilla_saves_root() -> PathBuf {
    dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("SlayTheSpire2")
        .join("steam")
}

/// 备份根目录
fn backups_root(game_root: &Path) -> PathBuf {
    game_root.join("BepInEx").join("save_backups")
}

/// 构造存档槽位的完整路径
/// 原版和模组版在同一根目录下，模组版多一层 modded/
/// 原版: <save_root>/<SteamID64>/profile{slot_index}/saves/
/// 模组: <save_root>/<SteamID64>/modded/profile{slot_index}/saves/
fn slot_path(
    _game_root: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
) -> PathBuf {
    let user_dir = vanilla_saves_root().join(steam_user_id);
    match kind {
        SaveKind::Vanilla => user_dir.join(format!("profile{}", slot_index)).join("saves"),
        SaveKind::Modded => user_dir.join("modded").join(format!("profile{}", slot_index)).join("saves"),
    }
}

/// 备份元数据文件路径
fn backups_meta_path(game_root: &Path) -> PathBuf {
    backups_root(game_root).join("_backups.json")
}

// ---------------------------------------------------------------------------
// 扫描
// ---------------------------------------------------------------------------

/// 扫描所有存档槽位（原版 + 模组版，共用同一根目录）
pub fn list_save_slots(game_root: &Path) -> Vec<SaveSlot> {
    let mut slots = Vec::new();

    let root = vanilla_saves_root();
    if !root.exists() {
        return slots;
    }

    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return slots,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let user_dir = entry.path();
        if !user_dir.is_dir() {
            continue;
        }
        let steam_user_id = user_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 跳过云存档制品目录（非真实用户）
        if steam_user_id.contains("_cloud_artifact") {
            continue;
        }

        // 每个 Steam 用户下扫描 3 个原版槽位 + 3 个模组版槽位
        for kind in [SaveKind::Vanilla, SaveKind::Modded] {
            for i in 1u32..=3u32 {
                let slot_dir = slot_path(game_root, &steam_user_id, &kind, i);
                let slot = build_save_slot(&slot_dir, &steam_user_id, &kind, i);
                slots.push(slot);
            }
        }
    }

    slots
}

fn build_save_slot(
    dir: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
) -> SaveSlot {
    // 有存档 = progress.save 存在
    let has_data = dir.join("progress.save").is_file();
    // 有当前局 = current_run.save 存在
    let has_current_run = dir.join("current_run.save").is_file();

    let mut file_count = 0u32;
    let mut last_modified: Option<String> = None;

    if dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    file_count += 1;
                }
                if let Ok(meta) = path.metadata()
                    && let Ok(mtime) = meta.modified()
                        && path.file_name().is_some_and(|n| n == "progress.save") {
                            last_modified = Some(filetime_to_rfc3339(mtime));
                        }
            }
        }

    SaveSlot {
        steam_user_id: steam_user_id.to_string(),
        kind: kind.clone(),
        slot_index,
        path: dir.to_string_lossy().to_string(),
        has_data,
        has_current_run,
        file_count,
        last_modified_at: last_modified,
    }
}

fn filetime_to_rfc3339(ft: std::time::SystemTime) -> String {
    let duration = ft
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let dt = chrono::DateTime::from_timestamp(secs as i64, 0)
        .unwrap_or_default();
    dt.to_rfc3339()
}

// ---------------------------------------------------------------------------
// 传输预览
// ---------------------------------------------------------------------------

pub fn preview_save_transfer(
    game_root: &Path,
    source: &SaveSlotRef,
    target: &SaveSlotRef,
) -> SaveTransferPreview {
    let src_path = slot_path(game_root, &source.steam_user_id, &source.kind, source.slot_index);
    let tgt_path = slot_path(game_root, &target.steam_user_id, &target.kind, target.slot_index);

    let source_has_data = src_path.is_dir()
        && std::fs::read_dir(&src_path)
            .map(|mut r| r.next().is_some())
            .unwrap_or(false);
    let target_has_data = tgt_path.is_dir()
        && std::fs::read_dir(&tgt_path)
            .map(|mut r| r.next().is_some())
            .unwrap_or(false);
    let backup_will_be_created = target_has_data;

    let summary = if !source_has_data {
        "源槽位无数据，无法传输".to_string()
    } else if backup_will_be_created {
        format!(
            "将从 {} 槽位 {} 复制到 {} 槽位 {}（目标将被备份）",
            kind_label(&source.kind),
            source.slot_index,
            kind_label(&target.kind),
            target.slot_index,
        )
    } else {
        format!(
            "将从 {} 槽位 {} 复制到 {} 槽位 {}",
            kind_label(&source.kind),
            source.slot_index,
            kind_label(&target.kind),
            target.slot_index,
        )
    };

    SaveTransferPreview {
        source: source.clone(),
        target: target.clone(),
        source_has_data,
        target_has_data,
        backup_will_be_created,
        summary,
    }
}

fn kind_label(kind: &SaveKind) -> &str {
    match kind {
        SaveKind::Vanilla => "原版",
        SaveKind::Modded => "模组版",
    }
}

// ---------------------------------------------------------------------------
// 执行传输
// ---------------------------------------------------------------------------

pub fn transfer_save(
    game_root: &Path,
    source: &SaveSlotRef,
    target: &SaveSlotRef,
) -> Result<SaveBackupEntry, AppError> {
    let src_path = slot_path(game_root, &source.steam_user_id, &source.kind, source.slot_index);
    let tgt_path = slot_path(game_root, &target.steam_user_id, &target.kind, target.slot_index);

    if !src_path.exists() {
        return Err(AppError::Other("源槽位无数据".to_string()));
    }

    // 目标有数据则先备份
    let mut backup_entry: Option<SaveBackupEntry> = None;
    if tgt_path.exists() {
        backup_entry = Some(create_backup_internal(
            game_root,
            &target.steam_user_id,
            &target.kind,
            target.slot_index,
            "存档传输前自动备份",
            false,
        )?);
    }

    // 确保目标目录
    if let Some(parent) = tgt_path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    // 如果目标已存在（且已备份），先删除
    if tgt_path.exists() {
        std::fs::remove_dir_all(&tgt_path).map_err(AppError::Io)?;
    }

    // 复制
    copy_dir_recursive(&src_path, &tgt_path)?;

    Ok(backup_entry.unwrap_or_else(|| SaveBackupEntry {
        id: String::new(),
        steam_user_id: target.steam_user_id.clone(),
        kind: target.kind.clone(),
        slot_index: target.slot_index,
        backup_path: String::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
        reason: "直接传输（无备份）".to_string(),
        manual: Some(false),
    }))
}

// ---------------------------------------------------------------------------
// 备份
// ---------------------------------------------------------------------------

pub fn create_save_backup(
    game_root: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
    reason: &str,
) -> Result<SaveBackupEntry, AppError> {
    let slot_dir = slot_path(game_root, steam_user_id, kind, slot_index);
    if !slot_dir.exists() {
        return Err(AppError::Other("槽位无数据，无需备份".to_string()));
    }
    create_backup_internal(game_root, steam_user_id, kind, slot_index, reason, true)
}

fn create_backup_internal(
    game_root: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
    reason: &str,
    manual: bool,
) -> Result<SaveBackupEntry, AppError> {
    let backups_dir = backups_root(game_root);
    std::fs::create_dir_all(&backups_dir).map_err(AppError::Io)?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let sanitized_reason = reason.replace([' ', '/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let backup_name = format!(
        "{}_{}_{}_{}_{}",
        match kind {
            SaveKind::Vanilla => "vanilla",
            SaveKind::Modded => "modded",
        },
        steam_user_id,
        slot_index,
        timestamp,
        sanitized_reason,
    );
    let backup_dir = backups_dir.join(&backup_name);

    let slot_dir = slot_path(game_root, steam_user_id, kind, slot_index);
    copy_dir_recursive(&slot_dir, &backup_dir)?;

    let entry = SaveBackupEntry {
        id: backup_name.clone(),
        steam_user_id: steam_user_id.to_string(),
        kind: kind.clone(),
        slot_index,
        backup_path: backup_dir.to_string_lossy().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        reason: reason.to_string(),
        manual: Some(manual),
    };

    // 追加备份元数据
    append_backup_meta(game_root, &entry)?;

    // 清理旧备份（从设置读取保留份数）
    let keep_count = settings_repo::load_settings()
        .map(|s| s.auto_backup_keep_count)
        .unwrap_or(5);
    trim_old_backups(game_root, steam_user_id, kind, slot_index, keep_count)?;

    Ok(entry)
}

pub fn list_save_backups(
    game_root: &Path,
    steam_user_id: Option<&str>,
    kind_filter: Option<&SaveKind>,
    slot_index_filter: Option<u32>,
) -> Vec<SaveBackupEntry> {
    let meta_path = backups_meta_path(game_root);
    if !meta_path.exists() {
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&meta_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let all: Vec<SaveBackupEntry> = serde_json::from_str(&content).unwrap_or_default();

    // 按创建时间倒序
    let mut filtered: Vec<SaveBackupEntry> = all
        .into_iter()
        .filter(|e| {
            steam_user_id.is_none_or(|uid| e.steam_user_id == uid)
                && kind_filter.as_ref().is_none_or(|k| std::mem::discriminant(&e.kind) == std::mem::discriminant(k))
                && slot_index_filter.is_none_or(|i| e.slot_index == i)
        })
        .collect();

    filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    filtered
}

pub fn restore_save_backup(game_root: &Path, backup_id: &str) -> Result<(), AppError> {
    let backups = list_save_backups(game_root, None, None, None);
    let entry = backups
        .iter()
        .find(|e| e.id == backup_id)
        .ok_or_else(|| AppError::Other(format!("备份不存在: {}", backup_id)))?;

    let backup_dir = Path::new(&entry.backup_path);
    if !backup_dir.exists() {
        return Err(AppError::Other("备份目录不存在".to_string()));
    }

    let slot_dir = slot_path(game_root, &entry.steam_user_id, &entry.kind, entry.slot_index);

    // 覆盖前先备份当前数据
    if slot_dir.exists() {
        let _ = create_backup_internal(
            game_root,
            &entry.steam_user_id,
            &entry.kind,
            entry.slot_index,
            "恢复前自动备份",
            false,
        );
        std::fs::remove_dir_all(&slot_dir).map_err(AppError::Io)?;
    }

    if let Some(parent) = slot_dir.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    copy_dir_recursive(backup_dir, &slot_dir)?;
    Ok(())
}

pub fn delete_save_backup(game_root: &Path, backup_id: &str) -> Result<(), AppError> {
    let backups = list_save_backups(game_root, None, None, None);
    let entry = backups
        .iter()
        .find(|e| e.id == backup_id)
        .ok_or_else(|| AppError::Other(format!("备份不存在: {}", backup_id)))?;

    let backup_dir = Path::new(&entry.backup_path);
    if backup_dir.exists() {
        std::fs::remove_dir_all(backup_dir).map_err(AppError::Io)?;
    }

    remove_backup_meta(game_root, backup_id)?;
    Ok(())
}

/// 将备份恢复到用户指定的槽位（而非原始槽位）
pub fn restore_save_backup_to_slot(
    game_root: &Path,
    backup_id: &str,
    target_steam_user_id: &str,
    target_kind: &SaveKind,
    target_slot_index: u32,
) -> Result<(), AppError> {
    let backups = list_save_backups(game_root, None, None, None);
    let entry = backups
        .iter()
        .find(|e| e.id == backup_id)
        .ok_or_else(|| AppError::Other(format!("备份不存在: {}", backup_id)))?;

    let backup_dir = Path::new(&entry.backup_path);
    if !backup_dir.exists() {
        return Err(AppError::Other("备份目录不存在".to_string()));
    }

    let target_slot_dir = slot_path(game_root, target_steam_user_id, target_kind, target_slot_index);

    // 覆盖前先备份当前数据
    if target_slot_dir.exists() {
        let _ = create_backup_internal(
            game_root,
            target_steam_user_id,
            target_kind,
            target_slot_index,
            "恢复备份前自动备份",
            false,
        );
        std::fs::remove_dir_all(&target_slot_dir).map_err(AppError::Io)?;
    }

    if let Some(parent) = target_slot_dir.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    copy_dir_recursive(backup_dir, &target_slot_dir)?;
    Ok(())
}

/// 清空存档槽位（删除前自动创建最终备份）
pub fn delete_save_slot(
    game_root: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
) -> Result<(), AppError> {
    let slot_dir = slot_path(game_root, steam_user_id, kind, slot_index);
    if !slot_dir.exists() {
        return Ok(()); // 已经为空，无需操作
    }

    // 删除前创建最终备份
    let _ = create_backup_internal(
        game_root,
        steam_user_id,
        kind,
        slot_index,
        "手动删除前自动备份",
        false,
    );

    // 清空 saves 目录

    for entry in std::fs::read_dir(&slot_dir).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(AppError::Io)?;
        } else {
            std::fs::remove_file(&path).map_err(AppError::Io)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 配对同步
// ---------------------------------------------------------------------------

/// 收集存档目录下的 Steam 用户 ID，无用户时返回 ["default"]
fn collect_steam_user_ids() -> Vec<String> {
    let vanilla_root = vanilla_saves_root();
    let user_ids: Vec<String> = if vanilla_root.exists() {
        std::fs::read_dir(&vanilla_root)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| {
                        let name = e.file_name().to_string_lossy().to_string();
                        if name.contains("_cloud_artifact") { None } else { Some(name) }
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    if user_ids.is_empty() { vec!["default".to_string()] } else { user_ids }
}

/// 同步一对存档槽位，决定方向并执行复制。无需同步时返回 Ok(None)。
fn sync_pair(
    game_root: &Path,
    steam_user_id: &str,
    pair: &SaveSyncPair,
) -> Result<Option<SyncDetail>, AppError> {
    let vanilla_path = slot_path(game_root, steam_user_id, &SaveKind::Vanilla, pair.vanilla_slot);
    let modded_path = slot_path(game_root, steam_user_id, &SaveKind::Modded, pair.modded_slot);

    let v_has = vanilla_path.join("progress.save").is_file();
    let m_has = modded_path.join("progress.save").is_file();
    let v_time = dir_last_modified(&vanilla_path);
    let m_time = dir_last_modified(&modded_path);

    let direction = match (v_has, m_has) {
        (true, false) => Some(SyncDirection::VanillaToModded),
        (false, true) => Some(SyncDirection::ModdedToVanilla),
        (true, true) => match (v_time, m_time) {
            (Some(v), Some(m)) if v > m => Some(SyncDirection::VanillaToModded),
            (Some(v), Some(m)) if m > v => Some(SyncDirection::ModdedToVanilla),
            _ => None,
        },
        _ => None,
    };

    match direction {
        Some(SyncDirection::VanillaToModded) => {
            if m_has {
                let _ = create_backup_internal(
                    game_root, steam_user_id, &SaveKind::Modded, pair.modded_slot,
                    "同步前自动备份",
                    false,
                );
            }
            incremental_sync_dir(&vanilla_path, &modded_path)?;
            Ok(Some(SyncDetail {
                slot_index: pair.modded_slot,
                direction: SyncDirection::VanillaToModded,
                backup_created: m_has,
            }))
        }
        Some(SyncDirection::ModdedToVanilla) => {
            if v_has {
                let _ = create_backup_internal(
                    game_root, steam_user_id, &SaveKind::Vanilla, pair.vanilla_slot,
                    "同步前自动备份",
                    false,
                );
            }
            incremental_sync_dir(&modded_path, &vanilla_path)?;
            Ok(Some(SyncDetail {
                slot_index: pair.vanilla_slot,
                direction: SyncDirection::ModdedToVanilla,
                backup_created: v_has,
            }))
        }
        _ => Ok(None),
    }
}

pub fn sync_saves(game_root: &Path, pairs: &[SaveSyncPair]) -> Result<SaveSyncResult, AppError> {
    let user_ids = collect_steam_user_ids();
    let mut details = Vec::new();

    for steam_user_id in &user_ids {
        for pair in pairs {
            if let Ok(Some(detail)) = sync_pair(game_root, steam_user_id, pair) {
                details.push(detail);
            }
        }
    }

    Ok(SaveSyncResult {
        synced_count: details.len() as u32,
        details,
    })
}

fn dir_last_modified(dir: &Path) -> Option<std::time::SystemTime> {
    if !dir.is_dir() {
        return None;
    }
    let mut latest: Option<std::time::SystemTime> = None;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(meta) = entry.metadata()
                && let Ok(mtime) = meta.modified()
                    && latest.is_none_or(|l| mtime > l) {
                        latest = Some(mtime);
                    }
        }
    }
    latest
}

// ---------------------------------------------------------------------------
// 工具
// ---------------------------------------------------------------------------

/// 增量同步目录：只复制有新/修改的文件，删除目标中多余的文件
fn incremental_sync_dir(from: &Path, to: &Path) -> Result<(), AppError> {
    if !from.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(to).map_err(AppError::Io)?;

    let mut to_files: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    if to.exists() {
        collect_files_recursive(to, to, &mut to_files);
    }

    let mut from_files: std::collections::HashMap<String, PathBuf> = std::collections::HashMap::new();
    collect_files_recursive(from, from, &mut from_files);

    for (rel_path, src_path) in &from_files {
        let dst_path = to.join(rel_path);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        let needs_copy = match to_files.get(rel_path) {
            None => true,
            Some(existing) => {
                let src_mtime = std::fs::metadata(src_path).ok().and_then(|m| m.modified().ok());
                let dst_mtime = std::fs::metadata(existing).ok().and_then(|m| m.modified().ok());
                src_mtime != dst_mtime
            }
        };

        if needs_copy {
            std::fs::copy(src_path, &dst_path).map_err(AppError::Io)?;
        }
        to_files.remove(rel_path);
    }

    for extra_path in to_files.values() {
        let _ = std::fs::remove_file(extra_path);
    }

    Ok(())
}

fn collect_files_recursive(base: &Path, dir: &Path, out: &mut std::collections::HashMap<String, PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(base, &path, out);
            } else if let Ok(rel) = path.strip_prefix(base) {
                out.insert(rel.to_string_lossy().to_string(), path);
            }
        }
    }
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
    if !src.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(dst).map_err(AppError::Io)?;
    for entry in std::fs::read_dir(src).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 备份元数据 JSON
// ---------------------------------------------------------------------------

fn append_backup_meta(game_root: &Path, entry: &SaveBackupEntry) -> Result<(), AppError> {
    let path = backups_meta_path(game_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    let mut entries: Vec<SaveBackupEntry> = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    entries.push(entry.clone());

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Other(format!("备份元数据序列化失败: {}", e)))?;
    std::fs::write(&path, json).map_err(AppError::Io)?;

    Ok(())
}

fn remove_backup_meta(game_root: &Path, backup_id: &str) -> Result<(), AppError> {
    let path = backups_meta_path(game_root);
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let mut entries: Vec<SaveBackupEntry> = serde_json::from_str(&content).unwrap_or_default();
    entries.retain(|e| e.id != backup_id);

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Other(format!("备份元数据序列化失败: {}", e)))?;
    std::fs::write(&path, json).map_err(AppError::Io)?;

    Ok(())
}

/// 升级旧版备份：为 manual 字段为 null 的条目设置明确的标记
pub fn upgrade_backup_manual_flag(
    game_root: &Path,
    backup_id: &str,
    manual: bool,
) -> Result<(), AppError> {
    let path = backups_meta_path(game_root);
    if !path.exists() {
        return Err(AppError::Other("备份元数据不存在".to_string()));
    }

    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let mut entries: Vec<SaveBackupEntry> = serde_json::from_str(&content).unwrap_or_default();
    let entry = entries
        .iter_mut()
        .find(|e| e.id == backup_id)
        .ok_or_else(|| AppError::Other(format!("备份不存在: {}", backup_id)))?;

    entry.manual = Some(manual);

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Other(format!("备份元数据序列化失败: {}", e)))?;
    std::fs::write(&path, json).map_err(AppError::Io)?;

    Ok(())
}

fn trim_old_backups(
    game_root: &Path,
    steam_user_id: &str,
    kind: &SaveKind,
    slot_index: u32,
    keep_count: usize,
) -> Result<(), AppError> {
    let path = backups_meta_path(game_root);
    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let mut entries: Vec<SaveBackupEntry> = serde_json::from_str(&content).unwrap_or_default();

    let mut matching: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.manual == Some(false)
                && e.steam_user_id == steam_user_id
                && e.slot_index == slot_index
                && std::mem::discriminant(&e.kind) == std::mem::discriminant(kind)
        })
        .map(|(i, _)| i)
        .collect();

    if matching.len() <= keep_count {
        return Ok(());
    }

    // 按时间升序，删除最旧的
    matching.sort_by(|&a, &b| {
        entries[a].created_at.cmp(&entries[b].created_at)
    });

    let to_remove = &matching[..matching.len() - keep_count];
    for &idx in to_remove.iter() {
        let backup_dir = PathBuf::from(&entries[idx].backup_path);
        if backup_dir.exists() {
            let _ = std::fs::remove_dir_all(&backup_dir);
        }
    }

    // 更新元数据
    let remove_ids: Vec<String> = to_remove.iter().map(|&i| entries[i].id.clone()).collect();
    entries.retain(|e| !remove_ids.contains(&e.id));

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| AppError::Other(format!("备份元数据序列化失败: {}", e)))?;
    std::fs::write(&path, json).map_err(AppError::Io)?;

    Ok(())
}
