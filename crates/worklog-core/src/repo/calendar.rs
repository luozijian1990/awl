use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{Result, WorklogError};
use crate::model::{NewWorkCalendar, WorkCalendar, WorkCalendarPatch};

use super::codec::{encode_datetime, map_work_calendar, new_uid};

const DEFAULT_CALENDAR_NAME: &str = "Work";

/// 不存在默认本地工作日历时创建；始终返回当前默认日历。
pub fn ensure_default_calendar(conn: &Connection) -> Result<WorkCalendar> {
    if let Some(calendar) = get_default_calendar(conn)? {
        return Ok(calendar);
    }

    let now = Utc::now();
    conn.execute(
        "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at)
         VALUES (?1, ?2, 1, ?3, ?4)",
        params![
            new_uid(),
            DEFAULT_CALENDAR_NAME,
            encode_datetime(now),
            encode_datetime(now)
        ],
    )?;

    get_default_calendar(conn)?
        .ok_or_else(|| WorklogError::NotFound("default calendar after insert".to_string()))
}

/// 创建工作日历。若 `is_default=true`，会在同一事务里清掉旧默认项。
pub fn create_work_calendar(conn: &Connection, input: NewWorkCalendar) -> Result<WorkCalendar> {
    validate_calendar_name(&input.name)?;
    let tx = conn.unchecked_transaction()?;
    let now = Utc::now();

    if input.is_default {
        tx.execute("UPDATE work_calendars SET is_default = 0", [])?;
    }

    tx.execute(
        "INSERT INTO work_calendars (uid, name, color, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_uid(),
            input.name.trim(),
            input.color,
            input.is_default,
            encode_datetime(now),
            encode_datetime(now),
        ],
    )?;
    let id = tx.last_insert_rowid();
    tx.commit()?;

    get_work_calendar(conn, id)?
        .ok_or_else(|| WorklogError::NotFound(format!("work calendar {id}")))
}

/// 局部更新工作日历（改名 / 颜色）。只写入 `Some` 字段，刷新 `updated_at`。
pub fn update_work_calendar(
    conn: &Connection,
    id: i64,
    patch: WorkCalendarPatch,
) -> Result<WorkCalendar> {
    if get_work_calendar(conn, id)?.is_none() {
        return Err(WorklogError::NotFound(format!("work calendar {id}")));
    }

    let now = encode_datetime(Utc::now());
    if let Some(name) = patch.name.as_deref() {
        validate_calendar_name(name)?;
        conn.execute(
            "UPDATE work_calendars SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name.trim(), now, id],
        )?;
    }
    if let Some(color) = patch.color.as_deref() {
        conn.execute(
            "UPDATE work_calendars SET color = ?1, updated_at = ?2 WHERE id = ?3",
            params![color, now, id],
        )?;
    }

    get_work_calendar(conn, id)?
        .ok_or_else(|| WorklogError::NotFound(format!("work calendar {id}")))
}

/// 删除工作日历。默认日历不可删除；级联删除组内 work_entries（其 projections 经 FK
/// `ON DELETE CASCADE` 自动清理）。返回删除的日历行数（成功为 1）。
pub fn delete_work_calendar(conn: &Connection, id: i64) -> Result<usize> {
    let calendar = get_work_calendar(conn, id)?
        .ok_or_else(|| WorklogError::NotFound(format!("work calendar {id}")))?;
    if calendar.is_default {
        return Err(WorklogError::Invalid(
            "cannot delete the default calendar".to_string(),
        ));
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM work_entries WHERE calendar_id = ?1",
        params![id],
    )?;
    let removed = tx.execute("DELETE FROM work_calendars WHERE id = ?1", params![id])?;
    tx.commit()?;
    Ok(removed)
}

/// 按创建顺序列出工作日历。
pub fn list_work_calendars(conn: &Connection) -> Result<Vec<WorkCalendar>> {
    let mut stmt = conn.prepare(
        "SELECT id, uid, name, color, is_default, created_at, updated_at
         FROM work_calendars
         ORDER BY id ASC",
    )?;
    let calendars = stmt
        .query_map([], map_work_calendar)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(calendars)
}

/// 查询单个工作日历。
pub fn get_work_calendar(conn: &Connection, id: i64) -> Result<Option<WorkCalendar>> {
    conn.query_row(
        "SELECT id, uid, name, color, is_default, created_at, updated_at
         FROM work_calendars
         WHERE id = ?1",
        params![id],
        map_work_calendar,
    )
    .optional()
    .map_err(Into::into)
}

/// 返回当前默认工作日历。
pub fn get_default_calendar(conn: &Connection) -> Result<Option<WorkCalendar>> {
    conn.query_row(
        "SELECT id, uid, name, color, is_default, created_at, updated_at
         FROM work_calendars
         WHERE is_default = 1
         ORDER BY id
         LIMIT 1",
        [],
        map_work_calendar,
    )
    .optional()
    .map_err(Into::into)
}

/// 将指定工作日历设为默认，并清掉旧默认项。
pub fn set_default_calendar(conn: &Connection, id: i64) -> Result<WorkCalendar> {
    if get_work_calendar(conn, id)?.is_none() {
        return Err(WorklogError::NotFound(format!("work calendar {id}")));
    }

    let tx = conn.unchecked_transaction()?;
    let now = Utc::now();
    tx.execute("UPDATE work_calendars SET is_default = 0", [])?;
    tx.execute(
        "UPDATE work_calendars
         SET is_default = 1, updated_at = ?1
         WHERE id = ?2",
        params![encode_datetime(now), id],
    )?;
    tx.commit()?;

    get_work_calendar(conn, id)?
        .ok_or_else(|| WorklogError::NotFound(format!("work calendar {id}")))
}

fn validate_calendar_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(WorklogError::Invalid("name must not be empty".to_string()));
    }
    Ok(())
}
