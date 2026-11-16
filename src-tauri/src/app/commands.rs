use crate::app::state::{AppSettings, AppState, SaveSyncPair};
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
use crate::repositories::updates_cache_repo;
use crate::services::{
    backup_service, discover_service, download_service, game_service, mod_service, profile_service, save_service,
};
use crate::utils::error::AppError;
use crate::workflows::{install_archive_workflow, update_check};
use crate::workflows::update_check::ModUpdateInfo;
use crate::workflows::update_check::ModUpdateCheckCache;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;
use tauri::Manager;
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
    pub active_profile_id: Option<String>,
    pub locale: String,
    pub save_auto_sync: bool,
    pub save_sync_pairs: Vec<SaveSyncPair>,
    pub nexus_api_key: Option<String>,
    pub nexus_is_premium: bool,
    pub nexus_user_name: Option<String>,
    pub proxy_url: Option<String>,
    pub auto_backup_keep_count: usize,
    pub backup_on_path_switch: bool,
    pub theme_mode: String,
    pub theme_color: String,
    pub launch_mode: String,
    pub launch_check_cloud_save: bool,
    pub vanilla_launch: bool,
    pub has_workshop_mods: bool,
    pub auto_check_update: bool,
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

fn build_app_bootstrap(settings: &AppSettings) -> AppBootstrap {
    // 优先检查 exe 文件存在（游戏是否安装），再检查 mods（Mod 管理是否就绪）
    let game_dir_valid = settings
        .game_root_dir
        .as_ref()
        .map(|d| game_service::contains_game_executable(Path::new(d)))
        .unwrap_or(false);

    // 统计已安装/禁用的 Mod 数量（含创意工坊）
    let (installed_count, disabled_count, has_workshop_mods) =
        if let Some(ref root) = settings.game_root_dir {
            let game_root = Path::new(root);
            let enabled = mod_service::scan_enabled_mods(game_root);
            let disabled = if game_service::has_mods_dir(game_root) {
                mod_service::scan_disabled_mods(game_root)
            } else {
                Vec::new()
            };
            let workshop = enabled.iter().any(|m| m.source == "workshop");
            (enabled.len(), disabled.len(), workshop)
        } else {
            (0, 0, false)
        };

    AppBootstrap {
        app_name: "SlayMuManager".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        game_directory: settings.game_root_dir.clone(),
        game_directory_valid: game_dir_valid,
        installed_count,
        disabled_count,
        active_profile_name: settings.active_profile_name.clone(),
        active_profile_id: {
            let profiles = crate::repositories::profile_repo::load_profiles();
            profiles.iter().find(|p| p.name == settings.active_profile_name).map(|p| p.id.clone())
        },
        locale: settings.locale.clone(),
        save_auto_sync: settings.save_auto_sync,
        save_sync_pairs: settings.save_sync_pairs.clone(),
        nexus_api_key: settings.nexus_api_key.clone(),
        nexus_is_premium: settings.nexus_is_premium,
        nexus_user_name: settings.nexus_user_name.clone(),
        proxy_url: settings.proxy_url.clone(),
        auto_backup_keep_count: settings.auto_backup_keep_count,
        backup_on_path_switch: settings.backup_on_path_switch,
        theme_mode: settings.theme_mode.clone(),
        theme_color: settings.theme_color.clone(),
        launch_mode: settings.launch_mode.clone(),
        launch_check_cloud_save: settings.launch_check_cloud_save,
        vanilla_launch: settings.vanilla_launch,
        has_workshop_mods,
        auto_check_update: settings.auto_check_update,
    }
}

#[tauri::command]
pub async fn get_app_bootstrap(state: State<'_, AppState>) -> Result<AppBootstrap, String> {
    let settings = state.settings.read().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || build_app_bootstrap(&settings))
        .await
        .map_err(|e| format!("后台任务失败: {}", e))
}

#[tauri::command]
pub async fn detect_game_install(state: State<'_, AppState>) -> Result<Option<GameInstall>, String> {
    // 1. 先检查设置中已有的路径（验证 exe 存在）
    let (saved_dir, has_no_config) = {
        let settings = state.settings.read().unwrap();
        (settings.game_root_dir.clone(), settings.game_root_dir.is_none())
    };

    let detected = tauri::async_runtime::spawn_blocking(move || {
        if let Some(ref saved) = saved_dir {
            let p = Path::new(saved);
            if game_service::contains_game_executable(p) {
                return Some((saved.clone(), "Settings".to_string(), false));
            }
        }

        let results = game_service::auto_detect_game_install();
        results.into_iter().next().map(|(path, source)| {
            (path.to_string_lossy().to_string(), source, has_no_config)
        })
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?;

    if let Some((root_dir, found_from, should_save)) = detected {
        // 首次自动检测到 → 自动写入 settings.json（参照原项目做法）
        if should_save {
            let mut w = state.settings.write().unwrap();
            if w.game_root_dir.is_none() {
                w.game_root_dir = Some(root_dir.clone());
                let _ = settings_repo::save_settings(&w);
            }
        }
        return Ok(Some(GameInstall { root_dir, found_from }));
    }

    Ok(None)
}

#[tauri::command]
pub async fn update_game_root_dir(
    root_dir: String,
    state: State<'_, AppState>,
) -> Result<AppBootstrap, String> {
    let settings = {
        let mut settings = state.settings.write().unwrap();
        settings.game_root_dir = if root_dir.is_empty() { None } else { Some(root_dir) };
        // 持久化
        let _ = settings_repo::save_settings(&settings);
        settings.clone()
    };
    tauri::async_runtime::spawn_blocking(move || build_app_bootstrap(&settings))
        .await
        .map_err(|e| format!("后台任务失败: {}", e))
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

/// 检查游戏进程是否正在运行
#[tauri::command]
pub async fn is_game_running() -> bool {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq SlayTheSpire2.exe", "/NH"])
                .output()
                .ok()
                .is_some_and(|o| {
                    let out = String::from_utf8_lossy(&o.stdout);
                    out.contains("SlayTheSpire2.exe")
                })
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("pgrep")
                .arg("-x")
                .arg("SlayTheSpire2")
                .output()
                .ok()
                .map_or(false, |o| o.status.success())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("pgrep")
                .arg("-x")
                .arg("SlayTheSpire2")
                .output()
                .ok()
                .map_or(false, |o| o.status.success())
        }
    })
    .await
    .unwrap_or(false)
}

