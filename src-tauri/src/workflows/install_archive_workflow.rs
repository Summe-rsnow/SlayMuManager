use crate::domain::mod_entity::{
    BatchImportPreview, BatchInstallItem, BatchInstallResult, ConflictResolution,
    DiscoveredMod, DiscoveredModSourceType, DiscoveredModStatus, InstalledMod, InstalledModState,
};
use crate::integrations::manifest::ModManifest;
use crate::services::mod_service;
use crate::services::save_service;
use crate::utils::error::AppError;
use rars::ArchiveReader;
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::Emitter;

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------

/// 递归扫描最大深度（对齐 SlaySP2Manager）
const MAX_DISCOVERY_DEPTH: u32 = 3;

// ---------------------------------------------------------------------------
// 归档格式检测（magic bytes + 扩展名兜底）
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
    Rar,
}

/// 检测归档格式：优先魔数，其次扩展名
pub fn detect_archive_format(path: &Path) -> Option<ArchiveFormat> {
    // 1. 魔数检测
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buf = [0u8; 8];
        if file.read_exact(&mut buf).is_ok() {
            // ZIP: 50 4B 03 04 | 50 4B 05 06 (empty) | 50 4B 07 08 (spanned)
            if buf[0] == 0x50 && buf[1] == 0x4B {
                return Some(ArchiveFormat::Zip);
            }
            // 7z: 37 7A BC AF 27 1C
            if buf[0] == 0x37 && buf[1] == 0x7A && buf[2] == 0xBC
                && buf[3] == 0xAF && buf[4] == 0x27 && buf[5] == 0x1C
            {
                return Some(ArchiveFormat::SevenZ);
            }
            // RAR: 52 61 72 21 1A 07 00 (1.5-4.x) | 52 61 72 21 1A 07 01 00 (5.0+)
            if buf[0] == 0x52 && buf[1] == 0x61 && buf[2] == 0x72
                && buf[3] == 0x21 && buf[4] == 0x1A && buf[5] == 0x07
                && (buf[6] == 0x00 || buf[6] == 0x01)
            {
                return Some(ArchiveFormat::Rar);
            }
        }
    }

    // 2. 扩展名兜底
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "zip" => Some(ArchiveFormat::Zip),
        "7z" | "7zip" => Some(ArchiveFormat::SevenZ),
        "rar" => Some(ArchiveFormat::Rar),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 归档解压
// ---------------------------------------------------------------------------

/// 解压归档到临时目录，返回解压根目录
pub fn extract_archive(archive_path: &Path) -> Result<PathBuf, AppError> {
    let format = detect_archive_format(archive_path).ok_or_else(|| {
        AppError::Other(format!("不支持的归档格式: {}", archive_path.display()))
    })?;

    let temp_dir = std::env::temp_dir()
        .join("slaymumanager")
        .join("extract")
        .join(uuid::Uuid::new_v4().to_string());

    std::fs::create_dir_all(&temp_dir).map_err(AppError::Io)?;

    match format {
        ArchiveFormat::Zip => extract_zip(archive_path, &temp_dir)?,
        ArchiveFormat::SevenZ => extract_7z(archive_path, &temp_dir)?,
        ArchiveFormat::Rar => extract_rar(archive_path, &temp_dir)?,
    }

    Ok(temp_dir)
}

fn extract_zip(archive_path: &Path, output_dir: &Path) -> Result<(), AppError> {
    let file = std::fs::File::open(archive_path).map_err(AppError::Io)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| AppError::Other(format!("ZIP 读取失败: {}", e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("ZIP 条目读取失败: {}", e)))?;

        let name = entry.name().to_string();
        let out_path = output_dir.join(&name);

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(AppError::Io)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(AppError::Io)?;
            }
            let mut out_file = std::fs::File::create(&out_path).map_err(AppError::Io)?;
            std::io::copy(&mut entry, &mut out_file).map_err(AppError::Io)?;
        }
    }

    Ok(())
}

fn extract_7z(archive_path: &Path, output_dir: &Path) -> Result<(), AppError> {
    sevenz_rust::decompress_file(archive_path, output_dir)
        .map_err(|e| AppError::Other(format!("7z 解压失败: {}", e)))?;
    Ok(())
}

