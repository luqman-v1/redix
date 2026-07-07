<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface StreamEntry {
    id: string;
    fields: [string, string][];
  }

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let entries = $state<StreamEntry[]>([]);
  let expanded = $state<Set<string>>(new Set());

  function toggleExpand(id: string) {
    const next = new Set(expanded);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expanded = next;
  }

  async function load() {
    loading = true;
    error = null;
    try {
      entries = await invoke<StreamEntry[]>("get_stream_range", {
        connectionId,
        key,
        start: "-",
        end: "+",
        count: 100,
      });
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
      <span class="count">{entries.length} entries</span>
    </div>

    {#if entries.length === 0}
      <div class="state-msg">Stream is empty</div>
    {:else}
      <div class="entry-list">
        {#each entries as entry (entry.id)}
          <div class="entry">
            <button class="entry-header" onclick={() => toggleExpand(entry.id)}>
              <span class="expand-icon">{expanded.has(entry.id) ? "▼" : "▶"}</span>
              <code class="entry-id">{entry.id}</code>
              <span class="field-count">{entry.fields.length} fields</span>
            </button>
            {#if expanded.has(entry.id)}
              <table class="field-table">
                <tbody>
                  {#each entry.fields as [field, value] (field)}
                    <tr>
                      <td class="col-field"><code>{field}</code></td>
                      <td class="col-value"><code>{value}</code></td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        {/each}
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
    justify-content: flex-end;
  }

  .count {
    color: var(--color-muted, #888);
    font-size: 0.6875rem;
    white-space: nowrap;
  }

  .entry-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .entry {
    background: var(--color-surface-raised, #222);
    border-radius: 4px;
    overflow: hidden;
  }

  .entry-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.375rem 0.5rem;
    border: none;
    background: none;
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.8125rem;
    text-align: left;
  }

  .entry-header:hover {
    background: var(--color-surface, #2a2a2a);
  }

  .expand-icon {
    font-size: 0.625rem;
    color: var(--color-muted, #888);
    flex-shrink: 0;
  }

  .entry-id {
    font-family: monospace;
    font-size: 0.75rem;
    color: var(--color-accent, #5b8def);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .field-count {
    color: var(--color-muted, #888);
    font-size: 0.6875rem;
    white-space: nowrap;
  }

  .field-table {
    width: 100%;
    border-collapse: collapse;
    border-top: 1px solid var(--color-border, #333);
    font-size: 0.8125rem;
  }

  .field-table td {
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--color-border, #333);
    vertical-align: top;
  }

  .field-table code {
    font-family: monospace;
    font-size: 0.75rem;
    word-break: break-all;
  }

  .col-field {
    width: 30%;
    min-width: 80px;
    color: var(--color-muted, #888);
  }
</style>
