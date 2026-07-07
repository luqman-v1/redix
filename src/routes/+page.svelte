<script lang="ts">
  import Layout from "$lib/components/Layout.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
</script>

{#snippet sidebar()}
  <div
    style:padding="1rem"
    style:display="flex"
    style:flex-direction="column"
    style:gap="0.75rem"
    style:height="100%"
  >
    <div
      style:display="flex"
      style:align-items="center"
      style:justify-content="space-between"
    >
      <h2 style:font-size="1rem" style:font-weight="600" style:color="var(--color-fg)" style:margin="0">
        Connections
      </h2>
      <ThemeToggle />
    </div>
    <p style:color="var(--color-muted)" style:margin="0" style:font-size="0.875rem">
      No connections yet
    </p>
  </div>
{/snippet}

{#snippet main()}
  <div
    style:display="flex"
    style:align-items="center"
    style:justify-content="center"
    style:height="100%"
    style:color="var(--color-muted)"
  >
    Select a connection to browse keys
  </div>
{/snippet}

{#snippet bottom()}
  <div
    style:padding="0.75rem 1rem"
    style:height="100%"
    style:display="flex"
    style:align-items="center"
    style:color="var(--color-muted)"
    style:font-size="0.875rem"
  >
    Command console
  </div>
{/snippet}

<Layout {sidebar} {main} {bottom} />