fn extract_rar(archive_path: &Path, output_dir: &Path) -> Result<(), AppError> {
    let archive = ArchiveReader::read_path(archive_path)
        .map_err(|e| AppError::Other(format!("RAR 读取失败: {}", e)))?;
    archive
        .extract_to(None, |meta| {
            let path_str = meta.name_lossy();
            let out_path = output_dir.join(&path_str);
            if meta.is_directory {
                std::fs::create_dir_all(&out_path)?;
                Ok(Box::new(std::io::sink()) as Box<dyn std::io::Write>)
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let file = std::fs::File::create(&out_path)?;
                Ok(Box::new(file) as Box<dyn std::io::Write>)
            }
        })
        .map_err(|e| AppError::Other(format!("RAR 解压失败: {}", e)))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mod 发现：在目录中递归发现所有 Mod 文件夹
// 对齐 SlaySP2Manager recursive_discover_from_dir：
//   manifest → mods/ → 子文件夹递归 → 嵌套压缩包自动解压 → 回退推理
// ---------------------------------------------------------------------------

/// 入口：触发 3 层深度限制的递归 Mod 发现
pub fn discover_mods_in_dir(
    root: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> Vec<DiscoveredMod> {
    recursive_discover_from_dir(root, source_archive, source_type, 0)
}

/// 递归扫描目录：每层检测 manifest / mods/ 结构 / 嵌套压缩包
fn recursive_discover_from_dir(
    dir: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
    depth: u32,
) -> Vec<DiscoveredMod> {
    if depth > MAX_DISCOVERY_DEPTH {
        return Vec::new();
    }

    let mut mods: Vec<DiscoveredMod> = Vec::new();

    // 1. 当前目录自身就是 Mod（直接包含 manifest）
    if has_manifest(dir) {
        // 深度 0（解压根目录）：找最佳 manifest，避免无关 .json 干扰
        let (manifest_path, manifest) = if depth == 0 {
            find_best_manifest(dir)
        } else {
            let p = find_manifest_path(dir);
            let m = p.as_ref().and_then(|p| ModManifest::from_file(p));
            (p, m)
        };
        let folder_name = if depth == 0 {
            // 参考同类实现：用 manifest 文件名（去扩展名.stem）作为文件夹名
            // 如 STS2Advisor.json → STS2Advisor，mod_manifest.json 除外
            manifest_path
                .as_ref()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .filter(|s| *s != "mod_manifest")
                .map(|s| s.to_string())
                .or_else(|| {
                    source_archive.and_then(|s| {
                        std::path::Path::new(s)
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .map(|n| n.to_string())
                    })
                })
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };
        mods.push(build_discovered_mod(
            &manifest,
            &folder_name,
            source_archive,
            source_type,
        ));
        return mods;
    }

    // 2. 扫描子条目
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return mods,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            // 2a. mods/ 特殊目录：其子文件夹都是 Mod
            if path.file_name().and_then(|n| n.to_str()) == Some("mods") {
                mods.extend(discover_from_plugins_dir(
                    &path,
                    source_archive,
                    source_type,
                ));
            }
            // 2b. 带 manifest 的子目录 → 识别为 Mod
            else if has_manifest(&path) {
                let folder_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let manifest = find_manifest_path(&path)
                    .and_then(|p| ModManifest::from_file(&p));
                mods.push(build_discovered_mod(
                    &manifest,
                    &folder_name,
                    source_archive,
                    source_type,
                ));
            }
            // 2c. 普通子目录 → 递归深入
            else {
                mods.extend(recursive_discover_from_dir(
                    &path,
                    source_archive,
                    source_type,
                    depth + 1,
                ));
            }
        } else if path.is_file() {
            // 2d. 嵌套压缩包 → 自动解压后递归扫描
            if detect_archive_format(&path).is_some() {
                let archive_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                match extract_archive(&path) {
                    Ok(temp_dir) => {
                        let nested = recursive_discover_from_dir(
                            &temp_dir,
                            Some(archive_name),
                            DiscoveredModSourceType::Archive,
                            depth + 1,
                        );
                        mods.extend(nested);
                        cleanup_extract_dir(&temp_dir);
                    }
                    Err(_) => {
                        // 解压失败的嵌套压缩包静默跳过
                    }
                }
            }
        }
    }

    // 3. 一无所获 → 回退推理
    if mods.is_empty() {
        return fallback_discovery(dir, source_archive, source_type);
    }

    mods
}

