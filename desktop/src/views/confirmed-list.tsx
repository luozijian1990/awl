import { entries, loading, errorMsg, filters, setFilters, calendars } from "../store";
import { PlusIcon } from "../icons";
import { EntryRow } from "./entry-row";
import { openForm } from "./entry-form";

const SOURCES = ["claude", "codex", "cli", "desktop", "manual"];

export function ConfirmedList() {
  const list = entries.value;
  const f = filters.value;

  return (
    <>
      <header class="topbar">
        <div>
          <h1 class="topbar__title">已确认</h1>
          <p class="topbar__sub">{list.length} 条工作项 · 仅 confirmed 进入报表导出</p>
        </div>
        <div class="topbar__tools">
          <button class="newbtn" title="新建工作项 (⌘N)" onClick={() => openForm()}>
            <PlusIcon size={15} />
            <span>新建</span>
          </button>
        </div>
      </header>

      <div class="filterbar">
        <div class="segmented">
          <button class={`seg ${f.range === "all" ? "seg--on" : ""}`} onClick={() => setFilters({ range: "all" })}>
            全部
          </button>
          <button class={`seg ${f.range === "today" ? "seg--on" : ""}`} onClick={() => setFilters({ range: "today" })}>
            今天
          </button>
          <button class={`seg ${f.range === "week" ? "seg--on" : ""}`} onClick={() => setFilters({ range: "week" })}>
            近 7 天
          </button>
        </div>

        <select
          class="input select filter-select"
          value={f.calendarId != null ? String(f.calendarId) : ""}
          onChange={(e) => {
            const v = (e.currentTarget as HTMLSelectElement).value;
            setFilters({ calendarId: v ? Number(v) : null });
          }}
        >
          <option value="">全部分组</option>
          {calendars.value.map((c) => (
            <option value={String(c.id)}>{c.name}</option>
          ))}
        </select>

        <select
          class="input select filter-select"
          value={f.source ?? ""}
          onChange={(e) => setFilters({ source: (e.currentTarget as HTMLSelectElement).value || null })}
        >
          <option value="">全部来源</option>
          {SOURCES.map((s) => (
            <option value={s}>{s}</option>
          ))}
        </select>
      </div>

      {errorMsg.value && <div class="banner banner--error">{errorMsg.value}</div>}

      <div class="list">
        {loading.value && list.length === 0 ? (
          <div class="empty">载入中…</div>
        ) : list.length === 0 ? (
          <div class="empty">
            <div class="empty__title">没有匹配的工作项</div>
            <div class="empty__sub">换个时间范围或过滤条件，或在收件箱里确认草稿。</div>
          </div>
        ) : (
          list.map((e, i) => (
            <div class="list__item" key={e.id} style={{ animationDelay: `${i * 40}ms` }}>
              <EntryRow entry={e} selectable={false} />
            </div>
          ))
        )}
      </div>
    </>
  );
}
