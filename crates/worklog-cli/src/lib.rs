use std::io::Write;

use worklog_core::{db, Result, WorklogError};

mod args;
mod commands;
mod output;

const HELP: &str = r#"Usage: worklog [--db <path>] <command> [<args>]

Global options:
  --db <path>      Use a specific SQLite database
  -h, --help       Show this help
  -V, --version    Show version

Commands:
  entry            Add, edit, list, confirm, archive, and remove work entries
  calendar         Manage local calendar groups
  export           Export report source data
  help             Show this help

Common commands:
  worklog entry add --title <text> --start <datetime> --end <datetime> [--actor human|ai] [--source <source>]
  worklog entry list [--status draft|confirmed|archived] [--format table|json]
  worklog entry edit <id> [--title <text>] [--status draft|confirmed|archived]
  worklog entry confirm <id>
  worklog entry archive <id>
  worklog entry rm <id>
  worklog calendar add --name <name> [--color <hex>] [--default]
  worklog calendar list
  worklog calendar default <id>
  worklog export report-source --start <datetime> --end <datetime> --format json|markdown [--output <path>]
"#;

pub fn run_with_args<I, S, W>(args: I, out: &mut W) -> Result<i32>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    W: Write,
{
    let mut args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(WorklogError::Invalid("missing argv[0]".to_string()));
    }
    args.remove(0);

    let db_path = args::extract_global_db_path(&mut args)?;
    if help_requested(&args) {
        write!(out, "{HELP}")?;
        return Ok(0);
    }
    if version_requested(&args) {
        writeln!(out, "worklog {}", env!("CARGO_PKG_VERSION"))?;
        return Ok(0);
    }

    let conn = match db_path {
        Some(path) => db::open_at(path)?,
        None => db::open()?,
    };

    match args.first().map(String::as_str) {
        Some("entry") => commands::entry::run(&conn, &args[1..], out)?,
        Some("calendar") => commands::calendar::run(&conn, &args[1..], out)?,
        Some("export") => commands::export::run(&conn, &args[1..], out)?,
        Some(command) => return Err(WorklogError::Invalid(format!("unknown command: {command}"))),
        None => return Err(WorklogError::Invalid("missing command".to_string())),
    }

    Ok(0)
}

fn help_requested(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("help"))
        || args.iter().any(|arg| arg == "--help" || arg == "-h")
}

fn version_requested(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("version"))
        || args.iter().any(|arg| arg == "--version" || arg == "-V")
}

#[cfg(test)]
mod tests;
