use crate::domain::mod_entity::InstalledMod;
use crate::domain::remote_mod::{ModFileInfo, RemoteModSearchResult};
use crate::integrations::nexus_client;
use crate::repositories::settings_repo;
use crate::services::mod_service;
use crate::utils::error::AppError;
use std::path::Path;

const DEFAULT_GAME_SLUG: &str = "slaythespire2";

/// 搜索 Nexus Mods
pub fn search_remote_mods(
    query: &str,
    page: u32,
    sort_by: &str,
) -> Result<RemoteModSearchResult, AppError> {
    let settings = settings_repo::load_settings().unwrap_or_default();
    let api_key = settings
        .nexus_api_key
        .ok_or_else(|| AppError::Other("请先在设置中配置 Nexus Mods API Key".to_string()))?;

    nexus_client::search_mods(
        DEFAULT_GAME_SLUG,
        query,
        page.max(1),
        sort_by,
        &api_key,
        settings.proxy_url.as_deref(),
    )
}

/// 获取 Mod 文件列表
pub fn get_mod_files(mod_id: u32) -> Result<Vec<ModFileInfo>, AppError> {
    let settings = settings_repo::load_settings().unwrap_or_default();
    let api_key = settings
        .nexus_api_key
        .ok_or_else(|| AppError::Other("请先在设置中配置 Nexus Mods API Key".to_string()))?;

    nexus_client::get_mod_files(
        DEFAULT_GAME_SLUG,
        mod_id,
        &api_key,
        settings.proxy_url.as_deref(),
    )
}

/// 获取下载链接
pub fn get_download_link(mod_id: u32, file_id: u32) -> Result<String, AppError> {
    let settings = settings_repo::load_settings().unwrap_or_default();
    let api_key = settings
        .nexus_api_key
        .ok_or_else(|| AppError::Other("请先在设置中配置 Nexus Mods API Key".to_string()))?;

    nexus_client::get_download_link(
        DEFAULT_GAME_SLUG,
        mod_id,
        file_id,
        &api_key,
        settings.proxy_url.as_deref(),
    )
}

/// 下载并安装 Mod（从 URL 下载 → 保存到临时目录 → 通过归档安装流程安装）
pub fn download_and_install_mod(
    mod_id: u32,
    file_id: u32,
    enable_after: bool,
    game_root: &Path,
) -> Result<Vec<InstalledMod>, AppError> {
    // 获取下载链接
    let download_url = get_download_link(mod_id, file_id)?;

    // 如果是 Nexus 页面链接（免费用户），直接返回
    if download_url.contains("nexusmods.com") && !download_url.ends_with(".zip") {
        return Err(AppError::Other(format!(
            "免费用户请在浏览器中下载: {}",
            download_url
        )));
    }

    // 下载文件（带代理支持 + 重定向跟随）
    let settings = settings_repo::load_settings().unwrap_or_default();
    let mut builder = reqwest::blocking::Client::builder()
        .user_agent("SlayMuManager/0.1.0")
        .timeout(std::time::Duration::from_secs(300))
        .redirect(reqwest::redirect::Policy::limited(5));

    if let Some(ref proxy) = settings.proxy_url {
        if !proxy.is_empty() {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy)
                    .map_err(|e| AppError::Other(format!("代理配置错误: {}", e)))?,
            );
        }
    }

    let client = builder
        .build()
        .map_err(|e| AppError::Other(format!("HTTP 客户端构建失败: {}", e)))?;

    let resp = client
        .get(&download_url)
        .send()
        .map_err(|e| AppError::Other(format!("下载失败: {}", e)))?;

    let bytes = resp
        .bytes()
        .map_err(|e| AppError::Other(format!("下载读取失败: {}", e)))?;

    // 保存到临时文件
    let temp_dir = std::env::temp_dir().join("slaymumanager").join("downloads");
    std::fs::create_dir_all(&temp_dir).map_err(AppError::Io)?;
    let temp_file = temp_dir.join(format!("nexus_{}_{}.zip", mod_id, file_id));
    std::fs::write(&temp_file, &bytes).map_err(AppError::Io)?;

    // 通过归档安装流程安装
    crate::workflows::install_archive_workflow::execute_install(
        &temp_file,
        game_root,
        enable_after,
        false, // 不自动替换冲突
    )
}
