<script lang="ts">
  import { onMount } from "svelte";
  import { tasks, loadTasks } from "../stores/tasks";
  import { COLUMNS, COLUMN_COLORS, type Task, type TaskColumn } from "../types";
  import Column from "./Column.svelte";
  import TaskModal from "./TaskModal.svelte";

  onMount(() => {
    loadTasks();
  });

  let activeColumn: { key: TaskColumn; label: string } | null = null;

  function tasksByColumn(allTasks: Task[], column: TaskColumn): Task[] {
    return allTasks
      .filter((t) => t.column === column)
      .sort((a, b) => a.sort_order - b.sort_order);
  }
</script>

<div class="board">
  {#each COLUMNS as col, i}
    <div class="column-wrapper" style="animation-delay: {i * 80}ms">
      <Column
        column={col.key}
        label={col.label}
        items={tasksByColumn($tasks, col.key)}
        onAddTask={() => (activeColumn = col)}
        color={COLUMN_COLORS[col.key]}
      />
    </div>
  {/each}
</div>

{#if activeColumn}
  <TaskModal
    task={null}
    columnLabel={activeColumn.label}
    columnColor={COLUMN_COLORS[activeColumn.key]}
    onClose={() => (activeColumn = null)}
  />
{/if}

<style>
  .board {
    display: flex;
    gap: 14px;
    padding: 18px;
    height: 100%;
    overflow-x: auto;
  }
  .column-wrapper {
    flex: 1;
    min-width: 250px;
    max-width: 320px;
    animation: columnRise 0.5s ease-out both;
  }
  @keyframes columnRise {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
