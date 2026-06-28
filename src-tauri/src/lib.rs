mod app;
mod domain;
mod integrations;
mod repositories;
mod services;
mod utils;
mod workflows;

use app::state::AppState;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;
use tauri::Emitter as _;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 全局 panic hook：崩溃时打印诊断信息
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[SlayMuManager] Panic: {info}");
        if let Some(loc) = info.location() {
            eprintln!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
        }
    }));

    tauri::Builder::default()
        .setup(|app| {
            let state = AppState::default();
            let app_handle = app.handle().clone();
            app.manage(state);

            // 启动文件系统监听（检测外部 Mod 变更）
            start_mods_watcher(app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 5.1 启动与初始化
            app::commands::get_app_bootstrap,
            app::commands::detect_game_install,
            app::commands::update_game_root_dir,
            app::commands::update_app_locale,
            // 5.2 游戏操作
            app::commands::is_game_running,
            app::commands::launch_game,
            app::commands::open_path_in_explorer,
            app::commands::open_url_in_browser,
            // 5.3 Mod 操作
            app::commands::list_installed_mods,
            app::commands::list_disabled_mods,
            app::commands::enable_mod,
            app::commands::disable_mod,
            app::commands::uninstall_mod,
            app::commands::open_mod_folder,
            app::commands::open_mods_directory,
            app::commands::get_mod_files,
            // 5.4 ZIP 安装 + 批量导入
            app::commands::pick_archive_file,
            app::commands::pick_archive_files,
            app::commands::pick_import_folder,
            app::commands::pick_game_folder,
            app::commands::preview_install_archive,
            app::commands::install_archive,
            app::commands::process_import_targets,
            app::commands::batch_install_mods,
            // 5.5 预设操作
            app::commands::list_profiles,
            app::commands::create_profile,
            app::commands::update_profile,
            app::commands::delete_profile,
            app::commands::apply_profile,
            // 5.6 整合包
            app::commands::export_preset_bundle,
            app::commands::preview_preset_bundle,
            app::commands::confirm_import_preset_bundle,
            app::commands::pick_save_bundle_path,
            app::commands::pick_preset_bundle,
            // 5.7 存档管理
            app::commands::list_save_slots,
            app::commands::preview_save_transfer,
            app::commands::transfer_save,
            app::commands::create_save_backup,
            app::commands::list_save_backups,
            app::commands::restore_save_backup,
            app::commands::restore_save_backup_to_slot,
            app::commands::delete_save_backup,
            app::commands::delete_save_slot,
            app::commands::toggle_save_auto_sync,
            app::commands::update_save_sync_pairs,
            app::commands::sync_saves,
            // 5.8 Steam 云存档
            app::commands::get_cloud_save_status,
            app::commands::list_cloud_save_diff_entries,
            app::commands::copy_cloud_save_diff_side,
            app::commands::ascend_to_cloud_full,
            app::commands::descend_from_cloud_full,
            app::commands::get_backup_artifact_status,
            app::commands::cleanup_backup_artifacts,
            // 5.9 Nexus Mods 集成
            app::commands::search_remote_mods,
            app::commands::start_remote_search,
            app::commands::start_mod_update_check,
            app::commands::check_mod_updates,
            app::commands::get_cached_mod_updates,
            // 5.10 Steam 创意工坊
            app::commands::check_steam_status,
            app::commands::search_workshop,
            app::commands::subscribe_workshop_mod,
            app::commands::unsubscribe_workshop_mod,
            // 5.11 设置
            app::commands::update_nexus_api_key,
            app::commands::update_proxy_url,
            app::commands::test_proxy,
            app::commands::update_auto_backup_keep_count,
            app::commands::update_backup_on_path_switch,
            app::commands::update_theme_mode,
            app::commands::update_theme_color,
            app::commands::update_launch_mode,
            app::commands::update_launch_check_cloud_save,
            app::commands::update_vanilla_launch,
            // 5.10.x 应用更新
            app::commands::update_auto_check_update,
            // 5.11 日志
            app::commands::list_activity_logs,
            // 5.12 翻译
            app::commands::translate_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 启动文件系统监听：检测 mods/ 和 mods_disabled/ 的外部变更
fn start_mods_watcher(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        // 轮询检测游戏目录是否已设置
        let game_root: Option<std::path::PathBuf> = loop {
            std::thread::sleep(Duration::from_secs(5));
            let state = app_handle.state::<AppState>();
            let settings = state.settings.read().unwrap();
            if let Some(dir) = &settings.game_root_dir {
                break Some(std::path::PathBuf::from(dir));
            }
        };

        let game_root = match game_root {
            Some(p) => p,
            None => return,
        };

        let mods_dir = game_root.join("mods");
        let disabled_dir = game_root.join("mods_disabled");

        // 确保目录存在
        let _ = std::fs::create_dir_all(&mods_dir);
        let _ = std::fs::create_dir_all(&disabled_dir);

        let (tx, rx) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                let _ = tx.send(res);
            },
        ) {
            Ok(w) => w,
            Err(_) => return,
        };

        let _ = watcher.watch(&mods_dir, RecursiveMode::NonRecursive);
        let _ = watcher.watch(&disabled_dir, RecursiveMode::NonRecursive);

        let app_handle = app_handle.clone();
        let mut last_emit = std::time::Instant::now();

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    // 只响应实质变更（忽略 access 和无关事件）
                    let is_relevant = matches!(
                        event.kind,
                        EventKind::Create(_)
                            | EventKind::Modify(_)
                            | EventKind::Remove(_)
                    );
                    if !is_relevant {
                        continue;
                    }

                    // 防抖：500ms 内只发一次
                    let now = std::time::Instant::now();
                    if now.duration_since(last_emit) < Duration::from_millis(500) {
                        continue;
                    }
                    last_emit = now;

                    let _ = app_handle.emit("slaymgr:mods-changed", ());
                }
                _ => break,
            }
        }
    });
}
