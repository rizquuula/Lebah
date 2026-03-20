<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  export let taskId: string;

  let lines: string[] = [];
  let unlisten: UnlistenFn | null = null;
  let terminalEl: HTMLDivElement;

  onMount(async () => {
    unlisten = await listen<string>(`claude-output-${taskId}`, (event) => {
      lines = [...lines, event.payload];
      if (terminalEl) {
        terminalEl.scrollTop = terminalEl.scrollHeight;
      }
    });
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });
</script>

<div class="terminal" bind:this={terminalEl}>
  {#if lines.length === 0}
    <span class="placeholder">No output yet...</span>
  {:else}
    {#each lines as line}
      <div class="line">{line}</div>
    {/each}
  {/if}
</div>

<style>
  .terminal {
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12px;
    padding: 8px;
    border-radius: 0 0 6px 6px;
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .placeholder {
    color: #666;
    font-style: italic;
  }
  .line {
    line-height: 1.4;
  }
</style>
