use crate::domain::remote_mod::{RemoteMod, RemoteModSearchResult};
use crate::utils::error::AppError;
use lru::LruCache;
use serde::Deserialize;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

const NEXUS_GRAPHQL_URL: &str = "https://api.nexusmods.com/v2/graphql";
const NEXUS_GAME_ID: &str = "8916";
const CACHE_CAPACITY: usize = 50;
const CACHE_TTL: Duration = Duration::from_secs(300); // 5 分钟
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;

// ---------------------------------------------------------------------------
// 缓存
// ---------------------------------------------------------------------------

struct CachedEntry {
    result: RemoteModSearchResult,
    created_at: Instant,
}

static SEARCH_CACHE: LazyLock<Mutex<LruCache<String, CachedEntry>>> = LazyLock::new(|| {
    Mutex::new(LruCache::new(NonZeroUsize::new(CACHE_CAPACITY).unwrap()))
});

fn cache_key(query: &str, page: u32, page_size: u32, sort_by: &str) -> String {
    format!("{}:{}:{}:{}", query, page, page_size, sort_by)
}

fn get_cached(key: &str) -> Option<RemoteModSearchResult> {
    let mut cache = SEARCH_CACHE.lock().unwrap();
    if let Some(entry) = cache.get(key) {
        if entry.created_at.elapsed() < CACHE_TTL {
            return Some(entry.result.clone());
        }
        cache.pop(key); // TTL 过期，移除
    }
    None
}

fn set_cache(key: String, result: RemoteModSearchResult) {
    let mut cache = SEARCH_CACHE.lock().unwrap();
    cache.put(key, CachedEntry {
        result,
        created_at: Instant::now(),
    });
}

// ---------------------------------------------------------------------------
// 懒初始化 Client（按需创建，代理变化时重建）
// ---------------------------------------------------------------------------

struct ClientState {
    client: reqwest::blocking::Client,
    proxy_url: Option<String>,
    api_key: String,
}

static CLIENT: std::sync::OnceLock<Mutex<Option<ClientState>>> = std::sync::OnceLock::new();

fn get_or_create_client(api_key: &str, proxy_url: Option<&str>) -> Result<reqwest::blocking::Client, AppError> {
    let lock = CLIENT.get_or_init(|| Mutex::new(None));
    let mut guard = lock.lock().unwrap();

    // 代理或 API key 变化时重建
    let needs_rebuild = match &*guard {
        Some(state) => state.api_key != api_key || state.proxy_url.as_deref() != proxy_url,
        None => true,
    };

    if needs_rebuild {
        let client = build_client_inner(api_key, proxy_url)?;
        *guard = Some(ClientState {
            client: client.clone(),
            proxy_url: proxy_url.map(|s| s.to_string()),
            api_key: api_key.to_string(),
        });
        return Ok(client);
    }

    Ok(guard.as_ref().unwrap().client.clone())
}

// ---------------------------------------------------------------------------
// 重试 + 指数退避
// ---------------------------------------------------------------------------

fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(match attempt {
        0 => 1,
        1 => 3,
        _ => 6,
    })
}

/// 发送 GraphQL 请求，带重试
fn send_graphql_request(
    client: &reqwest::blocking::Client,
    gql_query: &str,
) -> Result<RemoteModSearchResult, AppError> {
    let mut last_error = None;

    'retry: for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            std::thread::sleep(retry_delay(attempt - 1));
        }

        let resp = match client
            .post(NEXUS_GRAPHQL_URL)
            .header("content-type", "application/json")
            .json(&serde_json::json!({ "query": gql_query }))
            .send()
        {
            Ok(r) => r,
            Err(e) => {
                let is_retryable = e.is_timeout() || e.is_connect();
                if is_retryable && attempt < MAX_RETRIES - 1 {
                    last_error = Some(AppError::Other(format!("Nexus GraphQL 请求失败: {}", e)));
                    continue 'retry;
                }
                return Err(AppError::Other(format!("Nexus GraphQL 请求失败: {}", e)));
            }
        };

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            let code = status.as_u16();
            if code == 401 || code == 403 {
                return Err(AppError::Other(format!(
                    "Nexus API 认证失败 ({}): 请检查 API Key 是否正确", code
                )));
            }
            if (code == 429 || code >= 500) && attempt < MAX_RETRIES - 1 {
                last_error = Some(AppError::Other(format!(
                    "Nexus API 返回错误: {} {}", code, body
                )));
                continue 'retry;
            }
            return Err(AppError::Other(format!(
                "Nexus API 返回错误: {} {}", code, body
            )));
        }

        let raw: GraphQLResponse = match resp.json() {
            Ok(r) => r,
            Err(e) => return Err(AppError::Other(format!("Nexus GraphQL 解析失败: {}", e))),
        };

        return Ok(convert_response(raw));
    }

    Err(last_error.unwrap_or_else(|| AppError::Other("Nexus 请求重试耗尽".to_string())))
}

