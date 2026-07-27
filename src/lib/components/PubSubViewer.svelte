<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { ConnectionConfig } from "$lib/types/connection";

  interface Props {
    config: ConnectionConfig;
  }

  let { config }: Props = $props();

  let channel = $state("");
  let isSubscribed = $state(false);
  let messages = $state<{ time: Date; channel: string; payload: string }[]>([]);
  let reversedMessages = $derived(messages.slice().reverse());
  let unlisten: UnlistenFn | null = null;
  let error = $state("");
  let subscriptionId = $state<string | null>(null);

  async function handleSubscribe() {
    if (!channel.trim()) return;
    error = "";
    
    try {
      if (isSubscribed && subscriptionId) {
        await invoke("unsubscribe_channel", { subId: subscriptionId });
        isSubscribed = false;
        subscriptionId = null;
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
      } else {
        // Clear messages on new subscription?
        // messages = [];
        
        subscriptionId = await invoke<string>("subscribe_channel", {
          config,
          channel: channel.trim(),
        });
        
        isSubscribed = true;
        
        unlisten = await listen<{ channel: string; payload: string }>(`pubsub-${subscriptionId}`, (event) => {
          messages.push({
            time: new Date(),
            channel: event.payload.channel,
            payload: event.payload.payload,
          });
          if (messages.length > 10000) {
            messages.shift();
          }
        });
      }
    } catch (e) {
      error = String(e);
      isSubscribed = false;
    }
  }

  onDestroy(async () => {
    if (unlisten) unlisten();
    if (subscriptionId) {
      try {
        await invoke("unsubscribe_channel", { subId: subscriptionId });
      } catch (e) {}
    }
  });
</script>

<div class="pubsub-wrapper">
  <div class="pubsub-header">
    <div class="input-group">
      <input 
        type="text" 
        bind:value={channel} 
        placeholder="Channel name (e.g. events, user:*)" 
        disabled={isSubscribed}
        autocapitalize="off"
        autocomplete="off"
        autocorrect="off"
        spellcheck="false"
      />
      <button class="btn btn-primary" onclick={handleSubscribe} disabled={!channel.trim()}>
        {isSubscribed ? "Unsubscribe" : "Subscribe"}
      </button>
      <button class="btn" onclick={() => messages = []} disabled={messages.length === 0}>
        Clear
      </button>
    </div>
    {#if error}
      <div class="error">{error}</div>
    {/if}
  </div>

  <div class="pubsub-body">
    {#if messages.length === 0}
      <div class="empty">No messages received yet.</div>
    {:else}
      <div class="messages">
        {#each reversedMessages as msg}
          <div class="msg-row">
            <span class="time">{msg.time.toLocaleTimeString()}</span>
            <span class="channel">{msg.channel}</span>
            <span class="payload">{msg.payload}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .pubsub-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1rem;
    gap: 1rem;
    background: var(--color-bg);
  }
  .pubsub-header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .input-group {
    display: flex;
    gap: 0.5rem;
  }
  .input-group input {
    flex: 1;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-fg);
  }
  .error {
    color: var(--color-error);
    font-size: 0.85rem;
  }
  .pubsub-body {
    flex: 1;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .empty {
    padding: 2rem;
    text-align: center;
    color: var(--color-muted);
    font-style: italic;
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }
  .msg-row {
    display: flex;
    gap: 1rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--color-border);
    font-family: monospace;
    font-size: 0.85rem;
  }
  .msg-row:last-child {
    border-bottom: none;
  }
  .time {
    color: var(--color-muted);
    white-space: nowrap;
  }
  .channel {
    color: var(--color-accent);
    font-weight: 600;
    white-space: nowrap;
  }
  .payload {
    color: var(--color-fg);
    word-break: break-all;
  }
</style>
