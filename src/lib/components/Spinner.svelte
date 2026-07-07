<script lang="ts">
  let { size = 16 }: { size?: number } = $props();
</script>

<svg class="animate-spin" width={size} height={size} viewBox="0 0 24 24" fill="none">
  <circle cx="12" cy="12" r="10" stroke="var(--color-muted)" stroke-width="3" opacity="0.3" />
  <path d="M12 2a10 10 0 0 1 10 10" stroke="var(--color-accent)" stroke-width="3" stroke-linecap="round" />
</svg>