/// 检查目录是否包含 mod manifest（多文件名兼容）
fn has_manifest(dir: &Path) -> bool {
    let folder_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    dir.join(format!("{}.json", folder_name)).exists()
        || dir.join("mod_manifest.json").exists()
        || dir.join("manifest.json").exists()
        || dir
            .read_dir()
            .map(|mut rd| {
                rd.any(|e| {
                    e.as_ref().is_ok_and(|entry| {
                        entry.path().extension().is_some_and(|ext| ext == "json")
                    })
                })
            })
            .unwrap_or(false)
}

/// 在目录中找到 manifest 文件路径（优先级对齐 ModManifest::find_in_dir）
fn find_manifest_path(dir: &Path) -> Option<PathBuf> {
    ModManifest::find_in_dir(dir).map(|(p, _)| p)
}

/// 在目录中找最佳 manifest：按优先级探测，优先返回 is_valid() 通过的，
/// 同时返回文件路径（用于取 stem 作为文件夹名）和解析结果。
/// 参考同类实现逻辑。
fn find_best_manifest(dir: &Path) -> (Option<PathBuf>, Option<ModManifest>) {
    let folder_name = match dir.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_string(),
        None => return (None, None),
    };

    // 1. <folderName>.json（StS2 标准命名约定）
    let named = dir.join(format!("{folder_name}.json"));
    if let Some(m) = ModManifest::from_file(&named)
        && m.is_valid() {
            return (Some(named), Some(m));
        }

    // 2. mod_manifest.json
    let alt = dir.join("mod_manifest.json");
    if let Some(m) = ModManifest::from_file(&alt)
        && m.is_valid() {
            return (Some(alt), Some(m));
        }

    // 3. manifest.json
    let def = dir.join("manifest.json");
    if let Some(m) = ModManifest::from_file(&def)
        && m.is_valid() {
            return (Some(def), Some(m));
        }

    // 4. 任意 .json — 扫描全部，优先 is_valid，否则回退到第一个可解析的
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut fb_path: Option<PathBuf> = None;
        let mut fb_manifest: Option<ModManifest> = None;
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|e| e == "json")
                && let Some(m) = ModManifest::from_file(&p) {
                    if m.is_valid() {
                        return (Some(p), Some(m));
                    }
                    if fb_manifest.is_none() {
                        fb_path = Some(p);
                        fb_manifest = Some(m);
                    }
                }
        }
        if let (Some(fp), Some(fm)) = (fb_path, fb_manifest) {
            return (Some(fp), Some(fm));
        }
    }

    (None, None)
}

