use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

/// Mod 的 manifest JSON 结构
/// 参考项目要求 4 个必需字段：id, name, has_pck, has_dll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub affects_gameplay: bool,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_dependencies")]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub has_pck: Option<bool>,
    #[serde(default)]
    pub has_dll: Option<bool>,
}

impl ModManifest {
    /// 严格验证：id、name 非空，has_pck、has_dll 必须存在
    pub fn is_valid(&self) -> bool {
        let has_id = self.id.as_ref().map_or(false, |s| !s.trim().is_empty());
        let has_name = self.name.as_ref().map_or(false, |s| !s.trim().is_empty());
        let has_pck = self.has_pck.is_some();
        let has_dll = self.has_dll.is_some();
        has_id && has_name && has_pck && has_dll
    }

    /// 从 JSON 字符串解析（支持 UTF-8 BOM）
    pub fn parse_json(json: &str) -> Result<Self, serde_json::Error> {
        // 剥离 UTF-8 BOM 头
        let cleaned = json.strip_prefix('\u{FEFF}').unwrap_or(json);
        serde_json::from_str(cleaned)
    }

    /// 从文件路径读取并解析（支持 UTF-8 和 UTF-16 LE 编码）
    pub fn from_file(path: &Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;

        let content = if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
            // UTF-16 LE (BOM: 0xFF 0xFE)
            let utf16: Vec<u16> = bytes[2..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16(&utf16).ok()?
        } else {
            // UTF-8（parse_json 会处理 UTF-8 BOM）
            String::from_utf8(bytes).ok()?
        };

        Self::parse_json(&content).ok()
    }

    /// 在 Mod 目录中查找 manifest 文件
    /// 优先级：<folderName>.json → mod_manifest.json → manifest.json → 任意 .json
    pub fn find_in_dir(mod_dir: &Path) -> Option<(PathBuf, Self)> {
        let folder_name = mod_dir.file_name()?.to_str()?;

        // 1. <folderName>.json
        let named = mod_dir.join(format!("{}.json", folder_name));
        if let Some(m) = Self::from_file(&named) {
            return Some((named, m));
        }

        // 2. mod_manifest.json
        let alt = mod_dir.join("mod_manifest.json");
        if let Some(m) = Self::from_file(&alt) {
            return Some((alt, m));
        }

        // 3. manifest.json
        let def = mod_dir.join("manifest.json");
        if let Some(m) = Self::from_file(&def) {
            return Some((def, m));
        }

        // 4. 任意 .json 兜底
        if let Ok(entries) = std::fs::read_dir(mod_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |e| e == "json") {
                    if let Some(m) = Self::from_file(&p) {
                        return Some((p, m));
                    }
                }
            }
        }

        None
    }

}

/// 依赖字段兼容两种格式：
/// - 字符串数组：["BaseLib"]
/// - 对象数组：[{"id": "BaseLib", "min_version": "3.3.0"}]
fn deserialize_dependencies<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;
    let raw: Value = Deserialize::deserialize(deserializer)?;
    match raw {
        Value::Array(arr) => Ok(arr
            .into_iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s),
                Value::Object(mut m) => m
                    .remove("id")
                    .and_then(|v| v.as_str().map(String::from)),
                _ => None,
            })
            .collect()),
        _ => Ok(Vec::new()),
    }
}
