<script lang="ts">
  import { onMount } from "svelte";

  let {
    title,
    initialKeyName,
    initialValue,
    onSave,
    onCancel,
  }: {
    title: string;
    initialKeyName?: string;
    initialValue: string;
    onSave: (value: string, newKeyName?: string) => Promise<void>;
    onCancel: () => void;
  } = $props();

  let value = $state("");
  let keyName = $state("");
  let saving = $state(false);
  let error = $state("");
  let format = $state("text");

  onMount(() => {
    value = initialValue;
    keyName = initialKeyName || "";
    if (value.trim() && (value.trim().startsWith("{") || value.trim().startsWith("["))) {
      try {
        JSON.parse(value);
        format = "json-pretty";
      } catch (e) {
        format = "text";
      }
    }
    prevFormat = format;
    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  });

  function applyFormat(fmt: string) {
    error = "";
    try {
      if (fmt === "json-pretty") {
        value = JSON.stringify(JSON.parse(value), null, 2);
      } else if (fmt === "json-minify") {
        value = JSON.stringify(JSON.parse(value));
      } else if (fmt === "to-hex") {
        value = Array.from(new TextEncoder().encode(value))
          .map(b => b.toString(16).padStart(2, '0'))
          .join('');
      } else if (fmt === "from-hex") {
        const hex = value.replace(/\s/g, '');
        if (hex.length % 2 !== 0 || !/^[0-9A-Fa-f]*$/.test(hex)) {
          throw new Error("Invalid hex string");
        }
        const bytes = new Uint8Array(hex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || []);
        value = new TextDecoder().decode(bytes);
      }
    } catch (e) {
      error = "Format error: " + (e instanceof Error ? e.message : String(e));
    }
  }

  let prevFormat = "text";
  $effect(() => {
    if (format !== prevFormat) {
      applyFormat(format);
      prevFormat = format;
    }
  });

  async function handleSave() {
    saving = true;
    error = "";
    try {
      if (initialKeyName !== undefined) {
        if (!keyName.trim()) throw new Error("Key/field name cannot be empty");
        await onSave(value, keyName.trim());
      } else {
        await onSave(value);
      }
    } catch (e) {
      error = String(e);
      saving = false; // only reset if error. on success, modal closes.
    }
  }

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !saving) onCancel();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !saving) {
      onCancel();
    }
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      handleSave();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onBackdropClick}>
  <div class="modal">
    <div class="header">
      <h3 class="title">{title}</h3>
      <button class="close-btn" onclick={onCancel} disabled={saving}>&#10005;</button>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div class="body">
      {#if initialKeyName !== undefined}
        <div class="form-group" style="margin-bottom:1rem;">
          <label for="key-name-input" style="display:block; font-size:0.75rem; font-weight:600; color:var(--color-muted); margin-bottom:0.25rem;">Key / Field Name</label>
          <input id="key-name-input" type="text" class="key-input" bind:value={keyName} disabled={saving} />
        </div>
      {/if}
      <div class="toolbar">
        <label for="format-select-input" style="font-size:0.75rem; font-weight:600; color:var(--color-muted);">Value</label>
        <select id="format-select-input" class="format-select" bind:value={format}>
          <option value="text">Plain Text</option>
          <option value="json-pretty">JSON Pretty</option>
          <option value="json-minify">JSON Minify</option>
          <option value="to-hex">Text &rarr; Hex</option>
          <option value="from-hex">Hex &rarr; Text</option>
        </select>
      </div>
      <textarea
        class="edit-area"
        bind:value
        disabled={saving}
        placeholder="Enter value..."
        spellcheck="false"
      ></textarea>
    </div>

    <div class="footer">
      <span class="hint">Press Cmd+Enter or Ctrl+Enter to save</span>
      <div class="actions">
        <button class="btn" onclick={onCancel} disabled={saving}>Cancel</button>
        <button class="btn btn-primary" onclick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save Changes"}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--color-surface, #1e1e1e);
    border: 1px solid var(--color-border, #333);
    border-radius: 12px;
    width: 600px;
    max-width: 90vw;
    height: 70vh;
    max-height: 800px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border, #333);
  }

  .title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-muted);
    font-size: 1.25rem;
    cursor: pointer;
    line-height: 1;
    padding: 0.25rem;
    border-radius: 4px;
  }

  .close-btn:hover {
    color: var(--color-fg);
    background: var(--color-bg);
  }

  .error {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    color: #f87171;
    padding: 0.75rem 1.25rem;
    font-size: 0.75rem;
    border-bottom: 1px solid var(--color-border);
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1.25rem;
    background: var(--color-bg, #111);
    border-bottom: 1px solid var(--color-border);
  }
  .key-input {
    width: 100%;
    padding: 0.5rem 0.625rem;
    background: var(--color-bg, #111);
    border: 1px solid var(--color-border);
    color: var(--color-fg);
    border-radius: 4px;
    font-family: inherit;
    font-size: 0.8125rem;
  }
  .key-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-height: 0;
    padding: 1rem 1.25rem;
  }


  .format-select {
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    color: var(--color-fg);
    font-size: 0.75rem;
    outline: none;
  }

  .format-select:focus {
    border-color: var(--color-accent, #5b8def);
  }


  .edit-area {
    flex: 1;
    width: 100%;
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    border-radius: 6px;
    color: var(--color-fg);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.75rem;
    padding: 1rem;
    resize: none;
    outline: none;
    line-height: 1.5;
  }

  .edit-area:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--color-border, #333);
    background: var(--color-bg);
    border-radius: 0 0 12px 12px;
  }

  .hint {
    color: var(--color-muted);
    font-size: 0.75rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-border, #333);
    border-radius: 6px;
    background: var(--color-input-bg, #1a1a1a);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 500;
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
</style>