fn discover_from_plugins_dir(
    plugins_dir: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> Vec<DiscoveredMod> {
    let mut mods = Vec::new();
    let entries = match std::fs::read_dir(plugins_dir) {
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
        let manifest_path = path.join("manifest.json");
        let manifest = ModManifest::from_file(&manifest_path);

        mods.push(build_discovered_mod(
            &manifest,
            &folder_name,
            source_archive,
            source_type,
        ));
    }

    mods
}

fn fallback_discovery(
    root: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> Vec<DiscoveredMod> {
    let subdirs: Vec<_> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    let file_count = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count();

    if subdirs.len() == 1 && file_count == 0 {
        // 单个子文件夹 → 视为该 Mod 本身
        let mod_dir = &subdirs[0].path();
        let folder_name = mod_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let manifest_path = mod_dir.join("manifest.json");
        let manifest = ModManifest::from_file(&manifest_path);
        return vec![build_discovered_mod(
            &manifest,
            &folder_name,
            source_archive,
            source_type,
        )];
    }

    if subdirs.is_empty() && file_count > 0 {
        // 散文件 → 用顶层目录名作为文件夹名包裹
        let folder_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let manifest_path = root.join("manifest.json");
        let manifest = ModManifest::from_file(&manifest_path);

        let mut dm = build_discovered_mod(&manifest, &folder_name, source_archive, source_type);
        dm.status_message = Some("散文件结构，将自动包裹为文件夹".to_string());
        return vec![dm];
    }

    // 无法识别
    vec![DiscoveredMod {
        mod_id: "unknown".to_string(),
        name: format!("从 {}", source_archive.unwrap_or("未知来源")),
        version: None,
        author: None,
        folder_name: "unknown".to_string(),
        status: DiscoveredModStatus::UnsupportedFormat,
        conflicts: Vec::new(),
        status_message: Some(
            "归档结构无法识别，请确认包含 manifest.json 或 mods/ 结构".to_string(),
        ),
        source_archive: source_archive.map(|s| s.to_string()),
        source_type,
    }]
}

fn build_discovered_mod(
    manifest: &Option<ModManifest>,
    folder_name: &str,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> DiscoveredMod {
    let mod_id = manifest
        .as_ref()
        .and_then(|m| m.id.clone())
        .unwrap_or_else(|| format!("unknown:{}", folder_name));
    let name = manifest
        .as_ref()
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| folder_name.to_string());

    DiscoveredMod {
        mod_id,
        name,
        version: manifest.as_ref().and_then(|m| m.version.clone()),
        author: manifest.as_ref().and_then(|m| m.author.clone()),
        folder_name: folder_name.to_string(),
        status: DiscoveredModStatus::Ready,
        conflicts: Vec::new(),
        status_message: None,
        source_archive: source_archive.map(|s| s.to_string()),
        source_type,
    }
}

// ---------------------------------------------------------------------------
// 冲突检测
// ---------------------------------------------------------------------------

/// 对发现的 Mod 进行冲突检测（同时检查启用和禁用目录，ID 大小写不敏感）
pub fn detect_conflicts(
    discovered: &mut [DiscoveredMod],
    enabled_mods: &[InstalledMod],
    disabled_mods: &[InstalledMod],
) {
    for dmod in discovered.iter_mut() {
        if !matches!(dmod.status, DiscoveredModStatus::Ready) {
            continue;
        }
        let all_installed = enabled_mods.iter().chain(disabled_mods.iter());
        for installed in all_installed {
            // ID 冲突（大小写不敏感）
            if dmod.mod_id.to_lowercase() == installed.id.to_lowercase() {
                dmod.conflicts.push(format!(
                    "与已安装 Mod「{}」(id={}) ID 冲突",
                    installed.name, installed.id
                ));
            }
            // 文件夹名冲突
            if dmod.folder_name.to_lowercase() == installed.folder_name.to_lowercase() {
                let msg = format!(
                    "与已安装 Mod「{}」文件夹名冲突: {}",
                    installed.name, installed.folder_name
                );
                if !dmod.conflicts.contains(&msg) {
                    dmod.conflicts.push(msg);
                }
            }
        }
        if !dmod.conflicts.is_empty() {
            dmod.status = DiscoveredModStatus::Conflict;
        }
    }
}

/// 便捷方法：自动扫描启用+禁用目录进行冲突检测
/// 如果提供 extracted_root，额外检查文件级哈希冲突
pub fn detect_conflicts_full(
    discovered: &mut [DiscoveredMod],
    game_root: &Path,
    extracted_root: Option<&Path>,
) {
    let enabled = mod_service::scan_enabled_mods(game_root);
    let disabled = mod_service::scan_disabled_mods(game_root);
    detect_conflicts(discovered, &enabled, &disabled);

    // 文件哈希级冲突检测
    if let Some(extracted) = extracted_root {
        for dmod in discovered.iter_mut() {
            if !matches!(dmod.status, DiscoveredModStatus::Ready) {
                continue;
            }
            if let Ok(source_folder) = find_source_mod_folder(extracted, &dmod.folder_name)
                && let Ok(hashes) = mod_service::compute_mod_hashes(&source_folder) {
                    let file_conflicts = mod_service::find_file_conflicts(&hashes);
                    for conflict in file_conflicts {
                        if !dmod.conflicts.contains(&conflict) {
                            dmod.conflicts.push(conflict);
                        }
                    }
                    if !dmod.conflicts.is_empty() {
                        dmod.status = DiscoveredModStatus::Conflict;
                    }
                }
        }
    }
}

// ---------------------------------------------------------------------------
// 安装
// ---------------------------------------------------------------------------

/// 将发现的 Mod 从解压目录安装到 plugins/ 或 mods_disabled/
/// 仅安装 selected_ids 中勾选的 Mod（空列表 = 安装全部）
pub fn install_discovered_mods(
    extracted_root: &Path,
    discovered: &[DiscoveredMod],
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
    selected_ids: &[String],
    _source_archive: Option<&str>,
) -> Result<Vec<InstalledMod>, AppError> {
    let target_dir = if enable {
        game_root.join("mods")
    } else {
        game_root.join("mods_disabled")
    };
    std::fs::create_dir_all(&target_dir).map_err(AppError::Io)?;

    let filter_by_selection = !selected_ids.is_empty();
    let mut installed = Vec::new();

    for dmod in discovered {
        if !matches!(
            dmod.status,
            DiscoveredModStatus::Ready | DiscoveredModStatus::Conflict
        ) {
            continue;
        }

        // 用户勾选过滤（仅当有 selected_ids 时启用）
        if filter_by_selection && !selected_ids.contains(&dmod.mod_id) {
            continue;
        }

        // 检查冲突解析策略
        let resolution = resolutions
            .iter()
            .find(|(id, _)| id == &dmod.mod_id)
            .map(|(_, r)| r)
            .unwrap_or(&ConflictResolution::Skip);

        if matches!(resolution, ConflictResolution::Skip) && !dmod.conflicts.is_empty() {
            continue;
        }

        let source_folder = find_source_mod_folder(extracted_root, &dmod.folder_name)?;
        let dest_folder_name = dmod.folder_name.clone();

        let dest_path = target_dir.join(&dest_folder_name);

        // ① 按文件夹名删除旧目录（新旧同名时直接覆盖）
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).map_err(AppError::Io)?;
        }

        // ② 按 mod_id 查找并删除旧 Mod 文件夹（新旧不同名时生效）
        if matches!(resolution, ConflictResolution::Replace) && !dmod.conflicts.is_empty() {
            for base in [game_root.join("mods"), game_root.join("mods_disabled")] {
                if let Ok(old_path) = mod_service::find_mod_folder(&base, &dmod.mod_id)
                    && old_path != dest_path {
                        let _ = std::fs::remove_dir_all(&old_path);
                    }
            }
        }

        save_service::copy_dir_recursive(&source_folder, &dest_path)?;

        let dest_path = target_dir.join(&dest_folder_name);

        // 使用 ModManifest::find_in_dir 兼容多种 manifest 文件名
        let (manifest_path, manifest): (Option<PathBuf>, Option<ModManifest>) =
            ModManifest::find_in_dir(&dest_path)
                .map(|(p, m)| (Some(p), Some(m)))
                .unwrap_or((None, None));
        // 计算并存储文件哈希
        let mod_id_for_hash = manifest
            .as_ref()
            .and_then(|m| m.id.clone())
            .unwrap_or_else(|| dmod.mod_id.clone());
        if let Ok(hashes) = mod_service::compute_mod_hashes(&dest_path) {
            let _ = crate::repositories::mod_hashes_repo::set_mod_hashes(&mod_id_for_hash, hashes);
        }

        let state = if enable {
            InstalledModState::Enabled
        } else {
            InstalledModState::Disabled
        };

        installed.push(InstalledMod {
            id: manifest
                .as_ref()
                .and_then(|m| m.id.clone())
                .unwrap_or_else(|| dmod.mod_id.clone()),
            name: manifest
                .as_ref()
                .and_then(|m| m.name.clone())
                .unwrap_or_else(|| dmod.name.clone()),
            version: manifest.as_ref().and_then(|m| m.version.clone()),
            author: manifest.as_ref().and_then(|m| m.author.clone()),
            folder_name: dmod.folder_name.clone(),
            install_dir: dest_path.to_string_lossy().to_string(),
            manifest_path: manifest_path.map(|p| p.to_string_lossy().to_string()),
            affects_gameplay: manifest
                .as_ref()
                .map(|m| m.affects_gameplay)
                .unwrap_or(false),
            state,
            source: "local".to_string(),
            workshop_id: None,
        });
    }

    Ok(installed)
}

