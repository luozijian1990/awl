//! 领域模型：`WorkEntry`（事实源）、`WorkCalendar`、`CalendarProjection`，
//! 以及创建/更新用的 input/patch DTO。
//!
//! 约束：`WorkEntry` 不持有任何外部日历事件 id；calendar event id 只出现在
//! [`CalendarProjection`] 中（Phase 2 同步投影）。

use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Result, WorklogError};

/// 工作项状态。报表只使用 [`Status::Confirmed`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Draft,
    Confirmed,
    Archived,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Draft => "draft",
            Status::Confirmed => "confirmed",
            Status::Archived => "archived",
        }
    }
}

impl FromStr for Status {
    type Err = WorklogError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "draft" => Ok(Status::Draft),
            "confirmed" => Ok(Status::Confirmed),
            "archived" => Ok(Status::Archived),
            other => Err(WorklogError::Invalid(format!("unknown status: {other}"))),
        }
    }
}

/// 记录创建者。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Actor {
    Ai,
    Human,
}

impl Actor {
    pub fn as_str(self) -> &'static str {
        match self {
            Actor::Ai => "ai",
            Actor::Human => "human",
        }
    }
}

impl FromStr for Actor {
    type Err = WorklogError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ai" => Ok(Actor::Ai),
            "human" => Ok(Actor::Human),
            other => Err(WorklogError::Invalid(format!("unknown actor: {other}"))),
        }
    }
}

/// 已知的 source 值。允许 custom string，但必须非空。
pub const KNOWN_SOURCES: &[&str] = &["codex", "claude", "cli", "desktop", "manual"];

/// 记录来源。已知值见 [`KNOWN_SOURCES`]，同时允许任意 custom 非空字符串。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Source(String);

impl Source {
    /// 校验并构造。空白字符串视为非法。
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(WorklogError::Invalid("source must not be empty".into()));
        }
        Ok(Source(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 是否属于 [`KNOWN_SOURCES`]。
    pub fn is_known(&self) -> bool {
        KNOWN_SOURCES.contains(&self.0.as_str())
    }
}

impl FromStr for Source {
    type Err = WorklogError;

    fn from_str(s: &str) -> Result<Self> {
        Source::new(s)
    }
}

/// 同步投影状态（Phase 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Pending,
    Synced,
    Failed,
}

impl SyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncStatus::Pending => "pending",
            SyncStatus::Synced => "synced",
            SyncStatus::Failed => "failed",
        }
    }
}

impl FromStr for SyncStatus {
    type Err = WorklogError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(SyncStatus::Pending),
            "synced" => Ok(SyncStatus::Synced),
            "failed" => Ok(SyncStatus::Failed),
            other => Err(WorklogError::Invalid(format!(
                "unknown sync_status: {other}"
            ))),
        }
    }
}

/// 本地工作日历（分组 + 桌面视图颜色元数据）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCalendar {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub color: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建工作日历的输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkCalendar {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}

/// 局部更新工作日历。仅 `Some` 字段会被写入。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkCalendarPatch {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// 事实源工作项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkEntry {
    pub id: i64,
    pub uid: String,
    pub calendar_id: i64,
    pub title: String,
    pub body: Option<String>,
    pub raw_input: Option<String>,
    pub project: Option<String>,
    pub status: Status,
    pub actor: Actor,
    pub source: Source,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub evidence: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建工作项的输入。`status`/`calendar_id` 为 `None` 时由仓储补默认值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWorkEntry {
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub raw_input: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    /// `None` → 仓储按 actor/source 决定默认（AI 来源默认 `draft`）。
    #[serde(default)]
    pub status: Option<Status>,
    pub actor: Actor,
    pub source: Source,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    /// `None` → 使用默认日历。
    #[serde(default)]
    pub calendar_id: Option<i64>,
}

/// 局部更新工作项。仅 `Some` 字段会被写入。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkEntryPatch {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub raw_input: Option<String>,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub actor: Option<Actor>,
    #[serde(default)]
    pub source: Option<Source>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub evidence: Option<Vec<String>>,
    #[serde(default)]
    pub calendar_id: Option<i64>,
}

/// Phase 2 同步投影。`external_event_id` 只在这里出现。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarProjection {
    pub id: i64,
    pub work_entry_id: i64,
    pub provider: String,
    pub external_event_id: Option<String>,
    pub sync_status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_parses_and_renders() {
        assert_eq!(Status::from_str("draft").unwrap(), Status::Draft);
        assert_eq!(Status::from_str("confirmed").unwrap(), Status::Confirmed);
        assert_eq!(Status::from_str("archived").unwrap(), Status::Archived);
        assert!(Status::from_str("bogus").is_err());
        assert_eq!(Status::Confirmed.as_str(), "confirmed");
    }

    #[test]
    fn actor_parses_and_rejects_unknown() {
        assert_eq!(Actor::from_str("ai").unwrap(), Actor::Ai);
        assert_eq!(Actor::from_str("human").unwrap(), Actor::Human);
        assert!(Actor::from_str("robot").is_err());
    }

    #[test]
    fn source_allows_known_and_custom_but_not_empty() {
        assert!(Source::new("codex").unwrap().is_known());
        assert!(Source::new("claude").unwrap().is_known());
        assert!(!Source::new("my-internal-tool").unwrap().is_known());
        assert!(Source::new("   ").is_err());
        assert!(Source::new("").is_err());
    }

    #[test]
    fn sync_status_roundtrip() {
        assert_eq!(
            SyncStatus::from_str("pending").unwrap(),
            SyncStatus::Pending
        );
        assert!(SyncStatus::from_str("nope").is_err());
        assert_eq!(SyncStatus::Failed.as_str(), "failed");
    }
}
