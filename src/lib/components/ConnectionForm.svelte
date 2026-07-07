<script lang="ts">
  import type { ConnectionConfig, ConnectionType } from "$lib/types/connection";
  import { connections } from "$lib/stores/connections";

  let {
    editing = null,
    onclose,
  }: {
    editing?: ConnectionConfig | null;
    onclose: () => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  const e = editing;
  let name = $state(e?.name ?? "");
  let host = $state(e?.host ?? "127.0.0.1");
  let port = $state(e?.port ?? 6379);
  let password = $state(e?.password ?? "");
  let connectionType = $state<ConnectionType>(e?.type ?? "standalone");
  let db = $state(e?.db ?? 0);
  let error = $state("");

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = "";

    if (!name.trim()) {
      error = "Name is required";
      return;
    }
    if (!host.trim()) {
      error = "Host is required";
      return;
    }

    try {
      if (editing) {
        await connections.update({
          ...editing,
          name: name.trim(),
          host: host.trim(),
          port,
          password: password || undefined,
          type: connectionType,
          db,
        });
      } else {
        await connections.add({
          name: name.trim(),
          host: host.trim(),
          port,
          password: password || undefined,
          type: connectionType,
          db,
          key_separator: ":",
          readonly: false,
        });
      }
      onclose();
    } catch (err) {
      error = String(err);
    }
  }

  function onBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onclose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onBackdropClick}>
  <form class="modal" onsubmit={handleSubmit}>
    <h3 style:margin="0 0 1rem 0" style:font-size="1.125rem" style:font-weight="600">
      {editing ? "Edit Connection" : "New Connection"}
    </h3>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <label class="field">
      <span class="label">Name</span>
      <input type="text" bind:value={name} placeholder="My Redis" />
    </label>

    <div class="row">
      <label class="field" style:flex="3">
        <span class="label">Host</span>
        <input type="text" bind:value={host} placeholder="127.0.0.1" />
      </label>
      <label class="field" style:flex="1">
        <span class="label">Port</span>
        <input type="number" bind:value={port} min="1" max="65535" />
      </label>
    </div>

    <label class="field">
      <span class="label">Password</span>
      <input type="password" bind:value={password} placeholder="(optional)" />
    </label>

    <div class="row">
      <label class="field" style:flex="1">
        <span class="label">Type</span>
        <select bind:value={connectionType}>
          <option value="standalone">Standalone</option>
          <option value="cluster">Cluster</option>
          <option value="sentinel">Sentinel</option>
        </select>
      </label>
      <label class="field" style:flex="1">
        <span class="label">Database</span>
        <input type="number" bind:value={db} min="0" max="15" />
      </label>
    </div>

    <div class="actions">
      <button type="button" class="btn-secondary" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn-primary">
        {editing ? "Save" : "Add"}
      </button>
    </div>
  </form>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--color-surface);
    color: var(--color-fg);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 1.5rem;
    width: 24rem;
    max-width: 90vw;
    max-height: 90vh;
    overflow-y: auto;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
  }

  .label {
    font-size: 0.8125rem;
    color: var(--color-muted);
    font-weight: 500;
  }

  input,
  select {
    background: var(--color-bg);
    color: var(--color-fg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 0.5rem 0.625rem;
    font-size: 0.875rem;
    outline: none;
  }

  input:focus,
  select:focus {
    border-color: var(--color-accent, #6366f1);
  }

  .row {
    display: flex;
    gap: 0.75rem;
  }

  .error {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    color: #f87171;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.8125rem;
    margin-bottom: 0.75rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .btn-secondary {
    background: var(--color-bg);
    color: var(--color-fg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-primary {
    background: var(--color-accent, #6366f1);
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    cursor: pointer;
    font-weight: 500;
  }
</style>
