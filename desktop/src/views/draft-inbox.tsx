import { signal } from "@preact/signals";
import { entries, loading, errorMsg, bulkConfirm, bulkArchive, bulkDelete } from "../store";
import { selected, selectedCount, clearSelected, isAllSelected, setSelection } from "../selection";
import { CheckIcon, ArchiveIcon, PlusIcon } from "../icons";
import { EntryRow } from "./entry-row";
import { openForm } from "./entry-form";

// 删除前的二次确认（不可撤销）。
const confirmingDelete = signal(false);

function selectedIds(): number[] {
  return [...selected.value];
}

function reset(): void {
  clearSelected();
  confirmingDelete.value = false;
}

export function DraftInbox() {
  const list = entries.value;
  const ids = list.map((e) => e.id);
  const allSel = isAllSelected(ids);

  return (
    <>
      <header class="topbar">
        <div>
          <h1 class="topbar__title">Drafts</h1>
          <p class="topbar__sub">{list.length} 条待复核 · AI 记录默认进收件箱</p>
        </div>
        <div class="topbar__tools">
          {list.length > 0 && (
            <button class="linkbtn" onClick={() => (allSel ? clearSelected() : setSelection(ids))}>
              {allSel ? "取消全选" : "全选"}
            </button>
          )}
          <button class="newbtn" title="新建工作项 (⌘N)" onClick={() => openForm()}>
            <PlusIcon size={15} />
            <span>新建</span>
          </button>
        </div>
      </header>

      {errorMsg.value && <div class="banner banner--error">{errorMsg.value}</div>}

      <div class="list">
        {loading.value && list.length === 0 ? (
          <div class="empty">载入中…</div>
        ) : list.length === 0 ? (
          <div class="empty">
            <div class="empty__title">收件箱已清空</div>
            <div class="empty__sub">AI 记录的草稿会出现在这里，等你复核确认。</div>
          </div>
        ) : (
          list.map((e, i) => (
            <div class="list__item" key={e.id} style={{ animationDelay: `${i * 40}ms` }}>
              <EntryRow entry={e} />
            </div>
          ))
        )}
      </div>

      {selectedCount.value > 0 && (
        <div class="bulkbar">
          {confirmingDelete.value ? (
            <>
              <span class="bulkbar__count">删除 {selectedCount.value} 项？不可撤销</span>
              <div class="bulkbar__actions">
                <button class="bulk bulk--danger" onClick={() => void bulkDelete(selectedIds()).then(reset)}>
                  确认删除
                </button>
                <button class="bulk" onClick={() => (confirmingDelete.value = false)}>
                  返回
                </button>
              </div>
            </>
          ) : (
            <>
              <span class="bulkbar__count">{selectedCount.value} 项已选</span>
              <div class="bulkbar__actions">
                <button class="bulk bulk--primary" onClick={() => void bulkConfirm(selectedIds()).then(reset)}>
                  <CheckIcon size={14} /> 确认
                </button>
                <button class="bulk" onClick={() => void bulkArchive(selectedIds()).then(reset)}>
                  <ArchiveIcon size={14} /> 归档
                </button>
                <button class="bulk bulk--danger" onClick={() => (confirmingDelete.value = true)}>
                  删除
                </button>
              </div>
              <button class="bulkbar__clear" onClick={reset}>
                取消
              </button>
            </>
          )}
        </div>
      )}
    </>
  );
}