fn find_source_mod_folder(extracted_root: &Path, folder_name: &str) -> Result<PathBuf, AppError> {
    let direct = extracted_root.join(folder_name);
    if direct.is_dir() {
        return Ok(direct);
    }
    let mods_dir = extracted_root
        .join("mods")
        .join(folder_name);
    if mods_dir.is_dir() {
        return Ok(mods_dir);
    }
    if let Some(found) = find_dir_recursive(extracted_root, folder_name) {
        return Ok(found);
    }
    // 回退：松散文件直接位于解压根目录（如 StS2 标准 Mod ZIP）
    if extracted_root.is_dir() {
        return Ok(extracted_root.to_path_buf());
    }
    Err(AppError::Other(format!("在解压内容中找不到 Mod 文件夹: {}", folder_name)))
}

fn find_dir_recursive(root: &Path, target: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some(target) {
            return Some(path);
        }
        if let Some(found) = find_dir_recursive(&path, target) {
            return Some(found);
        }
    }
    None
}

/// 清理临时解压目录
pub fn cleanup_extract_dir(temp_dir: &Path) {
    let _ = std::fs::remove_dir_all(temp_dir);
}

// ---------------------------------------------------------------------------
// 高层流程：预览 + 安装
// ---------------------------------------------------------------------------

