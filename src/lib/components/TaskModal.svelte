<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createTask, updateTask } from "../stores/tasks";
  import type { Task } from "../types";

  export let task: Task | null = null;
  export let onClose: () => void;
  export let columnLabel: string = "";
  export let columnColor: string = "#89b4fa";

  let description = task?.description ?? "";
  let claudePath = task?.claude_path ?? "";
  let worktree = task?.worktree ?? "";
  let worktreeError = "";
  let model = task?.model ?? "sonnet";
  let generatingWorktree = false;

  async function generateWorktreeName() {
    worktreeError = "";
    generatingWorktree = true;
    try {
      const name = await invoke<string>("generate_worktree_name", {
        description: description.trim(),
        model: model || null,
        claudePath: claudePath.trim() || null,
      });
      worktree = name.trim();
    } catch (e) {
      worktreeError = String(e);
    } finally {
      generatingWorktree = false;
    }
  }

  async function handleSubmit() {
    if (!description.trim()) return;

    const pathVal = claudePath.trim() || null;
    const worktreeVal = worktree.trim().replace(/\//g, '-') || null;
    const modelVal = model.trim() || null;

    if (!task) {
      if (!worktreeVal) {
        worktreeError = "Worktree name is required";
        return;
      }
    }
    worktreeError = "";

    try {
      if (task) {
        await updateTask({
          ...task,
          description: description.trim(),
          claude_path: pathVal,
          model: modelVal,
        });
      } else {
        await createTask(description.trim(), pathVal, worktreeVal, modelVal);
      }
      onClose();
    } catch (e) {
      worktreeError = String(e);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:click={onClose}>
  <div class="modal" on:click|stopPropagation>
    <div class="modal-glow"></div>
    <h3>{task ? "Edit Task" : "New Task"}</h3>
    {#if columnLabel && !task}
      <div class="stage-badge">Adding to <span style="--stage-color: {columnColor}">{columnLabel}</span></div>
    {/if}
    <form on:submit|preventDefault={handleSubmit}>
      <label class="field-label" for="task-description">Description</label>
      <textarea
        id="task-description"
        bind:value={description}
        placeholder="Describe the task for Claude..."
        rows="4"
      ></textarea>

      <label class="field-label" for="task-model">Model</label>
      <select id="task-model" bind:value={model} class="text-input">
        <option value="sonnet">sonnet</option>
        <option value="opus">opus</option>
        <option value="haiku">haiku</option>
      </select>

      <label class="field-label" for="task-claude-path">Claude Code Path</label>
      <input
        id="task-claude-path"
        type="text"
        bind:value={claudePath}
        placeholder="claude (default)"
        class="text-input"
      />

      <label class="field-label" for="task-worktree">Worktree Name</label>
      {#if task}
        <div class="text-input readonly-field">{task.worktree ?? "—"}</div>
      {:else}
        <div class="worktree-row">
          <input
            id="task-worktree"
            type="text"
            bind:value={worktree}
            placeholder="feat-my-feature"
            class="text-input"
            class:input-error={!!worktreeError}
            disabled={generatingWorktree}
          />
          <button
            type="button"
            class="btn-generate"
            on:click={generateWorktreeName}
            disabled={generatingWorktree || !description.trim()}
            title="Generate worktree name with AI"
          >
            <svg class="star-icon" class:spin={generatingWorktree} width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
          </button>
        </div>
        {#if worktreeError}
          <div class="field-error">{worktreeError}</div>
        {/if}
      {/if}

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
  }
  .modal {
    background: rgba(39, 39, 42, 0.75);
    backdrop-filter: blur(24px) saturate(1.5);
    -webkit-backdrop-filter: blur(24px) saturate(1.5);
    border: 1px solid rgba(137, 180, 250, 0.12);
    border-radius: 16px;
    padding: 28px;
    width: 520px;
    max-width: 90vw;
    position: relative;
    overflow: hidden;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4), 0 0 40px rgba(137, 180, 250, 0.05);
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
  .stage-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 500;
    color: rgba(205, 214, 244, 0.45);
    text-transform: uppercase;
    letter-spacing: 0.6px;
    margin-bottom: 18px;
    margin-top: -10px;
  }
  .stage-badge span {
    color: var(--stage-color, #89b4fa);
    background: color-mix(in srgb, var(--stage-color, #89b4fa) 13%, transparent);
    border: 1px solid color-mix(in srgb, var(--stage-color, #89b4fa) 25%, transparent);
    border-radius: 6px;
    padding: 2px 8px;
    font-weight: 600;
  }
  .field-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: rgba(205, 214, 244, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 6px;
    margin-top: 14px;
  }
  .field-label:first-of-type {
    margin-top: 0;
  }
  textarea, .text-input {
    width: 100%;
    background: rgba(63, 63, 70, 0.5);
    color: #cdd6f4;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 12px;
    font-family: inherit;
    font-size: 14px;
    box-sizing: border-box;
    line-height: 1.5;
  }
  textarea {
    resize: vertical;
  }
  .text-input {
    height: 42px;
  }
  textarea:focus, .text-input:focus {
    outline: none;
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 16px rgba(137, 180, 250, 0.08);
  }
  select.text-input {
    color-scheme: dark;
    padding: 0 12px;
    appearance: auto;
  }
  textarea::placeholder, .text-input::placeholder {
    color: rgba(108, 112, 134, 0.6);
  }
  .worktree-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .worktree-row .text-input {
    flex: 1;
  }
  .btn-generate {
    flex-shrink: 0;
    width: 42px;
    height: 42px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(203, 166, 247, 0.15);
    color: #cba6f7;
    border: 1px solid rgba(203, 166, 247, 0.25);
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.15s, box-shadow 0.15s, transform 0.1s;
  }
  .btn-generate:hover:not(:disabled) {
    background: rgba(203, 166, 247, 0.28);
    box-shadow: 0 0 14px rgba(203, 166, 247, 0.15);
    transform: translateY(-1px);
  }
  .btn-generate:active:not(:disabled) {
    transform: translateY(0);
  }
  .btn-generate:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .spin {
    animation: spin 0.8s linear infinite;
  }
  .readonly-field {
    display: flex;
    align-items: center;
    color: rgba(205, 214, 244, 0.45);
    cursor: default;
    user-select: text;
  }
  .input-error {
    border-color: rgba(239, 68, 68, 0.5) !important;
    box-shadow: 0 0 12px rgba(239, 68, 68, 0.1);
  }
  .field-error {
    font-size: 12px;
    color: rgba(239, 68, 68, 0.85);
    margin-top: 4px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 20px;
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
  }
  .btn-cancel {
    background: rgba(82, 82, 91, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }
  .btn-cancel:hover {
    background: rgba(82, 82, 91, 0.7);
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
  .worktree-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .worktree-row .text-input {
    flex: 1;
  }
  .btn-generate {
    flex-shrink: 0;
    width: 42px;
    height: 42px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(203, 166, 247, 0.15);
    color: #cba6f7;
    border: 1px solid rgba(203, 166, 247, 0.25);
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.15s, box-shadow 0.15s, transform 0.1s;
  }
  .btn-generate:hover:not(:disabled) {
    background: rgba(203, 166, 247, 0.28);
    box-shadow: 0 0 14px rgba(203, 166, 247, 0.15);
    transform: translateY(-1px);
  }
  .btn-generate:active:not(:disabled) {
    transform: translateY(0);
  }
  .btn-generate:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .spin {
    animation: spin 0.8s linear infinite;
  }
</style>
