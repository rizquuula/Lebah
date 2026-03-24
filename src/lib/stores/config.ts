import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "../types";

export const projectConfig = writable<ProjectConfig>({
  review_template: null,
  merge_template: null,
  inprogress_template: null,
  claude_path: null,
  worktree_model: null,
  default_use_plan: null,
  default_yolo: null,
  default_auto: null,
  env_vars: null,
});

export async function loadProjectConfig(): Promise<void> {
  try {
    const config = await invoke<ProjectConfig>("get_project_config");
    projectConfig.set(config);
  } catch {
    projectConfig.set({ review_template: null, merge_template: null, inprogress_template: null, claude_path: null, worktree_model: null, default_use_plan: null, default_yolo: null, default_auto: null, env_vars: null });
  }
}

export async function saveProjectConfig(config: ProjectConfig): Promise<void> {
  await invoke("set_project_config", { config });
  projectConfig.set(config);
}
