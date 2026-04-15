<script lang="ts">
  export let usePlan: boolean;
  export let yolo: boolean;
  export let auto: boolean = false;
  export let showPlan: boolean = true;
  export let showAuto: boolean = true;
  export let disablePlan: boolean = false;
  export let disableYolo: boolean = false;
  export let onTogglePlan: () => void;
  export let onToggleYolo: () => void;
  export let onToggleAuto: () => void = () => {};
</script>

<div class="toggles">
  {#if showPlan}
  <label class="toggle" class:disabled={disablePlan} title={disablePlan ? "Not supported by this agent" : "Use Plan"}>
    <div class="toggle-track" class:active={usePlan && !disablePlan}>
      <div class="toggle-thumb"></div>
    </div>
    <input type="checkbox" data-testid="toggle-plan" checked={usePlan} on:change={onTogglePlan} class="sr-only" disabled={disablePlan} />
    <span class="toggle-label">Plan</span>
  </label>
  {/if}
  <label class="toggle" class:disabled={disableYolo} title={disableYolo ? "Not supported by this agent" : "Skip permissions (--dangerously-skip-permissions)"}>
    <div class="toggle-track" class:active={yolo && !disableYolo} class:yolo={yolo && !disableYolo}>
      <div class="toggle-thumb"></div>
    </div>
    <input type="checkbox" data-testid="toggle-yolo" checked={yolo} on:change={onToggleYolo} class="sr-only" disabled={disableYolo} />
    <span class="toggle-label">Yolo</span>
  </label>
  {#if showAuto}
  <label class="toggle" title="Auto-advance through columns on success">
    <div class="toggle-track" class:active={auto} class:auto={auto}>
      <div class="toggle-thumb"></div>
    </div>
    <input type="checkbox" data-testid="toggle-auto" checked={auto} on:change={onToggleAuto} class="sr-only" />
    <span class="toggle-label">Auto</span>
  </label>
  {/if}
</div>

<style>
  .toggles {
    display: flex;
    flex-direction: row;
    gap: 10px;
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
  .toggle-track.auto {
    background: rgba(148, 226, 213, 0.25);
    border-color: rgba(148, 226, 213, 0.4);
  }
  .toggle-track.auto .toggle-thumb {
    background: #94e2d5;
    box-shadow: 0 0 6px rgba(148, 226, 213, 0.4);
  }
  .toggle-label { user-select: none; }
  .toggle.disabled { opacity: 0.35; cursor: not-allowed; }
</style>
