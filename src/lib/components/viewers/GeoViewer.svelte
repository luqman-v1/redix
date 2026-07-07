<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface GeoMember {
    member: string;
    longitude: number;
    latitude: number;
    score: number;
  }

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let members = $state<GeoMember[]>([]);

  async function load() {
    loading = true;
    error = null;
    try {
      members = await invoke<GeoMember[]>("get_geo_members", { connectionId, key });
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
      <span class="count">{members.length} members</span>
    </div>

    {#if members.length === 0}
      <div class="state-msg">No geo members found</div>
    {:else}
      <div class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>Member</th>
              <th>Longitude</th>
              <th>Latitude</th>
              <th>Score</th>
            </tr>
          </thead>
          <tbody>
            {#each members as m (m.member)}
              <tr>
                <td><code>{m.member}</code></td>
                <td class="col-num">{m.longitude.toFixed(6)}</td>
                <td class="col-num">{m.latitude.toFixed(6)}</td>
                <td class="col-num">{m.score.toFixed(2)}</td>
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

  .col-num {
    text-align: right;
    font-family: monospace;
    font-size: 0.8125rem;
    width: 110px;
  }
</style>
