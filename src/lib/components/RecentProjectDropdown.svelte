<script lang="ts">
  import { onMount } from "svelte";
  import { getRecentProjects, switchProject } from "../stores/project";
  import { projectPath } from "../stores/project";
  import { cycleNext, moveNext, movePrev, initialIndex } from "../utils/keyboard-nav";

  let recentProjects: string[] = [];
  let isOpen = false;
  let dropdownRef: HTMLDivElement;
  let listRef: HTMLDivElement;

  // Keyboard navigation state
  let selectedIndex = -1;
  let keyNavActive = false; // true while Ctrl is held and we opened via shortcut

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
    selectedIndex = -1;
    keyNavActive = false;
  }

  function toggleDropdown() {
    isOpen = !isOpen;
    if (isOpen) {
      loadRecentProjects();
      selectedIndex = -1;
    } else {
      selectedIndex = -1;
      keyNavActive = false;
    }
  }

  function handleClickOutside(event: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
      isOpen = false;
      selectedIndex = -1;
      keyNavActive = false;
    }
  }

  function truncatePath(path: string, length: number = 40): string {
    if (path.length <= length) return path;
    return "..." + path.slice(-length);
  }

  function scrollSelectedIntoView() {
    if (!listRef) return;
    const items = listRef.querySelectorAll<HTMLElement>(".dropdown-item");
    if (items[selectedIndex]) {
      items[selectedIndex].scrollIntoView({ block: "nearest" });
    }
  }

  async function handleKeydown(e: KeyboardEvent) {
    // Ctrl+` — open/cycle through recent projects
    if (e.ctrlKey && e.key === "`") {
      e.preventDefault();
      if (!isOpen) {
        await loadRecentProjects();
        isOpen = true;
        selectedIndex = initialIndex(recentProjects.length);
        keyNavActive = true;
      } else {
        selectedIndex = cycleNext(selectedIndex, recentProjects.length);
      }
      scrollSelectedIntoView();
      return;
    }

    if (!isOpen) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = moveNext(selectedIndex, recentProjects.length);
      scrollSelectedIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = movePrev(selectedIndex, recentProjects.length);
      scrollSelectedIntoView();
    } else if (e.key === "Enter" && selectedIndex >= 0) {
      e.preventDefault();
      const path = recentProjects[selectedIndex];
      if (path) await handleSelectProject(path);
    } else if (e.key === "Escape") {
      e.preventDefault();
      isOpen = false;
      selectedIndex = -1;
      keyNavActive = false;
    }
  }

  async function handleKeyup(e: KeyboardEvent) {
    // When Ctrl is released while keyboard nav is active, open selected project
    if (e.key === "Control" && keyNavActive && isOpen) {
      if (selectedIndex >= 0 && recentProjects[selectedIndex]) {
        await handleSelectProject(recentProjects[selectedIndex]);
      } else {
        isOpen = false;
        keyNavActive = false;
        selectedIndex = -1;
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} on:mousedown={handleClickOutside} />

<div class="recent-project-dropdown" bind:this={dropdownRef}>
  <button
    class="btn-recent"
    class:open={isOpen}
    on:click={toggleDropdown}
    type="button"
    title="Recent projects (Ctrl+`)"
  >
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10"/>
      <polyline points="12 6 12 12 16 14"/>
    </svg>
    Recent
    <kbd class="shortcut-hint">Ctrl+`</kbd>
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="chevron" class:rotated={isOpen}>
      <polyline points="6 9 12 15 18 9"/>
    </svg>
  </button>

  {#if isOpen && recentProjects.length > 0}
    <div class="dropdown-menu" bind:this={listRef}>
      {#each recentProjects as path, index}
        <button
          class="dropdown-item"
          on:click={() => handleSelectProject(path)}
          class:active={path === currentProject}
          class:keyboard-selected={index === selectedIndex}
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

  .btn-recent:hover,
  .btn-recent.open {
    background: rgba(137, 180, 250, 0.2);
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 12px rgba(137, 180, 250, 0.15);
  }

  .shortcut-hint {
    font-size: 10px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    background: rgba(137, 180, 250, 0.15);
    color: rgba(137, 180, 250, 0.7);
    border: 1px solid rgba(137, 180, 250, 0.2);
    border-radius: 3px;
    padding: 1px 4px;
    line-height: 1.4;
  }

  .chevron {
    transition: transform 0.2s;
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 280px;
    max-width: 400px;
    max-height: 320px;
    overflow-y: auto;
    background: rgba(24, 24, 27, 0.98);
    border: 1px solid rgba(137, 180, 250, 0.15);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 10000;
    overflow-x: hidden;
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

  .dropdown-item.keyboard-selected {
    background: rgba(137, 180, 250, 0.2);
    color: #89b4fa;
    outline: 1px solid rgba(137, 180, 250, 0.4);
    outline-offset: -1px;
  }

  .dropdown-item.active.keyboard-selected {
    background: rgba(166, 227, 161, 0.2);
    color: #a6e3a1;
    outline-color: rgba(166, 227, 161, 0.4);
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
