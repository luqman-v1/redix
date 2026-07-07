<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

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
        placeholder="Filter fields..."
      />
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
            </tr>
          </thead>
          <tbody>
            {#each filtered as [field, value] (field)}
              <tr>
                <td class="col-field"><code>{field}</code></td>
                <td class="col-value"><code>{value}</code></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
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

  .table-wrap {
    overflow: auto;
    flex: 1;
    min-height: 0;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  .data-table th {
    text-align: left;
    padding: 0.375rem 0.5rem;
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
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--color-border, #333);
    vertical-align: top;
  }

  .data-table code {
    font-family: monospace;
    font-size: 0.8125rem;
    word-break: break-all;
  }

  .col-field {
    width: 30%;
    min-width: 120px;
  }
</style>
