use std::io::Write;

use worklog_core::{repo, Actor, NewWorkEntry, Result, Source, WorkEntryPatch, WorklogError};

use crate::args::{
    optional_datetime, optional_parse, optional_parse_i64, optional_source, optional_vec,
    parse_datetime, parse_flags, parse_id, parse_or_default, required,
};
use crate::output::{write_entry_json, CliEntry, EntryListJson};

pub(crate) fn run<W: Write>(
    conn: &rusqlite::Connection,
    args: &[String],
    out: &mut W,
) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("add") => add(conn, &args[1..], out),
        Some("edit") => edit(conn, &args[1..], out),
        Some("confirm") => {
            let entry = repo::confirm_work_entry(conn, parse_id(args.get(1), "entry id")?)?;
            write_entry_json(out, &entry)
        }
        Some("archive") => {
            let entry = repo::archive_work_entry(conn, parse_id(args.get(1), "entry id")?)?;
            write_entry_json(out, &entry)
        }
        Some("rm") => {
            let id = parse_id(args.get(1), "entry id")?;
            let deleted = repo::delete_work_entry(conn, id)?;
            writeln!(out, "{deleted}").map_err(Into::into)
        }
        Some("list") => list(conn, &args[1..], out),
        Some(command) => Err(WorklogError::Invalid(format!(
            "unknown entry command: {command}"
        ))),
        None => Err(WorklogError::Invalid("missing entry command".to_string())),
    }
}

fn add<W: Write>(conn: &rusqlite::Connection, args: &[String], out: &mut W) -> Result<()> {
    let flags = parse_flags(args)?;
    let input = NewWorkEntry {
        title: required(&flags, "title")?,
        body: flags.one("body"),
        raw_input: flags.one("raw-input"),
        project: flags.one("project"),
        status: optional_parse(flags.one("status"))?,
        actor: parse_or_default(flags.one("actor"), Actor::Human)?,
        source: Source::new(flags.one("source").unwrap_or_else(|| "cli".to_string()))?,
        started_at: parse_datetime(&required(&flags, "start")?)?,
        ended_at: parse_datetime(&required(&flags, "end")?)?,
        tags: flags.many("tag"),
        evidence: flags.many("evidence"),
        calendar_id: optional_parse_i64(flags.one("calendar"))?,
    };
    let entry = repo::create_work_entry(conn, input)?;
    write_entry_json(out, &entry)
}

fn edit<W: Write>(conn: &rusqlite::Connection, args: &[String], out: &mut W) -> Result<()> {
    let id = parse_id(args.first(), "entry id")?;
    let flags = parse_flags(&args[1..])?;
    let patch = WorkEntryPatch {
        title: flags.one("title"),
        body: flags.one("body"),
        raw_input: flags.one("raw-input"),
        project: flags.one("project"),
        status: optional_parse(flags.one("status"))?,
        actor: optional_parse(flags.one("actor"))?,
        source: optional_source(flags.one("source"))?,
        started_at: optional_datetime(flags.one("start"))?,
        ended_at: optional_datetime(flags.one("end"))?,
        tags: optional_vec(flags.values.get("tag")),
        evidence: optional_vec(flags.values.get("evidence")),
        calendar_id: optional_parse_i64(flags.one("calendar"))?,
    };
    let entry = repo::update_work_entry(conn, id, patch)?;
    write_entry_json(out, &entry)
}

fn list<W: Write>(conn: &rusqlite::Connection, args: &[String], out: &mut W) -> Result<()> {
    let flags = parse_flags(args)?;
    let filter = repo::WorkEntryFilter {
        start: optional_datetime(flags.one("start"))?,
        end: optional_datetime(flags.one("end"))?,
        status: optional_parse(flags.one("status"))?,
        calendar_id: optional_parse_i64(flags.one("calendar"))?,
        project: flags.one("project"),
        actor: optional_parse(flags.one("actor"))?,
        source: flags.one("source"),
    };
    let entries = repo::list_work_entries(conn, filter)?;
    match flags.one("format").as_deref() {
        Some("json") => {
            let payload = EntryListJson {
                entries: entries.iter().map(CliEntry::from).collect(),
            };
            writeln!(out, "{}", serde_json::to_string_pretty(&payload)?).map_err(Into::into)
        }
        Some("table") | None => {
            for entry in entries {
                writeln!(
                    out,
                    "{}\t{}\t{}\t{}\t{}",
                    entry.id,
                    entry.status.as_str(),
                    entry.actor.as_str(),
                    entry.source.as_str(),
                    entry.title
                )?;
            }
            Ok(())
        }
        Some(format) => Err(WorklogError::Invalid(format!(
            "unknown list format: {format}"
        ))),
    }
}
