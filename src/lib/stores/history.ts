import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

function createHistoryStore() {
  const { subscribe, set, update } = writable<string[]>([]);
  return {
    subscribe,
    async load(connectionId: string) {
      const list = await invoke<string[]>("get_history", { connectionId });
      set(list);
    },
    async add(connectionId: string, command: string) {
      await invoke("add_to_history", { connectionId, command });
      update((prev) => [...prev, command]);
    },
  };
}

export const history = createHistoryStore();
