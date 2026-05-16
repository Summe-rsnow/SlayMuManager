use crate::domain::remote_mod::{RemoteMod, RemoteModSearchResult};
use crate::utils::error::AppError;
use serde::Deserialize;

const NEXUS_GRAPHQL_URL: &str = "https://api.nexusmods.com/v2/graphql";
const NEXUS_GAME_ID: &str = "8916";
// NEXUS_GAME_DOMAIN = "slaythespire2" — 通过 game_slug 参数传入
const PAGE_SIZE: u32 = 18;

/// 将前端 sortBy 映射为 GraphQL sort 字段
fn graphql_sort_clause(sort_by: &str) -> &str {
    match sort_by {
        "latest_updated" => "{ updatedAt: { direction: DESC } }",
        "trending" => "{ endorsements: { direction: DESC } }",
        "downloads" => "{ downloads: { direction: DESC } }",
        _ => "{ createdAt: { direction: DESC } }", // latest_added 默认
    }
}

/// 搜索 Nexus Mods（GraphQL v2）
pub fn search_mods(
    game_slug: &str,
    query: &str,
    page: u32,
    sort_by: &str,
    api_key: &str,
    proxy_url: Option<&str>,
) -> Result<RemoteModSearchResult, AppError> {
    let offset = (page.max(1) - 1) * PAGE_SIZE;
    let sort = graphql_sort_clause(sort_by);

    // 构建 GraphQL 过滤器
    let filter = if query.is_empty() {
        format!("filter: {{ gameId: {{ value: \"{}\", op: EQUALS }} }}", NEXUS_GAME_ID)
    } else {
        // 转义查询中的双引号和反斜杠
        let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "filter: {{ gameId: {{ value: \"{}\", op: EQUALS }}, OR: [{{ name: {{ value: \"{}\", op: EQUALS }} }}, {{ description: {{ value: \"{}\", op: MATCHES }} }}] }}",
            NEXUS_GAME_ID, escaped, escaped
        )
    };

    let gql_query = format!(
        "{{ mods({}, sort: [{}], offset: {}, count: {}) {{ totalCount nodes {{ modId name summary author version pictureUrl thumbnailUrl thumbnailLargeUrl endorsements downloads }} }} }}",
        filter, sort, offset, PAGE_SIZE
    );

    let client = build_client(api_key, proxy_url)?;
    let resp = client
        .post(NEXUS_GRAPHQL_URL)
        .header("content-type", "application/json")
        .json(&serde_json::json!({ "query": gql_query }))
        .send()
        .map_err(|e| AppError::Other(format!("Nexus GraphQL 请求失败: {}", e)))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(AppError::Other(format!(
                "Nexus API 认证失败 ({}): 请检查 API Key 是否正确",
                status.as_u16()
            )));
        }
        return Err(AppError::Other(format!(
            "Nexus API 返回错误: {} {}",
            status.as_u16(), body
        )));
    }

    let raw: GraphQLResponse = resp
        .json()
        .map_err(|e| AppError::Other(format!("Nexus GraphQL 解析失败: {}", e)))?;

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
    Ok(RemoteModSearchResult {
        items,
        total_count: mods_data.total_count as u32,
        offset,
        count,
    })
}

/// 测试代理连通性
pub fn test_proxy(proxy_url: &str) -> Result<bool, AppError> {
    let client = reqwest::blocking::Client::builder()
        .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| AppError::Other(format!("代理配置错误: {}", e)))?)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Other(format!("客户端构建失败: {}", e)))?;

    match client.get("https://api.nexusmods.com/v1/games.json").send() {
        Ok(resp) => Ok(resp.status().is_success() || resp.status().as_u16() == 401), // 401 = OK but needs auth
        Err(_) => Ok(false),
    }
}

// ---------------------------------------------------------------------------
// 内部工具
// ---------------------------------------------------------------------------

fn build_client(api_key: &str, proxy_url: Option<&str>) -> Result<reqwest::blocking::Client, AppError> {
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
        .timeout(std::time::Duration::from_secs(30));

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
// GraphQL v2 响应类型（搜索）
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

