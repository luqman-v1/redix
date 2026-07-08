<script lang="ts">
  import type { Toast } from "$lib/stores/toasts";
  import { toasts } from "$lib/stores/toasts";

  let { toast }: { toast: Toast } = $props();
</script>

<div class="toast toast--{toast.type}" role="alert">
  <span class="toast-message">{toast.message}</span>
  <button class="toast-close" onclick={() => toasts.remove(toast.id)} aria-label="Dismiss">
    &times;
  </button>
</div>

<style>
  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.375rem 0.625rem;
    background: var(--color-surface);
    color: var(--color-fg);
    border-radius: 6px;
    border-left: 4px solid;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    min-width: 280px;
    max-width: 400px;
    font-size: 0.75rem;
    animation: slide-in 0.2s ease-out;
  }

  .toast--success {
    border-left-color: var(--color-success);
  }

  .toast--error {
    border-left-color: var(--color-error);
  }

  .toast--warning {
    border-left-color: var(--color-warning);
  }

  .toast-message {
    flex: 1;
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1.25rem;
    line-height: 1;
    padding: 0;
  }

  .toast-close:hover {
    color: var(--color-fg);
  }

  @keyframes slide-in {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }
</style>
