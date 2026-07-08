<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { listen } from "@tauri-apps/api/event";
  import Layout from "$lib/components/Layout.svelte";
  import ToastContainer from "$lib/components/ToastContainer.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import ConnectionList from "$lib/components/ConnectionList.svelte";
  import KeyTree from "$lib/components/KeyTree.svelte";
  import Console from "$lib/components/Console.svelte";
  import ValueViewer from "$lib/components/viewers/ValueViewer.svelte";
  import PromptModal from "$lib/components/PromptModal.svelte";
  import AddKeyModal from "$lib/components/AddKeyModal.svelte";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import { toasts } from "$lib/stores/toasts";
  import { connections, activeConnection, disconnectFromServer } from "$lib/stores/connections";
  import { theme } from "$lib/stores/theme";
  import { registerShortcuts } from "$lib/utils/shortcuts";
  import type { ConnectionConfig } from "$lib/types/connection";

  let active = $state<ConnectionConfig | null>(null);
  interface Tab { key: string; type: string | null; ttl: number | null; }
  let openTabs = $state<Tab[]>([]);
  let activeTabIndex = $state(0);
  let appVersion = $state("");
  let selectedKey = $derived(openTabs[activeTabIndex]?.key ?? null);
  let selectedKeyType = $derived(openTabs[activeTabIndex]?.type ?? null);
  let selectedKeyTtl = $derived(openTabs[activeTabIndex]?.ttl ?? null);
  let showConnections = $state(false);
  
  interface CommandLog { command: string; duration: number; timestamp: number; }
  let commandLogs = $state<CommandLog[]>([]);
  let showCommandLog = $state(false);
  
  let tabContextMenu = $state<{ x: number, y: number, index: number } | null>(null);

  function handleTabContextMenu(e: MouseEvent, index: number) {
    e.preventDefault();
    tabContextMenu = { x: e.clientX, y: e.clientY, index };
  }

  function closeAllTabs() {
    openTabs = [];
    activeTabIndex = 0;
    tabContextMenu = null;
  }

  function closeOtherTabs() {
    if (tabContextMenu) {
      openTabs = [openTabs[tabContextMenu.index]];
      activeTabIndex = 0;
      tabContextMenu = null;
    }
  }

  $effect(() => {
    function handleClick() { tabContextMenu = null; }
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  });

  function openConsoleTab() {
    let idx = openTabs.findIndex(t => t.key === '__REDIS_CONSOLE__');
    if (idx === -1) {
      openTabs.push({ key: '__REDIS_CONSOLE__', type: null, ttl: null });
      activeTabIndex = openTabs.length - 1;
    } else {
      activeTabIndex = idx;
    }
  }

  let renamingKey = $state(false);
  let deletingKey = $state(false);
  let addingKey = $state(false);
  let lookingUpKey = $state(false);
  let editingTtl = $state(false);
  let refreshKeyCount = $state(0);

  // show connections by default if there's no active connection
  $effect(() => {
    if (!active && !showConnections) {
      showConnections = true;
    }
  });

  $effect(() => {
    const unsub = activeConnection.subscribe((v) => { active = v; });
    return unsub;
  });

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
          openConsoleTab();
          // wait a tick for the console to mount, then focus
          requestAnimationFrame(() => {
            const input = document.querySelector<HTMLInputElement>("[data-console-input]");
            input?.focus();
          });
        },
        description: "Open console",
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

    getVersion().then(v => {
      appVersion = v;
    });


    let unlistenLog: () => void;
    listen<{command: string, duration: number}>("command-log", (e) => {
      commandLogs.push({
        command: e.payload.command,
        duration: e.payload.duration,
        timestamp: Date.now()
      });
      if (commandLogs.length > 200) commandLogs.shift();
    }).then(fn => unlistenLog = fn);

    return () => {
      cleanup();
      if (unlistenLog) unlistenLog();
    };
  });

  async function handleKeySelect(key: string) {
    if (!active) return;
    try {
      const typeStr = await invoke<string>("get_key_type", {
        connectionId: active.id,
        key,
      });
      const ttlVal = await invoke<number>("get_key_ttl", {
        connectionId: active.id,
        key,
      });
      
      const existingIdx = openTabs.findIndex(t => t.key === key);
      if (existingIdx >= 0) {
        openTabs[existingIdx].type = typeStr;
        openTabs[existingIdx].ttl = ttlVal;
        activeTabIndex = existingIdx;
      } else {
        openTabs.push({ key, type: typeStr, ttl: ttlVal });
        activeTabIndex = openTabs.length - 1;
      }
    } catch (e) {
      toasts.add(String(e), "error");
    }
  }

  async function handleDisconnect() {
    if (active) {
      await disconnectFromServer(active.id);
      activeConnection.set(null);
      openTabs = [];
      activeTabIndex = 0;
    }
  }

  async function handleRename(newName: string) {
    if (!active || !selectedKey) return;
    try {
      await invoke("rename_key", {
        connectionId: active.id,
        oldName: selectedKey,
        newName,
      });
      openTabs[activeTabIndex].key = newName;
      renamingKey = false;
    } catch (e) {
      toasts.add(String(e), "error");
      throw e;
    }
  }

  async function handleSetTtl(newTtlStr: string) {
    if (!active || !selectedKey) return;
    try {
      const parsed = parseInt(newTtlStr, 10);
      if (isNaN(parsed)) throw new Error("Invalid TTL number");
      
      await invoke("set_key_ttl", {
        connectionId: active.id,
        key: selectedKey,
        ttl: parsed,
      });
      
      const newTtl = await invoke<number>("get_key_ttl", {
        connectionId: active.id,
        key: selectedKey,
      });
      
      openTabs[activeTabIndex].ttl = newTtl;
      editingTtl = false;
      toasts.add("TTL updated successfully", "success");
    } catch (e) {
      toasts.add(String(e), "error");
      throw e;
    }
  }

  async function handleDeleteKey() {
    if (!active || !selectedKey) return;
    try {
      await invoke("delete_key", {
        connectionId: active.id,
        key: selectedKey,
      });
      openTabs.splice(activeTabIndex, 1);
      if (activeTabIndex >= openTabs.length) {
        activeTabIndex = Math.max(0, openTabs.length - 1);
      }
      deletingKey = false;
    } catch (e) {
      toasts.add(String(e), "error");
    }
  }

  async function handleRefreshKey() {
    if (!active || !selectedKey) return;
    try {
      const typeStr = await invoke<string>("get_key_type", {
        connectionId: active.id,
        key: selectedKey,
      });
      const ttlVal = await invoke<number>("get_key_ttl", {
        connectionId: active.id,
        key: selectedKey,
      });
      openTabs[activeTabIndex].type = typeStr;
      openTabs[activeTabIndex].ttl = ttlVal;
      refreshKeyCount++;
    } catch(e) {
      toasts.add("Failed to refresh key: " + String(e), "error");
    }
  }

  async function changeDb(e: Event) {
    if (!active) return;
    const newDb = Number((e.target as HTMLSelectElement).value);
    if (active.db === newDb) return;
    try {
      const updated = { ...active, db: newDb };
      await connections.save(updated);
      await invoke("reconnect", { connectionId: active.id });
      activeConnection.set(updated);
      openTabs = [];
      activeTabIndex = 0;
    } catch (err) {
      toasts.add(String(err), "error");
    }
  }

  function handleKeyAdded(keyName: string) {
    if (!active) return;
    
    addingKey = false;
    // We just wait for the user to click the key in the tree or we can forcefully reload the tree, 
    // but setting selectedKey isn't enough to fetch the type automatically unless handleKeySelect is called.
    handleKeySelect(keyName);
  }

  function closeTab(index: number) {
    openTabs.splice(index, 1);
    if (activeTabIndex >= openTabs.length) {
      activeTabIndex = Math.max(0, openTabs.length - 1);
    } else if (activeTabIndex > index) {
      activeTabIndex--;
    }
  }

  async function copyKeyToClipboard() {
    if (!selectedKey) return;
    try {
      await navigator.clipboard.writeText(selectedKey);
      toasts.add("Key copied to clipboard", "success");
    } catch (e) {
      toasts.add("Failed to copy key", "error");
    }
  }
