use crate::app::state::{AppState, SaveSyncPair};
use crate::domain::mod_entity::{
    BatchImportPreview, BatchInstallResult, ConflictResolution, InstalledMod, ModToggleResult,
};
use crate::domain::profile::{ApplyProfileResult, ModProfile};
use crate::domain::save::{
    CloudSaveDiffEntry, CloudSaveStatus, SaveBackupEntry, SaveKind, SaveSlot, SaveSlotRef,
    SaveSyncResult, SaveTransferPreview,
};
use crate::domain::remote_mod::RemoteModSearchResult;
use crate::domain::task::ActivityLogEntry;
use crate::repositories::settings_repo;
use crate::services::{
    backup_service, discover_service, game_service, mod_service, profile_service, save_service,
};
use crate::workflows::install_archive_workflow;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

// --- DTOs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub app_name: String,
    pub app_version: String,
    pub game_directory: Option<String>,
    pub game_directory_valid: bool,
    pub installed_count: usize,
    pub disabled_count: usize,
    pub active_profile_name: String,
    pub locale: String,
    pub save_auto_sync: bool,
    pub save_sync_pairs: Vec<SaveSyncPair>,
    pub nexus_api_key: Option<String>,
    pub nexus_is_premium: bool,
    pub nexus_user_name: Option<String>,
    pub proxy_url: Option<String>,
    pub auto_backup_keep_count: usize,
    pub theme_mode: String,
    pub theme_color: String,
    pub launch_mode: String,
    pub launch_check_cloud_save: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInstall {
    pub root_dir: String,
    pub found_from: String,
}

// =========================================================================
// 5.1 启动与初始化
// =========================================================================

