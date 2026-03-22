<script lang="ts">
  export let usePlan: boolean;
  export let yolo: boolean;
  export let onTogglePlan: () => void;
  export let onToggleYolo: () => void;
</script>

<div class="toggles">
  <label class="toggle" title="Use Plan">
    <div class="toggle-track" class:active={usePlan}>
      <div class="toggle-thumb"></div>
    </div>
    <input type="checkbox" checked={usePlan} on:change={onTogglePlan} class="sr-only" />
    <span class="toggle-label">Plan</span>
  </label>
  <label class="toggle" title="Skip permissions (--dangerously-skip-permissions)">
    <div class="toggle-track" class:active={yolo} class:yolo={yolo}>
      <div class="toggle-thumb"></div>
    </div>
    <input type="checkbox" checked={yolo} on:change={onToggleYolo} class="sr-only" />
    <span class="toggle-label">Yolo</span>
  </label>
</div>

<style>
  .toggles {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: rgba(166, 173, 200, 0.8);
    cursor: pointer;
    user-select: none;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
  }
  .toggle-track {
    width: 28px;
    height: 16px;
    border-radius: 8px;
    background: rgba(82, 82, 91, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    position: relative;
  }
  .toggle-track.active {
    background: rgba(137, 180, 250, 0.25);
    border-color: rgba(137, 180, 250, 0.35);
  }
  .toggle-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: rgba(205, 214, 244, 0.6);
    position: absolute;
    top: 1px;
    left: 1px;
    transition: left 0.1s;
  }
  .toggle-track.active .toggle-thumb {
    left: 13px;
    background: #89b4fa;
    box-shadow: 0 0 6px rgba(137, 180, 250, 0.4);
  }
  .toggle-track.yolo {
    background: rgba(249, 115, 22, 0.25);
    border-color: rgba(249, 115, 22, 0.4);
  }
  .toggle-track.yolo .toggle-thumb {
    background: #f97316;
    box-shadow: 0 0 6px rgba(249, 115, 22, 0.4);
  }
  .toggle-label { user-select: none; }
</style>
