<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import 'xterm/css/xterm.css';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { portal } from '../actions/portal';

  let { onClose }: { onClose: () => void } = $props();

  let containerEl: HTMLDivElement;
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let unlisten: UnlistenFn | null = null;
  let resizeObserver: ResizeObserver | null = null;
  let spawned = false;

  onMount(async () => {
    term = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', Menlo, Monaco, monospace",
      theme: {
        background: '#1a1a2e',
        foreground: '#e0e0e0',
        cursor: '#f0c674',
        selectionBackground: '#3a3a5e',
      },
      convertEol: true,
      scrollback: 5000,
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(containerEl);

    await new Promise(r => setTimeout(r, 50));
    fitAddon.fit();

    const cols = term.cols;
    const rows = term.rows;

    try {
      await invoke('spawn_terminal', { cols, rows });
      spawned = true;
    } catch (e: any) {
      term.writeln(`\r\n\x1b[31mFailed to start terminal: ${e}\x1b[0m`);
      return;
    }

    unlisten = await listen<string>('terminal-output', (event) => {
      term?.write(event.payload);
    });

    term.onData((data: string) => {
      invoke('write_terminal', { data }).catch(() => {});
    });

    resizeObserver = new ResizeObserver(() => {
      if (fitAddon && term) {
        fitAddon.fit();
        invoke('resize_terminal', { cols: term.cols, rows: term.rows }).catch(() => {});
      }
    });
    resizeObserver.observe(containerEl);
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    unlisten?.();
    term?.dispose();
    if (spawned) {
      invoke('close_terminal').catch(() => {});
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="terminal-overlay" use:portal onkeydown={handleKeydown}>
  <div class="terminal-modal">
    <div class="terminal-header">
      <span class="terminal-title">Terminal</span>
      <button class="terminal-close" onclick={onClose} title="Close terminal">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>
    <div class="terminal-container" bind:this={containerEl}></div>
  </div>
</div>

<style>
  .terminal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .terminal-modal {
    width: 85vw;
    height: 70vh;
    background: #1a1a2e;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .terminal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    user-select: none;
  }

  .terminal-title {
    font-size: 12px;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.7);
    letter-spacing: 0.5px;
  }

  .terminal-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
  }

  .terminal-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .terminal-container {
    flex: 1;
    padding: 4px;
    overflow: hidden;
  }

  .terminal-container :global(.xterm) {
    height: 100%;
  }

  .terminal-container :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>
