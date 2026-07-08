import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig } from "$lib/types/connection";
import { toasts } from "./toasts";

function createConnectionStore() {
  const { subscribe, set, update: _update } = writable<ConnectionConfig[]>([]);

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
      _update((list) => [...list, created]);
      return created;
    },
    async save(config: ConnectionConfig) {
      const updated = await invoke<ConnectionConfig>("update_connection", { config });
      _update((list) => list.map((c) => (c.id === updated.id ? updated : c)));
      return updated;
    },
    async remove(id: string) {
      await invoke("delete_connection", { id });
      _update((list) => list.filter((c) => c.id !== id));
    },
  };
}

export const connections = createConnectionStore();
export const activeConnection = writable<ConnectionConfig | null>(null);
export const connectedId = writable<string | null>(null);

export async function connectToServer(config: ConnectionConfig): Promise<void> {
  await invoke("connect_to_server", { connectionId: config.id });
  activeConnection.set(config);
  connectedId.set(config.id);
}

export async function disconnectFromServer(connectionId: string): Promise<void> {
  await invoke("disconnect_server", { connectionId });
  connectedId.set(null);
}

export async function withReconnect<T>(connId: string, fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.toString() : String(e);
    if (msg.includes("connection") || msg.includes("refused")) {
      toasts.add("Connection lost, reconnecting...", "warning");
      await invoke("reconnect", { connectionId: connId });
      return await fn();
    }
    throw e;
  }
}
