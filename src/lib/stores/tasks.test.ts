import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";
import { TaskColumn, TaskStatus, type Task } from "../types";

const mockInvoke = vi.fn();
const mockListen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
  emit: vi.fn(),
}));

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: "task-1",
    description: "test task",
    column: TaskColumn.Todo,
    status: TaskStatus.Idle,
    use_plan: false,
    yolo: false,
    auto: false,
    sort_order: 0,
    created_at: "2026-01-01T00:00:00Z",
    completed_at: null,
    claude_path: null,
    worktree: null,
    has_run: false,
    model: null,
    lines_added: null,
    lines_removed: null,
    ...overrides,
  };
}

describe("tasks store", () => {
  beforeEach(() => {
    vi.resetModules();
    mockInvoke.mockReset();
    mockListen.mockReset().mockResolvedValue(() => {});
  });

  describe("isAnyMergeRunning", () => {
    it("returns false when no merge tasks exist", async () => {
      const { isAnyMergeRunning, tasks } = await import("./tasks");
      tasks.set([makeTask({ column: TaskColumn.InProgress, status: TaskStatus.Running })]);
      expect(isAnyMergeRunning()).toBe(false);
    });

    it("returns true when a merge task is running", async () => {
      const { isAnyMergeRunning, tasks } = await import("./tasks");
      tasks.set([makeTask({ column: TaskColumn.Merge, status: TaskStatus.Running })]);
      expect(isAnyMergeRunning()).toBe(true);
    });

    it("returns false when merge task is not running", async () => {
      const { isAnyMergeRunning, tasks } = await import("./tasks");
      tasks.set([makeTask({ column: TaskColumn.Merge, status: TaskStatus.Success })]);
      expect(isAnyMergeRunning()).toBe(false);
    });
  });

  describe("queueMergeTask / cancelMergeWait", () => {
    it("queueMergeTask sets task status to Waiting in store", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { queueMergeTask, tasks } = await import("./tasks");
      const task = makeTask({ id: "t1", column: TaskColumn.Merge, status: TaskStatus.Idle });
      tasks.set([task]);

      await queueMergeTask({
        id: "t1",
        description: "desc",
        usePlan: false,
        yolo: false,
        worktree: null,
        model: null,
        hasRun: false,
        template: null,
      });

      const updated = get(tasks).find((t) => t.id === "t1");
      expect(updated?.status).toBe(TaskStatus.Waiting);
    });

    it("queueMergeTask calls update_task on backend", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { queueMergeTask, tasks } = await import("./tasks");
      tasks.set([makeTask({ id: "t1", column: TaskColumn.Merge })]);

      await queueMergeTask({
        id: "t1",
        description: "desc",
        usePlan: false,
        yolo: false,
        worktree: null,
        model: null,
        hasRun: false,
        template: null,
      });

      expect(mockInvoke).toHaveBeenCalledWith("update_task", expect.objectContaining({ task: expect.objectContaining({ status: TaskStatus.Waiting }) }));
    });

    it("cancelMergeWait removes task from queue", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const { queueMergeTask, cancelMergeWait, tasks } = await import("./tasks");
      tasks.set([makeTask({ id: "t1", column: TaskColumn.Merge })]);

      await queueMergeTask({
        id: "t1",
        description: "desc",
        usePlan: false,
        yolo: false,
        worktree: null,
        model: null,
        hasRun: false,
        template: null,
      });

      cancelMergeWait("t1");
      // After cancel, the task's waitingMergeSessions entry is removed.
      // We verify loadTasks no longer overrides status.
      mockInvoke.mockResolvedValueOnce([makeTask({ id: "t1", column: TaskColumn.Merge, status: TaskStatus.Idle })]);
      const { loadTasks } = await import("./tasks");
      await loadTasks();
      const found = get(tasks).find((t) => t.id === "t1");
      expect(found?.status).toBe(TaskStatus.Idle);
    });
  });

  describe("loadTasks", () => {
    it("preserves Running status for in-progress sessions", async () => {
      // Import tasks module fresh
      const { loadTasks, runClaudeSession, tasks } = await import("./tasks");

      // Set up task in store
      tasks.set([makeTask({ id: "s1", column: TaskColumn.InProgress, status: TaskStatus.Idle })]);

      // Mock listen to capture the event handler but not call it (session stays running)
      let capturedUnlisten: (() => void) | null = null;
      mockListen.mockImplementation((_event: string, _handler: unknown) => {
        return Promise.resolve(() => { capturedUnlisten = () => {}; });
      });
      mockInvoke.mockResolvedValue(undefined);

      // Start session (adds s1 to runningSessions)
      const sessionPromise = runClaudeSession("s1", "do stuff", false, false);
      await sessionPromise;

      // Now loadTasks returns stale Idle status from DB
      mockInvoke.mockResolvedValueOnce([
        makeTask({ id: "s1", column: TaskColumn.InProgress, status: TaskStatus.Idle }),
      ]);
      await loadTasks();

      // In-memory Running status should be preserved
      const found = get(tasks).find((t) => t.id === "s1");
      expect(found?.status).toBe(TaskStatus.Running);

      void capturedUnlisten;
    });

    it("sets tasks to empty array on backend error", async () => {
      const { loadTasks, tasks } = await import("./tasks");
      tasks.set([makeTask()]);
      mockInvoke.mockRejectedValueOnce(new Error("DB error"));

      await loadTasks();

      expect(get(tasks)).toEqual([]);
    });
  });
});
