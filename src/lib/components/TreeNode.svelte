<script lang="ts">
  import type { TreeNode } from "$lib/utils/tree-builder";
  import Self from "./TreeNode.svelte";

  interface Props {
    node: TreeNode;
    depth: number;
    onselect: (key: string) => void;
  }

  let { node, depth, onselect }: Props = $props();
  let expanded = $state(false);
</script>

{#if node.isLeaf}
  <button
    class="tree-item leaf"
    style:padding-left="{depth * 16 + 8}px"
    onclick={() => onselect(node.path)}
  >
    <span class="icon">🔑</span>
    <span class="name">{node.name}</span>
  </button>
{:else}
  <button
    class="tree-item folder"
    style:padding-left="{depth * 16 + 8}px"
    onclick={() => (expanded = !expanded)}
  >
    <span class="toggle">{expanded ? "▼" : "▶"}</span>
    <span class="icon">📁</span>
    <span class="name">{node.name}</span>
    <span class="badge">{node.count}</span>
  </button>
  {#if expanded}
    {#each node.children as child (child.path)}
      <Self node={child} depth={depth + 1} {onselect} />
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
</style>
