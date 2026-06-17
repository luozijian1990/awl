//! 同步适配器 trait surface：在 Section 4 定义。
//!
//! MVP 只保留接口与 projection 类型，不实现 Google / macOS Calendar / ICS /
//! 企业日历等任何实际同步。

use serde::{Deserialize, Serialize};

use crate::{Result, SyncStatus, WorkEntry};

/// 同步投影草稿。MVP 只用于定义 phase 2 适配器边界，不执行外部同步。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionDraft {
    pub work_entry_id: i64,
    pub provider: String,
    pub external_event_id: Option<String>,
    pub sync_status: SyncStatus,
    pub last_error: Option<String>,
}

/// Phase 2 外部日历同步适配器接口。MVP 不提供任何具体实现。
pub trait SyncAdapter {
    fn provider(&self) -> &'static str;

    fn project(&self, entry: &WorkEntry) -> Result<ProjectionDraft>;
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::SyncAdapter;
    use crate::{Actor, Source, Status, SyncStatus, WorkEntry};

    struct FakeAdapter;

    impl super::SyncAdapter for FakeAdapter {
        fn provider(&self) -> &'static str {
            "fake"
        }

        fn project(&self, entry: &WorkEntry) -> crate::Result<super::ProjectionDraft> {
            Ok(super::ProjectionDraft {
                work_entry_id: entry.id,
                provider: self.provider().to_string(),
                external_event_id: None,
                sync_status: SyncStatus::Pending,
                last_error: None,
            })
        }
    }

    #[test]
    fn sync_adapter_trait_can_create_projection_draft_without_external_sync() {
        let adapter = FakeAdapter;
        let entry = entry();

        let draft = adapter.project(&entry).unwrap();

        assert_eq!(adapter.provider(), "fake");
        assert_eq!(draft.work_entry_id, entry.id);
        assert_eq!(draft.provider, "fake");
        assert_eq!(draft.sync_status, SyncStatus::Pending);
        assert!(draft.external_event_id.is_none());
    }

    fn entry() -> WorkEntry {
        let started_at = Utc.with_ymd_and_hms(2026, 6, 16, 9, 0, 0).single().unwrap();
        WorkEntry {
            id: 1,
            uid: "entry-1".to_string(),
            calendar_id: 1,
            title: "Confirmed work".to_string(),
            body: None,
            raw_input: None,
            project: None,
            status: Status::Confirmed,
            actor: Actor::Human,
            source: Source::new("manual").unwrap(),
            started_at,
            ended_at: started_at + Duration::hours(1),
            tags: Vec::new(),
            evidence: Vec::new(),
            created_at: started_at,
            updated_at: started_at,
        }
    }
}
