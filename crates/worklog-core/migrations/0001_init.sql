-- 0001_init: 初始 schema。语句使用 IF NOT EXISTS，启动时可幂等执行。

-- work_calendars: 本地分组与桌面视图颜色元数据
CREATE TABLE IF NOT EXISTS work_calendars (
    id         INTEGER PRIMARY KEY,
    uid        TEXT    NOT NULL UNIQUE,
    name       TEXT    NOT NULL,
    color      TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL
);

-- 最多只能有一个默认 calendar（部分唯一索引）
CREATE UNIQUE INDEX IF NOT EXISTS idx_work_calendars_single_default
    ON work_calendars (is_default) WHERE is_default = 1;

-- work_entries: 事实源工作项
CREATE TABLE IF NOT EXISTS work_entries (
    id            INTEGER PRIMARY KEY,
    uid           TEXT    NOT NULL UNIQUE,
    calendar_id   INTEGER NOT NULL REFERENCES work_calendars (id),
    title         TEXT    NOT NULL,
    body          TEXT,
    raw_input     TEXT,
    project       TEXT,
    status        TEXT    NOT NULL CHECK (status IN ('draft', 'confirmed', 'archived')),
    actor         TEXT    NOT NULL CHECK (actor IN ('ai', 'human')),
    source        TEXT    NOT NULL CHECK (length(trim(source)) > 0),
    started_at    TEXT    NOT NULL,
    ended_at      TEXT    NOT NULL,
    tags_json     TEXT    NOT NULL DEFAULT '[]',
    evidence_json TEXT    NOT NULL DEFAULT '[]',
    created_at    TEXT    NOT NULL,
    updated_at    TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_work_entries_started_at ON work_entries (started_at);
CREATE INDEX IF NOT EXISTS idx_work_entries_status ON work_entries (status);
CREATE INDEX IF NOT EXISTS idx_work_entries_calendar ON work_entries (calendar_id);

-- calendar_projections: Phase 2 同步状态（MVP 建表但不实际同步）
CREATE TABLE IF NOT EXISTS calendar_projections (
    id                INTEGER PRIMARY KEY,
    work_entry_id     INTEGER NOT NULL REFERENCES work_entries (id) ON DELETE CASCADE,
    provider          TEXT    NOT NULL,
    external_event_id TEXT,
    sync_status       TEXT    NOT NULL,
    last_synced_at    TEXT,
    last_error        TEXT,
    created_at        TEXT    NOT NULL,
    updated_at        TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_calendar_projections_entry
    ON calendar_projections (work_entry_id);
