use chrono::{Duration, TimeZone, Utc};
use rusqlite::Connection;

use crate::db;
use crate::{Actor, NewWorkEntry, Source, Status, WorkEntryPatch, WorklogError};

use super::{
    archive_work_entry, confirm_work_entry, create_work_calendar, create_work_entry,
    delete_work_calendar, delete_work_entry, ensure_default_calendar, get_default_calendar,
    get_work_calendar, list_work_calendars, list_work_entries, set_default_calendar,
    update_work_calendar, update_work_entry, WorkEntryFilter,
};

#[test]
fn ensure_default_calendar_creates_one_when_missing() {
    let conn = mem();

    let calendar = ensure_default_calendar(&conn).unwrap();

    assert_eq!(calendar.name, "Work");
    assert!(calendar.is_default);
    assert!(!calendar.uid.is_empty());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM work_calendars", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn ensure_default_calendar_reuses_existing_default() {
    let conn = mem();

    let first = ensure_default_calendar(&conn).unwrap();
    let second = ensure_default_calendar(&conn).unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(first.uid, second.uid);

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_calendars WHERE is_default = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn create_work_entry_defaults_ai_entries_to_draft_and_default_calendar() {
    let conn = mem();
    let started_at = dt(2026, 6, 16, 9, 0);
    let ended_at = started_at + Duration::hours(1);

    let entry = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Implement repository".to_string(),
            body: Some("Core CRUD".to_string()),
            raw_input: Some("raw note".to_string()),
            project: Some("worklog".to_string()),
            status: None,
            actor: Actor::Ai,
            source: Source::new("codex").unwrap(),
            started_at,
            ended_at,
            tags: vec!["rust".to_string(), "sqlite".to_string()],
            evidence: vec!["docs/plans/tasks.md".to_string()],
            calendar_id: None,
        },
    )
    .unwrap();

    assert_eq!(entry.title, "Implement repository");
    assert_eq!(entry.status, Status::Draft);
    assert_eq!(entry.actor, Actor::Ai);
    assert_eq!(entry.source.as_str(), "codex");
    assert_eq!(entry.started_at, started_at);
    assert_eq!(entry.ended_at, ended_at);
    assert_eq!(entry.tags, vec!["rust", "sqlite"]);
    assert_eq!(entry.evidence, vec!["docs/plans/tasks.md"]);
    assert!(!entry.uid.is_empty());

    let calendar = ensure_default_calendar(&conn).unwrap();
    assert_eq!(entry.calendar_id, calendar.id);
}

#[test]
fn create_work_entry_defaults_human_entries_to_confirmed() {
    let conn = mem();
    let entry = create_work_entry(&conn, new_entry("Manual note", Actor::Human)).unwrap();

    assert_eq!(entry.status, Status::Confirmed);
}

#[test]
fn create_work_entry_rejects_empty_title_and_non_positive_time_range() {
    let conn = mem();

    let err = create_work_entry(&conn, new_entry("  ", Actor::Human)).unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("title")));

    let started_at = dt(2026, 6, 16, 9, 0);
    let err = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Bad time".to_string(),
            started_at,
            ended_at: started_at,
            ..new_entry("Bad time", Actor::Human)
        },
    )
    .unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("ended_at")));
}

#[test]
fn update_work_entry_updates_only_patch_fields_and_refreshes_updated_at() {
    let conn = mem();
    let created = create_work_entry(&conn, new_entry("Original", Actor::Human)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));

    let updated = update_work_entry(
        &conn,
        created.id,
        WorkEntryPatch {
            title: Some("Updated title".to_string()),
            project: Some("worklog".to_string()),
            tags: Some(vec!["repo".to_string()]),
            ..WorkEntryPatch::default()
        },
    )
    .unwrap();

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "Updated title");
    assert_eq!(updated.project.as_deref(), Some("worklog"));
    assert_eq!(updated.tags, vec!["repo"]);
    assert_eq!(updated.status, created.status);
    assert_eq!(updated.actor, created.actor);
    assert_eq!(updated.source.as_str(), created.source.as_str());
    assert_eq!(updated.started_at, created.started_at);
    assert_eq!(updated.ended_at, created.ended_at);
    assert!(updated.updated_at > created.updated_at);
}

#[test]
fn update_work_entry_rejects_missing_entry_and_invalid_patch() {
    let conn = mem();

    let err = update_work_entry(
        &conn,
        404,
        WorkEntryPatch {
            title: Some("Missing".to_string()),
            ..WorkEntryPatch::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, WorklogError::NotFound(message) if message.contains("404")));

    let created = create_work_entry(&conn, new_entry("Original", Actor::Human)).unwrap();
    let err = update_work_entry(
        &conn,
        created.id,
        WorkEntryPatch {
            title: Some("".to_string()),
            ..WorkEntryPatch::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("title")));
}

