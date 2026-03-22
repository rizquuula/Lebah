import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Task, TaskColumn, TaskStatus } from "../types";

export const tasks = writable<Task[]>([]);

// Track which sessions are actively running in-process so that loadTasks()
// doesn't overwrite their in-memory Running status with stale DB state.
const runningSessions = new Set<string>();

export async function loadTasks() {
  try {
    const result = await invoke<Task[]>("get_tasks");
    tasks.set(
      result.map((t) =>
        runningSessions.has(t.id) ? { ...t, status: "Running" as TaskStatus } : t,
      ),
    );
  } catch (e) {
    console.error("loadTasks failed:", e);
    tasks.set([]);
  }
}

export async function createTask(
  description: string,
  claudePath: string | null = null,
  claudeCommand: string | null = null,
  worktree: string | null = null,
  model: string | null = null,
): Promise<Task> {
  const task = await invoke<Task>("create_task", {
    description,
    claudePath,
    claudeCommand,
    worktree,
    model,
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
  model: string | null = null,
): Promise<void> {
  runningSessions.add(id);

  const unlisten = await listen<string>(`claude-output-${id}`, async (event) => {
    try {
      const msg = JSON.parse(event.payload);
      if (msg.type === "result") {
        unlisten();
        runningSessions.delete(id);
        const status: TaskStatus = msg.is_error ? "Failed" : "Success";
        let taskColumn: string | undefined;
        tasks.update((all) => {
          const found = all.find((t) => t.id === id);
          if (found) taskColumn = found.column;
          return all.map((t) => (t.id === id ? { ...t, status } : t));
        });
        if (status === "Success" && taskColumn) {
          if (taskColumn === "Review") await moveTask(id, "Merge", 0);
          else if (taskColumn === "Merge") await moveTask(id, "Completed", 0);
        }
      }
    } catch {}
  });

  try {
    await invoke("run_claude_session", { id, description, usePlan, yolo, claudePath, claudeCommand, worktree, model });
    await loadTasks();
  } catch {
    unlisten();
    runningSessions.delete(id);
    await loadTasks();
    throw new Error("Session failed to start");
  }
}

export async function getOutputBuffer(id: string): Promise<string[]> {
  return invoke<string[]>("get_output_buffer", { id });
}

export async function stopClaudeSession(id: string): Promise<void> {
  await invoke("stop_claude_session", { id });
  await loadTasks();
}

export async function sendInput(id: string, input: string, model: string | null = null, usePlan: boolean = false, yolo: boolean = false): Promise<void> {
  await invoke("send_input", { id, input, model, usePlan, yolo });
}

export async function sendInputWithListener(
  id: string,
  input: string,
  model: string | null = null,
  usePlan: boolean = false,
  yolo: boolean = false,
): Promise<void> {
  runningSessions.add(id);
  tasks.update((all) =>
    all.map((t) => (t.id === id ? { ...t, status: "Running" as TaskStatus } : t)),
  );

  // Emit synthetic user message so TerminalModal shows the sent text
  const { emit } = await import("@tauri-apps/api/event");
  await emit(`claude-output-${id}`, JSON.stringify({ type: "user_input", text: input }));

  const unlisten = await listen<string>(`claude-output-${id}`, async (event) => {
    try {
      const msg = JSON.parse(event.payload);
      if (msg.type === "result") {
        unlisten();
        runningSessions.delete(id);
        const status: TaskStatus = msg.is_error ? "Failed" : "Success";
        let taskColumn: string | undefined;
        tasks.update((all) => {
          const t = all.find((t) => t.id === id);
          if (t) taskColumn = t.column;
          return all.map((t) => (t.id === id ? { ...t, status } : t));
        });
        if (status === "Success" && taskColumn) {
          if (taskColumn === "Review") await moveTask(id, "Merge", 0);
          else if (taskColumn === "Merge") await moveTask(id, "Completed", 0);
        }
      }
    } catch {}
  });

  try {
    await sendInput(id, input, model, usePlan, yolo);
  } catch (e) {
    unlisten();
    runningSessions.delete(id);
    throw e;
  }
}

export async function moveTaskBatch(
  moves: { id: string; column: TaskColumn; sortOrder: number }[],
): Promise<void> {
  await Promise.all(
    moves.map((m) => invoke("move_task", { id: m.id, column: m.column, sortOrder: m.sortOrder })),
  );
  await loadTasks();
}

export async function resetTaskSession(id: string): Promise<Task> {
  const newTask = await invoke<Task>("reset_task_session", { id });
  await loadTasks();
  return newTask;
}
