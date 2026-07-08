<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { history } from "$lib/stores/history";
  import { toasts } from "$lib/stores/toasts";

  let { connectionId, onclose }: { connectionId: string, onclose?: () => void } = $props();

  interface ConsoleEntry {
    command: string;
    result: string;
    duration: number;
    isError: boolean;
  }

  let entries = $state<ConsoleEntry[]>([]);
  const MAX_ENTRIES = 100;
  let input = $state("");
  let loading = $state(false);
  let historyIndex = $state(-1);
  let outputEl = $state<HTMLDivElement | null>(null);
  let inputEl = $state<HTMLTextAreaElement | null>(null);

  const COMMANDS = [
    "APPEND", "AUTH", "BGREWRITEAOF", "BGSAVE", "BITCOUNT", "BITOP", "BITPOS", 
    "BLPOP", "BRPOP", "BRPOPLPUSH", "CLIENT", "CLUSTER", "COMMAND", "CONFIG", 
    "DBSIZE", "DEBUG", "DECR", "DECRBY", "DEL", "DISCARD", "DUMP", "ECHO", "EVAL", 
    "EXEC", "EXISTS", "EXPIRE", "EXPIREAT", "FLUSHALL", "FLUSHDB", "GEOADD", 
    "GEODIST", "GEOHASH", "GEOPOS", "GEORADIUS", "GET", "GETBIT", "GETRANGE", 
    "GETSET", "HDEL", "HEXISTS", "HGET", "HGETALL", "HINCRBY", "HINCRBYFLOAT", 
    "HKEYS", "HLEN", "HMGET", "HMSET", "HSCAN", "HSET", "HSETNX", "HSTRLEN", 
    "HVALS", "INCR", "INCRBY", "INCRBYFLOAT", "INFO", "KEYS", "LASTSAVE", "LINDEX", 
    "LINSERT", "LLEN", "LPOP", "LPUSH", "LPUSHX", "LRANGE", "LREM", "LSET", 
    "LTRIM", "MGET", "MONITOR", "MOVE", "MSET", "MSETNX", "MULTI", "PERSIST", 
    "PEXPIRE", "PEXPIREAT", "PFADD", "PFCOUNT", "PFMERGE", "PING", "PSETEX", 
    "PSUBSCRIBE", "PTTL", "PUBLISH", "PUBSUB", "PUNSUBSCRIBE", "QUIT", "RANDOMKEY", 
    "RENAME", "RENAMENX", "RESTORE", "ROLE", "RPOP", "RPOPLPUSH", "RPUSH", 
    "RPUSHX", "SADD", "SAVE", "SCAN", "SCARD", "SCRIPT", "SDIFF", "SDIFFSTORE", 
    "SELECT", "SET", "SETBIT", "SETEX", "SETNX", "SETRANGE", "SHUTDOWN", "SINTER", 
    "SINTERSTORE", "SISMEMBER", "SLAVEOF", "SLOWLOG", "SMEMBERS", "SMOVE", "SORT", 
    "SPOP", "SRANDMEMBER", "SREM", "SSCAN", "STRLEN", "SUBSCRIBE", "SUNION", 
    "SUNIONSTORE", "SWAPDB", "SYNC", "TIME", "TTL", "TYPE", "UNSUBSCRIBE", "UNWATCH", 
    "WAIT", "WATCH", "ZADD", "ZCARD", "ZCOUNT", "ZINCRBY", "ZINTERSTORE", "ZLEXCOUNT", 
    "ZPOPMAX", "ZPOPMIN", "ZRANGE", "ZRANGEBYLEX", "ZRANGEBYSCORE", "ZRANK", "ZREM", 
    "ZREMRANGEBYLEX", "ZREMRANGEBYRANK", "ZREMRANGEBYSCORE", "ZREVRANGE", "ZREVRANGEBYSCORE", "ZREVRANK", "ZSCAN", "ZSCORE", "ZUNIONSTORE"
  ];
  let selectedSuggestionIndex = $state(0);
  let showSuggestions = $state(false);

  let suggestions = $derived((() => {
    if (!input) return [];
    // Only suggest if they are typing the first word
    if (input.includes(" ")) return [];
    const term = input.toUpperCase();
    return COMMANDS.filter(c => c.startsWith(term)).slice(0, 8);
  })());

  $effect(() => {
    if (suggestions.length > 0) {
      showSuggestions = true;
      if (selectedSuggestionIndex >= suggestions.length) {
        selectedSuggestionIndex = 0;
      }
    } else {
      showSuggestions = false;
    }
  });

  // Teleport compatibility
  let teleportMode = $state(false);
  const TELEPORT_UNSUPPORTED = new Set([
    "ACL", "ASKING", "CLIENT", "CLUSTER", "CONFIG", "DEBUG", 
    "EXEC", "HELLO", "INFO", "LATENCY", "MEMORY", "MIGRATE", 
    "MODULE", "MONITOR", "MULTI", "PFDEBUG", "PFSELFTEST"
  ]);

  let historyItems = $state<string[]>([]);

  $effect(() => {
    const unsub = history.subscribe((v) => { historyItems = v; });
    return unsub;
  });

  $effect(() => {
    connectionId;
    history.load(connectionId);
    entries = [];
    historyIndex = -1;
    teleportMode = false;
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

    const cmdName = trimmed.split(" ")[0].toUpperCase();

    if (cmdName === "CLEAR") {
      entries = [];
      loading = false;
      return;
    }

    if (teleportMode && TELEPORT_UNSUPPORTED.has(cmdName)) {
      entries = [...entries, {
        command: trimmed,
        result: `ERR Teleport: command '${cmdName}' not supported (Blocked by UI)`,
        duration: 0,
        isError: true,
      }].slice(-MAX_ENTRIES);
      loading = false;
      scrollToBottom();
      return;
    }

    try {
      const response = await invoke<{
        result: any;
        duration_ms: number;
      }>("execute_command", { connectionId, command: trimmed });

      const isError = response.result?.type === "Error";
      
      const formatRedisValue = (val: any, indent = 0): string => {
        if (!val) return "(nil)";
        if (val.type === "Nil") return "(nil)";
        if (val.type === "String") return val.value; // Removed quotes for cleaner copy
        if (val.type === "Integer" || val.type === "Float") return String(val.value);
        if (val.type === "Status") return String(val.value);
        if (val.type === "Error") return `(error) ${val.value}`;
        if (val.type === "Bool") return String(val.value);
        if (val.type === "Array") {
          const arr = val.value as any[];
          if (!arr || arr.length === 0) return "(empty array)";
          let out = "";
          for (let i = 0; i < arr.length; i++) {
            // For arrays, we add quotes back if it's a string so it's easier to distinguish elements,
            // but the user wanted no quotes. Let's just output the value without quotes.
            const formatted = formatRedisValue(arr[i], indent + 2);
            out += `${" ".repeat(indent)}${i + 1}) ${formatted}`;
            if (i < arr.length - 1) out += "\n";
          }
          return out;
        }
        return JSON.stringify(val);
      };

      const newEntry = {
        command: trimmed,
        result: formatRedisValue(response.result),
        duration: response.duration_ms,
        isError,
      };
      entries = [...entries, newEntry].slice(-MAX_ENTRIES);

      if (isError && typeof response.result?.value === "string" && response.result.value.includes("Teleport")) {
        teleportMode = true;
      }

      await history.add(connectionId, trimmed);
    } catch (e) {
      const errMsg = String(e);
      if (errMsg.includes("Teleport")) {
        teleportMode = true;
      }
      
      const errEntry = {
        command: trimmed,
        result: errMsg,
        duration: 0,
        isError: true,
      };
      entries = [...entries, errEntry].slice(-MAX_ENTRIES);
    } finally {
      loading = false;
      scrollToBottom();
    }
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (showSuggestions && suggestions.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex + 1) % suggestions.length;
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex - 1 + suggestions.length) % suggestions.length;
        return;
      }
      if (e.key === "Tab" || e.key === "Enter") {
        e.preventDefault();
        input = suggestions[selectedSuggestionIndex] + " ";
        showSuggestions = false;
        // Focus stays on input, ready to type arguments
        return;
      }
      if (e.key === "Escape") {
        showSuggestions = false;
        return;
      }
    }

    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      await execute(input);
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

  function handleBlur() {
    // Hide suggestions slightly delayed so clicks register
    setTimeout(() => {
      showSuggestions = false;
    }, 150);
    historyIndex = -1;
  }

  async function copyToClipboard(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      toasts.add("Copied to clipboard", "success");
    } catch (err) {
      toasts.add("Failed to copy", "error");
    }
  }
