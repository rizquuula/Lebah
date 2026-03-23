<script lang="ts">
  import type { ChatEntry } from "../types";

  export let entries: ChatEntry[];

  let chatEl: HTMLDivElement;

  function fmt(n: number, decimals = 0): string {
    return n.toLocaleString(undefined, { maximumFractionDigits: decimals });
  }

  function formatToolInput(name: string, rawJson: string): string {
    if (!rawJson) return "";
    try {
      const p = JSON.parse(rawJson);
      switch (name) {
        case "Read": return p.file_path ?? "";
        case "Write": return p.file_path ?? "";
        case "Edit": {
          const file = p.file_path ?? "";
          const old = p.old_string ? p.old_string.slice(0, 60).replace(/\n/g, "↵") : "";
          return old ? `${file} · ${old}` : file;
        }
        case "Glob": return p.path ? `${p.pattern} in ${p.path}` : p.pattern ?? "";
        case "Grep": {
          let s = p.pattern ?? "";
          if (p.path) s += ` in ${p.path}`;
          if (p.glob) s += ` (${p.glob})`;
          return s;
        }
        case "Bash": return p.command?.slice(0, 120) ?? "";
        case "Agent": return p.description ?? p.prompt?.slice(0, 80) ?? "";
        case "WebFetch": return p.url ?? "";
        case "WebSearch": return p.query ?? "";
        default: {
          const vals = Object.values(p).filter((v): v is string => typeof v === "string" && v.length < 200);
          return vals[0] ?? "";
        }
      }
    } catch {
      return rawJson.slice(0, 100);
    }
  }

  function scrollToBottom() {
    setTimeout(() => { if (chatEl) chatEl.scrollTop = chatEl.scrollHeight; }, 0);
  }

  // Auto-scroll whenever entries change
  $: entries, scrollToBottom();
</script>

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
          {#if formatToolInput(entry.name, entry.input)}
            <span class="tool-input">{formatToolInput(entry.name, entry.input)}</span>
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

<style>
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
    max-width: 500px;
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
  .result-status { font-weight: 600; color: rgba(205, 214, 244, 0.8); }
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
</style>
