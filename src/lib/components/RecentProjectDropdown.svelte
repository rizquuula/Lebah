<script lang="ts">
  import { onMount } from "svelte";
  import { getRecentProjects, switchProject } from "../stores/project";
  import { projectPath } from "../stores/project";

  let recentProjects: string[] = [];
  let isOpen = false;
  let dropdownRef: HTMLDivElement;

  $: currentProject = $projectPath;

  onMount(() => {
    loadRecentProjects();
  });

  async function loadRecentProjects() {
    try {
      recentProjects = await getRecentProjects();
    } catch {
      recentProjects = [];
    }
  }

  async function handleSelectProject(path: string) {
    await switchProject(path);
    isOpen = false;
  }

  function toggleDropdown() {
    isOpen = !isOpen;
    if (isOpen) {
      loadRecentProjects();
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      isOpen = false;
    }
  }

  function truncatePath(path: string, length: number = 40): string {
    if (path.length <= length) return path;
    return "..." + path.slice(-length);
  }
</script>

<div class="recent-project-dropdown" bind:this={dropdownRef}>
  <button class="btn-recent" on:click={toggleDropdown} type="button">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10"/>
      <polyline points="12 6 12 12 16 14"/>
    </svg>
    Recent
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chevron">
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  </button>

  {#if isOpen && recentProjects.length > 0}
    <div class="dropdown-menu">
      {#each recentProjects as path, index}
        <button
          class="dropdown-item"
          on:click={() => handleSelectProject(path)}
          class:active={path === currentProject}
        >
          <span class="project-name" title={path}>{truncatePath(path)}</span>
          {#if path === currentProject}
            <span class="active-badge">Current</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}

  {#if isOpen && recentProjects.length === 0}
    <div class="dropdown-menu empty">
      <span class="empty-message">No recent projects</span>
    </div>
  {/if}
</div>

<style>
  .recent-project-dropdown {
    position: relative;
  }

  .btn-recent {
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

  .btn-recent:hover {
    background: rgba(137, 180, 250, 0.2);
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 12px rgba(137, 180, 250, 0.15);
  }

  .chevron {
    transition: transform 0.2s;
  }

  .btn-recent:global(.open) .chevron {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 280px;
    max-width: 400px;
    background: rgba(24, 24, 27, 0.98);
    border: 1px solid rgba(137, 180, 250, 0.15);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 10000;
    overflow: hidden;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 10px 14px;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    color: rgba(205, 214, 244, 0.8);
    cursor: pointer;
    font-size: 12px;
    font-family: inherit;
    text-align: left;
    transition: background 0.15s, color 0.15s;
  }

  .dropdown-item:last-child {
    border-bottom: none;
  }

  .dropdown-item:hover {
    background: rgba(137, 180, 250, 0.1);
    color: #89b4fa;
  }

  .dropdown-item.active {
    background: rgba(166, 227, 161, 0.1);
    color: #a6e3a1;
  }

  .project-name {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .active-badge {
    font-size: 10px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(166, 227, 161, 0.2);
    color: #a6e3a1;
    font-weight: 600;
    flex-shrink: 0;
    margin-left: 8px;
  }

  .dropdown-menu.empty {
    min-width: 200px;
  }

  .empty-message {
    display: block;
    padding: 14px;
    text-align: center;
    color: rgba(205, 214, 244, 0.4);
    font-size: 12px;
  }
</style>