</script>

<div class="console">
  <div class="output" bind:this={outputEl}>
    <div class="console-actions">
      {#if entries.length > 0}
        <button class="clear-btn" onclick={() => entries = []} title="Clear Console">
          &#x1F5D1; Clear
        </button>
      {/if}
      {#if onclose}
        <button class="close-console-btn" onclick={onclose} title="Close Console">
          &#10005;
        </button>
      {/if}
    </div>
    {#each entries as entry}
      <div class="entry">
        <div class="entry-command">
          <span class="prompt">&gt;</span>
          <span class="cmd-text">{entry.command}</span>
          <span class="duration">{entry.duration}ms</span>
        </div>
        <div class="entry-result" class:error={entry.isError}>
          <button class="copy-entry-btn" onclick={() => copyToClipboard(entry.result)} title="Copy Output">
            <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
          </button>
          {#if typeof entry.result === 'string' && entry.result.includes('\n')}
            <pre>{entry.result}</pre>
          {:else}
            {entry.result}
          {/if}
        </div>
      </div>
    {/each}
    {#if loading}
      <div class="loading">Executing...</div>
    {/if}
  </div>

  <div class="input-area">
    <div style="display:flex; flex-direction:column; flex:1;">
      {#if teleportMode}
        <div class="teleport-badge" title="Teleport restricted mode active. Some commands are disabled.">
          Teleport Mode Active
        </div>
      {/if}
      <div style="display:flex; align-items:flex-start; gap:0.5rem; position:relative;">
        {#if showSuggestions && suggestions.length > 0}
          <div class="suggestions">
            {#each suggestions as sug, i}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div 
                class="suggestion-item" 
                class:active={i === selectedSuggestionIndex}
                onmousedown={(e) => { e.preventDefault(); input = sug + " "; showSuggestions = false; inputEl?.focus(); }}
              >
                {sug}
              </div>
            {/each}
          </div>
        {/if}
        <span class="prompt-icon">&gt;</span>
        <textarea
          data-console-input
          bind:this={inputEl}
          bind:value={input}
          onkeydown={handleKeydown}
          onblur={handleBlur}
          onfocus={() => { if(suggestions.length > 0) showSuggestions = true; }}
          placeholder="Enter Redis command..."
          rows="1"
          disabled={loading}
        ></textarea>
      </div>
    </div>
  </div>
</div>

<style>
  .console {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas,
      "Liberation Mono", monospace;
    font-size: 0.75rem;
    background: var(--color-bg);
    color: var(--color-fg);
  }

  .output {
    flex: 1;
    overflow-y: auto;
    padding: 0.25rem 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    position: relative;
  }


  .console-actions {
    position: absolute;
    top: 0.5rem;
    right: 1rem;
    display: flex;
    gap: 0.375rem;
    z-index: 10;
  }

  .clear-btn, .close-console-btn {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-muted);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.6875rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-console-btn {
    padding: 0.25rem 0.375rem;
  }

  .clear-btn:hover, .close-console-btn:hover {
    background: var(--color-border);
    color: var(--color-fg);
  }

  .entry {
    background: var(--color-surface);
    border-radius: 6px;
    padding: 0.5rem;
    font-size: 0.8125rem;
  }

  .entry-command {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--color-muted);
    margin-bottom: 0.25rem;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  }

  .prompt {
    color: var(--color-accent);
    font-weight: 700;
  }

  .cmd-text {
    flex: 1;
    color: var(--color-fg);
  }

  .duration {
    font-size: 0.6875rem;
    opacity: 0.5;
  }

  .entry-result {
    color: var(--color-fg);
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    white-space: pre-wrap;
    word-break: break-all;
    position: relative;
    padding-right: 2rem;
  }
  
  .copy-entry-btn {
    position: absolute;
    top: 0;
    right: 0;
    background: transparent;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: all 0.2s;
  }

  .entry-result:hover .copy-entry-btn {
    opacity: 1;
    background: var(--color-surface-hover, rgba(255,255,255,0.1));
  }

  .copy-entry-btn:hover {
    color: var(--color-fg);
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
    padding: 0.25rem 0.5rem;
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

  .teleport-badge {
    font-size: 0.6875rem;
    color: #e8a427;
    background: color-mix(in srgb, #e8a427 10%, transparent);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    align-self: flex-start;
    margin-bottom: 0.25rem;
    border: 1px solid color-mix(in srgb, #e8a427 30%, transparent);
  }

  .suggestions {
    position: absolute;
    bottom: 100%;
    left: 1.25rem;
    margin-bottom: 0.25rem;
    background: var(--color-surface, #1e1e1e);
    border: 1px solid var(--color-border, #333);
    border-radius: 6px;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    padding: 0.25rem;
    min-width: 150px;
    z-index: 50;
  }

  .suggestion-item {
    padding: 0.25rem 0.5rem;
    cursor: pointer;
    border-radius: 4px;
    color: var(--color-fg);
  }

  .suggestion-item:hover, .suggestion-item.active {
    background: var(--color-accent, #5b8def);
    color: #fff;
  }
</style>
