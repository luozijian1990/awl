use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{Result, WorklogError};
use crate::model::{Actor, NewWorkEntry, Status, WorkEntry, WorkEntryPatch};

use super::calendar::ensure_default_calendar;
use super::codec::{encode_datetime, encode_string_vec, map_work_entry, new_uid};

/// `list_work_entries` 的过滤条件。时间范围使用半开区间：`started_at >= start`
/// 且 `started_at < end`。
#[derive(Debug, Clone, Default)]
pub struct WorkEntryFilter {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub status: Option<Status>,
    pub calendar_id: Option<i64>,
    pub project: Option<String>,
    pub actor: Option<Actor>,
    pub source: Option<String>,
}

/// 创建工作项：校验输入、补齐默认 status/calendar，insert 后返回完整 entry。
pub fn create_work_entry(conn: &Connection, input: NewWorkEntry) -> Result<WorkEntry> {
    validate_title(&input.title)?;
    validate_time_range(input.started_at, input.ended_at)?;

    let calendar_id = match input.calendar_id {
        Some(calendar_id) => calendar_id,
        None => ensure_default_calendar(conn)?.id,
    };
    let status = input
        .status
        .unwrap_or_else(|| default_status(input.actor, input.source.as_str()));
    let now = Utc::now();

    conn.execute(
        "INSERT INTO work_entries (
            uid, calendar_id, title, body, raw_input, project, status, actor, source,
            started_at, ended_at, tags_json, evidence_json, created_at, updated_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            new_uid(),
            calendar_id,
            input.title.trim(),
            input.body,
            input.raw_input,
            input.project,
            status.as_str(),
            input.actor.as_str(),
            input.source.as_str(),
            encode_datetime(input.started_at),
            encode_datetime(input.ended_at),
            encode_string_vec(&input.tags)?,
            encode_string_vec(&input.evidence)?,
            encode_datetime(now),
            encode_datetime(now),
        ],
    )?;

    get_work_entry(conn, conn.last_insert_rowid())?
        .ok_or_else(|| WorklogError::NotFound("created work entry".to_string()))
}

/// 局部更新工作项：不存在时报错，只更新传入字段，并刷新 `updated_at`。
pub fn update_work_entry(conn: &Connection, id: i64, patch: WorkEntryPatch) -> Result<WorkEntry> {
    let existing = require_work_entry(conn, id)?;

    let title = patch.title.unwrap_or(existing.title);
    validate_title(&title)?;

    let started_at = patch.started_at.unwrap_or(existing.started_at);
    let ended_at = patch.ended_at.unwrap_or(existing.ended_at);
    validate_time_range(started_at, ended_at)?;

    let body = patch.body.or(existing.body);
    let raw_input = patch.raw_input.or(existing.raw_input);
    let project = patch.project.or(existing.project);
    let status = patch.status.unwrap_or(existing.status);
    let actor = patch.actor.unwrap_or(existing.actor);
    let source = patch.source.unwrap_or(existing.source);
    let tags = patch.tags.unwrap_or(existing.tags);
    let evidence = patch.evidence.unwrap_or(existing.evidence);
    let calendar_id = patch.calendar_id.unwrap_or(existing.calendar_id);
    let updated_at = Utc::now();

    conn.execute(
        "UPDATE work_entries
         SET calendar_id = ?1,
             title = ?2,
             body = ?3,
             raw_input = ?4,
             project = ?5,
             status = ?6,
             actor = ?7,
             source = ?8,
             started_at = ?9,
             ended_at = ?10,
             tags_json = ?11,
             evidence_json = ?12,
             updated_at = ?13
         WHERE id = ?14",
        params![
            calendar_id,
            title.trim(),
            body,
            raw_input,
            project,
            status.as_str(),
            actor.as_str(),
            source.as_str(),
            encode_datetime(started_at),
            encode_datetime(ended_at),
            encode_string_vec(&tags)?,
            encode_string_vec(&evidence)?,
            encode_datetime(updated_at),
            id,
        ],
    )?;

    require_work_entry(conn, id)
}

