<script lang="ts">
  import type { Snippet } from "svelte";
  import Panel from "./Panel.svelte";

  let {
    sidebar,
    main,
    bottom,
  }: {
    sidebar: Snippet;
    main: Snippet;
    bottom: Snippet;
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
    <Panel initialSize={260} minSize={180} maxSize={400} direction="horizontal">
      {@render sidebar()}
    </Panel>
    <div style:flex="1" style:overflow="auto" style:min-width="0">
      {@render main()}
    </div>
  </div>
  <Panel initialSize={200} minSize={100} maxSize={500} direction="vertical">
    {@render bottom()}
  </Panel>
</div>
