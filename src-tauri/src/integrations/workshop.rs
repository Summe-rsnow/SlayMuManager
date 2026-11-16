use crate::domain::workshop_mod::{WorkshopMod, WorkshopSearchResult};
use parking_lot::Mutex;
use std::sync::Arc;
use steamworks::{AppIDs, AppId, Client, PublishedFileId, UGCQueryType, UGCType};

static STEAM_LOCK: Mutex<()> = Mutex::new(());
static STEAM_WORKER_LOCK: Mutex<()> = Mutex::new(());

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

/// 执行 Steamworks 操作：按需临时初始化，并在操作结束后立即 Drop 客户端，
/// 从而触发 SteamAPI_Shutdown()，彻底杜绝 SlayMuManager 被 Steam 误识别为长期在玩游戏的问题。
fn with_client<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Client) -> Result<R, String>,
{
    ensure_steam_ready()?;
    let _lock = STEAM_LOCK.lock();

    let client = Client::init_app(AppId(2868840)).map_err(|e| {
        let msg = format!("{:?}", e);
        if msg.contains("ConnectToGlobalUser") || msg.contains("NoSteamClient") {
            "请确保 Steam 已完全启动并登录账号，然后重试".to_string()
        } else {
            format!("Steam 初始化失败: {}", msg)
        }
    })?;

    let result = f(&client);
    drop(client); // 立即释放并调用 SteamAPI_Shutdown()，退出游戏状态
    result
}

/// 持续调用 run_callbacks 直到 callback 设置结果或超时
fn poll_callback<T>(
    client: &Client,
    result: &Arc<std::sync::Mutex<Option<Result<T, String>>>>,
    timeout_secs: u64,
) -> Result<T, String> {
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

fn direct_search_workshop(
    query: &str,
    page: u32,
    page_size: u32,
    sort_by: &str,
) -> Result<WorkshopSearchResult, String> {
    with_client(|client| {
        let ugc = client.ugc();
        let subscribed = ugc.subscribed_items(true);

        let shared: Arc<std::sync::Mutex<Option<Result<WorkshopSearchResult, String>>>> =
            Arc::new(std::sync::Mutex::new(None));
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
            let mapped = result
                .map(|qr| {
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

pub fn search_workshop(
    query: &str,
    page: u32,
    page_size: u32,
    sort_by: &str,
) -> Result<WorkshopSearchResult, String> {
    ensure_steam_ready()?;
    let _worker_lock = STEAM_WORKER_LOCK.lock();
    let current_exe = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;
    let mut cmd = std::process::Command::new(current_exe);
    let query_param = if query.is_empty() { "__EMPTY__" } else { query };
    cmd.args([
        "--steam-worker",
        "search",
        query_param,
        &page.to_string(),
        &page_size.to_string(),
        sort_by,
    ]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().map_err(|e| format!("启动 Steam Worker 查询失败: {}", e))?;
    if output.status.success() {
        serde_json::from_slice::<WorkshopSearchResult>(&output.stdout)
            .map_err(|e| format!("解析工坊查询结果失败: {}", e))
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err_msg.is_empty() {
            Err(format!("Steam 工坊查询失败 (退出码: {:?})", output.status.code()))
        } else {
            Err(err_msg)
        }
    }
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
/// 直接在进程内执行 Steam 订阅操作（仅供 worker 进程调用）
fn direct_subscribe_mod(published_file_id: u64) -> Result<(), String> {
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<std::sync::Mutex<Option<Result<(), String>>>> =
            Arc::new(std::sync::Mutex::new(None));
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

/// 直接在进程内执行 Steam 取消订阅操作（仅供 worker 进程调用）
fn direct_unsubscribe_mod(published_file_id: u64) -> Result<(), String> {
    with_client(|client| {
        let ugc = client.ugc();
        let shared: Arc<std::sync::Mutex<Option<Result<(), String>>>> =
            Arc::new(std::sync::Mutex::new(None));
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

/// 轻量独立 Worker 子进程入口
pub fn run_steam_worker(args: &[String]) {
    if args.is_empty() {
        std::process::exit(1);
    }
    let action = &args[0];
    match action.as_str() {
        "search" => {
            let raw_query = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let query = if raw_query == "__EMPTY__" { "" } else { raw_query };
            let page = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            let page_size = args.get(3).and_then(|s| s.parse::<u32>().ok()).unwrap_or(20);
            let sort_by = args.get(4).map(|s| s.as_str()).unwrap_or("trending");
            match direct_search_workshop(query, page, page_size, sort_by) {
                Ok(res) => {
                    if let Ok(json) = serde_json::to_string(&res) {
                        print!("{}", json);
                        std::process::exit(0);
                    }
                    std::process::exit(3);
                }
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(2);
                }
            }
        }
        "subscribe" => {
            if let Some(id_str) = args.get(1)
                && let Ok(id) = id_str.parse::<u64>() {
                    match direct_subscribe_mod(id) {
                        Ok(_) => std::process::exit(0),
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(2);
                        }
                    }
                }
            std::process::exit(1);
        }
        "unsubscribe" => {
            if let Some(id_str) = args.get(1)
                && let Ok(id) = id_str.parse::<u64>() {
                    match direct_unsubscribe_mod(id) {
                        Ok(_) => std::process::exit(0),
                        Err(err) => {
                            eprintln!("{}", err);
                            std::process::exit(2);
                        }
                    }
                }
            std::process::exit(1);
        }
        _ => std::process::exit(1),
    }
}

/// 启动自身轻量 Worker 子进程执行 Steam 订阅/取消操作，
/// 从而使 Steamworks 初始化与绑定只发生在短暂的 Worker 进程中，
/// 主 GUI 进程绝不被 Steam 绑定为游戏运行进程，解决误判运行和强杀主程序问题。
fn execute_worker(action: &str, id: u64) -> Result<(), String> {
    ensure_steam_ready()?;
    let _worker_lock = STEAM_WORKER_LOCK.lock();
    let current_exe = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {}", e))?;
    let mut cmd = std::process::Command::new(current_exe);
    cmd.args(["--steam-worker", action, &id.to_string()]);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW
        cmd.creation_flags(0x08000000);
    }

    let output = cmd.output().map_err(|e| format!("启动 Steam Worker 失败: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if err_msg.is_empty() {
            Err(format!("Steam {} 操作失败 (进程退出码: {:?})", action, output.status.code()))
        } else {
            Err(err_msg)
        }
    }
}

pub fn subscribe_mod(published_file_id: u64) -> Result<(), String> {
    execute_worker("subscribe", published_file_id)
}

pub fn unsubscribe_mod(published_file_id: u64) -> Result<(), String> {
    let res = execute_worker("unsubscribe", published_file_id);
    // 同时物理删除本地下载目录中的该创意工坊文件夹，避免取消订阅后因文件残留仍显示在已安装列表中
    remove_workshop_item_dir(published_file_id);
    res
}
pub fn remove_workshop_item_dir(published_file_id: u64) {
    let id_str = published_file_id.to_string();
    let workshop_dirs = crate::integrations::steam::get_workshop_dirs();
    for dir in workshop_dirs {
        let item_path = dir.join(&id_str);
        if item_path.exists() {
            let _ = std::fs::remove_dir_all(&item_path);
        }
    }
}
