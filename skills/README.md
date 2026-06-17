# Worklog Skills

把原型阶段的两个个人 skill（`log-work` 和 `write-oa-weekly-report`，依赖 macOS EventKit + 企业微信同步的 `oa ` 日历）迁移到 worklog 工具的稳定 CLI 上。

| 原型 skill | 新 skill | 写入/读取方式的变化 |
|-----------|---------|--------------------|
| `log-work`（Swift EventKit 写 `oa ` 日历） | [`log-work/`](./log-work/SKILL.md) | 改用 `worklog entry add` 写本地 SQLite |
| `write-oa-weekly-report`（导出 `oa ` 日历 + `organizer==null` 过滤） | [`weekly-report/`](./weekly-report/SKILL.md) | 改用 `worklog export report-source --format json`（只含 confirmed entries） |

## 迁移要点

- **CLI 是稳定接入面**：skill 只调 `worklog` 命令，不直接读写 SQLite（见 `docs/ai-client-contract.md`）。
- **AI 默认进 draft**：`worklog entry add` 在 `--actor ai` 或 AI client `--source`（`claude` / `codex`）下，省略 `--status` 即落 `draft` 收件箱；经人工确认的流程显式传 `--status confirmed`。
- **`organizer == null` 的替代**：旧周报靠"事件无 attendee → organizer 为 null"筛个人工作。新流程不需要这个约定——`worklog export report-source` 本身只导出 `status=confirmed` 的工作项。需要进一步细分时用 `--actor` / `--source`（在 `entry list` 上）。
- **分组从 `oa ` 日历 → 日历组**：`oa ` 这个具名日历换成 worklog 的本地日历组（`worklog calendar list/add`，默认组）。
- **CLI 与桌面 app 共库**：两者默认都用同一个 SQLite（`~/Library/Application Support/worklog/worklog.db`），CLI 写的工作项会出现在桌面 app 里。

## 历史参考（MVP 不调用）

原型的 macOS EventKit 脚本保留在原仓库作为历史参考，MVP 流程**不再调用**它们：

- `~/git/claude-worklog/skill/log-work/scripts/write_calendar_event.swift`
- `~/.claude/skills/write-oa-weekly-report/scripts/export_calendar_eventkit.swift`
- `~/.claude/skills/write-oa-weekly-report/scripts/print_null_organizer_titles.py`

## 安装

这两个 skill 随项目版本管理。要在 Claude Code 里启用，可软链到 `~/.claude/skills/`（会替代原型版同名 skill）：

```sh
ln -s "$PWD/skills/log-work" ~/.claude/skills/log-work
ln -s "$PWD/skills/weekly-report" ~/.claude/skills/weekly-report
```
