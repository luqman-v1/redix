import { writable, derived } from "svelte/store";

const operations = writable<Set<string>>(new Set());

export function startLoading(id: string) {
  operations.update(set => { const s = new Set(set); s.add(id); return s; });
}

export function stopLoading(id: string) {
  operations.update(set => { const s = new Set(set); s.delete(id); return s; });
}

export const isLoading = derived(operations, $ops => (id: string) => $ops.has(id));
export const anyLoading = derived(operations, $ops => $ops.size > 0);
