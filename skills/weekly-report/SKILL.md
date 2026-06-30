---
name: weekly-report
description: 根据 worklog 工具里已确认（confirmed）的工作项生成纯编号 Markdown 周报。用户提出“帮我写周报”“汇总某个时间范围的工作”“生成周报”等请求时使用。通过 worklog export report-source --format json 按日期范围导出 confirmed 工作项作为事实源，再按用户偏好的「动作 + 结果 + 价值」合并改写成纯编号 Markdown 列表。
---

# 生成 worklog 周报

## 概述

从 worklog 工具导出指定时间范围内的 **confirmed** 工作项，作为事实源，合并改写成纯编号的 Markdown 周报。

不依赖外部日历导出或个人日历过滤规则——`worklog export report-source` 本身只导出 `status=confirmed` 的工作项，等价于"用户已认可的个人工作"。

## 前置

- 已安装 `worklog` CLI（`cargo build -p worklog-cli` 后用 `target/debug/worklog`，或安装到 PATH）。
- 周报只覆盖 `confirmed` 工作项。draft 收件箱里的内容需先在桌面 app 或 CLI 确认（`worklog entry confirm <id>`）才会进周报。

## 工作流

1. 确认日期范围。
   - 用户没给完整起止就问。优先 `YYYY-MM-DD`。

2. 导出该范围的源数据（JSON）。
   ```sh
   worklog export report-source \
     --start "<start> 00:00" --end "<end> 23:59" \
     --format json \
     --output /tmp/worklog-report-<start>-<end>.json
   ```
   - 时间可用 `YYYY-MM-DD HH:MM`（UTC）或带时区 ISO8601。
   - 省略 `--output` 则打到 stdout。
   - 返回的每条 confirmed 工作项含：`title`、`project`、`status`、`actor`、`source`、起止时间、`tags`、`evidence`。**这些就是事实源，不要编造范围外的内容。**

3. 起草周报。
   - 读 `references/report-format.md`。
   - 把原始 `title` 改写成「动作 + 结果 + 价值」的编号条目。
   - **先判断连续性**：多条明显属于同一工作线的，合并成一条周报。
   - 用 `project` 字段辅助判断归属——同一 `project` 跨多天的多条，通常是一条工作线。
   - 典型可合并模式：`开发 -> 配置 -> 验证 -> 上线`、`排查 -> 修复 -> 回归`、`部署 -> 测试 -> 交付`，以及围绕同一服务/系统名的重复条目。
   - 除非用户要求重新分组，否则保留源顺序。

4. 写出最终 Markdown 到固定输出目录。
   - 用户指定输出目录时按用户要求写入。
   - 用户未指定时，默认写入当前工作区的 `weekly-reports/` 目录。
   - 默认文件名：`weekly-report-<start>-<end>.md`。
   - 给用户呈现最终文件时用绝对路径。

5. 措辞与状态。
   - 导出的 `status` 都是 `confirmed`（复核状态），**不代表工作完成度**——完成与否从标题动词判断。
   - 标题已表达完成（"完成/修复/上线"）就用完成式；表达推进中（"推进/持续"）就保守描述，别夸大成已完成。
   - 合并条目里同时有完成和进行中的步骤时，整体用保守措辞（如 `推进`、`持续完善`）。
   - 去掉原始标记（`==...==`、结尾状态括号、内联链接），除非用户明确要求保留。

6. 检查输出。
   - 只输出 Markdown。
   - 只输出 `1. ...`、`2. ...` 这样的纯编号列表。
   - 不加标题、总结段、嵌套层级。
   - 每条简洁、可直接使用。

## 资源

- `references/report-format.md`：用户偏好的周报结构与写作风格、改写示例。
- CLI 契约：项目内 `docs/ai-client-contract.md`。
