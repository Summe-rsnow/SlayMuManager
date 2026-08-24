use std::path::PathBuf;

// Slay the Spire 2 的 Steam App ID（游戏检测 + 云存档）
const STS2_APP_ID: &str = "2868840";

/// SteamID64 = AccountID + 76561197960265728
const STEAM_ID64_OFFSET: u64 = 76561197960265728;

/// 将 AccountID（短号）转换为 SteamID64（17 位）
pub fn account_id_to_steam_id64(account_id: u64) -> u64 {
    account_id + STEAM_ID64_OFFSET
}

/// 获取当前活跃的 Steam AccountID（短号，用于 userdata 目录查找）
pub fn get_active_steam_account_id() -> Option<String> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(active) = hkcu.open_subkey("SOFTWARE\\Valve\\Steam\\ActiveProcess")
        && let Ok(user_id) = active.get_value::<u32, _>("ActiveUser") {
            return Some(user_id.to_string());
        }
    None
}

/// 从 Windows 注册表读取 Steam 安装路径（多源尝试）
pub fn get_steam_install_path() -> Option<PathBuf> {
    // 优先尝试 HKLM（最常见的安装位置）
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);

    // 64-bit 注册表路径
    if let Ok(steam_key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
        && let Ok(steam_path) = steam_key.get_value::<String, _>("InstallPath") {
            let path = PathBuf::from(&steam_path);
            if path.exists() {
                return Some(path);
            }
        }

    // 原生 64 位路径
    if let Ok(steam_key) = hklm.open_subkey("SOFTWARE\\Valve\\Steam")
        && let Ok(steam_path) = steam_key.get_value::<String, _>("InstallPath") {
            let path = PathBuf::from(&steam_path);
            if path.exists() {
                return Some(path);
            }
        }

    // HKCU 当前用户路径（兜底）
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(steam_key) = hkcu.open_subkey("SOFTWARE\\Valve\\Steam")
        && let Ok(steam_path) = steam_key.get_value::<String, _>("SteamPath") {
            let path = PathBuf::from(&steam_path);
            if path.exists() {
                return Some(path);
            }
        }

    None
}

/// 解析 libraryfolders.vdf，获取包含 STS2 的库路径优先，
/// 然后补充其他库。返回 (common 目录, 是否包含 STS2)
fn get_steam_library_folders() -> Vec<(PathBuf, bool)> {
    let mut entries: Vec<(PathBuf, bool)> = Vec::new();

    let steam_path = match get_steam_install_path() {
        Some(p) => p,
        None => return entries,
    };

    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    let content = match std::fs::read_to_string(&vdf_path) {
        Ok(c) => c,
        Err(_) => {
            // VDF 不存在，回退到默认路径
            entries.push((steam_path.join("steamapps").join("common"), false));
            return entries;
        }
    };

    // 解析 VDF：逐行扫描，提取每个库的 path 并检查其 apps 是否包含 STS2
    let mut current_lib_path: Option<PathBuf> = None;
    let mut current_has_sts2 = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检测 "path" 行 → 提取库路径
        if let Some(rest) = trimmed.strip_prefix("\"path\"") {
            let rest = rest.trim();
            if let Some(start) = rest.find('"') {
                let after_quote = &rest[start + 1..];
                if let Some(end) = after_quote.find('"') {
                    // 保存上一个库
                    if let Some(lib) = current_lib_path.take() {
                        entries.push((lib.join("steamapps").join("common"), current_has_sts2));
                    }
                    let lib_path = after_quote[..end].replace("\\\\", "\\");
                    current_lib_path = Some(PathBuf::from(lib_path));
                    current_has_sts2 = false;
                }
            }
        }

        // 检测是否包含 STS2 的 App ID
        if current_lib_path.is_some() && trimmed.contains(STS2_APP_ID) {
            current_has_sts2 = true;
        }
    }

    // 保存最后一个库
    if let Some(lib) = current_lib_path.take() {
        entries.push((lib.join("steamapps").join("common"), current_has_sts2));
    }

    // 如果 VDF 解析不到任何路径，回退到默认路径
    if entries.is_empty() {
        entries.push((steam_path.join("steamapps").join("common"), false));
    }

    // 包含 STS2 的库排前面
    entries.sort_by_key(|b| std::cmp::Reverse(b.1));
    entries
}