#[tauri::command]
pub fn launch_game(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.read().unwrap();
    let vanilla = settings.vanilla_launch;
    let mode = settings.launch_mode.clone();
    let exe_dir = settings.game_root_dir.clone();
    drop(settings);

    if mode == "direct" {
        let exe_path = exe_dir
            .as_ref()
            .map(|d| std::path::Path::new(d).join("SlayTheSpire2.exe"))
            .filter(|p| p.exists())
            .ok_or_else(|| "游戏路径未设置或 SlayTheSpire2.exe 不存在".to_string())?;

        let mut cmd = std::process::Command::new(&exe_path);
        if vanilla {
            cmd.arg("--nomods");
        }
        cmd.spawn().map_err(|e| format!("启动游戏失败: {}", e))?;
        return Ok(());
    }

    // Steam 启动：统一走 steam.exe -applaunch（更可靠，不依赖 steam:// 协议）
    match find_steam_path() {
        Some(steam_path) => {
            let mut cmd = std::process::Command::new(steam_path.join("steam.exe"));
            cmd.args(["-applaunch", "2868840"]);
            if vanilla {
                cmd.arg("--nomods");
            }
            cmd.spawn()
                .map_err(|e| format!("启动游戏失败: {}", e))?;
        }
        None => {
            // 找不到 steam.exe 则回退到 steam:// 协议
            launch_via_steam_url()?;
        }
    }

    Ok(())
}

/// 通过 steam:// 协议启动（不传递命令行参数）
fn launch_via_steam_url() -> Result<(), String> {
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

/// 查找 Steam 安装路径下的 steam.exe
fn find_steam_path() -> Option<std::path::PathBuf> {
    let path = crate::integrations::steam::get_steam_install_path()?;
    let exe = path.join("steam.exe");
    if exe.exists() { Some(path) } else { None }
}

#[tauri::command]
pub fn open_path_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(p)
            .spawn()
            .map_err(|e| format!("打开资源管理器失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(p)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_url_in_browser(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败: {}", e))?;
    }
    Ok(())
}

// =========================================================================
// 辅助函数：更新激活预设的 mod 列表
// =========================================================================

/// 向激活的非内置预设添加 mod_id
fn add_mod_to_active_profile(mod_id: &str, active_name: &str) {
    if active_name.is_empty() {
        return;
    }
    let profiles = profile_service::list_profiles();
    if let Some(active_profile) = profiles.iter().find(|p| p.name == active_name)
        && !active_profile.builtin && !active_profile.mod_ids.contains(&mod_id.to_string()) {
            let mut new_ids = active_profile.mod_ids.clone();
            new_ids.push(mod_id.to_string());
            let _ = profile_service::update_profile(
                active_profile.id.clone(),
                active_profile.name.clone(),
                active_profile.description.clone(),
                new_ids,
            );
        }
}

/// 从激活的非内置预设移除 mod_id
fn remove_mod_from_active_profile(mod_id: &str, active_name: &str) {
    if active_name.is_empty() {
        return;
    }
    let profiles = profile_service::list_profiles();
    if let Some(active_profile) = profiles.iter().find(|p| p.name == active_name)
        && !active_profile.builtin && active_profile.mod_ids.contains(&mod_id.to_string()) {
            let new_ids: Vec<String> = active_profile
                .mod_ids
                .iter()
                .filter(|id| *id != mod_id)
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

fn require_game_root(state: &AppState) -> Result<String, String> {
    let settings = state.settings.read().unwrap();
    settings
        .game_root_dir
        .clone()
        .ok_or_else(|| "游戏目录未设置".to_string())
}

async fn run_blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| format!("后台任务失败: {}", e))
}

