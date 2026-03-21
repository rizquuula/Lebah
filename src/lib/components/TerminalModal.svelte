<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { sendInput, getOutputBuffer } from "../stores/tasks";
  import type { Task } from "../types";
  import { STATUS_COLORS } from "../types";

  export let task: Task;
  export let onClose: () => void;

  interface UsageInfo {
    input_tokens: number;
    output_tokens: number;
    cache_read_input_tokens: number;
    cache_creation_input_tokens: number;
  }

  type ChatEntry =
    | { kind: "user"; text: string }
    | { kind: "assistant"; text: string }
    | { kind: "tool_use"; name: string; input: string }
    | { kind: "usage"; input: number; output: number; cacheRead: number; cacheCreate: number }
    | { kind: "file_output"; path: string; content: string }
    | { kind: "result"; success: boolean; cost: number; duration_ms: number; usage: UsageInfo }
    | { kind: "system"; text: string };

  let entries: ChatEntry[] = [{ kind: "user", text: task.description }];
  let unlisten: UnlistenFn | null = null;
  let chatEl: HTMLDivElement;
  let inputValue = "";
  let inputEl: HTMLInputElement;

  $: borderColor = STATUS_COLORS[task.status];

  function parseJsonLine(raw: string): void {
    if (!raw.trim()) return;
    try {
      const obj = JSON.parse(raw);

      if (obj.type === "system" && obj.subtype === "init") {
        entries = [...entries, { kind: "system", text: `Session started · ${obj.model ?? ""}` }];
        return;
      }

      if (obj.type === "assistant") {
        // Full message (--print mode): message.content array
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
              entries = [...entries, { kind: "tool_use", name: part.name ?? "unknown", input: "" }];
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
        // Streaming deltas
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
          entries = [...entries, { kind: "tool_use", name: cb.name ?? "unknown", input: "" }];
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

  function scrollToBottom() {
    setTimeout(() => { if (chatEl) chatEl.scrollTop = chatEl.scrollHeight; }, 0);
  }

  onMount(async () => {
    unlisten = await listen<string>(`claude-output-${task.id}`, (event) => {
      parseJsonLine(event.payload);
      scrollToBottom();
    });
    try {
      const buffered = await getOutputBuffer(task.id);
      if (buffered.length > 0) {
        entries = [{ kind: "user", text: task.description }];
        for (const raw of buffered) parseJsonLine(raw);
        scrollToBottom();
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
    entries = [...entries, { kind: "user", text }];
    scrollToBottom();
    try {
      await sendInput(task.id, text);
    } catch (err) {
      entries = [...entries, { kind: "system", text: `Send failed: ${err}` }];
      scrollToBottom();
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

  function fmt(n: number, decimals = 0): string {
    return n.toLocaleString(undefined, { maximumFractionDigits: decimals });
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

    <div class="chat" bind:this={chatEl}>
      {#if entries.length <= 1}
        <div class="placeholder">
          <span class="cursor-blink">_</span> Waiting for output...
        </div>
      {:else}
        {#each entries as entry}
          {#if entry.kind === "user"}
            <div class="bubble user-bubble">{entry.text}</div>
          {:else if entry.kind === "assistant"}
            <div class="bubble assistant-bubble">{entry.text}</div>
          {:else if entry.kind === "tool_use"}
            <div class="tool-badge">
              <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
              </svg>
              {entry.name}
              {#if entry.input}
                <span class="tool-input">{entry.input}</span>
              {/if}
            </div>
          {:else if entry.kind === "usage"}
            <div class="usage-line">
              {fmt(entry.output)} out · {fmt(entry.input + entry.cacheRead + entry.cacheCreate)} in{#if entry.cacheRead > 0} · {fmt(entry.cacheRead)} cached{/if}
            </div>
          {:else if entry.kind === "file_output"}
            <div class="file-block">
              <div class="file-header">{entry.path.split("/").pop()}</div>
              <pre class="file-content">{entry.content}</pre>
            </div>
          {:else if entry.kind === "result"}
            <div class="result-bar" class:result-error={!entry.success}>
              <span class="result-icon">{entry.success ? "✓" : "✗"}</span>
              <span class="result-status">{entry.success ? "Completed" : "Error"}</span>
              <span class="result-sep">·</span>
              <span class="result-stat">${entry.cost.toFixed(4)}</span>
              <span class="result-sep">·</span>
              <span class="result-stat">{fmt(entry.usage.output_tokens)} out / {fmt(entry.usage.input_tokens + entry.usage.cache_read_input_tokens + entry.usage.cache_creation_input_tokens)} in</span>
              {#if entry.usage.cache_read_input_tokens > 0}
                <span class="result-cache">{fmt(entry.usage.cache_read_input_tokens)} cached</span>
              {/if}
              <span class="result-sep">·</span>
              <span class="result-stat">{(entry.duration_ms / 1000).toFixed(1)}s</span>
            </div>
          {:else if entry.kind === "system"}
            <div class="system-line">{entry.text}</div>
          {/if}
        {/each}
      {/if}
    </div>

    <div class="input-bar">
      <span class="prompt">›</span>
      <input
        bind:this={inputEl}
        bind:value={inputValue}
        on:keydown={handleKeydown}
        placeholder="Send input to Claude... (Enter to send)"
        class="stdin-input"
        autocomplete="off"
        spellcheck="false"
      />
      <button class="btn-send" on:click={handleSend} title="Send (Enter)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="2" x2="11" y2="13"/>
          <polygon points="22 2 15 22 11 13 2 9 22 2"/>
        </svg>
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: rgba(18, 18, 20, 0.92);
    backdrop-filter: blur(24px) saturate(1.5);
    -webkit-backdrop-filter: blur(24px) saturate(1.5);
    border: 1px solid rgba(137, 180, 250, 0.1);
    border-radius: 14px;
    width: 860px;
    max-width: 92vw;
    height: 80vh;
    max-height: 700px;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
    box-shadow: 0 32px 64px rgba(0, 0, 0, 0.5), 0 0 60px rgba(137, 180, 250, 0.04);
  }
  .modal-glow {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--border-color, rgba(137, 180, 250, 0.5));
    opacity: 0.7;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    gap: 12px;
    flex-shrink: 0;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 6px currentColor;
  }
  .task-desc {
    color: rgba(205, 214, 244, 0.85);
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .header-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }
  .status-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.8px;
  }
  .btn-close {
    background: rgba(82, 82, 91, 0.4);
    color: rgba(205, 214, 244, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 7px;
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .btn-close:hover {
    background: rgba(243, 139, 168, 0.2);
    border-color: rgba(243, 139, 168, 0.3);
    color: #f38ba8;
  }

  /* Chat area */
  .chat {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .chat::-webkit-scrollbar { width: 6px; }
  .chat::-webkit-scrollbar-track { background: transparent; }
  .chat::-webkit-scrollbar-thumb {
    background: rgba(137, 180, 250, 0.15);
    border-radius: 3px;
  }
  .placeholder {
    color: rgba(108, 112, 134, 0.5);
    font-style: italic;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12.5px;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cursor-blink { color: #89b4fa; }

  .bubble {
    max-width: 80%;
    padding: 9px 13px;
    border-radius: 10px;
    font-size: 13px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .user-bubble {
    align-self: flex-end;
    background: rgba(137, 180, 250, 0.12);
    border: 1px solid rgba(137, 180, 250, 0.2);
    color: rgba(205, 214, 244, 0.9);
  }
  .assistant-bubble {
    align-self: flex-start;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.07);
    color: rgba(205, 214, 244, 0.88);
  }

  .tool-badge {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    background: rgba(166, 227, 161, 0.08);
    border: 1px solid rgba(166, 227, 161, 0.18);
    border-radius: 20px;
    color: rgba(166, 227, 161, 0.7);
    font-size: 11px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
  }
  .tool-input {
    color: rgba(166, 227, 161, 0.45);
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-bar {
    align-self: stretch;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    background: rgba(166, 227, 161, 0.06);
    border: 1px solid rgba(166, 227, 161, 0.15);
    border-radius: 8px;
    font-size: 11.5px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    flex-wrap: wrap;
  }
  .result-bar.result-error {
    background: rgba(243, 139, 168, 0.06);
    border-color: rgba(243, 139, 168, 0.2);
  }
  .result-icon { font-size: 12px; }
  .result-bar:not(.result-error) .result-icon { color: #a6e3a1; }
  .result-bar.result-error .result-icon { color: #f38ba8; }
  .result-status {
    font-weight: 600;
    color: rgba(205, 214, 244, 0.8);
  }
  .result-sep { color: rgba(108, 112, 134, 0.5); }
  .result-stat { color: rgba(205, 214, 244, 0.6); }
  .result-cache {
    color: rgba(137, 180, 250, 0.55);
    font-size: 10.5px;
    padding: 1px 6px;
    background: rgba(137, 180, 250, 0.08);
    border-radius: 10px;
  }

  .usage-line {
    align-self: flex-start;
    font-size: 10px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    color: rgba(108, 112, 134, 0.5);
    padding: 1px 8px;
    margin-top: -6px;
  }
  .file-block {
    align-self: stretch;
    flex-shrink: 0;
    border: 1px solid rgba(137, 180, 250, 0.12);
    border-radius: 8px;
    overflow: hidden;
  }
  .file-header {
    padding: 5px 10px;
    font-size: 10.5px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    color: rgba(137, 180, 250, 0.6);
    background: rgba(137, 180, 250, 0.06);
    border-bottom: 1px solid rgba(137, 180, 250, 0.08);
  }
  .file-content {
    padding: 10px 12px;
    font-size: 11.5px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    color: rgba(205, 214, 244, 0.75);
    background: rgba(0, 0, 0, 0.2);
    margin: 0;
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.5;
  }
  .file-content::-webkit-scrollbar { width: 4px; }
  .file-content::-webkit-scrollbar-thumb {
    background: rgba(137, 180, 250, 0.15);
    border-radius: 2px;
  }

  .system-line {
    align-self: center;
    font-size: 10.5px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    color: rgba(108, 112, 134, 0.55);
    padding: 2px 8px;
    background: rgba(108, 112, 134, 0.06);
    border-radius: 4px;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Input bar */
  .input-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(9, 9, 11, 0.4);
    flex-shrink: 0;
  }
  .prompt {
    color: #89b4fa;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 14px;
    font-weight: 700;
    flex-shrink: 0;
    line-height: 1;
  }
  .stdin-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: rgba(205, 214, 244, 0.9);
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12.5px;
    caret-color: #89b4fa;
  }
  .stdin-input::placeholder {
    color: rgba(108, 112, 134, 0.4);
    font-family: inherit;
  }
  .btn-send {
    background: rgba(137, 180, 250, 0.15);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.2);
    border-radius: 7px;
    width: 30px;
    height: 30px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .btn-send:hover {
    background: rgba(137, 180, 250, 0.28);
    box-shadow: 0 0 10px rgba(137, 180, 250, 0.15);
  }
</style>
