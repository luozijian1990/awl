use chrono::{DateTime, Utc};
use rusqlite::Row;
use uuid::Uuid;

use crate::error::{Result, WorklogError};
use crate::model::{WorkCalendar, WorkEntry};

pub(crate) fn new_uid() -> String {
    Uuid::new_v4().to_string()
}

pub(crate) fn encode_datetime(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(crate) fn encode_string_vec(values: &[String]) -> Result<String> {
    serde_json::to_string(values).map_err(Into::into)
}

pub(crate) fn decode_string_vec_for_row(
    value: String,
    column: usize,
) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(&value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

pub(crate) fn decode_datetime_for_row(
    value: String,
    column: usize,
) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                column,
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

pub(crate) fn map_work_calendar(row: &Row<'_>) -> rusqlite::Result<WorkCalendar> {
    let created_at: String = row.get(5)?;
    let updated_at: String = row.get(6)?;

    Ok(WorkCalendar {
        id: row.get(0)?,
        uid: row.get(1)?,
        name: row.get(2)?,
        color: row.get(3)?,
        is_default: row.get::<_, bool>(4)?,
        created_at: decode_datetime_for_row(created_at, 5)?,
        updated_at: decode_datetime_for_row(updated_at, 6)?,
    })
}

pub(crate) fn map_work_entry(row: &Row<'_>) -> rusqlite::Result<WorkEntry> {
    let status: String = row.get(7)?;
    let actor: String = row.get(8)?;
    let source: String = row.get(9)?;
    let started_at: String = row.get(10)?;
    let ended_at: String = row.get(11)?;
    let tags_json: String = row.get(12)?;
    let evidence_json: String = row.get(13)?;
    let created_at: String = row.get(14)?;
    let updated_at: String = row.get(15)?;

    Ok(WorkEntry {
        id: row.get(0)?,
        uid: row.get(1)?,
        calendar_id: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        raw_input: row.get(5)?,
        project: row.get(6)?,
        status: status.parse().map_err(to_sql_conversion_error(7))?,
        actor: actor.parse().map_err(to_sql_conversion_error(8))?,
        source: source.parse().map_err(to_sql_conversion_error(9))?,
        started_at: decode_datetime_for_row(started_at, 10)?,
        ended_at: decode_datetime_for_row(ended_at, 11)?,
        tags: decode_string_vec_for_row(tags_json, 12)?,
        evidence: decode_string_vec_for_row(evidence_json, 13)?,
        created_at: decode_datetime_for_row(created_at, 14)?,
        updated_at: decode_datetime_for_row(updated_at, 15)?,
    })
}

fn to_sql_conversion_error(column: usize) -> impl FnOnce(WorklogError) -> rusqlite::Error {
    move |err| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    }
}