/// 预览单个归档：解压 → 发现 → 冲突检测 → 返回预览
pub fn preview_archive(
    archive_path: &Path,
    game_root: &Path,
) -> Result<BatchImportPreview, AppError> {
    let archive_name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let temp_dir = extract_archive(archive_path)?;
    let mut discovered =
        discover_mods_in_dir(&temp_dir, Some(archive_name), DiscoveredModSourceType::Archive);
    detect_conflicts_full(&mut discovered, game_root, None);
    cleanup_extract_dir(&temp_dir);

    Ok(BatchImportPreview {
        total_targets_scanned: 1,
        discovered_mods: discovered,
    })
}

/// 执行安装：解压 → 发现 → 冲突检测 → 安装 → 清理
pub fn execute_install(
    archive_path: &Path,
    game_root: &Path,
    enable: bool,
    replace_existing: bool,
) -> Result<Vec<InstalledMod>, AppError> {
    let archive_name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let temp_dir = extract_archive(archive_path)?;
    let mut discovered =
        discover_mods_in_dir(&temp_dir, Some(archive_name), DiscoveredModSourceType::Archive);
    detect_conflicts_full(&mut discovered, game_root, None);

    let default_resolution = if replace_existing {
        ConflictResolution::Replace
    } else {
        ConflictResolution::Skip
    };
    let resolutions: Vec<_> = discovered
        .iter()
        .map(|d| (d.mod_id.clone(), default_resolution.clone()))
        .collect();

    let result = install_discovered_mods(
        &temp_dir,
        &discovered,
        game_root,
        enable,
        &resolutions,
        &[],               // 无勾选过滤 → 安装全部
        Some(archive_name),
    )?;
    cleanup_extract_dir(&temp_dir);
    Ok(result)
}

/// 批量预览：多个归档/文件夹 → 扫描 → 冲突检测
pub fn batch_preview(
    paths: &[String],
    game_root: &Path,
) -> Result<BatchImportPreview, AppError> {
    let total = paths.len() as u32;
    let mut all_discovered = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() {
            all_discovered.push(DiscoveredMod {
                mod_id: format!("error:{}", path_str),
                name: format!("{} (未找到)", path_str),
                version: None,
                author: None,
                folder_name: String::new(),
                status: DiscoveredModStatus::Error,
                conflicts: Vec::new(),
                status_message: Some("文件或目录不存在".to_string()),
                source_archive: Some(path_str.clone()),
                source_type: DiscoveredModSourceType::Archive,
            });
            continue;
        }

        if path.is_dir() {
            let mut discovered = discover_mods_in_dir(
                path,
                Some(path_str),
                DiscoveredModSourceType::Folder,
            );
            detect_conflicts_full(&mut discovered, game_root, None);
            all_discovered.extend(discovered);
        } else if detect_archive_format(path).is_some() {
            match extract_archive(path) {
                Ok(temp_dir) => {
                    let archive_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path_str);
                    let mut discovered = discover_mods_in_dir(
                        &temp_dir,
                        Some(archive_name),
                        DiscoveredModSourceType::Archive,
                    );
                    detect_conflicts_full(&mut discovered, game_root, None);
                    all_discovered.extend(discovered);
                    cleanup_extract_dir(&temp_dir);
                }
                Err(e) => {
                    all_discovered.push(DiscoveredMod {
                        mod_id: format!("error:{}", path_str),
                        name: format!("{} (解压失败)", path_str),
                        version: None,
                        author: None,
                        folder_name: String::new(),
                        status: DiscoveredModStatus::Error,
                        conflicts: Vec::new(),
                        status_message: Some(format!("解压失败: {}", e)),
                        source_archive: Some(path_str.clone()),
                        source_type: DiscoveredModSourceType::Archive,
                    });
                }
            }
        } else {
            all_discovered.push(DiscoveredMod {
                mod_id: format!("error:{}", path_str),
                name: format!("{} (不支持的格式)", path_str),
                version: None,
                author: None,
                folder_name: String::new(),
                status: DiscoveredModStatus::UnsupportedFormat,
                conflicts: Vec::new(),
                status_message: Some("不支持的格式，仅支持 .zip / .7z / .rar 归档和文件夹".to_string()),
                source_archive: Some(path_str.clone()),
                source_type: DiscoveredModSourceType::Archive,
            });
        }
    }

    Ok(BatchImportPreview {
        total_targets_scanned: total,
        discovered_mods: all_discovered,
    })
}

