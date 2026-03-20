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
    <div class="placeholder">
      <span class="cursor-blink">_</span> Waiting for output...
    </div>
  {:else}
    {#each lines as line}
      <div class="line">{line}</div>
    {/each}
  {/if}
</div>

<style>
  .terminal {
    background: rgba(10, 10, 26, 0.7);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    color: rgba(224, 224, 224, 0.9);
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 12px;
    padding: 10px;
    border-radius: 8px;
    border: 1px solid rgba(137, 180, 250, 0.06);
    max-height: 200px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .placeholder {
    color: rgba(108, 112, 134, 0.5);
    font-style: italic;
    display: flex;
    align-items: center;
    gap: 4px;
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
    line-height: 1.5;
    padding: 0 2px;
    border-radius: 2px;
    animation: lineAppear 0.15s ease-out;
  }
  @keyframes lineAppear {
    from { opacity: 0; transform: translateX(-4px); }
    to { opacity: 1; transform: translateX(0); }
  }
  .line:hover {
    background: rgba(137, 180, 250, 0.04);
  }
</style>
