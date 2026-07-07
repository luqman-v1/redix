<script lang="ts">
  import { toasts } from "$lib/stores/toasts";
  import Toast from "$lib/components/Toast.svelte";
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <Toast {toast} />
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 100;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    pointer-events: none;
  }

  .toast-container :global(.toast) {
    pointer-events: auto;
  }
</style>
