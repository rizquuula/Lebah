<script lang="ts">
  import { dragHandleZone } from "svelte-dnd-action";
  import { moveTask } from "../stores/tasks";
  import type { Task, TaskColumn } from "../types";
  import TaskCard from "./TaskCard.svelte";

  export let column: TaskColumn;
  export let label: string;
  export let items: Task[];
  export let onAddTask: () => void;
  export let color: string = "#89b4fa";

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

<div class="column" style="--col-color: {color}">
  <div class="column-header-glow"></div>
  <div class="header">
    <h2>{label}</h2>
    <span class="count">{items.length}</span>
    {#if column === "Todo"}
      <button class="btn-add" on:click={onAddTask} title="Add task" aria-label="Add task">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M7 1v12M1 7h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    {/if}
  </div>

  <div
    class="task-list"
    use:dragHandleZone={{ items, flipDurationMs: 250, dropTargetStyle: { outline: "2px dashed rgba(137, 180, 250, 0.4)", borderRadius: "8px" } }}
    on:consider={handleDndConsider}
    on:finalize={handleDndFinalize}
  >
    {#each items as task (task.id)}
      <TaskCard {task} />
    {/each}
  </div>
</div>

<style>
  .column {
    background: rgba(39, 39, 42, 0.85);
    border: 1px solid color-mix(in srgb, var(--col-color) 12%, transparent);
    border-radius: 14px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 80px);
    height: 100%;
    position: relative;
    overflow: hidden;
  }
  .column:hover {
    border-color: color-mix(in srgb, var(--col-color) 22%, transparent);
  }
  .column-header-glow {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 60%;
    height: 1px;
    background: linear-gradient(90deg, transparent, color-mix(in srgb, var(--col-color) 40%, transparent), transparent);
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
    background: color-mix(in srgb, var(--col-color) 15%, transparent);
    color: var(--col-color);
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
  }
  .btn-add {
    margin-left: auto;
    background: color-mix(in srgb, var(--col-color) 13%, transparent);
    color: var(--col-color);
    border: 1px solid color-mix(in srgb, var(--col-color) 20%, transparent);
    border-radius: 8px;
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-add:hover {
    background: color-mix(in srgb, var(--col-color) 28%, transparent);
    border-color: color-mix(in srgb, var(--col-color) 45%, transparent);
  }
  .task-list {
    flex: 1;
    overflow-y: auto;
    min-height: 50px;
    padding-right: 2px;
  }
</style>
