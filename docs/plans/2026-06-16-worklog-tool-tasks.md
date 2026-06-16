# Tasks: Worklog 工具 MVP

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.
>
> **Design:** @docs/plans/2026-06-16-worklog-tool-design.md

**Goal:** 构建一个 Rust/Tauri 桌面工作日历：AI skill 和 CLI 写入本地 `WorkEntry`，用户在桌面端复核/确认，报表导出 confirmed source items。

---

## 1. 工具链与仓库脚手架

- [ ] 1.1 新建 `docs/development.md`：记录 Rust stable、Cargo、Tauri v2 各系统 prerequisites、Node/Vite 前端依赖、SQLite 构建注意事项。约束：不能假设当前本机已经安装 Rust。
- [ ] 1.2 新建根 `Cargo.toml` workspace，成员包括 `crates/worklog-core`、`crates/worklog-cli`、`desktop/src-tauri`。
- [ ] 1.3 新建 `.gitignore`：覆盖 Rust、Tauri、Node、本地 SQLite DB、构建产物。约束：不要忽略 planning docs。
- [ ] 1.4 如果实现过程中架构或命令名偏离 design，同步更新 `README.md`。约束：README 不能重新描述为 Web 项目。

## 2. Core Crate：领域模型与存储

- [ ] 2.1 创建 `crates/worklog-core`，模块包括 `model`、`db`、`repo`、`report`、`sync`。约束：core crate 不包含 Tauri UI 代码，不解析 CLI argv。
- [ ] 2.2 在 `model` 定义 `WorkCalendar`、`WorkEntry`、`CalendarProjection` 和 input/patch DTO。约束：`WorkEntry` 是事实源，calendar event id 只能出现在 projection 类型中。
- [ ] 2.3 为 `status=draft|confirmed|archived`、`actor=ai|human`、已知 `source` 值增加校验，同时允许 custom source string。
- [ ] 2.4 在 `db` 实现 SQLite 路径解析：优先 `WORKLOG_DB` → 否则使用 OS-appropriate app data path → 创建父目录 → 打开 SQLite。约束：不能写死用户个人路径。
- [ ] 2.5 使用 `include_str!` 嵌入 migrations：创建 `work_calendars`、`work_entries`、`calendar_projections` → 启动时幂等执行 → 开启 foreign keys。
- [ ] 2.6 保证 `work_calendars` 最多一个默认项：用 schema constraint 或 repo transaction。约束：创建新默认项时要清掉旧默认项，或以可预期错误失败。

## 3. Core Crate：Repository 操作

- [ ] 3.1 实现 `ensure_default_calendar()`：不存在默认本地工作日历时创建 → 返回默认日历。
- [ ] 3.2 实现 `create_work_entry(input)`：校验 title 非空 → 校验 `ended_at > started_at` → AI source 默认 `status=draft` → calendar 缺省取默认项 → insert → 返回完整 entry。
- [ ] 3.3 实现 `update_work_entry(id, patch)`：不存在则报错 → 只更新传入字段 → 刷新 `updated_at` → 返回完整 entry。
- [ ] 3.4 实现 `confirm_work_entry(id)` 和 `archive_work_entry(id)`：切换 status → 刷新 `updated_at` → 返回完整 entry。约束：报表只使用 `confirmed`，不使用 `draft`。
- [ ] 3.5 实现 `delete_work_entry(id)`：不存在则报错 → 先删除 projections 或依赖 cascade → 返回删除结果。
- [ ] 3.6 实现 `list_work_entries(filter)`：支持 `start`、`end`、`status`、`calendar_id`、`project`、`actor`、`source` 过滤 → 按 `started_at` 和 title 排序。
- [ ] 3.7 实现 MVP 需要的 work calendar CRUD：create、list、set default、get default。

## 4. Core Crate：报表源数据与导出

- [ ] 4.1 实现 `report_source(start, end, format)`：只查询 `confirmed` entries → 输出 title、project、status、actor、source、time range、tags、evidence。
- [ ] 4.2 实现 raw numbered Markdown 导出：一条 confirmed entry title 对应一行 → 不加标题、总结、嵌套列表或 AI 改写。
- [ ] 4.3 实现 JSON 导出：字段名稳定 → 保留 source order → 包含周报/月报 AI 汇总所需元数据。
- [ ] 4.4 增加 `sync::SyncAdapter` 和 projection 类型。约束：MVP 不实现 Google、macOS Calendar、ICS 或企业日历同步。

## 5. CLI Crate

