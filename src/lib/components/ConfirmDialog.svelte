<script lang="ts">
  export let title: string;
  export let detail: string;
  export let confirmLabel: string = "Confirm";
  export let loading: boolean = false;
  export let onConfirm: () => void;
  export let onCancel: () => void;
</script>

<div class="confirm-overlay" role="presentation" on:click={onCancel} on:keydown={(e) => e.key === 'Escape' && onCancel()}>
  <div class="confirm-dialog" data-testid="confirm-dialog" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown|stopPropagation>
    <p class="confirm-title" data-testid="confirm-title">{title}</p>
    <p class="confirm-detail">{detail}</p>
    <div class="confirm-actions">
      <button class="btn-cancel" data-testid="confirm-cancel-btn" disabled={loading} on:click={onCancel}>Cancel</button>
      <button class="btn-confirm" data-testid="confirm-ok-btn" disabled={loading} on:click={onConfirm}>
        {loading ? "…" : confirmLabel}
      </button>
    </div>
  </div>
</div>

<style>
  .confirm-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .confirm-dialog {
    background: #1e1e2e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 24px;
    max-width: 320px;
    width: 90%;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  .confirm-title {
    color: #cdd6f4;
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 8px;
  }
  .confirm-detail {
    color: rgba(205, 214, 244, 0.6);
    font-size: 13px;
    margin: 0 0 20px;
    line-height: 1.5;
  }
  .confirm-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .btn-cancel {
    background: rgba(82, 82, 91, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 7px;
    padding: 6px 14px;
    font-size: 13px;
    cursor: pointer;
  }
  .btn-cancel:hover:not(:disabled) {
    background: rgba(82, 82, 91, 0.8);
    color: #cdd6f4;
  }
  .btn-cancel:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-confirm {
    background: rgba(239, 68, 68, 0.2);
    color: #f38ba8;
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 7px;
    padding: 6px 14px;
    font-size: 13px;
    cursor: pointer;
    font-weight: 600;
    min-width: 72px;
  }
  .btn-confirm:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.35);
  }
  .btn-confirm:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
