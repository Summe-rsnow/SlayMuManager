use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("游戏目录未设置")]
    GameDirNotSet,

    #[error("游戏目录无效: {0}")]
    InvalidGameDir(String),

    #[error("Mod 未找到: {0}")]
    ModNotFound(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("HTTP 错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}
