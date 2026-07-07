import { writable } from "svelte/store";

export interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "warning";
  duration: number;
}

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);
  return {
    subscribe,
    add(message: string, type: Toast["type"] = "success", duration = 4000) {
      const id = crypto.randomUUID();
      update((list) => [...list, { id, message, type, duration }]);
      setTimeout(
        () => update((list) => list.filter((t) => t.id !== id)),
        duration,
      );
    },
    remove(id: string) {
      update((list) => list.filter((t) => t.id !== id));
    },
  };
}

export const toasts = createToastStore();
