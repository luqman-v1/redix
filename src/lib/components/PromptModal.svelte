<script lang="ts">
  import { onMount } from "svelte";

  let {
    title,
    initialValue = "",
    placeholder = "",
    saveText = "Save",
    savingText = "Saving...",
    onSave,
    onCancel,
  }: {
    title: string;
    initialValue?: string;
    placeholder?: string;
    saveText?: string;
    savingText?: string;
    onSave: (value: string) => Promise<void>;
    onCancel: () => void;
  } = $props();

  let value = $state("");
  onMount(() => {
    value = initialValue;
  });
  let saving = $state(false);
  let error = $state("");

  async function handleSave() {
    if (!value.trim()) return;
    saving = true;
    error = "";
    try {
      await onSave(value.trim());
    } catch (e) {
      error = String(e);
      saving = false;
    }
  }

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !saving) onCancel();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !saving) {
      onCancel();
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  });
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
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        class="input-field"
        bind:value
        disabled={saving}
        {placeholder}
        onkeydown={(e) => e.key === "Enter" && handleSave()}
        autofocus
      />
    </div>

    <div class="footer">
      <div class="actions">
        <button class="btn" onclick={onCancel} disabled={saving}>Cancel</button>
        <button class="btn btn-primary" onclick={handleSave} disabled={saving || !value.trim()}>
          {saving ? savingText : saveText}
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
    width: 400px;
    max-width: 90vw;
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

  .body {
    padding: 1.25rem;
  }

  .input-field {
    width: 100%;
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    border-radius: 6px;
    color: var(--color-fg);
    font-size: 0.75rem;
    padding: 0.625rem 0.75rem;
    outline: none;
    box-sizing: border-box;
  }

  .input-field:focus {
    border-color: var(--color-accent, #5b8def);
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--color-border, #333);
    background: var(--color-bg);
    border-radius: 0 0 12px 12px;
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
