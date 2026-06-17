//! 报表源数据与导出：在 Section 4 实现。
//!
//! 只查询 `confirmed` 工作项，输出 raw numbered Markdown 或稳定字段的 JSON，
//! 不做合并/润色/AI 改写（那是 client 的事）。

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;

use crate::error::Result;
use crate::repo::{self, WorkEntryFilter};
use crate::{Status, WorkEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Markdown,
}

/// 导出报表源数据。只读取 `confirmed` entries。
pub fn report_source(
    conn: &Connection,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    format: ReportFormat,
) -> Result<String> {
    let entries = repo::list_work_entries(
        conn,
        WorkEntryFilter {
            start: Some(start),
            end: Some(end),
            status: Some(Status::Confirmed),
            ..WorkEntryFilter::default()
        },
    )?;

    match format {
        ReportFormat::Json => export_json(start, end, &entries),
        ReportFormat::Markdown => Ok(export_markdown(&entries)),
    }
}

fn export_markdown(entries: &[WorkEntry]) -> String {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| format!("{}. {}\n", index + 1, entry.title))
        .collect()
}

fn export_json(start: DateTime<Utc>, end: DateTime<Utc>, entries: &[WorkEntry]) -> Result<String> {
    let payload = ReportSourceJson {
        start: start.to_rfc3339(),
        end: end.to_rfc3339(),
        items: entries.iter().map(ReportSourceItem::from).collect(),
    };
    serde_json::to_string_pretty(&payload).map_err(Into::into)
}

#[derive(Debug, Serialize)]
struct ReportSourceJson {
    start: String,
    end: String,
    items: Vec<ReportSourceItem>,
}

#[derive(Debug, Serialize)]
struct ReportSourceItem {
    id: i64,
    uid: String,
    title: String,
    body: Option<String>,
    raw_input: Option<String>,
    project: Option<String>,
    status: String,
    actor: String,
    source: String,
    started_at: String,
    ended_at: String,
    tags: Vec<String>,
    evidence: Vec<String>,
    calendar_id: i64,
    created_at: String,
    updated_at: String,
}

impl From<&WorkEntry> for ReportSourceItem {
    fn from(entry: &WorkEntry) -> Self {
        Self {
            id: entry.id,
            uid: entry.uid.clone(),
            title: entry.title.clone(),
            body: entry.body.clone(),
            raw_input: entry.raw_input.clone(),
            project: entry.project.clone(),
            status: entry.status.as_str().to_string(),
            actor: entry.actor.as_str().to_string(),
            source: entry.source.as_str().to_string(),
            started_at: entry.started_at.to_rfc3339(),
            ended_at: entry.ended_at.to_rfc3339(),
            tags: entry.tags.clone(),
            evidence: entry.evidence.clone(),
            calendar_id: entry.calendar_id,
            created_at: entry.created_at.to_rfc3339(),
            updated_at: entry.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use rusqlite::Connection;
    use serde_json::Value;

    use crate::db;
    use crate::repo;
    use crate::{Actor, NewWorkEntry, Source, Status};

    #[test]
    fn report_source_json_includes_only_confirmed_entries_with_stable_fields() {
        let conn = mem();
        let start = dt(2026, 6, 16, 9, 0);
        seed_entries(&conn, start);

        let output = super::report_source(
            &conn,
            start - Duration::minutes(1),
            start + Duration::hours(3),
            super::ReportFormat::Json,
        )
        .unwrap();
        let value: Value = serde_json::from_str(&output).unwrap();
        let items = value["items"].as_array().unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["title"], "Alpha confirmed");
        assert_eq!(items[0]["project"], "worklog");
        assert_eq!(items[0]["status"], "confirmed");
        assert_eq!(items[0]["actor"], "human");
        assert_eq!(items[0]["source"], "manual");
        assert_eq!(items[0]["tags"], serde_json::json!(["repo"]));
        assert_eq!(items[0]["evidence"], serde_json::json!(["ticket-1"]));
        assert_eq!(items[1]["title"], "Beta confirmed");
        assert!(items[0].get("started_at").is_some());
        assert!(items[0].get("ended_at").is_some());
    }

    #[test]
    fn report_source_markdown_is_raw_numbered_titles_only() {
        let conn = mem();
        let start = dt(2026, 6, 16, 9, 0);
        seed_entries(&conn, start);

        let output = super::report_source(
            &conn,
            start - Duration::minutes(1),
            start + Duration::hours(3),
            super::ReportFormat::Markdown,
        )
        .unwrap();

        assert_eq!(output, "1. Alpha confirmed\n2. Beta confirmed\n");
    }

    fn seed_entries(conn: &Connection, start: chrono::DateTime<Utc>) {
        repo::create_work_entry(
            conn,
            NewWorkEntry {
                title: "Alpha confirmed".to_string(),
                project: Some("worklog".to_string()),
                status: Some(Status::Confirmed),
                actor: Actor::Human,
                source: Source::new("manual").unwrap(),
                started_at: start,
                ended_at: start + Duration::hours(1),
                tags: vec!["repo".to_string()],
                evidence: vec!["ticket-1".to_string()],
                ..new_entry("Alpha confirmed", start)
            },
        )
        .unwrap();
        repo::create_work_entry(
            conn,
            NewWorkEntry {
                title: "Draft ignored".to_string(),
                status: Some(Status::Draft),
                actor: Actor::Ai,
                source: Source::new("codex").unwrap(),
                started_at: start + Duration::minutes(30),
                ended_at: start + Duration::hours(1),
                ..new_entry("Draft ignored", start)
            },
        )
        .unwrap();
        repo::create_work_entry(
            conn,
            NewWorkEntry {
                title: "Beta confirmed".to_string(),
                status: Some(Status::Confirmed),
                actor: Actor::Human,
                source: Source::new("manual").unwrap(),
                started_at: start + Duration::hours(2),
                ended_at: start + Duration::hours(3),
                ..new_entry("Beta confirmed", start)
            },
        )
        .unwrap();
    }

    fn new_entry(title: &str, start: chrono::DateTime<Utc>) -> NewWorkEntry {
        NewWorkEntry {
            title: title.to_string(),
            body: None,
            raw_input: None,
            project: None,
            status: None,
            actor: Actor::Human,
            source: Source::new("manual").unwrap(),
            started_at: start,
            ended_at: start + Duration::hours(1),
            tags: Vec::new(),
            evidence: Vec::new(),
            calendar_id: None,
        }
    }

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        db::run_migrations(&conn).unwrap();
        conn
    }

    fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .unwrap()
    }
}
