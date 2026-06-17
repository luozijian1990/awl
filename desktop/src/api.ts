// 前端唯一的后端入口：对 Tauri command 的强类型封装。
// 约束（design D3/6.4）：frontend 不写 SQL，只调 core API 暴露的 command。
//
// dev fallback：在浏览器里（`npm run dev`，无 Tauri runtime）退回一个内存 store，
// 让风格/交互在浏览器中可点可截图。真正打包进 Tauri 时走真实 invoke。
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type { Actor, Status, WorkEntry } from "./data";

export interface EntryFilter {
  start?: string | null;
  end?: string | null;
  status?: Status | null;
  calendar_id?: number | null;
  project?: string | null;
  actor?: Actor | null;
  source?: string | null;
}

export interface NewWorkEntry {
  title: string;
  body?: string | null;
  raw_input?: string | null;
  project?: string | null;
  status?: Status | null;
  actor: Actor;
  source: string;
  started_at: string;
  ended_at: string;
  tags?: string[];
  evidence?: string[];
  calendar_id?: number | null;
}

export interface WorkEntryPatch {
  title?: string;
  body?: string | null;
  project?: string | null;
  status?: Status;
  started_at?: string;
  ended_at?: string;
  tags?: string[];
  evidence?: string[];
  calendar_id?: number;
}

export interface WorkCalendar {
  id: number;
  uid: string;
  name: string;
  color: string | null;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface NewWorkCalendar {
  name: string;
  color?: string | null;
  is_default?: boolean;
}

export interface WorkCalendarPatch {
  name?: string;
  color?: string;
}

const inTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return inTauri ? tauriInvoke<T>(cmd, args) : devInvoke<T>(cmd, args);
}

export const api = {
  listEntries: (filter: EntryFilter) =>
    call<WorkEntry[]>("list_work_entries", { filter }),
  createEntry: (input: NewWorkEntry) =>
    call<WorkEntry>("create_work_entry", { input }),
  updateEntry: (id: number, patch: WorkEntryPatch) =>
    call<WorkEntry>("update_work_entry", { id, patch }),
  confirmEntry: (id: number) => call<WorkEntry>("confirm_work_entry", { id }),
  archiveEntry: (id: number) => call<WorkEntry>("archive_work_entry", { id }),
  deleteEntry: (id: number) => call<number>("delete_work_entry", { id }),

  listCalendars: () => call<WorkCalendar[]>("list_work_calendars"),
  createCalendar: (input: NewWorkCalendar) =>
    call<WorkCalendar>("create_work_calendar", { input }),
  updateCalendar: (id: number, patch: WorkCalendarPatch) =>
    call<WorkCalendar>("update_work_calendar", { id, patch }),
  deleteCalendar: (id: number) => call<number>("delete_work_calendar", { id }),
  setDefaultCalendar: (id: number) =>
    call<WorkCalendar>("set_default_calendar", { id }),

  exportReportSource: (start: string, end: string, format: "json" | "markdown") =>
    call<string>("export_report_source", { start, end, format }),
};

// ---------------------------------------------------------------------------
// 浏览器 dev fallback（仅 `npm run dev`，Tauri 打包时不会走到这里）
// ---------------------------------------------------------------------------
let seq = 100;
let calSeq = 1;
const now = new Date();
const iso = (dayOffset: number, h: number, m: number) => {
  const d = new Date(now);
  d.setDate(d.getDate() + dayOffset);
  d.setHours(h, m, 0, 0);
  return d.toISOString();
};

const devCalendars: WorkCalendar[] = [
  { id: 1, uid: "c1", name: "默认日历", color: null, is_default: true, created_at: "", updated_at: "" },
  { id: 2, uid: "c2", name: "worklog", color: null, is_default: false, created_at: "", updated_at: "" },
  { id: 3, uid: "c3", name: "周报", color: null, is_default: false, created_at: "", updated_at: "" },
];
calSeq = 3;

function seed(
  title: string,
  status: Status,
  actor: Actor,
  source: string,
  calendarId: number,
  day: number,
  sh: number,
  eh: number,
  tags: string[] = [],
  evidence: string[] = [],
): WorkEntry {
  const id = ++seq;
  const ts = iso(day, sh, 0);
  return {
    id,
    uid: `wk_${id}`,
    calendar_id: calendarId,
    title,
    body: null,
    raw_input: null,
    project: null,
    status,
    actor,
    source,
    started_at: iso(day, sh, 0),
    ended_at: iso(day, eh, 0),
    tags,
    evidence,
    created_at: ts,
    updated_at: ts,
  };
}

const devStore: WorkEntry[] = [
  seed("重构 worklog core 领域模型，确立 WorkEntry 为事实源", "draft", "ai", "claude", 2, 0, 10, 11, ["rust", "core"], ["crates/worklog-core/src/model.rs"]),
  seed("编写 CLI contract 文档，给 skill / agent 稳定接入面", "draft", "ai", "cli", 2, 0, 11, 12, ["docs"], ["docs/ai-client-contract.md"]),
  seed("实现 report_source 导出，只取 confirmed entries 生成周报源数据", "draft", "ai", "codex", 2, 0, 14, 15, ["report"], []),
  seed("梳理 Tauri command 列表，确认 frontend 不直接写 SQL", "draft", "human", "desktop", 1, 0, 16, 17, [], ["desktop/src-tauri/src/commands.rs"]),
  seed("完成 SQLite 迁移与默认日历约束，启动幂等执行", "confirmed", "ai", "codex", 2, -1, 9, 11, ["rust", "db"], ["crates/worklog-core/migrations/0001_init.sql"]),
  seed("评审产品定位，确认 local-first 桌面工作日历方向", "confirmed", "human", "desktop", 3, -2, 13, 14, ["product"], []),
  seed("接入企业微信周报导出流程预研", "confirmed", "human", "manual", 3, -3, 15, 17, ["report"], []),
];

