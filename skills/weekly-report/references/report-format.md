# 周报格式

## 固定结构

只输出纯编号 Markdown 列表：

```md
1. ...
2. ...
3. ...
```

不要添加：

- section headings
- summary paragraphs
- nested bullets
- extra explanations before or after the list

默认最终输出路径：

```text
/Users/luozijian/Documents/weekly-reports/weekly-report-<start-date>-<end-date>.md
```

## 源数据处理

源数据来自 `worklog export report-source --format json`，是该日期范围内的 **confirmed** 工作项（已替代旧的 `organizer == null` 过滤）。

- 以导出的 confirmed 工作项为唯一事实源，不要补充范围外内容。
- 写之前先判断连续性。
- 多条明显属于同一工作线时，合并成一条编号条目。
- 用 `project` 字段辅助判断归属：同一 `project` 跨多天的多条，通常是同一项目/服务/故障/部署/优化/特性链。
- 没有明显连续性时，一条源工作项对应一条编号。
- 除非用户明确要求重新分组，否则保留源顺序。
- 去掉原始标记（`==...==`、状态括号、内联文档链接），除非用户明确要求保留。
- `tags` / `evidence` 可作为改写时的上下文参考，但不要直接堆进周报正文。

## 状态处理

- 导出的 `status` 都是 `confirmed`，是**复核状态**，不代表工作完成度。完成与否从标题动词判断。
- 标题表达已完成（含 `完成`/`上线`/`修复` 等）→ 用完成式：`完成`、`修复`、`上线`、`开发并上线`。
- 标题表达推进中（含 `推进`/`持续`/`设计中` 等）→ 用进行式：`推进`、`持续处理`、`持续优化`；不要claim完成。

## 写作风格

- Keep each numbered item to one sentence.
- Use the pattern `动作 + 结果 + 价值`.
- Start with verbs such as `完成`、`推进`、`修复`、`开发`、`上线`、`优化`、`处理`.
- Do not dump raw titles directly unless the user explicitly asks for raw output.
- Do not exaggerate outcomes. If the result or value is uncertain, use a conservative description.

## 改写示例

Raw source（`worklog export` 返回的某条 `title`）:

```text
修复测试环境192.168.1.146 定时任务没有关闭 chrome 造成load异常
```

Rewrite:

```md
1. 修复测试环境 `192.168.1.146` 定时任务未关闭 Chrome 导致的 load 异常问题，恢复任务执行稳定性，降低测试环境资源异常波动风险。
```

Raw source:

```text
paas新增docker应用管理需求设计
```

Rewrite:

```md
2. 推进 PaaS 新增 Docker 应用管理需求设计，当前方案已进入设计阶段，为后续平台统一管理 Docker 应用提供支撑。
```

连续事项示例（同一 `project`，多天多条 `title`）:

```text
temporal-metrics 监控服务开发
temporal-metrics 添加基于 workflow_type 定义报警组
temporal 添加前端管理
```

建议改写：

```md
3. 持续完善 Temporal 监控与管理能力，完成 `temporal-metrics` 服务开发、按 `workflow_type` 配置报警组并推进前端管理建设，提升任务链路可观测性和运维效率。
```

连续事项示例：

```text
langfuse 本地kubernetes 部署 && 测试
修复测试环境langfuse zookeeper 资源不足oom
langfuse 生产环境上线
```

建议改写：

```md
4. 持续推进 Langfuse 环境建设，完成本地 Kubernetes 部署测试、修复测试环境资源问题并推动生产环境上线，为后续正式使用提供稳定支撑。
```
