<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import Board from "./lib/components/Board.svelte";
  import { projectPath, gitStatus, openProject, loadProjectPath, refreshGitStatus } from "./lib/stores/project";

  let gitPollInterval: ReturnType<typeof setInterval>;

  onMount(async () => {
    await loadProjectPath();
    gitPollInterval = setInterval(() => {
      if ($projectPath) refreshGitStatus();
    }, 10000);
  });

  onDestroy(() => {
    clearInterval(gitPollInterval);
  });

  async function handleOpenProject() {
    const selected = await open({ directory: true, multiple: false });
    if (selected && typeof selected === "string") {
      await openProject(selected);
    }
  }
</script>

<main>
  <header>
    <div class="logo-group">
      <div class="logo-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
          <path d="M12 2L2 7l10 5 10-5-10-5z" fill="url(#g1)" opacity="0.9"/>
          <path d="M2 17l10 5 10-5" stroke="url(#g2)" stroke-width="1.5" fill="none" opacity="0.7"/>
          <path d="M2 12l10 5 10-5" stroke="url(#g2)" stroke-width="1.5" fill="none" opacity="0.85"/>
          <defs>
            <linearGradient id="g1" x1="2" y1="2" x2="22" y2="12">
              <stop offset="0%" stop-color="#89b4fa"/>
              <stop offset="100%" stop-color="#b4befe"/>
            </linearGradient>
            <linearGradient id="g2" x1="2" y1="12" x2="22" y2="22">
              <stop offset="0%" stop-color="#89b4fa"/>
              <stop offset="100%" stop-color="#cba6f7"/>
            </linearGradient>
          </defs>
        </svg>
      </div>
      <h1>LebahTempa</h1>
      <span class="subtitle">Claude Code Orchestrator</span>
    </div>

    <div class="project-bar">
      <button class="btn-open" on:click={handleOpenProject}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
        Open Project
      </button>

      {#if $projectPath}
        <span class="project-path" title={$projectPath}>{$projectPath}</span>
      {/if}

      {#if $gitStatus}
        <div class="git-info">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/>
            <path d="M6 21V9a9 9 0 009 9"/>
          </svg>
          <span class="branch-name">{$gitStatus.branch}</span>
          {#if $gitStatus.ahead > 0}
            <span class="git-badge ahead" title="{$gitStatus.ahead} ahead">↑{$gitStatus.ahead}</span>
          {/if}
          {#if $gitStatus.behind > 0}
            <span class="git-badge behind" title="{$gitStatus.behind} behind">↓{$gitStatus.behind}</span>
          {/if}
          {#if $gitStatus.changed_files > 0}
            <span class="git-badge changes" title="{$gitStatus.changed_files} changed files">●{$gitStatus.changed_files}</span>
          {/if}
        </div>
      {/if}
    </div>

    <div class="header-glow"></div>
  </header>
  <Board />
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }
  :global(body) {
    margin: 0;
    background: #09090b;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    color: #cdd6f4;
    overflow: hidden;
  }
  :global(body::before) {
    content: '';
    position: fixed;
    inset: 0;
    background:
      radial-gradient(ellipse 800px 600px at 20% 20%, rgba(113, 113, 122, 0.06) 0%, transparent 70%),
      radial-gradient(ellipse 600px 400px at 80% 80%, rgba(113, 113, 122, 0.04) 0%, transparent 70%);
    pointer-events: none;
    z-index: 0;
    animation: bgShift 20s ease-in-out infinite alternate;
  }
  @keyframes bgShift {
    0% { opacity: 1; }
    50% { opacity: 0.7; }
    100% { opacity: 1; }
  }
  :global(::-webkit-scrollbar) {
    width: 6px;
    height: 6px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: rgba(137, 180, 250, 0.2);
    border-radius: 3px;
  }
  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(137, 180, 250, 0.35);
  }
  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
    z-index: 1;
  }
  header {
    display: flex;
    align-items: center;
    padding: 14px 20px;
    gap: 16px;
    background: rgba(24, 24, 27, 0.6);
    backdrop-filter: blur(20px) saturate(1.5);
    -webkit-backdrop-filter: blur(20px) saturate(1.5);
    border-bottom: 1px solid rgba(137, 180, 250, 0.1);
    position: relative;
    overflow: hidden;
    animation: headerSlide 0.6s ease-out;
  }
  @keyframes headerSlide {
    from { transform: translateY(-100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  .header-glow {
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.4), rgba(203, 166, 247, 0.3), transparent);
    animation: glowPulse 3s ease-in-out infinite;
  }
  @keyframes glowPulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }
  .logo-group {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
  }
  .logo-icon {
    display: flex;
    align-items: center;
    animation: logoFloat 4s ease-in-out infinite;
  }
  @keyframes logoFloat {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-2px); }
  }
  h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    background: linear-gradient(135deg, #89b4fa, #b4befe, #cba6f7);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .subtitle {
    font-size: 13px;
    color: rgba(108, 112, 134, 0.8);
    padding-left: 4px;
    border-left: 1px solid rgba(108, 112, 134, 0.3);
  }

  /* Project bar */
  .project-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
    flex-shrink: 1;
    min-width: 0;
  }
  .btn-open {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    background: rgba(137, 180, 250, 0.1);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.15);
    border-radius: 8px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    transition: all 0.2s ease;
    font-family: inherit;
  }
  .btn-open:hover {
    background: rgba(137, 180, 250, 0.2);
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 12px rgba(137, 180, 250, 0.15);
  }
  .project-path {
    font-size: 12px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    color: rgba(205, 214, 244, 0.6);
    background: rgba(63, 63, 70, 0.4);
    padding: 5px 10px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.04);
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }
  .git-info {
    display: flex;
    align-items: center;
    gap: 6px;
    color: rgba(205, 214, 244, 0.7);
    font-size: 13px;
    flex-shrink: 0;
  }
  .branch-name {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 12px;
    color: #a6e3a1;
  }
  .git-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 8px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
  }
  .git-badge.ahead {
    background: rgba(137, 180, 250, 0.15);
    color: #89b4fa;
  }
  .git-badge.behind {
    background: rgba(243, 139, 168, 0.15);
    color: #f38ba8;
  }
  .git-badge.changes {
    background: rgba(249, 226, 175, 0.15);
    color: #f9e2af;
  }
</style>
