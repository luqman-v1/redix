# Phase 4: Value Display & Edit

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement task-by-task.

**Goal:** Type-aware value display and editing for all 9 Redis data types with beautification.

---

## Task 15: Value Tauri Commands (Rust)

**Files:**
- Create: `src-tauri/src/commands/values.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/main.rs`

**Interfaces:**
- `get_string_value(conn_id, key) -> string`
- `set_string_value(conn_id, key, value) -> ()`
- `get_hash_all(conn_id, key) -> Vec<[string, string]>`
- `set_hash_field(conn_id, key, field, value) -> ()`
- `del_hash_field(conn_id, key, field) -> ()`
- `get_list_range(conn_id, key, start, stop) -> Vec<string>`
- `set_list_value(conn_id, key, index, value) -> ()`
- `get_set_members(conn_id, key) -> Vec<string>`
- `add_set_member(conn_id, key, member) -> ()`
- `del_set_member(conn_id, key, member) -> ()`
- `get_sorted_set_range(conn_id, key, start, stop) -> Vec<{score, member}>`
- `add_sorted_set(conn_id, key, score, member) -> ()`
- `get_stream_range(conn_id, key, start, end, count) -> Vec<{id, fields}>`
- `get_hyperloglog_count(conn_id, key) -> i64`
- `get_geo_members(conn_id, key) -> Vec<{member, lng, lat, score}>`

**Steps:**

- [ ] **Step 1: Implement all value commands**

Each command follows pattern:
```rust
#[tauri::command]
pub async fn get_string_value(
    connection_id: String,
    key: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<String, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    let result = client.execute("GET", &[&key]).await.map_err(|e| e.to_string())?;
    match result {
        RedisValue::String(s) => Ok(s),
        RedisValue::Nil => Ok(String::new()),
        _ => Err("Unexpected type".to_string()),
    }
}
```

Hash commands use `HGETALL`, List uses `LRANGE`, Set uses `SMEMBERS`, Sorted Set uses `ZRANGE WITHSCORES`, Stream uses `XRANGE`, HyperLogLog uses `PFCOUNT`, Geo uses `ZRANGE WITHSCORES` (geo stored as sorted set).

- [ ] **Step 2: Register all commands in main.rs**

- [ ] **Step 3: Build verify**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: value tauri commands for all redis types"
```

---

## Task 16: Value Beautifier Utility (Frontend)

**Files:**
- Create: `src/lib/utils/format-detector.ts`
- Create: `src/lib/utils/beautifier.ts`

**Interfaces:**
- `detectFormat(value: string): 'json' | 'xml' | 'binary' | 'text'`
- `beautify(value: string, format: Format): string`
- `toHex(value: string): string`

**Steps:**

- [ ] **Step 1: Implement format detector + beautifier**

```ts
// src/lib/utils/format-detector.ts
export type DataFormat = "json" | "xml" | "binary" | "text";

export function detectFormat(value: string): DataFormat {
  if (!value) return "text";
  const trimmed = value.trim();

  // JSON
  if ((trimmed.startsWith("{") && trimmed.endsWith("}")) ||
      (trimmed.startsWith("[") && trimmed.endsWith("]"))) {
    try { JSON.parse(trimmed); return "json"; } catch {}
  }

  // XML
  if (trimmed.startsWith("<") && trimmed.endsWith(">")) return "xml";

  // Binary (non-printable chars)
  if (/[\x00-\x08\x0E-\x1F]/.test(value)) return "binary";

  return "text";
}

export function isBinary(value: string): boolean {
  return detectFormat(value) === "binary";
}
```

```ts
// src/lib/utils/beautifier.ts
import { detectFormat, type DataFormat } from "./format-detector";

export function beautify(value: string): { formatted: string; format: DataFormat } {
  const format = detectFormat(value);

  switch (format) {
    case "json":
      return { formatted: JSON.stringify(JSON.parse(value), null, 2), format };
    case "xml":
      return { formatted: formatXml(value), format };
    case "binary":
      return { formatted: toHex(value), format };
    default:
      return { formatted: value, format };
  }
}

