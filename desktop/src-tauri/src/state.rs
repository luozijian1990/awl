use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use directories::ProjectDirs;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use worklog_core::{db, Result, WorklogError};

pub struct AppState {
    conn: Mutex<Connection>,
    settings: Mutex<DesktopSettings>,
    settings_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub db_path: Option<PathBuf>,
    pub current_filters: EntryFiltersSetting,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntryFiltersSetting {
    pub status: Option<String>,
    pub calendar_id: Option<i64>,
    pub project: Option<String>,
    pub actor: Option<String>,
    pub source: Option<String>,
}

impl AppState {
    pub fn open() -> Result<Self> {
        let db_path = db::resolve_db_path()?;
        Self::open_at_with_settings_path(db_path, default_settings_path()?)
    }

    pub fn open_at_with_settings_path(db_path: PathBuf, settings_path: PathBuf) -> Result<Self> {
        let settings = read_settings(&settings_path)?.unwrap_or_else(|| DesktopSettings {
            db_path: Some(db_path.clone()),
            current_filters: EntryFiltersSetting::default(),
        });
        let active_db_path = settings.db_path.clone().unwrap_or(db_path);
        let conn = db::open_at(&active_db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
            settings: Mutex::new(DesktopSettings {
                db_path: Some(active_db_path),
                current_filters: settings.current_filters,
            }),
            settings_path: Some(settings_path),
        })
    }

    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", true)?;
        db::run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            settings: Mutex::new(DesktopSettings::default()),
            settings_path: None,
        })
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let conn = self.conn.lock().expect("app db connection mutex poisoned");
        f(&conn)
    }

    pub fn get_settings(&self) -> DesktopSettings {
        self.settings
            .lock()
            .expect("desktop settings mutex poisoned")
            .clone()
    }

    pub fn save_settings(&self, settings: DesktopSettings) -> Result<DesktopSettings> {
        let next_db_path = settings.db_path.clone();

        if let Some(db_path) = next_db_path.as_ref() {
            let mut conn = self.conn.lock().expect("app db connection mutex poisoned");
            *conn = db::open_at(db_path)?;
        }

        {
            let mut current = self
                .settings
                .lock()
                .expect("desktop settings mutex poisoned");
            *current = settings;
            if let Some(path) = &self.settings_path {
                write_settings(path, &current)?;
            }
            Ok(current.clone())
        }
    }

    pub fn update_filters(&self, filters: EntryFiltersSetting) -> Result<DesktopSettings> {
        let next = {
            let mut settings = self
                .settings
                .lock()
                .expect("desktop settings mutex poisoned");
            settings.current_filters = filters;
            settings.clone()
        };
        if let Some(path) = &self.settings_path {
            write_settings(path, &next)?;
        }
        Ok(next)
    }
}

fn default_settings_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "worklog").ok_or(WorklogError::NoDataDir)?;
    Ok(dirs.config_dir().join("desktop-settings.json"))
}

fn read_settings(path: &Path) -> Result<Option<DesktopSettings>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map(Some).map_err(Into::into)
}

fn write_settings(path: &Path, settings: &DesktopSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_vec_pretty(settings)?;
    fs::write(path, json)?;
    Ok(())
}
