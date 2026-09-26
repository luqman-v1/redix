import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig } from "$lib/types/connection";
import { toasts } from "$lib/stores/toasts";
import type { Tab } from "$lib/stores/tabs";

export interface KeyMeta {
  type: string;
  ttl: number;
}

export async function fetchKeyMeta(connectionId: string, key: string): Promise<KeyMeta> {
  return invoke<KeyMeta>("get_key_meta", { connectionId, key });
}

export async function selectKey(
  active: ConnectionConfig | null,
  tabs: Tab[],
  activeIndex: number,
  key: string,
  open: (tabs: Tab[], activeIndex: number, key: string, type: string, ttl: number) => { tabs: Tab[]; activeIndex: number },
): Promise<{ tabs: Tab[]; activeIndex: number } | null> {
  if (!active) return null;
  try {
    const meta = await fetchKeyMeta(active.id, key);
    return open(tabs, activeIndex, key, meta.type, meta.ttl);
  } catch (e) {
    toasts.add(String(e), "error");
    return null;
  }
}

export async function refreshKeyMeta(
  active: ConnectionConfig | null,
  key: string | null,
): Promise<KeyMeta | null> {
  if (!active || !key) return null;
  try {
    return await fetchKeyMeta(active.id, key);
  } catch (e) {
    toasts.add("Failed to refresh key: " + String(e), "error");
    return null;
  }
}

export async function renameKey(active: ConnectionConfig | null, oldName: string | null, newName: string): Promise<boolean> {
  if (!active || !oldName) return false;
  try {
    await invoke("rename_key", { connectionId: active.id, oldName, newName });
    return true;
  } catch (e) {
    toasts.add(String(e), "error");
    throw e;
  }
}

export async function setKeyTtl(
  active: ConnectionConfig | null,
  key: string | null,
  ttlStr: string,
): Promise<number | null> {
  if (!active || !key) return null;
  const parsed = parseInt(ttlStr, 10);
  if (isNaN(parsed)) throw new Error("Invalid TTL number");
  try {
    await invoke("set_key_ttl", { connectionId: active.id, key, ttl: parsed });
    const newTtl = await invoke<number>("get_key_ttl", { connectionId: active.id, key });
    toasts.add("TTL updated successfully", "success");
    return newTtl;
  } catch (e) {
    toasts.add(String(e), "error");
    throw e;
  }
}

export async function deleteKey(active: ConnectionConfig | null, key: string | null): Promise<boolean> {
  if (!active || !key) return false;
  try {
    await invoke("delete_key", { connectionId: active.id, key });
    return true;
  } catch (e) {
    toasts.add(String(e), "error");
    return false;
  }
}

export async function switchDatabase(
  active: ConnectionConfig | null,
  newDb: number,
  save: (config: ConnectionConfig) => Promise<ConnectionConfig>,
  setActive: (config: ConnectionConfig | null) => void,
): Promise<boolean> {
  if (!active || active.db === newDb) return false;
  try {
    const updated = { ...active, db: newDb };
    await save(updated);
    await invoke("reconnect", { connectionId: active.id });
    setActive(updated);
    return true;
  } catch (err) {
    toasts.add(String(err), "error");
    return false;
  }
}
