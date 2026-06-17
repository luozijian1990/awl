import { computed, signal } from "@preact/signals";

// 跨视图共享的选中集合，驱动批量操作条（7.4）。
export const selected = signal<Set<number>>(new Set());
export const selectedCount = computed(() => selected.value.size);

export function isSelected(id: number): boolean {
  return selected.value.has(id);
}

export function toggleSelected(id: number): void {
  const next = new Set(selected.value);
  next.has(id) ? next.delete(id) : next.add(id);
  selected.value = next;
}

export function clearSelected(): void {
  if (selected.value.size) selected.value = new Set();
}

export function isAllSelected(ids: number[]): boolean {
  return ids.length > 0 && ids.every((id) => selected.value.has(id));
}

export function setSelection(ids: number[]): void {
  selected.value = new Set(ids);
}
