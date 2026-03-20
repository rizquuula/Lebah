<script lang="ts">
  import { updateTask, runClaudeSession, stopClaudeSession, deleteTask } from "../stores/tasks";
  import { STATUS_COLORS, type Task } from "../types";
  import TaskTerminal from "./TaskTerminal.svelte";
  import TaskModal from "./TaskModal.svelte";

  export let task: Task;

  let showTerminal = false;
  let showEditModal = false;

  $: borderColor = STATUS_COLORS[task.status];

  async function togglePlan() {
    await updateTask({ ...task, use_plan: !task.use_plan });
  }

  async function handlePlay() {
    if (task.status === "Running") {
      await stopClaudeSession(task.id);
    } else {
      await runClaudeSession(task.id, task.description, task.use_plan);
    }
  }
</script>

<div class="card" style="border-top: 3px solid {borderColor}">
  <p class="description">{task.description}</p>

  <div class="controls">
    <button class="btn-icon" title="Edit" on:click={() => (showEditModal = true)}>
      ✎
    </button>

    <label class="toggle" title="Use Plan">
      <input type="checkbox" checked={task.use_plan} on:change={togglePlan} />
      <span class="toggle-label">Plan</span>
    </label>

    <button
      class="btn-icon play"
      title={task.status === "Running" ? "Stop" : "Run"}
      on:click={handlePlay}
    >
      {task.status === "Running" ? "⏹" : "▶"}
    </button>

    <button
      class="btn-icon"
      title="Terminal"
      on:click={() => (showTerminal = !showTerminal)}
    >
      ⌘
    </button>

    <button class="btn-icon delete" title="Delete" on:click={() => deleteTask(task.id)}>
      ✕
    </button>
  </div>

  <div class="meta">
    <span class="uuid" title={task.id}>{task.id.slice(0, 8)}</span>
    <span class="status">{task.status}</span>
  </div>

  {#if showTerminal}
    <TaskTerminal taskId={task.id} />
  {/if}
</div>

{#if showEditModal}
  <TaskModal {task} onClose={() => (showEditModal = false)} />
{/if}

<style>
  .card {
    background: #313244;
    border-radius: 6px;
    padding: 10px;
    margin-bottom: 8px;
  }
  .description {
    color: #cdd6f4;
    font-size: 13px;
    margin: 0 0 8px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .btn-icon {
    background: #45475a;
    color: #cdd6f4;
    border: none;
    border-radius: 4px;
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
  }
  .btn-icon:hover {
    background: #585b70;
  }
  .btn-icon.play {
    background: #a6e3a1;
    color: #1e1e2e;
  }
  .btn-icon.delete {
    margin-left: auto;
  }
  .btn-icon.delete:hover {
    background: #f38ba8;
    color: #1e1e2e;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: #a6adc8;
    cursor: pointer;
  }
  .toggle input {
    accent-color: #89b4fa;
  }
  .toggle-label {
    user-select: none;
  }
  .meta {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: #6c7086;
    margin-top: 6px;
  }
  .uuid {
    font-family: monospace;
  }
</style>
