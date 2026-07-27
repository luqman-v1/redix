<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toasts";
  import { onMount } from "svelte";

  interface Props {
    connectionId: string;
  }
  let { connectionId }: Props = $props();

  interface SlowLogEntry {
    id: number;
    timestamp: number;
    duration_micros: number;
    command: string;
    client_address: string;
    client_name: string;
  }

  let logs = $state<SlowLogEntry[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function fetchLogs() {
    loading = true;
    error = null;
    try {
      logs = await invoke<SlowLogEntry[]>("get_slow_logs", { connectionId });
    } catch (e) {
      error = String(e);
      toasts.add("Failed to fetch slow logs: " + error, "error");
    } finally {
      loading = false;
    }
  }

  async function resetLogs() {
    try {
      await invoke("reset_slow_logs", { connectionId });
      toasts.add("Slow logs reset successfully", "success");
      await fetchLogs();
    } catch (e) {
      toasts.add("Failed to reset logs: " + String(e), "error");
    }
  }

  onMount(() => {
    fetchLogs();
  });

  function formatTime(ts: number) {
    return new Date(ts * 1000).toLocaleString();
  }

  function formatDuration(micros: number) {
    if (micros < 1000) return `${micros}µs`;
    return `${(micros / 1000).toFixed(2)}ms`;
  }
</script>

<div class="slowlog-container">
  <div class="header-section">
    <div class="title-group">
      <h2>Slow Log Profiler</h2>
      <p class="subtitle">Monitor queries that exceed the `slowlog-log-slower-than` configuration.</p>
    </div>
    <div class="controls">
      <button class="btn btn-secondary" onclick={resetLogs} disabled={loading}>
        Reset Logs
      </button>
      <button class="btn btn-primary" onclick={fetchLogs} disabled={loading}>
        {loading ? "Refreshing..." : "Refresh"}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  <div class="table-container">
    <table class="data-table">
      <thead>
        <tr>
          <th style="width: 80px;">ID</th>
          <th style="width: 200px;">Timestamp</th>
          <th style="width: 100px; text-align: right;">Duration</th>
          <th>Command</th>
          <th style="width: 200px;">Client Info</th>
        </tr>
      </thead>
      <tbody>
        {#if loading && logs.length === 0}
          <tr>
            <td colspan="5" class="empty-cell">
              <div class="spinner-container">
                <div class="spinner"></div>
                <span>Fetching slow logs...</span>
              </div>
            </td>
          </tr>
        {:else if logs.length === 0}
          <tr>
            <td colspan="5" class="empty-cell">No slow logs found. Your Redis is running fast! 🚀</td>
          </tr>
        {:else}
          {#each logs as log (log.id)}
            <tr>
              <td class="col-id">#{log.id}</td>
              <td class="col-time">{formatTime(log.timestamp)}</td>
              <td class="col-duration" style:color={log.duration_micros > 50000 ? 'var(--color-error)' : 'inherit'}>
                {formatDuration(log.duration_micros)}
              </td>
              <td class="col-command">
                <code>{log.command}</code>
              </td>
              <td class="col-client">
                {#if log.client_address}
                  <div class="client-addr">{log.client_address}</div>
                {/if}
                {#if log.client_name}
                  <div class="client-name">{log.client_name}</div>
                {/if}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>

<style>
  .slowlog-container {
    padding: 1.5rem 2rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    background: var(--color-bg);
  }

  .header-section {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding-bottom: 1.5rem;
    border-bottom: 1px solid var(--color-border);
  }

  .title-group h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  .subtitle {
    margin: 0.25rem 0 0 0;
    font-size: 0.875rem;
    color: var(--color-muted);
  }

  .controls {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.2s;
  }

  .btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
    transform: translateY(-1px);
  }

  .btn-secondary {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-fg);
  }
  
  .btn-secondary:hover:not(:disabled) {
    background: var(--color-surface);
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    color: var(--color-error);
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid rgba(239, 68, 68, 0.2);
    font-size: 0.875rem;
  }

  .table-container {
    flex: 1;
    overflow-y: auto;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
  }

  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .data-table th {
    position: sticky;
    top: 0;
    background: var(--color-surface-raised);
    color: var(--color-muted);
    font-weight: 600;
    text-align: left;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
    z-index: 10;
  }

  .data-table td {
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-fg);
    vertical-align: middle;
  }

  .data-table tr:hover td {
    background: rgba(255, 255, 255, 0.02);
  }
  
  :global(.light) .data-table tr:hover td {
    background: rgba(0, 0, 0, 0.02);
  }

  .empty-cell {
    text-align: center;
    padding: 3rem !important;
    color: var(--color-muted);
  }

  .spinner-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .col-id {
    color: var(--color-muted);
    font-variant-numeric: tabular-nums;
  }

  .col-time {
    color: var(--color-muted);
    font-size: 0.8em;
  }

  .col-duration {
    text-align: right;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .col-command code {
    font-family: "JetBrains Mono", monospace;
    font-size: 0.875em;
    background: var(--color-surface-input);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    word-break: break-all;
    display: inline-block;
  }

  .col-client {
    font-size: 0.8em;
    color: var(--color-muted);
  }
  
  .client-addr {
    font-family: "JetBrains Mono", monospace;
  }
  
  .client-name {
    margin-top: 0.125rem;
    font-style: italic;
  }
</style>
