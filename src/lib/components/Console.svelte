<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { history } from "$lib/stores/history";

  let { connectionId }: { connectionId: string } = $props();

  interface ConsoleEntry {
    command: string;
    result: string;
    duration: number;
    isError: boolean;
  }

  let entries = $state<ConsoleEntry[]>([]);
  let input = $state("");
  let loading = $state(false);
  let historyIndex = $state(-1);
  let historyItems = $state<string[]>([]);
  let outputEl = $state<HTMLDivElement | null>(null);
  let inputEl = $state<HTMLTextAreaElement | null>(null);

  // Track history store
  history.subscribe((v) => (historyItems = v));

  $effect(() => {
    connectionId;
    history.load(connectionId);
    entries = [];
    historyIndex = -1;
  });

  function scrollToBottom() {
    if (outputEl) {
      outputEl.scrollTop = outputEl.scrollHeight;
    }
  }

  async function execute(command: string) {
    const trimmed = command.trim();
    if (!trimmed || loading) return;

    loading = true;
    input = "";
    historyIndex = -1;

    try {
      const response = await invoke<{
        result: { type: string; value: string };
        duration_ms: number;
      }>("execute_command", { connectionId, command: trimmed });

      const isError = response.result.type === "error";
      entries = [
        ...entries,
        {
          command: trimmed,
          result: response.result.value,
          duration: response.duration_ms,
          isError,
        },
      ];

      await history.add(connectionId, trimmed);
    } catch (e) {
      entries = [
        ...entries,
        {
          command: trimmed,
          result: String(e),
          duration: 0,
          isError: true,
        },
      ];
    } finally {
      loading = false;
      scrollToBottom();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      execute(input);
      return;
    }

    if (e.key === "Enter" && e.shiftKey) {
      e.preventDefault();
      return;
    }

    if (e.key === "ArrowUp") {
      if (historyItems.length === 0) return;
      e.preventDefault();
      if (historyIndex === -1) {
        historyIndex = historyItems.length - 1;
      } else if (historyIndex > 0) {
        historyIndex -= 1;
      }
      input = historyItems[historyIndex];
      return;
    }

    if (e.key === "ArrowDown") {
      if (historyIndex === -1) return;
      e.preventDefault();
      if (historyIndex < historyItems.length - 1) {
        historyIndex += 1;
        input = historyItems[historyIndex];
      } else {
        historyIndex = -1;
        input = "";
      }
    }
  }
</script>

<div class="console">
  <div class="output" bind:this={outputEl}>
    {#each entries as entry}
      <div class="entry">
        <div class="entry-command">
          <span class="prompt">&gt;</span>
          <span class="cmd">{entry.command}</span>
        </div>
        <div class="entry-result" class:error={entry.isError}>
          {entry.result}
        </div>
        <span class="entry-duration">{entry.duration}ms</span>
      </div>
    {/each}
    {#if loading}
      <div class="loading">Executing...</div>
    {/if}
  </div>

  <div class="input-area">
    <span class="prompt-icon">&gt;</span>
    <textarea
      data-console-input
      bind:this={inputEl}
      bind:value={input}
      onkeydown={handleKeydown}
      placeholder="Enter Redis command..."
      rows="1"
      disabled={loading}
    ></textarea>
  </div>
</div>

<style>
  .console {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas,
      "Liberation Mono", monospace;
    font-size: 0.8125rem;
    background: var(--color-bg);
    color: var(--color-fg);
  }

  .output {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .entry {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .entry-command {
    display: flex;
    gap: 0.5rem;
    color: var(--color-accent);
  }

  .prompt,
  .prompt-icon {
    color: var(--color-accent);
    user-select: none;
    flex-shrink: 0;
  }

  .cmd {
    color: var(--color-accent);
    white-space: pre-wrap;
    word-break: break-all;
  }

  .entry-result {
    color: var(--color-fg);
    white-space: pre-wrap;
    word-break: break-all;
    padding-left: 1.25rem;
  }

  .entry-result.error {
    color: var(--color-error);
  }

  .entry-duration {
    color: var(--color-muted);
    font-size: 0.6875rem;
    padding-left: 1.25rem;
  }

  .loading {
    color: var(--color-muted);
    font-style: italic;
  }

  .input-area {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--color-border);
  }

  .input-area textarea {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--color-fg);
    font-family: inherit;
    font-size: inherit;
    resize: none;
    line-height: 1.4;
  }

  .input-area textarea::placeholder {
    color: var(--color-muted);
  }

  .input-area textarea:disabled {
    opacity: 0.5;
  }

  .prompt-icon {
    line-height: 1.4;
    margin-top: 0.0625rem;
  }
</style>
