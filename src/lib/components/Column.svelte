<script lang="ts">
  import { dragHandleZone } from "svelte-dnd-action";
  import { moveTaskBatch } from "../stores/tasks";
  import { projectConfig, saveProjectConfig } from "../stores/config";
  import { DEFAULT_REVIEW_TEMPLATE, DEFAULT_MERGE_TEMPLATE, type Task, type TaskColumn } from "../types";
  import TaskCard from "./TaskCard.svelte";

  export let column: TaskColumn;
  export let label: string;
  export let items: Task[];
  export let onAddTask: () => void;
  export let color: string = "#89b4fa";

  let showTemplatePopover = false;
  let editingTemplate = "";

  $: hasTemplate = column === "Review" || column === "Merge";

  function openTemplatePopover() {
    if (column === "Review") {
      editingTemplate = $projectConfig.review_template ?? DEFAULT_REVIEW_TEMPLATE;
    } else {
      editingTemplate = $projectConfig.merge_template ?? DEFAULT_MERGE_TEMPLATE;
    }
    showTemplatePopover = !showTemplatePopover;
  }

  async function saveTemplate() {
    const updated = { ...$projectConfig };
    if (column === "Review") updated.review_template = editingTemplate;
    else updated.merge_template = editingTemplate;
    await saveProjectConfig(updated);
    showTemplatePopover = false;
  }

  function handleDndConsider(e: CustomEvent) {
    items = e.detail.items;
  }

  async function handleDndFinalize(e: CustomEvent) {
    items = e.detail.items;
    const moves = items
      .map((item, i) => ({ id: item.id, column, sortOrder: i, item }))
      .filter(({ item, sortOrder }) => item.column !== column || item.sort_order !== sortOrder)
      .map(({ id, column: col, sortOrder }) => ({ id, column: col, sortOrder }));
    if (moves.length > 0) {
      await moveTaskBatch(moves);
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
    {#if hasTemplate}
      <div class="template-wrapper">
        <button class="btn-add" on:click={openTemplatePopover} title="Template message" aria-label="Template message">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
        </button>
        {#if showTemplatePopover}
          <div class="template-popover">
            <textarea bind:value={editingTemplate} rows="4"></textarea>
            <div class="template-actions">
              <button class="btn-tpl-cancel" on:click={() => (showTemplatePopover = false)}>Cancel</button>
              <button class="btn-tpl-save" on:click={saveTemplate}>Save</button>
            </div>
          </div>
        {/if}
      </div>
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
  .template-wrapper {
    margin-left: auto;
    position: relative;
  }
  .template-popover {
    position: absolute;
    top: 36px;
    right: 0;
    width: 280px;
    background: #1e1e2e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    padding: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 100;
  }
  .template-popover textarea {
    width: 100%;
    background: rgba(63, 63, 70, 0.6);
    color: rgba(205, 214, 244, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 8px;
    font-size: 12px;
    font-family: inherit;
    resize: vertical;
    line-height: 1.5;
    box-sizing: border-box;
  }
  .template-popover textarea:focus {
    outline: none;
    border-color: color-mix(in srgb, var(--col-color) 50%, transparent);
  }
  .template-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    margin-top: 8px;
  }
  .btn-tpl-cancel {
    background: rgba(82, 82, 91, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 11px;
    cursor: pointer;
  }
  .btn-tpl-cancel:hover {
    background: rgba(82, 82, 91, 0.8);
  }
  .btn-tpl-save {
    background: color-mix(in srgb, var(--col-color) 20%, transparent);
    color: var(--col-color);
    border: 1px solid color-mix(in srgb, var(--col-color) 30%, transparent);
    border-radius: 6px;
    padding: 4px 10px;
    font-size: 11px;
    cursor: pointer;
    font-weight: 600;
  }
  .btn-tpl-save:hover {
    background: color-mix(in srgb, var(--col-color) 35%, transparent);
  }
  .task-list {
    flex: 1;
    overflow-y: auto;
    min-height: 50px;
    padding-right: 2px;
  }
</style>
