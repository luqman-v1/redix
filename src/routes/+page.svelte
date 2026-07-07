<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { invoke } from "@tauri-apps/api/core";
  import Layout from "$lib/components/Layout.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import ConnectionList from "$lib/components/ConnectionList.svelte";
  import KeyTree from "$lib/components/KeyTree.svelte";
  import Console from "$lib/components/Console.svelte";
  import ValueViewer from "$lib/components/viewers/ValueViewer.svelte";
  import { connections, activeConnection } from "$lib/stores/connections";
  import { theme } from "$lib/stores/theme";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import type { ConnectionConfig } from "$lib/types/connection";

  let active = $derived(get(activeConnection));
  let selectedKey = $state<string | null>(null);
  let selectedKeyType = $state<string | null>(null);

  onMount(() => {
    connections.load();

    const cleanup = registerShortcuts([
      {
        key: "t",
        ctrl: true,
        action: () => theme.toggle(),
        description: "Toggle theme",
      },
      {
        key: "k",
        ctrl: true,
        action: () => {
          const input = document.querySelector<HTMLInputElement>(
            "[data-console-input]",
          );
          input?.focus();
        },
        description: "Focus console input",
      },
      {
        key: "f",
        ctrl: true,
        action: () => {
          const input = document.querySelector<HTMLInputElement>(
            "[data-key-search]",
          );
          input?.focus();
        },
        description: "Focus key search",
      },
    ]);

    return cleanup;
  });

  async function handleKeySelect(key: string) {
    if (!active) return;
    selectedKey = key;
    selectedKeyType = await invoke<string>("get_key_type", {
      connectionId: active.id,
      key,
    });
  }
</script>

{#snippet sidebar()}
  <div class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Redix</h2>
      <ThemeToggle />
    </div>
    <ConnectionList />
    {#if active}
      <KeyTree
        connectionId={active.id}
        separator={active.key_separator}
        onselect={handleKeySelect}
      />
    {/if}
  </div>
{/snippet}

{#snippet main()}
  {#if active}
    {#if selectedKey && selectedKeyType}
      <div class="key-header">
        <span class="key-name">{selectedKey}</span>
        <span class="key-type-badge">{selectedKeyType}</span>
      </div>
      <ValueViewer type={selectedKeyType} connectionId={active.id} key={selectedKey} />
    {:else}
      <div class="main-active">
        <h3 style:margin="0" style:font-weight="600">{active.name}</h3>
        <p class="active-addr">{active.host}:{active.port} &mdash; db{active.db}</p>
        <p class="active-type">{active.type}</p>
      </div>
    {/if}
  {:else}
    <div class="main-empty">
      Select a connection to browse keys
    </div>
  {/if}
{/snippet}

{#snippet bottom()}
  {#if active}
    <Console connectionId={active.id} />
  {:else}
    <div class="bottom-bar">
      Command console
    </div>
  {/if}
{/snippet}

<ToastContainer />
<Layout {sidebar} {main} {bottom} />

<style>
  .sidebar {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    height: 100%;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sidebar-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-fg);
    margin: 0;
  }

  .main-active {
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--color-fg);
  }

  .active-addr {
    color: var(--color-muted);
    font-size: 0.875rem;
    margin: 0;
  }

  .active-type {
    color: var(--color-muted);
    font-size: 0.8125rem;
    margin: 0;
    text-transform: capitalize;
  }

  .main-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-muted);
  }

  .bottom-bar {
    padding: 0.75rem 1rem;
    height: 100%;
    display: flex;
    align-items: center;
    color: var(--color-muted);
    font-size: 0.875rem;
  }

  .key-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--color-border);
  }

  .key-name {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-fg);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .key-type-badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
    background: var(--color-surface);
    color: var(--color-accent);
    font-weight: 500;
    flex-shrink: 0;
  }
</style>
