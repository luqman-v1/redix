<script lang="ts">
  import { theme } from "$lib/stores/theme";
</script>

<button
  aria-label="Toggle theme"
  title="Toggle Theme (Ctrl+T)"
  onclick={() => theme.toggle()}
  style:background-color="var(--color-surface)"
  style:border="1px solid var(--color-border)"
  style:border-radius="8px"
  style:padding="0.5rem 0.75rem"
  style:cursor="pointer"
  style:font-size="1.25rem"
  style:line-height="1"
  style:color="var(--color-fg)"
>
  {$theme === "dark" ? "🌙" : "☀️"}
</button>