#[test]
fn confirm_and_archive_work_entry_switch_status_and_refresh_updated_at() {
    let conn = mem();
    let created = create_work_entry(&conn, new_entry("Draft", Actor::Ai)).unwrap();
    assert_eq!(created.status, Status::Draft);
    std::thread::sleep(std::time::Duration::from_millis(2));

    let confirmed = confirm_work_entry(&conn, created.id).unwrap();
    assert_eq!(confirmed.status, Status::Confirmed);
    assert!(confirmed.updated_at > created.updated_at);
    std::thread::sleep(std::time::Duration::from_millis(2));

    let archived = archive_work_entry(&conn, created.id).unwrap();
    assert_eq!(archived.status, Status::Archived);
    assert!(archived.updated_at > confirmed.updated_at);
}

#[test]
fn confirm_and_archive_missing_entry_return_not_found() {
    let conn = mem();

    let err = confirm_work_entry(&conn, 404).unwrap_err();
    assert!(matches!(err, WorklogError::NotFound(message) if message.contains("404")));

    let err = archive_work_entry(&conn, 405).unwrap_err();
    assert!(matches!(err, WorklogError::NotFound(message) if message.contains("405")));
}

#[test]
fn delete_work_entry_removes_entry_and_dependent_projections() {
    let conn = mem();
    let entry = create_work_entry(&conn, new_entry("Delete me", Actor::Human)).unwrap();
    conn.execute(
        "INSERT INTO calendar_projections (
            work_entry_id, provider, sync_status, created_at, updated_at
         )
         VALUES (?1, 'test', 'pending', ?2, ?2)",
        rusqlite::params![entry.id, entry.created_at.to_rfc3339()],
    )
    .unwrap();

    let deleted = delete_work_entry(&conn, entry.id).unwrap();

    assert_eq!(deleted, 1);
    let entry_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM work_entries WHERE id = ?1",
            [entry.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(entry_count, 0);
    let projection_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM calendar_projections WHERE work_entry_id = ?1",
            [entry.id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(projection_count, 0);
}

#[test]
fn delete_work_entry_missing_entry_returns_not_found() {
    let conn = mem();

    let err = delete_work_entry(&conn, 404).unwrap_err();
    assert!(matches!(err, WorklogError::NotFound(message) if message.contains("404")));
}

#[test]
fn list_work_entries_filters_and_sorts_by_started_at_then_title() {
    let conn = mem();
    let default_calendar = ensure_default_calendar(&conn).unwrap();
    let other_calendar_id = insert_calendar_fixture(&conn, "Other", false);

    let base = dt(2026, 6, 16, 9, 0);
    let alpha = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Alpha".to_string(),
            project: Some("worklog".to_string()),
            status: Some(Status::Confirmed),
            actor: Actor::Human,
            source: Source::new("manual").unwrap(),
            started_at: base,
            ended_at: base + Duration::hours(1),
            calendar_id: Some(default_calendar.id),
            ..new_entry("Alpha", Actor::Human)
        },
    )
    .unwrap();
    let beta = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Beta".to_string(),
            project: Some("worklog".to_string()),
            status: Some(Status::Confirmed),
            actor: Actor::Human,
            source: Source::new("manual").unwrap(),
            started_at: base,
            ended_at: base + Duration::hours(1),
            calendar_id: Some(default_calendar.id),
            ..new_entry("Beta", Actor::Human)
        },
    )
    .unwrap();
    let _draft = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Draft".to_string(),
            project: Some("worklog".to_string()),
            status: Some(Status::Draft),
            actor: Actor::Ai,
            source: Source::new("codex").unwrap(),
            started_at: base + Duration::hours(2),
            ended_at: base + Duration::hours(3),
            calendar_id: Some(default_calendar.id),
            ..new_entry("Draft", Actor::Ai)
        },
    )
    .unwrap();
    let _other_calendar = create_work_entry(
        &conn,
        NewWorkEntry {
            title: "Other calendar".to_string(),
            project: Some("ops".to_string()),
            status: Some(Status::Confirmed),
            actor: Actor::Human,
            source: Source::new("manual").unwrap(),
            started_at: base + Duration::hours(4),
            ended_at: base + Duration::hours(5),
            calendar_id: Some(other_calendar_id),
            ..new_entry("Other calendar", Actor::Human)
        },
    )
    .unwrap();

    let entries = list_work_entries(
        &conn,
        WorkEntryFilter {
            start: Some(base - Duration::minutes(1)),
            end: Some(base + Duration::hours(2)),
            status: Some(Status::Confirmed),
            calendar_id: Some(default_calendar.id),
            project: Some("worklog".to_string()),
            actor: Some(Actor::Human),
            source: Some("manual".to_string()),
        },
    )
    .unwrap();

    assert_eq!(
        entries.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        vec![alpha.id, beta.id]
    );
}

