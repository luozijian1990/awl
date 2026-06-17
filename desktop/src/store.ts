import { signal } from "@preact/signals";
import {
  api,
  type EntryFilter,
  type NewWorkEntry,
  type WorkCalendar,
  type WorkEntryPatch,
} from "./api";
import { clearSelected } from "./selection";
import type { WorkEntry } from "./data";

export type View = "inbox" | "confirmed";

export const view = signal<View>("inbox");
export const entries = signal<WorkEntry[]>([]);
export const loading = signal(false);
export const errorMsg = signal<string | null>(null);

// 日历组：唯一的分组维度（替代旧的 project 字符串）。
export const calendars = signal<WorkCalendar[]>([]);

export function calendarName(id: number): string {
  return calendars.value.find((c) => c.id === id)?.name ?? "未分组";
}

export function defaultCalendarId(): number | undefined {
  return (calendars.value.find((c) => c.is_default) ?? calendars.value[0])?.id;
}

// confirmed 视图过滤：日期范围 + 日历组 + source。inbox 固定 status=draft。
export interface ConfirmedFilters {
  range: "all" | "today" | "week";
  calendarId: number | null;
  source: string | null;
}
export const filters = signal<ConfirmedFilters>({ range: "all", calendarId: null, source: null });

function startOfToday(): Date {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  return d;
}

function rangeBounds(range: ConfirmedFilters["range"]): { start?: string; end?: string } {
  if (range === "all") return {};
  const start = startOfToday();
  if (range === "week") start.setDate(start.getDate() - 6);
  const end = new Date(start);
  end.setDate(end.getDate() + (range === "week" ? 7 : 1));
  return { start: start.toISOString(), end: end.toISOString() };
}

function currentFilter(): EntryFilter {
  if (view.value === "inbox") return { status: "draft" };
  const f = filters.value;
  return { status: "confirmed", calendar_id: f.calendarId, source: f.source, ...rangeBounds(f.range) };
}

export async function loadEntries(): Promise<void> {
  loading.value = true;
  errorMsg.value = null;
  try {
    entries.value = await api.listEntries(currentFilter());
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

export async function loadCalendars(): Promise<void> {
  try {
    calendars.value = await api.listCalendars();
  } catch (e) {
    errorMsg.value = String(e);
  }
}

export function setView(v: View): void {
  if (view.value === v) return;
  clearSelected();
  view.value = v;
  void loadEntries();
}

export function setFilters(next: Partial<ConfirmedFilters>): void {
  filters.value = { ...filters.value, ...next };
  void loadEntries();
}

export async function confirmEntry(id: number): Promise<void> {
  await api.confirmEntry(id);
  await loadEntries();
}

export async function archiveEntry(id: number): Promise<void> {
  await api.archiveEntry(id);
  await loadEntries();
}

export async function deleteEntry(id: number): Promise<void> {
  await api.deleteEntry(id);
  await loadEntries();
}

export async function createEntry(input: NewWorkEntry): Promise<void> {
  await api.createEntry(input);
  await loadEntries();
}

export async function updateEntry(id: number, patch: WorkEntryPatch): Promise<void> {
  await api.updateEntry(id, patch);
  await loadEntries();
}

export async function bulkConfirm(ids: number[]): Promise<void> {
  for (const id of ids) await api.confirmEntry(id);
  await loadEntries();
}

export async function bulkArchive(ids: number[]): Promise<void> {
  for (const id of ids) await api.archiveEntry(id);
  await loadEntries();
}

export async function bulkDelete(ids: number[]): Promise<void> {
  for (const id of ids) await api.deleteEntry(id);
  await loadEntries();
}

// ---- 日历组 CRUD ----
export async function createCalendar(name: string): Promise<void> {
  if (!name.trim()) return;
  await api.createCalendar({ name: name.trim() });
  await loadCalendars();
}

export async function renameCalendar(id: number, name: string): Promise<void> {
  if (!name.trim()) return;
  await api.updateCalendar(id, { name: name.trim() });
  await loadCalendars();
  await loadEntries();
}

export async function setDefaultCalendar(id: number): Promise<void> {
  await api.setDefaultCalendar(id);
  await loadCalendars();
}

/** 级联删除：组 + 组内工作项一起删。 */
export async function deleteCalendar(id: number): Promise<void> {
  await api.deleteCalendar(id);
  if (filters.value.calendarId === id) filters.value = { ...filters.value, calendarId: null };
  await loadCalendars();
  await loadEntries();
}
