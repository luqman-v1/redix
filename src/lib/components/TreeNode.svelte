<script lang="ts">
  import { untrack } from "svelte";
  import type { TreeNode } from "$lib/utils/tree-builder";
  import Self from "./TreeNode.svelte";

  interface Props {
    node: TreeNode;
    depth: number;
    onselect: (key: string) => void;
    expandedPaths?: Set<string>;
    ontoggle?: (path: string) => void;
    now?: number;
  }

  let { node, depth, onselect, expandedPaths, ontoggle, now = 0 }: Props = $props();
  let internalExpanded = $state(false);
  let isOpen = $derived(ontoggle && expandedPaths ? expandedPaths.has(node.path) : internalExpanded);

  function toggle() {
    if (ontoggle) ontoggle(node.path);
    else internalExpanded = !internalExpanded;
  }

  let baseTtl = $derived(node.ttl ?? -1);
  let countdown = $state(0);
  let currentTtl = $derived(baseTtl > 0 ? baseTtl - countdown : baseTtl);

  $effect(() => {
    baseTtl;
    untrack(() => { countdown = 0; });
  });

  // Driven by the single parent interval (`now`), not per-leaf subscriptions.
  $effect(() => {
    const t = now;
    if (t <= 0) return;
    untrack(() => {
      if (baseTtl > 0 && baseTtl - countdown > 0) countdown++;
    });
  });

  function formatTtl(s: number): string {
    if (s <= 0) return "";
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    if (m < 60) return `${m}m ${s % 60}s`;
    const h = Math.floor(m / 60);
    return `${h}h ${m % 60}m`;
  }
</script>

{#if node.isLeaf}
  <button
    class="tree-item leaf"
    style:padding-left="{depth * 16 + 8}px"
    onclick={() => onselect(node.path)}
  >
    <span class="icon">🔑</span>
    <span class="name">{node.name}</span>
    {#if currentTtl > 0}
      <span class="ttl-badge">⏳ {formatTtl(currentTtl)}</span>
    {/if}
  </button>
{:else}
  <button
    class="tree-item folder"
    style:padding-left="{depth * 16 + 8}px"
    onclick={toggle}
  >
    <span class="toggle">{isOpen ? "▼" : "▶"}</span>
    <span class="icon">📁</span>
    <span class="name">{node.name}</span>
    <span class="badge">{node.count}</span>
  </button>
  {#if isOpen}
    {#each node.children as child (child.path)}
      <Self node={child} depth={depth + 1} {onselect} {expandedPaths} {ontoggle} {now} />
    {/each}
  {/if}
{/if}

<style>
  .tree-item {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    width: 100%;
    border: none;
    background: none;
    color: var(--color-fg);
    font-size: 0.75rem;
    font-family: inherit;
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
    padding-right: 0.5rem;
    cursor: pointer;
    text-align: left;
    border-radius: 4px;
    white-space: nowrap;
    user-select: none;
    -webkit-user-select: none;
  }

  .tree-item:hover {
    background: var(--color-hover, rgba(128, 128, 128, 0.1));
  }

  .toggle {
    width: 1rem;
    text-align: center;
    font-size: 0.625rem;
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .icon {
    flex-shrink: 0;
    font-size: 0.75rem;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .badge {
    margin-left: auto;
    font-size: 0.6875rem;
    color: var(--color-muted);
    background: var(--color-badge-bg, rgba(128, 128, 128, 0.15));
    padding: 0.0625rem 0.375rem;
    border-radius: 9999px;
    flex-shrink: 0;
  }

  .ttl-badge {
    margin-left: auto;
    font-size: 0.65rem;
    color: var(--color-muted);
    background: var(--color-surface-input);
    padding: 0.1rem 0.35rem;
    border-radius: 4px;
    font-family: "JetBrains Mono", monospace;
    opacity: 0.8;
  }
</style>
