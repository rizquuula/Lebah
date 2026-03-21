import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { GitStatus } from "../types";
import { loadTasks } from "./tasks";

export const projectPath = writable<string | null>(null);
export const gitStatus = writable<GitStatus | null>(null);

export async function openProject(path: string): Promise<void> {
  await invoke("set_project_path", { path });
  projectPath.set(path);
  await loadTasks();
  await refreshGitStatus();
}

export async function refreshGitStatus(): Promise<void> {
  try {
    const status = await invoke<GitStatus>("get_git_status");
    gitStatus.set(status);
  } catch {
    gitStatus.set(null);
  }
}

export async function loadProjectPath(): Promise<void> {
  const path = await invoke<string | null>("get_project_path");
  projectPath.set(path);
  if (path) {
    await loadTasks();
    await refreshGitStatus();
  }
}
