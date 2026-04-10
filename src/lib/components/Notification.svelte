<script lang="ts">
  import { fly } from "svelte/transition";
  import { notification, clearNotification } from "../stores/notifications";
</script>

{#if $notification}
  <div
    class="snackbar"
    role="status"
    aria-live="polite"
    transition:fly={{ y: 20, duration: 250 }}
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" class="icon-check" aria-hidden="true">
      <polyline points="20 6 9 17 4 12"/>
    </svg>
    <span>{$notification}</span>
    <button class="btn-dismiss" on:click={clearNotification} title="Dismiss" aria-label="Dismiss notification">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
        <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>
{/if}

<style>
  .snackbar {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(166, 227, 161, 0.12);
    border: 1px solid rgba(166, 227, 161, 0.3);
    color: #a6e3a1;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 13px;
    font-family: inherit;
    display: flex;
    align-items: center;
    gap: 8px;
    z-index: 3000;
    max-width: 400px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    white-space: nowrap;
  }

  .icon-check {
    flex-shrink: 0;
    color: #a6e3a1;
  }

  .btn-dismiss {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: rgba(166, 227, 161, 0.5);
    cursor: pointer;
    padding: 2px;
    border-radius: 4px;
    margin-left: 4px;
    flex-shrink: 0;
    font-family: inherit;
  }

  .btn-dismiss:hover {
    color: #a6e3a1;
    background: rgba(166, 227, 161, 0.1);
  }
</style>
