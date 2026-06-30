use chrono::{TimeZone, Utc};

use std::path::Path;

use crate::commands::{
    archive_work_entry_for_state, confirm_work_entry_for_state, create_work_calendar_for_state,
    create_work_entry_for_state, delete_work_entry_for_state, export_report_source_for_state,
    get_settings_for_state, list_work_calendars_for_state, list_work_entries_for_state,
    save_filters_for_state, save_settings_for_state, update_work_entry_for_state, EntryFilterDto,
    ReportFormatDto,
};
use crate::state::{AppState, DesktopSettings, EntryFiltersSetting};
use worklog_core::{Actor, NewWorkCalendar, NewWorkEntry, Source, Status, WorkEntryPatch};

#[test]
fn desktop_commands_use_core_for_entry_calendar_and_report_operations() {
    let state = AppState::in_memory().unwrap();
    let start = Utc.with_ymd_and_hms(2026, 6, 16, 9, 0, 0).single().unwrap();
    let end = Utc
        .with_ymd_and_hms(2026, 6, 16, 10, 0, 0)
        .single()
        .unwrap();

    let calendar = create_work_calendar_for_state(
        &state,
        NewWorkCalendar {
            name: "Desktop".to_string(),
            color: Some("#3366ff".to_string()),
            is_default: true,
        },
    )
    .unwrap();
    assert_eq!(list_work_calendars_for_state(&state).unwrap().len(), 1);

    let entry = create_work_entry_for_state(
        &state,
        NewWorkEntry {
            title: "Desktop draft".to_string(),
            body: None,
            raw_input: None,
            project: Some("worklog".to_string()),
            status: None,
            actor: Actor::Ai,
            source: Source::new("codex").unwrap(),
            started_at: start,
            ended_at: end,
            tags: vec!["desktop".to_string()],
            evidence: Vec::new(),
            calendar_id: Some(calendar.id),
        },
    )
    .unwrap();
    assert_eq!(entry.status, Status::Draft);

    let drafts = list_work_entries_for_state(
        &state,
        EntryFilterDto {
            status: Some(Status::Draft),
            ..EntryFilterDto::default()
        },
    )
    .unwrap();
    assert_eq!(drafts.len(), 1);

    let updated = update_work_entry_for_state(
        &state,
        entry.id,
        WorkEntryPatch {
            title: Some("Desktop draft edited".to_string()),
            ..WorkEntryPatch::default()
        },
    )
    .unwrap();
    assert_eq!(updated.title, "Desktop draft edited");

    let confirmed = confirm_work_entry_for_state(&state, entry.id).unwrap();
    assert_eq!(confirmed.status, Status::Confirmed);

    let exported = export_report_source_for_state(
        &state,
        start - chrono::Duration::hours(1),
        end + chrono::Duration::hours(1),
        ReportFormatDto::Markdown,
    )
    .unwrap();
    assert_eq!(exported, "1. Desktop draft edited\n");

    let archived = archive_work_entry_for_state(&state, entry.id).unwrap();
    assert_eq!(archived.status, Status::Archived);

    assert_eq!(delete_work_entry_for_state(&state, entry.id).unwrap(), 1);
}

#[test]
fn desktop_shell_registers_stable_command_names_for_frontend() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = std::fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();

    for command in [
        "commands::list_work_entries",
        "commands::create_work_entry",
        "commands::update_work_entry",
        "commands::confirm_work_entry",
        "commands::archive_work_entry",
        "commands::delete_work_entry",
        "commands::list_work_calendars",
        "commands::create_work_calendar",
        "commands::export_report_source",
        "commands::get_settings",
        "commands::save_settings",
        "commands::save_filters",
    ] {
        assert!(
            lib.contains(command),
            "Tauri invoke handler should register {command}"
        );
    }
}

#[test]
fn desktop_release_binary_uses_windows_gui_subsystem() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let main = std::fs::read_to_string(manifest_dir.join("src/main.rs")).unwrap();

    assert!(
        main.contains(r#"windows_subsystem = "windows""#),
        "Windows release desktop binary should use the GUI subsystem so it does not open a console"
    );
    assert!(
        main.contains("not(debug_assertions)"),
        "the GUI subsystem setting should be limited to release builds"
    );
}

#[test]
fn desktop_settings_persist_filters_to_json() {
    let db_path = temp_path("settings-filters.db");
    let settings_path = temp_path("settings-filters.json");

    let state =
        AppState::open_at_with_settings_path(db_path.clone(), settings_path.clone()).unwrap();
    let saved = save_filters_for_state(
        &state,
        EntryFiltersSetting {
            status: Some("draft".to_string()),
            calendar_id: Some(42),
            project: Some("worklog".to_string()),
            actor: Some("ai".to_string()),
            source: Some("codex".to_string()),
        },
    )
    .unwrap();
    assert_eq!(saved.current_filters.project.as_deref(), Some("worklog"));

    let reopened = AppState::open_at_with_settings_path(db_path, settings_path).unwrap();
    let settings = get_settings_for_state(&reopened);
    assert_eq!(settings.current_filters.status.as_deref(), Some("draft"));
    assert_eq!(settings.current_filters.calendar_id, Some(42));
    assert_eq!(settings.current_filters.project.as_deref(), Some("worklog"));
    assert_eq!(settings.current_filters.actor.as_deref(), Some("ai"));
    assert_eq!(settings.current_filters.source.as_deref(), Some("codex"));
}

#[test]
fn desktop_settings_reopen_storage_when_db_path_changes() {
    let first_db = temp_path("settings-first.db");
    let second_db = temp_path("settings-second.db");
    let settings_path = temp_path("settings-switch.json");
    let start = Utc.with_ymd_and_hms(2026, 6, 16, 9, 0, 0).single().unwrap();

    let state =
        AppState::open_at_with_settings_path(first_db.clone(), settings_path.clone()).unwrap();
    create_work_entry_for_state(
        &state,
        NewWorkEntry {
            title: "Entry in first db".to_string(),
            body: None,
            raw_input: None,
            project: None,
            status: Some(Status::Confirmed),
            actor: Actor::Human,
            source: Source::new("desktop").unwrap(),
            started_at: start,
            ended_at: start + chrono::Duration::hours(1),
            tags: Vec::new(),
            evidence: Vec::new(),
            calendar_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        list_work_entries_for_state(&state, EntryFilterDto::default())
            .unwrap()
            .len(),
        1
    );

    let settings = save_settings_for_state(
        &state,
        DesktopSettings {
            db_path: Some(second_db.clone()),
            current_filters: EntryFiltersSetting {
                status: Some("confirmed".to_string()),
                ..EntryFiltersSetting::default()
            },
        },
    )
    .unwrap();

    assert_eq!(settings.db_path.as_deref(), Some(second_db.as_path()));
    assert_eq!(
        list_work_entries_for_state(&state, EntryFilterDto::default())
            .unwrap()
            .len(),
        0,
        "after switching db path, the active connection should point at the new database"
    );

    let reopened = AppState::open_at_with_settings_path(first_db, settings_path).unwrap();
    assert_eq!(
        get_settings_for_state(&reopened).db_path.as_deref(),
        Some(second_db.as_path())
    );
}

fn temp_path(file_name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "worklog-desktop-{}-{file_name}",
        uuid::Uuid::new_v4()
    ))
}
