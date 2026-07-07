# Phase 5: Command Console

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement task-by-task.

**Goal:** Manual Redis command input with syntax highlighting, history, and formatted output.

---

## Task 18: Command Execution Tauri Command (Rust)

**Files:**
- Modify: `src-tauri/src/commands/keys.rs` (or new `commands/console.rs`)
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `execute_command(conn_id, command) -> { result: RedisValue, duration_ms: u64 }`

**Steps:**

- [ ] **Step 1: Implement**

```rust
// src-tauri/src/commands/console.rs
use crate::commands::keys::ConnectionManager;
use crate::redis::RedisValue;
use serde::Serialize;

#[derive(Serialize)]
pub struct CommandResult {
    pub result: RedisValue,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn execute_command(
    connection_id: String,
    command: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<CommandResult, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let cmd = parts[0];
    let args = &parts[1..];

    let start = std::time::Instant::now();
    let result = client.execute(cmd, args).await.map_err(|e| e.to_string())?;
    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(CommandResult { result, duration_ms })
}
```

- [ ] **Step 2: Register in main.rs**

```rust
commands::console::execute_command,
```

- [ ] **Step 3: Commit**

```bash
git add . && git commit -m "feat: execute redis command tauri command"
```

---

## Task 19: Command History Store (Rust + Frontend)

**Files:**
- Create: `src-tauri/src/commands/history.rs`
- Create: `src/lib/stores/history.ts`

**Interfaces:**
- Rust: `get_history(conn_id) -> Vec<string>`, `add_history(conn_id, command)`
- Frontend: `history` store per connection, persist to localStorage

**Steps:**

- [ ] **Step 1: Implement history in Rust**

```rust
// src-tauri/src/commands/history.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type HistoryStore = Arc<Mutex<HashMap<String, Vec<String>>>>;

#[tauri::command]
pub async fn get_history(
    connection_id: String,
    store: tauri::State<'_, HistoryStore>,
) -> Result<Vec<String>, String> {
    let map = store.lock().await;
    Ok(map.get(&connection_id).cloned().unwrap_or_default())
}

#[tauri::command]
pub async fn add_to_history(
    connection_id: String,
    command: String,
    store: tauri::State<'_, HistoryStore>,
) -> Result<(), String> {
    let mut map = store.lock().await;
    let list = map.entry(connection_id).or_insert_with(Vec::new);
    list.push(command);
    if list.len() > 1000 { list.remove(0); }
    Ok(())
}
```

- [ ] **Step 2: Register in main.rs**

```rust
.manage(Arc::new(Mutex::new(HashMap::new())) as HistoryStore)
// + invoke_handler entries
```

- [ ] **Step 3: Write frontend history store**

```ts
// src/lib/stores/history.ts
import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

function createHistoryStore() {
  const { subscribe, set } = writable<string[]>([]);

  return {
    subscribe,
    async load(connectionId: string) {
      const list = await invoke<string[]>("get_history", { connectionId });
      set(list);
    },
    async add(connectionId: string, command: string) {
      await invoke("add_to_history", { connectionId, command });
      set((prev) => [...prev, command]);
    },
  };
}

export const history = createHistoryStore();
```

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: command history store"
```

---

## Task 20: Command Console UI (Frontend)

**Files:**
- Create: `src/lib/components/Console.svelte`

**Interfaces:**
- `<Console connectionId={string} />`
- Features: input with Enter to execute, Shift+Enter for newline, output display, history nav (Up/Down)

**Steps:**

- [ ] **Step 1: Write Console component**

```svelte
<!-- src/lib/components/Console.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { history } from "$lib/stores/history";

  let { connectionId }: { connectionId: string } = $props();

  interface OutputLine {
    command: string;
    result: string;
    duration: number;
    isError: boolean;
  }

  let command = $state("");
  let output = $state<OutputLine[]>([]);
  let historyIndex = $state(-1);
  let loading = $state(false);

  async function execute() {
    const cmd = command.trim();
    if (!cmd) return;

    loading = true;
    try {
      const res = await invoke<{ result: { type: string; value: any }; duration_ms: number }>(
        "execute_command", { connectionId, command: cmd }
      );
      const resultStr = res.result.value?.toString?.() ?? JSON.stringify(res.result);
      output = [...output, {
        command: cmd,
        result: resultStr,
        duration: res.duration_ms,
        isError: res.result.type === "Error",
      }];
      await history.add(connectionId, cmd);
    } catch (e: any) {
      output = [...output, { command: cmd, result: e.toString(), duration: 0, isError: true }];
    }
    command = "";
    historyIndex = -1;
    loading = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      execute();
    } else if (e.key === "ArrowUp" && !command) {
      e.preventDefault();
      const h = $history;
      if (h.length > 0) {
        historyIndex = Math.min(historyIndex + 1, h.length - 1);
        command = h[h.length - 1 - historyIndex];
      }
    } else if (e.key === "ArrowDown" && historyIndex >= 0) {
      e.preventDefault();
      historyIndex -= 1;
      const h = $history;
      command = historyIndex >= 0 ? h[h.length - 1 - historyIndex] : "";
    }
  }

  $effect(() => { if (connectionId) history.load(connectionId); });
</script>

<div class="flex flex-col h-full">
  <!-- Output -->
  <div class="flex-1 overflow-auto p-3 space-y-2 font-mono text-sm">
    {#each output as line, i (i)}
      <div>
        <div style:color="var(--color-accent)">❯ {line.command}</div>
        <div style:color={line.isError ? "var(--color-error)" : "var(--color-fg)"}>
          {line.result}
        </div>
        <div class="text-xs" style:color="var(--color-muted)">{line.duration}ms</div>
      </div>
    {/each}
    {#if loading}
      <div style:color="var(--color-muted)">Executing...</div>
    {/if}
  </div>

  <!-- Input -->
  <div class="border-t p-2" style:border-color="var(--color-border)">
    <div class="flex items-center gap-2">
      <span style:color="var(--color-accent)">❯</span>
      <input
        bind:value={command}
        onkeydown={handleKeydown}
        placeholder="Enter Redis command..."
        class="flex-1 bg-transparent text-sm outline-none font-mono"
        style:color="var(--color-fg)"
        disabled={loading}
      />
    </div>
  </div>
</div>
```

- [ ] **Step 2: Wire into page bottom panel**

```svelte
<!-- In +page.svelte bottom snippet -->
{#if $activeConnection}
  <Console connectionId={$activeConnection.id} />
{:else}
  <p class="text-sm p-4" style:color="var(--color-muted)">Connect to a server to use console</p>
{/if}
```

- [ ] **Step 3: Verify**

Type `PING` → shows `PONG` + duration. Up arrow recalls last command.

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: command console with history and output display"
```