function formatXml(xml: string): string {
  let indent = "";
  const tab = "  ";
  return xml.replace(/(>)(<)(\/*)/g, "$1\n$2$3")
    .split("\n")
    .map(line => {
      if (line.match(/^<\/\w/)) indent = indent.substring(tab.length);
      const formatted = indent + line;
      if (line.match(/^<\w[^>]*[^\/]>.*$/)) indent += tab;
      return formatted;
    })
    .join("\n");
}

export function toHex(value: string): string {
  return Array.from(value)
    .map(c => c.charCodeAt(0).toString(16).padStart(2, "0"))
    .join(" ");
}
```

- [ ] **Step 2: Commit**

```bash
git add . && git commit -m "feat: format detection and beautifier utility"
```

---

## Task 17: Type Viewer Components (Frontend)

**Files:**
- Create: `src/lib/components/viewers/StringViewer.svelte`
- Create: `src/lib/components/viewers/HashViewer.svelte`
- Create: `src/lib/components/viewers/ListViewer.svelte`
- Create: `src/lib/components/viewers/SetViewer.svelte`
- Create: `src/lib/components/viewers/SortedSetViewer.svelte`
- Create: `src/lib/components/viewers/StreamViewer.svelte`
- Create: `src/lib/components/viewers/GeoViewer.svelte`
- Create: `src/lib/components/viewers/HyperLogLogViewer.svelte`
- Create: `src/lib/components/viewers/BitmapViewer.svelte`
- Create: `src/lib/components/viewers/ValueViewer.svelte` (router)

**Interfaces:**
- `<ValueViewer type={string} connectionId={string} key={string} />` — dispatches to correct viewer

**Steps:**

- [ ] **Step 1: Write StringViewer**

```svelte
<!-- src/lib/components/viewers/StringViewer.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { beautify } from "$lib/utils/beautifier";

  let { connectionId, key }: { connectionId: string; key: string } = $props();
  let value = $state("");
  let formatted = $state("");
  let format = $state("text");
  let editing = $state(false);
  let editValue = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    value = await invoke("get_string_value", { connectionId, key });
    const result = beautify(value);
    formatted = result.formatted;
    format = result.format;
    loading = false;
  }

  async function save() {
    await invoke("set_string_value", { connectionId, key, value: editValue });
    editing = false;
    await load();
  }

  $effect(() => { if (key) load(); });
</script>

{#if loading}
  <p style:color="var(--color-muted)">Loading...</p>
{:else}
  <div class="flex items-center gap-2 mb-2">
    <span class="text-xs px-2 py-1 rounded" style:background-color="var(--color-surface)" style:color="var(--color-muted)">{format}</span>
    <button onclick={() => { editValue = value; editing = !editing; }}
      class="text-xs px-2 py-1 rounded" style:background-color="var(--color-accent)" style:color="var(--color-bg)">
      {editing ? "Cancel" : "Edit"}
    </button>
    {#if editing}
      <button onclick={save} class="text-xs px-2 py-1 rounded" style:background-color="var(--color-success)" style:color="var(--color-bg)">Save</button>
    {/if}
  </div>

  {#if editing}
    <textarea bind:value={editValue} rows="15"
      class="w-full rounded p-3 font-mono text-sm"
      style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)"></textarea>
  {:else}
    <pre class="rounded p-3 overflow-auto text-sm font-mono max-h-[500px]"
      style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)">{formatted}</pre>
  {/if}
{/if}
```

- [ ] **Step 2: Write HashViewer (table)**

```svelte
<!-- src/lib/components/viewers/HashViewer.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { connectionId, key }: { connectionId: string; key: string } = $props();
  let entries = $state<[string, string][]>([]);
  let filter = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    entries = await invoke("get_hash_all", { connectionId, key });
    loading = false;
  }

  $effect(() => { if (key) load(); });

  let filtered = $derived(
    filter ? entries.filter(([f, v]) => f.includes(filter) || v.includes(filter)) : entries
  );
</script>

<input bind:value={filter} placeholder="Filter fields..."
  class="w-full rounded px-2 py-1 text-sm mb-2"
  style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />

{#if loading}
  <p style:color="var(--color-muted)">Loading...</p>
{:else}
  <table class="w-full text-sm">
    <thead>
      <tr style:border-bottom="1px solid var(--color-border)">
        <th class="text-left p-2" style:color="var(--color-muted)">Field</th>
        <th class="text-left p-2" style:color="var(--color-muted)">Value</th>
      </tr>
    </thead>
    <tbody>
      {#each filtered as [field, val] (field)}
        <tr style:border-bottom="1px solid var(--color-border)">
          <td class="p-2 font-mono" style:color="var(--color-accent)">{field}</td>
          <td class="p-2 font-mono" style:color="var(--color-fg)">{val}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
```

- [ ] **Step 3: Write remaining viewers (pattern: List/Set/SortedSet/Stream/Geo/HLL/Bitmap)**

Each viewer: load data → display in type-appropriate format. List = indexed table, Set = member list, SortedSet = score+member table, Stream = timeline, Geo = coordinate table, HLL = count display, Bitmap = hex view.

- [ ] **Step 4: Write ValueViewer router**

```svelte
<!-- src/lib/components/viewers/ValueViewer.svelte -->
<script lang="ts">
  import StringViewer from "./StringViewer.svelte";
  import HashViewer from "./HashViewer.svelte";
  import ListViewer from "./ListViewer.svelte";
  import SetViewer from "./SetViewer.svelte";
  import SortedSetViewer from "./SortedSetViewer.svelte";
  import StreamViewer from "./StreamViewer.svelte";
  import GeoViewer from "./GeoViewer.svelte";
  import HyperLogLogViewer from "./HyperLogLogViewer.svelte";
  import BitmapViewer from "./BitmapViewer.svelte";

  let { type, connectionId, key }: { type: string; connectionId: string; key: string } = $props();
</script>

{#if type === "string"}
  <StringViewer {connectionId} {key} />
{:else if type === "hash"}
  <HashViewer {connectionId} {key} />
{:else if type === "list"}
  <ListViewer {connectionId} {key} />
{:else if type === "set"}
  <SetViewer {connectionId} {key} />
{:else if type === "zset"}
  <SortedSetViewer {connectionId} {key} />
{:else if type === "stream"}
  <StreamViewer {connectionId} {key} />
{:else if type === "geo"}
  <GeoViewer {connectionId} {key} />
{:else if type === "string" && type === "hyperloglog"}
  <HyperLogLogViewer {connectionId} {key} />
{:else}
  <BitmapViewer {connectionId} {key} />
{/if}
```

- [ ] **Step 5: Wire into main page**

In `+page.svelte` main snippet: when key selected + type loaded, show `<ValueViewer>`.

- [ ] **Step 6: Commit**

```bash
git add . && git commit -m "feat: type-aware value viewers for all redis types"
```