/// 将工作项切换为 confirmed；报表层只会读取 confirmed entries。
pub fn confirm_work_entry(conn: &Connection, id: i64) -> Result<WorkEntry> {
    update_work_entry(
        conn,
        id,
        WorkEntryPatch {
            status: Some(Status::Confirmed),
            ..WorkEntryPatch::default()
        },
    )
}

/// 将工作项切换为 archived。
pub fn archive_work_entry(conn: &Connection, id: i64) -> Result<WorkEntry> {
    update_work_entry(
        conn,
        id,
        WorkEntryPatch {
            status: Some(Status::Archived),
            ..WorkEntryPatch::default()
        },
    )
}

/// 删除工作项。依赖 schema 的 `ON DELETE CASCADE` 清理 projections。
pub fn delete_work_entry(conn: &Connection, id: i64) -> Result<usize> {
    require_work_entry(conn, id)?;
    let deleted = conn.execute("DELETE FROM work_entries WHERE id = ?1", params![id])?;
    Ok(deleted)
}

/// 查询工作项，支持 MVP 过滤条件，并按 `started_at, title` 稳定排序。
pub fn list_work_entries(conn: &Connection, filter: WorkEntryFilter) -> Result<Vec<WorkEntry>> {
    let mut sql = String::from(
        "SELECT id, uid, calendar_id, title, body, raw_input, project, status, actor, source,
                started_at, ended_at, tags_json, evidence_json, created_at, updated_at
         FROM work_entries
         WHERE 1 = 1",
    );
    let mut values: Vec<String> = Vec::new();

    if let Some(start) = filter.start {
        sql.push_str(" AND started_at >= ?");
        values.push(encode_datetime(start));
    }
    if let Some(end) = filter.end {
        sql.push_str(" AND started_at < ?");
        values.push(encode_datetime(end));
    }
    if let Some(status) = filter.status {
        sql.push_str(" AND status = ?");
        values.push(status.as_str().to_string());
    }
    if let Some(calendar_id) = filter.calendar_id {
        sql.push_str(" AND calendar_id = ?");
        values.push(calendar_id.to_string());
    }
    if let Some(project) = filter.project {
        sql.push_str(" AND project = ?");
        values.push(project);
    }
    if let Some(actor) = filter.actor {
        sql.push_str(" AND actor = ?");
        values.push(actor.as_str().to_string());
    }
    if let Some(source) = filter.source {
        sql.push_str(" AND source = ?");
        values.push(source);
    }

    sql.push_str(" ORDER BY started_at ASC, title ASC");

    let mut stmt = conn.prepare(&sql)?;
    let entries = stmt
        .query_map(rusqlite::params_from_iter(values), map_work_entry)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(entries)
}

fn get_work_entry(conn: &Connection, id: i64) -> Result<Option<WorkEntry>> {
    conn.query_row(
        "SELECT id, uid, calendar_id, title, body, raw_input, project, status, actor, source,
                started_at, ended_at, tags_json, evidence_json, created_at, updated_at
         FROM work_entries
         WHERE id = ?1",
        params![id],
        map_work_entry,
    )
    .optional()
    .map_err(Into::into)
}

fn require_work_entry(conn: &Connection, id: i64) -> Result<WorkEntry> {
    get_work_entry(conn, id)?.ok_or_else(|| WorklogError::NotFound(format!("work entry {id}")))
}

fn validate_title(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        return Err(WorklogError::Invalid("title must not be empty".to_string()));
    }
    Ok(())
}

fn validate_time_range(started_at: DateTime<Utc>, ended_at: DateTime<Utc>) -> Result<()> {
    if ended_at <= started_at {
        return Err(WorklogError::Invalid(
            "ended_at must be later than started_at".to_string(),
        ));
    }
    Ok(())
}

fn default_status(actor: Actor, source: &str) -> Status {
    if actor == Actor::Ai || is_ai_source(source) {
        Status::Draft
    } else {
        Status::Confirmed
    }
}

fn is_ai_source(source: &str) -> bool {
    matches!(source, "codex" | "claude")
}
