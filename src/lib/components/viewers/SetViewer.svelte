<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { exportCsv } from "$lib/utils/csv";

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let members = $state<string[]>([]);
  let filter = $state("");
  let newMember = $state("");
  let mutating = $state(false);

  let filtered = $derived(
    filter
      ? members.filter((m) => m.toLowerCase().includes(filter.toLowerCase()))
      : members
  );

  async function load() {
    loading = true;
    error = null;
    try {
      members = await invoke<string[]>("get_set_members", { connectionId, key });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function addMember() {
    if (!newMember.trim()) return;
    mutating = true;
    error = null;
    try {
      await invoke("add_set_member", { connectionId, key, member: newMember });
      newMember = "";
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      mutating = false;
    }
  }

  async function removeMember(member: string) {
    mutating = true;
    error = null;
    try {
      await invoke("del_set_member", { connectionId, key, member });
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      mutating = false;
    }
  }

  $effect(() => {
    if (connectionId && key) {
      load();
    }
  });
</script>

<div class="viewer">
  {#if loading}
    <div class="state-msg">Loading...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else}
    <div class="toolbar">
      <input
        class="filter-input"
        type="text"
        bind:value={filter}
        placeholder="Filter members..."
      />
      <button 
        class="btn btn-secondary" 
        style="margin-left: auto;"
        onclick={() => exportCsv(`${key.split(':').pop()}_set`, ['Member'], filtered.map(m => [m]))}
        title="Export to CSV"
      >
        &#128190; Export CSV
      </button>
      <span class="count">{filtered.length} / {members.length}</span>
    </div>

    <div class="add-row">
      <input
        class="add-input"
        type="text"
        bind:value={newMember}
        placeholder="New member..."
        onkeydown={(e) => e.key === "Enter" && addMember()}
      />
      <button class="btn btn-primary" onclick={addMember} disabled={mutating || !newMember.trim()}>
        Add
      </button>
    </div>

    {#if filtered.length === 0}
      <div class="state-msg">No members found</div>
    {:else}
      <ul class="member-list">
        {#each filtered as member (member)}
          <li class="member-item">
            <code class="member-value">{member}</code>
            <button
              class="btn btn-danger"
              onclick={() => removeMember(member)}
              disabled={mutating}
              title="Remove member"
            >
              x
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .viewer {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 0;
    padding: 0.75rem;
  }

  .state-msg {
    color: var(--color-muted, #888);
    font-size: 0.8125rem;
    text-align: center;
    padding: 1rem 0;
  }

  .error {
    color: var(--color-error, #e55);
    font-size: 0.8125rem;
    padding: 0.25rem 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .filter-input {
    flex: 1;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    font-size: 0.8125rem;
    outline: none;
  }

  .filter-input:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .count {
    color: var(--color-muted, #888);
    font-size: 0.6875rem;
    white-space: nowrap;
  }

  .add-row {
    display: flex;
    gap: 0.375rem;
  }

  .add-input {
    flex: 1;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    font-size: 0.8125rem;
    outline: none;
  }

  .add-input:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .btn {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.75rem;
    white-space: nowrap;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
    color: #fff;
  }

  .btn-danger {
    color: var(--color-error, #e55);
    border-color: var(--color-error, #e55);
    font-weight: 700;
    padding: 0.125rem 0.375rem;
  }

  .member-list {
    list-style: none;
    margin: 0;
    padding: 0.5rem;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: color-mix(in srgb, var(--color-surface) 30%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.05);
  }

  .member-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: var(--color-surface-raised, #222);
    border-radius: 3px;
  }

  .member-value {
    font-family: monospace;
    font-size: 0.8125rem;
    word-break: break-all;
    flex: 1;
    min-width: 0;
  }
</style>
