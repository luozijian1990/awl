import { signal } from "@preact/signals";
import { useState } from "preact/hooks";
import {
  view,
  setView,
  setFilters,
  filters,
  calendars,
  createCalendar,
  renameCalendar,
  deleteCalendar,
  setDefaultCalendar,
} from "./store";
import type { WorkCalendar } from "./api";
import { DraftInbox } from "./views/draft-inbox";
import { ConfirmedList } from "./views/confirmed-list";
import { EntryForm } from "./views/entry-form";
import { ExportPanel, openExport } from "./views/export-panel";
import { SettingsPanel, openSettings } from "./views/settings-panel";
import { PlusIcon, EditIcon, TrashIcon, GearIcon } from "./icons";

const navItems = [
  { id: "inbox", label: "Drafts" },
  { id: "confirmed", label: "已确认" },
] as const;

const addingCalendar = signal(false);

function commitNewCalendar(input: HTMLInputElement): void {
  const name = input.value.trim();
  input.value = "";
  addingCalendar.value = false;
  if (name) void createCalendar(name);
}

function CalendarItem({ cal }: { cal: WorkCalendar }) {
  const [editing, setEditing] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const active = view.value === "confirmed" && filters.value.calendarId === cal.id;

  if (editing) {
    return (
      <input
        class="nav__addinput"
        autofocus
        value={cal.name}
        onKeyDown={(ev) => {
          if (ev.key === "Enter") {
            void renameCalendar(cal.id, (ev.currentTarget as HTMLInputElement).value);
            setEditing(false);
          } else if (ev.key === "Escape") {
            setEditing(false);
          }
        }}
        onBlur={() => setEditing(false)}
      />
    );
  }

  return (
    <div class={`nav__item nav__item--cal${active ? " nav__item--active" : ""}`}>
      <button
        class="nav__calmain"
        onClick={() => {
          setView("confirmed");
          setFilters({ calendarId: cal.id, range: "all" });
        }}
      >
        <span class="nav__hash">#</span>
        <span class="nav__label">{cal.name}</span>
      </button>

      {confirming ? (
        <span class="nav__confirm">
          <button class="nav__cbtn nav__cbtn--danger" title="连同组内工作项一起删除" onClick={() => void deleteCalendar(cal.id)}>
            删
          </button>
          <button class="nav__cbtn" onClick={() => setConfirming(false)}>
            取消
          </button>
        </span>
      ) : (
        <>
          {cal.is_default && (
            <span class="nav__star" title="默认日历组（不可删除）">★</span>
          )}
          <span class="nav__calacts">
            {!cal.is_default && (
              <button class="nav__act" title="设为默认" onClick={() => void setDefaultCalendar(cal.id)}>
                ☆
              </button>
            )}
            <button class="nav__act" title="改名" onClick={() => setEditing(true)}>
              <EditIcon size={13} />
            </button>
            {!cal.is_default && (
              <button class="nav__act" title="删除（连同组内工作项）" onClick={() => setConfirming(true)}>
                <TrashIcon size={13} />
              </button>
            )}
          </span>
        </>
      )}
    </div>
  );
}

function Sidebar() {
  const cals = calendars.value;

  return (
    <aside class="sidebar" data-tauri-drag-region>
      <div class="sidebar__brand">Worklog</div>

      <nav class="nav">
        {navItems.map((item) => (
          <button
            class={`nav__item${view.value === item.id ? " nav__item--active" : ""}`}
            onClick={() => setView(item.id)}
          >
            <span class="nav__dot" />
            <span class="nav__label">{item.label}</span>
          </button>
        ))}
      </nav>

      <div class="nav__section nav__section--row">
        <span>日历组</span>
        <button class="nav__add" title="新增日历组" onClick={() => (addingCalendar.value = !addingCalendar.value)}>
          <PlusIcon size={14} />
        </button>
      </div>

      {addingCalendar.value && (
        <input
          class="nav__addinput"
          autofocus
          placeholder="日历组名，回车添加"
          onKeyDown={(ev) => {
            if (ev.key === "Enter") commitNewCalendar(ev.currentTarget as HTMLInputElement);
            else if (ev.key === "Escape") addingCalendar.value = false;
          }}
          onBlur={(ev) => commitNewCalendar(ev.currentTarget as HTMLInputElement)}
        />
      )}

      <nav class="nav">
        {cals.map((c) => (
          <CalendarItem key={c.id} cal={c} />
        ))}
      </nav>

      <div class="sidebar__spacer" />
      <div class="sidebar__footer">
        <button class="export" onClick={openExport}>导出报表源…</button>
        <button class="iconbtn" title="配置" onClick={() => void openSettings()}>
          <GearIcon size={16} />
        </button>
      </div>
    </aside>
  );
}

export function App() {
  return (
    <div class="app">
      <Sidebar />
      <main class="content">
        {view.value === "inbox" ? <DraftInbox /> : <ConfirmedList />}
      </main>
      <EntryForm />
      <ExportPanel />
      <SettingsPanel />
    </div>
  );
}
