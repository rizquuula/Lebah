<script lang="ts">
  import { updateTask, runClaudeSession, stopClaudeSession, deleteTask, resetTaskSession } from "../stores/tasks";
  import { STATUS_COLORS, type Task } from "../types";
  import TerminalModal from "./TerminalModal.svelte";
  import TaskModal from "./TaskModal.svelte";
  import { portal } from "../actions/portal";
  import { dragHandle } from "svelte-dnd-action";

  export let task: Task;

  let showTerminal = false;
  let showEditModal = false;
  let showConfirmReset = false;
  let showConfirmDelete = false;

  $: borderColor = STATUS_COLORS[task.status];
  $: isRunning = task.status === "Running";
  $: glowColor = task.status === "Running" ? "rgba(234, 179, 8, 0.15)"
    : task.status === "Success" ? "rgba(34, 197, 94, 0.1)"
    : task.status === "Failed" ? "rgba(239, 68, 68, 0.1)"
    : "transparent";

  async function togglePlan() {
    await updateTask({ ...task, use_plan: !task.use_plan });
  }

  async function toggleYolo() {
    await updateTask({ ...task, yolo: !task.yolo });
  }

  async function handlePlay() {
    if (task.status === "Running") {
      await stopClaudeSession(task.id);
    } else if (task.has_run) {
      showConfirmReset = true;
    } else {
      try {
        await runClaudeSession(task.id, task.description, task.use_plan, task.yolo, task.claude_path, task.claude_command, task.worktree);
      } catch (e) {
        showTerminal = true;
      }
    }
  }

  async function handleConfirmReset() {
    showConfirmReset = false;
    try {
      const newTask = await resetTaskSession(task.id);
      await runClaudeSession(newTask.id, newTask.description, newTask.use_plan, newTask.yolo, newTask.claude_path, newTask.claude_command, newTask.worktree);
    } catch (e) {
      showTerminal = true;
    }
  }
</script>

<div
  class="card"
  class:running={isRunning}
  style="--border-color: {borderColor}; --glow-color: {glowColor}"