</script>

{#snippet sidebar()}
  <div class="sidebar">
    <div class="sidebar-header">
      <div class="conn-group" style="flex: 1; margin-right: 0.5rem;">
        <button class="conn-selector" onclick={() => showConnections = true} title={active?.name}>
          <span class="conn-name">{active ? active.name : "Select Connection..."}</span>
          <span style="font-size:0.7em; opacity:0.7;">▼</span>
        </button>
        {#if active}
          <select class="db-selector" value={active.db} onchange={changeDb} title="Select Database">
            {#each Array(16) as _, i}
              <option value={i}>DB {i}</option>
            {/each}
          </select>
        {/if}
      </div>
      <ThemeToggle />
    </div>
    {#if active}
      <div class="sidebar-actions">
        <button class="action-btn" onclick={() => addingKey = true} title="Create a new key">+ New Key</button>
        <button class="action-btn" onclick={() => lookingUpKey = true} title="Lookup an exact key">🔍 Lookup</button>
        
        <button 
          class="action-btn" 
          style="flex: 0 0 auto; padding: 0.375rem 0.5rem;"
          title="Open Console (Ctrl+K)"
          onclick={openConsoleTab}
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="13 17 18 12 13 7"></polyline>
            <line x1="6" y1="17" x2="11" y2="12"></line>
            <line x1="6" y1="7" x2="11" y2="12"></line>
          </svg>
        </button>
        
      </div>
      <KeyTree
        connectionId={active.id}
        separator={active.key_separator}
        onselect={handleKeySelect}
      />
      
      <div style="font-size: 0.65rem; color: var(--color-muted); text-align: center; padding-top: 0.5rem; opacity: 0.6; pointer-events: none;">
        Redix v{appVersion}
      </div>
    {/if}
  </div>
{/snippet}

{#snippet main()}
  {#if active}
    {#if selectedKey}
      {#if openTabs.length > 0}
        <div class="tabs-bar">
          {#each openTabs as tab, i}
            <div 
              class="tab" 
              class:active={i === activeTabIndex} 
              onclick={() => activeTabIndex = i}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') activeTabIndex = i; }}
              oncontextmenu={(e) => handleTabContextMenu(e, i)}
              role="tab"
              tabindex="0"
              aria-selected={i === activeTabIndex}
            >
              {#if tab.key === '__REDIS_CONSOLE__'}
                <span class="tab-title" title="Console">📺 Console</span>
              {:else}
                <span class="tab-title" title={tab.key}>{tab.key.split(active.key_separator || ":").pop() || tab.key}</span>
              {/if}
              <button class="tab-close" onclick={(e) => { e.stopPropagation(); closeTab(i); }}>&times;</button>
            </div>
          {/each}
        </div>
      {/if}

      {#if tabContextMenu}
        <div class="context-menu" style="left: {tabContextMenu.x}px; top: {tabContextMenu.y}px;">
          <button class="ctx-item" onclick={closeOtherTabs}>Close Others</button>
          <button class="ctx-item" onclick={closeAllTabs}>Close All</button>
        </div>
      {/if}

      {#if selectedKey === '__REDIS_CONSOLE__'}
        <div style="flex: 1; display: flex; flex-direction: column; overflow: hidden; background: #000; padding: 0.5rem 0 0 0;">
          <Console connectionId={active.id} onclose={() => closeTab(activeTabIndex)} />
        </div>
      {:else if selectedKeyType}
        {@const parts = selectedKey.split(active.key_separator || ":")}
        <div class="key-header">
          <div class="key-breadcrumb-wrap">
            <div class="key-breadcrumb" style="user-select: all;">{#each parts as part, i}<span class="breadcrumb-part">{part}</span>{#if i < parts.length - 1}<span class="breadcrumb-sep">{active.key_separator || ":"}</span>{/if}{/each}</div>
            <button class="icon-btn" onclick={copyKeyToClipboard} title="Copy full key path">
              <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
            </button>
          </div>
          <div class="key-meta">
            <span class="key-type-badge">{selectedKeyType}</span>
            {#if selectedKeyTtl !== null}
              <button class="key-ttl-badge clickable" title="Click to edit TTL" onclick={() => editingTtl = true}>
                TTL: {selectedKeyTtl < 0 ? "Infinite" : `${selectedKeyTtl}s`}
              </button>
            {/if}
            <div class="key-actions">
              <button class="icon-btn" onclick={handleRefreshKey} title="Refresh Data">
                <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path><path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path></svg>
              </button>
              <button class="icon-btn" onclick={() => renamingKey = true} title="Rename Key">&#9998;</button>
              {#if deletingKey}
                <button class="confirm-btn" onclick={handleDeleteKey}>Sure?</button>
                <button class="icon-btn" onclick={() => deletingKey = false}>&#10005;</button>
              {:else}
                <button class="icon-btn danger" onclick={() => deletingKey = true} title="Delete Key">&#128465;</button>
              {/if}
            </div>
          </div>
        </div>
        
        {#key refreshKeyCount}
          <ValueViewer type={selectedKeyType} connectionId={active.id} key={selectedKey} />
        {/key}

        {#if renamingKey}
          <PromptModal
            title="Rename Key"
            initialValue={selectedKey}
            onSave={handleRename}
            onCancel={() => renamingKey = false}
          />
        {/if}

        {#if editingTtl}
          <PromptModal
            title="Edit TTL (seconds, -1 for infinite)"
            initialValue={String(selectedKeyTtl)}
            onSave={handleSetTtl}
            onCancel={() => editingTtl = false}
          />
        {/if}
      {/if}
    {:else}
      <Dashboard connectionId={active.id} name={active.name} />
    {/if}
  {:else}
    <div class="main-empty">
      <div class="empty-content">
        <div class="empty-logo">
          <img src="/logo.svg" alt="Redix Logo" />
        </div>
        <h2 class="empty-title">Redix</h2>
        <p class="empty-desc">Connect to a Redis server to get started</p>
        <button class="btn btn-primary btn-lg" onclick={() => showConnections = true}>Connect Now</button>
      </div>
    </div>
  {/if}
{/snippet}

{#snippet bottom()}
  <div class="bottom-panel">
    <div class="bottom-header">
      <span style="font-weight: 600; font-size: 0.75rem;">Command Log</span>
      <div style="display:flex; gap: 0.5rem; align-items:center;">
        <button class="clear-btn" onclick={() => commandLogs = []}>Clear</button>
        <button class="clear-btn" style="padding: 0 0.25rem; font-size: 0.8rem;" onclick={() => showCommandLog = false} title="Close Log">&#10005;</button>
      </div>
    </div>
    <div class="bottom-content">
      {#each commandLogs.slice().reverse() as log}
        <div class="log-entry">
          <span class="log-time">{new Date(log.timestamp).toLocaleTimeString([], {hour12:false, hour:'2-digit', minute:'2-digit', second:'2-digit'})}</span>
          <span class="log-cmd">{log.command}</span>
          <span class="log-dur">{log.duration}ms</span>
        </div>
      {/each}
    </div>
  </div>
{/snippet}

<ToastContainer />
<Layout {sidebar} {main} {bottom} showBottom={active !== null && showCommandLog} />

{#if active}
  <button 
    class="log-toggle-btn"
    class:active={showCommandLog}
    onclick={() => showCommandLog = !showCommandLog}
    title="Toggle Command Log"
  >
    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="4 17 10 11 4 5"></polyline>
      <line x1="12" y1="19" x2="20" y2="19"></line>
    </svg>
  </button>
{/if}

{#if showConnections}
  <ConnectionList onclose={() => showConnections = false} />
{/if}

{#if addingKey && active}
  <AddKeyModal
    connectionId={active.id}
    onSave={handleKeyAdded}
    onCancel={() => addingKey = false}
  />
{/if}

{#if lookingUpKey && active}
  <PromptModal
    title="Lookup Key (Exact Match)"
    placeholder="Enter exact key name..."
    saveText="Find"
    savingText="Finding..."
    onSave={async (key) => { lookingUpKey = false; await handleKeySelect(key.trim()); }}
    onCancel={() => lookingUpKey = false}
  />
{/if}

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
    gap: 0.5rem;
  }

  .conn-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .conn-selector {
    background: var(--color-surface-input);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.625rem;
    color: var(--color-fg);
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex: 1;
    min-width: 0;
    transition: all 0.2s;
  }

  .conn-selector:hover {
    border-color: var(--color-border-hover);
    background: var(--color-surface-btn-hover);
  }

  .conn-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .db-selector {
    width: auto !important;
    padding-top: 0.4rem !important;
    padding-bottom: 0.4rem !important;
    font-size: 0.75rem !important;
    font-weight: 600;
  }

  .sidebar-actions {
    display: flex;
    gap: 0.5rem;
    padding: 0 0.25rem;
  }

  .action-btn {
    flex: 1;
    background: transparent;
    border: 1px dashed var(--color-border, #333);
    border-radius: 6px;
    padding: 0.375rem;
    color: var(--color-muted);
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
  }

  .action-btn:hover {
    color: var(--color-fg);
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }

  .main-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-muted);
  }

  .empty-content {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    animation: fadeIn 0.5s ease-out;
  }
  
  .empty-logo {
    width: 72px;
    height: 72px;
    color: var(--color-accent);
    margin-bottom: 1rem;
    filter: drop-shadow(0 0 24px color-mix(in srgb, var(--color-accent) 50%, transparent));
  }
  
  .empty-logo img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .empty-title {
    margin: 0 0 0.5rem 0;
    font-weight: 700;
    font-size: 2.5rem;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, #fff 0%, #a0a0a0 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  :global(.light) .empty-title {
    background: linear-gradient(135deg, #111 0%, #555 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  
  .empty-desc {
    margin: 0 0 2rem 0;
    color: var(--color-muted);
    font-size: 0.875rem;
  }

  .bottom-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
  }

  .bottom-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0.75rem;
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
    color: var(--color-fg);
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 0.75rem;
  }
  
  .clear-btn:hover {
    color: var(--color-error, #f87171);
  }

  .bottom-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-family: var(--font-mono, monospace);
    font-size: 0.75rem;
  }

  .log-entry {
    display: flex;
    gap: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    align-items: flex-start;
  }
  
  .log-entry:hover {
    background: var(--color-bg);
  }

  .log-time {
    color: var(--color-muted);
    min-width: 60px;
  }

  .log-cmd {
    color: var(--color-accent);
    flex: 1;
    word-break: break-all;
  }

  .log-dur {
    color: var(--color-muted);
    text-align: right;
    min-width: 40px;
  }

  .btn-lg {
    padding: 0.75rem 2rem !important;
    border-radius: 9999px !important;
    font-size: 1rem !important;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .tabs-bar {
    display: flex;
    overflow-x: auto;
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
  }

  .tabs-bar::-webkit-scrollbar {
    height: 4px;
  }
  
  .tab {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.75rem;
    border-right: 1px solid var(--color-border);
    cursor: pointer;
    font-size: 0.75rem;
    color: var(--color-muted);
    background: color-mix(in srgb, var(--color-bg) 80%, black);
    min-width: 0;
    max-width: 150px;
  }
  
  .tab.active {
    background: var(--color-surface);
    color: var(--color-fg);
    border-bottom: 1px solid var(--color-accent);
  }
  
  .tab-title {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  }
  
  .tab-close {
    background: none;
    border: none;
    color: inherit;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.25rem;
    cursor: pointer;
    opacity: 0.5;
  }
  
  .tab-close:hover {
    opacity: 1;
  }

  .context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--color-surface, #1e1e1e);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.5);
    display: flex;
    flex-direction: column;
    padding: 0.25rem;
    min-width: 120px;
  }

  .ctx-item {
    background: none;
    border: none;
    color: var(--color-fg);
    padding: 0.375rem 0.75rem;
    text-align: left;
    font-size: 0.75rem;
    cursor: pointer;
    border-radius: 4px;
  }

  .ctx-item:hover {
    background: var(--color-bg);
    color: var(--color-accent);
  }

  .key-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-bg) 60%, transparent);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .key-breadcrumb-wrap {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .key-breadcrumb {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    font-size: 0.8125rem;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    color: var(--color-fg);
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
    padding: 0.25rem 0.75rem;
    border-radius: 6px;
    border: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
  }

  .breadcrumb-part {
    font-weight: 600;
  }

  .breadcrumb-sep {
    color: var(--color-muted);
    margin: 0 0.125rem;
  }

  .key-meta {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .key-type-badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.75rem;
    border-radius: 9999px;
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
    color: var(--color-accent);
    font-weight: 600;
    flex-shrink: 0;
    box-shadow: 0 0 10px color-mix(in srgb, var(--color-accent) 10%, transparent);
  }

  .key-ttl-badge {
    font-size: 0.75rem;
    padding: 0.125rem 0.75rem;
    border-radius: 9999px;
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
    color: var(--color-muted);
    white-space: nowrap;
    transition: all 0.2s;
  }

  .key-ttl-badge.clickable:hover {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
    cursor: pointer;
  }

  .key-ttl-badge.clickable {
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .key-ttl-badge.clickable:hover {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
  }

  .log-toggle-btn {
    position: fixed;
    bottom: 0.75rem;
    left: 0.75rem;
    z-index: 50;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-muted);
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .log-toggle-btn:hover {
    color: var(--color-fg);
    border-color: var(--color-accent);
  }

  .log-toggle-btn.active {
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
    border-color: var(--color-accent);
  }

  .key-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: 0.5rem;
    padding-left: 0.5rem;
    border-left: 1px solid var(--color-border);
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    font-size: 1rem;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .icon-btn:hover {
    color: var(--color-fg);
    background: var(--color-surface);
  }

  .icon-btn.danger:hover {
    color: #ef4444;
  }

  .confirm-btn {
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 4px;
    padding: 0.125rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
  }

  .confirm-btn:hover {
    background: #dc2626;
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

  .btn-primary {
    background: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
    color: #fff;
  }
</style>
