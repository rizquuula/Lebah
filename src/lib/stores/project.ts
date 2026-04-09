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

export async function switchProject(path: string): Promise<void> {
  await openProject(path);
}

export async function getRecentProjects(): Promise<string[]> {
  return await invoke<string[]>("get_recent_projects", { maxCount: 10 });
}

export async function removeRecentProject(path: string): Promise<void> {
  await invoke("remove_recent_project", { path });
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