/// 内部辅助：提取 enable_mod / disable_mod 的公共逻辑
async fn toggle_mod_internal(
    state: &AppState,
    mod_id: String,
    toggle_fn: fn(&Path, &str, &[SaveSyncPair], bool) -> Result<ModToggleResult, AppError>,
    modify_profile: fn(&str, &str),
    action_label: &str,
) -> Result<ModToggleResult, String> {
    let (game_root, sync_pairs, backup_on_switch, active_name) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (
            game_root,
            settings.save_sync_pairs.clone(),
            settings.backup_on_path_switch,
            settings.active_profile_name.clone(),
        )
    };

    let result = run_blocking(move || {
        let res = toggle_fn(Path::new(&game_root), &mod_id, &sync_pairs, backup_on_switch)
            .map_err(|e| e.to_string())?;
        modify_profile(&mod_id, &active_name);
        Ok::<_, String>(res)
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("{action_label} Mod: {}", result.mod_item.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.3 Mod 操作
// =========================================================================

#[tauri::command]
pub async fn list_installed_mods(state: State<'_, AppState>) -> Result<Vec<InstalledMod>, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || mod_service::scan_enabled_mods(Path::new(&game_root))).await
}

#[tauri::command]
pub async fn list_disabled_mods(state: State<'_, AppState>) -> Result<Vec<InstalledMod>, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        let root = Path::new(&game_root);
        if !game_service::has_mods_dir(root) {
            return Vec::new();
        }
        mod_service::scan_disabled_mods(root)
    })
    .await
}

#[tauri::command]
pub async fn enable_mod(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<ModToggleResult, String> {
    toggle_mod_internal(
        &state,
        mod_id,
        mod_service::enable_mod,
        add_mod_to_active_profile,
        "启用",
    )
    .await
}

#[tauri::command]
pub async fn disable_mod(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<ModToggleResult, String> {
    toggle_mod_internal(
        &state,
        mod_id,
        mod_service::disable_mod,
        remove_mod_from_active_profile,
        "禁用",
    )
    .await
}

#[tauri::command]
pub async fn uninstall_mod(mod_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    let mod_id_clone = mod_id.clone();

    run_blocking(move || {
        mod_service::uninstall_mod(Path::new(&game_root), &mod_id_clone)
            .map_err(|e| e.to_string())?;
        // 清理文件哈希
        let _ = crate::repositories::mod_hashes_repo::remove_mod_hashes(&mod_id_clone);
        // 清除更新缓存
        updates_cache_repo::clear_updates_cache();
        Ok::<_, String>(())
    })
    .await??;

    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("卸载 Mod: {}", mod_id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn open_mod_folder(mod_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let game_root = require_game_root(&state)?;

    run_blocking(move || {
        // 先尝试本地目录
        let plugins_dir = Path::new(&game_root).join("mods");
        let disabled_dir = Path::new(&game_root).join("mods_disabled");

        let mod_folder = match mod_service::find_mod_folder(&plugins_dir, &mod_id)
            .or_else(|_| mod_service::find_mod_folder(&disabled_dir, &mod_id))
        {
            Ok(p) => p,
            Err(_) => {
                // 回退：在已启用的 mod 列表中按 id 查找（含创意工坊）
                let all = mod_service::scan_enabled_mods(Path::new(&game_root));
                let m = all
                    .iter()
                    .find(|m| m.id == mod_id)
                    .ok_or_else(|| format!("Mod 未找到: {}", mod_id))?;
                std::path::PathBuf::from(&m.install_dir)
            }
        };

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&mod_folder)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&mod_folder)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&mod_folder)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        Ok(())
    })
    .await?
}

#[tauri::command]
pub async fn uninstall_mod_instance(
    install_dir: String,
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    let install_dir_clone = install_dir.clone();
    let mod_id_clone = mod_id.clone();

    run_blocking(move || {
        mod_service::uninstall_mod_by_path(Path::new(&game_root), &install_dir_clone)
            .map_err(|e| e.to_string())?;
        // 清理文件哈希
        let _ = crate::repositories::mod_hashes_repo::remove_mod_hashes(&mod_id_clone);
        // 清除更新缓存
        updates_cache_repo::clear_updates_cache();
        Ok::<_, String>(())
    })
    .await??;

    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("卸载 Mod 实例: {}", mod_id),
        detail: Some(install_dir),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn open_mod_folder_by_path(install_dir: String) -> Result<(), String> {
    run_blocking(move || {
        let path = std::path::PathBuf::from(&install_dir);
        if !path.exists() {
            return Err(format!("目录不存在: {}", install_dir));
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        Ok(())
    })
    .await?
}

#[tauri::command]
pub async fn keep_mod_version_and_clean_others(
    install_dir: String,
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let (game_root, active_name) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (game_root, settings.active_profile_name.clone())
    };
    let mod_id_clone = mod_id.clone();

    let count = run_blocking(move || {
        let count = mod_service::keep_mod_version_and_clean_others(
            Path::new(&game_root),
            &install_dir,
            &mod_id_clone,
        )
        .map_err(|e| e.to_string())?;
        // 确保 active profile 包含此 mod_id
        add_mod_to_active_profile(&mod_id_clone, &active_name);
        updates_cache_repo::clear_updates_cache();
        Ok::<_, String>(count)
    })
    .await??;

    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("保留 Mod 版本并清理旧版: {}", mod_id),
        detail: Some(format!("清理了 {} 个旧版本", count)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(count)
}

#[tauri::command]
pub async fn switch_active_mod_version(
    install_dir: String,
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (game_root, active_name) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (game_root, settings.active_profile_name.clone())
    };
    let install_dir_clone = install_dir.clone();
    let mod_id_clone = mod_id.clone();

    run_blocking(move || {
        mod_service::switch_active_mod_version(
            Path::new(&game_root),
            &install_dir_clone,
            &mod_id_clone,
        )
        .map_err(|e| e.to_string())?;
        add_mod_to_active_profile(&mod_id_clone, &active_name);
        Ok::<_, String>(())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("切换激活 Mod 版本: {}", mod_id),
        detail: Some(install_dir),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn open_mods_directory(state: State<'_, AppState>) -> Result<(), String> {
    let game_root = require_game_root(&state)?;

    run_blocking(move || {
        let plugins = game_service::mods_dir(Path::new(&game_root));
        std::fs::create_dir_all(&plugins).map_err(|e| format!("创建目录失败: {}", e))?;

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&plugins)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&plugins)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&plugins)
                .spawn()
                .map_err(|e| format!("打开文件夹失败: {}", e))?;
        }

        Ok(())
    })
    .await?
}

#[tauri::command]
pub async fn get_mod_files(
    mod_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        mod_service::get_mod_files(Path::new(&game_root), &mod_id).map_err(|e| e.to_string())
    })
    .await?
}

