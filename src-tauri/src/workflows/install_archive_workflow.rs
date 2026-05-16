use crate::domain::mod_entity::{
    BatchImportPreview, BatchInstallItem, BatchInstallResult, ConflictResolution,
    DiscoveredMod, DiscoveredModSourceType, DiscoveredModStatus, InstalledMod, InstalledModState,
};
use crate::integrations::manifest::ModManifest;
use crate::services::mod_service;
use crate::utils::error::AppError;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// 归档格式检测
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchiveFormat {
    Zip,
    SevenZ,
}

pub fn detect_archive_format(path: &Path) -> Option<ArchiveFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "zip" => Some(ArchiveFormat::Zip),
        "7z" | "7zip" => Some(ArchiveFormat::SevenZ),
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

// ---------------------------------------------------------------------------
// Mod 发现：在解压内容中查找 Mod 文件夹
// ---------------------------------------------------------------------------

/// 在目录中递归发现所有 Mod 文件夹
/// 策略：mods/ 结构 → manifest.json 文件夹 → 回退推理
pub fn discover_mods_in_dir(
    root: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> Vec<DiscoveredMod> {
    // 1. 检查是否有 mods/ 结构
    let bepinex_plugins = root.join("mods");
    if bepinex_plugins.is_dir() {
        let mods = discover_from_plugins_dir(&bepinex_plugins, source_archive, source_type);
        if !mods.is_empty() {
            return mods;
        }
    }

    // 2. 递归查找所有 manifest.json
    let manifest_mods = find_mod_folders_by_manifest(root, source_archive, source_type);
    if !manifest_mods.is_empty() {
        return manifest_mods;
    }

    // 3. 回退：分析顶层结构
    fallback_discovery(root, source_archive, source_type)
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

fn find_mod_folders_by_manifest(
    root: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
) -> Vec<DiscoveredMod> {
    let mut mods = Vec::new();
    find_manifests_recursive(root, source_archive, source_type, &mut mods);
    mods
}

fn find_manifests_recursive(
    current: &Path,
    source_archive: Option<&str>,
    source_type: DiscoveredModSourceType,
    out: &mut Vec<DiscoveredMod>,
) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join("manifest.json").exists() {
                let folder_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let manifest = ModManifest::from_file(&path.join("manifest.json"));
                out.push(build_discovered_mod(
                    &manifest,
                    &folder_name,
                    source_archive,
                    source_type,
                ));
            } else {
                find_manifests_recursive(&path, source_archive, source_type, out);
            }
        }
    }
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
        mod_id: mod_id,
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
pub fn detect_conflicts_full(discovered: &mut [DiscoveredMod], game_root: &Path) {
    let enabled = mod_service::scan_enabled_mods(game_root);
    let disabled = mod_service::scan_disabled_mods(game_root);
    detect_conflicts(discovered, &enabled, &disabled);
}

// ---------------------------------------------------------------------------
// 安装
// ---------------------------------------------------------------------------

/// 将发现的 Mod 从解压目录安装到 plugins/ 或 mods_disabled/
pub fn install_discovered_mods(
    extracted_root: &Path,
    discovered: &[DiscoveredMod],
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
    _source_archive: Option<&str>,
) -> Result<Vec<InstalledMod>, AppError> {
    let target_dir = if enable {
        game_root.join("mods")
    } else {
        game_root.join("mods_disabled")
    };
    std::fs::create_dir_all(&target_dir).map_err(AppError::Io)?;

    let mut installed = Vec::new();

    for dmod in discovered {
        if !matches!(
            dmod.status,
            DiscoveredModStatus::Ready | DiscoveredModStatus::Conflict
        ) {
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
        let mut dest_folder_name = dmod.folder_name.clone();

        // Rename 策略：先复制到临时位置，重写 manifest id，再移动到目标
        if matches!(resolution, ConflictResolution::Rename) && !dmod.conflicts.is_empty() {
            let suffix = format!("_{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("0"));
            let new_id = format!("{}{}", dmod.mod_id, suffix);
            // 使用临时目录存放重命名后的 Mod
            let temp_rename = extracted_root.join(format!("_rename_{}", &new_id));
            copy_dir_recursive(&source_folder, &temp_rename)?;
            if let Err(e) = ModManifest::rewrite_manifest_id(&temp_rename, &new_id) {
                let _ = std::fs::remove_dir_all(&temp_rename);
                return Err(AppError::Other(format!("重命名失败: {}", e)));
            }
            dest_folder_name = format!("{}{}", dmod.folder_name, suffix);
            let dest_path = target_dir.join(&dest_folder_name);
            if dest_path.exists() {
                std::fs::remove_dir_all(&dest_path).map_err(AppError::Io)?;
            }
            copy_dir_recursive(&temp_rename, &dest_path)?;
            let _ = std::fs::remove_dir_all(&temp_rename);
        } else {
            let dest_path = target_dir.join(&dest_folder_name);
            if dest_path.exists() {
                std::fs::remove_dir_all(&dest_path).map_err(AppError::Io)?;
            }
            copy_dir_recursive(&source_folder, &dest_path)?;
        }

        let dest_path = target_dir.join(&dest_folder_name);

        let manifest_path = dest_path.join("manifest.json");
        let manifest = ModManifest::from_file(&manifest_path);
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
            manifest_path: if manifest_path.exists() {
                Some(manifest_path.to_string_lossy().to_string())
            } else {
                None
            },
            affects_gameplay: manifest
                .as_ref()
                .map(|m| m.affects_gameplay)
                .unwrap_or(false),
            state,
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
    find_dir_recursive(extracted_root, folder_name).ok_or_else(|| {
        AppError::Other(format!("在解压内容中找不到 Mod 文件夹: {}", folder_name))
    })
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), AppError> {
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
    detect_conflicts_full(&mut discovered, game_root);
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
    detect_conflicts_full(&mut discovered, game_root);

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
            detect_conflicts_full(&mut discovered, game_root);
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
                    detect_conflicts_full(&mut discovered, game_root);
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
                status_message: Some("不支持的格式，仅支持 .zip / .7z 归档和文件夹".to_string()),
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

/// 批量安装：对所有 ready 状态的 Mod 执行安装
pub fn batch_install(
    app_handle: &tauri::AppHandle,
    paths: &[String],
    game_root: &Path,
    enable: bool,
    resolutions: &[(String, ConflictResolution)],
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
            install_from_folder(path, game_root, enable, resolutions)
        } else if detect_archive_format(path).is_some() {
            install_single_archive(path, game_root, enable, resolutions)
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
    detect_conflicts_full(&mut discovered, game_root);

    let mods = install_discovered_mods(
        folder_path,
        &discovered,
        game_root,
        enable,
        resolutions,
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
    detect_conflicts_full(&mut discovered, game_root);

    let mods = install_discovered_mods(
        &temp_dir,
        &discovered,
        game_root,
        enable,
        resolutions,
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
