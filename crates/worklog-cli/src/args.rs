use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use worklog_core::{Result, Source, WorklogError};

#[derive(Debug, Default)]
pub(crate) struct Flags {
    pub(crate) values: HashMap<String, Vec<String>>,
    switches: Vec<String>,
}

impl Flags {
    pub(crate) fn one(&self, key: &str) -> Option<String> {
        self.values
            .get(key)
            .and_then(|values| values.last())
            .cloned()
    }

    pub(crate) fn many(&self, key: &str) -> Vec<String> {
        self.values.get(key).cloned().unwrap_or_default()
    }

    pub(crate) fn has(&self, key: &str) -> bool {
        self.switches.iter().any(|switch| switch == key)
    }
}

pub(crate) fn extract_global_db_path(args: &mut Vec<String>) -> Result<Option<PathBuf>> {
    let mut db_path = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--db" => {
                db_path = Some(PathBuf::from(take_value(args, index, "--db")?));
                args.drain(index..=index + 1);
            }
            _ => index += 1,
        }
    }
    Ok(db_path)
}

pub(crate) fn parse_flags(args: &[String]) -> Result<Flags> {
    let mut flags = Flags::default();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !arg.starts_with("--") {
            return Err(WorklogError::Invalid(format!(
                "unexpected positional arg: {arg}"
            )));
        }
        let key = arg.trim_start_matches("--").to_string();
        if key == "default" {
            flags.switches.push(key);
            index += 1;
            continue;
        }
        let value = take_value(args, index, arg)?;
        flags.values.entry(key).or_default().push(value);
        index += 2;
    }
    Ok(flags)
}

pub(crate) fn take_value(args: &[String], index: usize, flag: &str) -> Result<String> {
    args.get(index + 1)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| WorklogError::Invalid(format!("missing value for {flag}")))
}

pub(crate) fn required(flags: &Flags, key: &str) -> Result<String> {
    flags
        .one(key)
        .ok_or_else(|| WorklogError::Invalid(format!("missing --{key}")))
}

pub(crate) fn parse_id(value: Option<&String>, label: &str) -> Result<i64> {
    value
        .ok_or_else(|| WorklogError::Invalid(format!("missing {label}")))?
        .parse::<i64>()
        .map_err(|_| WorklogError::Invalid(format!("invalid {label}")))
}

pub(crate) fn parse_datetime(value: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M")
        .map(|dt| Utc.from_utc_datetime(&dt))
        .map_err(|_| WorklogError::Invalid(format!("invalid datetime: {value}")))
}

pub(crate) fn optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value.as_deref().map(parse_datetime).transpose()
}

pub(crate) fn optional_parse<T>(value: Option<String>) -> Result<Option<T>>
where
    T: FromStr<Err = WorklogError>,
{
    value.as_deref().map(str::parse).transpose()
}

pub(crate) fn parse_or_default<T>(value: Option<String>, default: T) -> Result<T>
where
    T: FromStr<Err = WorklogError>,
{
    value.map_or(Ok(default), |value| value.parse())
}

pub(crate) fn optional_parse_i64(value: Option<String>) -> Result<Option<i64>> {
    value
        .map(|value| {
            value
                .parse::<i64>()
                .map_err(|_| WorklogError::Invalid(format!("invalid integer: {value}")))
        })
        .transpose()
}

pub(crate) fn optional_source(value: Option<String>) -> Result<Option<Source>> {
    value.map(Source::new).transpose()
}

pub(crate) fn optional_vec(values: Option<&Vec<String>>) -> Option<Vec<String>> {
    values.cloned()
}
