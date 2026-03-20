<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { sendInput, getOutputBuffer } from "../stores/tasks";
  import type { Task } from "../types";
  import { STATUS_COLORS } from "../types";

  export let task: Task;
  export let onClose: () => void;

  let lines: string[] = [];
  let unlisten: UnlistenFn | null = null;
  let terminalEl: HTMLDivElement;
  let inputValue = "";
  let inputEl: HTMLInputElement;

  $: borderColor = STATUS_COLORS[task.status];

  onMount(async () => {
    // Register live listener first to avoid missing events during buffer load
    unlisten = await listen<string>(`claude-output-${task.id}`, (event) => {
      lines = [...lines, event.payload];
      if (terminalEl) {
        setTimeout(() => { terminalEl.scrollTop = terminalEl.scrollHeight; }, 0);
      }
    });
    // Load past output so late-opening modals see prior lines.
    // Use buffer as source of truth; new live events will append after this.
    try {
      const buffered = await getOutputBuffer(task.id);
      if (buffered.length > 0) {
        lines = buffered;
        setTimeout(() => { if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight; }, 0);
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
    await sendInput(task.id, text);
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

    <div class="terminal" bind:this={terminalEl}>
      {#if lines.length === 0}
        <div class="placeholder">
          <span class="cursor-blink">_</span> Waiting for output...
        </div>
      {:else}
        {#each lines as line}
          <div class="line">{line}</div>
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
    animation: overlayFade 0.2s ease-out;
  }
  @keyframes overlayFade {
    from { opacity: 0; }
    to { opacity: 1; }
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
    animation: modalPop 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  @keyframes modalPop {
    from { opacity: 0; transform: scale(0.94) translateY(12px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
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
    transition: all 0.2s ease;
  }
  .btn-close:hover {
    background: rgba(243, 139, 168, 0.2);
    border-color: rgba(243, 139, 168, 0.3);
    color: #f38ba8;
  }
  .terminal {
    flex: 1;
    overflow-y: auto;
    padding: 14px 16px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12.5px;
    color: rgba(224, 224, 224, 0.9);
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.6;
  }
  .terminal::-webkit-scrollbar {
    width: 6px;
  }
  .terminal::-webkit-scrollbar-track {
    background: transparent;
  }
  .terminal::-webkit-scrollbar-thumb {
    background: rgba(137, 180, 250, 0.15);
    border-radius: 3px;
  }
  .placeholder {
    color: rgba(108, 112, 134, 0.5);
    font-style: italic;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .cursor-blink {
    animation: blink 1s step-end infinite;
    color: #89b4fa;
  }
  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }
  .line {
    padding: 1px 2px;
    border-radius: 2px;
    animation: lineAppear 0.12s ease-out;
  }
  @keyframes lineAppear {
    from { opacity: 0; transform: translateX(-3px); }
    to { opacity: 1; transform: translateX(0); }
  }
  .line:hover {
    background: rgba(137, 180, 250, 0.04);
  }
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
    transition: all 0.2s ease;
    flex-shrink: 0;
  }
  .btn-send:hover {
    background: rgba(137, 180, 250, 0.28);
    box-shadow: 0 0 10px rgba(137, 180, 250, 0.15);
  }
</style>
