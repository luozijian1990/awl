// WorkEntry 类型镜像 worklog-core 的 serde 形状（snake_case 字段、lowercase 枚举、
// source 序列化成裸字符串），这样 Section 7 后续接 `list_work_entries` 时零改动。

export type Status = "draft" | "confirmed" | "archived";
export type Actor = "ai" | "human";

export interface WorkEntry {
  id: number;
  uid: string;
  calendar_id: number;
  title: string;
  body: string | null;
  raw_input: string | null;
  project: string | null;
  status: Status;
  actor: Actor;
  source: string;
  started_at: string;
  ended_at: string;
  tags: string[];
  evidence: string[];
  created_at: string;
  updated_at: string;
}

const pad = (n: number) => n.toString().padStart(2, "0");

/** "10:00 – 11:00" 本地时间。 */
export function clockRange(startIso: string, endIso: string): string {
  const s = new Date(startIso);
  const e = new Date(endIso);
  return `${pad(s.getHours())}:${pad(s.getMinutes())}–${pad(e.getHours())}:${pad(e.getMinutes())}`;
}

/** 时长 "1h" / "30m" / "1h30m"。 */
export function duration(startIso: string, endIso: string): string {
  const mins = Math.round((+new Date(endIso) - +new Date(startIso)) / 60000);
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  if (h && m) return `${h}h${m}m`;
  if (h) return `${h}h`;
  return `${m}m`;
}
