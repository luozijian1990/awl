use std::io::Write;

use serde::Serialize;
use worklog_core::{Result, WorkEntry};

pub(crate) fn write_entry_json<W: Write>(out: &mut W, entry: &WorkEntry) -> Result<()> {
    writeln!(
        out,
        "{}",
        serde_json::to_string_pretty(&CliEntry::from(entry))?
    )
    .map_err(Into::into)
}

#[derive(Debug, Serialize)]
pub(crate) struct EntryListJson {
    pub(crate) entries: Vec<CliEntry>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CliEntry {
    id: i64,
    uid: String,
    calendar_id: i64,
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
    created_at: String,
    updated_at: String,
}

impl From<&WorkEntry> for CliEntry {
    fn from(entry: &WorkEntry) -> Self {
        Self {
            id: entry.id,
            uid: entry.uid.clone(),
            calendar_id: entry.calendar_id,
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
            created_at: entry.created_at.to_rfc3339(),
            updated_at: entry.updated_at.to_rfc3339(),
        }
    }
}
