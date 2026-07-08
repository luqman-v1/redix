<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { disconnectFromServer, activeConnection } from "$lib/stores/connections";

  let { connectionId, name }: { connectionId: string; name: string } = $props();

  let info = $state<Record<string, string> | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let timer: ReturnType<typeof setInterval>;

  async function loadInfo() {
    try {
      info = await invoke("get_server_info", { connectionId });
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleDisconnect() {
    await disconnectFromServer(connectionId);
    activeConnection.set(null);
  }

  onMount(() => {
    loadInfo();
    timer = setInterval(loadInfo, 5000); // refresh every 5s
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  function formatBytes(bytesStr: string | undefined) {
    if (!bytesStr) return "0 B";
    const bytes = parseInt(bytesStr, 10);
    if (isNaN(bytes)) return bytesStr;
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function parseKeyspace(ks: string | undefined) {
    if (!ks) return "0";
    // keys=123,expires=10,avg_ttl=100
    const keysMatch = ks.match(/keys=(\d+)/);
    return keysMatch ? keysMatch[1] : "0";
  }

  let totalKeys = $derived(
    info ? Object.keys(info)
      .filter(k => k.startsWith("db"))
      .reduce((acc, k) => acc + parseInt(parseKeyspace(info![k])), 0)
    : 0
  );

  function formatUptime(secondsStr: string | undefined) {
    if (!secondsStr) return "0s";
    const totalSeconds = parseInt(secondsStr, 10);
    if (isNaN(totalSeconds)) return secondsStr;
    
    const d = Math.floor(totalSeconds / 86400);
    const h = Math.floor((totalSeconds % 86400) / 3600);
    const m = Math.floor((totalSeconds % 3600) / 60);
    const s = totalSeconds % 60;
    
    const parts = [];
    if (d > 0) parts.push(`${d}d`);
    if (h > 0) parts.push(`${h}h`);
    if (m > 0) parts.push(`${m}m`);
    parts.push(`${s}s`);
    
    return parts.join(' ');
  }
</script>

<div class="dashboard">
  <div class="header">
    <div style="display:flex; align-items:center; gap:1rem;">
      <h2 class="title">Dashboard - {name}</h2>
      {#if loading && !info}
        <span class="loading">Loading...</span>
      {/if}
    </div>
    <button class="disconnect-btn" onclick={handleDisconnect} title="Disconnect from this server">
      Disconnect
    </button>
  </div>

  {#if error}
    <div class="error">
      Failed to load server info: {error}
    </div>
  {/if}

  {#if info}
    <div class="metrics-grid">
      <div class="metric-card">
        <span class="metric-label">Memory Usage</span>
        <span class="metric-value highlight">{formatBytes(info.used_memory)}</span>
        <span class="metric-sub">Peak: {formatBytes(info.used_memory_peak)}</span>
      </div>
      
      <div class="metric-card">
        <span class="metric-label">Total Keys</span>
        <span class="metric-value">{totalKeys.toLocaleString()}</span>
        <span class="metric-sub">Across all databases</span>
      </div>

      <div class="metric-card">
        <span class="metric-label">Connected Clients</span>
        <span class="metric-value">{info.connected_clients || '0'}</span>
        <span class="metric-sub">Blocked: {info.blocked_clients || '0'}</span>
      </div>

      <div class="metric-card">
        <span class="metric-label">Uptime</span>
        <span class="metric-value">{formatUptime(info.uptime_in_seconds)}</span>
        <span class="metric-sub">Up since start</span>
      </div>
    </div>

    <div class="info-section">
      <h3 class="section-title">Server Details</h3>
      <table class="info-table">
        <tbody>
          <tr><td>Version</td><td>{info.redis_version || 'N/A'}</td></tr>
          <tr><td>OS</td><td>{info.os || 'N/A'}</td></tr>
          <tr><td>Mode</td><td>{info.redis_mode || 'standalone'}</td></tr>
          <tr><td>Role</td><td>{info.role || 'master'}</td></tr>
          <tr><td>TCP Port</td><td>{info.tcp_port || '6379'}</td></tr>
          <tr><td>Executable</td><td>{info.executable || 'N/A'}</td></tr>
          <tr><td>Config File</td><td>{info.config_file || 'N/A'}</td></tr>
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    color: var(--color-fg);
    overflow-y: auto;
    height: 100%;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .disconnect-btn {
    background: color-mix(in srgb, var(--color-error) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
    color: var(--color-error);
    padding: 0.375rem 0.75rem;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .disconnect-btn:hover {
    background: color-mix(in srgb, var(--color-error) 20%, transparent);
    border-color: color-mix(in srgb, var(--color-error) 50%, transparent);
    box-shadow: 0 0 12px color-mix(in srgb, var(--color-error) 20%, transparent);
  }

  .title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    background: linear-gradient(90deg, #fff, #a0a0a0);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  :global(.light) .title {
    background: linear-gradient(90deg, #111, #555);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .loading {
    font-size: 0.875rem;
    color: var(--color-muted);
  }

  .error {
    background: color-mix(in srgb, var(--color-error, #ef4444) 15%, transparent);
    color: var(--color-error, #ef4444);
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--color-error, #ef4444) 30%, transparent);
    font-size: 0.875rem;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .metric-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    backdrop-filter: blur(10px);
    transition: transform 0.2s, background 0.2s;
  }

  :global(.light) .metric-card {
    background: rgba(0, 0, 0, 0.02);
    border: 1px solid rgba(0, 0, 0, 0.05);
  }

  .metric-card:hover {
    transform: translateY(-2px);
    background: rgba(255, 255, 255, 0.05);
  }

  :global(.light) .metric-card:hover {
    background: rgba(0, 0, 0, 0.04);
  }

  .metric-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted);
    font-weight: 600;
  }

  .metric-value {
    font-size: 2rem;
    font-weight: 700;
    line-height: 1.2;
  }
  
  .metric-value.highlight {
    color: var(--color-accent);
  }

  .metric-sub {
    font-size: 0.75rem;
    color: var(--color-muted);
    opacity: 0.8;
  }

  .info-section {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 1.5rem;
  }

  :global(.light) .info-section {
    background: rgba(0, 0, 0, 0.01);
    border: 1px solid rgba(0, 0, 0, 0.05);
  }

  .section-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .info-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .info-table td {
    padding: 0.75rem 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  
  :global(.light) .info-table td {
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  }

  .info-table tr:last-child td {
    border-bottom: none;
  }

  .info-table td:first-child {
    color: var(--color-muted);
    width: 200px;
  }

  .info-table td:last-child {
    font-family: ui-monospace, SFMono-Regular, monospace;
  }
</style>
