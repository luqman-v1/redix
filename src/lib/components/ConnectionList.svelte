<script lang="ts">
  import type { ConnectionConfig } from "$lib/types/connection";
  import { connections, activeConnection, connectToServer } from "$lib/stores/connections";
  import ConnectionForm from "./ConnectionForm.svelte";

  let showForm = $state(false);
  let editingConnection = $state<ConnectionConfig | null>(null);
  let connectionList = $state<ConnectionConfig[]>([]);

  connections.subscribe((v) => (connectionList = v));

  let active = $state<ConnectionConfig | null>(null);
  activeConnection.subscribe((v) => (active = v));

  function openAdd() {
    editingConnection = null;
    showForm = true;
  }

  function openEdit(conn: ConnectionConfig) {
    editingConnection = conn;
    showForm = true;
  }

  function closeForm() {
    showForm = false;
    editingConnection = null;
  }

  async function handleDelete(conn: ConnectionConfig) {
    if (!confirm(`Delete "${conn.name}"?`)) return;
    await connections.remove(conn.id);
    if (active?.id === conn.id) activeConnection.set(null);
  }

  async function selectActive(conn: ConnectionConfig) {
    await connectToServer(conn);
  }
</script>

<div class="header">
  <h2 class="title">Connections</h2>
  <button class="add-btn" onclick={openAdd}>+ Add</button>
</div>

{#if connectionList.length === 0}
  <p class="empty">No connections yet</p>
{:else}
  <ul class="list">
    {#each connectionList as conn (conn.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <li
        class="item"
        class:active={active?.id === conn.id}
        onclick={() => selectActive(conn)}
      >
        <div class="info">
          <span class="name">{conn.name}</span>
          <span class="addr">{conn.host}:{conn.port}</span>
        </div>
        <div class="item-actions">
          <button
            class="icon-btn"
            onclick={(e) => { e.stopPropagation(); openEdit(conn); }}
            title="Edit"
          >
            &#9998;
          </button>
          <button
            class="icon-btn danger"
            onclick={(e) => { e.stopPropagation(); handleDelete(conn); }}
            title="Delete"
          >
            &#128465;
          </button>
        </div>
      </li>
    {/each}
  </ul>
{/if}

{#if showForm}
  <ConnectionForm editing={editingConnection} onclose={closeForm} />
{/if}

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0;
  }

  .title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-fg);
    margin: 0;
  }

  .add-btn {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-accent, #6366f1);
    padding: 0.25rem 0.5rem;
    font-size: 0.8125rem;
    cursor: pointer;
    font-weight: 500;
  }

  .add-btn:hover {
    background: var(--color-bg);
  }

  .empty {
    color: var(--color-muted);
    font-size: 0.875rem;
    margin: 0.5rem 0;
  }

  .list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    overflow-y: auto;
    flex: 1;
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.625rem;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .item:hover {
    background: var(--color-bg);
  }

  .item.active {
    background: var(--color-bg);
    outline: 2px solid var(--color-accent, #6366f1);
    outline-offset: -2px;
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    min-width: 0;
  }

  .name {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .addr {
    font-size: 0.75rem;
    color: var(--color-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-actions {
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .item:hover .item-actions {
    opacity: 1;
  }

  .icon-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.25rem;
    border-radius: 4px;
    color: var(--color-muted);
    line-height: 1;
  }

  .icon-btn:hover {
    background: var(--color-surface);
    color: var(--color-fg);
  }

  .icon-btn.danger:hover {
    color: #ef4444;
  }
</style>
