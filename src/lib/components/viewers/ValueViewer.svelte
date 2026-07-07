<script lang="ts">
  import StringViewer from "./StringViewer.svelte";
  import HashViewer from "./HashViewer.svelte";
  import ListViewer from "./ListViewer.svelte";
  import SetViewer from "./SetViewer.svelte";
  import SortedSetViewer from "./SortedSetViewer.svelte";
  import StreamViewer from "./StreamViewer.svelte";
  import GeoViewer from "./GeoViewer.svelte";
  import HyperLogLogViewer from "./HyperLogLogViewer.svelte";
  import BitmapViewer from "./BitmapViewer.svelte";

  interface Props {
    type: string;
    connectionId: string;
    key: string;
  }

  let { type, connectionId, key }: Props = $props();
</script>

<div class="value-viewer">
  {#if type === "string"}
    <StringViewer {connectionId} {key} />
  {:else if type === "hash"}
    <HashViewer {connectionId} {key} />
  {:else if type === "list"}
    <ListViewer {connectionId} {key} />
  {:else if type === "set"}
    <SetViewer {connectionId} {key} />
  {:else if type === "zset"}
    <SortedSetViewer {connectionId} {key} />
  {:else if type === "stream"}
    <StreamViewer {connectionId} {key} />
  {:else if type === "geo"}
    <GeoViewer {connectionId} {key} />
  {:else if type === "hyperloglog"}
    <HyperLogLogViewer {connectionId} {key} />
  {:else if type === "bitmap"}
    <BitmapViewer {connectionId} {key} />
  {:else}
    <div class="unsupported">
      Unsupported type: <code>{type}</code>
    </div>
  {/if}
</div>

<style>
  .value-viewer {
    display: flex;
    flex-direction: column;
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }

  .unsupported {
    color: var(--color-muted, #888);
    font-size: 0.8125rem;
    padding: 1rem 0.75rem;
    text-align: center;
  }

  .unsupported code {
    color: var(--color-error, #e55);
    font-weight: 600;
  }
</style>
