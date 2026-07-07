<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { beautify } from "$lib/utils/beautifier";

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

  async function save() {
    saving = true;
    error = null;
    try {
      await invoke("set_string_value", { connectionId, key, value: editValue });
      rawValue = editValue;
      const result = beautify(editValue);
      formatted = result.formatted;
      format = result.format;
      editing = false;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function startEdit() {
    editValue = rawValue;
    editing = true;
  }

  function cancelEdit() {
    editing = false;
    editValue = "";
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
      {#if !editing}
        <button class="btn" onclick={startEdit}>Edit</button>
      {/if}
    </div>

    {#if editing}
      <textarea
        class="edit-area"
        bind:value={editValue}
        rows="12"
      ></textarea>
      <div class="edit-actions">
        <button class="btn btn-primary" onclick={save} disabled={saving}>
          {saving ? "Saving..." : "Save"}
        </button>
        <button class="btn" onclick={cancelEdit}>Cancel</button>
      </div>
    {:else}
      <pre class="value-block"><code>{formatted}</code></pre>
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

  .btn-primary {
    background: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
    color: #fff;
  }

  .value-block {
    background: var(--color-surface-raised, #222);
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    padding: 0.75rem;
    overflow: auto;
    font-size: 0.8125rem;
    line-height: 1.5;
    color: var(--color-fg);
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .edit-area {
    width: 100%;
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    color: var(--color-fg);
    font-family: monospace;
    font-size: 0.8125rem;
    padding: 0.75rem;
    resize: vertical;
    outline: none;
  }

  .edit-area:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .edit-actions {
    display: flex;
    gap: 0.375rem;
  }
</style>
