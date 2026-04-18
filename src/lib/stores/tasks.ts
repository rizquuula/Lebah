import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { TaskColumn, TaskStatus, type Task, DEFAULT_REVIEW_TEMPLATE, DEFAULT_MERGE_TEMPLATE } from "../types";
import { projectConfig } from "./config";

export const tasks = writable<Task[]>([]);

// Track which sessions are actively running in-process so that loadTasks()
// doesn't overwrite their in-memory Running status with stale DB state.
const runningSessions = new Set<string>();

// Track waiting merge task IDs so loadTasks() doesn't overwrite their status.
const waitingMergeSessions = new Set<string>();

// Terminal statuses recorded by session listeners that may have fired while a
// different project was active or concurrently with a loadTasks() await. The
// next loadTasks() for a matching task applies and persists the status,
// preventing stuck Running state after a project switch.
const recentlyCompleted = new Map<string, TaskStatus>();

// Captured task state for auto-advance (survives project switches)
interface CapturedTaskInfo {
  auto: boolean;
  model: string | null;
  yolo: boolean;
  description: string;
  use_plan: boolean;
  worktree: string | null;
  agent_name: string | null;
  has_run: boolean;
}

// Queue for sequential merge processing
interface MergeJob {
  id: string;
  description: string;
  usePlan: boolean;
  yolo: boolean;
  worktree: string | null;
  model: string | null;
  agentName: string | null;
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
  const taskInfo: CapturedTaskInfo = {
    auto: true, model: job.model, yolo: job.yolo, description: job.description,
    use_plan: job.usePlan, worktree: job.worktree, agent_name: job.agentName, has_run: job.hasRun,
  };
  if (job.hasRun && job.template) {
    await sendInputWithListener(job.id, job.template, TaskColumn.Merge, taskInfo, job.model, job.yolo);
  } else {
    await runAgentSession(job.id, job.description, job.usePlan, job.yolo, job.worktree, job.model, job.agentName);
  }
}

async function handleAutoAdvance(id: string, taskColumn: TaskColumn, taskInfo: CapturedTaskInfo): Promise<void> {
  if (!taskInfo.auto) return;

  const cfg = get(projectConfig);

  if (taskColumn === TaskColumn.InProgress) {
    await moveTask(id, TaskColumn.Review, 0);
    const tpl = cfg.review_template ?? DEFAULT_REVIEW_TEMPLATE;
    await sendInputWithListener(id, tpl, TaskColumn.Review, taskInfo, taskInfo.model, taskInfo.yolo);
  } else if (taskColumn === TaskColumn.Review) {
    // moveTask to Merge already done by caller
    const tpl = cfg.merge_template ?? DEFAULT_MERGE_TEMPLATE;
    if (get(tasks).some((t) => t.column === TaskColumn.Merge && t.status === TaskStatus.Running && t.id !== id)) {
      await queueMergeTask({ id, description: taskInfo.description, usePlan: taskInfo.use_plan, yolo: taskInfo.yolo, worktree: taskInfo.worktree, model: taskInfo.model, agentName: taskInfo.agent_name, hasRun: taskInfo.has_run, template: tpl });
    } else {
      await sendInputWithListener(id, tpl, TaskColumn.Merge, taskInfo, taskInfo.model, taskInfo.yolo);
    }
  }
  // Merge → Completed already handled, no further action needed
}

