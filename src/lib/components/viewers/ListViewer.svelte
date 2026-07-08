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
  let items = $state<string[]>([]);
  let pushValue = $state("");
  let pushing = $state(false);

  async function load() {
    loading = true;
    error = null;
    try {
      items = await invoke<string[]>("get_list_range", { connectionId, key, start: 0, stop: -1 });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function pushItem(side: "left" | "right") {
    if (!pushValue.trim()) return;
    pushing = true;
    error = null;
    try {
      await invoke("list_push", { connectionId, key, value: pushValue, side });
      pushValue = "";
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      pushing = false;
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
        class="push-input"
        type="text"
        bind:value={pushValue}
        placeholder="New item value..."
        onkeydown={(e) => e.key === "Enter" && pushItem("right")}
      />
      <button class="btn" onclick={() => pushItem("left")} disabled={pushing || !pushValue.trim()}>
        Push Head
      </button>
      <button class="btn" onclick={() => pushItem("right")} disabled={pushing || !pushValue.trim()}>
        Push Tail
      </button>
      <button 
        class="btn btn-secondary" 
        style="margin-left: auto;"
        onclick={() => exportCsv(`${key.split(':').pop()}_list`, ['Index', 'Value'], items.map((item, i) => [String(i), item]))}
        title="Export to CSV"
      >
        &#128190; Export CSV
      </button>
      <span class="count">{items.length} items</span>
    </div>

    {#if items.length === 0}
      <div class="state-msg">List is empty</div>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th class="col-index">Index</th>
              <th class="col-value">Value</th>
            </tr>
          </thead>
          <tbody>
            {#each items as item, i (i)}
              <tr>
                <td class="col-index"><code>{i}</code></td>
                <td class="col-value"><code>{item}</code></td>
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
    gap: 0.375rem;
  }

  .push-input {
    flex: 1;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    font-size: 0.75rem;
    outline: none;
  }

  .push-input:focus {
    border-color: var(--color-accent, #5b8def);
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

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
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

  .col-index {
    width: 60px;
    text-align: right;
  }
</style>