function matches(e: WorkEntry, f: EntryFilter): boolean {
  if (f.status && e.status !== f.status) return false;
  if (f.calendar_id && e.calendar_id !== f.calendar_id) return false;
  if (f.source && e.source !== f.source) return false;
  if (f.actor && e.actor !== f.actor) return false;
  if (f.start && e.started_at < f.start) return false;
  if (f.end && e.started_at > f.end) return false;
  return true;
}

function defaultCalId(): number {
  return (devCalendars.find((c) => c.is_default) ?? devCalendars[0]).id;
}

async function devInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((r) => setTimeout(r, 50));
  const a = args ?? {};
  switch (cmd) {
    case "list_work_entries": {
      const f = (a.filter as EntryFilter) ?? {};
      return devStore
        .filter((e) => matches(e, f))
        .sort((x, y) => x.started_at.localeCompare(y.started_at) || x.title.localeCompare(y.title))
        .map((e) => ({ ...e })) as unknown as T;
    }
    case "create_work_entry": {
      const input = a.input as NewWorkEntry;
      const id = ++seq;
      const ts = new Date().toISOString();
      const entry: WorkEntry = {
        id,
        uid: `wk_${id}`,
        calendar_id: input.calendar_id ?? defaultCalId(),
        title: input.title,
        body: input.body ?? null,
        raw_input: input.raw_input ?? null,
        project: input.project ?? null,
        status: input.status ?? (input.actor === "ai" ? "draft" : "confirmed"),
        actor: input.actor,
        source: input.source,
        started_at: input.started_at,
        ended_at: input.ended_at,
        tags: input.tags ?? [],
        evidence: input.evidence ?? [],
        created_at: ts,
        updated_at: ts,
      };
      devStore.push(entry);
      return { ...entry } as unknown as T;
    }
    case "update_work_entry": {
      const e = devStore.find((x) => x.id === a.id)!;
      Object.assign(e, a.patch, { updated_at: new Date().toISOString() });
      return { ...e } as unknown as T;
    }
    case "confirm_work_entry":
    case "archive_work_entry": {
      const e = devStore.find((x) => x.id === a.id)!;
      e.status = cmd === "confirm_work_entry" ? "confirmed" : "archived";
      e.updated_at = new Date().toISOString();
      return { ...e } as unknown as T;
    }
    case "delete_work_entry": {
      const i = devStore.findIndex((x) => x.id === a.id);
      if (i >= 0) devStore.splice(i, 1);
      return 1 as unknown as T;
    }

    case "list_work_calendars":
      return devCalendars.map((c) => ({ ...c })) as unknown as T;
    case "create_work_calendar": {
      const input = a.input as NewWorkCalendar;
      if (input.is_default) devCalendars.forEach((c) => (c.is_default = false));
      const cal: WorkCalendar = {
        id: ++calSeq,
        uid: `c${calSeq}`,
        name: input.name.trim(),
        color: input.color ?? null,
        is_default: input.is_default ?? false,
        created_at: "",
        updated_at: "",
      };
      devCalendars.push(cal);
      return { ...cal } as unknown as T;
    }
    case "update_work_calendar": {
      const c = devCalendars.find((x) => x.id === a.id)!;
      const patch = a.patch as WorkCalendarPatch;
      if (patch.name != null) c.name = patch.name.trim();
      if (patch.color != null) c.color = patch.color;
      return { ...c } as unknown as T;
    }
    case "set_default_calendar": {
      devCalendars.forEach((c) => (c.is_default = c.id === a.id));
      return { ...devCalendars.find((c) => c.id === a.id)! } as unknown as T;
    }
    case "delete_work_calendar": {
      const c = devCalendars.find((x) => x.id === a.id);
      if (!c) throw new Error("calendar not found");
      if (c.is_default) throw new Error("cannot delete the default calendar");
      // 级联删组内 entries
      for (let i = devStore.length - 1; i >= 0; i--) {
        if (devStore[i].calendar_id === a.id) devStore.splice(i, 1);
      }
      devCalendars.splice(devCalendars.indexOf(c), 1);
      return 1 as unknown as T;
    }

    case "export_report_source": {
      const start = a.start as string;
      const end = a.end as string;
      const rows = devStore
        .filter((e) => e.status === "confirmed" && e.started_at >= start && e.started_at <= end)
        .sort((x, y) => x.started_at.localeCompare(y.started_at));
      if (a.format === "json") {
        return JSON.stringify(rows.map((e) => ({ title: e.title, project: e.project, status: e.status, actor: e.actor, source: e.source, started_at: e.started_at, ended_at: e.ended_at, tags: e.tags, evidence: e.evidence })), null, 2) as unknown as T;
      }
      return rows.map((e, i) => `${i + 1}. ${e.title}`).join("\n") as unknown as T;
    }
    default:
      throw new Error(`dev invoke: 未实现的 command ${cmd}`);
  }
}
