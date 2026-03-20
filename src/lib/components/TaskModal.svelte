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
    <div class="modal-glow"></div>
    <h3>{task ? "Edit Task" : "New Task"}</h3>
    <form on:submit|preventDefault={handleSubmit}>
      <textarea
        bind:value={description}
        placeholder="Describe the task for Claude..."
        rows="4"
      ></textarea>
      <div class="actions">
        <button type="button" class="btn-cancel" on:click={onClose}>Cancel</button>
        <button type="submit" class="btn-save">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          Save
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: overlayFade 0.2s ease-out;
  }
  @keyframes overlayFade {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  .modal {
    background: rgba(30, 30, 46, 0.75);
    backdrop-filter: blur(24px) saturate(1.5);
    -webkit-backdrop-filter: blur(24px) saturate(1.5);
    border: 1px solid rgba(137, 180, 250, 0.12);
    border-radius: 16px;
    padding: 28px;
    width: 480px;
    max-width: 90vw;
    position: relative;
    overflow: hidden;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4), 0 0 40px rgba(137, 180, 250, 0.05);
    animation: modalPop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  @keyframes modalPop {
    from { opacity: 0; transform: scale(0.92) translateY(10px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }
  .modal-glow {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 50%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.5), transparent);
  }
  h3 {
    margin: 0 0 18px;
    color: rgba(205, 214, 244, 0.9);
    font-size: 16px;
    font-weight: 600;
  }
  textarea {
    width: 100%;
    background: rgba(49, 50, 68, 0.5);
    color: #cdd6f4;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 12px;
    font-family: inherit;
    font-size: 14px;
    resize: vertical;
    box-sizing: border-box;
    transition: border-color 0.25s ease, box-shadow 0.25s ease;
    line-height: 1.5;
  }
  textarea:focus {
    outline: none;
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 16px rgba(137, 180, 250, 0.08);
  }
  textarea::placeholder {
    color: rgba(108, 112, 134, 0.6);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  button {
    padding: 9px 18px;
    border: none;
    border-radius: 9px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
    transition: all 0.2s ease;
  }
  .btn-cancel {
    background: rgba(69, 71, 90, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }
  .btn-cancel:hover {
    background: rgba(69, 71, 90, 0.7);
    color: #cdd6f4;
  }
  .btn-save {
    background: rgba(137, 180, 250, 0.2);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.25);
    font-weight: 600;
  }
  .btn-save:hover {
    background: rgba(137, 180, 250, 0.3);
    box-shadow: 0 0 16px rgba(137, 180, 250, 0.15);
    transform: translateY(-1px);
  }
  .btn-save:active {
    transform: translateY(0);
  }
</style>
