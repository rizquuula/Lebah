<script lang="ts">
  import { onMount } from "svelte";
  import { tasks, loadTasks } from "../stores/tasks";
  import { COLUMNS, type Task, type TaskColumn } from "../types";
  import Column from "./Column.svelte";

  onMount(() => {
    loadTasks();
  });

  function tasksByColumn(allTasks: Task[], column: TaskColumn): Task[] {
    return allTasks
      .filter((t) => t.column === column)
      .sort((a, b) => a.sort_order - b.sort_order);
  }
</script>

<div class="board">
  {#each COLUMNS as col}
    <Column
      column={col.key}
      label={col.label}
      items={tasksByColumn($tasks, col.key)}
    />
  {/each}
</div>

<style>
  .board {
    display: flex;
    gap: 12px;
    padding: 16px;
    height: 100%;
    overflow-x: auto;
  }
</style>
