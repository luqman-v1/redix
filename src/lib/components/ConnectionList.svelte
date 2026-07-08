<script lang="ts">
  import type { ConnectionConfig } from "$lib/types/connection";
  import { connections, activeConnection, connectToServer, disconnectFromServer } from "$lib/stores/connections";
  import { toasts } from "$lib/stores/toasts";
  import ConnectionForm from "./ConnectionForm.svelte";

  let { onclose }: { onclose: () => void } = $props();

  let showForm = $state(false);
  let editingConnection = $state<ConnectionConfig | null>(null);
  let connectionList = $state<ConnectionConfig[]>([]);
  let active = $state<ConnectionConfig | null>(null);
  let deletingId = $state<string | null>(null);
  let connectingId = $state<string | null>(null);

  let groupedConnections = $derived((() => {
    const groups: Record<string, ConnectionConfig[]> = {};
    for (const conn of connectionList) {
      const folder = conn.folder || "Uncategorized";
      if (!groups[folder]) groups[folder] = [];
      groups[folder].push(conn);
    }
    return Object.entries(groups).sort((a, b) => {
      if (a[0] === "Uncategorized") return 1;
      if (b[0] === "Uncategorized") return -1;
      return a[0].localeCompare(b[0]);
    }).map(([folder, conns]) => ({ folder, conns }));
  })());

  // ponytail: $derived(get(store)) only reads once — need manual subscribe for reactivity
  $effect(() => {
    const unsub = connections.subscribe((v) => { connectionList = v; });
    return unsub;
  });
  $effect(() => {
    const unsub = activeConnection.subscribe((v) => { active = v; });
    return unsub;
  });

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
    try {
      if (active?.id === conn.id) {
        await disconnectFromServer(conn.id);
      }
      await connections.remove(conn.id);
      deletingId = null;
    } catch (e) {
      toasts.add(String(e), "error");
    }
  }

  async function selectActive(conn: ConnectionConfig) {
    if (connectingId) return;
    connectingId = conn.id;
    try {
      await connectToServer(conn);
      onclose();
    } catch (e) {
      toasts.add(String(e), "error");
    } finally {
      connectingId = null;
    }
  }

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onBackdropClick}>
  <div class="modal">
    <div class="header">
      <h2 class="title">Connections</h2>
      <div class="header-actions">
        <button class="add-btn" onclick={openAdd}>+ Add</button>
        <button class="close-btn" onclick={onclose}>&#10005;</button>
      </div>
    </div>

    {#if connectionList.length === 0}
      <p class="empty">No connections yet</p>
    {:else}
      <ul class="list">
        {#each groupedConnections as group (group.folder)}
          <div class="folder-header">
            <span>{group.folder}</span>
          </div>
          {#each group.conns as conn (conn.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <li
              class="item"
              class:active={active?.id === conn.id}
              onclick={() => selectActive(conn)}
            >
              <div class="info">
                <span class="name">
                  {conn.name}
                  {#if connectingId === conn.id}
                    <span style="font-size:0.65rem; color:var(--color-accent); margin-left:0.25rem;">(Connecting...)</span>
                  {/if}
                </span>
                <span class="addr">{conn.host}:{conn.port}</span>
              </div>
              <div class="item-actions">
                {#if deletingId === conn.id}
                  <button
                    class="confirm-btn"
                    onclick={(e) => { e.stopPropagation(); handleDelete(conn); }}
                  >
                    Sure?
                  </button>
                  <button
                    class="icon-btn"
                    onclick={(e) => { e.stopPropagation(); deletingId = null; }}
                  >
                    &#10005;
                  </button>
                {:else}
                  <button
                    class="icon-btn"
                    onclick={(e) => { e.stopPropagation(); openEdit(conn); }}
                    title="Edit"
                  >
                    &#9998;
                  </button>
                  <button
                    class="icon-btn danger"
                    onclick={(e) => { e.stopPropagation(); deletingId = conn.id; }}
                    title="Delete"
                  >
                    &#128465;
                  </button>
                {/if}
              </div>
            </li>
          {/each}
        {/each}
      </ul>
    {/if}

  </div>

  {#if showForm}
    <ConnectionForm editing={editingConnection} onclose={closeForm} />
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 40; /* under the ConnectionForm modal which is 50 */
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }

  .modal {
    background: var(--color-surface, #1e1e1e);
    border: 1px solid var(--color-border, #333);
    border-radius: 12px;
    width: 36rem;
    max-width: 90vw;
    min-height: 24rem;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
  }
  .close-btn:hover {
    color: var(--color-fg);
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
    font-size: 0.75rem;
    cursor: pointer;
    font-weight: 500;
  }

  .add-btn:hover {
    background: var(--color-bg);
  }

  .empty {
    color: var(--color-muted);
    font-size: 0.75rem;
    margin: 0.5rem 0;
  }

  .list {
    list-style: none;
    padding: 0.5rem;
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
    margin-left: 0.5rem;
  }

  .folder-header {
    font-size: 0.6875rem;
    font-weight: 700;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0.75rem 0.5rem 0.25rem 0.5rem;
    display: flex;
    align-items: center;
  }

  .folder-header:first-child {
    padding-top: 0.25rem;
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
    font-size: 0.75rem;
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
    font-size: 0.75rem;
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

  .confirm-btn {
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
  }

  .confirm-btn:hover {
    background: #dc2626;
  }
</style>
