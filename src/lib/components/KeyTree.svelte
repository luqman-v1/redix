<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
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

  async function scanKeys() {
    loading = true;
    error = null;
    tree = [];
    allKeys = [];
    keyCount = 0;
    displayedCount = 0;

    try {
      const keys: string[] = [];
      let cursor = "0";

      do {
        const result = await invoke<{ cursor: string; keys: string[] }>(
          "scan_keys",
          {
            connectionId,
            cursor,
            count: 500,
            pattern,
          }
        );
        cursor = result.cursor;
        keys.push(...result.keys);
      } while (cursor !== "0");

      allKeys = keys;
      displayedCount = Math.min(PAGE_SIZE, keys.length);
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
      scanKeys();
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
      onkeydown={(e) => e.key === "Enter" && scanKeys()}
    />
    <button class="refresh-btn" onclick={scanKeys} disabled={loading} title="Refresh">
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
      {#if keyCount > 0}
        <div class="key-count">
          Showing {displayedCount} of {keyCount} keys
        </div>
        {#if displayedCount < allKeys.length}
          <button class="load-more-btn" onclick={loadMore}>
            Load more... ({allKeys.length - displayedCount} remaining)
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
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    font-size: 0.8125rem;
    font-family: inherit;
    outline: none;
  }

  .pattern-input:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .refresh-btn {
    padding: 0.375rem 0.5rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.875rem;
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    color: var(--color-error, #e55);
    font-size: 0.8125rem;
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
    font-size: 0.8125rem;
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
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-accent, #5b8def);
    cursor: pointer;
    font-size: 0.8125rem;
    font-family: inherit;
    text-align: center;
  }

  .load-more-btn:hover {
    background: var(--color-border, #333);
  }
</style>
