# Phase 3: Key Browser

> **For agentic workers:** Use superpowers:subagent-driven-development or superpowers:executing-plans to implement task-by-task.

**Goal:** Folder-style key browser with SCAN-based loading, tree view, search/filter, and key operations.

**Tech Stack:** Svelte 5, Tauri commands wrapping RedisClient trait

---

## Task 13: Key Scanning Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/keys.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `scan_keys(connection_id, cursor, count, pattern) -> { cursor, keys }`
- Produces: `get_key_type(connection_id, key) -> string`
- Produces: `get_key_ttl(connection_id, key) -> i64`

**Steps:**

- [ ] **Step 1: Implement key commands**

```rust
// src-tauri/src/commands/keys.rs
use crate::config::ConnectionStore;
use crate::redis::{StandaloneClient, RedisClient};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
pub struct ScanResult {
    pub cursor: u64,
    pub keys: Vec<String>,
}

// pongolong: ConnectionManager holds active connections
pub type ConnectionManager = Arc<Mutex<std::collections::HashMap<String, Box<dyn RedisClient>>>>;

#[tauri::command]
pub async fn scan_keys(
    connection_id: String,
    cursor: u64,
    count: u64,
    pattern: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<ScanResult, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    let (new_cursor, keys) = client.scan_keys(cursor, count, &pattern).await.map_err(|e| e.to_string())?;
    Ok(ScanResult { cursor: new_cursor, keys })
}

#[tauri::command]
pub async fn get_key_type(
    connection_id: String,
    key: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<String, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    client.get_type(&key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_key_ttl(
    connection_id: String,
    key: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    client.get_ttl(&key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_key(
    connection_id: String,
    key: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    client.del(&[&key]).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_key(
    connection_id: String,
    old_name: String,
    new_name: String,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map.get(&connection_id).ok_or("Not connected")?;
    client.rename(&old_name, &new_name).await.map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Register commands in main.rs**

```rust
// Add to invoke_handler
commands::keys::scan_keys,
commands::keys::get_key_type,
commands::keys::get_key_ttl,
commands::keys::delete_key,
commands::keys::rename_key,
```

- [ ] **Step 3: Add ConnectionManager state**

```rust
// In main()
.manage(Arc::new(Mutex::new(std::collections::HashMap::new())) as ConnectionManager)
```

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: key scanning tauri commands"
```

---

## Task 14: Tree View Component (Frontend)

**Files:**
- Create: `src/lib/utils/tree-builder.ts`
- Create: `src/lib/components/KeyTree.svelte`
- Create: `src/lib/components/TreeNode.svelte`

**Interfaces:**
- Produces: `buildTree(keys: string[], separator: string) -> TreeNode[]`
- Produces: `TreeNode` type: `{ name: string, path: string, children: TreeNode[], isLeaf: boolean, count?: number }`
- Produces: `<KeyTree>` — renders full tree with lazy expand
- Produces: `<TreeNode>` — recursive folder/leaf node

**Steps:**

- [ ] **Step 1: Write tree builder**

```ts
// src/lib/utils/tree-builder.ts
export interface TreeNode {
  name: string;
  path: string;
  children: TreeNode[];
  isLeaf: boolean;
  count: number;
}

export function buildTree(keys: string[], separator: string = ":"): TreeNode[] {
  const root: Map<string, TreeNode> = new Map();

  for (const key of keys) {
    const parts = key.split(separator);
    let current = root;
    let path = "";

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      path = path ? `${path}${separator}${part}` : part;
      const isLeaf = i === parts.length - 1;

      if (!current.has(part)) {
        current.set(part, {
          name: part,
          path,
          children: [],
          isLeaf,
          count: isLeaf ? 0 : 0,
        });
      }

      const node = current.get(part)!;
      if (isLeaf) {
        node.isLeaf = true;
      }

      if (i < parts.length - 1) {
        // Build children map for next level
        const childMap = new Map(node.children.map(c => [c.name, c]));
        // Continue to next level
        current = childMap;
        // Re-assign children after potential modifications
        node.children = Array.from(current.values());
      }
    }
  }

  const sortNodes = (nodes: TreeNode[]): TreeNode[] => {
    return nodes
      .sort((a, b) => {
        if (a.isLeaf !== b.isLeaf) return a.isLeaf ? 1 : -1;
        return a.name.localeCompare(b.name);
      })
      .map(n => ({ ...n, children: sortNodes(n.children), count: countLeaves(n) }));
  };

  return sortNodes(Array.from(root.values()));
}

function countLeaves(node: TreeNode): number {
  if (node.isLeaf) return 1;
  return node.children.reduce((sum, child) => sum + countLeaves(child), 0);
}
```

