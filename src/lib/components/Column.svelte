<script lang="ts">
  import { dndzone } from "svelte-dnd-action";
  import { moveTask } from "../stores/tasks";
  import type { Task, TaskColumn } from "../types";
  import TaskCard from "./TaskCard.svelte";
  import TaskModal from "./TaskModal.svelte";

  export let column: TaskColumn;
  export let label: string;
  export let items: Task[];

  let showNewModal = false;

  function handleDndConsider(e: CustomEvent) {
    items = e.detail.items;
  }

  async function handleDndFinalize(e: CustomEvent) {
    items = e.detail.items;
    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      if (item.column !== column || item.sort_order !== i) {
        await moveTask(item.id, column, i);
      }
    }
  }
</script>

<div class="column">
  <div class="column-header-glow"></div>
  <div class="header">
    <h2>{label}</h2>
    <span class="count">{items.length}</span>
    <button class="btn-add" on:click={() => (showNewModal = true)}>
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
    </button>
  </div>

  <div
    class="task-list"
    use:dndzone={{ items, flipDurationMs: 250, dropTargetStyle: { outline: "2px dashed rgba(137, 180, 250, 0.4)", borderRadius: "8px" } }}
    on:consider={handleDndConsider}
    on:finalize={handleDndFinalize}
  >
    {#each items as task (task.id)}
      <TaskCard {task} />
    {/each}
  </div>
</div>

{#if showNewModal}
  <TaskModal task={null} onClose={() => (showNewModal = false)} />
{/if}

<style>
  .column {
    background: rgba(30, 30, 46, 0.45);
    backdrop-filter: blur(16px) saturate(1.3);
    -webkit-backdrop-filter: blur(16px) saturate(1.3);
    border: 1px solid rgba(137, 180, 250, 0.08);
    border-radius: 14px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 80px);
    height: 100%;
    position: relative;
    overflow: hidden;
    transition: border-color 0.3s ease, box-shadow 0.3s ease;
  }
  .column:hover {
    border-color: rgba(137, 180, 250, 0.15);
    box-shadow: 0 0 30px rgba(137, 180, 250, 0.05);
  }
  .column-header-glow {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 60%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.3), transparent);
    opacity: 0.6;
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 14px;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: rgba(205, 214, 244, 0.85);
    text-transform: uppercase;
    letter-spacing: 0.8px;
  }
  .count {
    background: rgba(137, 180, 250, 0.12);
    color: #89b4fa;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    transition: background 0.2s ease;
  }
  .btn-add {
    margin-left: auto;
    background: rgba(137, 180, 250, 0.1);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.15);
    border-radius: 8px;
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.25s ease;
  }
  .btn-add:hover {
    background: rgba(137, 180, 250, 0.25);
    border-color: rgba(137, 180, 250, 0.4);
    box-shadow: 0 0 12px rgba(137, 180, 250, 0.2);
    transform: scale(1.1);
  }
  .btn-add:active {
    transform: scale(0.95);
  }
  .task-list {
    flex: 1;
    overflow-y: auto;
    min-height: 50px;
    padding-right: 2px;
  }
</style>
