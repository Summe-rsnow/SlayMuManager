use crate::domain::mod_entity::InstalledMod;
use crate::domain::remote_mod::RemoteMod;
use crate::integrations::nexus_client;
use crate::services::mod_service;
use crate::utils::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateInfo {
    pub mod_id: String,
    pub name: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub has_update: bool,
    pub remote_mod: Option<RemoteMod>,
}

/// 更新检查的完整缓存结果（含时间戳和统计）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateCheckCache {
    pub results: Vec<ModUpdateInfo>,
    pub checked_at: String,
    pub total_mods: usize,
    pub updated_mods: usize,
}

// ---------------------------------------------------------------------------
// 公开 API
// ---------------------------------------------------------------------------

/// 检测所有已安装 Mod 是否有 Nexus 更新
///
/// 流程：
/// 1. 扫描 mods/ + mods_disabled/ 下所有 Mod
/// 2. 对每个有名称的 Mod，搜索 Nexus 并精确匹配名称（大小写不敏感）
/// 3. 比较版本号，返回更新信息
/// 4. 最多 3 个并发 HTTP 请求
pub fn check_mod_updates(game_root: &Path) -> Result<Vec<ModUpdateInfo>, AppError> {
    // 清除上次的搜索缓存，确保拿到最新结果
    nexus_client::clear_search_cache();

    let settings = crate::repositories::settings_repo::load_settings().unwrap_or_default();
    let api_key = match &settings.nexus_api_key {
        Some(k) => k.clone(),
        None => return Ok(Vec::new()), // 无 API Key 不报错，直接返回空
    };

    // 收集所有 Mod（已启用 + 已禁用），过滤无意义名称
    let enabled = mod_service::scan_enabled_mods(game_root);
    let disabled = mod_service::scan_disabled_mods(game_root);

    let all_mods: Vec<&InstalledMod> = enabled
        .iter()
        .chain(disabled.iter())
        .filter(|m| !m.name.is_empty() && !m.name.starts_with("unknown:"))
        .collect();

    if all_mods.is_empty() {
        return Ok(Vec::new());
    }

    let proxy_url = settings.proxy_url.clone();
    let mut results = Vec::new();

    // 分块处理：每批 3 个并发，避免 Nexus API 限流
    for chunk in all_mods.chunks(3) {
        let batch = std::sync::Mutex::new(Vec::new());

        std::thread::scope(|s| {
            for mod_item in chunk {
                s.spawn(|| {
                    let id = &mod_item.id;
                    let name = &mod_item.name;
                    let local_ver = &mod_item.version;
                    let info = check_single_mod(
                        id, name, local_ver.as_deref(),
                        &api_key, proxy_url.as_deref(),
                    );
                    batch.lock().unwrap().push(info);
                });
            }
        });

        results.extend(batch.into_inner().unwrap());
    }

    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(results)
}

// ---------------------------------------------------------------------------
// 内部逻辑
// ---------------------------------------------------------------------------

/// 检查单个 Mod 的更新
fn check_single_mod(
    mod_id: &str,
    mod_name: &str,
    local_version: Option<&str>,
    api_key: &str,
    proxy_url: Option<&str>,
) -> ModUpdateInfo {
    // 搜索 Nexus（按名称，取前 10 条）
    let search_result = match nexus_client::search_mods(
        "slaythespire2",
        mod_name,
        1,
        10,
        "latest_added",
        api_key,
        proxy_url,
    ) {
        Ok(r) => r,
        Err(_) => {
            return ModUpdateInfo {
                mod_id: mod_id.to_string(),
                name: mod_name.to_string(),
                local_version: local_version.map(|s| s.to_string()),
                remote_version: None,
                has_update: false,
                remote_mod: None,
            };
        }
    };

    // 精确匹配名称（大小写不敏感）
    let matched = search_result
        .items
        .into_iter()
        .find(|remote| remote.name.to_lowercase() == mod_name.to_lowercase());

    match matched {
        Some(rm) => {
            let has_update = match (local_version, rm.latest_version.as_deref()) {
                (Some(local), Some(remote)) => compare_versions(local, remote),
                (None, Some(_)) => true,  // 本地无版本号 → 认为有更新
                _ => false,
            };

            ModUpdateInfo {
                mod_id: mod_id.to_string(),
                name: mod_name.to_string(),
                local_version: local_version.map(|s| s.to_string()),
                remote_version: rm.latest_version.clone(),
                has_update,
                remote_mod: Some(rm),
            }
        }
        None => ModUpdateInfo {
            mod_id: mod_id.to_string(),
            name: mod_name.to_string(),
            local_version: local_version.map(|s| s.to_string()),
            remote_version: None,
            has_update: false,
            remote_mod: None,
        },
    }
}

/// 版本号比较：local < remote → true（有更新）
///
/// 规则：
/// - 去除首字母 "v"
/// - 按 `.` `-` `+` 分段
/// - 每段优先数值比较，回退到字符串比较
fn compare_versions(local: &str, remote: &str) -> bool {
    let local = local.strip_prefix('v').or_else(|| local.strip_prefix('V')).unwrap_or(local);
    let remote = remote.strip_prefix('v').or_else(|| remote.strip_prefix('V')).unwrap_or(remote);

    let local_parts: Vec<&str> = local
        .split(|c| c == '.' || c == '-' || c == '+')
        .collect();
    let remote_parts: Vec<&str> = remote
        .split(|c| c == '.' || c == '-' || c == '+')
        .collect();

    let max_len = local_parts.len().max(remote_parts.len());

    for i in 0..max_len {
        let l = local_parts.get(i).unwrap_or(&"0");
        let r = remote_parts.get(i).unwrap_or(&"0");

        // 优先数值比较
        match (l.parse::<u64>(), r.parse::<u64>()) {
            (Ok(ln), Ok(rn)) => {
                if ln < rn {
                    return true;
                }
                if ln > rn {
                    return false;
                }
            }
            _ => {
                // 字符串回退
                if l < r {
                    return true;
                }
                if l > r {
                    return false;
                }
            }
        }
    }

    false // 相等
}
