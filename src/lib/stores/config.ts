import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "../types";
import { projectPath } from "./project";

export const projectConfig = writable<ProjectConfig>({
  review_template: null,
  merge_template: null,
  inprogress_template: null,
  claude_path: null,
  opencode_path: null,
  worktree_model: null,
  default_use_plan: null,
  default_yolo: null,
  default_auto: null,
  env_vars: null,
  disabled_env_var_keys: null,
  worktree_links: null,
});

export async function loadProjectConfig(): Promise<void> {
  try {
    const config = await invoke<ProjectConfig>("get_project_config");
    projectConfig.set(config);
  } catch {
    projectConfig.set({ review_template: null, merge_template: null, inprogress_template: null, claude_path: null, opencode_path: null, worktree_model: null, default_use_plan: null, default_yolo: null, default_auto: null, env_vars: null, disabled_env_var_keys: null, worktree_links: null });
  }
}

// Subscribe to projectPath changes to auto-reload config
let configSubscription: ReturnType<typeof projectPath.subscribe> | null = null;

export function initializeConfigSubscription(): void {
  // Clean up existing subscription if any
  if (configSubscription) {
    configSubscription();
  }
  // Subscribe to projectPath and reload config when it changes
  configSubscription = projectPath.subscribe((path) => {
    if (path) {
      loadProjectConfig();
    }
  });
}

export async function saveProjectConfig(config: ProjectConfig): Promise<void> {
  await invoke("set_project_config", { config });
  projectConfig.set(config);
}