#[tauri::command]
pub fn get_app_bootstrap(state: State<AppState>) -> AppBootstrap {
    let settings = state.settings.read().unwrap();

    // 优先检查 exe 文件存在（游戏是否安装），再检查 mods（Mod 管理是否就绪）
    let game_dir_valid = settings
        .game_root_dir
        .as_ref()
        .map(|d| game_service::contains_game_executable(Path::new(d)))
        .unwrap_or(false);

    // 统计已安装/禁用的 Mod 数量（只有 mods 就绪时才有 Mod）
    let (installed_count, disabled_count) = if let Some(ref root) = settings.game_root_dir {
        if game_service::has_mods_dir(Path::new(root)) {
            let game_root = Path::new(root);
            (
                mod_service::scan_enabled_mods(game_root).len(),
                mod_service::scan_disabled_mods(game_root).len(),
            )
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    AppBootstrap {
        app_name: "SlayMuManager".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        game_directory: settings.game_root_dir.clone(),
        game_directory_valid: game_dir_valid,
        installed_count,
        disabled_count,
        active_profile_name: settings.active_profile_name.clone(),
        locale: settings.locale.clone(),
        save_auto_sync: settings.save_auto_sync,
        save_sync_pairs: settings.save_sync_pairs.clone(),
        nexus_api_key: settings.nexus_api_key.clone(),
        nexus_is_premium: settings.nexus_is_premium,
        nexus_user_name: settings.nexus_user_name.clone(),
        proxy_url: settings.proxy_url.clone(),
        auto_backup_keep_count: settings.auto_backup_keep_count,
        theme_mode: settings.theme_mode.clone(),
        theme_color: settings.theme_color.clone(),
        launch_mode: settings.launch_mode.clone(),
        launch_check_cloud_save: settings.launch_check_cloud_save,
    }
}

#[tauri::command]
pub fn detect_game_install(state: State<AppState>) -> Option<GameInstall> {
    // 1. 先检查设置中已有的路径（验证 exe 存在）
    let has_no_config = {
        let settings = state.settings.read().unwrap();
        if let Some(ref saved) = settings.game_root_dir {
            let p = Path::new(saved);
            if game_service::contains_game_executable(p) {
                return Some(GameInstall {
                    root_dir: saved.clone(),
                    found_from: "Settings".to_string(),
                });
            }
        }
        settings.game_root_dir.is_none()
    };

    // 2. 自动扫描
    let results = game_service::auto_detect_game_install();
    if let Some((path, source)) = results.into_iter().next() {
        let root_dir = path.to_string_lossy().to_string();

        // 首次自动检测到 → 自动写入 settings.json（参照原项目做法）
        if has_no_config {
            let mut w = state.settings.write().unwrap();
            w.game_root_dir = Some(root_dir.clone());
            let _ = settings_repo::save_settings(&w);
        }

        return Some(GameInstall {
            root_dir,
            found_from: source,
        });
    }

    None
}

#[tauri::command]
pub fn update_game_root_dir(root_dir: String, state: State<AppState>) -> AppBootstrap {
    {
        let mut settings = state.settings.write().unwrap();
        settings.game_root_dir = Some(root_dir);
        // 持久化
        let _ = settings_repo::save_settings(&settings);
    }
    get_app_bootstrap(state)
}

#[tauri::command]
pub fn update_app_locale(locale: String, state: State<AppState>) {
    let mut settings = state.settings.write().unwrap();
    settings.locale = locale;
    let _ = settings_repo::save_settings(&settings);
}

// =========================================================================
// 5.2 游戏操作
// =========================================================================

#[tauri::command]
pub fn launch_game(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();

    if settings.launch_mode == "direct" {
        // 直接启动 exe
        let exe_path = settings
            .game_root_dir
            .as_ref()
            .map(|d| std::path::Path::new(d).join("SlayTheSpire2.exe"))
            .filter(|p| p.exists())
            .ok_or_else(|| "游戏路径未设置或 SlayTheSpire2.exe 不存在".to_string())?;

        std::process::Command::new(&exe_path)
            .spawn()
            .map_err(|e| format!("启动游戏失败: {}", e))?;
        return Ok(());
    }

    // Steam 协议启动
    let steam_url = "steam://rungameid/2868840";

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", steam_url])
            .spawn()
            .map_err(|e| format!("启动游戏失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(steam_url)
            .spawn()
            .map_err(|e| format!("启动游戏失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(steam_url)
            .spawn()
            .map_err(|e| format!("启动游戏失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_path_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    std::process::Command::new("explorer")
        .arg(p)
        .spawn()
        .map_err(|e| format!("打开资源管理器失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn open_url_in_browser(url: String) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .spawn()
        .map_err(|e| format!("打开浏览器失败: {}", e))?;
    Ok(())
}

// =========================================================================
// 5.3 Mod 操作
// =========================================================================

#[tauri::command]
pub fn list_installed_mods(state: State<AppState>) -> Result<Vec<InstalledMod>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    let root = Path::new(game_root);

    if !game_service::has_mods_dir(root) {
        return Err("游戏目录无效（缺少 mods/ 目录）".to_string());
    }

    Ok(mod_service::scan_enabled_mods(root))
}

#[tauri::command]
pub fn list_disabled_mods(state: State<AppState>) -> Result<Vec<InstalledMod>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    let root = Path::new(game_root);

    if !game_service::has_mods_dir(root) {
        return Err("游戏目录无效（缺少 mods/ 目录）".to_string());
    }

    Ok(mod_service::scan_disabled_mods(root))
}

#[tauri::command]
pub fn enable_mod(mod_id: String, state: State<AppState>) -> Result<ModToggleResult, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let sync_pairs = settings.save_sync_pairs.clone();
    let result = mod_service::enable_mod(Path::new(game_root), &mod_id, &sync_pairs)
        .map_err(|e| e.to_string())?;

    // 如果当前有激活的非内置预设，自动将 mod_id 加入其列表
    let active_name = settings.active_profile_name.clone();
    drop(settings);
    if !active_name.is_empty() {
        let profiles = profile_service::list_profiles();
        if let Some(active_profile) = profiles.iter().find(|p| p.name == active_name) {
            if !active_profile.builtin && !active_profile.mod_ids.contains(&mod_id) {
                let mut new_ids = active_profile.mod_ids.clone();
                new_ids.push(mod_id.clone());
                let _ = profile_service::update_profile(
                    active_profile.id.clone(),
                    active_profile.name.clone(),
                    active_profile.description.clone(),
                    new_ids,
                );
            }
        }
    }

    // 记录活动日志
    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("启用 Mod: {}", result.mod_item.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

#[tauri::command]
pub fn disable_mod(mod_id: String, state: State<AppState>) -> Result<ModToggleResult, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let sync_pairs = settings.save_sync_pairs.clone();
    let result = mod_service::disable_mod(Path::new(game_root), &mod_id, &sync_pairs)
        .map_err(|e| e.to_string())?;

    // 如果当前有激活的非内置预设，自动将 mod_id 从其列表移除
    let active_name = settings.active_profile_name.clone();
    drop(settings);
    if !active_name.is_empty() {
        let profiles = profile_service::list_profiles();
        if let Some(active_profile) = profiles.iter().find(|p| p.name == active_name) {
            if !active_profile.builtin && active_profile.mod_ids.contains(&mod_id) {
                let new_ids: Vec<String> = active_profile
                    .mod_ids
                    .iter()
                    .filter(|id| *id != &mod_id)
                    .cloned()
                    .collect();
                let _ = profile_service::update_profile(
                    active_profile.id.clone(),
                    active_profile.name.clone(),
                    active_profile.description.clone(),
                    new_ids,
                );
            }
        }
    }

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("禁用 Mod: {}", result.mod_item.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

#[tauri::command]
pub fn uninstall_mod(mod_id: String, state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    mod_service::uninstall_mod(Path::new(game_root), &mod_id).map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("卸载 Mod: {}", mod_id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn open_mod_folder(mod_id: String, state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let plugins_dir = Path::new(game_root).join("mods");
    let disabled_dir = Path::new(game_root).join("mods_disabled");

    let mod_folder = mod_service::find_mod_folder(&plugins_dir, &mod_id)
        .or_else(|_| mod_service::find_mod_folder(&disabled_dir, &mod_id))
        .map_err(|e| e.to_string())?;

    std::process::Command::new("explorer")
        .arg(&mod_folder)
        .spawn()
        .map_err(|e| format!("打开文件夹失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn open_mods_directory(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let plugins = game_service::mods_dir(Path::new(game_root));
    std::fs::create_dir_all(&plugins).map_err(|e| format!("创建目录失败: {}", e))?;

    std::process::Command::new("explorer")
        .arg(&plugins)
        .spawn()
        .map_err(|e| format!("打开文件夹失败: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_mod_files(mod_id: String, state: State<AppState>) -> Result<Vec<String>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    mod_service::get_mod_files(Path::new(game_root), &mod_id).map_err(|e| e.to_string())
}

// =========================================================================
// 5.4 ZIP 安装 + 批量导入
// =========================================================================

#[tauri::command]
pub async fn pick_archive_file() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("归档文件", &["zip", "7z"])
        .pick_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn pick_archive_files() -> Result<Vec<String>, String> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter("归档文件", &["zip", "7z"])
        .pick_files()
        .await;
    Ok(files
        .map(|fs| fs.iter().map(|f| f.path().to_string_lossy().to_string()).collect())
        .unwrap_or_default())
}

#[tauri::command]
pub async fn pick_import_folder() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new().pick_folder().await;
    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn pick_game_folder() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new().pick_folder().await;
    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub fn preview_install_archive(
    archive_path: String,
    enable_after_install: bool,
    state: State<AppState>,
) -> Result<BatchImportPreview, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let preview = install_archive_workflow::preview_archive(
        Path::new(&archive_path),
        Path::new(game_root),
    )
    .map_err(|e| e.to_string())?;

    let _ = enable_after_install;

    Ok(preview)
}

#[tauri::command]
pub fn install_archive(
    archive_path: String,
    enable_after_install: bool,
    replace_existing: bool,
    state: State<AppState>,
) -> Result<Vec<InstalledMod>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let installed = install_archive_workflow::execute_install(
        Path::new(&archive_path),
        Path::new(game_root),
        enable_after_install,
        replace_existing,
    )
    .map_err(|e| e.to_string())?;

    // 记录日志
    let names: Vec<_> = installed.iter().map(|m| m.name.as_str()).collect();
    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("安装 Mod: {}", names.join(", ")),
        detail: Some(format!("来源: {}", archive_path)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(installed)
}

#[tauri::command]
pub fn process_import_targets(
    paths: Vec<String>,
    enable_now: bool,
    state: State<AppState>,
) -> Result<BatchImportPreview, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let preview = install_archive_workflow::batch_preview(&paths, Path::new(game_root))
        .map_err(|e| e.to_string())?;

    let _ = enable_now;
    Ok(preview)
}

#[tauri::command]
pub fn batch_install_mods(
    app_handle: tauri::AppHandle,
    paths: Vec<String>,
    enable_now: bool,
    has_conflicts: bool,
    selected_ids: Vec<String>,
    resolutions: Vec<(String, String)>, // (modId, "skip"|"replace")
    state: State<AppState>,
) -> Result<BatchInstallResult, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let _ = has_conflicts;

    // 转换解析策略
    let parsed_resolutions: Vec<(String, ConflictResolution)> = resolutions
        .into_iter()
        .filter(|(id, _)| selected_ids.contains(id))
        .map(|(id, r)| {
            let cr = match r.as_str() {
                "replace" => ConflictResolution::Replace,
                _ => ConflictResolution::Skip,
            };
            (id, cr)
        })
        .collect();

    let result = install_archive_workflow::batch_install(
        &app_handle,
        &paths,
        Path::new(game_root),
        enable_now,
        &parsed_resolutions,
        &selected_ids,
    )
    .map_err(|e| e.to_string())?;

    // Save Guard：从 0 个 Mod → N 个 Mod 时自动触发存档备份 + 同步
    if enable_now && result.success_count > 0 {
        let sync_pairs = settings.save_sync_pairs.clone();
        let installed_count = mod_service::scan_enabled_mods(Path::new(game_root)).len();
        // 仅当安装前 mods/ 为空时才触发（first-time activation）
        if installed_count == result.success_count as usize {
            let _ = save_service::sync_saves(Path::new(game_root), &sync_pairs);
        }
    }

    // 记录日志
    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!(
            "批量导入: {} 成功, {} 失败",
            result.success_count, result.failure_count
        ),
        detail: Some(format!("共 {} 个目标", paths.len())),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.5 预设操作
// =========================================================================

#[tauri::command]
pub fn list_profiles() -> Vec<ModProfile> {
    profile_service::list_profiles()
}

#[tauri::command]
pub fn create_profile(
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
    state: State<AppState>,
) -> Result<ModProfile, String> {
    let profile = profile_service::create_profile(name, description, mod_ids)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("创建预设: {}", profile.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(profile)
}

#[tauri::command]
pub fn update_profile(
    id: String,
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
    state: State<AppState>,
) -> Result<ModProfile, String> {
    let profile = profile_service::update_profile(id, name, description, mod_ids)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("更新预设: {}", profile.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(profile)
}

#[tauri::command]
pub fn delete_profile(id: String, state: State<AppState>) -> Result<(), String> {
    // 禁止删除当前激活的预设
    let is_active = {
        let settings = state.settings.read().unwrap();
        let profiles = profile_service::list_profiles();
        profiles.iter().any(|p| p.id == id && p.name == settings.active_profile_name)
    };
    if is_active {
        return Err("当前激活的预设不能直接删除，请先切换到其他预设".to_string());
    }

    profile_service::delete_profile(&id).map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("删除预设: {}", id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn apply_profile(id: String, state: State<AppState>) -> Result<ApplyProfileResult, String> {
    let (game_root, sync_pairs) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .as_ref()
            .ok_or("游戏目录未设置")?
            .clone();
        let sync_pairs = settings.save_sync_pairs.clone();
        (game_root, sync_pairs)
    };

    let result = profile_service::apply_profile(&id, Path::new(&game_root), &sync_pairs)
        .map_err(|e| e.to_string())?;

    // 同步内存中的 active_profile_name，确保 get_app_bootstrap 返回最新值
    {
        let mut settings = state.settings.write().unwrap();
        settings.active_profile_name = result.profile.name.clone();
    }

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!(
            "应用预设「{}」: 启用 {}, 禁用 {}, 缺失 {}",
            result.profile.name,
            result.enabled_mod_ids.len(),
            result.disabled_mod_ids.len(),
            result.missing_mod_ids.len()
        ),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.6 整合包
// =========================================================================

#[tauri::command]
pub fn export_preset_bundle(
    profile_id: String,
    output_path: String,
    state: State<AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    profile_service::export_bundle(&profile_id, &output_path, Path::new(game_root))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn preview_preset_bundle(
    bundle_path: String,
    state: State<AppState>,
) -> Result<profile_service::BundlePreview, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    profile_service::preview_bundle(&bundle_path, Path::new(game_root))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn confirm_import_preset_bundle(
    bundle_path: String,
    apply_profile: bool,
    resolutions: Vec<(String, String)>,
    state: State<AppState>,
) -> Result<ApplyProfileResult, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let sync_pairs = settings.save_sync_pairs.clone();
    profile_service::import_bundle(
        &bundle_path,
        Path::new(game_root),
        apply_profile,
        &resolutions,
        &sync_pairs,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_preset_bundle() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("整合包", &["zip", "spm"])
        .pick_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

// =========================================================================
// 5.7 存档管理
// =========================================================================

#[tauri::command]
pub fn list_save_slots(state: State<AppState>) -> Result<Vec<SaveSlot>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    Ok(save_service::list_save_slots(Path::new(game_root)))
}

#[tauri::command]
pub fn preview_save_transfer(
    source: SaveSlotRef,
    target: SaveSlotRef,
    state: State<AppState>,
) -> Result<SaveTransferPreview, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    Ok(save_service::preview_save_transfer(
        Path::new(game_root),
        &source,
        &target,
    ))
}

#[tauri::command]
pub fn transfer_save(
    source: SaveSlotRef,
    target: SaveSlotRef,
    state: State<AppState>,
) -> Result<SaveBackupEntry, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let result = save_service::transfer_save(Path::new(game_root), &source, &target)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!(
            "存档传输: {:?} slot{} → {:?} slot{}",
            source.kind, source.slot_index, target.kind, target.slot_index
        ),
        detail: Some(format!("用户: {}", source.steam_user_id)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

#[tauri::command]
pub fn create_save_backup(
    steam_user_id: String,
    kind: SaveKind,
    slot_index: u32,
    reason: String,
    state: State<AppState>,
) -> Result<SaveBackupEntry, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    save_service::create_save_backup(
        Path::new(game_root),
        &steam_user_id,
        &kind,
        slot_index,
        &reason,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_save_backups(
    steam_user_id: Option<String>,
    kind: Option<SaveKind>,
    slot_index: Option<u32>,
    state: State<AppState>,
) -> Result<Vec<SaveBackupEntry>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    Ok(save_service::list_save_backups(
        Path::new(game_root),
        steam_user_id.as_deref(),
        kind.as_ref(),
        slot_index,
    ))
}

#[tauri::command]
pub fn restore_save_backup(
    backup_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    save_service::restore_save_backup(Path::new(game_root), &backup_id)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!("恢复存档备份: {}", backup_id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn delete_save_backup(
    backup_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    save_service::delete_save_backup(Path::new(game_root), &backup_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_save_backup_to_slot(
    backup_id: String,
    target_steam_user_id: String,
    target_kind: String, // "vanilla" | "modded"
    target_slot_index: u32,
    state: State<AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let kind = match target_kind.as_str() {
        "modded" => SaveKind::Modded,
        _ => SaveKind::Vanilla,
    };

    save_service::restore_save_backup_to_slot(
        Path::new(game_root),
        &backup_id,
        &target_steam_user_id,
        &kind,
        target_slot_index,
    )
    .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!(
            "恢复备份 → {} {} 槽位 {}",
            target_steam_user_id, target_kind, target_slot_index
        ),
        detail: Some(format!("备份 ID: {}", backup_id)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn delete_save_slot(
    steam_user_id: String,
    kind: String,
    slot_index: u32,
    state: State<AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;

    let save_kind = match kind.as_str() {
        "modded" => SaveKind::Modded,
        _ => SaveKind::Vanilla,
    };

    save_service::delete_save_slot(Path::new(game_root), &steam_user_id, &save_kind, slot_index)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!("清空存档: {} {} 槽位 {}", kind, steam_user_id, slot_index),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn toggle_save_auto_sync(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.save_auto_sync = enabled;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_save_sync_pairs(
    pairs: Vec<SaveSyncPair>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.save_sync_pairs = pairs;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn sync_saves(state: State<AppState>) -> Result<SaveSyncResult, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    let pairs = settings.save_sync_pairs.clone();

    let result = save_service::sync_saves(Path::new(game_root), &pairs)
        .map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!("存档同步: {} 对已同步", result.synced_count),
        detail: Some(format!("共 {} 对配对", pairs.len())),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.8 Steam 云存档
// =========================================================================

#[tauri::command]
pub fn get_cloud_save_status(state: State<AppState>) -> Result<CloudSaveStatus, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::get_cloud_save_status(Path::new(game_root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_cloud_save_diff_entries(
    state: State<AppState>,
) -> Result<Vec<CloudSaveDiffEntry>, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::list_cloud_save_diff_entries(Path::new(game_root))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_cloud_save_diff_side(
    relative_path: String,
    side: String,
    state: State<AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::copy_cloud_save_diff_side(Path::new(game_root), &relative_path, &side)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ascend_to_cloud_full(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::ascend_to_cloud_full(Path::new(game_root)).map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "cloud".to_string(),
        title: "全量上传到 Steam 云".to_string(),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn descend_from_cloud_full(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::descend_from_cloud_full(Path::new(game_root)).map_err(|e| e.to_string())?;

    let mut activity = state.recent_activity.write().unwrap();
    activity.push(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "cloud".to_string(),
        title: "从 Steam 云全量下载".to_string(),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub fn get_backup_artifact_status(state: State<AppState>) -> Result<bool, String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::get_backup_artifact_status(Path::new(game_root)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cleanup_backup_artifacts(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let game_root = settings
        .game_root_dir
        .as_ref()
        .ok_or("游戏目录未设置")?;
    backup_service::cleanup_backup_artifacts(Path::new(game_root)).map_err(|e| e.to_string())
}

// =========================================================================
// 5.9 Nexus Mods 集成
// =========================================================================

#[tauri::command]
pub fn search_remote_mods(
    query: String,
    page: Option<u32>,
    page_size: Option<u32>,
    sort_by: Option<String>,
) -> Result<RemoteModSearchResult, String> {
    discover_service::search_remote_mods(
        &query,
        page.unwrap_or(1),
        page_size.unwrap_or(18),
        &sort_by.unwrap_or_else(|| "latest_added".to_string()),
    )
    .map_err(|e| e.to_string())
}

// =========================================================================
// 5.10 设置
// =========================================================================

#[tauri::command]
pub fn update_nexus_api_key(api_key: String, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.nexus_api_key = Some(api_key);
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_proxy_url(url: Option<String>, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.proxy_url = url;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn test_proxy(url: String) -> Result<bool, String> {
    crate::integrations::nexus_client::test_proxy(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_auto_backup_keep_count(count: usize, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.auto_backup_keep_count = count;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_theme_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.theme_mode = mode;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_theme_color(color: String, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.theme_color = color;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_launch_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.launch_mode = mode;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_launch_check_cloud_save(check: bool, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.launch_check_cloud_save = check;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

// =========================================================================
// 5.11 日志
// =========================================================================

#[tauri::command]
pub fn list_activity_logs(state: State<AppState>) -> Vec<ActivityLogEntry> {
    state.recent_activity.read().unwrap().clone()
}
