<script lang="ts">
  import { onMount } from "svelte";
  import Layout from "$lib/components/Layout.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import ConnectionList from "$lib/components/ConnectionList.svelte";
  import { connections, activeConnection } from "$lib/stores/connections";
  import type { ConnectionConfig } from "$lib/types/connection";

  let active = $state<ConnectionConfig | null>(null);
  activeConnection.subscribe((v) => (active = v));

  onMount(() => {
    connections.load();
  });
</script>

{#snippet sidebar()}
  <div class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Redix</h2>
      <ThemeToggle />
    </div>
    <ConnectionList />
  </div>
{/snippet}

{#snippet main()}
  {#if active}
    <div class="main-active">
      <h3 style:margin="0" style:font-weight="600">{active.name}</h3>
      <p class="active-addr">{active.host}:{active.port} &mdash; db{active.db}</p>
      <p class="active-type">{active.type}</p>
    </div>
  {:else}
    <div class="main-empty">
      Select a connection to browse keys
    </div>
  {/if}
{/snippet}

{#snippet bottom()}
  <div class="bottom-bar">
    Command console
  </div>
{/snippet}

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
</style>