fn convert_response(raw: GraphQLResponse) -> RemoteModSearchResult {
    // 注意：game_slug 在外部传入，这里我们用 "slaythespire2"
    // 实际 game_slug 应该从上下文获取，这里简化处理
    let game_slug = "slaythespire2";
    let mods_data = raw.data.mods;
    let items: Vec<RemoteMod> = mods_data
        .nodes
        .into_iter()
        .map(|m| RemoteMod {
            remote_id: m.mod_id.to_string(),
            provider: "nexusmods".to_string(),
            name: m.name,
            summary: m.summary,
            author: m.author,
            latest_version: m.version,
            picture_url: m.picture_url,
            thumbnail_url: m.thumbnail_url,
            thumbnail_large_url: m.thumbnail_large_url,
            detail_url: format!("https://www.nexusmods.com/{}/mods/{}", game_slug, m.mod_id),
            endorsement_count: m.endorsements.unwrap_or(0) as u32,
            download_count: m.downloads.unwrap_or(0) as u32,
            unique_downloads: 0,
        })
        .collect();

    let count = items.len() as u32;
    RemoteModSearchResult {
        items,
        total_count: mods_data.total_count as u32,
        offset: 0,
        count,
    }
}

// ---------------------------------------------------------------------------
// 公开 API
// ---------------------------------------------------------------------------

/// 搜索 Nexus Mods（GraphQL v2），带缓存 + 重试
pub fn search_mods(
    game_slug: &str,
    query: &str,
    page: u32,
    page_size: u32,
    sort_by: &str,
    api_key: &str,
    proxy_url: Option<&str>,
) -> Result<RemoteModSearchResult, AppError> {
    let ck = cache_key(query, page, page_size, sort_by);
    if let Some(cached) = get_cached(&ck) {
        return Ok(cached);
    }

    let offset = (page.max(1) - 1) * page_size;
    let sort = graphql_sort_clause(sort_by);

    let filter = if query.is_empty() {
        format!("gameId: [{{ value: \"{}\", op: EQUALS }}]", NEXUS_GAME_ID)
    } else {
        let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "op: AND, filter: [{{ gameId: [{{ value: \"{}\", op: EQUALS }}] }}, {{ op: OR, filter: [{{ name: [{{ value: \"{}\", op: WILDCARD }}] }}, {{ description: [{{ value: \"{}\", op: MATCHES }}] }}] }}]",
            NEXUS_GAME_ID, escaped, escaped
        )
    };

    let gql_query = format!(
        "{{ mods(filter: {{ {} }}, sort: [{}], offset: {}, count: {}) {{ totalCount nodes {{ modId name summary author version pictureUrl thumbnailUrl thumbnailLargeUrl endorsements downloads }} }} }}",
        filter, sort, offset, page_size
    );

    let client = get_or_create_client(api_key, proxy_url)?;
    let mut result = send_graphql_request(&client, &gql_query)?;

    // 回填正确的 offset 和 game_slug detail_url
    result.offset = offset;
    for item in &mut result.items {
        item.detail_url = format!("https://www.nexusmods.com/{}/mods/{}", game_slug, item.remote_id);
    }

    set_cache(ck, result.clone());
    Ok(result)
}

/// 测试代理连通性
pub fn test_proxy(proxy_url: &str) -> Result<bool, AppError> {
    let client = reqwest::blocking::Client::builder()
        .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| AppError::Other(format!("代理配置错误: {}", e)))?)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Other(format!("客户端构建失败: {}", e)))?;

    match client.get("https://api.nexusmods.com/v1/games.json").send() {
        Ok(resp) => Ok(resp.status().is_success() || resp.status().as_u16() == 401),
        Err(_) => Ok(false),
    }
}

// ---------------------------------------------------------------------------
// 内部工具
// ---------------------------------------------------------------------------

fn graphql_sort_clause(sort_by: &str) -> &str {
    match sort_by {
        "latest_updated" => "{ updatedAt: { direction: DESC } }",
        "trending" => "{ endorsements: { direction: DESC } }",
        "downloads" => "{ downloads: { direction: DESC } }",
        _ => "{ createdAt: { direction: DESC } }",
    }
}

fn build_client_inner(api_key: &str, proxy_url: Option<&str>) -> Result<reqwest::blocking::Client, AppError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "apikey",
        reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|e| AppError::Other(format!("API Key 格式错误: {}", e)))?,
    );
    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_static("SlayMuManager/0.1.0"),
    );
    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let mut builder = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT);

    if let Some(proxy) = proxy_url {
        if !proxy.is_empty() {
            builder = builder
                .proxy(reqwest::Proxy::all(proxy).map_err(|e| AppError::Other(format!("代理配置错误: {}", e)))?);
        }
    }

    builder
        .build()
        .map_err(|e| AppError::Other(format!("HTTP 客户端构建失败: {}", e)))
}

// ---------------------------------------------------------------------------
// GraphQL v2 响应类型
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GraphQLResponse {
    data: GraphQLData,
}

#[derive(Deserialize)]
struct GraphQLData {
    mods: GraphQLMods,
}

#[derive(Deserialize)]
struct GraphQLMods {
    #[serde(default)]
    #[serde(rename = "totalCount")]
    total_count: u64,
    #[serde(default)]
    nodes: Vec<GraphQLNode>,
}

#[derive(Deserialize)]
struct GraphQLNode {
    #[serde(rename = "modId")]
    mod_id: u32,
    name: String,
    summary: Option<String>,
    author: Option<String>,
    version: Option<String>,
    #[serde(rename = "pictureUrl")]
    picture_url: Option<String>,
    #[serde(rename = "thumbnailUrl")]
    thumbnail_url: Option<String>,
    #[serde(rename = "thumbnailLargeUrl")]
    thumbnail_large_url: Option<String>,
    endorsements: Option<u64>,
    downloads: Option<u64>,
}
