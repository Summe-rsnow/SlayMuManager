use crate::domain::remote_mod::RemoteModSearchResult;
use crate::integrations::nexus_client;
use crate::repositories::settings_repo;
use crate::utils::error::AppError;

const DEFAULT_GAME_SLUG: &str = "slaythespire2";

/// 搜索 Nexus Mods
pub fn search_remote_mods(
    query: &str,
    page: u32,
    page_size: u32,
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
        page_size.clamp(1, 100),
        sort_by,
        &api_key,
        settings.proxy_url.as_deref(),
    )
}
