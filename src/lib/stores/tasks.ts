import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TaskColumn, TaskStatus, type Task } from "../types";

export const tasks = writable<Task[]>([]);

// Track which sessions are actively running in-process so that loadTasks()
// doesn't overwrite their in-memory Running status with stale DB state.
const runningSessions = new Set<string>();

// Track waiting merge task IDs so loadTasks() doesn't overwrite their status.
const waitingMergeSessions = new Set<string>();

// Queue for sequential merge processing
interface MergeJob {
  id: string;
  description: string;
  usePlan: boolean;
  yolo: boolean;
  claudePath: string | null;
  worktree: string | null;
  model: string | null;
  hasRun: boolean;
  template: string | null;
}
const mergeWaitQueue: MergeJob[] = [];

export function isAnyMergeRunning(): boolean {
  return get(tasks).some((t) => t.column === TaskColumn.Merge && t.status === TaskStatus.Running);
}

export async function queueMergeTask(job: MergeJob): Promise<void> {
  mergeWaitQueue.push(job);
  waitingMergeSessions.add(job.id);
  tasks.update((all) =>
    all.map((t) => (t.id === job.id ? { ...t, status: TaskStatus.Waiting } : t)),
  );
  const currentTask = get(tasks).find((t) => t.id === job.id);
  if (currentTask) {
    await invoke("update_task", { task: { ...currentTask, status: TaskStatus.Waiting } });
  }
}

export function cancelMergeWait(id: string): void {
  const idx = mergeWaitQueue.findIndex((j) => j.id === id);
  if (idx !== -1) mergeWaitQueue.splice(idx, 1);
  waitingMergeSessions.delete(id);
}

async function startNextWaitingMerge(): Promise<void> {
  const job = mergeWaitQueue.shift();
  if (!job) return;
  waitingMergeSessions.delete(job.id);
  if (job.hasRun && job.template) {
    await sendInputWithListener(job.id, job.template, job.model, job.yolo);
  } else {
    await runClaudeSession(job.id, job.description, job.usePlan, job.yolo, job.claudePath, job.worktree, job.model);
  }
}

export async function loadTasks() {
  try {
    const result = await invoke<Task[]>("get_tasks");
    tasks.set(
      result.map((t) => {
        if (runningSessions.has(t.id)) return { ...t, status: TaskStatus.Running };
        if (waitingMergeSessions.has(t.id)) return { ...t, status: TaskStatus.Waiting };
        return t;
      }),
    );
  } catch (e) {
    console.error("loadTasks failed:", e);
    tasks.set([]);
  }
}

export async function createTask(
  description: string,
  claudePath: string | null = null,
  worktree: string | null = null,
  model: string | null = null,
): Promise<Task> {
  const task = await invoke<Task>("create_task", {
    description,
    claudePath,
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
        const status: TaskStatus = msg.is_error ? TaskStatus.Failed : TaskStatus.Success;
        let taskColumn: TaskColumn | undefined;
        tasks.update((all) => {
          const found = all.find((t) => t.id === id);
          if (found) taskColumn = found.column;
          return all.map((t) => (t.id === id ? { ...t, status } : t));
        });
        if (status === TaskStatus.Success && taskColumn) {
          if (taskColumn === TaskColumn.Review) await moveTask(id, TaskColumn.Merge, 0);
          else if (taskColumn === TaskColumn.Merge) {
            await moveTask(id, TaskColumn.Completed, 0);
            await startNextWaitingMerge();
          }
        }
      }
    } catch {}
  });

  try {
    await invoke("run_claude_session", { id, description, usePlan, yolo, claudePath, worktree, model });
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

export async function sendInput(id: string, input: string, model: string | null = null, yolo: boolean = false): Promise<void> {
  await invoke("send_input", { id, input, model, yolo });
}

export async function sendInputWithListener(
  id: string,
  input: string,
  model: string | null = null,
  yolo: boolean = false,
): Promise<void> {
  runningSessions.add(id);
  tasks.update((all) =>
    all.map((t) => (t.id === id ? { ...t, status: TaskStatus.Running } : t)),
  );

  const unlisten = await listen<string>(`claude-output-${id}`, async (event) => {
    try {
      const msg = JSON.parse(event.payload);
      if (msg.type === "result") {
        unlisten();
        runningSessions.delete(id);
        const status: TaskStatus = msg.is_error ? TaskStatus.Failed : TaskStatus.Success;
        let taskColumn: TaskColumn | undefined;
        tasks.update((all) => {
          const t = all.find((t) => t.id === id);
          if (t) taskColumn = t.column;
          return all.map((t) => (t.id === id ? { ...t, status } : t));
        });
        if (status === TaskStatus.Success && taskColumn) {
          if (taskColumn === TaskColumn.Review) await moveTask(id, TaskColumn.Merge, 0);
          else if (taskColumn === TaskColumn.Merge) {
            await moveTask(id, TaskColumn.Completed, 0);
            await startNextWaitingMerge();
          }
        }
      }
    } catch {}
  });

  try {
    await sendInput(id, input, model, yolo);
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
