use sha1::{Digest, Sha1};
use std::io::Read;
use std::path::Path;

/// 计算文件的 SHA1 哈希值，返回十六进制字符串
pub fn sha1_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>())
}

