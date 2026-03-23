<script lang="ts">
  import { updateTask, moveTask, runClaudeSession, stopClaudeSession, deleteTask, resetTaskSession, sendInputWithListener, isAnyMergeRunning, queueMergeTask, cancelMergeWait } from "../stores/tasks";
  import { projectConfig } from "../stores/config";
  import { setError } from "../stores/errors";
  import { STATUS_COLORS, DEFAULT_REVIEW_TEMPLATE, DEFAULT_MERGE_TEMPLATE, DEFAULT_INPROGRESS_TEMPLATE, type Task } from "../types";
  import TerminalModal from "./TerminalModal.svelte";
  import TaskModal from "./TaskModal.svelte";
  import TaskDetailModal from "./TaskDetailModal.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import TaskToggles from "./TaskToggles.svelte";
  import { portal } from "../actions/portal";
  import { dragHandle } from "svelte-dnd-action";

  export let task: Task;

  let showTerminal = false;
  let showEditModal = false;
  let showDetailModal = false;
  let showConfirmReset = false;
  let showConfirmDelete = false;
  let isPlaying = false;
  let isDeleting = false;
  let isResetting = false;
  let isMoving = false;

  $: borderColor = STATUS_COLORS[task.status];
  $: displayDate = (() => {
    const raw = task.column === "Completed" && task.completed_at ? task.completed_at : task.created_at;
    const d = new Date(raw);
    return d.toLocaleString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
  })();
  $: isRunning = task.status === "Running";
  $: isWaiting = task.status === "Waiting";
  $: glowColor = task.status === "Running" ? "rgba(234, 179, 8, 0.15)"
    : task.status === "Success" ? "rgba(34, 197, 94, 0.1)"
    : task.status === "Failed" ? "rgba(239, 68, 68, 0.1)"
    : "transparent";

  function getTemplate(): string | null {
    if (task.column === "InProgress") {
      const tpl = $projectConfig.inprogress_template ?? DEFAULT_INPROGRESS_TEMPLATE;
      return tpl.replace("<TASK_DESCRIPTION>", task.description);
    }
    if (task.column === "Review") return $projectConfig.review_template ?? DEFAULT_REVIEW_TEMPLATE;
    if (task.column === "Merge") return $projectConfig.merge_template ?? DEFAULT_MERGE_TEMPLATE;
    return null;
  }

  async function handlePlay() {
    if (isPlaying) return;
    isPlaying = true;
    try {
      if (task.status === "Running") {
        await stopClaudeSession(task.id);
      } else if (task.column === "Merge" && task.status === "Waiting") {
        // Already queued — clicking play cancels the waiting status
        cancelMergeWait(task.id);
        await updateTask({ ...task, status: "Idle" });
      } else if ((task.column === "InProgress" || task.column === "Review" || task.column === "Merge") && task.has_run) {
        const template = getTemplate();
        if (template) {
          if (task.column === "Merge" && isAnyMergeRunning()) {
            await queueMergeTask({ id: task.id, description: task.description, usePlan: task.use_plan, yolo: task.yolo, claudePath: task.claude_path, worktree: task.worktree, model: task.model, hasRun: task.has_run, template });
          } else {
            try { await sendInputWithListener(task.id, template, task.model, task.yolo); }
            catch { showTerminal = true; }
          }
        }
      } else if (task.has_run) {
        showConfirmReset = true;
      } else {
        const template = getTemplate();
        const description = task.column === "InProgress" && template ? `${task.description}\n${template}` : task.description;
        if (task.column === "Merge" && isAnyMergeRunning()) {
          await queueMergeTask({ id: task.id, description: task.description, usePlan: task.use_plan, yolo: task.yolo, claudePath: task.claude_path, worktree: task.worktree, model: task.model, hasRun: task.has_run, template });
        } else {
          try { await runClaudeSession(task.id, description, task.use_plan, task.yolo, task.claude_path, task.worktree, task.model); }
          catch { showTerminal = true; }
        }
      }
    } finally {
      isPlaying = false;
    }
  }

  async function handleMoveToInProgress() {
    if (isMoving) return;
    isMoving = true;
    try { await moveTask(task.id, "InProgress", 0); }
    catch (e) { setError(`Failed to move task: ${e}`); }
    finally { isMoving = false; }
  }

  async function handleConfirmReset() {
    if (isResetting) return;
    showConfirmReset = false;
    isResetting = true;
    try {
      const t = await resetTaskSession(task.id);
      await runClaudeSession(t.id, t.description, t.use_plan, t.yolo, t.claude_path, t.worktree, t.model);
    } catch { showTerminal = true; }
    finally { isResetting = false; }
  }

  async function handleConfirmDelete() {
    if (isDeleting) return;
    showConfirmDelete = false;
    isDeleting = true;
    try { await deleteTask(task.id); }
    catch (e) { setError(`Failed to delete task: ${e}`); }
    finally { isDeleting = false; }
  }
</script>

