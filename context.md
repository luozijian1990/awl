# Context - 缘起与产品方向

## 这个项目从哪来

这个项目来自两个个人原型 skill：

1. `log-work`：在一次 design / 计划 / 实现工作收尾时，把上下文总结成一条“动作 + 结果 + 价值”的工作项，经人确认后写入 macOS 的企业微信 `oa ` 日历。
2. `write-oa-weekly-report`：导出 `oa ` 日历，筛出 `organizer == null` 的个人工作项，再让 AI 合并、改写成周报 Markdown。

这两个原型证明了工作流是成立的：

- AI 可以生成有价值的工作记录。
- 人需要确认或编辑，避免噪音进入正式台账。
- 后续周报/月报可以从已保存的源数据里自动生成。

但原型也有明显限制：

- 绑定 macOS EventKit。
- 绑定企业微信同步出来的 `oa ` 日历。
- 把 CalendarEvent 当成事实源。
- 格式和流程偏个人使用习惯。

## 新定位

`worklog` 是一个面向 AI 时代工作流的通用本地桌面工作日历。

事实源是 `WorkEntry`，不是日历事件。桌面日历是主要复核界面；Google、macOS Calendar、ICS、企业日历等只是后续同步目标。

## 当前状态

**planning** —— 本仓库还未开始实现。

当前计划已经从旧的 Bun sidecar 方案调整为：

```text
Rust core + SQLite
  + Rust CLI 给 skill / agent / script 调用
  + Tauri desktop 做复核、日历视图、编辑和确认
  + adapter 接口留给未来外部日历同步
```

## 如何承接

1. 读 `docs/plans/2026-06-16-worklog-tool-design.md`。
2. 读 `docs/plans/2026-06-16-worklog-tool-tasks.md`。
3. 实现前先安装 Rust/Tauri 依赖。
4. 按 task 逐个实现，始终保持 `WorkEntry` 是领域模型，日历同步只是投影。

## 关键约束

- `WorkEntry` 是核心对象。
- `CalendarEvent` 只是展示或同步投影。
- AI 创建的记录默认进入 `draft`；周报/月报只使用 `confirmed`。
- Desktop 是一等入口；CLI 是 AI/skill/agent 写入入口。
- MVP 不同步 Google、macOS Calendar、ICS、企业日历。
- MVP 不内置 LLM；AI 汇总由读取导出数据的 skill / client 完成。
