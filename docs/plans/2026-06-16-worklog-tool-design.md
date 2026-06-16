# Worklog 工具 - Design（MVP）

> 2026-06-16。本设计把已有的个人 `log-work` 和 `write-oa-weekly-report` skill 原型，重构成一个通用的 local-first 桌面工作日历。

---

## 1. 背景

当前原型已经跑通一个闭环：

1. AI 完成一段有意义的工作。
2. `log-work` 把上下文总结成一条“动作 + 结果 + 价值”标题。
3. 用户确认或编辑。
4. macOS EventKit 脚本把事件写入企业微信同步出来的 `oa ` 日历。
5. `write-oa-weekly-report` 后续导出该日历，筛出个人工作项，再让 AI 合并/改写成周报。

问题在于这个原型是个人化且平台绑定的：依赖 macOS EventKit、`oa ` 日历和 `organizer == null` 约定。

新的产品方向是：做一个通用桌面应用，让 skill、agent、CLI 和人都能写入本地工作项。日历 UI 和外部日历同步只是这些工作项的展示/投影。

---

## 2. Goals / Non-Goals

### Goals

- 做一个 local-first 的 AI 工作记录桌面应用。
- 让 `WorkEntry` 成为事实源。
- 提供稳定 CLI contract，供 AI skill 和本地 agent 调用。
- AI 创建的记录默认进入 `draft`；报表只使用 `confirmed`。
- 提供 Tauri desktop UI：draft 复核、编辑、日历/列表查看、确认/归档。
- 所有数据通过共享 Rust core 写入本地 SQLite。
- 导出已确认工作项，供 AI 生成周报/月报。
- 同步适配器只留接口，放到 phase 2。

### Non-Goals

- 不做通用日历竞品。
- 不把 Google Calendar、macOS Calendar、ICS 或企业日历作为事实源。
- MVP 不实现任何外部同步。
- 不把 LLM 内置进 core 或 desktop。
- core 产品路径不依赖 macOS EventKit。
- MVP 不做 recurrence/RRULE。
- desktop core path 不使用 Bun sidecar 或本地 HTTP server。

---

## 3. 架构

```text
AI skills / coding agents / shell scripts
              |
              v
        worklog CLI
              |
              v
┌────────────────────────────────────────────┐
│ Rust core                                  │
│ - domain model: WorkEntry, WorkCalendar    │
│ - SQLite migrations and repositories       │
│ - report/export queries                    │
│ - sync adapter traits only                 │
└───────────────────┬────────────────────────┘
                    │
                    v
              local SQLite
                    ^
                    │
┌───────────────────┴────────────────────────┐
│ Tauri desktop                              │
│ - draft inbox                              │
│ - calendar/list views                      │
│ - create/edit/confirm/archive entries      │
│ - export source items for reports          │
└────────────────────────────────────────────┘

Phase 2:
  WorkEntry -> CalendarProjection -> Google / macOS Calendar / ICS / enterprise
```

Tauri desktop 通过 Rust command 调用同一个 core crate。MVP 没有隐藏的本地 Web server。

---

## 4. 数据模型

### `work_calendars`

本地分组和桌面视图颜色元数据。

- `id` INTEGER PRIMARY KEY
- `uid` TEXT UNIQUE NOT NULL
- `name` TEXT NOT NULL
- `color` TEXT
- `is_default` INTEGER NOT NULL DEFAULT 0
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

约束：最多只能有一个默认 calendar。

### `work_entries`

事实源工作项。

- `id` INTEGER PRIMARY KEY
- `uid` TEXT UNIQUE NOT NULL
- `calendar_id` INTEGER NOT NULL REFERENCES `work_calendars(id)`
- `title` TEXT NOT NULL
- `body` TEXT
- `raw_input` TEXT
- `project` TEXT
- `status` TEXT NOT NULL (`draft` / `confirmed` / `archived`)
- `actor` TEXT NOT NULL (`ai` / `human`)
- `source` TEXT NOT NULL (`codex` / `claude` / `cli` / `desktop` / `manual` / custom string)
- `started_at` TEXT NOT NULL
- `ended_at` TEXT NOT NULL
- `tags_json` TEXT NOT NULL DEFAULT `[]`
- `evidence_json` TEXT NOT NULL DEFAULT `[]`
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

