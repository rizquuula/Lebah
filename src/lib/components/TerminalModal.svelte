<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { sendInputWithListener, getOutputBuffer } from "../stores/tasks";
  import type { Task, ChatEntry, UsageInfo } from "../types";
  import { STATUS_COLORS, AGENT_MODELS } from "../types";
  import TerminalChat from "./TerminalChat.svelte";

  export let task: Task;
  export let onClose: () => void;
  export let readonly: boolean = false;

  let selectedModel = task.model ?? "sonnet";
  $: agentModelConfig = AGENT_MODELS[task.agent_name ?? "claude"] ?? AGENT_MODELS.claude;
  let entries: ChatEntry[] = [];
  let unlisten: UnlistenFn | null = null;
  let inputValue = "";
  let inputEl: HTMLInputElement;

  $: borderColor = STATUS_COLORS[task.status];

  function parseJsonLine(raw: string): void {
    if (!raw.trim()) return;
    try {
      const obj = JSON.parse(raw);

      if (obj.type === "user_input") {
        entries = [...entries, { kind: "user", text: obj.text ?? "" }];
        return;
      }

      if (obj.type === "system" && obj.subtype === "init") {
        entries = [...entries, { kind: "system", text: `Session started · ${obj.model ?? ""}` }];
        return;
      }

      if (obj.type === "assistant") {
        if (obj.message?.content) {
          for (const part of obj.message.content) {
            if (part.type === "text" && part.text) {
              const last = entries[entries.length - 1];
              if (last?.kind === "assistant") {
                entries[entries.length - 1] = { kind: "assistant", text: last.text + part.text };
                entries = entries;
              } else {
                entries = [...entries, { kind: "assistant", text: part.text }];
              }
            } else if (part.type === "tool_use") {
              entries = [...entries, { kind: "tool_use", name: part.name ?? "unknown", input: part.input ? JSON.stringify(part.input) : "" }];
            }
          }
          const u = obj.message.usage;
          if (u && u.output_tokens > 0) {
            entries = [...entries, {
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
          const last = entries[entries.length - 1];
          if (last?.kind === "assistant") {
            entries[entries.length - 1] = { kind: "assistant", text: last.text + delta.text };
            entries = entries;
          } else {
            entries = [...entries, { kind: "assistant", text: delta.text }];
          }
          return;
        }
        if (delta?.type === "input_json_delta" && delta.partial_json) {
          const last = entries[entries.length - 1];
          if (last?.kind === "tool_use") {
            entries[entries.length - 1] = { ...last, input: last.input + delta.partial_json };
            entries = entries;
          }
          return;
        }
        const cb = obj.content_block;
        if (cb?.type === "tool_use") {
          entries = [...entries, { kind: "tool_use", name: cb.name ?? "unknown", input: cb.input ? JSON.stringify(cb.input) : "" }];
          return;
        }
        if (cb?.type === "text" && cb.text) {
          entries = [...entries, { kind: "assistant", text: cb.text }];
          return;
        }
        return;
      }

      if (obj.type === "user") {
        const contents = obj.message?.content;
        if (Array.isArray(contents)) {
          for (const part of contents) {
            const tr = part.tool_use_result ?? obj.tool_use_result;
            if (tr?.type === "create" && tr.filePath?.endsWith(".md") && tr.content) {
              entries = [...entries, { kind: "file_output", path: tr.filePath, content: tr.content }];
            }
          }
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
        entries = [...entries, {
          kind: "result",
          success: !obj.is_error,
          cost: obj.total_cost_usd ?? 0,
          duration_ms: obj.duration_ms ?? 0,
          usage,
        }];
        return;
      }
    } catch {
      // Non-JSON line (stderr etc.) — show as system note
      if (raw.trim()) {
        entries = [...entries, { kind: "system", text: raw }];
      }
    }
  }

  onMount(async () => {
    unlisten = await listen<string>(`claude-output-${task.id}`, (event) => {
      parseJsonLine(event.payload);
    });
    try {
      const buffered = await getOutputBuffer(task.id);
      // Reset entries — the initial user_input comes from the backend output stream
      entries = [];
      if (buffered.length > 0) {
        // Parse all buffered lines to reconstruct the chat history
        for (const raw of buffered) {
          parseJsonLine(raw);
        }
        // Force reactivity after loading buffer
        entries = [...entries];
      }
    } catch (_) {}
    inputEl?.focus();
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  async function handleSend() {
    const text = inputValue.trim();
    if (!text) return;
    inputValue = "";
    try {
      await sendInputWithListener(task.id, text, null, null, selectedModel, task.yolo);
    } catch (err) {
      entries = [...entries, { kind: "system", text: `Send failed: ${err}` }];
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    } else if (e.key === "Escape") {
      onClose();
    }
  }

  function handleOverlayKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:click={onClose} on:keydown={handleOverlayKey}>
  <div class="modal" on:click|stopPropagation>
    <div class="modal-glow" style="--border-color: {borderColor}"></div>

    <div class="header">
      <div class="header-left">
        <div class="status-dot" style="background: {borderColor}"></div>
        <span class="task-desc">{task.description}</span>
      </div>
      <div class="header-right">
        <span class="status-label" style="color: {borderColor}">{task.status}</span>
        <button class="btn-close" on:click={onClose} title="Close (Esc)">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <TerminalChat {entries} />

    {#if !readonly}
    <div class="input-bar">
      <span class="prompt">›</span>
      <input
        bind:this={inputEl}
        bind:value={inputValue}
        on:keydown={handleKeydown}
        placeholder="Send input... (Enter to send)"
        class="stdin-input"
        autocomplete="off"
        spellcheck="false"
      />
      {#if agentModelConfig.type === 'select'}
        <select class="model-select" bind:value={selectedModel} title="Model">
          {#each agentModelConfig.options ?? [] as opt}
            <option value={opt}>{opt}</option>
          {/each}
        </select>
      {:else}
        <input class="model-input" bind:value={selectedModel} placeholder="provider/model" title="Model" />
      {/if}
      <button class="btn-send" on:click={handleSend} title="Send (Enter)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="2" x2="11" y2="13"/>
          <polygon points="22 2 15 22 11 13 2 9 22 2"/>
        </svg>
      </button>
    </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0; z-index: 100;
    background: rgba(0,0,0,0.7);
    backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
    display: flex; align-items: center; justify-content: center;
  }
  .modal {
    background: rgba(18,18,20,0.92);
    backdrop-filter: blur(24px) saturate(1.5); -webkit-backdrop-filter: blur(24px) saturate(1.5);
    border: 1px solid rgba(137,180,250,0.1); border-radius: 14px;
    width: 860px; max-width: 92vw; height: 80vh; max-height: 700px;
    display: flex; flex-direction: column; position: relative; overflow: hidden;
    box-shadow: 0 32px 64px rgba(0,0,0,0.5), 0 0 60px rgba(137,180,250,0.04);
  }
  .modal-glow {
    position: absolute; top: 0; left: 0; right: 0; height: 2px;
    background: var(--border-color, rgba(137,180,250,0.5)); opacity: 0.7;
  }
  .header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 14px 16px 12px; gap: 12px; flex-shrink: 0;
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }
  .header-left { display: flex; align-items: center; gap: 10px; min-width: 0; }
  .header-right { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
  .status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; box-shadow: 0 0 6px currentColor; }
  .task-desc {
    color: rgba(205,214,244,0.85); font-size: 13px; font-weight: 500;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .status-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.8px; }
  .btn-close {
    background: rgba(82,82,91,0.4); color: rgba(205,214,244,0.6);
    border: 1px solid rgba(255,255,255,0.06); border-radius: 7px;
    width: 28px; height: 28px; cursor: pointer;
    display: flex; align-items: center; justify-content: center;
  }
  .btn-close:hover { background: rgba(243,139,168,0.2); border-color: rgba(243,139,168,0.3); color: #f38ba8; }
  .input-bar {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 14px; flex-shrink: 0;
    border-top: 1px solid rgba(255,255,255,0.06);
    background: rgba(9,9,11,0.4);
  }
  .prompt {
    color: #89b4fa; font-family: "JetBrains Mono","Fira Code",monospace;
    font-size: 14px; font-weight: 700; flex-shrink: 0; line-height: 1;
  }
  .stdin-input {
    flex: 1; background: transparent; border: none; outline: none;
    color: rgba(205,214,244,0.9); font-family: "JetBrains Mono","Fira Code",monospace;
    font-size: 12.5px; caret-color: #89b4fa;
  }
  .stdin-input::placeholder { color: rgba(108,112,134,0.4); font-family: inherit; }
  .model-select {
    background: rgba(30,30,46,0.9); color: #cdd6f4; color-scheme: dark;
    border: 1px solid rgba(137,180,250,0.2); border-radius: 7px;
    height: 30px; padding: 0 6px; font-size: 0.75rem; font-family: inherit;
    cursor: pointer; outline: none;
  }
  .model-select:focus { border-color: rgba(137,180,250,0.5); }
  .model-input {
    background: rgba(30,30,46,0.9); color: #cdd6f4;
    border: 1px solid rgba(137,180,250,0.2); border-radius: 7px;
    height: 30px; padding: 0 6px; font-size: 0.75rem; font-family: inherit;
    outline: none; width: 140px;
  }
  .model-input:focus { border-color: rgba(137,180,250,0.5); }
  .btn-send {
    background: rgba(137,180,250,0.15); color: #89b4fa;
    border: 1px solid rgba(137,180,250,0.2); border-radius: 7px;
    width: 30px; height: 30px; cursor: pointer; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
  }
  .btn-send:hover { background: rgba(137,180,250,0.28); box-shadow: 0 0 10px rgba(137,180,250,0.15); }
</style>
