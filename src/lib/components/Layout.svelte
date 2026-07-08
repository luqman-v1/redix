<script lang="ts">
  import type { Snippet } from "svelte";
  import Panel from "./Panel.svelte";

  let {
    sidebar,
    main,
    bottom,
    showBottom = false,
  }: {
    sidebar: Snippet;
    main: Snippet;
    bottom: Snippet;
    showBottom?: boolean;
  } = $props();
</script>

<div
  style:display="flex"
  style:flex-direction="column"
  style:height="100vh"
  style:width="100vw"
  style:background-color="var(--color-bg)"
  style:color="var(--color-fg)"
  style:overflow="hidden"
>
  <div style:display="flex" style:flex-direction="row" style:flex="1" style:min-height="0">
    <Panel initialSize={320} minSize={240} maxSize={500} direction="horizontal">
      {@render sidebar()}
    </Panel>
    <div style:flex="1" style:overflow="hidden" style:min-width="0" style:display="flex" style:flex-direction="column">
      {@render main()}
    </div>
  </div>
  {#if showBottom}
    <Panel initialSize={200} minSize={100} maxSize={500} direction="vertical">
      {@render bottom()}
    </Panel>
  {/if}
</div>
