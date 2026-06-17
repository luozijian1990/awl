use std::io::Write;

use worklog_core::{repo, NewWorkCalendar, Result, WorklogError};

use crate::args::{parse_flags, parse_id, required};

pub(crate) fn run<W: Write>(
    conn: &rusqlite::Connection,
    args: &[String],
    out: &mut W,
) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("add") => {
            let flags = parse_flags(&args[1..])?;
            let calendar = repo::create_work_calendar(
                conn,
                NewWorkCalendar {
                    name: required(&flags, "name")?,
                    color: flags.one("color"),
                    is_default: flags.has("default"),
                },
            )?;
            writeln!(out, "{}", serde_json::to_string_pretty(&calendar)?).map_err(Into::into)
        }
        Some("list") => {
            let calendars = repo::list_work_calendars(conn)?;
            writeln!(out, "{}", serde_json::to_string_pretty(&calendars)?).map_err(Into::into)
        }
        Some("default") => {
            let calendar = repo::set_default_calendar(conn, parse_id(args.get(1), "calendar id")?)?;
            writeln!(out, "{}", serde_json::to_string_pretty(&calendar)?).map_err(Into::into)
        }
        Some(command) => Err(WorklogError::Invalid(format!(
            "unknown calendar command: {command}"
        ))),
        None => Err(WorklogError::Invalid(
            "missing calendar command".to_string(),
        )),
    }
}
