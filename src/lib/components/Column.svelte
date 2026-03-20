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
  <div class="header">
    <h2>{label}</h2>
    <span class="count">{items.length}</span>
    <button class="btn-add" on:click={() => (showNewModal = true)}>+</button>
  </div>

  <div
    class="task-list"
    use:dndzone={{ items, flipDurationMs: 200, dropTargetStyle: { outline: "2px dashed #89b4fa" } }}
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
    background: #1e1e2e;
    border-radius: 8px;
    padding: 12px;
    min-width: 250px;
    max-width: 300px;
    flex: 1;
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 80px);
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }
  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: #cdd6f4;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .count {
    background: #45475a;
    color: #a6adc8;
    font-size: 12px;
    padding: 1px 7px;
    border-radius: 10px;
  }
  .btn-add {
    margin-left: auto;
    background: #45475a;
    color: #cdd6f4;
    border: none;
    border-radius: 4px;
    width: 24px;
    height: 24px;
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-add:hover {
    background: #89b4fa;
    color: #1e1e2e;
  }
  .task-list {
    flex: 1;
    overflow-y: auto;
    min-height: 50px;
  }
</style>