`title` 是简洁的“动作 + 结果 + 价值”。`raw_input` 保留清洗前的来源文本。`body`、tags 和 evidence links 用于更丰富的后续汇总。

### `calendar_projections`

Phase 2 同步状态。MVP 可以建表或只定义 trait surface，但不做实际同步。

- `id` INTEGER PRIMARY KEY
- `work_entry_id` INTEGER NOT NULL REFERENCES `work_entries(id)`
- `provider` TEXT NOT NULL
- `external_event_id` TEXT
- `sync_status` TEXT NOT NULL (`pending` / `synced` / `failed`)
- `last_synced_at` TEXT
- `last_error` TEXT
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

---

## 5. CLI Contract

CLI 是 skill 和 agent 的稳定接入面。

示例：

```sh
worklog entry add \
  --title "完成 worklog 产品定位重构，明确 WorkEntry 优先模型，为 Rust/Tauri 桌面实现提供边界" \
  --start "2026-06-16 10:00" \
  --end "2026-06-16 11:00" \
  --actor ai \
  --source codex \
  --status draft

worklog entry confirm <id>
worklog entry list --start 2026-06-01 --end 2026-06-30 --status confirmed
worklog export report-source --start 2026-06-01 --end 2026-06-30 --format json
```

AI client 默认写 `draft`。受信任流程可以在用户确认后传 `--status confirmed`，或先写入再调用 `confirm`。

---

## 6. Decisions

- **D1 WorkEntry first**：产品存工作记录，不存日历事件；日历视图和外部同步只是投影。
- **D2 Rust core + SQLite**：领域模型、迁移、仓储和报表查询放在共享 Rust crate，CLI 和 desktop 共用。
- **D3 Tauri desktop without sidecar HTTP**：desktop 是一等入口，直接调用 Rust command；MVP 不使用 Bun、sidecar server 或本地 HTTP IPC。
- **D4 CLI is the AI integration surface**：skill 和 agent 调稳定 CLI，不依赖 desktop 内部实现。
- **D5 Draft by default for AI-created entries**：AI 输出先进入 inbox；confirmed entries 才进入报表和导出。
- **D6 Reports export source data, not final prose**：core 只输出 raw numbered Markdown 或 JSON source items；合并、润色、月报叙事由 AI client 完成。
- **D7 Sync adapters are phase 2**：MVP 只定义 projection trait/table，不实现 Google/macOS/ICS/企业日历同步。
- **D8 No recurrence in MVP**：工作记录优先解决时间范围和复核流程，不进入 recurrence 复杂语义。

---

## 7. Risks / Trade-offs

- **Rust/Tauri 工具链门槛**：当前机器可能还没装 Rust。缓解：先写清 prerequisites，发布构建可交给 CI。
- **日历外观带来 scope creep**：用户可能期待会议、重复、参会人、提醒、忙闲状态。缓解：UI 文案和数据模型持续强调 `WorkEntry`。
- **draft inbox 可能积累噪音**：AI 可能过度记录。缓解：报表只取 confirmed，desktop 必须让批量确认/归档足够顺手。
- **报表质量依赖 client**：MVP 不做 LLM 改写。缓解：导出足够字段：title、project、status、actor、source、time、tags、evidence。

---

## 8. 验收标准

1. CLI 能创建一条 AI draft `WorkEntry` 到 SQLite，并能查出来。
2. Desktop 能展示 draft，编辑一条，确认后在日历/列表视图可见。
3. CLI 和 desktop 使用同一个 Rust core 和同一个 SQLite 数据库。
4. Export command 能按日期范围输出 confirmed entries 的 JSON 和 raw numbered Markdown。
5. MVP 不同步 Google、macOS Calendar、ICS 或企业日历。
6. 正常 desktop 使用不需要 Bun sidecar 或本地 HTTP server。
