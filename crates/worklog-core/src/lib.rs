//! worklog-core: 领域模型、SQLite 存储与报表查询。
//!
//! core 是 CLI 与 Tauri desktop 共用的事实源逻辑层：不含 UI、不解析 CLI argv。

pub mod db;
pub mod error;
pub mod model;
pub mod repo;
pub mod report;
pub mod sync;

pub use error::{Result, WorklogError};
pub use model::{
    Actor, CalendarProjection, NewWorkCalendar, NewWorkEntry, Source, Status, SyncStatus,
    WorkCalendar, WorkCalendarPatch, WorkEntry, WorkEntryPatch, KNOWN_SOURCES,
};
