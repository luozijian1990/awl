use std::io::Write;

use worklog_core::{db, Result, WorklogError};

mod args;
mod commands;
mod output;

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

#[cfg(test)]
mod tests;
