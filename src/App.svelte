<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import Board from "./lib/components/Board.svelte";
  import SettingsModal from "./lib/components/SettingsModal.svelte";
  import RecentProjectDropdown from "./lib/components/RecentProjectDropdown.svelte";
  import ConfirmDialog from "./lib/components/ConfirmDialog.svelte";
  import { projectPath, gitStatus, openProject, loadProjectPath, refreshGitStatus } from "./lib/stores/project";
  import { lastError, clearError } from "./lib/stores/errors";
  import { initializeConfigSubscription } from "./lib/stores/config";

  let showSettings = false;
  let showPushDialog = false;
  let isPushing = false;

  let gitPollInterval: ReturnType<typeof setInterval>;

  onMount(async () => {
    await loadProjectPath();
    initializeConfigSubscription();
    gitPollInterval = setInterval(() => {
      if ($projectPath) refreshGitStatus();
    }, 30000);
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

  async function handleGitPush() {
    showPushDialog = true;
  }

  async function confirmPush() {
    showPushDialog = false;
    isPushing = true;
    try {
      await invoke<string>("git_push");
      refreshGitStatus();
    } catch (e) {
      console.error("Push failed:", e);
    } finally {
      isPushing = false;
    }
  }
</script>

<main>
  <header>
    <div class="logo-group">
      <div class="logo-icon">
        <img src="/lebah-logo.png" alt="Lebah" />
      </div>
      <h1>Lebah</h1>
      <span class="subtitle">Claude Code Orchestrator</span>
    </div>

    <div class="project-bar">
      <RecentProjectDropdown />
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
          <button
            class="btn-push"
            on:click={handleGitPush}
            disabled={isPushing}
            title="Push to remote"
          >
            {#if isPushing}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="spinner">
                <path d="M21 12a9 9 0 11-6.219-8.56"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/>
              </svg>
            {/if}
          </button>
        </div>
      {/if}

      <button class="btn-settings" on:click={() => showSettings = true} title="Settings">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
        </svg>
      </button>
    </div>

    <div class="header-glow"></div>
  </header>
  {#if $lastError}
    <button class="error-banner" on:click={clearError}>
      <span>{$lastError}</span>
      <span class="error-close" aria-hidden="true">✕</span>
    </button>
  {/if}
  <Board />
  {#if showSettings}
    <SettingsModal onClose={() => showSettings = false} />
  {/if}
  {#if showPushDialog}
    <ConfirmDialog
      title="Push to Remote"
      detail="Push the current branch to its upstream remote?"
      confirmLabel="Push"
      onConfirm={confirmPush}
      onCancel={() => showPushDialog = false}
      loading={isPushing}
    />
  {/if}
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
    background: rgba(24, 24, 27, 0.95);
    border-bottom: 1px solid rgba(137, 180, 250, 0.1);
    position: relative;
    overflow: visible;
  }
  .header-glow {
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.4), rgba(203, 166, 247, 0.3), transparent);
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
  }
  .logo-icon img {
    width: 36px;
    height: 36px;
    object-fit: contain;
  }
  h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 800;
    background: linear-gradient(135deg, #ffd700 0%, #ffb300 45%, #e8890c 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.8));
    letter-spacing: 0.02em;
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
  .btn-push {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    background: rgba(137, 180, 250, 0.1);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.15);
    border-radius: 6px;
    cursor: pointer;
    flex-shrink: 0;
    transition: all 0.15s ease;
    font-family: inherit;
  }
  .btn-push:hover:not(:disabled) {
    background: rgba(137, 180, 250, 0.2);
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 10px rgba(137, 180, 250, 0.2);
  }
  .btn-push:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-push .spinner {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .btn-settings {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    padding: 0;
    background: rgba(63, 63, 70, 0.4);
    color: rgba(205, 214, 244, 0.5);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    cursor: pointer;
    flex-shrink: 0;
    font-family: inherit;
  }
  .btn-settings:hover {
    background: rgba(137, 180, 250, 0.15);
    color: #89b4fa;
    border-color: rgba(137, 180, 250, 0.25);
  }
  .error-banner {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.35);
    color: #f38ba8;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 13px;
    font-family: inherit;
    display: flex;
    align-items: center;
    gap: 10px;
    z-index: 2000;
    cursor: pointer;
    max-width: 500px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
    text-align: left;
  }
  .error-banner:hover { background: rgba(239, 68, 68, 0.25); }
  .error-close { font-size: 12px; opacity: 0.7; flex-shrink: 0; }
</style>
