//! core 统一错误类型。仓储与校验失败都以可预期的 `WorklogError` 返回。

#[derive(Debug, thiserror::Error)]
pub enum WorklogError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 校验失败（非法 status/actor/source、空 title、时间区间非法等）。
    #[error("invalid value: {0}")]
    Invalid(String),

    /// 目标记录不存在。
    #[error("not found: {0}")]
    NotFound(String),

    /// 无法解析出 OS app data 目录。
    #[error("could not resolve application data directory")]
    NoDataDir,
}

pub type Result<T> = std::result::Result<T, WorklogError>;
