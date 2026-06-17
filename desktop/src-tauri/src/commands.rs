use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use worklog_core::report::{self, ReportFormat};
use worklog_core::{
    repo, Actor, NewWorkCalendar, NewWorkEntry, Result, Status, WorkCalendar, WorkCalendarPatch,
    WorkEntry, WorkEntryPatch,
};

use crate::state::{AppState, DesktopSettings, EntryFiltersSetting};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryFilterDto {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub status: Option<Status>,
    pub calendar_id: Option<i64>,
    pub project: Option<String>,
    pub actor: Option<Actor>,
    pub source: Option<String>,
}

impl From<EntryFilterDto> for repo::WorkEntryFilter {
    fn from(filter: EntryFilterDto) -> Self {
        Self {
            start: filter.start,
            end: filter.end,
            status: filter.status,
            calendar_id: filter.calendar_id,
            project: filter.project,
            actor: filter.actor,
            source: filter.source,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormatDto {
    Json,
    Markdown,
}

impl From<ReportFormatDto> for ReportFormat {
    fn from(format: ReportFormatDto) -> Self {
        match format {
            ReportFormatDto::Json => ReportFormat::Json,
            ReportFormatDto::Markdown => ReportFormat::Markdown,
        }
    }
}

pub fn list_work_entries_for_state(
    state: &AppState,
    filter: EntryFilterDto,
) -> Result<Vec<WorkEntry>> {
    state.with_conn(|conn| repo::list_work_entries(conn, filter.into()))
}

pub fn create_work_entry_for_state(state: &AppState, input: NewWorkEntry) -> Result<WorkEntry> {
    state.with_conn(|conn| repo::create_work_entry(conn, input))
}

pub fn update_work_entry_for_state(
    state: &AppState,
    id: i64,
    patch: WorkEntryPatch,
) -> Result<WorkEntry> {
    state.with_conn(|conn| repo::update_work_entry(conn, id, patch))
}

pub fn confirm_work_entry_for_state(state: &AppState, id: i64) -> Result<WorkEntry> {
    state.with_conn(|conn| repo::confirm_work_entry(conn, id))
}

pub fn archive_work_entry_for_state(state: &AppState, id: i64) -> Result<WorkEntry> {
    state.with_conn(|conn| repo::archive_work_entry(conn, id))
}

pub fn delete_work_entry_for_state(state: &AppState, id: i64) -> Result<usize> {
    state.with_conn(|conn| repo::delete_work_entry(conn, id))
}

pub fn list_work_calendars_for_state(state: &AppState) -> Result<Vec<WorkCalendar>> {
    state.with_conn(repo::list_work_calendars)
}

pub fn create_work_calendar_for_state(
    state: &AppState,
    input: NewWorkCalendar,
) -> Result<WorkCalendar> {
    state.with_conn(|conn| repo::create_work_calendar(conn, input))
}

pub fn update_work_calendar_for_state(
    state: &AppState,
    id: i64,
    patch: WorkCalendarPatch,
) -> Result<WorkCalendar> {
    state.with_conn(|conn| repo::update_work_calendar(conn, id, patch))
}

pub fn delete_work_calendar_for_state(state: &AppState, id: i64) -> Result<usize> {
    state.with_conn(|conn| repo::delete_work_calendar(conn, id))
}

pub fn set_default_calendar_for_state(state: &AppState, id: i64) -> Result<WorkCalendar> {
    state.with_conn(|conn| repo::set_default_calendar(conn, id))
}

pub fn export_report_source_for_state(
    state: &AppState,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    format: ReportFormatDto,
) -> Result<String> {
    state.with_conn(|conn| report::report_source(conn, start, end, format.into()))
}

pub fn get_settings_for_state(state: &AppState) -> DesktopSettings {
    state.get_settings()
}

pub fn save_settings_for_state(
    state: &AppState,
    settings: DesktopSettings,
) -> Result<DesktopSettings> {
    state.save_settings(settings)
}

pub fn save_filters_for_state(
    state: &AppState,
    filters: EntryFiltersSetting,
) -> Result<DesktopSettings> {
    state.update_filters(filters)
}

#[tauri::command]
pub fn list_work_entries(
    state: State<'_, AppState>,
    filter: EntryFilterDto,
) -> std::result::Result<Vec<WorkEntry>, String> {
    list_work_entries_for_state(&state, filter).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn create_work_entry(
    state: State<'_, AppState>,
    input: NewWorkEntry,
) -> std::result::Result<WorkEntry, String> {
    create_work_entry_for_state(&state, input).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn update_work_entry(
    state: State<'_, AppState>,
    id: i64,
    patch: WorkEntryPatch,
) -> std::result::Result<WorkEntry, String> {
    update_work_entry_for_state(&state, id, patch).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn confirm_work_entry(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<WorkEntry, String> {
    confirm_work_entry_for_state(&state, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn archive_work_entry(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<WorkEntry, String> {
    archive_work_entry_for_state(&state, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn delete_work_entry(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<usize, String> {
    delete_work_entry_for_state(&state, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_work_calendars(
    state: State<'_, AppState>,
) -> std::result::Result<Vec<WorkCalendar>, String> {
    list_work_calendars_for_state(&state).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn create_work_calendar(
    state: State<'_, AppState>,
    input: NewWorkCalendar,
) -> std::result::Result<WorkCalendar, String> {
    create_work_calendar_for_state(&state, input).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn update_work_calendar(
    state: State<'_, AppState>,
    id: i64,
    patch: WorkCalendarPatch,
) -> std::result::Result<WorkCalendar, String> {
    update_work_calendar_for_state(&state, id, patch).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn delete_work_calendar(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<usize, String> {
    delete_work_calendar_for_state(&state, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn set_default_calendar(
    state: State<'_, AppState>,
    id: i64,
) -> std::result::Result<WorkCalendar, String> {
    set_default_calendar_for_state(&state, id).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn export_report_source(
    state: State<'_, AppState>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    format: ReportFormatDto,
) -> std::result::Result<String, String> {
    export_report_source_for_state(&state, start, end, format).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> DesktopSettings {
    get_settings_for_state(&state)
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    settings: DesktopSettings,
) -> std::result::Result<DesktopSettings, String> {
    save_settings_for_state(&state, settings).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_filters(
    state: State<'_, AppState>,
    filters: EntryFiltersSetting,
) -> std::result::Result<DesktopSettings, String> {
    save_filters_for_state(&state, filters).map_err(|err| err.to_string())
}