>
  <div class="card-border-top"></div>
  <div class="drag-handle" use:dragHandle title="Drag to move">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="9" cy="5" r="1.5"/><circle cx="15" cy="5" r="1.5"/>
      <circle cx="9" cy="12" r="1.5"/><circle cx="15" cy="12" r="1.5"/>
      <circle cx="9" cy="19" r="1.5"/><circle cx="15" cy="19" r="1.5"/>
    </svg>
  </div>
  <p class="description">{task.description}</p>

  <div class="controls">
    {#if task.column !== "Todo"}
      <div class="toggles">
        <label class="toggle" title="Use Plan">
          <div class="toggle-track" class:active={task.use_plan}>
            <div class="toggle-thumb"></div>
          </div>
          <input type="checkbox" checked={task.use_plan} on:change={togglePlan} class="sr-only" />
          <span class="toggle-label">Plan</span>
        </label>
        <label class="toggle yolo-toggle" title="Skip permissions (--dangerously-skip-permissions)">
          <div class="toggle-track" class:active={task.yolo} class:yolo={task.yolo}>
            <div class="toggle-thumb"></div>
          </div>
          <input type="checkbox" checked={task.yolo} on:change={toggleYolo} class="sr-only" />
          <span class="toggle-label">Yolo</span>
        </label>
      </div>

      <button
        class="btn-icon play"
        class:active={isRunning}
        title={isRunning ? "Stop" : "Run"}
        on:click={handlePlay}
      >
        {#if isRunning}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <rect x="1" y="1" width="10" height="10" rx="1"/>
          </svg>
        {:else}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M2 1.5l9 4.5-9 4.5V1.5z"/>
          </svg>
        {/if}
      </button>

      <button
        class="btn-icon terminal-btn"
        class:active={showTerminal}
        title="Terminal"
        on:click={() => (showTerminal = !showTerminal)}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
      </button>
    {/if}

    <div class="actions">
      <button class="btn-icon" title="Edit" on:click={() => (showEditModal = true)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
        </svg>
      </button>
      <button class="btn-icon delete" title="Delete" on:click={() => (showConfirmDelete = true)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    </div>
  </div>

  <div class="meta">
    <span class="uuid" title={task.id}>{task.id.slice(0, 8)}</span>
    <span class="status" style="color: {borderColor}">{task.status}</span>
  </div>

</div>

{#if showTerminal}
  <div use:portal>
    <TerminalModal {task} onClose={() => (showTerminal = false)} />
  </div>
{/if}

{#if showEditModal}
  <div use:portal>
    <TaskModal {task} onClose={() => (showEditModal = false)} />
  </div>
{/if}

{#if showConfirmDelete}
  <div use:portal>
    <div class="confirm-overlay" role="presentation" on:click={() => (showConfirmDelete = false)} on:keydown={(e) => e.key === 'Escape' && (showConfirmDelete = false)}>
      <div class="confirm-dialog" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown|stopPropagation>
        <p class="confirm-title">Delete task?</p>
        <p class="confirm-detail">This will permanently remove the task and its worktree.</p>
        <div class="confirm-actions">
          <button class="btn-cancel" on:click={() => (showConfirmDelete = false)}>Cancel</button>
          <button class="btn-confirm" on:click={() => { showConfirmDelete = false; deleteTask(task.id); }}>Delete</button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if showConfirmReset}
  <div use:portal>
    <div class="confirm-overlay" role="presentation" on:click={() => (showConfirmReset = false)} on:keydown={(e) => e.key === 'Escape' && (showConfirmReset = false)}>
      <div class="confirm-dialog" role="dialog" aria-modal="true" tabindex="-1" on:click|stopPropagation on:keydown|stopPropagation>
        <p class="confirm-title">Start over?</p>
        <p class="confirm-detail">The worktree will be removed and a fresh Claude session will start.</p>
        <div class="confirm-actions">
          <button class="btn-cancel" on:click={() => (showConfirmReset = false)}>Cancel</button>
          <button class="btn-confirm" on:click={handleConfirmReset}>Start Over</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .card {
    background: rgba(63, 63, 70, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 12px;
    margin-bottom: 8px;
    position: relative;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  .card:hover {
    border-color: rgba(255, 255, 255, 0.1);
  }
  .card.running::before {
    content: '';
    position: absolute;
    inset: -2px;
    border-radius: inherit;
    box-shadow: 0 0 18px rgba(234, 179, 8, 0.2);
    pointer-events: none;
  }
  .drag-handle {
    position: absolute;
    top: 8px;
    right: 8px;
    color: rgba(205, 214, 244, 0.2);
    cursor: grab;
    padding: 2px;
    border-radius: 4px;
    line-height: 0;
  }
  .card:hover .drag-handle {
    color: rgba(205, 214, 244, 0.5);
  }
  .drag-handle:active {
    cursor: grabbing;
  }
  .card-border-top {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--border-color);
    opacity: 0.8;
  }
  .card:hover .card-border-top {
    opacity: 1;
  }
  .description {
    color: rgba(205, 214, 244, 0.9);
    font-size: 13px;
    margin: 4px 0 10px;
    user-select: none;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.5;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .toggles {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .btn-icon {
    background: rgba(82, 82, 91, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 7px;
    width: 30px;
    height: 30px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-icon:hover {
    background: rgba(137, 180, 250, 0.15);
    border-color: rgba(137, 180, 250, 0.2);
    color: #cdd6f4;
  }
  .btn-icon.play {
    background: rgba(166, 227, 161, 0.15);
    color: #a6e3a1;
    border-color: rgba(166, 227, 161, 0.2);
  }
  .btn-icon.play:hover {
    background: rgba(166, 227, 161, 0.3);
  }
  .btn-icon.play.active {
    background: rgba(234, 179, 8, 0.15);
    color: #eab308;
    border-color: rgba(234, 179, 8, 0.2);
    position: relative;
  }
  .btn-icon.play.active::before {
    content: '';
    position: absolute;
    inset: -2px;
    border-radius: inherit;
    box-shadow: 0 0 8px rgba(234, 179, 8, 0.3);
    pointer-events: none;
  }
  .btn-icon.terminal-btn.active {
    background: rgba(137, 180, 250, 0.2);
    color: #89b4fa;
    border-color: rgba(137, 180, 250, 0.25);
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .btn-icon.delete:hover {
    background: rgba(243, 139, 168, 0.2);
    border-color: rgba(243, 139, 168, 0.3);
    color: #f38ba8;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: rgba(166, 173, 200, 0.8);
    cursor: pointer;
    user-select: none;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
  }
  .toggle-track {
    width: 28px;
    height: 16px;
    border-radius: 8px;
    background: rgba(82, 82, 91, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    position: relative;
  }
  .toggle-track.active {
    background: rgba(137, 180, 250, 0.25);
    border-color: rgba(137, 180, 250, 0.35);
  }
  .toggle-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: rgba(205, 214, 244, 0.6);
    position: absolute;
    top: 1px;
    left: 1px;
  }
  .toggle-track.active .toggle-thumb {
    left: 13px;
    background: #89b4fa;
    box-shadow: 0 0 6px rgba(137, 180, 250, 0.4);
  }
  .toggle-track.yolo {
    background: rgba(249, 115, 22, 0.25);
    border-color: rgba(249, 115, 22, 0.4);
  }
  .toggle-track.yolo .toggle-thumb {
    background: #f97316;
    box-shadow: 0 0 6px rgba(249, 115, 22, 0.4);
  }
  .toggle-label {
    user-select: none;
  }
  .meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: rgba(108, 112, 134, 0.7);
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid rgba(255, 255, 255, 0.03);
  }
  .uuid {
    font-family: "JetBrains Mono", "Fira Code", monospace;
    letter-spacing: 0.3px;
  }
  .status {
    font-weight: 600;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.5px;
  }
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
  .btn-cancel:hover {
    background: rgba(82, 82, 91, 0.8);
    color: #cdd6f4;
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
  }
  .btn-confirm:hover {
    background: rgba(239, 68, 68, 0.35);
  }
</style>