#[test]
fn work_calendar_crud_creates_lists_gets_and_sets_default() {
    let conn = mem();
    let default = ensure_default_calendar(&conn).unwrap();

    let project = create_work_calendar(
        &conn,
        crate::NewWorkCalendar {
            name: "Project".to_string(),
            color: Some("#ff0000".to_string()),
            is_default: true,
        },
    )
    .unwrap();

    assert_eq!(project.name, "Project");
    assert_eq!(project.color.as_deref(), Some("#ff0000"));
    assert!(project.is_default);

    let old_default = get_default_calendar(&conn).unwrap().unwrap();
    assert_eq!(old_default.id, project.id);

    let calendars = list_work_calendars(&conn).unwrap();
    assert_eq!(calendars.len(), 2);
    assert_eq!(calendars[0].id, default.id);
    assert!(!calendars[0].is_default);
    assert_eq!(calendars[1].id, project.id);

    let restored = set_default_calendar(&conn, default.id).unwrap();
    assert_eq!(restored.id, default.id);
    assert!(restored.is_default);
    assert!(
        !get_work_calendar(&conn, project.id)
            .unwrap()
            .unwrap()
            .is_default
    );
}

#[test]
fn work_calendar_crud_rejects_empty_name_and_missing_default_target() {
    let conn = mem();

    let err = create_work_calendar(
        &conn,
        crate::NewWorkCalendar {
            name: " ".to_string(),
            color: None,
            is_default: false,
        },
    )
    .unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("name")));

    let err = set_default_calendar(&conn, 404).unwrap_err();
    assert!(matches!(err, WorklogError::NotFound(message) if message.contains("404")));
}

#[test]
fn update_work_calendar_renames_and_recolors() {
    let conn = mem();
    let id = insert_calendar_fixture(&conn, "Old", false);

    let updated = update_work_calendar(
        &conn,
        id,
        crate::WorkCalendarPatch {
            name: Some("New".to_string()),
            color: Some("#00ff00".to_string()),
        },
    )
    .unwrap();
    assert_eq!(updated.name, "New");
    assert_eq!(updated.color.as_deref(), Some("#00ff00"));

    let err = update_work_calendar(
        &conn,
        id,
        crate::WorkCalendarPatch {
            name: Some("  ".to_string()),
            color: None,
        },
    )
    .unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("name")));
}

#[test]
fn delete_work_calendar_cascades_entries_and_protects_default() {
    let conn = mem();
    let default = ensure_default_calendar(&conn).unwrap();
    let group = insert_calendar_fixture(&conn, "Throwaway", false);

    // 组里放两条工作项。
    for title in ["a", "b"] {
        create_work_entry(
            &conn,
            NewWorkEntry {
                calendar_id: Some(group),
                ..new_entry(title, Actor::Human)
            },
        )
        .unwrap();
    }
    assert_eq!(
        list_work_entries(&conn, WorkEntryFilter::default())
            .unwrap()
            .len(),
        2
    );

    // 默认日历不可删。
    let err = delete_work_calendar(&conn, default.id).unwrap_err();
    assert!(matches!(err, WorklogError::Invalid(message) if message.contains("default")));

    // 删非默认组 → 级联删掉组内 entries。
    let removed = delete_work_calendar(&conn, group).unwrap();
    assert_eq!(removed, 1);
    assert!(get_work_calendar(&conn, group).unwrap().is_none());
    assert_eq!(
        list_work_entries(&conn, WorkEntryFilter::default())
            .unwrap()
            .len(),
        0
    );
}

fn mem() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    db::run_migrations(&conn).unwrap();
    conn
}

fn new_entry(title: &str, actor: Actor) -> NewWorkEntry {
    let started_at = dt(2026, 6, 16, 9, 0);
    NewWorkEntry {
        title: title.to_string(),
        body: None,
        raw_input: None,
        project: None,
        status: None,
        actor,
        source: Source::new("manual").unwrap(),
        started_at,
        ended_at: started_at + Duration::hours(1),
        tags: Vec::new(),
        evidence: Vec::new(),
        calendar_id: None,
    }
}

fn insert_calendar_fixture(conn: &Connection, name: &str, is_default: bool) -> i64 {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO work_calendars (uid, name, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        rusqlite::params![uuid::Uuid::new_v4().to_string(), name, is_default, now],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn dt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .unwrap()
}