/// 批量安装：仅安装 selected_ids 中勾选的 Mod（空列表 = 安装全部）
pub fn batch_install(
    app_handle: &tauri::AppHandle,
    paths: &[String],
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
    selected_ids: &[String],
) -> Result<BatchInstallResult, AppError> {
    #[derive(Serialize, Clone)]
    struct InstallProgress<'a> {
        current: usize,
        total: usize,
        name: &'a str,
        status: &'a str,
    }

    let total = paths.len();
    let mut results = Vec::new();
    let mut success_count = 0u32;
    let mut failure_count = 0u32;

    for (i, path_str) in paths.iter().enumerate() {
        let path = Path::new(path_str);

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path_str);

        // 发送进度事件
        let _ = app_handle.emit("install-progress", InstallProgress {
            current: i + 1,
            total,
            name: file_name,
            status: "installing",
        });

        if !path.exists() {
            failure_count += 1;
            results.push(BatchInstallItem {
                mod_id: path_str.clone(),
                name: path_str.clone(),
                success: false,
                error_message: Some("文件不存在".to_string()),
            });
            continue;
        }

        let outcome = if path.is_dir() {
            install_from_folder(path, game_root, enable, resolutions, selected_ids)
        } else if detect_archive_format(path).is_some() {
            install_single_archive(path, game_root, enable, resolutions, selected_ids)
        } else {
            Err(AppError::Other("不支持的格式".to_string()))
        };

        match outcome {
            Ok(items) => {
                for item in &items {
                    if item.success {
                        success_count += 1;
                    } else {
                        failure_count += 1;
                    }
                    results.push(item.clone());
                }
            }
            Err(e) => {
                failure_count += 1;
                results.push(BatchInstallItem {
                    mod_id: path_str.clone(),
                    name: path_str.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }

    // 发送完成事件
    let _ = app_handle.emit("install-progress", InstallProgress {
        current: total,
        total,
        name: "",
        status: "done",
    });

    Ok(BatchInstallResult {
        success_count,
        failure_count,
        results,
    })
}

fn install_from_folder(
    folder_path: &Path,
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
    selected_ids: &[String],
) -> Result<Vec<BatchInstallItem>, AppError> {
    let folder_name = folder_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let mut discovered = discover_mods_in_dir(
        folder_path,
        Some(folder_name),
        DiscoveredModSourceType::Folder,
    );
    detect_conflicts_full(&mut discovered, game_root, None);

    let mods = install_discovered_mods(
        folder_path,
        &discovered,
        game_root,
        enable,
        resolutions,
        selected_ids,
        Some(folder_name),
    )?;

    Ok(mods
        .into_iter()
        .map(|m| BatchInstallItem {
            mod_id: m.id,
            name: m.name,
            success: true,
            error_message: None,
        })
        .collect())
}

fn install_single_archive(
    archive_path: &Path,
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
    selected_ids: &[String],
) -> Result<Vec<BatchInstallItem>, AppError> {
    let archive_name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let temp_dir = extract_archive(archive_path)?;
    let mut discovered = discover_mods_in_dir(
        &temp_dir,
        Some(archive_name),
        DiscoveredModSourceType::Archive,
    );
    detect_conflicts_full(&mut discovered, game_root, None);

    let mods = install_discovered_mods(
        &temp_dir,
        &discovered,
        game_root,
        enable,
        resolutions,
        selected_ids,
        Some(archive_name),
    )?;
    cleanup_extract_dir(&temp_dir);

    Ok(mods
        .into_iter()
        .map(|m| BatchInstallItem {
            mod_id: m.id,
            name: m.name,
            success: true,
            error_message: None,
        })
        .collect())
}
