<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    initialSize = 200,
    minSize = 100,
    maxSize = 500,
    direction = "horizontal",
    children,
  }: {
    initialSize?: number;
    minSize?: number;
    maxSize?: number;
    direction?: "horizontal" | "vertical";
    children: Snippet;
  } = $props();

  // svelte-ignore state_referenced_locally
  let size = $state(initialSize);
  let dragging = $state(false);
  let startPos = $state(0);
  let startSize = $state(0);
  let panelEl: HTMLElement | null = $state(null);

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    startPos = direction === "horizontal" ? e.clientX : e.clientY;
    startSize = size;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    const delta =
      direction === "horizontal" ? e.clientX - startPos : e.clientY - startPos;
    size = Math.max(minSize, Math.min(maxSize, startSize + delta));
  }

  function onPointerUp() {
    dragging = false;
  }

  function onKeydown(e: KeyboardEvent) {
    const step = 10;
    if (direction === "horizontal") {
      if (e.key === "ArrowLeft") size = Math.max(minSize, size - step);
      if (e.key === "ArrowRight") size = Math.min(maxSize, size + step);
    } else {
      if (e.key === "ArrowUp") size = Math.max(minSize, size - step);
      if (e.key === "ArrowDown") size = Math.min(maxSize, size + step);
    }
  }
</script>

<div
  bind:this={panelEl}
  style:display="flex"
  style:flex-direction={direction === "horizontal" ? "row" : "column"}
  style:overflow="hidden"
>
  <div
    style:width={direction === "horizontal" ? `${size}px` : "100%"}
    style:height={direction === "vertical" ? `${size}px` : "100%"}
    style:flex-shrink="0"
    style:overflow="auto"
    style:background-color="var(--color-surface)"
    style:color="var(--color-fg)"
  >
    {@render children()}
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    style:flex-shrink="0"
    style:width={direction === "horizontal" ? "6px" : "100%"}
    style:height={direction === "vertical" ? "6px" : "100%"}
    style:cursor={direction === "horizontal" ? "col-resize" : "row-resize"}
    style:background-color="var(--color-border)"
    style:opacity="0.5"
    style:user-select={dragging ? "none" : "auto"}
    style:transition="opacity 0.15s"
    role="separator"
    tabindex="0"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onkeydown={onKeydown}
    onpointerenter={(e) => { (e.currentTarget as HTMLElement).style.opacity = '1'; }}
    onpointerleave={(e) => { if (!dragging) (e.currentTarget as HTMLElement).style.opacity = '0.5'; }}
  ></div>
</div>
