<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import EditModal from "../EditModal.svelte";
  import PromptModal from "../PromptModal.svelte";
  import { toasts } from "$lib/stores/toasts";
  import { exportCsv } from "$lib/utils/csv";

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let entries = $state<[string, string][]>([]);
  let filter = $state("");

  let filtered = $derived(
    filter
      ? entries.filter(
          ([f, v]) =>
            f.toLowerCase().includes(filter.toLowerCase()) ||
            v.toLowerCase().includes(filter.toLowerCase())
        )
      : entries
  );

  async function load() {
    loading = true;
    error = null;
    try {
      entries = await invoke<[string, string][]>("get_hash_all", { connectionId, key });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  let editingField = $state<string | null>(null);
  let editingValue = $state<string>("");
  let addingField = $state(false);

  async function handleSaveField(value: string, newKeyName?: string) {
    if (!editingField) return;
    try {
      if (newKeyName && newKeyName !== editingField) {
        // field renamed
        await invoke("set_hash_field", { connectionId, key, field: newKeyName, value });
        await invoke("del_hash_field", { connectionId, key, fields: [editingField] });
      } else {
        await invoke("set_hash_field", { connectionId, key, field: editingField, value });
      }
      editingField = null;
      await load();
    } catch (e) {
      toasts.add(String(e), "error");
      throw e;
    }
  }

  async function handleAddField(field: string) {
    if (!field.trim()) return;
    addingField = false;
    editingField = field.trim();
    editingValue = "";
  }

  async function handleDeleteField(field: string) {
    if (!confirm(`Delete field "${field}"?`)) return;
    try {
      await invoke("del_hash_field", { connectionId, key, field });
      await load();
    } catch (e) {
      toasts.add(String(e), "error");
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
      <button class="btn btn-primary" onclick={() => addingField = true}>+ Add Field</button>
      <input
        class="filter-input"
        type="text"
        bind:value={filter}
        placeholder="Filter fields..."
      />
      <button 
        class="btn btn-secondary" 
        style="margin-left: auto;"
        onclick={() => exportCsv(`${key.split(':').pop()}_hash`, ['Field', 'Value'], filtered)}
        title="Export to CSV"
      >
        &#128190; Export CSV
      </button>
      <span class="count">{filtered.length} / {entries.length} fields</span>
    </div>

    {#if filtered.length === 0}
      <div class="state-msg">No fields found</div>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th class="col-field">Field</th>
              <th class="col-value">Value</th>
              <th class="col-actions"></th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as [field, value] (field)}
              <tr>
                <td class="col-field"><code>{field}</code></td>
                <td class="col-value">
                  <div class="val-content"><code>{value}</code></div>
                </td>
                <td class="col-actions">
                  <button class="icon-btn" onclick={() => { editingField = field; editingValue = value; }} title="Edit">&#9998;</button>
                  <button class="icon-btn danger" onclick={() => handleDeleteField(field)} title="Delete">&#128465;</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    {#if editingField}
      <EditModal
        title={`Edit Field: ${editingField}`}
        initialKeyName={editingField}
        initialValue={editingValue}
        onSave={handleSaveField}
        onCancel={() => editingField = null}
      />
    {/if}

    {#if addingField}
      <PromptModal
        title="New Field Name"
        placeholder="field_name"
        onSave={handleAddField}
        onCancel={() => addingField = false}
      />
    {/if}
  {/if}
</div>

<style>
  .viewer {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 0;
    padding: 0.375rem 0.625rem;
  }

  .state-msg {
    color: var(--color-muted, #888);
    font-size: 0.75rem;
    text-align: center;
    padding: 1rem 0;
  }

  .error {
    color: var(--color-error, #e55);
    font-size: 0.75rem;
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
    font-size: 0.75rem;
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

  .table-wrap {
    overflow: auto;
    flex: 1;
    min-height: 0;
    background: color-mix(in srgb, var(--color-surface) 30%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0,0,0,0.05);
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.75rem;
  }

  .data-table th {
    text-align: left;
    padding: 0.375rem 0.625rem;
    border-bottom: 1px solid var(--color-border, #333);
    color: var(--color-muted, #888);
    font-weight: 600;
    font-size: 0.6875rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    position: sticky;
    top: 0;
    background: var(--color-surface, #1e1e1e);
  }

  .data-table td {
    padding: 0.375rem 0.625rem;
    border-bottom: 1px solid var(--color-border, #333);
    vertical-align: top;
  }

  .data-table tbody tr {
    transition: background-color 0.15s;
  }

  .data-table tbody tr:hover {
    background-color: var(--color-surface, #222);
  }

  .data-table code {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.75rem;
    word-break: break-all;
  }

  .col-field {
    width: 30%;
    min-width: 120px;
  }

  .col-actions {
    width: 80px;
    text-align: right;
  }

  .val-content {
    max-height: 100px;
    overflow-y: auto;
  }

  .btn {
    padding: 0.25rem 0.625rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.75rem;
    white-space: nowrap;
  }

  .btn-primary {
    background: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
    color: #fff;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    font-size: 0.75rem;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    color: var(--color-fg);
    background: var(--color-surface, #222);
  }

  .icon-btn.danger:hover {
    color: #ef4444;
  }
</style>
