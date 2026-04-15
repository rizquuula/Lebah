<script lang="ts">
  import { onMount } from "svelte";
  import { tasks, loadTasks } from "../stores/tasks";
  import { loadProjectConfig } from "../stores/config";
  import { COLUMNS, COLUMN_COLORS, TaskColumn, type Task } from "../types";
  import Column from "./Column.svelte";
  import TaskModal from "./TaskModal.svelte";

  onMount(() => {
    loadTasks();
    loadProjectConfig();
  });

  let activeColumn: { key: TaskColumn; label: string } | null = null;

  function tasksByColumn(allTasks: Task[], column: TaskColumn): Task[] {
    const filtered = allTasks.filter((t) => t.column === column);
    if (column === TaskColumn.Completed) {
      return filtered.sort((a, b) => {
        const ta = a.completed_at ?? a.created_at;
        const tb = b.completed_at ?? b.created_at;
        return tb.localeCompare(ta);
      });
    }
    return filtered.sort((a, b) => a.sort_order - b.sort_order);
  }
</script>

<div class="board" data-testid="board">
  {#each COLUMNS as col, i}
    <div class="column-wrapper" data-testid="column-{col.key}" style="animation-delay: {i * 80}ms">
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
  }
</style>
