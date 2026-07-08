<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { toasts } from "$lib/stores/toasts";

  let { connectionId, onSave, onCancel }: { connectionId: string, onSave: (key: string) => void, onCancel: () => void } = $props();

  let keyName = $state("");
  let keyType = $state("string");

  // Type-specific values
  let stringValue = $state("");
  
  let hashField = $state("");
  let hashValue = $state("");
  
  let listValue = $state("");
  
  let setMember = $state("");
  
  let zsetScore = $state(0);
  let zsetMember = $state("");

  let saving = $state(false);

  let stringFormat = $state("text");
  let stringError = $state("");

  function applyStringFormat(fmt: string) {
    stringError = "";
    try {
      if (fmt === "json-pretty") {
        stringValue = JSON.stringify(JSON.parse(stringValue), null, 2);
      } else if (fmt === "json-minify") {
        stringValue = JSON.stringify(JSON.parse(stringValue));
      }
    } catch (e) {
      stringError = "Format error: " + (e instanceof Error ? e.message : String(e));
    }
  }

  let prevStringFormat = "text";
  $effect(() => {
    if (stringFormat !== prevStringFormat) {
      applyStringFormat(stringFormat);
      prevStringFormat = stringFormat;
    }
  });

  async function handleSave() {
    if (!keyName.trim()) {
      toasts.add("Key name is required", "error");
      return;
    }

    saving = true;
    try {
      if (keyType === "string") {
        await invoke("set_string_value", {
          connectionId,
          key: keyName.trim(),
          value: stringValue,
        });
      } else if (keyType === "hash") {
        if (!hashField.trim()) throw new Error("Hash field is required");
        await invoke("set_hash_field", {
          connectionId,
          key: keyName.trim(),
          field: hashField.trim(),
          value: hashValue,
        });
      } else if (keyType === "list") {
        if (!listValue.trim()) throw new Error("List value is required");
        await invoke("list_push", {
          connectionId,
          key: keyName.trim(),
          value: listValue,
          side: "right",
        });
      } else if (keyType === "set") {
        if (!setMember.trim()) throw new Error("Set member is required");
        await invoke("add_set_member", {
          connectionId,
          key: keyName.trim(),
          member: setMember,
        });
      } else if (keyType === "zset") {
        if (!zsetMember.trim()) throw new Error("ZSet member is required");
        await invoke("add_sorted_set", {
          connectionId,
          key: keyName.trim(),
          score: Number(zsetScore),
          member: zsetMember,
        });
      }

      onSave(keyName.trim());
    } catch (e) {
      toasts.add(String(e), "error");
    } finally {
      saving = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={(e) => e.target === e.currentTarget && onCancel()}>
  <div class="modal">
    <div class="modal-header">
      <h3 class="modal-title">New Redis Key</h3>
      <button class="close-btn" onclick={onCancel}>&#10005;</button>
    </div>
    
    <div class="modal-body">
      <div class="form-group">
        <label for="add-key-type">Type</label>
        <select id="add-key-type" class="input" bind:value={keyType}>
          <option value="string">String</option>
          <option value="hash">Hash</option>
          <option value="list">List</option>
          <option value="set">Set</option>
          <option value="zset">Sorted Set (ZSet)</option>
        </select>
      </div>

      <hr class="divider" />

      <div class="form-group">
        <label for="add-key-name">Key Name</label>
        <input id="add-key-name" type="text" class="input" bind:value={keyName} placeholder="e.g. user:1001" />
      </div>


      {#if keyType === "string"}
        <div class="form-group">
          <div style="display:flex; justify-content:space-between; align-items:center;">
            <label for="add-string-format">Value</label>
            <select id="add-string-format" class="input" bind:value={stringFormat} style="padding:0.125rem 0.375rem; font-size:0.6875rem;">
              <option value="text">Plain Text</option>
              <option value="json-pretty">JSON Pretty</option>
              <option value="json-minify">JSON Minify</option>
            </select>
          </div>
          {#if stringError}
            <div style="color:var(--color-error); font-size:0.75rem;">{stringError}</div>
          {/if}
          <textarea id="add-string-value" class="input textarea" bind:value={stringValue} placeholder="String value..." rows="6"></textarea>
        </div>
      {:else if keyType === "hash"}
        <div class="form-group">
          <label for="add-hash-field">Field Name</label>
          <input id="add-hash-field" type="text" class="input" bind:value={hashField} placeholder="e.g. name" />
        </div>
        <div class="form-group">
          <label for="add-hash-value">Value</label>
          <textarea id="add-hash-value" class="input textarea" bind:value={hashValue} placeholder="Field value..." rows="3"></textarea>
        </div>
      {:else if keyType === "list"}
        <div class="form-group">
          <label for="add-list-value">Initial Value</label>
          <input id="add-list-value" type="text" class="input" bind:value={listValue} placeholder="Item value" />
        </div>
      {:else if keyType === "set"}
        <div class="form-group">
          <label for="add-set-member">Initial Member</label>
          <input id="add-set-member" type="text" class="input" bind:value={setMember} placeholder="Member value" />
        </div>
      {:else if keyType === "zset"}
        <div class="form-group">
          <label for="add-zset-score">Score</label>
          <input id="add-zset-score" type="number" class="input" bind:value={zsetScore} placeholder="0" />
        </div>
        <div class="form-group">
          <label for="add-zset-member">Member</label>
          <input id="add-zset-member" type="text" class="input" bind:value={zsetMember} placeholder="Member value" />
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button class="btn" onclick={onCancel} disabled={saving}>Cancel</button>
      <button class="btn btn-primary" onclick={handleSave} disabled={saving}>
        {saving ? "Creating..." : "Create Key"}
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }

  .modal {
    background: var(--color-surface, #1e1e1e);
    border: 1px solid var(--color-border, #333);
    border-radius: 12px;
    width: 28rem;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--color-border);
  }

  .modal-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-fg);
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-muted);
    font-size: 1rem;
    cursor: pointer;
    line-height: 1;
  }
  .close-btn:hover {
    color: var(--color-fg);
  }

  .modal-body {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .form-group label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-muted);
  }

  .input {
    background: var(--color-input-bg, #1a1a1a);
    border: 1px solid var(--color-border, #333);
    border-radius: 6px;
    padding: 0.5rem;
    color: var(--color-fg);
    font-size: 0.75rem;
    outline: none;
  }

  .input:focus {
    border-color: var(--color-accent, #6366f1);
  }

  .textarea {
    resize: vertical;
    font-family: monospace;
  }

  .divider {
    border: none;
    border-top: 1px solid var(--color-border);
    margin: 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
    border-bottom-left-radius: 12px;
    border-bottom-right-radius: 12px;
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
  .btn:hover:not(:disabled) {
    background: var(--color-surface, #2a2a2a);
  }

  .btn-primary {
    background: var(--color-accent, #5b8def);
    border-color: var(--color-accent, #5b8def);
    color: #fff;
  }
  .btn-primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 80%, black);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
