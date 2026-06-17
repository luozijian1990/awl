use std::io::Write;

use worklog_core::report::{self, ReportFormat};
use worklog_core::{Result, WorklogError};

use crate::args::{parse_datetime, parse_flags, required};

pub(crate) fn run<W: Write>(
    conn: &rusqlite::Connection,
    args: &[String],
    out: &mut W,
) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("report-source") => {
            let flags = parse_flags(&args[1..])?;
            let start = parse_datetime(&required(&flags, "start")?)?;
            let end = parse_datetime(&required(&flags, "end")?)?;
            let format = match required(&flags, "format")?.as_str() {
                "json" => ReportFormat::Json,
                "markdown" => ReportFormat::Markdown,
                other => {
                    return Err(WorklogError::Invalid(format!(
                        "unknown report format: {other}"
                    )))
                }
            };
            let output = report::report_source(conn, start, end, format)?;
            if let Some(path) = flags.one("output") {
                std::fs::write(path, output)?;
            } else {
                writeln!(out, "{output}")?;
            }
            Ok(())
        }
        Some(command) => Err(WorklogError::Invalid(format!(
            "unknown export command: {command}"
        ))),
        None => Err(WorklogError::Invalid("missing export command".to_string())),
    }
}
