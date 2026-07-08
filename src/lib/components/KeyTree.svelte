<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { untrack } from "svelte";
  import { buildTree } from "$lib/utils/tree-builder";
  import type { TreeNode } from "$lib/utils/tree-builder";
  import TreeNodeComponent from "./TreeNode.svelte";

  interface Props {
    connectionId: string;
    separator: string;
    onselect: (key: string) => void;
  }

  let { connectionId, separator, onselect }: Props = $props();

  const PAGE_SIZE = 500;

  let pattern = $state("*");
  let loading = $state(false);
  let allKeys = $state<string[]>([]);
  let displayedCount = $state(0);
  let tree = $state<TreeNode[]>([]);
  let keyCount = $state(0);
  let error = $state<string | null>(null);

  function updateTree() {
    const slice = allKeys.slice(0, displayedCount);
    keyCount = allKeys.length;
    tree = buildTree(slice, separator);
  }

  function loadMore() {
    displayedCount = Math.min(displayedCount + PAGE_SIZE, allKeys.length);
    updateTree();
  }

  let currentCursor = $state(0);

  async function scanKeys(reset = true) {
    if (loading) return;
    loading = true;
    error = null;
    
    if (reset) {
      tree = [];
      allKeys = [];
      keyCount = 0;
      displayedCount = 0;
      currentCursor = 0;
    }

    try {
      const newKeys: string[] = [];
      let c = currentCursor;
      let iterations = 0;
      
      const isExact = pattern && !pattern.includes('*') && !pattern.includes('?') && !pattern.includes('[');
      
      if (isExact && reset) {
        const type = await invoke<string>("get_key_type", { connectionId, key: pattern });
        if (type !== "none") {
          newKeys.push(pattern);
        }
        c = 0;
      } else {
        do {
          if (iterations > 0 && c === 0) break;
          const result = await invoke<{ cursor: number; keys: string[] }>(
            "scan_keys",
            {
              connectionId,
              cursor: c,
              count: 10000,
              pattern,
            }
          );
          c = result.cursor;
          newKeys.push(...result.keys);
          iterations++;
          // ponytail: removed iteration limit so sparse wildcard searches (e.g. foo:*) 
          // don't abort prematurely on huge DBs. Will scan until PAGE_SIZE found or DB end.
        } while (newKeys.length < PAGE_SIZE && c !== 0);
      }

      currentCursor = c;
      const uniqueKeys = Array.from(new Set([...allKeys, ...newKeys]));
      allKeys = uniqueKeys;
      displayedCount = allKeys.length;
      updateTree();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // auto-scan when connectionId changes
  $effect(() => {
    if (connectionId) {
      untrack(() => {
        scanKeys();
      });
    }
  });
</script>

<div class="key-tree">
  <div class="toolbar">
    <input
      data-key-search
      class="pattern-input"
      type="text"
      bind:value={pattern}
      placeholder="Filter pattern (e.g. user:*)"
      onkeydown={(e) => e.key === "Enter" && scanKeys(true)}
    />
    <button class="refresh-btn" onclick={() => scanKeys(true)} disabled={loading} title="Refresh">
      🔄
    </button>
  </div>

  {#if error}
    <div class="error">{error}</div>
  {/if}

  <div class="tree-scroll">
    {#if loading && tree.length === 0}
      <div class="state-msg">Scanning keys...</div>
    {:else if !loading && keyCount === 0 && !error}
      <div class="state-msg">No keys found</div>
    {:else}
      <div class="tree-list">
        {#each tree as node (node.path)}
          <TreeNodeComponent {node} depth={0} {onselect} />
        {/each}
      </div>
      {#if keyCount > 0 || currentCursor !== 0}
        <div class="key-count">
          Showing {displayedCount} keys
          {#if currentCursor === 0}
             (All loaded)
          {/if}
        </div>
        {#if currentCursor !== 0}
          <button class="load-more-btn" onclick={() => scanKeys(false)} disabled={loading}>
            {loading ? 'Scanning DB...' : 'Scan More Keys'}
          </button>
        {/if}
      {/if}
    {/if}
  </div>
</div>

<style>
  .key-tree {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 0;
    flex: 1;
  }

  .toolbar {
    display: flex;
    gap: 0.375rem;
  }

  .pattern-input {
    flex: 1;
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-surface-input);
    color: var(--color-fg);
    font-size: 0.75rem;
    font-family: inherit;
    outline: none;
  }

  .pattern-input:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .refresh-btn {
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-surface-btn);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.75rem;
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    color: var(--color-error, #e55);
    font-size: 0.75rem;
    padding: 0.25rem 0;
  }

  .tree-scroll {
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .tree-list {
    display: flex;
    flex-direction: column;
  }

  .state-msg {
    color: var(--color-muted);
    font-size: 0.75rem;
    text-align: center;
    padding: 1rem 0;
  }

  .key-count {
    color: var(--color-muted);
    font-size: 0.75rem;
    padding: 0.5rem 0.5rem 0;
    border-top: 1px solid var(--color-border, #333);
    margin-top: 0.5rem;
  }

  .load-more-btn {
    margin: 0.375rem 0.5rem;
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-surface-btn);
    color: var(--color-accent);
    cursor: pointer;
    font-size: 0.75rem;
    font-family: inherit;
    text-align: center;
  }

  .load-more-btn:hover {
    background: var(--color-surface-btn-hover);
  }
</style>
