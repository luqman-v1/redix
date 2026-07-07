# Phase 6: Polish & Cross-Cutting

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement task-by-task.

**Goal:** Loading states, toast notifications, auto-reconnect, and keyboard shortcuts.

---

## Task 21: Loading & Progress States

**Files:**
- Create: `src/lib/stores/loading.ts`
- Create: `src/lib/components/Spinner.svelte`
- Modify: all command-calling components

**Interfaces:**
- `loading` store: `Set<string>` of active operation IDs
- `isLoading(opId: string): boolean`
- `<Spinner />` — animated spinner component

**Steps:**

- [ ] **Step 1: Write loading store**

```ts
// src/lib/stores/loading.ts
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
```

- [ ] **Step 2: Write Spinner component**

```svelte
<!-- src/lib/components/Spinner.svelte -->
<script lang="ts">
  let { size = 16 }: { size?: number } = $props();
</script>

<svg class="animate-spin" width={size} height={size} viewBox="0 0 24 24" fill="none">
  <circle cx="12" cy="12" r="10" stroke="var(--color-muted)" stroke-width="3" opacity="0.3" />
  <path d="M12 2a10 10 0 0 1 10 10" stroke="var(--color-accent)" stroke-width="3" stroke-linecap="round" />
</svg>
```

- [ ] **Step 3: Integrate into key operations**

Wrap invoke calls: `startLoading("scan-keys")` before, `stopLoading("scan-keys")` after.

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: global loading state with spinner component"
```

---

## Task 22: Toast Notifications

**Files:**
- Create: `src/lib/stores/toasts.ts`
- Create: `src/lib/components/Toast.svelte`
- Create: `src/lib/components/ToastContainer.svelte`

**Interfaces:**
- `addToast(message, type: 'success' | 'error' | 'warning', duration?: number)`
- `<ToastContainer />` — renders all active toasts, auto-dismiss

**Steps:**

- [ ] **Step 1: Write toast store**

```ts
// src/lib/stores/toasts.ts
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
      update(list => [...list, { id, message, type, duration }]);
      setTimeout(() => {
        update(list => list.filter(t => t.id !== id));
      }, duration);
    },
    remove(id: string) {
      update(list => list.filter(t => t.id !== id));
    },
  };
}

export const toasts = createToastStore();
```

- [ ] **Step 2: Write Toast + ToastContainer**

```svelte
<!-- src/lib/components/Toast.svelte -->
<script lang="ts">
  import type { Toast } from "$lib/stores/toasts";
  import { toasts } from "$lib/stores/toasts";

  let { toast }: { toast: Toast } = $props();

  const colors = {
    success: "var(--color-success)",
    error: "var(--color-error)",
    warning: "var(--color-warning)",
  };
</script>

<div class="flex items-center justify-between rounded-lg px-4 py-3 shadow-lg min-w-[280px]"
  style:background-color="var(--color-surface)" style:border-left="4px solid {colors[toast.type]}">
  <span class="text-sm" style:color="var(--color-fg)">{toast.message}</span>
  <button onclick={() => toasts.remove(toast.id)} class="ml-3 text-sm" style:color="var(--color-muted)">×</button>
</div>
```

```svelte
<!-- src/lib/components/ToastContainer.svelte -->
<script lang="ts">
  import { toasts } from "$lib/stores/toasts";
  import Toast from "./Toast.svelte";
</script>

<div class="fixed top-4 right-4 z-[100] flex flex-col gap-2">
  {#each $toasts as toast (toast.id)}
    <Toast {toast} />
  {/each}
</div>
```

- [ ] **Step 3: Add ToastContainer to +page.svelte**

```svelte
<ToastContainer />
```

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: toast notification system"
```

---

## Task 23: Auto-Reconnect

**Files:**
- Modify: `src-tauri/src/commands/connections.rs` (add reconnect logic)
- Modify: `src/lib/stores/connections.ts` (reconnect on error)

**Steps:**

- [ ] **Step 1: Add reconnect on connection error in frontend**

```ts
// In connections store, wrap invoke calls with retry
async function withReconnect<T>(connId: string, fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e: any) {
    if (e.toString().includes("connection") || e.toString().includes("refused")) {
      toasts.add("Connection lost, reconnecting...", "warning");
      await invoke("reconnect", { connectionId: connId });
      return await fn();
    }
    throw e;
  }
}
```

- [ ] **Step 2: Add reconnect Tauri command**

```rust
#[tauri::command]
pub async fn reconnect(
    connection_id: String,
    store: State<ConnectionStore>,
    manager: State<ConnectionManager>,
) -> Result<(), String> {
    // Get config, create new client, replace in manager
    let configs = store.load().map_err(|e| e.to_string())?;
    let config = configs.into_iter().find(|c| c.id == connection_id)
        .ok_or("Config not found")?;
    let mut client = StandaloneClient::new(config);
    client.connect().await.map_err(|e| e.to_string())?;
    let mut map = manager.lock().await;
    map.insert(connection_id, Box::new(client));
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add . && git commit -m "feat: auto-reconnect on connection drop"
```

---

## Task 24: Keyboard Shortcuts

**Files:**
- Create: `src/lib/utils/shortcuts.ts`
- Modify: `src/routes/+page.svelte`

**Steps:**

- [ ] **Step 1: Implement global keyboard handler**

```ts
// src/lib/utils/shortcuts.ts
export interface Shortcut {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  action: () => void;
  description: string;
}

export function registerShortcuts(shortcuts: Shortcut[]) {
  function handler(e: KeyboardEvent) {
    for (const s of shortcuts) {
      const ctrlMatch = s.ctrl ? (e.ctrlKey || e.metaKey) : true;
      const shiftMatch = s.shift ? e.shiftKey : true;
      if (e.key === s.key && ctrlMatch && shiftMatch) {
        e.preventDefault();
        s.action();
        return;
      }
    }
  }

  window.addEventListener("keydown", handler);
  return () => window.removeEventListener("keydown", handler);
}
```

- [ ] **Step 2: Register shortcuts in +page.svelte**

```ts
import { registerShortcuts } from "$lib/utils/shortcuts";

onMount(() => {
  connections.load();
  return registerShortcuts([
    { key: "n", ctrl: true, action: () => { showNewConnection = true; }, description: "New connection" },
    { key: "k", ctrl: true, action: () => { focusConsole(); }, description: "Focus console" },
    { key: "f", ctrl: true, action: () => { focusSearch(); }, description: "Focus search" },
    { key: "t", ctrl: true, action: () => { theme.toggle(); }, description: "Toggle theme" },
  ]);
});
```

- [ ] **Step 3: Commit**

```bash
git add . && git commit -m "feat: global keyboard shortcuts"
```
