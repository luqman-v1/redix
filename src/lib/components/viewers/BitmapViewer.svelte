<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toHex } from "$lib/utils/beautifier";

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let rawValue = $state("");
  let hexValue = $state("");

  async function load() {
    loading = true;
    error = null;
    try {
      rawValue = await invoke<string>("get_string_value", { connectionId, key });
      hexValue = toHex(rawValue);
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
      <span class="size-label">{rawValue.length} bytes</span>
    </div>
    <pre class="hex-block"><code>{hexValue}</code></pre>
    <p class="note">Full bitmap visualization not yet implemented. Showing hex dump.</p>
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

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }

  .size-label {
    color: var(--color-muted, #888);
    font-size: 0.6875rem;
  }

  .hex-block {
    background: var(--color-surface-raised, #222);
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    padding: 0.75rem;
    overflow: auto;
    font-size: 0.75rem;
    line-height: 1.6;
    color: var(--color-fg);
    margin: 0;
    word-break: break-all;
  }

  .note {
    color: var(--color-muted, #888);
    font-size: 0.6875rem;
    font-style: italic;
  }
</style>
