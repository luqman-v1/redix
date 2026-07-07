import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig } from "$lib/types/connection";

function createConnectionStore() {
  const { subscribe, set, update } = writable<ConnectionConfig[]>([]);

  return {
    subscribe,
    async load() {
      const list = await invoke<ConnectionConfig[]>("get_connections");
      set(list);
    },
    async add(config: Omit<ConnectionConfig, "id">) {
      const withId: ConnectionConfig = {
        ...config,
        id: crypto.randomUUID(),
      };
      const created = await invoke<ConnectionConfig>("add_connection", { config: withId });
      update((list) => [...list, created]);
      return created;
    },
    async update(config: ConnectionConfig) {
      const updated = await invoke<ConnectionConfig>("update_connection", { config });
      update((list) => list.map((c) => (c.id === updated.id ? updated : c)));
      return updated;
    },
    async remove(id: string) {
      await invoke("delete_connection", { id });
      update((list) => list.filter((c) => c.id !== id));
    },
  };
}

export const connections = createConnectionStore();
export const activeConnection = writable<ConnectionConfig | null>(null);
