<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { beautify } from "$lib/utils/beautifier";
  import { highlightSyntax } from "$lib/utils/highlighter";
  import EditModal from "../EditModal.svelte";
  import { toasts } from "$lib/stores/toasts";

  interface Props {
    connectionId: string;
    key: string;
  }

  let { connectionId, key }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let rawValue = $state("");
  let formatted = $state("");
  let format = $state<string>("text");
  let highlighted = $derived(highlightSyntax(formatted, format));
  let editing = $state(false);
  let editValue = $state("");
  let saving = $state(false);

  async function load() {
    loading = true;
    error = null;
    try {
      rawValue = await invoke<string>("get_string_value", { connectionId, key });
      const result = beautify(rawValue);
      formatted = result.formatted;
      format = result.format;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function save(newValue: string) {
    saving = true;
    error = null;
    try {
      await invoke("set_string_value", { connectionId, key, value: newValue });
      rawValue = newValue;
      const result = beautify(newValue);
      formatted = result.formatted;
      format = result.format;
      editing = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      throw e; // throw so the modal stays open if there's an error
    } finally {
      saving = false;
    }
  }

  function startEdit() {
    editing = true;
  }

  function cancelEdit() {
    editing = false;
  }

  async function copyValue() {
    try {
      await navigator.clipboard.writeText(rawValue);
      toasts.add("Value copied to clipboard", "success");
    } catch (e) {
      toasts.add("Failed to copy value", "error");
    }
  }

  $effect(() => {
    if (connectionId && key) {
      editing = false;
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
      <span class="format-badge" data-format={format}>{format}</span>
      <button class="btn" onclick={copyValue}>Copy</button>
      {#if !editing}
        <button class="btn" onclick={startEdit}>Edit</button>
      {/if}
    </div>

    {#if editing}
      <EditModal
        title="Edit String Value"
        initialValue={rawValue}
        onSave={save}
        onCancel={cancelEdit}
      />
    {/if}

    <pre class="value-block"><code>{@html highlighted}</code></pre>
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
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .format-badge {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    background: var(--color-surface-raised, #222);
    color: var(--color-muted, #888);
    border: 1px solid var(--color-border, #333);
    margin-right: auto;
  }

  .format-badge[data-format="json"] {
    color: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
  }

  .format-badge[data-format="xml"] {
    color: #c08;
    border-color: #c08;
  }

  .format-badge[data-format="binary"] {
    color: #e8a427;
    border-color: #e8a427;
  }

  .btn {
    padding: 0.25rem 0.625rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.75rem;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .value-block {
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
    border-radius: 8px;
    padding: 1.25rem;
    overflow: auto;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.75rem;
    line-height: 1.6;
    color: var(--color-fg);
    white-space: pre-wrap;
    word-break: break-all;
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
  }

  /* Syntax Highlighting Themes */
  :global(.hl-key) { color: #9cdcfe; }
  :global(.hl-string) { color: #ce9178; }
  :global(.hl-number) { color: #b5cea8; }
  :global(.hl-boolean) { color: #569cd6; }
  :global(.hl-null) { color: #569cd6; font-style: italic; }
  :global(.hl-tag) { color: #569cd6; }
  :global(.hl-attr) { color: #9cdcfe; }

  :global(.light .hl-key) { color: #1d4ed8; }
  :global(.light .hl-string) { color: #b91c1c; }
  :global(.light .hl-number) { color: #15803d; }
  :global(.light .hl-boolean) { color: #0369a1; }
  :global(.light .hl-null) { color: #0369a1; }
  :global(.light .hl-tag) { color: #991b1b; }
  :global(.light .hl-attr) { color: #e11d48; }
</style>
