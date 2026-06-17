//! Repository 操作：work entry / calendar 的 SQLite CRUD。
//!
//! CLI 与 desktop 共用同一套逻辑，不各自写 SQL。

mod calendar;
mod codec;
mod entry;

pub use calendar::{
    create_work_calendar, delete_work_calendar, ensure_default_calendar, get_default_calendar,
    get_work_calendar, list_work_calendars, set_default_calendar, update_work_calendar,
};
pub use entry::{
    archive_work_entry, confirm_work_entry, create_work_entry, delete_work_entry,
    list_work_entries, update_work_entry, WorkEntryFilter,
};

#[cfg(test)]
mod tests;
