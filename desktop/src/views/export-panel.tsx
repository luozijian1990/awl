import { signal } from "@preact/signals";
import { api } from "../api";

// 报表源导出面板（7.5）：选日期范围 → JSON / raw Markdown → 预览 → 复制 / 保存。
// 只导出 confirmed entries（由 core report_source 保证）。
type Format = "json" | "markdown";

const open = signal(false);
const startDate = signal("");
const endDate = signal("");
const format = signal<Format>("markdown");
const preview = signal("");
const busy = signal(false);
const error = signal<string | null>(null);
const copied = signal(false);

function fmtDate(d: Date): string {
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function openExport(): void {
  const today = new Date();
  startDate.value = fmtDate(new Date(today.getFullYear(), today.getMonth(), 1));
  endDate.value = fmtDate(today);
  format.value = "markdown";
  error.value = null;
  copied.value = false;
  open.value = true;
  void refresh();
}

function closeExport(): void {
  open.value = false;
}

async function refresh(): Promise<void> {
  busy.value = true;
  error.value = null;
  copied.value = false;
  try {
    const startIso = new Date(`${startDate.value}T00:00:00`).toISOString();
    const endIso = new Date(`${endDate.value}T23:59:59`).toISOString();
    preview.value = await api.exportReportSource(startIso, endIso, format.value);
  } catch (e) {
    error.value = String(e);
    preview.value = "";
  } finally {
    busy.value = false;
  }
}

async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(preview.value);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch {
    // 无剪贴板权限时静默；用户可手动选中预览复制
  }
}

function save(): void {
  const ext = format.value === "json" ? "json" : "md";
  const mime = format.value === "json" ? "application/json" : "text/markdown";
  const blob = new Blob([preview.value], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `worklog-report-${startDate.value}_${endDate.value}.${ext}`;
  a.click();
  URL.revokeObjectURL(url);
}

export function ExportPanel() {
  if (!open.value) return null;

  return (
    <div class="sheet-backdrop" onClick={closeExport}>
      <div class="sheet sheet--wide" onClick={(ev) => ev.stopPropagation()}>
        <div class="sheet__head">
          <h2 class="sheet__title">导出报表源</h2>
          <div class="segmented">
            <button
              class={`seg ${format.value === "markdown" ? "seg--on" : ""}`}
              onClick={() => {
                format.value = "markdown";
                void refresh();
              }}
            >
              Markdown
            </button>
            <button
              class={`seg ${format.value === "json" ? "seg--on" : ""}`}
              onClick={() => {
                format.value = "json";
                void refresh();
              }}
            >
              JSON
            </button>
          </div>
        </div>

        <div class="sheet__body">
          <div class="field-row">
            <label class="field">
              <span class="field__label">开始日期</span>
              <input
                class="input"
                type="date"
                value={startDate.value}
                onChange={(e) => {
                  startDate.value = (e.currentTarget as HTMLInputElement).value;
                  void refresh();
                }}
              />
            </label>
            <label class="field">
              <span class="field__label">结束日期</span>
              <input
                class="input"
                type="date"
                value={endDate.value}
                onChange={(e) => {
                  endDate.value = (e.currentTarget as HTMLInputElement).value;
                  void refresh();
                }}
              />
            </label>
          </div>

          {error.value && <div class="banner banner--error">{error.value}</div>}

          <div class="field">
            <span class="field__label">
              预览 · 仅 confirmed 工作项{format.value === "markdown" ? " · raw 编号列表" : ""}
            </span>
            <pre class="export-preview">{busy.value ? "生成中…" : preview.value || "（该时间范围内没有已确认工作项）"}</pre>
          </div>
        </div>

        <div class="sheet__foot">
          <button class="btn" onClick={closeExport}>
            关闭
          </button>
          <button class="btn" onClick={save} disabled={!preview.value}>
            保存…
          </button>
          <button class="btn btn--primary" onClick={() => void copy()} disabled={!preview.value}>
            {copied.value ? "已复制 ✓" : "复制"}
          </button>
        </div>
      </div>
    </div>
  );
}
