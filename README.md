# worklog

> AI-native 本地桌面工作日历：让 skill、CLI、AI agent 和人都能把“做了什么、产生了什么结果、有什么价值”写进本地 SQLite 台账，在桌面日历里复核，并在后续生成周报/月报或同步到外部日历。

**状态：pre-alpha / planning** —— 当前方向已收敛为 Rust core + Tauri desktop。详见 `docs/plans/`。

## 为什么

现在越来越多工作由 AI agent、coding assistant 和本地工具完成。它们产出的工作记录很有价值，但通常散落在聊天记录、终端输出、提交历史和临时笔记里。

`worklog` 的目标是把这些产出沉淀成一个本地优先、可复核、可汇总的工作台账：AI 可以提议记录，人可以确认或编辑，之后再基于干净的源数据生成周报、月报或同步到外部日历。

这不是通用日历 App。核心对象是 `WorkEntry`；日历只是桌面展示方式和未来同步投影。

## MVP 范围

- **Core**：Rust + SQLite，本地优先的 `WorkEntry` 存储。
- **CLI**：给 AI skill、agent、shell 脚本和人调用的稳定入口。
- **Desktop**：Tauri 桌面端，包含 draft inbox、日历/列表视图、编辑和确认流程。
- **Reports**：导出已确认工作项，供 AI 做周报/月报汇总。
- **Sync adapters**：MVP 只留接口；Google Calendar、macOS Calendar、ICS、企业日历同步放到 phase 2。

## 架构

```text
AI skill / agent / shell
        |
        v
worklog CLI --------------+
                          |
                          v
                    Rust core + SQLite
                          ^
                          |
Tauri desktop ------------+
  draft inbox / calendar view / edit / confirm

Phase 2: sync projections -> Google / macOS Calendar / ICS / enterprise calendars
```

## 技术栈

Rust · SQLite · Tauri · TypeScript UI

## Roadmap

- [ ] **MVP**：Rust core + CLI + Tauri desktop，跑通 `WorkEntry` draft/confirmed 流程。
- [ ] **Phase 2**：同步到 Google Calendar、macOS Calendar、ICS、企业日历。
- [ ] **Phase 2+**：更完整的 AI 汇总、多端同步、团队/共享报告。

## 文档

- 设计：`docs/plans/2026-06-16-worklog-tool-design.md`
- 任务：`docs/plans/2026-06-16-worklog-tool-tasks.md`
- 上下文：`context.md`

## License

TBD
