<script lang="ts">
  import { projectConfig, saveProjectConfig } from "../stores/config";
  import type { ProjectConfig } from "../types";

  export let onClose: () => void;

  let activeTab: "general" | "env" = "general";

  let claudePath = $projectConfig.claude_path ?? "";
  let worktreeModel = $projectConfig.worktree_model ?? "haiku";
  let defaultUsePlan = $projectConfig.default_use_plan ?? false;
  let defaultYolo = $projectConfig.default_yolo ?? true;
  let defaultAuto = $projectConfig.default_auto ?? false;

  let envVars: { key: string; value: string }[] = (() => {
    const vars = $projectConfig.env_vars;
    if (vars && Object.keys(vars).length > 0) {
      return Object.entries(vars).map(([key, value]) => ({ key, value }));
    }
    return [{ key: "IS_SANDBOX", value: "0" }];
  })();

  function addEnvVar() {
    envVars = [...envVars, { key: "", value: "" }];
  }

  function removeEnvVar(index: number) {
    envVars = envVars.filter((_, i) => i !== index);
  }

  async function handleSave() {
    const envMap: Record<string, string> = {};
    for (const { key, value } of envVars) {
      const k = key.trim();
      if (k) envMap[k] = value;
    }

    const config: ProjectConfig = {
      ...$projectConfig,
      claude_path: claudePath.trim() || null,
      worktree_model: worktreeModel || null,
      default_use_plan: defaultUsePlan,
      default_yolo: defaultYolo,
      default_auto: defaultAuto,
      env_vars: envMap,
    };

    await saveProjectConfig(config);
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" on:click={onClose}>
  <div class="modal" on:click|stopPropagation>
    <div class="modal-glow"></div>
    <div class="header">
      <h3>Settings</h3>
      <button class="btn-close" on:click={onClose} title="Close settings">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="tabs">
      <button class="tab" class:active={activeTab === "general"} on:click={() => activeTab = "general"}>General</button>
      <button class="tab" class:active={activeTab === "env"} on:click={() => activeTab = "env"}>Environment Variables</button>
    </div>

    <div class="tab-content">
      {#if activeTab === "general"}
        <label class="field-label" for="s-claude-path">Claude Code Path</label>
        <input id="s-claude-path" type="text" bind:value={claudePath} placeholder="claude (default)" class="text-input" />

        <label class="field-label" for="s-worktree-model">Worktree Generator Model</label>
        <select id="s-worktree-model" bind:value={worktreeModel} class="text-input">
          <option value="haiku">haiku</option>
          <option value="sonnet">sonnet</option>
          <option value="opus">opus</option>
        </select>

        <div class="toggles-section">
          <label class="toggle-row">
            <span class="toggle-label">Default Plan Mode</span>
            <input type="checkbox" bind:checked={defaultUsePlan} class="toggle-input" />
            <span class="toggle-switch"></span>
          </label>
          <label class="toggle-row">
            <span class="toggle-label">Default YOLO Mode</span>
            <input type="checkbox" bind:checked={defaultYolo} class="toggle-input" />
            <span class="toggle-switch"></span>
          </label>
          <label class="toggle-row">
            <span class="toggle-label">Default Auto Mode</span>
            <input type="checkbox" bind:checked={defaultAuto} class="toggle-input" />
            <span class="toggle-switch"></span>
          </label>
        </div>
      {:else}
        <div class="env-list">
          {#each envVars as envVar, i}
            <div class="env-row">
              <input type="text" bind:value={envVar.key} placeholder="KEY" class="text-input env-key" />
              <span class="env-eq">=</span>
              <input type="text" bind:value={envVar.value} placeholder="value" class="text-input env-val" />
              <button type="button" class="btn-remove" on:click={() => removeEnvVar(i)} title="Remove">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
                  <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
          {/each}
        </div>
        <button type="button" class="btn-add" on:click={addEnvVar}>+ Add Variable</button>
      {/if}
    </div>

    <div class="actions">
      <button class="btn-cancel" on:click={onClose}>Cancel</button>
      <button class="btn-save" on:click={handleSave}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        Save
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .modal {
    background: rgba(39, 39, 42, 0.85);
    backdrop-filter: blur(24px) saturate(1.5);
    -webkit-backdrop-filter: blur(24px) saturate(1.5);
    border: 1px solid rgba(137, 180, 250, 0.12);
    border-radius: 16px;
    padding: 28px;
    width: 100vw;
    height: 100vh;
    max-width: 100vw;
    max-height: 100vh;
    position: relative;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
  }
  .modal-glow {
    position: absolute;
    top: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 30%;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.5), transparent);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }
  h3 {
    margin: 0;
    color: rgba(205, 214, 244, 0.9);
    font-size: 18px;
    font-weight: 600;
  }
  .btn-close {
    background: transparent;
    border: none;
    color: rgba(205, 214, 244, 0.5);
    cursor: pointer;
    padding: 4px;
    border-radius: 6px;
  }
  .btn-close:hover { color: #cdd6f4; background: rgba(255,255,255,0.06); }
  .tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    padding-bottom: 0;
  }
  .tab {
    background: transparent;
    border: none;
    color: rgba(205, 214, 244, 0.45);
    font-size: 14px;
    font-weight: 500;
    padding: 8px 16px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    border-radius: 0;
  }
  .tab:hover { color: rgba(205, 214, 244, 0.7); }
  .tab.active {
    color: #89b4fa;
    border-bottom-color: #89b4fa;
  }
  .tab-content {
    flex: 1;
    max-width: 560px;
  }
  .field-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: rgba(205, 214, 244, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 6px;
    margin-top: 14px;
  }
  .field-label:first-child { margin-top: 0; }
  .text-input {
    width: 100%;
    background: rgba(63, 63, 70, 0.5);
    color: #cdd6f4;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 12px;
    font-family: inherit;
    font-size: 14px;
    box-sizing: border-box;
    height: 42px;
  }
  .text-input:focus {
    outline: none;
    border-color: rgba(137, 180, 250, 0.35);
    box-shadow: 0 0 16px rgba(137, 180, 250, 0.08);
  }
  .text-input::placeholder { color: rgba(108, 112, 134, 0.6); }
  select.text-input {
    color-scheme: dark;
    padding: 0 12px;
    appearance: auto;
  }
  .toggles-section { margin-top: 20px; }
  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 0;
    cursor: pointer;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  .toggle-label {
    color: rgba(205, 214, 244, 0.8);
    font-size: 14px;
  }
  .toggle-input { display: none; }
  .toggle-switch {
    width: 40px;
    height: 22px;
    background: rgba(82, 82, 91, 0.6);
    border-radius: 11px;
    position: relative;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle-switch::after {
    content: "";
    position: absolute;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: rgba(205, 214, 244, 0.6);
    top: 3px;
    left: 3px;
    transition: transform 0.2s;
  }
  .toggle-input:checked + .toggle-switch {
    background: rgba(137, 180, 250, 0.4);
  }
  .toggle-input:checked + .toggle-switch::after {
    transform: translateX(18px);
    background: #89b4fa;
  }
  .env-list { display: flex; flex-direction: column; gap: 8px; }
  .env-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .env-key { width: 180px; flex-shrink: 0; font-family: monospace; }
  .env-eq {
    color: rgba(205, 214, 244, 0.35);
    font-size: 16px;
    flex-shrink: 0;
  }
  .env-val { flex: 1; font-family: monospace; }
  .btn-remove {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(239, 68, 68, 0.1);
    color: rgba(239, 68, 68, 0.6);
    border: 1px solid rgba(239, 68, 68, 0.15);
    border-radius: 8px;
    cursor: pointer;
  }
  .btn-remove:hover {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }
  .btn-add {
    margin-top: 12px;
    background: rgba(137, 180, 250, 0.1);
    color: rgba(137, 180, 250, 0.7);
    border: 1px dashed rgba(137, 180, 250, 0.2);
    border-radius: 8px;
    padding: 8px 16px;
    cursor: pointer;
    font-size: 13px;
  }
  .btn-add:hover {
    background: rgba(137, 180, 250, 0.18);
    color: #89b4fa;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 24px;
    padding-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }
  button {
    padding: 9px 18px;
    border: none;
    border-radius: 9px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .btn-cancel {
    background: rgba(82, 82, 91, 0.5);
    color: rgba(205, 214, 244, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }
  .btn-cancel:hover { background: rgba(82, 82, 91, 0.7); color: #cdd6f4; }
  .btn-save {
    background: rgba(137, 180, 250, 0.2);
    color: #89b4fa;
    border: 1px solid rgba(137, 180, 250, 0.25);
    font-weight: 600;
  }
  .btn-save:hover {
    background: rgba(137, 180, 250, 0.3);
    box-shadow: 0 0 16px rgba(137, 180, 250, 0.15);
  }
</style>
