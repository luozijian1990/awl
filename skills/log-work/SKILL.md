---
name: log-work
description: 在一段 design / 计划 / 实现工作收尾时，主动提议把本次工作总结成一条「动作+结果+价值」，经人工确认后用 worklog CLI 写入本地工作日历（worklog entry add）。AI 自动提议默认进 draft 收件箱，人工确认后写 confirmed。当 brainstorming / writing-plans / executing-plans / test-driven-development 等流程收尾、一批任务标记完成，或用户说“记一下这次的活”“记工作日志”“log work”“记到 worklog”时使用；也可手动调用。
---

# 记录本次工作到 worklog（log-work）

## 概述

用对话内已有的上下文，把"这次干了啥"总结成**一条**「动作 + 结果 + 价值」工作项，通过 `worklog` CLI 写入本地工作日历（SQLite，桌面 app 共库可见）。

不再依赖 macOS EventKit / `oa ` 日历 / `organizer == null` 约定——改用 worklog 的 `draft`/`confirmed` 状态：AI 自动记的先进 `draft` 收件箱待复核，经人工确认的直接写 `confirmed`。

## 何时触发

- **主动提议**：一段 design / 计划 / 实现工作明显收尾时（如 `brainstorming` / `writing-plans` / `executing-plans` / `test-driven-development` 流程结束，或一批 task 标记完成）。**提议一次即可**，被拒就停，别反复弹。
- **手动**：用户显式要求记录本次工作。
- **不要**在每勾掉一个内置 task-list 子项时触发——粒度太碎、噪音大。

## 前置

- 已安装 `worklog` CLI（`cargo build -p worklog-cli` 后用 `target/debug/worklog`，或安装到 PATH）。
- CLI 与桌面 app 默认共用同一个本地库；无需额外配置。

## 工作流

1. 总结一条工作项标题。
   - 用对话内上下文，产出**一条**「动作 + 结果 + 价值」标题。
   - 文风参照 `../weekly-report/references/report-format.md`（存在则读）。
   - 一条说不清就在 HITL 里和用户拆分/改写，别硬塞。

2. 估起止时间（时分粗略即可）。
   - `end` = 现在：`date "+%Y-%m-%d %H:%M"`。
   - `start` = 今天**当前工作 repo** 最早一次 commit：
     `git -C <当前项目目录> log --since="today 00:00" --reverse --format=%cI | head -1`，取首行；
     无输出则 `start = end - 1 小时`。
   - 时间可用 `YYYY-MM-DD HH:MM`（按 UTC 解析）或带时区的 ISO8601。

3. HITL 确认（**必经**）。
   - 用 `AskUserQuestion` 展示拟写入的 **{标题, 开始, 结束, 日历组}**，选项：确认 / 编辑 / 取消。
   - **编辑**：让用户用自然语言描述要改什么（标题 / 时间 / 换日历组），改完再确认一次。
   - **取消**：什么都不写，结束。
   - 日历组默认用默认组（不传 `--calendar`）。用户要指定别的组：先 `worklog calendar list` 拿到目标组的 `id`，再用 `--calendar <id>`。

4. 写入（确认后）。
   - 经人工确认 → 写 `confirmed`：
     ```sh
     worklog entry add \
       --title "<动作+结果+价值>" \
       --start "<开始>" --end "<结束>" \
       --source claude --actor ai --status confirmed \
       [--project <项目>] [--calendar <组id>] \
       [--tag <标签>]... [--evidence <文件或URL>]...
     ```
   - 若是自动提议、用户未逐条确认（只想先攒着待复核）→ 省略 `--status`，落 `draft` 收件箱：
     ```sh
     worklog entry add --title "..." --start "..." --end "..." --source claude --actor ai
     ```
   - `--tag` / `--evidence` 可重复多次。`--source` 用当前 AI client（Claude Code 用 `claude`）。

5. 回报结果。
   - 成功后把写入的标题 / 时间 / 状态简要告诉用户。
   - 提醒去向：`confirmed` 在桌面 app 的「已确认」视图；`draft` 在「Drafts」收件箱等复核。

## 资源

- 文风参考：`../weekly-report/references/report-format.md`
- CLI 契约：项目内 `docs/ai-client-contract.md`
- 历史参考（MVP 不调用）：原型用 macOS EventKit 写 `oa ` 日历，脚本在 `~/git/claude-worklog/skill/log-work/scripts/write_calendar_event.swift`。
