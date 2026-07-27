<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { untrack, onMount } from "svelte";
  import { buildTree } from "$lib/utils/tree-builder";
  import type { TreeNode } from "$lib/utils/tree-builder";
  import TreeNodeComponent from "./TreeNode.svelte";

  interface Props {
    connectionId: string;
    separator: string;
    refreshTrigger?: number;
    onselect: (key: string) => void;
  }

  let { connectionId, separator, refreshTrigger = 0, onselect }: Props = $props();

  const PAGE_SIZE = 500;
  const MAX_SCAN_PAGES = 200;
  const HISTORY_KEY = "redix_search_history";

  let pattern = $state("*");
  let loading = $state(false);
  let allKeys = $state<{key: string; ttl: number}[]>([]);
  let displayedCount = $state(0);
  let tree = $state<TreeNode[]>([]);
  let keyCount = $state(0);
  let error = $state<string | null>(null);
  let searchHistory = $state<string[]>([]);

  function loadSearchHistory() {
    try {
      const saved = localStorage.getItem(HISTORY_KEY);
      if (saved) {
        searchHistory = JSON.parse(saved);
      }
    } catch {}
  }

  function saveSearchHistory(term: string) {
    if (!term || term.trim() === "" || term.trim() === "*") return;
    const cleanTerm = term.trim();
    const filtered = searchHistory.filter((item) => item !== cleanTerm);
    const updated = [cleanTerm, ...filtered].slice(0, 20);
    searchHistory = updated;
    try {
      localStorage.setItem(HISTORY_KEY, JSON.stringify(updated));
    } catch {}
  }

  let showHistory = $state(false);
  let filteredHistory = $derived(
    searchHistory.filter(
      (item) => !pattern || pattern === "*" || item.toLowerCase().includes(pattern.toLowerCase())
    )
  );

  function selectHistoryItem(item: string) {
    pattern = item;
    showHistory = false;
    scanKeys(true);
  }

  function clearHistory(e: MouseEvent) {
    e.stopPropagation();
    searchHistory = [];
    try {
      localStorage.removeItem(HISTORY_KEY);
    } catch {}
  }

  let isMounted = false;

  onMount(() => {
    isMounted = true;
    loadSearchHistory();
    return () => { isMounted = false; };
  });

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
      if (pattern) {
        saveSearchHistory(pattern);
      }
    }

    try {
      let newKeys: {key: string; ttl: number}[] = [];
      let c = currentCursor;
      let iterations = 0;
      
      const isWildcard = pattern.includes('*') || pattern.includes('?') || pattern.includes('[');
      
      if (!isWildcard && pattern) {
        // exact match fallback
        const type = await invoke<string>("get_key_type", { connectionId, key: pattern });
        if (type !== "none") {
          const ttl = await invoke<number>("get_key_ttl", { connectionId, key: pattern });
          newKeys.push({ key: pattern, ttl });
        }
        c = 0;
      } else {
        do {
          if (iterations > 0 && c === 0) break;
          const result = await invoke<{ cursor: number; keys: {key: string; ttl: number}[] }>(
            "scan_keys",
            {
              connectionId,
              cursor: c,
              count: 1000,
              pattern,
            }
          );
          if (!isMounted) return;
          c = result.cursor;
          newKeys.push(...result.keys);
          iterations++;
          // ponytail: removed iteration limit so sparse wildcard searches (e.g. foo:*) 
          // don't abort prematurely on huge DBs. Will scan until PAGE_SIZE found or DB end.
        } while (newKeys.length < PAGE_SIZE && c !== 0 && iterations < MAX_SCAN_PAGES);
      }

      currentCursor = c;
      const keyMap = new Map();
      for (const item of allKeys) keyMap.set(item.key, item);
      for (const item of newKeys) keyMap.set(item.key, item);
      allKeys = Array.from(keyMap.values());
      displayedCount = allKeys.length;
      updateTree();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // auto-scan when connectionId or refreshTrigger changes
  $effect(() => {
    // Read the values so effect tracks them
    const currentConn = connectionId;
    const trigger = refreshTrigger;
    
    if (currentConn) {
      untrack(() => {
        // If trigger changed, reset the list completely
        scanKeys(trigger > 0);
      });
    }
  });
</script>

<div class="key-tree">
  <div class="toolbar">
    <div class="search-input-wrap">
      <input
        data-key-search
        class="pattern-input"
        type="text"
        autocapitalize="off"
        autocorrect="off"
        spellcheck="false"
        bind:value={pattern}
        placeholder="Filter pattern (e.g. user:*)"
        onfocus={() => (showHistory = true)}
        onblur={() => setTimeout(() => (showHistory = false), 200)}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            showHistory = false;
            scanKeys(true);
          }
        }}
      />
      {#if showHistory && filteredHistory.length > 0}
        <div class="history-dropdown">
          <div class="history-header">
            <span>Recent Searches</span>
            <button type="button" class="clear-history-btn" onmousedown={clearHistory}>Clear</button>
          </div>
          {#each filteredHistory as item}
            <button
              type="button"
              class="history-item"
              onmousedown={() => selectHistoryItem(item)}
            >
              <span class="history-icon">🔍</span>
              <span class="history-text">{item}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
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

  .search-input-wrap {
    position: relative;
    flex: 1;
    display: flex;
  }

  .history-dropdown {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--color-surface-raised, #ffffff);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 6px);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.25);
    z-index: 100;
    display: flex;
    flex-direction: column;
    padding: 0.25rem;
    max-height: 220px;
    overflow-y: auto;
    backdrop-filter: blur(12px);
  }

  .history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0.5rem;
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 0.25rem;
  }

  .clear-history-btn {
    font-size: 0.65rem;
    color: var(--color-error, #ef4444);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .clear-history-btn:hover {
    text-decoration: underline;
  }

  .history-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border: none;
    background: none;
    color: var(--color-fg);
    font-size: 0.75rem;
    text-align: left;
    border-radius: 4px;
    cursor: pointer;
    width: 100%;
  }

  .history-item:hover {
    background: var(--color-surface-btn-hover, rgba(0, 0, 0, 0.05));
    color: var(--color-accent);
  }

  .history-icon {
    font-size: 0.7rem;
    opacity: 0.7;
  }

  .history-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
