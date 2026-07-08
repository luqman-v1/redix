<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let count = $state(0);

  async function load() {
    loading = true;
    error = null;
    try {
      count = await invoke<number>("get_hyperloglog_count", { connectionId, key });
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
    <div class="count-display">
      <span class="count-label">Estimated cardinality</span>
      <span class="count-value">{count.toLocaleString()}</span>
    </div>
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
    font-size: 0.75rem;
    text-align: center;
    padding: 1rem 0;
  }

  .error {
    color: var(--color-error, #e55);
    font-size: 0.75rem;
    padding: 0.25rem 0;
  }

  .count-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.375rem;
    padding: 2rem 0;
  }

  .count-label {
    color: var(--color-muted, #888);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .count-value {
    font-size: 2rem;
    font-weight: 700;
    color: var(--color-fg);
    font-family: monospace;
  }
</style>
