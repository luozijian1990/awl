import { clockRange, duration, type WorkEntry } from "../data";
import { CheckIcon, ArchiveIcon, EditIcon, LinkIcon } from "../icons";
import { isSelected, toggleSelected } from "../selection";
import { archiveEntry, confirmEntry, calendarName } from "../store";
import { openForm } from "./entry-form";

// 两个视图（draft inbox / confirmed list）共用的行。
// 操作按 status 区分：draft 可确认/归档/编辑；confirmed 可归档/编辑。
// selectable 仅在 inbox 为 true（批量操作面向草稿，7.4）。
export function EntryRow({ entry, selectable = true }: { entry: WorkEntry; selectable?: boolean }) {
  const sel = isSelected(entry.id);

  return (
    <div class={`row${sel ? " row--selected" : ""}`}>
      {selectable && (
        <button
          class={`pick${sel ? " pick--on" : ""}`}
          onClick={() => toggleSelected(entry.id)}
          aria-label="选择"
        >
          {sel && <CheckIcon size={11} />}
        </button>
      )}

      <div class="row__main">
        <div class="row__title">{entry.title}</div>
        <div class="row__meta">
          <span class={`src src--${entry.source}`}>{entry.source}</span>
          <span class="dot">·</span>
          <span class="proj">{calendarName(entry.calendar_id)}</span>
          <span class="dot">·</span>
          <span>{clockRange(entry.started_at, entry.ended_at)}</span>
          <span class="muted">· {duration(entry.started_at, entry.ended_at)}</span>
          {entry.tags.map((t) => (
            <span class="tag">{t}</span>
          ))}
          {entry.evidence.length > 0 && (
            <span class="evidence">
              <LinkIcon /> {entry.evidence.length}
            </span>
          )}
        </div>
      </div>

      <div class="row__actions">
        {entry.status === "draft" && (
          <button class="act act--confirm" title="确认" onClick={() => void confirmEntry(entry.id)}>
            <CheckIcon />
          </button>
        )}
        <button class="act" title="归档" onClick={() => void archiveEntry(entry.id)}>
          <ArchiveIcon />
        </button>
        <button class="act" title="编辑" onClick={() => openForm(entry)}>
          <EditIcon />
        </button>
      </div>
    </div>
  );
}
