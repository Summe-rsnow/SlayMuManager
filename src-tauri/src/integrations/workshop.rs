use crate::domain::workshop_mod::{WorkshopMod, WorkshopSearchResult};
use std::sync::{Mutex, Arc};
use steamworks::{Client, PublishedFileId, AppId, UGCQueryType, UGCType, AppIDs};

static STEAM_CLIENT: Mutex<Option<Client>> = Mutex::new(None);

pub fn is_steam_running() -> bool {
    crate::integrations::steam::get_active_steam_account_id().is_some()
}

pub fn init_client() -> Result<(), String> {
    let mut guard = STEAM_CLIENT.lock().unwrap();
    if guard.is_none() {
        match Client::init_app(AppId(2868840)) {
            Ok(c) => {
                *guard = Some(c);
            }
            Err(e) => {
                return Err(format!("Steam 初始化失败: {:?}", e));
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
        if let Ok(mut guard) = result.try_lock() {
            if let Some(res) = guard.take() {
                return res;
            }
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
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let subscribed = ugc.subscribed_items(true);

        let shared: Arc<Mutex<Option<Result<WorkshopSearchResult, String>>>> = Arc::new(Mutex::new(None));
        let shared_clone = shared.clone();
        let limit = page_size.max(1).min(100);

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

pub fn subscribe_mod(published_file_id: u64) -> Result<(), String> {
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let shared_c = shared.clone();

        ugc.subscribe_item(PublishedFileId(published_file_id), move |result| {
            let r = result.map_err(|e| format!("{:?}", e));
            if let Ok(mut g) = shared_c.lock() {
                *g = Some(r);
            }
        });

        poll_callback(client, &shared, 15)
    })
}

pub fn unsubscribe_mod(published_file_id: u64) -> Result<(), String> {
    init_client()?;
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        let shared_c = shared.clone();

        ugc.unsubscribe_item(PublishedFileId(published_file_id), move |result| {
            let r = result.map_err(|e| format!("{:?}", e));
            if let Ok(mut g) = shared_c.lock() {
                *g = Some(r);
            }
        });

        poll_callback(client, &shared, 15)
    })
}
