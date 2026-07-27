<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toasts";
  import { onMount, onDestroy } from "svelte";

  interface Props {
    connectionId: string;
    onselect: (key: string) => void;
  }
  let { connectionId, onselect }: Props = $props();

  interface MemoryKeyInfo {
    key: string;
    bytes: number;
    key_type: string;
  }

  interface MemoryAnalysisResult {
    total_keys_scanned: number;
    top_keys: MemoryKeyInfo[];
  }

  let analyzing = $state(false);
  let sampleSize = $state(10000);
  let results = $state<MemoryAnalysisResult | null>(null);
  let totalTopKeysMemory = $derived(results ? results.top_keys.reduce((acc, k) => acc + k.bytes, 0) : 0);
  let error = $state<string | null>(null);

  async function startAnalysis() {
    analyzing = true;
    error = null;
    try {
      results = await invoke<MemoryAnalysisResult>("analyze_memory", {
        connectionId,
        sampleSize: Number(sampleSize)
      });
    } catch (e) {
      error = String(e);
      toasts.add("Failed to analyze memory: " + error, "error");
    } finally {
      analyzing = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function getTypeColor(type: string) {
    switch (type) {
      case "string": return "var(--color-accent)";
      case "hash": return "#eab308";
      case "list": return "#10b981";
      case "set": return "#8b5cf6";
      case "zset": return "#f43f5e";
      default: return "var(--color-muted)";
    }
  }
</script>

<div class="analyzer-container">
  <div class="header-section">
    <div class="title-group">
      <h2>Memory Analyzer</h2>
      <p class="subtitle">Identify memory-hungry keys in your database.</p>
    </div>
    
    <div class="controls">
      <div class="input-group">
        <label for="sample-size">Sample Size (keys)</label>
        <select id="sample-size" bind:value={sampleSize} class="select-input" disabled={analyzing}>
          <option value={1000}>1,000</option>
          <option value={10000}>10,000</option>
          <option value={50000}>50,000</option>
          <option value={100000}>100,000</option>
        </select>
      </div>
      <button class="btn btn-primary" onclick={startAnalysis} disabled={analyzing}>
        {analyzing ? "Analyzing..." : (results ? "Re-Analyze" : "Start Analysis")}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if analyzing}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Scanning keys and calculating memory usage...</p>
      <p style="font-size: 0.75rem; color: var(--color-muted); margin-top: 0.5rem;">This may take a while depending on the sample size.</p>
    </div>
  {:else if results}
    <div class="results-section">
      <div class="stats-cards">
        <div class="stat-card">
          <span class="stat-label">Keys Scanned</span>
          <span class="stat-value">{results.total_keys_scanned.toLocaleString()}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">Top Keys Found</span>
          <span class="stat-value">{results.top_keys.length}</span>
        </div>
        <div class="stat-card">
          <span class="stat-label">Total Memory (Top Keys)</span>
          <span class="stat-value">{formatBytes(totalTopKeysMemory)}</span>
        </div>
      </div>

      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th style="width: 50px;">#</th>
              <th>Key Name</th>
              <th style="width: 100px;">Type</th>
              <th style="width: 120px; text-align: right;">Size</th>
              <th style="width: 60px;"></th>
            </tr>
          </thead>
          <tbody>
            {#each results.top_keys as key, i (key.key)}
              <tr onclick={() => onselect(key.key)}>
                <td class="col-index">{i + 1}</td>
                <td class="col-key"><code>{key.key}</code></td>
                <td>
                  <span class="type-badge" style:border-color={getTypeColor(key.key_type)} style:color={getTypeColor(key.key_type)}>
                    {key.key_type.toUpperCase()}
                  </span>
                </td>
                <td class="col-size" style="text-align: right; font-weight: 600;">{formatBytes(key.bytes)}</td>
                <td class="col-actions">
                  <button class="icon-btn" onclick={(e) => { e.stopPropagation(); onselect(key.key); }} title="Open in Tab">
                    &#8594;
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="opacity: 0.5; margin-bottom: 1rem;">
        <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
        <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
        <line x1="6" y1="6" x2="6.01" y2="6"></line>
        <line x1="6" y1="18" x2="6.01" y2="18"></line>
      </svg>
      <p>Click "Start Analysis" to find out which keys are consuming the most memory.</p>
    </div>
  {/if}
</div>

<style>
  .analyzer-container {
    padding: 1.5rem 2rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    overflow-y: auto;
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
    align-items: flex-end;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .input-group label {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-muted);
  }

  .select-input {
    background: var(--color-surface-input);
    color: var(--color-fg);
    border: 1px solid var(--color-border);
    padding: 0.5rem 2rem 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.875rem;
    cursor: pointer;
    appearance: none;
    background-image: url('data:image/svg+xml;utf8,<svg fill="%239ca3af" height="24" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg"><path d="M7 10l5 5 5-5z"/></svg>');
    background-repeat: no-repeat;
    background-position: right 0.25rem center;
    background-size: 1.2rem;
  }
  
  .select-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
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

  .btn-primary:disabled {
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

  .loading-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-fg);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-muted);
    font-size: 0.875rem;
  }

  .results-section {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    flex: 1;
    min-height: 0;
  }

  .stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .stat-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-label {
    font-size: 0.75rem;
    color: var(--color-muted);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--color-fg);
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
  }

  .data-table tr {
    transition: background-color 0.15s;
    cursor: pointer;
  }

  .data-table tr:hover {
    background: rgba(255, 255, 255, 0.02);
  }
  
  :global(.light) .data-table tr:hover {
    background: rgba(0, 0, 0, 0.02);
  }

  .col-index {
    color: var(--color-muted);
    font-variant-numeric: tabular-nums;
  }

  .col-key code {
    font-family: "JetBrains Mono", monospace;
    font-size: 0.875em;
    background: var(--color-surface-input);
    padding: 0.125rem 0.25rem;
    border-radius: 4px;
    word-break: break-all;
  }

  .type-badge {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.125rem 0.375rem;
    border: 1px solid currentColor;
    border-radius: 4px;
    background: transparent;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1rem;
    padding: 0.25rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .icon-btn:hover {
    background: var(--color-surface-btn-hover);
    color: var(--color-fg);
  }
</style>
