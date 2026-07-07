<script lang="ts">
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
</script>

<main
  style:min-height="100vh"
  style:background-color="var(--color-bg)"
  style:color="var(--color-fg)"
  style:display="flex"
  style:flex-direction="column"
  style:align-items="center"
  style:padding-top="10vh"
>
  <header
    style:display="flex"
    style:align-items="center"
    style:gap="1rem"
    style:margin-bottom="0.5rem"
  >
    <h1
      style:font-size="2rem"
      style:font-weight="700"
      style:color="var(--color-accent)"
      style:margin="0"
    >
      Redix
    </h1>
    <ThemeToggle />
  </header>

  <p style:color="var(--color-muted)" style:margin="0">
    Redis GUI Client
  </p>
</main>