/// 核心检测函数：三关检测游戏安装
/// 1. Steam 默认路径（<Steam>/steamapps/common/Slay the Spire 2/）
/// 2. Steam 库扫描（libraryfolders.vdf）
///
/// 返回 (路径, 来源标签)
pub fn find_game_install() -> Vec<(PathBuf, String)> {
    let mut results: Vec<(PathBuf, String)> = Vec::new();

    let steam_path = match get_steam_install_path() {
        Some(p) => p,
        None => return results,
    };

    // 第1关：Steam 默认路径
    let default_path = steam_path.join("steamapps").join("common").join("Slay the Spire 2");
    if default_path.exists() {
        results.push((default_path, "Steam default".to_string()));
    }

    // 第2关：libraryfolders.vdf 多库扫描
    for (common_dir, has_sts2) in get_steam_library_folders() {
        let game_path = common_dir.join("Slay the Spire 2");
        if game_path.exists() && !results.iter().any(|(p, _)| p == &game_path) {
            let source = if has_sts2 {
                "Steam library".to_string()
            } else {
                "Steam library (other)".to_string()
            };
            results.push((game_path, source));
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Steam 云存档
// ---------------------------------------------------------------------------

/// 获取 Steam userdata 目录路径
pub fn get_userdata_dir() -> Option<PathBuf> {
    let steam_path = get_steam_install_path()?;
    let userdata = steam_path.join("userdata");
    if userdata.exists() {
        Some(userdata)
    } else {
        None
    }
}

/// 获取指定 Steam 用户的云存档目录
pub fn get_cloud_save_dir(steam_user_id: &str) -> Option<PathBuf> {
    let userdata = get_userdata_dir()?;
    let cloud_dir = userdata.join(steam_user_id).join(STS2_APP_ID).join("remote");
    if cloud_dir.exists() {
        Some(cloud_dir)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Steam 创意工坊
// ---------------------------------------------------------------------------

/// 获取所有 Steam 库路径下的 workshop 目录（用于扫描 STS2 创意工坊 Mod）
pub fn get_workshop_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let steam_path = match get_steam_install_path() {
        Some(p) => p,
        None => return dirs,
    };

    // 默认路径
    let default = steam_path.join("steamapps").join("workshop").join("content").join(STS2_APP_ID);
    if default.exists() {
        dirs.push(default);
    }

    // 从 libraryfolders.vdf 扫描其他库
    let vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("\"path\"") {
                let rest = rest.trim();
                if let Some(start) = rest.find('"') {
                    let after_quote = &rest[start + 1..];
                    if let Some(end) = after_quote.find('"') {
                        let lib_path = after_quote[..end].replace("\\\\", "\\");
                        let candidate = PathBuf::from(&lib_path)
                            .join("steamapps")
                            .join("workshop")
                            .join("content")
                            .join(STS2_APP_ID);
                        if candidate.exists() && !dirs.iter().any(|d| d == &candidate) {
                            dirs.push(candidate);
                        }
                    }
                }
            }
        }
    }

    dirs
}

/// 列出所有 Steam 用户 ID（扫描 userdata 目录下的数字子目录）
/// 优先返回注册表中的活跃用户
pub fn list_steam_users() -> Vec<String> {
    let mut users = Vec::new();

    // 先尝试从注册表获取活跃用户
    if let Some(active) = get_active_steam_account_id() {
        users.push(active);
    }

    // 再扫描 userdata 目录补充其他用户
    let userdata = match get_userdata_dir() {
        Some(d) => d,
        None => return users,
    };

    if let Ok(entries) = std::fs::read_dir(&userdata) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // 纯数字目录即为 Steam 用户（AccountID 至少 1 位）
                if name.chars().all(|c| c.is_ascii_digit()) && !name.is_empty()
                    && !users.contains(&name.to_string()) {
                        users.push(name.to_string());
                    }
            }
        }
    }
    users
}
