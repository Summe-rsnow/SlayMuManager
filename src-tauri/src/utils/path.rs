use std::path::{Path, PathBuf};

/// 确保目录存在，不存在则创建（含父目录）
pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 获取路径的规范化形式（解析符号链接、去掉 `..` 等）
pub fn canonicalize(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok()
}

/// 递归收集目录下所有文件的相对路径
pub fn collect_relative_paths(base: &Path, dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().to_string());
            }
            if path.is_dir() {
                collect_relative_paths(base, &path, out);
            }
        }
    }
}

/// 获取 Windows AppData 下的应用配置目录
pub fn app_data_dir() -> PathBuf {
    dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.slay.mumanager")
}