- [ ] **Step 2: Write TreeNode component**

```svelte
<!-- src/lib/components/TreeNode.svelte -->
<script lang="ts">
  import type { TreeNode } from "$lib/utils/tree-builder";

  let {
    node,
    depth = 0,
    onselect,
  }: {
    node: TreeNode;
    depth?: number;
    onselect: (key: string) => void;
  } = $props();

  let expanded = $state(false);
</script>

{#if node.isLeaf}
  <button
    class="flex items-center w-full text-left px-2 py-1 text-sm rounded hover:opacity-80"
    style:padding-left="{depth * 16 + 8}px"
    style:color="var(--color-fg)"
    onclick={() => onselect(node.path)}
  >
    <span class="mr-2">🔑</span>
    {node.name}
  </button>
{:else}
  <div>
    <button
      class="flex items-center w-full text-left px-2 py-1 text-sm rounded hover:opacity-80"
      style:padding-left="{depth * 16 + 8}px"
      style:color="var(--color-fg)"
      onclick={() => (expanded = !expanded)}
    >
      <span class="mr-1 text-xs" style:color="var(--color-muted)">
        {expanded ? "▼" : "▶"}
      </span>
      <span class="mr-2">📁</span>
      {node.name}
      <span class="ml-auto text-xs" style:color="var(--color-muted)">{node.count}</span>
    </button>
    {#if expanded}
      {#each node.children as child (child.name)}
        <svelte:self node={child} depth={depth + 1} {onselect} />
      {/each}
    {/if}
  </div>
{/if}
```

- [ ] **Step 3: Write KeyTree component**

```svelte
<!-- src/lib/components/KeyTree.svelte -->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { buildTree } from "$lib/utils/tree-builder";
  import type { TreeNode } from "$lib/utils/tree-builder";
  import TreeNodeComponent from "./TreeNode.svelte";

  let {
    connectionId,
    separator = ":",
    onselect,
  }: {
    connectionId: string;
    separator?: string;
    onselect: (key: string) => void;
  } = $props();

  let keys = $state<string[]>([]);
  let tree = $state<TreeNode[]>([]);
  let loading = $state(false);
  let searchPattern = $state("*");

  async function loadKeys() {
    loading = true;
    keys = [];
    let cursor = 0;

    do {
      const result = await invoke<{ cursor: number; keys: string[] }>("scan_keys", {
        connectionId,
        cursor,
        count: 1000,
        pattern: searchPattern,
      });
      keys = [...keys, ...result.keys];
      cursor = result.cursor;
    } while (cursor !== 0);

    tree = buildTree(keys, separator);
    loading = false;
  }

  // Load on connection change
  $effect(() => {
    if (connectionId) loadKeys();
  });
</script>

<div class="p-2">
  <div class="flex gap-2 mb-2">
    <input
      bind:value={searchPattern}
      placeholder="Filter: *"
      class="flex-1 rounded px-2 py-1 text-sm"
      style:background-color="var(--color-bg)"
      style:color="var(--color-fg)"
      style:border="1px solid var(--color-border)"
      onkeydown={(e) => e.key === "Enter" && loadKeys()}
    />
    <button
      onclick={loadKeys}
      class="rounded px-2 py-1 text-sm hover:opacity-80"
      style:background-color="var(--color-surface)"
      style:color="var(--color-fg)"
    >🔄</button>
  </div>

  {#if loading}
    <p class="text-sm" style:color="var(--color-muted)">Loading keys...</p>
  {:else if tree.length === 0}
    <p class="text-sm" style:color="var(--color-muted)">No keys found</p>
  {:else}
    <div class="overflow-auto">
      {#each tree as node (node.name)}
        <TreeNodeComponent {node} {onselect} />
      {/each}
    </div>
  {/if}
</div>
```

- [ ] **Step 4: Wire into page sidebar**

```svelte
<!-- In +page.svelte sidebar snippet, after ConnectionList -->
{#if $activeConnection}
  <KeyTree
    connectionId={$activeConnection.id}
    separator={$activeConnection.key_separator}
    onselect={(key) => { /* Phase 4: show value */ }}
  />
{/if}
```

- [ ] **Step 5: Verify**

Connect to Redis → keys appear in tree → click folder to expand → filter works.

- [ ] **Step 6: Commit**

```bash
git add . && git commit -m "feat: folder-style key tree with SCAN-based loading"
```
