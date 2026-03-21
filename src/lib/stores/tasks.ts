import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Task, TaskColumn } from "../types";

export const tasks = writable<Task[]>([]);

export async function loadTasks() {
  try {
    const result = await invoke<Task[]>("get_tasks");
    tasks.set(result);
  } catch {
    tasks.set([]);
  }
}

export async function createTask(
  description: string,
  claudePath: string | null = null,
  claudeCommand: string | null = null,
  worktree: string | null = null,
): Promise<Task> {
  const task = await invoke<Task>("create_task", {
    description,
    claudePath,
    claudeCommand,
    worktree,
  });
  await loadTasks();
  return task;
}

export async function updateTask(task: Task): Promise<void> {
  await invoke("update_task", { task });
  await loadTasks();
}

export async function deleteTask(id: string): Promise<void> {
  await invoke("delete_task", { id });
  await loadTasks();
}

export async function moveTask(
  id: string,
  column: TaskColumn,
  sortOrder: number,
): Promise<void> {
  await invoke("move_task", { id, column, sortOrder });
  await loadTasks();
}

export async function runClaudeSession(
  id: string,
  description: string,
  usePlan: boolean,
  yolo: boolean,
  claudePath: string | null = null,
  claudeCommand: string | null = null,
  worktree: string | null = null,
): Promise<void> {
  try {
    await invoke("run_claude_session", { id, description, usePlan, yolo, claudePath, claudeCommand, worktree });
  } finally {
    await loadTasks();
  }
}

export async function getOutputBuffer(id: string): Promise<string[]> {
  return invoke<string[]>("get_output_buffer", { id });
}

export async function stopClaudeSession(id: string): Promise<void> {
  await invoke("stop_claude_session", { id });
  await loadTasks();
}

export async function sendInput(id: string, input: string): Promise<void> {
  await invoke("send_input", { id, input });
}
