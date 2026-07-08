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
  let entries = $state<[string, number][]>([]);
  let sortField = $state<"member" | "score">("score");
  let sortAsc = $state(true);

  let sorted = $derived.by(() => {
    const copy = [...entries];
    copy.sort((a, b) => {
      const cmp =
        sortField === "score" ? a[1] - b[1] : a[0].localeCompare(b[0]);
      return sortAsc ? cmp : -cmp;
    });
    return copy;
  });

  function toggleSort(field: "member" | "score") {
    if (sortField === field) {
      sortAsc = !sortAsc;
    } else {
      sortField = field;
      sortAsc = true;
    }
  }

  async function load() {
    loading = true;
    error = null;
    try {
      const raw = await invoke<[string, number][]>("get_sorted_set_range", {
        connectionId,
        key,
        start: 0,
        stop: -1,
      });
      entries = raw;
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
      <button 
        class="btn btn-secondary" 
        style="margin-right: auto;"
        onclick={() => exportCsv(`${key.split(':').pop()}_zset`, ['Score', 'Member'], sorted.map(row => [String(row[1]), row[0]]))}
        title="Export to CSV"
      >
        &#128190; Export CSV
      </button>
      <span class="count">{entries.length} members</span>
    </div>

    {#if entries.length === 0}
      <div class="state-msg">Sorted set is empty</div>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th class="sortable" onclick={() => toggleSort("score")}>
                Score {sortField === "score" ? (sortAsc ? "↑" : "↓") : ""}
              </th>
              <th class="sortable" onclick={() => toggleSort("member")}>
                Member {sortField === "member" ? (sortAsc ? "↑" : "↓") : ""}
              </th>
            </tr>
          </thead>
          <tbody>
            {#each sorted as [member, score] (member)}
              <tr>
                <td class="col-score"><code>{score}</code></td>
                <td class="col-member"><code>{member}</code></td>
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
    justify-content: flex-end;
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

  .sortable {
    cursor: pointer;
    user-select: none;
  }

  .sortable:hover {
    color: var(--color-fg);
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

  .col-score {
    width: 100px;
    text-align: right;
  }
</style>