<div class="card" class:running={isRunning} style="--border-color: {borderColor}; --glow-color: {glowColor}">
  <div class="card-border-top"></div>
  <span class="timestamp">{displayDate}</span>
  <div class="drag-handle" use:dragHandle title="Drag to move">
    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
      <circle cx="9" cy="5" r="1.5"/><circle cx="15" cy="5" r="1.5"/>
      <circle cx="9" cy="12" r="1.5"/><circle cx="15" cy="12" r="1.5"/>
      <circle cx="9" cy="19" r="1.5"/><circle cx="15" cy="19" r="1.5"/>
    </svg>
  </div>
  <p class="description">{task.description}</p>

  <div class="controls">
    {#if task.column === "Todo"}
      <button class="btn-icon play" title="Move to In Progress" disabled={isMoving} on:click={handleMoveToInProgress}>
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><path d="M2 1.5l9 4.5-9 4.5V1.5z"/></svg>
      </button>
    {:else if task.column !== "Completed"}
      <button class="btn-icon play" class:active={isRunning} class:waiting={isWaiting}
        title={isRunning ? "Stop" : isWaiting ? "Waiting (click to cancel)" : "Run"}
        disabled={isPlaying || isResetting} on:click={handlePlay}>
        {#if isRunning}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><rect x="1" y="1" width="10" height="10" rx="1"/></svg>
        {:else if isWaiting}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><rect x="1" y="1" width="4" height="10" rx="1"/><rect x="7" y="1" width="4" height="10" rx="1"/></svg>
        {:else}
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><path d="M2 1.5l9 4.5-9 4.5V1.5z"/></svg>
        {/if}
      </button>
      <button class="btn-icon terminal-btn" class:active={showTerminal} title="Terminal"
        on:click={() => (showTerminal = !showTerminal)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
      </button>
    {/if}
    {#if task.column !== "Todo"}
      <button class="btn-icon" title="View details" on:click={() => (showDetailModal = true)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    {/if}
    <div class="actions">
      {#if task.column === "Todo"}
        <button class="btn-icon" title="Edit" on:click={() => (showEditModal = true)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
        </button>
      {/if}
      <button class="btn-icon delete" title="Delete" disabled={isDeleting}
        on:click={() => (showConfirmDelete = true)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>
      </button>
    </div>
  </div>

  {#if task.column !== "Todo" && task.column !== "Completed"}
    <div class="bottom-row">
      <TaskToggles
        usePlan={task.use_plan}
        yolo={task.yolo}
        showPlan={task.column !== "Review" && task.column !== "Merge"}
        onTogglePlan={() => updateTask({ ...task, use_plan: !task.use_plan })}
        onToggleYolo={() => updateTask({ ...task, yolo: !task.yolo })}
      />
    </div>
  {/if}

  <div class="meta">
    <span class="uuid" title={task.id}>{task.model || "sonnet"}</span>
    <span class="status" style="color: {borderColor}">{task.status}</span>
  </div>
</div>

{#if showTerminal}
  <div use:portal><TerminalModal {task} onClose={() => (showTerminal = false)} /></div>
{/if}
{#if showEditModal}
  <div use:portal><TaskModal {task} onClose={() => (showEditModal = false)} /></div>
{/if}
{#if showDetailModal}
  <div use:portal><TaskDetailModal {task} onClose={() => (showDetailModal = false)} /></div>
{/if}
{#if showConfirmDelete}
  <div use:portal>
    <ConfirmDialog title="Delete task?" detail="This will permanently remove the task and its worktree."
      confirmLabel="Delete" loading={isDeleting} onConfirm={handleConfirmDelete}
      onCancel={() => (showConfirmDelete = false)} />
  </div>
{/if}
{#if showConfirmReset}
  <div use:portal>
    <ConfirmDialog title="Start over?" detail="The worktree will be removed and a fresh Claude session will start."
      confirmLabel="Start Over" loading={isResetting} onConfirm={handleConfirmReset}
      onCancel={() => (showConfirmReset = false)} />
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
  .card:hover { border-color: rgba(255, 255, 255, 0.1); }
  .card.running::before {
    content: '';
    position: absolute;
    inset: -2px;
    border-radius: inherit;
    box-shadow: 0 0 18px rgba(234, 179, 8, 0.2);
    pointer-events: none;
  }
  .timestamp {
    position: absolute;
    top: 8px;
    left: 10px;
    font-size: 10px;
    color: rgba(108, 112, 134, 0.6);
    font-family: "JetBrains Mono", "Fira Code", monospace;
    letter-spacing: 0.2px;
    user-select: none;
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
  .card:hover .drag-handle { color: rgba(205, 214, 244, 0.5); }
  .drag-handle:active { cursor: grabbing; }
  .card-border-top {
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: var(--border-color);
    opacity: 0.8;
  }
  .card:hover .card-border-top { opacity: 1; }
  .description {
    color: rgba(205, 214, 244, 0.9);
    font-size: 13px;
    margin: 18px 0 10px;
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
  .btn-icon:hover:not(:disabled) {
    background: rgba(137, 180, 250, 0.15);
    border-color: rgba(137, 180, 250, 0.2);
    color: #cdd6f4;
  }
  .btn-icon:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-icon.play {
    background: rgba(166, 227, 161, 0.15);
    color: #a6e3a1;
    border-color: rgba(166, 227, 161, 0.2);
  }
  .btn-icon.play:hover:not(:disabled) { background: rgba(166, 227, 161, 0.3); }
  .btn-icon.play.waiting {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    border-color: rgba(59, 130, 246, 0.2);
  }
  .btn-icon.play.waiting:hover:not(:disabled) { background: rgba(59, 130, 246, 0.3); }
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
  .btn-icon.delete:hover:not(:disabled) {
    background: rgba(243, 139, 168, 0.2);
    border-color: rgba(243, 139, 168, 0.3);
    color: #f38ba8;
  }
  .actions { margin-left: auto; display: flex; gap: 6px; }
  .bottom-row {
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid rgba(255, 255, 255, 0.03);
  }
  .meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: rgba(108, 112, 134, 0.7);
    margin-top: 8px;
  }
  .uuid { font-family: "JetBrains Mono", "Fira Code", monospace; letter-spacing: 0.3px; }
  .status { font-weight: 600; text-transform: uppercase; font-size: 10px; letter-spacing: 0.5px; }
</style>
