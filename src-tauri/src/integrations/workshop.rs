use crate::domain::workshop_mod::{WorkshopMod, WorkshopSearchResult};
use std::sync::{Mutex, Arc};
use steamworks::{Client, PublishedFileId, AppId, UGCQueryType, UGCType, AppIDs};

static STEAM_CLIENT: Mutex<Option<Client>> = Mutex::new(None);

pub fn is_steam_running() -> bool {
    crate::integrations::steam::get_active_steam_account_id().is_some()
}

/// 确保 Steam 可连接，返回用户友好的错误提示
fn ensure_steam_ready() -> Result<(), String> {
    if !is_steam_running() {
        return Err("请先启动 Steam 并登录账号".to_string());
    }
    Ok(())
}

pub fn init_client() -> Result<(), String> {
    let mut guard = STEAM_CLIENT.lock().unwrap();
    if guard.is_none() {
        match Client::init_app(AppId(2868840)) {
            Ok(c) => {
                *guard = Some(c);
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                // "ConnectToGlobalUser failed" 意味着 Steam 运行中但用户会话未就绪
                if msg.contains("ConnectToGlobalUser") || msg.contains("NoSteamClient") {
                    return Err("请确保 Steam 已完全启动并登录账号，然后重试".to_string());
                }
                return Err(format!("Steam 初始化失败: {}", msg));
            }
        }
    }
    Ok(())
}

fn with_client<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Client) -> Result<R, String>,
{
    let guard = STEAM_CLIENT.lock().unwrap();
    let client = guard.as_ref().ok_or_else(|| "Steam 未初始化".to_string())?;
    f(client)
}

/// 持续调用 run_callbacks 直到 callback 设置结果或超时
fn poll_callback<T>(client: &Client, result: &Arc<Mutex<Option<Result<T, String>>>>, timeout_secs: u64) -> Result<T, String> {
    let start = std::time::Instant::now();
    let max_wait = std::time::Duration::from_secs(timeout_secs);
    loop {
        client.run_callbacks();
        if let Ok(mut guard) = result.try_lock()
            && let Some(res) = guard.take() {
                return res;
            }
        if start.elapsed() > max_wait {
            return Err("操作超时".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn ugc_query_type_from_sort(sort_by: &str) -> UGCQueryType {
    match sort_by {
        "trending" => UGCQueryType::RankedByTrend,
        "downloads" => UGCQueryType::RankedByTotalUniqueSubscriptions,
        _ => UGCQueryType::RankedByPublicationDate,
    }
}

pub fn search_workshop(query: &str, page: u32, page_size: u32, sort_by: &str) -> Result<WorkshopSearchResult, String> {
    ensure_steam_ready()?;
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let subscribed = ugc.subscribed_items(true);

        let shared: Arc<Mutex<Option<Result<WorkshopSearchResult, String>>>> = Arc::new(Mutex::new(None));
        let shared_clone = shared.clone();
        let limit = page_size.clamp(1, 100);

        let qh = ugc
            .query_all(
                ugc_query_type_from_sort(sort_by),
                UGCType::Items,
                AppIDs::ConsumerAppId(AppId(2868840)),
                page.max(1),
            )
            .map_err(|e| format!("创建查询失败: {:?}", e))?
            .set_language("schinese")
            .set_search_text(query)
            .include_long_desc(true);

        qh.fetch(move |result| {
            let mapped = result.map(|qr| {
                let total = qr.total_results();
                let returned = qr.returned_results();
                let take = (returned as usize).min(limit as usize);
                let mut mods = Vec::with_capacity(take);
                for i in 0..take {
                    if let Some(item) = qr.get(i as u32) {
                        let preview = qr.preview_url(i as u32);
                        mods.push(WorkshopMod {
                            id: item.published_file_id.0,
                            name: item.title,
                            author: item.owner.raw().to_string(),
                            description: item.description,
                            preview_url: preview,
                            tags: item.tags,
                            subscribers: 0,
                            votes_up: item.num_upvotes,
                            votes_down: item.num_downvotes,
                            subscribed: subscribed.contains(&item.published_file_id),
                        });
                    }
                }
                WorkshopSearchResult {
                    items: mods,
                    total_count: total,
                }
            })
            .map_err(|e| format!("{:?}", e));
            if let Ok(mut guard) = shared_clone.lock() {
                *guard = Some(mapped);
            }
        });

        poll_callback(client, &shared, 15)
    })
}

fn format_workshop_error(e: &str) -> String {
    if e.contains("AccessDenied") {
        "订阅失败 (权限不足)：Steam 限制客户端直接调用接口订阅该模组（通常因账号未拥有该游戏测试资格或未在 Steam 同意工坊协议）。请点击「前往网页订阅」在 Steam 社区页面一键订阅。".to_string()
    } else if e.contains("NoSteamClient") || e.contains("ConnectToGlobalUser") {
        "Steam 客户端未启动或未登录，请先启动 Steam 客户端并登录账号后重试。".to_string()
    } else if e.contains("NotLoggedOn") {
        "Steam 当前处于离线模式或网络未连接，请确保 Steam 在线后重试。".to_string()
    } else if e.contains("Timeout") || e.contains("操作超时") {
        "Steam 响应超时，请检查网络连接，或点击「前往网页订阅」在浏览器中直接订阅。".to_string()
    } else {
        format!("创意工坊操作失败: {}。您可点击「前往网页订阅」在 Steam 社区页面手动订阅。", e)
    }
}

pub fn subscribe_mod(published_file_id: u64) -> Result<(), String> {
    ensure_steam_ready()?;
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let shared_c = shared.clone();

        ugc.subscribe_item(PublishedFileId(published_file_id), move |result| {
            let r = result.map_err(|e| format_workshop_error(&format!("{:?}", e)));
            if let Ok(mut g) = shared_c.lock() {
                *g = Some(r);
            }
        });

        poll_callback(client, &shared, 15).map_err(|e| format_workshop_error(&e))
    })
}

pub fn unsubscribe_mod(published_file_id: u64) -> Result<(), String> {
    ensure_steam_ready()?;
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let shared_c = shared.clone();

        ugc.unsubscribe_item(PublishedFileId(published_file_id), move |result| {
            let r = result.map_err(|e| format_workshop_error(&format!("{:?}", e)));
            if let Ok(mut g) = shared_c.lock() {
                *g = Some(r);
            }
        });

        poll_callback(client, &shared, 15).map_err(|e| format_workshop_error(&e))
    })
}
