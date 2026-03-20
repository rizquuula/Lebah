<script lang="ts">
  import { createTask, updateTask } from "../stores/tasks";
  import type { Task } from "../types";

  export let task: Task | null = null;
  export let onClose: () => void;

  let description = task?.description ?? "";

  async function handleSubmit() {
    if (!description.trim()) return;

    if (task) {
      await updateTask({ ...task, description: description.trim() });
    } else {
      await createTask(description.trim());
    }
    onClose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:click={onClose}>
  <div class="modal" on:click|stopPropagation>
    <h3>{task ? "Edit Task" : "New Task"}</h3>
    <form on:submit|preventDefault={handleSubmit}>
      <textarea
        bind:value={description}
        placeholder="Task description..."
        rows="4"
      ></textarea>
      <div class="actions">
        <button type="button" class="btn-cancel" on:click={onClose}>Cancel</button>
        <button type="submit" class="btn-save">Save</button>
      </div>
    </form>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #1e1e2e;
    border-radius: 8px;
    padding: 24px;
    width: 480px;
    max-width: 90vw;
  }
  h3 {
    margin: 0 0 16px;
    color: #cdd6f4;
  }
  textarea {
    width: 100%;
    background: #313244;
    color: #cdd6f4;
    border: 1px solid #45475a;
    border-radius: 6px;
    padding: 10px;
    font-family: inherit;
    font-size: 14px;
    resize: vertical;
    box-sizing: border-box;
  }
  textarea:focus {
    outline: none;
    border-color: #89b4fa;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }
  button {
    padding: 8px 16px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }
  .btn-cancel {
    background: #45475a;
    color: #cdd6f4;
  }
  .btn-save {
    background: #89b4fa;
    color: #1e1e2e;
    font-weight: 600;
  }
</style>
