//! SQLite 路径解析、连接打开与迁移。
//!
//! 路径优先级：环境变量 `WORKLOG_DB` → OS app data 目录。绝不写死个人路径。

use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use rusqlite::Connection;

use crate::error::{Result, WorklogError};

/// 指向 SQLite 文件的环境变量。
pub const ENV_DB_PATH: &str = "WORKLOG_DB";

/// 内嵌迁移。语句均为幂等（`IF NOT EXISTS`），可在每次启动时执行。
const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");

/// 解析 SQLite 文件路径。
///
/// 优先使用 `WORKLOG_DB`；否则落到各 OS 的 app data 目录下的 `worklog.db`
/// （macOS `~/Library/Application Support/worklog/`、Linux `~/.local/share/worklog/`、
/// Windows `%APPDATA%\worklog\data\`）。
pub fn resolve_db_path() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os(ENV_DB_PATH) {
        return Ok(PathBuf::from(value));
    }
    let dirs = ProjectDirs::from("", "", "worklog").ok_or(WorklogError::NoDataDir)?;
    Ok(dirs.data_dir().join("worklog.db"))
}

/// 用 [`resolve_db_path`] 解析出的路径打开数据库。
pub fn open() -> Result<Connection> {
    open_at(resolve_db_path()?)
}

/// 在指定路径打开数据库：建父目录 → 开外键 → 幂等执行迁移。
pub fn open_at(path: impl AsRef<Path>) -> Result<Connection> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", true)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// 幂等执行迁移。
pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(MIGRATION_0001)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内存库 + 迁移，给约束类测试用。
    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn migrations_create_expected_tables_and_are_idempotent() {
        let conn = mem();
        // 再跑一次证明幂等。
        run_migrations(&conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        for expected in ["calendar_projections", "work_calendars", "work_entries"] {
            assert!(
                tables.contains(&expected.to_string()),
                "missing table {expected}"
            );
        }
    }

    #[test]
    fn foreign_keys_are_enabled() {
        let conn = mem();
        let enabled: bool = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert!(enabled);
    }

    #[test]
    fn at_most_one_default_calendar() {
        let conn = mem();
        let now = "2026-06-16T00:00:00+00:00";
        conn.execute(
            "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["cal-1", "Cal 1", 1, now, now],
        )
        .unwrap();

        let second = conn.execute(
            "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["cal-2", "Cal 2", 1, now, now],
        );
        assert!(
            second.is_err(),
            "a second default calendar must violate the unique index"
        );

        // 非默认日历可以有多个。
        conn.execute(
            "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["cal-3", "Cal 3", 0, now, now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["cal-4", "Cal 4", 0, now, now],
        )
        .unwrap();
    }

    #[test]
    fn work_entries_schema_rejects_invalid_status_actor_and_empty_source() {
        let conn = mem();
        let now = "2026-06-16T00:00:00+00:00";
        conn.execute(
            "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["cal-1", "Cal 1", 1, now, now],
        )
        .unwrap();

        let invalid_status =
            insert_entry_with_status_actor_source(&conn, "doing", "human", "manual");
        assert!(invalid_status.is_err(), "invalid status should be rejected");

        let invalid_actor =
            insert_entry_with_status_actor_source(&conn, "draft", "robot", "manual");
        assert!(invalid_actor.is_err(), "invalid actor should be rejected");

        let empty_source = insert_entry_with_status_actor_source(&conn, "draft", "human", " ");
        assert!(empty_source.is_err(), "empty source should be rejected");
    }

    #[test]
    fn open_at_creates_parent_dirs_and_file() {
        let mut base = std::env::temp_dir();
        base.push(format!("worklog-test-{}", uuid::Uuid::new_v4()));
        let db_path = base.join("nested").join("worklog.db");

        let conn = open_at(&db_path).unwrap();
        assert!(db_path.exists(), "db file should be created");
        run_migrations(&conn).unwrap(); // 幂等
        drop(conn);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn resolve_db_path_prefers_env() {
        std::env::set_var(ENV_DB_PATH, "/tmp/worklog-env-test.db");
        let path = resolve_db_path().unwrap();
        std::env::remove_var(ENV_DB_PATH);
        assert_eq!(path, PathBuf::from("/tmp/worklog-env-test.db"));
    }

    fn insert_entry_with_status_actor_source(
        conn: &Connection,
        status: &str,
        actor: &str,
        source: &str,
    ) -> rusqlite::Result<usize> {
        let now = "2026-06-16T00:00:00+00:00";
        conn.execute(
            "INSERT INTO work_entries (
                uid, calendar_id, title, status, actor, source,
                started_at, ended_at, created_at, updated_at
             )
             VALUES (?1, 1, 'Entry', ?2, ?3, ?4, ?5, ?6, ?5, ?5)",
            rusqlite::params![
                "entry-1",
                status,
                actor,
                source,
                now,
                "2026-06-16T01:00:00+00:00"
            ],
        )
    }
}
