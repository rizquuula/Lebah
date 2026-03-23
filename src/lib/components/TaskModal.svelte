<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { createTask, updateTask } from "../stores/tasks";
  import type { Task, ChatEntry, UsageInfo } from "../types";
  import TerminalChat from "./TerminalChat.svelte";

  export let task: Task | null = null;
  export let onClose: () => void;
  export let columnLabel: string = "";
  export let columnColor: string = "#89b4fa";

  let description = task?.description ?? "";
  let claudePath = task?.claude_path ?? "";
  let worktree = task?.worktree ?? "";
  let worktreeError = "";
  let model = task?.model ?? "sonnet";

  let generating = false;
  let genEntries: ChatEntry[] = [];

  function parseGenLine(raw: string): void {
    if (!raw.trim()) return;
    try {
      const obj = JSON.parse(raw);

      if (obj.type === "system" && obj.subtype === "init") {
        genEntries = [...genEntries, { kind: "system", text: `Generating · ${obj.model ?? model}` }];
        return;
      }

      if (obj.type === "assistant") {
        if (obj.message?.content) {
          for (const part of obj.message.content) {
            if (part.type === "text" && part.text) {
              const last = genEntries[genEntries.length - 1];
              if (last?.kind === "assistant") {
                genEntries[genEntries.length - 1] = { kind: "assistant", text: last.text + part.text };
                genEntries = genEntries;
              } else {
                genEntries = [...genEntries, { kind: "assistant", text: part.text }];
              }
            }
          }
          const u = obj.message.usage;
          if (u && u.output_tokens > 0) {
            genEntries = [...genEntries, {
              kind: "usage",
              input: u.input_tokens ?? 0,
              output: u.output_tokens ?? 0,
              cacheRead: u.cache_read_input_tokens ?? 0,
              cacheCreate: u.cache_creation_input_tokens ?? 0,
            }];
          }
          return;
        }
        const delta = obj.content_block?.delta;
        if (delta?.type === "text_delta" && delta.text) {
          const last = genEntries[genEntries.length - 1];
          if (last?.kind === "assistant") {
            genEntries[genEntries.length - 1] = { kind: "assistant", text: last.text + delta.text };
            genEntries = genEntries;
          } else {
            genEntries = [...genEntries, { kind: "assistant", text: delta.text }];
          }
          return;
        }
        return;
      }

      if (obj.type === "result") {
        const usage: UsageInfo = {
          input_tokens: obj.usage?.input_tokens ?? 0,
          output_tokens: obj.usage?.output_tokens ?? 0,
          cache_read_input_tokens: obj.usage?.cache_read_input_tokens ?? 0,
          cache_creation_input_tokens: obj.usage?.cache_creation_input_tokens ?? 0,
        };
        genEntries = [...genEntries, {
          kind: "result",
          success: !obj.is_error,
          cost: obj.total_cost_usd ?? 0,
          duration_ms: obj.duration_ms ?? 0,
          usage,
        }];
        return;
      }
    } catch {
      if (raw.trim()) {
        genEntries = [...genEntries, { kind: "system", text: raw }];
      }
    }
  }

  async function generateName() {
    if (!description.trim() || generating) return;
    generating = true;
    genEntries = [];

    const unlisten = await listen<string>("worktree-gen-line", (event) => {
      parseGenLine(event.payload);
    });

    try {
      const name = await invoke<string>("generate_worktree_name", {
        description: description.trim(),
        model: model || null,
        claudePath: claudePath.trim() || null,
      });
      worktree = name;
      worktreeError = "";
    } catch (e) {
      genEntries = [...genEntries, { kind: "system", text: `Error: ${e}` }];
    } finally {
      generating = false;
      unlisten();
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

      <div class="worktree-label-row">
        <label class="field-label" for="task-worktree">Worktree Name</label>
        {#if !task}
          <button
            type="button"
            class="btn-generate"
            on:click={generateName}
            disabled={generating || !description.trim()}
            title="Generate name from description using AI"
          >
            {#if generating}
              <span class="gen-spinner"></span>
            {:else}
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
              </svg>
            {/if}
            {generating ? "Generating…" : "Generate"}
          </button>
        {/if}
      </div>
      {#if task}
        <div class="text-input readonly-field">{task.worktree ?? "—"}</div>
      {:else}
        <input
          id="task-worktree"
          type="text"
          bind:value={worktree}
          placeholder="feat-my-feature"
          class="text-input"
          class:input-error={!!worktreeError}
        />
        {#if worktreeError}
          <div class="field-error">{worktreeError}</div>
        {/if}
        {#if genEntries.length > 0}
          <div class="gen-log">
            <TerminalChat entries={genEntries} />
          </div>
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
  .worktree-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
    margin-top: 14px;
  }
  .worktree-label-row .field-label {
    margin: 0;
  }
  .btn-generate {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    background: rgba(203, 166, 247, 0.1);
    color: rgba(203, 166, 247, 0.7);
    border: 1px solid rgba(203, 166, 247, 0.2);
    border-radius: 20px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }
  .btn-generate:hover:not(:disabled) {
    background: rgba(203, 166, 247, 0.18);
    color: rgba(203, 166, 247, 0.95);
  }
  .btn-generate:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .gen-spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid rgba(203, 166, 247, 0.3);
    border-top-color: rgba(203, 166, 247, 0.8);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .gen-log {
    margin-top: 8px;
    border: 1px solid rgba(137, 180, 250, 0.1);
    border-radius: 8px;
    overflow: hidden;
    max-height: 180px;
    background: rgba(0, 0, 0, 0.15);
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
</style>
