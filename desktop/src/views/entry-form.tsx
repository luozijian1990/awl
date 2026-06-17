import { signal } from "@preact/signals";
import type { Status, WorkEntry } from "../data";
import { createEntry, updateEntry, calendars, defaultCalendarId } from "../store";

// create/edit 共用一个 sheet（task 7.3）。
const open = signal(false);
const editing = signal<WorkEntry | null>(null);
const actor = signal<"human" | "ai">("human");
const saving = signal(false);
const formError = signal<string | null>(null);

/** 由 `+` 按钮（新建）或行内铅笔（编辑）调用。 */
export function openForm(entry?: WorkEntry): void {
  editing.value = entry ?? null;
  actor.value = entry?.actor ?? "human";
  formError.value = null;
  open.value = true;
}

function closeForm(): void {
  open.value = false;
}

// ISO → datetime-local 输入需要的 "YYYY-MM-DDTHH:mm"
function toLocalInput(iso: string): string {
  const d = new Date(iso);
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function parseList(v: string, sep: "," | "\n"): string[] {
  return v
    .split(sep)
    .map((s) => s.trim())
    .filter(Boolean);
}

async function onSubmit(ev: Event): Promise<void> {
  ev.preventDefault();
  const fd = new FormData(ev.currentTarget as HTMLFormElement);
  const title = ((fd.get("title") as string) ?? "").trim();
  const startLocal = (fd.get("start") as string) ?? "";
  const endLocal = (fd.get("end") as string) ?? "";

  if (!title) {
    formError.value = "标题不能为空";
    return;
  }
  const started_at = startLocal ? new Date(startLocal).toISOString() : new Date().toISOString();
  const ended_at = endLocal ? new Date(endLocal).toISOString() : started_at;
  if (ended_at <= started_at) {
    formError.value = "结束时间必须晚于开始时间";
    return;
  }

  const status = fd.get("status") as Status;
  const calendarId = Number(fd.get("calendar")) || undefined;
  const body = ((fd.get("body") as string) ?? "").trim() || null;
  const tags = parseList((fd.get("tags") as string) ?? "", ",");
  const evidence = parseList((fd.get("evidence") as string) ?? "", "\n");

  saving.value = true;
  formError.value = null;
  try {
    const e = editing.value;
    if (e) {
      await updateEntry(e.id, { title, body, status, started_at, ended_at, tags, evidence, calendar_id: calendarId });
    } else {
      const source = ((fd.get("source") as string) ?? "").trim() || (actor.value === "human" ? "desktop" : "claude");
      await createEntry({ title, body, status, actor: actor.value, source, started_at, ended_at, tags, evidence, calendar_id: calendarId });
    }
    open.value = false;
  } catch (err) {
    formError.value = String(err);
  } finally {
    saving.value = false;
  }
}

export function EntryForm() {
  if (!open.value) return null;

  const e = editing.value;
  const isEdit = !!e;
  // 本人添加默认「已确认」；AI 默认「草稿」，对齐 core 规则。
  const defaultStatus: Status = e?.status ?? (actor.value === "human" ? "confirmed" : "draft");
  const calId = e?.calendar_id ?? defaultCalendarId();

  return (
    <div class="sheet-backdrop" onClick={closeForm}>
      <form class="sheet" onClick={(ev) => ev.stopPropagation()} onSubmit={onSubmit}>
        <div class="sheet__head">
          <h2 class="sheet__title">{isEdit ? "编辑工作项" : "新建工作项"}</h2>
          {!isEdit && (
            <div class="actor-toggle">
              <button
                type="button"
                class={`actor ${actor.value === "human" ? "actor--on" : ""}`}
                onClick={() => (actor.value = "human")}
              >
                本人
              </button>
              <button
                type="button"
                class={`actor ${actor.value === "ai" ? "actor--on" : ""}`}
                onClick={() => (actor.value = "ai")}
              >
                AI
              </button>
            </div>
          )}
        </div>

        <div class="sheet__body">
          {formError.value && <div class="banner banner--error">{formError.value}</div>}

          <label class="field">
            <span class="field__label">标题</span>
            <input class="input input--lg" name="title" placeholder="动作 + 结果 + 价值" value={e?.title ?? ""} autofocus />
          </label>

          <label class="field">
            <span class="field__label">描述（可选）</span>
            <textarea class="input textarea" name="body" rows={3} value={e?.body ?? ""} />
          </label>

          <label class="field">
            <span class="field__label">日历组</span>
            <select class="input select" name="calendar" value={calId != null ? String(calId) : ""}>
              {calendars.value.map((c) => (
                <option value={String(c.id)}>{c.name}</option>
              ))}
            </select>
          </label>

          <div class="field-row">
            <label class="field">
              <span class="field__label">开始</span>
              <input class="input" type="datetime-local" name="start" value={e ? toLocalInput(e.started_at) : ""} />
            </label>
            <label class="field">
              <span class="field__label">结束</span>
              <input class="input" type="datetime-local" name="end" value={e ? toLocalInput(e.ended_at) : ""} />
            </label>
          </div>

          <div class="field-row">
            <label class="field">
              <span class="field__label">状态</span>
              <select class="input select" name="status" value={defaultStatus}>
                <option value="draft">草稿</option>
                <option value="confirmed">已确认</option>
                <option value="archived">已归档</option>
              </select>
            </label>
            <label class="field">
              <span class="field__label">来源</span>
              <input
                class="input"
                name="source"
                value={e?.source ?? (actor.value === "human" ? "desktop" : "claude")}
                disabled={isEdit}
              />
            </label>
          </div>

          <label class="field">
            <span class="field__label">标签</span>
            <input class="input" name="tags" placeholder="用逗号分隔，如 rust, docs" value={e?.tags.join(", ") ?? ""} />
          </label>

          <label class="field">
            <span class="field__label">证据链接</span>
            <textarea class="input textarea" name="evidence" rows={2} placeholder="文件路径或 URL，每行一个" value={e?.evidence.join("\n") ?? ""} />
          </label>
        </div>

        <div class="sheet__foot">
          <button type="button" class="btn" onClick={closeForm}>
            取消
          </button>
          <button type="submit" class="btn btn--primary" disabled={saving.value}>
            {saving.value ? "保存中…" : isEdit ? "保存" : "添加"}
          </button>
        </div>
      </form>
    </div>
  );
}
