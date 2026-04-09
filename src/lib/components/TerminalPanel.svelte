<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import 'xterm/css/xterm.css';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { portal } from '../actions/portal';
  import { projectPath } from '../stores/project';

  interface SessionInfo {
    id: string;
    name: string;
    project: string;
  }

  interface TerminalInstance {
    term: Terminal;
    fitAddon: FitAddon;
    unlisten: UnlistenFn | null;
  }

  interface ProjectTerminalState {
    sessions: SessionInfo[];
    activeSessionId: string | null;
    initialized: boolean;
  }

  let { onClose, visible = false }: { onClose: () => void; visible?: boolean } = $props();

  let containerEl: HTMLDivElement;
  let currentProject: string | null = $state(null);
  let resizeObserver: ResizeObserver | null = null;

  // Per-project state
  let projectStates: Map<string, ProjectTerminalState> = $state(new Map());
  // All terminal instances (keyed by session id, shared across projects)
  let terminals: Map<string, TerminalInstance> = new Map();

  // Reactive getters for current project
  let sessions: SessionInfo[] = $derived(
    currentProject ? (projectStates.get(currentProject)?.sessions ?? []) : []
  );
  let activeSessionId: string | null = $derived(
    currentProject ? (projectStates.get(currentProject)?.activeSessionId ?? null) : null
  );

  const termConfig = {
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
  };

  function getProjectState(project: string): ProjectTerminalState {
    let state = projectStates.get(project);
    if (!state) {
      state = { sessions: [], activeSessionId: null, initialized: false };
      projectStates.set(project, state);
    }
    return state;
  }

  function setActiveSessionId(sessionId: string | null) {
    if (!currentProject) return;
    const state = getProjectState(currentProject);
    state.activeSessionId = sessionId;
    // Trigger reactivity
    projectStates = new Map(projectStates);
  }

  function addSession(info: SessionInfo) {
    if (!currentProject) return;
    const state = getProjectState(currentProject);
    state.sessions = [...state.sessions, info];
    projectStates = new Map(projectStates);
  }

  function removeSession(sessionId: string) {
    if (!currentProject) return;
    const state = getProjectState(currentProject);
    state.sessions = state.sessions.filter(s => s.id !== sessionId);
    projectStates = new Map(projectStates);
  }

  async function initProjectIfNeeded() {
    if (!currentProject) return;
    const state = getProjectState(currentProject);
    if (state.initialized) return;
    state.initialized = true;

    try {
      const existing: SessionInfo[] = await invoke('list_terminal_sessions');
      state.sessions = existing;
      projectStates = new Map(projectStates);
      if (existing.length > 0) {
        await switchSession(existing[0].id);
      } else {
        await createSession();
      }
    } catch {
      await createSession();
    }
  }

  onMount(() => {
    resizeObserver = new ResizeObserver(() => {
      if (activeSessionId) {
        const inst = terminals.get(activeSessionId);
        if (inst) {
          inst.fitAddon.fit();
          invoke('resize_terminal', {
            sessionId: activeSessionId,
            cols: inst.term.cols,
            rows: inst.term.rows,
          }).catch(() => {});
        }
      }
    });
    resizeObserver.observe(containerEl);
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    for (const [, inst] of terminals) {
      inst.unlisten?.();
      inst.term.dispose();
    }
    terminals.clear();
  });

  async function createSession() {
    detachActive();

    const measureTerm = new Terminal(termConfig);
    const measureFit = new FitAddon();
    measureTerm.loadAddon(measureFit);
    measureTerm.open(containerEl);
    await new Promise(r => setTimeout(r, 50));
    measureFit.fit();
    const cols = measureTerm.cols;
    const rows = measureTerm.rows;
    measureTerm.dispose();

    try {
      const info: SessionInfo = await invoke('create_terminal_session', { cols, rows });
      addSession(info);

      const term = new Terminal(termConfig);
      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);
      term.open(containerEl);
      await new Promise(r => setTimeout(r, 50));
      fitAddon.fit();

      const unlisten = await listen<string>(`terminal-output-${info.id}`, (event) => {
        term.write(event.payload);
      });

      term.onData((data: string) => {
        invoke('write_terminal', { sessionId: info.id, data }).catch(() => {});
      });

      terminals.set(info.id, { term, fitAddon, unlisten });
      setActiveSessionId(info.id);
    } catch (e: any) {
      const errTerm = new Terminal(termConfig);
      errTerm.open(containerEl);
      errTerm.writeln(`\r\n\x1b[31mFailed to start terminal: ${e}\x1b[0m`);
    }
  }

  function detachActive() {
    if (activeSessionId) {
      const el = containerEl?.querySelector('.xterm');
      if (el) el.remove();
    }
  }

  async function switchSession(sessionId: string) {
    if (sessionId === activeSessionId) return;
    detachActive();

    const inst = terminals.get(sessionId);
    if (inst) {
      inst.term.open(containerEl);
      await new Promise(r => setTimeout(r, 50));
      inst.fitAddon.fit();
      setActiveSessionId(sessionId);
      invoke('resize_terminal', {
        sessionId,
        cols: inst.term.cols,
        rows: inst.term.rows,
      }).catch(() => {});
    } else {
      const term = new Terminal(termConfig);
      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);
      term.open(containerEl);
      await new Promise(r => setTimeout(r, 50));
      fitAddon.fit();

      const unlisten = await listen<string>(`terminal-output-${sessionId}`, (event) => {
        term.write(event.payload);
      });

      term.onData((data: string) => {
        invoke('write_terminal', { sessionId, data }).catch(() => {});
      });

      terminals.set(sessionId, { term, fitAddon, unlisten });
      setActiveSessionId(sessionId);
    }
  }

  async function closeSession(sessionId: string) {
    const inst = terminals.get(sessionId);
    if (inst) {
      inst.unlisten?.();
      if (sessionId === activeSessionId) {
        const el = containerEl?.querySelector('.xterm');
        if (el) el.remove();
      }
      inst.term.dispose();
      terminals.delete(sessionId);
    }

    await invoke('close_terminal_session', { sessionId }).catch(() => {});
    removeSession(sessionId);

    if (sessionId === activeSessionId) {
      setActiveSessionId(null);
      if (sessions.length > 0) {
        await switchSession(sessions[0].id);
      }
    }
  }

  // Track project changes
  $effect(() => {
    const project = $projectPath;
    if (project && project !== currentProject) {
      // Detach current terminal before switching
      if (currentProject && activeSessionId) {
        const el = containerEl?.querySelector('.xterm');
        if (el) el.remove();
      }
      currentProject = project;
    }
  });

  // Handle visibility and init
  $effect(() => {
    if (visible && currentProject) {
      const state = getProjectState(currentProject);
      if (!state.initialized) {
        initProjectIfNeeded();
      } else if (activeSessionId) {
        const inst = terminals.get(activeSessionId);
        if (inst) {
          setTimeout(() => {
            inst.fitAddon.fit();
            inst.term.focus();
          }, 50);
        }
      } else if (state.sessions.length > 0) {
        switchSession(state.sessions[0].id);
      }
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="terminal-overlay" class:hidden={!visible} use:portal onkeydown={handleKeydown}>
  <div class="terminal-modal">
    <div class="terminal-header">
      <div class="terminal-tabs">
        {#each sessions as session (session.id)}
          <button
            class="terminal-tab"
            class:active={session.id === activeSessionId}
            onclick={() => switchSession(session.id)}
          >
            <span class="tab-name">{session.name}</span>
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span
              class="tab-close"
              role="button"
              tabindex="-1"
              onclick={(e: MouseEvent) => { e.stopPropagation(); closeSession(session.id); }}
              onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') { e.stopPropagation(); closeSession(session.id); } }}
              title="Close session"
            >×</span>
          </button>
        {/each}
        <button class="terminal-tab add-tab" onclick={() => createSession()} title="New session">
          +
        </button>
      </div>
      <button class="terminal-close" onclick={onClose} title="Close panel">
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

  .terminal-overlay.hidden {
    display: none;
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
    padding: 0 8px 0 0;
    background: rgba(255, 255, 255, 0.05);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    user-select: none;
  }

  .terminal-tabs {
    display: flex;
    align-items: center;
    gap: 0;
    overflow-x: auto;
    flex: 1;
    min-width: 0;
  }

  .terminal-tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    font-size: 12px;
    white-space: nowrap;
    border-bottom: 2px solid transparent;
    transition: all 0.15s;
  }

  .terminal-tab:hover {
    color: rgba(255, 255, 255, 0.8);
    background: rgba(255, 255, 255, 0.05);
  }

  .terminal-tab.active {
    color: rgba(255, 255, 255, 0.9);
    border-bottom-color: #f0c674;
  }

  .terminal-tab.add-tab {
    font-size: 16px;
    padding: 6px 10px;
    font-weight: bold;
  }

  .tab-name {
    pointer-events: none;
  }

  .tab-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.3);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    line-height: 1;
    border-radius: 3px;
  }

  .tab-close:hover {
    color: white;
    background: rgba(255, 255, 255, 0.15);
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
    flex-shrink: 0;
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