export async function loadTasks() {
  try {
    const result = await invoke<Task[]>("get_tasks");
    tasks.set(
      result.map((t) => {
        if (runningSessions.has(t.id)) return { ...t, status: TaskStatus.Running };
        if (waitingMergeSessions.has(t.id)) return { ...t, status: TaskStatus.Waiting };
        const pending = recentlyCompleted.get(t.id);
        if (pending !== undefined) {
          recentlyCompleted.delete(t.id);
          if (t.status !== pending) {
            invoke("update_task", { task: { ...t, status: pending } }).catch(() => {});
          }
          return { ...t, status: pending };
        }
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
  agentName: string | null = null,
): Promise<Task> {
  const task = await invoke<Task>("create_task", {
    description,
    claudePath,
    worktree,
    model,
    agentName,
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

export async function runAgentSession(
  id: string,
  description: string,
  usePlan: boolean,
  yolo: boolean,
  worktree: string | null = null,
  model: string | null = null,
  agentName: string | null = null,
): Promise<void> {
  runningSessions.add(id);

  // Capture task state now so the listener doesn't depend on the tasks store
  // (which gets replaced when the user switches projects).
  const currentTask = get(tasks).find((t) => t.id === id);
  let capturedColumn = currentTask?.column;
  const capturedInfo: CapturedTaskInfo = {
    auto: currentTask?.auto ?? false,
    model: model,
    yolo: yolo,
    description: description,
    use_plan: usePlan,
    worktree: worktree,
    agent_name: agentName,
    has_run: currentTask?.has_run ?? false,
  };

  const unlisten = await listen<string>(`claude-output-${id}`, async (event) => {
    try {
      const msg = JSON.parse(event.payload);
      if (msg.type === "result") {
        unlisten();
        runningSessions.delete(id);
        const status: TaskStatus = msg.is_error ? TaskStatus.Failed : TaskStatus.Success;
        recentlyCompleted.set(id, status);
        tasks.update((all) =>
          all.map((t) => (t.id === id ? { ...t, status } : t)),
        );
        if (status === TaskStatus.Success && capturedColumn) {
          if (capturedColumn === TaskColumn.Review) {
            await moveTask(id, TaskColumn.Merge, 0);
          } else if (capturedColumn === TaskColumn.Merge) {
            await moveTask(id, TaskColumn.Completed, 0);
            await startNextWaitingMerge();
          }
          await handleAutoAdvance(id, capturedColumn, capturedInfo);
        }
      }
    } catch {}
  });

  try {
    await invoke("run_agent_session", { id, description, usePlan, yolo, claudePath: null, worktree, model, agentName });
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

export async function stopAgentSession(id: string): Promise<void> {
  await invoke("stop_claude_session", { id }); // Command name unchanged — agent-agnostic stop
  runningSessions.delete(id);
  await loadTasks();
}

export async function sendInput(id: string, input: string, model: string | null = null, yolo: boolean = false): Promise<void> {
  await invoke("send_input", { id, input, model, yolo });
}

export async function sendInputWithListener(
  id: string,
  input: string,
  taskColumn: TaskColumn | null = null,
  taskInfo: CapturedTaskInfo | null = null,
  model: string | null = null,
  yolo: boolean = false,
): Promise<void> {
  runningSessions.add(id);
  tasks.update((all) =>
    all.map((t) => (t.id === id ? { ...t, status: TaskStatus.Running } : t)),
  );

  // If caller didn't provide captured state, read from store now (works when same project is active).
  if (taskColumn === null || taskInfo === null) {
    const currentTask = get(tasks).find((t) => t.id === id);
    if (taskColumn === null) taskColumn = currentTask?.column ?? null;
    if (taskInfo === null && currentTask) {
      taskInfo = {
        auto: currentTask.auto, model: currentTask.model, yolo: currentTask.yolo,
        description: currentTask.description, use_plan: currentTask.use_plan,
        worktree: currentTask.worktree, agent_name: currentTask.agent_name, has_run: currentTask.has_run,
      };
    }
  }
  const col = taskColumn;
  const info = taskInfo;

  const unlisten = await listen<string>(`claude-output-${id}`, async (event) => {
    try {
      const msg = JSON.parse(event.payload);
      if (msg.type === "result") {
        unlisten();
        runningSessions.delete(id);
        const status: TaskStatus = msg.is_error ? TaskStatus.Failed : TaskStatus.Success;
        recentlyCompleted.set(id, status);
        tasks.update((all) =>
          all.map((t) => (t.id === id ? { ...t, status } : t)),
        );
        if (status === TaskStatus.Success && col) {
          if (col === TaskColumn.Review) {
            await moveTask(id, TaskColumn.Merge, 0);
          } else if (col === TaskColumn.Merge) {
            await moveTask(id, TaskColumn.Completed, 0);
            await startNextWaitingMerge();
          }
          if (info) {
            await handleAutoAdvance(id, col, info);
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