- [ ] 5.1 创建 `crates/worklog-cli` binary package，依赖 `worklog-core`。约束：CLI 只能调 core API，不能直接写 SQL。
- [ ] 5.2 实现命令解析：`entry add|edit|confirm|archive|rm|list`、`calendar add|list|default`、`export report-source`。约束：命令是 skill/agent 的稳定 contract。
- [ ] 5.3 实现 `entry add` 参数：`--title`、`--start`、`--end`、`--body`、`--raw-input`、`--project`、`--actor`、`--source`、`--status`、`--tag`、`--evidence`、`--calendar`。
- [ ] 5.4 当 `actor=ai` 或 `source` 是 AI client 时，`entry add` 默认 `status=draft`。约束：受信任流程必须显式传 `--status confirmed`。
- [ ] 5.5 实现 `entry list` 的 table 和 JSON 输出。约束：JSON mode 面向 AI client，字段必须稳定。
- [ ] 5.6 实现 `export report-source --start --end --format json|markdown`：调用 core report export → 输出到 stdout 或 `--output`。
- [ ] 5.7 新建 `docs/ai-client-contract.md`：写 future `log-work` 和周报/月报 skill 的 CLI 调用示例。

## 6. Desktop App：Tauri Shell 与 Commands

- [ ] 6.1 在 `desktop/` 创建 Tauri v2 app，前端使用最小 TypeScript。约束：这是 desktop app，正常使用不需要 standalone web server。
- [ ] 6.2 将 `desktop/src-tauri` 配成 Cargo workspace member，并依赖 `worklog-core`。
- [ ] 6.3 实现 Tauri commands：list/create/update/confirm/archive/delete work entries，list/create work calendars，export report source。
- [ ] 6.4 所有 Tauri commands 都调用 `worklog-core` API。约束：frontend 不写 SQL，desktop shell 不复制一套私有持久化逻辑。
- [ ] 6.5 保存 desktop settings：DB path、当前 filters。约束：切换 DB path 时必须干净重开 core storage，或提示需要重启。

## 7. Desktop App：UI

- [ ] 7.1 做 draft inbox view：列出 `draft` entries → 展示 title、source、project、time range、tags、evidence count → 支持 confirm/archive/edit。
- [ ] 7.2 做 confirmed entries 的日历/列表视图：MVP 可 list-first，再扩 day/week/month → 支持按日期、project、calendar、source、status 过滤。
- [ ] 7.3 做 `WorkEntry` create/edit form：title、body、project、start/end、status、actor、source、tags、evidence links、calendar。
- [ ] 7.4 做 draft 批量操作：confirm、archive、delete。约束：必须让清理 AI 过度记录足够顺手。
- [ ] 7.5 做 report-source export panel：选择日期范围 → 选择 JSON 或 raw Markdown → preview → copy/save。
- [ ] 7.6 UI 文案弱化传统日历语义：强调 work entries、drafts、projects、reports，避免把 MVP 引向 meetings/attendees/availability。

## 8. 原型 Skill 迁移说明

- [ ] 8.1 记录 `log-work` 到新 CLI 的映射：总结工作 → 人确认 → `worklog entry add --status confirmed`，或自动提议 → `--status draft`。
- [ ] 8.2 记录 `write-oa-weekly-report` 到新导出流程的映射：`worklog export report-source --format json` → AI 合并/改写 → Markdown report。
- [ ] 8.3 记录旧 `organizer == null` 过滤的替代方案：使用 `actor`、`source` 和 `status=confirmed`。
- [ ] 8.4 保留 macOS EventKit 脚本作为历史参考。约束：MVP 不调用这些脚本。

## 9. 验证

- [ ] 9.1 执行 `cargo fmt --check`。
- [ ] 9.2 执行 `cargo test --workspace`。
- [ ] 9.3 使用临时 `WORKLOG_DB` 做 CLI smoke test：创建 draft entry → list draft → confirm → export confirmed JSON 和 Markdown。
- [ ] 9.4 做 desktop dev smoke test：启动 Tauri → 创建 draft → 编辑 → 确认 → 验证在 confirmed 日历/列表视图可见。
- [ ] 9.5 确认 MVP 中不存在 Bun sidecar、正常使用不需要 local HTTP server、也没有外部日历同步实现。
- [ ] 9.6 确认 `docs/ai-client-contract.md` 已说明 skill 和 agent 的稳定 CLI contract。

## Decision→Task 映射检查

- D1 WorkEntry first → Tasks 2.2, 3.2, 6.4, 7.6 ✓
- D2 Rust core + SQLite → Tasks 1.2, 2.1, 2.4, 2.5, 6.2 ✓
- D3 Tauri desktop without sidecar HTTP → Tasks 6.1, 6.3, 9.5 ✓
- D4 CLI is the AI integration surface → Tasks 5.1, 5.2, 5.7, 8.1 ✓
- D5 Draft by default for AI-created entries → Tasks 3.2, 3.4, 5.4, 7.1, 7.4 ✓
- D6 Reports export source data, not final prose → Tasks 4.1, 4.2, 4.3, 5.6, 7.5, 8.2 ✓
- D7 Sync adapters are phase 2 → Tasks 4.4, 8.4, 9.5 ✓
- D8 No recurrence in MVP → Tasks 2.2, 9.5 ✓
