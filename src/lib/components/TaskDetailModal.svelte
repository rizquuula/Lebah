<script lang="ts">
  import { type Task, STATUS_COLORS, COLUMN_COLORS } from "../types";

  export let task: Task;
  export let onClose: () => void;

  $: borderColor = STATUS_COLORS[task.status];
  $: columnColor = COLUMN_COLORS[task.column];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  const columnLabel: Record<string, string> = {
    Todo: "To-Do",
    InProgress: "In Progress",
    Review: "Review",
    Merge: "Merge",
    Completed: "Completed",
  };
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="overlay" on:click|self={onClose}>
  <div class="modal">
    <div class="modal-header" style="border-bottom-color: {borderColor}33">
      <div class="header-left">
        <span class="column-badge" style="background: {columnColor}22; color: {columnColor}; border-color: {columnColor}44">
          {columnLabel[task.column] ?? task.column}
        </span>
        <span class="status-badge" style="background: {borderColor}22; color: {borderColor}; border-color: {borderColor}44">
          {task.status}
        </span>
      </div>
      <button class="close-btn" title="Close" on:click={onClose}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <div class="section">
        <label class="section-label">Description</label>
        <p class="description">{task.description}</p>
      </div>

      <div class="details-grid">
        <div class="detail-item">
          <label class="detail-label">Model</label>
          <span class="detail-value mono">{task.model ?? "sonnet"}</span>
        </div>
        <div class="detail-item">
          <label class="detail-label">Session ID</label>
          <span class="detail-value mono id-text" title={task.id}>{task.id}</span>
        </div>
        {#if task.worktree}
          <div class="detail-item">
            <label class="detail-label">Worktree</label>
            <span class="detail-value mono">{task.worktree}</span>
          </div>
        {/if}
        {#if task.claude_path}
          <div class="detail-item">
            <label class="detail-label">Claude Path</label>
            <span class="detail-value mono">{task.claude_path}</span>
          </div>
        {/if}
        {#if task.claude_command}
          <div class="detail-item">
            <label class="detail-label">Claude Command</label>
            <span class="detail-value mono">{task.claude_command}</span>
          </div>
        {/if}
        <div class="detail-item">
          <label class="detail-label">Created</label>
          <span class="detail-value">{new Date(task.created_at).toLocaleString()}</span>
        </div>
        <div class="detail-item">
          <label class="detail-label">Flags</label>
          <span class="detail-value flags">
            {#if task.use_plan}<span class="flag plan">Plan</span>{/if}
            {#if task.yolo}<span class="flag yolo">Yolo</span>{/if}
            {#if !task.use_plan && !task.yolo}<span class="flag none">None</span>{/if}
          </span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .modal {
    background: #1e1e2e;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    width: min(700px, 90vw);
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }
  .header-left { display: flex; gap: 8px; align-items: center; }
  .column-badge, .status-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 3px 8px;
    border-radius: 5px;
    border: 1px solid;
  }
  .close-btn {
    background: rgba(82, 82, 91, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: rgba(205, 214, 244, 0.6);
    border-radius: 7px;
    width: 30px;
    height: 30px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .close-btn:hover {
    background: rgba(243, 139, 168, 0.15);
    border-color: rgba(243, 139, 168, 0.3);
    color: #f38ba8;
  }
  .modal-body {
    padding: 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .section-label, .detail-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: rgba(108, 112, 134, 0.8);
    margin-bottom: 8px;
    display: block;
  }
  .description {
    font-size: 15px;
    color: rgba(205, 214, 244, 0.9);
    line-height: 1.7;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .details-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .detail-item { display: flex; flex-direction: column; gap: 4px; }
  .detail-value {
    font-size: 13px;
    color: rgba(205, 214, 244, 0.75);
  }
  .detail-value.mono {
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12px;
  }
  .id-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .flags { display: flex; gap: 6px; }
  .flag {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    padding: 2px 7px;
    border-radius: 4px;
    border: 1px solid;
  }
  .flag.plan { background: rgba(203, 166, 247, 0.15); color: #cba6f7; border-color: rgba(203, 166, 247, 0.3); }
  .flag.yolo { background: rgba(243, 139, 168, 0.15); color: #f38ba8; border-color: rgba(243, 139, 168, 0.3); }
  .flag.none { background: rgba(108, 112, 134, 0.15); color: rgba(108, 112, 134, 0.6); border-color: rgba(108, 112, 134, 0.2); }
</style>