// =========================================================================
// 5.4 ZIP 安装 + 批量导入
// =========================================================================

#[tauri::command]
pub async fn pick_archive_file() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("归档文件", &["zip", "7z", "rar"])
        .pick_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn pick_archive_files() -> Result<Vec<String>, String> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter("归档文件", &["zip", "7z", "rar"])
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
pub async fn preview_install_archive(
    archive_path: String,
    enable_after_install: bool,
    state: State<'_, AppState>,
) -> Result<BatchImportPreview, String> {
    let game_root = require_game_root(&state)?;
    let _ = enable_after_install;

    run_blocking(move || {
        install_archive_workflow::preview_archive(
            Path::new(&archive_path),
            Path::new(&game_root),
        )
        .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn install_archive(
    archive_path: String,
    enable_after_install: bool,
    replace_existing: bool,
    state: State<'_, AppState>,
) -> Result<Vec<InstalledMod>, String> {
    let game_root = require_game_root(&state)?;
    let archive_path_clone = archive_path.clone();

    let installed = run_blocking(move || {
        let installed = install_archive_workflow::execute_install(
            Path::new(&archive_path_clone),
            Path::new(&game_root),
            enable_after_install,
            replace_existing,
        )
        .map_err(|e| e.to_string())?;
        // 清除更新缓存
        updates_cache_repo::clear_updates_cache();
        Ok::<_, String>(installed)
    })
    .await??;

    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    // 记录日志
    let names: Vec<_> = installed.iter().map(|m| m.name.as_str()).collect();
    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("安装 Mod: {}", names.join(", ")),
        detail: Some(format!("来源: {}", archive_path)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(installed)
}

#[tauri::command]
pub async fn process_import_targets(
    paths: Vec<String>,
    enable_now: bool,
    state: State<'_, AppState>,
) -> Result<BatchImportPreview, String> {
    let game_root = require_game_root(&state)?;
    let _ = enable_now;

    run_blocking(move || {
        install_archive_workflow::batch_preview(&paths, Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn batch_install_mods(
    app_handle: tauri::AppHandle,
    paths: Vec<String>,
    enable_now: bool,
    has_conflicts: bool,
    selected_ids: Vec<String>,
    resolutions: Vec<(String, String)>, // (modId, "skip"|"replace")
    state: State<'_, AppState>,
) -> Result<BatchInstallResult, String> {
    let (game_root, backup_on_path_switch, sync_pairs) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (
            game_root,
            settings.backup_on_path_switch,
            settings.save_sync_pairs.clone(),
        )
    };

    let _ = has_conflicts;
    let paths_len = paths.len();

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

    let result = run_blocking(move || {
        let result = install_archive_workflow::batch_install(
            &app_handle,
            &paths,
            Path::new(&game_root),
            enable_now,
            &parsed_resolutions,
            &selected_ids,
        )
        .map_err(|e| e.to_string())?;

        // Save Guard：从 0 个 Mod → N 个 Mod 时自动触发存档备份 + 同步
        if enable_now && result.success_count > 0 && backup_on_path_switch {
            let installed_count = mod_service::scan_enabled_mods(Path::new(&game_root)).len();
            // 仅当安装前 mods/ 为空时才触发（first-time activation）
            if installed_count == result.success_count as usize {
                let _ = save_service::sync_saves(Path::new(&game_root), &sync_pairs);
            }
        }

        // 清除更新缓存
        updates_cache_repo::clear_updates_cache();
        Ok::<_, String>(result)
    })
    .await??;

    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    // 记录日志
    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!(
            "批量导入: {} 成功, {} 失败",
            result.success_count, result.failure_count
        ),
        detail: Some(format!("共 {} 个目标", paths_len)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.5 预设操作
// =========================================================================

#[tauri::command]
pub async fn list_profiles() -> Result<Vec<ModProfile>, String> {
    run_blocking(profile_service::list_profiles).await
}

#[tauri::command]
pub async fn create_profile(
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ModProfile, String> {
    let profile = run_blocking(move || {
        profile_service::create_profile(name, description, mod_ids).map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("创建预设: {}", profile.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(profile)
}

#[tauri::command]
pub async fn update_profile(
    id: String,
    name: String,
    description: Option<String>,
    mod_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ModProfile, String> {
    let profile = run_blocking(move || {
        profile_service::update_profile(id, name, description, mod_ids).map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("更新预设: {}", profile.name),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(profile)
}

#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let locale = state.settings.read().unwrap().locale.clone();
    let id_clone = id.clone();

    let profiles = run_blocking(move || {
        profile_service::delete_profile(&id_clone, &locale).map_err(|e| e.to_string())?;
        Ok::<_, String>(profile_service::list_profiles())
    })
    .await??;

    // 如果这是最后一个预设，delete_profile 会自动生成默认预设，自动应用它
    if profiles.len() == 1 {
        let default_p = &profiles[0];
        let mut w = state.settings.write().unwrap();
        if w.active_profile_name != default_p.name && w.active_profile_name == id {
            w.active_profile_name = default_p.name.clone();
            let _ = crate::repositories::settings_repo::save_settings(&w);
        }
    }

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "profile".to_string(),
        title: format!("删除预设: {}", id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn apply_profile(
    id: String,
    state: State<'_, AppState>,
) -> Result<ApplyProfileResult, String> {
    let (game_root, sync_pairs, backup_on_switch) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        let sync_pairs = settings.save_sync_pairs.clone();
        let backup_on_switch = settings.backup_on_path_switch;
        (game_root, sync_pairs, backup_on_switch)
    };

    let result = run_blocking(move || {
        profile_service::apply_profile(&id, Path::new(&game_root), &sync_pairs, backup_on_switch)
            .map_err(|e| e.to_string())
    })
    .await??;

    // 同步内存中的 active_profile_name，确保 get_app_bootstrap 返回最新值
    {
        let mut settings = state.settings.write().unwrap();
        settings.active_profile_name = result.profile.name.clone();
    }

    state.push_activity(ActivityLogEntry {
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
pub async fn export_preset_bundle(
    profile_id: String,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings.game_root_dir.clone().ok_or("游戏目录未设置")?
    };

    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        profile_service::export_bundle(&profile_id, &output_path, Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn preview_preset_bundle(
    bundle_path: String,
    state: State<'_, AppState>,
) -> Result<profile_service::BundlePreview, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        profile_service::preview_bundle(&bundle_path, Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn confirm_import_preset_bundle(
    bundle_path: String,
    apply_profile: bool,
    resolutions: Vec<(String, String)>,
    state: State<'_, AppState>,
) -> Result<ApplyProfileResult, String> {
    let (game_root, sync_pairs, backup_on_switch) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (
            game_root,
            settings.save_sync_pairs.clone(),
            settings.backup_on_path_switch,
        )
    };

    run_blocking(move || {
        profile_service::import_bundle(
            &bundle_path,
            Path::new(&game_root),
            apply_profile,
            &resolutions,
            &sync_pairs,
            backup_on_switch,
        )
        .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn pick_save_bundle_path(default_name: String) -> Result<Option<String>, String> {
    // 过滤掉 Windows 文件名非法字符，保留中文等 Unicode
    let safe_name: String = default_name
        .chars()
        .map(|c| if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect();
    let file_name = format!("{}.7z", safe_name.trim());
    let file = rfd::AsyncFileDialog::new()
        .add_filter("预设整合包", &["7z"])
        .set_file_name(&file_name)
        .save_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn pick_preset_bundle() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("预设整合包", &["7z"])
        .pick_file()
        .await;
    Ok(file.map(|f| f.path().to_string_lossy().to_string()))
}

// =========================================================================
// 5.7 存档管理
// =========================================================================

#[tauri::command]
pub async fn list_save_slots(state: State<'_, AppState>) -> Result<Vec<SaveSlot>, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || save_service::list_save_slots(Path::new(&game_root))).await
}

#[tauri::command]
pub async fn preview_save_transfer(
    source: SaveSlotRef,
    target: SaveSlotRef,
    state: State<'_, AppState>,
) -> Result<SaveTransferPreview, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        save_service::preview_save_transfer(Path::new(&game_root), &source, &target)
    })
    .await
}

#[tauri::command]
pub async fn transfer_save(
    source: SaveSlotRef,
    target: SaveSlotRef,
    state: State<'_, AppState>,
) -> Result<SaveBackupEntry, String> {
    let game_root = require_game_root(&state)?;
    let source_clone = source.clone();
    let target_clone = target.clone();

    let result = run_blocking(move || {
        save_service::transfer_save(Path::new(&game_root), &source_clone, &target_clone)
            .map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
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
pub async fn create_save_backup(
    steam_user_id: String,
    kind: SaveKind,
    slot_index: u32,
    reason: String,
    state: State<'_, AppState>,
) -> Result<SaveBackupEntry, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        save_service::create_save_backup(
            Path::new(&game_root),
            &steam_user_id,
            &kind,
            slot_index,
            &reason,
        )
        .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn list_save_backups(
    steam_user_id: Option<String>,
    kind: Option<SaveKind>,
    slot_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<SaveBackupEntry>, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        save_service::list_save_backups(
            Path::new(&game_root),
            steam_user_id.as_deref(),
            kind.as_ref(),
            slot_index,
        )
    })
    .await
}

#[tauri::command]
pub async fn restore_save_backup(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    let backup_id_clone = backup_id.clone();

    run_blocking(move || {
        save_service::restore_save_backup(Path::new(&game_root), &backup_id_clone)
            .map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!("恢复存档备份: {}", backup_id),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn delete_save_backup(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        save_service::delete_save_backup(Path::new(&game_root), &backup_id)
            .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn upgrade_backup_manual_flag(
    backup_id: String,
    manual: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        save_service::upgrade_backup_manual_flag(Path::new(&game_root), &backup_id, manual)
            .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn restore_save_backup_to_slot(
    backup_id: String,
    target_steam_user_id: String,
    target_kind: String, // "vanilla" | "modded"
    target_slot_index: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    let backup_id_clone = backup_id.clone();
    let target_steam_user_id_clone = target_steam_user_id.clone();
    let target_kind_clone = target_kind.clone();

    run_blocking(move || {
        let kind = match target_kind_clone.as_str() {
            "modded" => SaveKind::Modded,
            _ => SaveKind::Vanilla,
        };

        save_service::restore_save_backup_to_slot(
            Path::new(&game_root),
            &backup_id_clone,
            &target_steam_user_id_clone,
            &kind,
            target_slot_index,
        )
        .map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
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
pub async fn delete_save_slot(
    steam_user_id: String,
    kind: String,
    slot_index: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    let steam_user_id_clone = steam_user_id.clone();
    let kind_clone = kind.clone();

    run_blocking(move || {
        let save_kind = match kind_clone.as_str() {
            "modded" => SaveKind::Modded,
            _ => SaveKind::Vanilla,
        };

        save_service::delete_save_slot(
            Path::new(&game_root),
            &steam_user_id_clone,
            &save_kind,
            slot_index,
        )
        .map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
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
pub async fn sync_saves(state: State<'_, AppState>) -> Result<SaveSyncResult, String> {
    let (game_root, pairs) = {
        let settings = state.settings.read().unwrap();
        let game_root = settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?;
        (game_root, settings.save_sync_pairs.clone())
    };
    let pairs_len = pairs.len();

    let result = run_blocking(move || {
        save_service::sync_saves(Path::new(&game_root), &pairs).map_err(|e| e.to_string())
    })
    .await??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "save".to_string(),
        title: format!("存档同步: {} 对已同步", result.synced_count),
        detail: Some(format!("共 {} 对配对", pairs_len)),
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(result)
}

// =========================================================================
// 5.8 Steam 云存档
// =========================================================================

#[tauri::command]
pub async fn get_cloud_save_status(state: State<'_, AppState>) -> Result<CloudSaveStatus, String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or_else(|| "游戏目录未设置".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || {
        backup_service::get_cloud_save_status(Path::new(&game_root)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_cloud_save_diff_entries(
    state: State<'_, AppState>,
) -> Result<Vec<CloudSaveDiffEntry>, String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or_else(|| "游戏目录未设置".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || {
        backup_service::list_cloud_save_diff_entries(Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn copy_cloud_save_diff_side(
    relative_path: String,
    side: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or_else(|| "游戏目录未设置".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || {
        backup_service::copy_cloud_save_diff_side(Path::new(&game_root), &relative_path, &side)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ascend_to_cloud_full(state: State<'_, AppState>) -> Result<(), String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or_else(|| "游戏目录未设置".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || {
        backup_service::ascend_to_cloud_full(Path::new(&game_root))
    })
    .await
    .map_err(|e| e.to_string())??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "cloud".to_string(),
        title: "全量上传到 Steam 云".to_string(),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn descend_from_cloud_full(state: State<'_, AppState>) -> Result<(), String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or_else(|| "游戏目录未设置".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || {
        backup_service::descend_from_cloud_full(Path::new(&game_root))
    })
    .await
    .map_err(|e| e.to_string())??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "cloud".to_string(),
        title: "从 Steam 云全量下载".to_string(),
        detail: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    });

    Ok(())
}

#[tauri::command]
pub async fn get_backup_artifact_status(state: State<'_, AppState>) -> Result<bool, String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        backup_service::get_backup_artifact_status(Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await?
}

#[tauri::command]
pub async fn cleanup_backup_artifacts(state: State<'_, AppState>) -> Result<(), String> {
    let game_root = require_game_root(&state)?;
    run_blocking(move || {
        backup_service::cleanup_backup_artifacts(Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await?
}

// =========================================================================
// 5.9 Nexus Mods 集成
// =========================================================================

#[tauri::command]
pub async fn search_remote_mods(
    query: String,
    page: Option<u32>,
    page_size: Option<u32>,
    #[allow(non_snake_case)]
    pageSize: Option<u32>,
    sort_by: Option<String>,
) -> Result<RemoteModSearchResult, String> {
    let size = page_size.or(pageSize).unwrap_or(12);
    let page_num = page.unwrap_or(1);
    let sort = sort_by.unwrap_or_else(|| "latest_added".to_string());
    tauri::async_runtime::spawn_blocking(move || {
        discover_service::search_remote_mods(&query, page_num, size, &sort)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

// =========================================================================
// 5.10 Steam 创意工坊集成
// =========================================================================

#[tauri::command]
pub async fn check_steam_status() -> bool {
    tauri::async_runtime::spawn_blocking(crate::integrations::workshop::is_steam_running)
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub async fn search_workshop(
    query: String,
    page: u32,
    page_size: Option<u32>,
    #[allow(non_snake_case)]
    pageSize: Option<u32>,
    sort_by: String,
) -> Result<crate::domain::workshop_mod::WorkshopSearchResult, String> {
    let size = page_size.or(pageSize).unwrap_or(12);
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::workshop::search_workshop(&query, page, size, &sort_by)
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn subscribe_workshop_mod(published_file_id: u64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::workshop::subscribe_mod(published_file_id)
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn unsubscribe_workshop_mod(published_file_id: u64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::workshop::unsubscribe_mod(published_file_id)
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn check_mod_updates(state: State<'_, AppState>) -> Result<Vec<ModUpdateInfo>, String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings
            .game_root_dir
            .clone()
            .ok_or("游戏目录未设置")?
    };
    // 清理旧缓存，确保全新数据
    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    let results = tauri::async_runtime::spawn_blocking(move || {
        updates_cache_repo::clear_updates_cache();
        update_check::check_mod_updates(std::path::Path::new(&game_root))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))??;

    // 写入磁盘缓存
    let cache = ModUpdateCheckCache {
        results: results.clone(),
        checked_at: chrono::Local::now().to_rfc3339(),
        total_mods: results.len(),
        updated_mods: results.iter().filter(|r| r.has_update).count(),
    };
    let cache_for_disk = cache.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        let _ = updates_cache_repo::save_updates_cache(&cache_for_disk);
    })
    .await;
    // 更新内存缓存
    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = Some(cache);
    }
    Ok(results)
}

#[tauri::command]
pub fn get_cached_mod_updates(state: State<AppState>) -> Vec<ModUpdateInfo> {
    if let Ok(cache) = state.mod_updates_cache.read()
        && let Some(ref c) = *cache {
            return c.results.clone();
        }
    Vec::new()
}

// =========================================================================
// 5.14 后台更新检查（事件驱动）
// =========================================================================

/// 更新检查结果事件载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateCheckEvent {
    pub req_id: u64,
    pub success: bool,
    pub error: Option<String>,
    pub results: Vec<ModUpdateInfo>,
    pub summary: ModUpdateSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateSummary {
    pub total_mods: usize,
    pub updated_mods: usize,
}

/// 后台更新检查（立即返回，结果通过 `slaymgr:update-check-result` 事件推送）
#[tauri::command]
pub fn start_mod_update_check(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    req_id: u64,
) -> Result<(), String> {
    let game_root = {
        let settings = state.settings.read().unwrap();
        settings.game_root_dir.clone().ok_or("游戏目录未设置")?
    };

    // 清理内存缓存
    if let Ok(mut cache_lock) = state.mod_updates_cache.write() {
        *cache_lock = None;
    }

    std::thread::spawn(move || {
        // 清理磁盘缓存，确保全新数据
        updates_cache_repo::clear_updates_cache();

        let result = update_check::check_mod_updates(std::path::Path::new(&game_root));

        let event = match result {
            Ok(results) => {
                let updated = results.iter().filter(|r| r.has_update).count();
                let total = results.len();

                // 写入磁盘缓存
                let cache = ModUpdateCheckCache {
                    results: results.clone(),
                    checked_at: chrono::Local::now().to_rfc3339(),
                    total_mods: total,
                    updated_mods: updated,
                };
                let _ = updates_cache_repo::save_updates_cache(&cache);

                // 更新内存缓存
                if let Ok(mut cache_lock) = app_handle.state::<AppState>().mod_updates_cache.write() {
                    *cache_lock = Some(cache);
                }

                ModUpdateCheckEvent {
                    req_id,
                    success: true,
                    error: None,
                    results,
                    summary: ModUpdateSummary { total_mods: total, updated_mods: updated },
                }
            }
            Err(e) => ModUpdateCheckEvent {
                req_id,
                success: false,
                error: Some(e.to_string()),
                results: Vec::new(),
                summary: ModUpdateSummary { total_mods: 0, updated_mods: 0 },
            },
        };

        let _ = app_handle.emit("slaymgr:update-check-result", &event);
    });

    Ok(())
}

// =========================================================================
// 5.10 设置
// =========================================================================

#[tauri::command]
pub async fn update_nexus_api_key(
    api_key: String,
    state: State<'_, AppState>,
) -> Result<crate::integrations::nexus_client::NexusUserInfo, String> {
    let clean_key = api_key.trim().to_string();
    if clean_key.is_empty() {
        let mut settings = state.settings.write().unwrap();
        settings.nexus_api_key = None;
        settings.nexus_is_premium = false;
        settings.nexus_user_name = None;
        let _ = settings_repo::save_settings(&settings);
        return Err("API Key 不能为空".to_string());
    }

    let proxy = {
        let s = state.settings.read().unwrap();
        s.proxy_url.clone()
    };

    let key_for_req = clean_key.clone();
    let user_info = tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::nexus_client::validate_user(&key_for_req, proxy.as_deref())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))??;

    {
        let mut settings = state.settings.write().unwrap();
        settings.nexus_api_key = Some(clean_key);
        settings.nexus_is_premium = user_info.is_premium;
        settings.nexus_user_name = Some(user_info.name.clone());
        let _ = settings_repo::save_settings(&settings);
    }

    Ok(user_info)
}

#[tauri::command]
pub async fn validate_nexus_api_key(
    state: State<'_, AppState>,
) -> Result<crate::integrations::nexus_client::NexusUserInfo, String> {
    let (key, proxy) = {
        let s = state.settings.read().unwrap();
        (s.nexus_api_key.clone(), s.proxy_url.clone())
    };

    let key = key.ok_or_else(|| "未配置 Nexus API Key".to_string())?;
    let user_info = tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::nexus_client::validate_user(&key, proxy.as_deref())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))??;

    {
        let mut settings = state.settings.write().unwrap();
        settings.nexus_is_premium = user_info.is_premium;
        settings.nexus_user_name = Some(user_info.name.clone());
        let _ = settings_repo::save_settings(&settings);
    }

    Ok(user_info)
}

#[tauri::command]
pub async fn get_nexus_mod_files(
    mod_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<crate::integrations::nexus_client::NexusModFile>, String> {
    let (key, proxy) = {
        let s = state.settings.read().unwrap();
        (s.nexus_api_key.clone(), s.proxy_url.clone())
    };

    let key = key.ok_or_else(|| "请先在「设置」中配置 Nexus API Key".to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::nexus_client::get_mod_files(
            &key,
            proxy.as_deref(),
            "slaythespire2",
            mod_id,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn get_nexus_download_link(
    mod_id: u32,
    file_id: u64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (key, proxy) = {
        let s = state.settings.read().unwrap();
        (s.nexus_api_key.clone(), s.proxy_url.clone())
    };

    let key = key.ok_or_else(|| "请先在「设置」中配置 Nexus API Key".to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::nexus_client::get_download_link(
            &key,
            proxy.as_deref(),
            "slaythespire2",
            mod_id,
            file_id,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub async fn start_nexus_download(
    app_handle: tauri::AppHandle,
    mod_id: u32,
    file_id: u64,
    mod_name: String,
    file_name: String,
    state: State<'_, AppState>,
) -> Result<download_service::DownloadTask, String> {
    let (key, proxy) = {
        let s = state.settings.read().unwrap();
        (s.nexus_api_key.clone(), s.proxy_url.clone())
    };

    let key = key.ok_or_else(|| "请先在「设置」中配置 Nexus API Key".to_string())?;
    let mod_name_clone = mod_name.clone();
    let file_name_clone = file_name.clone();
    let task = tauri::async_runtime::spawn_blocking(move || -> Result<download_service::DownloadTask, String> {
        let download_url = crate::integrations::nexus_client::get_download_link(
            &key,
            proxy.as_deref(),
            "slaythespire2",
            mod_id,
            file_id,
        )
        .map_err(|e| e.to_string())?;

        download_service::start_download(
            app_handle,
            mod_id,
            file_id,
            mod_name_clone,
            file_name_clone,
            download_url,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))??;

    state.push_activity(ActivityLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        category: "mod".to_string(),
        title: format!("下载 Nexus 模组: {} ({})", mod_name, file_name),
        detail: Some(format!("mod_id: {}, file_id: {}", mod_id, file_id)),
        created_at: chrono::Local::now().to_rfc3339(),
    });

    Ok(task)
}

#[tauri::command]
pub fn pause_download(task_id: String) -> Result<(), String> {
    download_service::pause_download(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resume_download(app_handle: tauri::AppHandle, task_id: String) -> Result<(), String> {
    download_service::resume_download(app_handle, &task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cancel_download(task_id: String) -> Result<(), String> {
    download_service::cancel_download(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_download_tasks() -> Result<Vec<download_service::DownloadTask>, String> {
    Ok(download_service::get_download_tasks())
}

#[tauri::command]
pub fn clear_completed_downloads() -> Result<(), String> {
    download_service::clear_completed_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_downloads_folder() -> Result<(), String> {
    let dir = download_service::get_download_dir();
    crate::services::game_service::open_folder(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_proxy_url(url: Option<String>, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.proxy_url = url;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub async fn test_proxy(url: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::integrations::nexus_client::test_proxy(&url).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

#[tauri::command]
pub fn update_auto_backup_keep_count(count: usize, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.auto_backup_keep_count = count;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub fn update_backup_on_path_switch(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.backup_on_path_switch = enabled;
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

#[tauri::command]
pub fn update_vanilla_launch(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.vanilla_launch = enabled;
    let _ = settings_repo::save_settings(&settings);
    Ok(())
}

// =========================================================================
// 5.10.x 应用更新设置
// =========================================================================

#[tauri::command]
pub fn update_auto_check_update(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.auto_check_update = enabled;
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

// =========================================================================
// 5.12 翻译
// =========================================================================

/// 通过 MyMemory API 翻译文本（走 Rust 后端避免 WebView CORS/403 问题）
#[tauri::command]
pub async fn translate_text(text: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("HTTP 客户端构建失败: {}", e))?;

        let url = reqwest::Url::parse_with_params(
            "https://api.mymemory.translated.net/get",
            &[("q", &text as &str), ("langpair", "en|zh-CN")],
        )
        .map_err(|e| format!("URL 构建失败: {}", e))?;

        let resp = client
            .get(url)
            .send()
            .map_err(|e| format!("翻译请求失败: {}", e))?;

        if !resp.status().is_success() {
            let hints = match resp.status().as_u16() {
                403 => "免费 API 频率超限，稍后再试".to_string(),
                429 => "请求过于频繁，请稍候".to_string(),
                503 => "翻译服务暂时不可用".to_string(),
                code => format!("HTTP {}", code),
            };
            return Err(hints);
        }

        let data: serde_json::Value = resp
            .json()
            .map_err(|e| format!("响应解析失败: {}", e))?;

        let status = data["responseStatus"].as_i64().unwrap_or(0);
        if status != 200 {
            let details = data["responseDetails"].as_str().unwrap_or("");
            return Err(format!("翻译服务返回错误: {}", details));
        }

        let translated = data["responseData"]["translatedText"]
            .as_str()
            .ok_or_else(|| "翻译结果为空".to_string())?;

        Ok(translated.to_string())
    })
    .await
    .map_err(|e| format!("后台任务失败: {}", e))?
}

// --- 自定义背景壁纸管理 ---

fn bg_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("slaymumanager")
}

const BASE64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        result.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
        result.push(BASE64_ALPHABET[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_ALPHABET[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_ALPHABET[(b2 & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn file_to_data_url(path: &std::path::Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "avif" => "image/avif",
        _ => "image/png",
    };
    let bytes = std::fs::read(path).map_err(|e| format!("读取背景图片失败: {}", e))?;
    let encoded = base64_encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[tauri::command]
pub async fn pick_custom_background() -> Result<Option<String>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("图片文件", &["png", "jpg", "jpeg", "webp", "bmp", "gif", "avif"])
        .pick_file()
        .await;

    let handle = match file {
        Some(h) => h,
        None => return Ok(None),
    };

    let src_path = handle.path().to_path_buf();
    run_blocking(move || {
        let ext = src_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_lowercase();

        let target_dir = bg_dir();
        let _ = std::fs::create_dir_all(&target_dir);

        // 清理旧背景文件
        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.starts_with("custom_bg.")
                {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        let target_path = target_dir.join(format!("custom_bg.{}", ext));
        std::fs::copy(&src_path, &target_path).map_err(|e| format!("保存背景图片失败: {}", e))?;

        let data_url = file_to_data_url(&target_path)?;
        Ok(Some(data_url))
    })
    .await?
}

#[tauri::command]
pub async fn get_custom_background() -> Result<Option<String>, String> {
    run_blocking(|| {
        let target_dir = bg_dir();
        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.starts_with("custom_bg.")
                {
                    return file_to_data_url(&entry.path()).map(Some);
                }
            }
        }
        Ok(None)
    })
    .await?
}

#[tauri::command]
pub async fn clear_custom_background() -> Result<(), String> {
    run_blocking(|| {
        let target_dir = bg_dir();
        if let Ok(entries) = std::fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.starts_with("custom_bg.")
                {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
        Ok(())
    })
    .await?
}
