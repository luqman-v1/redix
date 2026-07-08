<script lang="ts">
  import type { ConnectionConfig, ConnectionType } from "$lib/types/connection";
  import { connections } from "$lib/stores/connections";
  import { open } from "@tauri-apps/plugin-dialog";

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
  let folder = $state(e?.folder ?? "");
  let host = $state(e?.host ?? "127.0.0.1");
  let port = $state(e?.port ?? 6379);
  let username = $state(e?.username ?? "");
  let password = $state(e?.password ?? "");
  let connectionType = $state<ConnectionType>(e?.type ?? "standalone");
  let keySeparator = $state(e?.key_separator ?? ":");
  let db = $state(e?.db ?? 0);
  let useSsl = $state(e?.use_ssl ?? false);
  let useSsh = $state(!!e?.ssh);
  let sshHost = $state(e?.ssh?.host ?? "");
  let sshPort = $state(e?.ssh?.port ?? 22);
  let sshUser = $state(e?.ssh?.username ?? "");
  
  // Extract auth info
  let sshPassword = $state(e?.ssh?.auth && 'password' in e.ssh.auth ? e.ssh.auth.password : "");
  let sshPrivateKey = $state(e?.ssh?.auth && 'keyfile' in e.ssh.auth ? e.ssh.auth.keyfile.path : "");
  let sshPassphrase = $state(e?.ssh?.auth && 'keyfile' in e.ssh.auth ? (e.ssh.auth.keyfile.passphrase ?? "") : "");
  
  let sslCaCert = $state(e?.ssl?.ca_cert ?? "");
  let sslClientCert = $state(e?.ssl?.client_cert ?? "");
  let sslClientKey = $state(e?.ssl?.client_key ?? "");
  
  let timeout = $state(e?.timeout ?? 30);
  let readonly = $state(e?.readonly ?? false);
  let error = $state("");
  let showPassword = $state(false);
  let showSshPassword = $state(false);

  async function browseFile(type: 'ca' | 'clientCert' | 'clientKey' | 'sshKey') {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
      });
      if (selected && typeof selected === 'string') {
        if (type === 'ca') sslCaCert = selected;
        else if (type === 'clientCert') sslClientCert = selected;
        else if (type === 'clientKey') sslClientKey = selected;
        else if (type === 'sshKey') sshPrivateKey = selected;
      }
    } catch (err) {
      console.error("Failed to open dialog", err);
    }
  }

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
        await connections.save({
          ...editing,
          name: name.trim(),
          folder: folder.trim() || undefined,
          host: host.trim(),
          port,
          username: username.trim() || undefined,
          password: password || undefined,
          type: connectionType,
          db,
          key_separator: keySeparator || ":",
          use_ssl: useSsl,
          ssl: useSsl ? {
            ca_cert: sslCaCert.trim() || undefined,
            client_cert: sslClientCert.trim() || undefined,
            client_key: sslClientKey.trim() || undefined,
            skip_verify: false,
          } : undefined,
          ssh: useSsh && sshHost ? {
            host: sshHost.trim(),
            port: sshPort,
            username: sshUser.trim() || "root",
            auth: sshPrivateKey.trim() 
              ? { keyfile: { path: sshPrivateKey.trim(), passphrase: sshPassphrase || undefined } }
              : { password: sshPassword || "" },
          } : undefined,
          readonly,
          timeout,
        });
      } else {
        await connections.add({
          name: name.trim(),
          folder: folder.trim() || undefined,
          host: host.trim(),
          port,
          username: username.trim() || undefined,
          password: password || undefined,
          type: connectionType,
          db,
          key_separator: keySeparator || ":",
          use_ssl: useSsl,
          ssl: useSsl ? {
            ca_cert: sslCaCert.trim() || undefined,
            client_cert: sslClientCert.trim() || undefined,
            client_key: sslClientKey.trim() || undefined,
            skip_verify: false,
          } : undefined,
          ssh: useSsh && sshHost ? {
            host: sshHost.trim(),
            port: sshPort,
            username: sshUser.trim() || "root",
            auth: sshPrivateKey.trim() 
              ? { keyfile: { path: sshPrivateKey.trim(), passphrase: sshPassphrase || undefined } }
              : { password: sshPassword || "" },
          } : undefined,
          readonly,
          timeout,
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
  <form class="modal modal-form" onsubmit={handleSubmit}>
    <h3 style:margin="0 0 1rem 0" style:font-size="1.125rem" style:font-weight="600">
      {editing ? "Edit Connection" : "New Connection"}
    </h3>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="row">
      <label class="field" style:flex="2">
        <span class="label">Name</span>
        <input type="text" bind:value={name} placeholder="My Redis" />
      </label>
      <label class="field" style:flex="1">
        <span class="label">Folder (Group)</span>
        <input type="text" bind:value={folder} placeholder="e.g. Production" />
      </label>
    </div>

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

    <div class="row">
      <label class="field" style:flex="1">
        <span class="label">Username</span>
        <input type="text" bind:value={username} placeholder="(optional ACL)" autocomplete="off" />
      </label>
      <label class="field" style:flex="1">
        <span class="label">Password</span>
        <div style="display:flex; gap:0.25rem;">
          <input type={showPassword ? "text" : "password"} bind:value={password} placeholder="(optional)" autocomplete="new-password" style="flex:1; width:100%;" />
          <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => showPassword = !showPassword}>
            {showPassword ? "Hide" : "Show"}
          </button>
        </div>
      </label>
    </div>

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
      <label class="field" style:flex="1">
        <span class="label">Separator</span>
        <input type="text" bind:value={keySeparator} placeholder=":" />
      </label>
      <label class="field" style:flex="1">
        <span class="label">Timeout (s)</span>
        <input type="number" bind:value={timeout} min="1" max="300" />
      </label>
    </div>


    <div class="row" style="margin-top: 0.5rem; margin-bottom: 0.5rem;">
      <label style="display:flex; align-items:center; gap:0.5rem; cursor:pointer; font-size:0.75rem;">
        <input type="checkbox" bind:checked={useSsl} style="margin:0; padding:0;" /> Use SSL/TLS
      </label>
      <label style="display:flex; align-items:center; gap:0.5rem; cursor:pointer; font-size:0.75rem;">
        <input type="checkbox" bind:checked={useSsh} style="margin:0; padding:0;" /> Use SSH Tunnel
      </label>
      <label style="display:flex; align-items:center; gap:0.5rem; cursor:pointer; font-size:0.75rem;">
        <input type="checkbox" bind:checked={readonly} style="margin:0; padding:0;" /> Read-Only
      </label>
    </div>

    {#if useSsl}
      <div class="ssl-group" style="padding: 1rem; background: rgba(0, 0, 0, 0.2); border-radius: 8px; border: 1px solid var(--color-border); margin-bottom: 0.75rem;">
        <label class="field" style="margin-bottom:0.5rem;">
          <span class="label">CA Certificate (Authority)</span>
          <div style="display:flex; gap:0.25rem;">
            <input type="text" bind:value={sslCaCert} placeholder="/path/to/ca.crt" style="flex:1; width:100%;" />
            <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => browseFile('ca')}>Browse</button>
          </div>
        </label>
        <div class="row">
          <label class="field" style="flex:1;">
            <span class="label">Public Key (Client Cert)</span>
            <div style="display:flex; gap:0.25rem;">
              <input type="text" bind:value={sslClientCert} placeholder="/path/to/client.crt" style="flex:1; width:100%;" />
              <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => browseFile('clientCert')}>Browse</button>
            </div>
          </label>
          <label class="field" style="flex:1;">
            <span class="label">Private Key</span>
            <div style="display:flex; gap:0.25rem;">
              <input type="text" bind:value={sslClientKey} placeholder="/path/to/client.key" style="flex:1; width:100%;" />
              <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => browseFile('clientKey')}>Browse</button>
            </div>
          </label>
        </div>
      </div>
    {/if}

    {#if useSsh}
      <div class="ssh-group" style="padding: 1rem; background: rgba(0, 0, 0, 0.2); border-radius: 8px; border: 1px solid var(--color-border); margin-bottom: 0.75rem;">
        <div class="row" style="margin-bottom:0.5rem;">
          <label class="field" style="flex:3;">
            <span class="label">SSH Host</span>
            <input type="text" bind:value={sshHost} placeholder="bastion.example.com" />
          </label>
          <label class="field" style="flex:1;">
            <span class="label">SSH Port</span>
            <input type="number" bind:value={sshPort} min="1" max="65535" />
          </label>
        </div>
        <div class="row" style="margin-bottom:0.5rem;">
          <label class="field" style="flex:1;">
            <span class="label">SSH Username</span>
            <input type="text" bind:value={sshUser} placeholder="ubuntu" />
          </label>
          <label class="field" style="flex:1;">
            <span class="label">SSH Password</span>
            <div style="display:flex; gap:0.25rem;">
              <input type={showSshPassword ? "text" : "password"} bind:value={sshPassword} placeholder="(optional)" style="flex:1; width:100%;" />
              <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => showSshPassword = !showSshPassword}>
                {showSshPassword ? "Hide" : "Show"}
              </button>
            </div>
          </label>
        </div>
        <div class="row">
          <label class="field" style="flex:1;">
            <span class="label">SSH Private Key</span>
            <div style="display:flex; gap:0.25rem;">
              <input type="text" bind:value={sshPrivateKey} placeholder="/path/to/id_rsa" style="flex:1; width:100%;" />
              <button type="button" class="btn" style="padding:0.25rem 0.5rem; white-space:nowrap;" onclick={() => browseFile('sshKey')}>Browse</button>
            </div>
          </label>
          <label class="field" style="flex:1;">
            <span class="label">Passphrase</span>
            <input type={showSshPassword ? "text" : "password"} bind:value={sshPassphrase} placeholder="(optional)" />
          </label>
        </div>
      </div>
    {/if}

    <div class="actions">
      <button type="button" class="btn" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn btn-primary">
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
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
  }

  .modal-form {
    padding: 1.5rem;
    width: 42rem;
    max-width: 100%;
    max-height: calc(100vh - 4rem);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .label {
    font-size: 0.75rem;
    color: var(--color-muted);
    font-weight: 500;
  }

  .row {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .row > * {
    flex: 1 1 auto;
    min-width: 0;
  }

  .error {
    background: color-mix(in srgb, var(--color-error) 15%, transparent);
    color: var(--color-error);
    padding: 0.5rem;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
</style>
