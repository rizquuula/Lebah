import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { emit as tauriEmit } from "@tauri-apps/api/event";
import { TaskColumn, TaskStatus, type Task, type GitStatus, type ProjectConfig } from "../types";

interface InvokeRecord {
  cmd: string;
  payload: unknown;
  at: number;
}

interface MockState {
  projectPath: string | null;
  recentProjects: string[];
  gitStatus: GitStatus | null;
  projectConfig: ProjectConfig;
  tasks: Task[];
  outputBuffers: Record<string, string[]>;
  appVersion: string;
  nextDialogReturn: string | null;
  gitPushError: string | null;
  generatedWorktreeName: string;
  invoked: InvokeRecord[];
  unknownCommands: string[];
  sortCounter: number;
}

function defaultConfig(): ProjectConfig {
  return {
    review_template: null,
    merge_template: null,
    inprogress_template: null,
    claude_path: null,
    opencode_path: null,
    worktree_model: null,
    default_use_plan: false,
    default_yolo: true,
    default_auto: false,
    env_vars: { IS_SANDBOX: "0" },
    disabled_env_var_keys: null,
    worktree_links: null,
  };
}

function freshState(): MockState {
  return {
    projectPath: null,
    recentProjects: [],
    gitStatus: null,
    projectConfig: defaultConfig(),
    tasks: [],
    outputBuffers: {},
    appVersion: "0.0.0-e2e",
    nextDialogReturn: null,
    gitPushError: null,
    generatedWorktreeName: "e2e-generated-worktree",
    invoked: [],
    unknownCommands: [],
    sortCounter: 0,
  };
}

const STORAGE_KEY = "__lebahMockState";

let state: MockState = freshState();

function persist(): void {
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {}
}

function loadPersisted(): MockState | null {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    return JSON.parse(raw) as MockState;
  } catch {
    return null;
  }
}

function clearPersisted(): void {
  try {
    sessionStorage.removeItem(STORAGE_KEY);
  } catch {}
}

function record(cmd: string, payload: unknown): void {
  state.invoked.push({ cmd, payload, at: Date.now() });
  persist();
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v));
}

function makeTask(partial: Partial<Task> & { description: string }): Task {
  return {
    id: crypto.randomUUID(),
    description: partial.description,
    column: partial.column ?? TaskColumn.Todo,
    status: partial.status ?? TaskStatus.Idle,
    use_plan: partial.use_plan ?? state.projectConfig.default_use_plan ?? false,
    yolo: partial.yolo ?? state.projectConfig.default_yolo ?? true,
    auto: partial.auto ?? state.projectConfig.default_auto ?? false,
    sort_order: partial.sort_order ?? state.sortCounter++,
    created_at: partial.created_at ?? new Date().toISOString(),
    completed_at: partial.completed_at ?? null,
    claude_path: partial.claude_path ?? null,
    worktree: partial.worktree ?? null,
    has_run: partial.has_run ?? false,
    model: partial.model ?? null,
    agent_name: partial.agent_name ?? "claude",
    lines_added: partial.lines_added ?? null,
    lines_removed: partial.lines_removed ?? null,
  };
}

function handle(cmd: string, payload: Record<string, unknown> | undefined): unknown {
  record(cmd, payload);
  const p = (payload ?? {}) as Record<string, unknown>;
  switch (cmd) {
    case "get_app_version":
      return state.appVersion;
    case "get_project_path":
      return state.projectPath;
    case "set_project_path":
      state.projectPath = p.path as string;
      if (!state.recentProjects.includes(p.path as string)) {
        state.recentProjects.unshift(p.path as string);
      }
      return null;
    case "get_recent_projects":
      return state.recentProjects.slice();
    case "remove_recent_project":
      state.recentProjects = state.recentProjects.filter((x) => x !== p.path);
      return null;
    case "get_git_status":
      if (!state.gitStatus) throw "no git status";
      return clone(state.gitStatus);
    case "git_push":
      if (state.gitPushError) throw state.gitPushError;
      return "pushed";
    case "get_project_config":
      return clone(state.projectConfig);
    case "set_project_config":
      state.projectConfig = clone(p.config as ProjectConfig);
      return null;
    case "get_tasks":
      return state.tasks.map(clone);
    case "create_task": {
      const task = makeTask({
        description: p.description as string,
        worktree: (p.worktree as string | null) ?? null,
        model: (p.model as string | null) ?? null,
        agent_name: (p.agentName as string | null) ?? "claude",
        claude_path: (p.claudePath as string | null) ?? null,
      });
      state.tasks.push(task);
      return clone(task);
    }
    case "update_task": {
      const incoming = p.task as Task;
      const idx = state.tasks.findIndex((t) => t.id === incoming.id);
      if (idx >= 0) state.tasks[idx] = clone(incoming);
      return null;
    }
    case "delete_task":
      state.tasks = state.tasks.filter((t) => t.id !== p.id);
      return null;
    case "move_task": {
      const idx = state.tasks.findIndex((t) => t.id === p.id);
      if (idx >= 0) {
        state.tasks[idx] = {
          ...state.tasks[idx],
          column: p.column as TaskColumn,
          sort_order: p.sortOrder as number,
          completed_at: p.column === TaskColumn.Completed ? new Date().toISOString() : state.tasks[idx].completed_at,
        };
      }
      return null;
    }
    case "run_agent_session": {
      const id = p.id as string;
      const idx = state.tasks.findIndex((t) => t.id === id);
      if (idx >= 0) {
        state.tasks[idx] = { ...state.tasks[idx], status: TaskStatus.Running, has_run: true };
      }
      if (!state.outputBuffers[id]) state.outputBuffers[id] = [];
      state.outputBuffers[id].push(
        JSON.stringify({ type: "system", subtype: "init", model: p.model ?? "sonnet" }),
      );
      return null;
    }
    case "stop_claude_session": {
      const id = p.id as string;
      const idx = state.tasks.findIndex((t) => t.id === id);
      if (idx >= 0) state.tasks[idx] = { ...state.tasks[idx], status: TaskStatus.Canceled };
      return null;
    }
    case "send_input": {
      const id = p.id as string;
      const buf = state.outputBuffers[id] ?? (state.outputBuffers[id] = []);
      buf.push(JSON.stringify({ type: "user_input", text: p.input }));
      const idx = state.tasks.findIndex((t) => t.id === id);
      if (idx >= 0) state.tasks[idx] = { ...state.tasks[idx], status: TaskStatus.Running, has_run: true };
      return null;
    }
    case "get_output_buffer":
      return (state.outputBuffers[p.id as string] ?? []).slice();
    case "reset_task_session": {
      const id = p.id as string;
      const idx = state.tasks.findIndex((t) => t.id === id);
      if (idx < 0) throw "task not found";
      state.tasks[idx] = { ...state.tasks[idx], status: TaskStatus.Idle, has_run: false };
      state.outputBuffers[id] = [];
      return clone(state.tasks[idx]);
    }
    case "generate_worktree_name":
      return state.generatedWorktreeName;
    case "list_terminal_sessions":
      return [];
    case "create_terminal_session":
      return { id: "tsess-e2e", cols: p.cols ?? 80, rows: p.rows ?? 24 };
    case "resize_terminal":
    case "write_terminal":
    case "close_terminal_session":
      return null;
    case "plugin:dialog|open": {
      const r = state.nextDialogReturn;
      state.nextDialogReturn = null;
      return r;
    }
    default:
      if (!state.unknownCommands.includes(cmd)) state.unknownCommands.push(cmd);
      return null;
  }
}

interface LebahTestApi {
  reset(): void;
  getState(): MockState;
  seedTasks(tasks: Partial<Task>[]): Task[];
  seedProject(path: string, git?: GitStatus | null, config?: Partial<ProjectConfig>): void;
  setDialogReturn(path: string | null): void;
  setGitPushError(err: string | null): void;
  setGeneratedWorktreeName(name: string): void;
  emitClaudeOutput(id: string, payload: string | Record<string, unknown>): Promise<void>;
  emitClaudeResult(id: string, isError?: boolean): Promise<void>;
  emitAssistantText(id: string, text: string): Promise<void>;
  invoked(cmd?: string): InvokeRecord[];
  lastInvoke(cmd: string): InvokeRecord | undefined;
  clearInvoked(): void;
}

export function installMockIpc(): void {
  clearMocks();
  const persisted = loadPersisted();
  state = persisted ?? freshState();
  mockIPC((cmd, payload) => handle(cmd, payload as Record<string, unknown>), {
    shouldMockEvents: true,
  });

  const api: LebahTestApi = {
    reset() {
      clearMocks();
      state = freshState();
      clearPersisted();
      mockIPC((cmd, payload) => handle(cmd, payload as Record<string, unknown>), {
        shouldMockEvents: true,
      });
    },
    getState() {
      return clone(state);
    },
    seedTasks(tasks) {
      const created = tasks.map((t) => makeTask({ description: "", ...t }));
      state.tasks.push(...created);
      persist();
      return created.map(clone);
    },
    seedProject(path, git = null, config) {
      state.projectPath = path;
      if (!state.recentProjects.includes(path)) state.recentProjects.unshift(path);
      state.gitStatus = git;
      if (config) state.projectConfig = { ...state.projectConfig, ...config };
      persist();
    },
    setDialogReturn(path) {
      state.nextDialogReturn = path;
      persist();
    },
    setGitPushError(err) {
      state.gitPushError = err;
      persist();
    },
    setGeneratedWorktreeName(name) {
      state.generatedWorktreeName = name;
      persist();
    },
    async emitClaudeOutput(id, payload) {
      const s = typeof payload === "string" ? payload : JSON.stringify(payload);
      (state.outputBuffers[id] ??= []).push(s);
      await tauriEmit(`claude-output-${id}`, s);
    },
    async emitClaudeResult(id, isError = false) {
      await api.emitClaudeOutput(id, {
        type: "result",
        is_error: isError,
        usage: {
          input_tokens: 10,
          output_tokens: 20,
          cache_read_input_tokens: 0,
          cache_creation_input_tokens: 0,
        },
        total_cost_usd: 0.001,
        duration_ms: 42,
      });
    },
    async emitAssistantText(id, text) {
      await api.emitClaudeOutput(id, {
        type: "assistant",
        message: { content: [{ type: "text", text }], usage: { output_tokens: 1 } },
      });
    },
    invoked(cmd) {
      return cmd ? state.invoked.filter((r) => r.cmd === cmd) : state.invoked.slice();
    },
    lastInvoke(cmd) {
      const arr = state.invoked.filter((r) => r.cmd === cmd);
      return arr[arr.length - 1];
    },
    clearInvoked() {
      state.invoked = [];
      persist();
    },
  };

  (window as unknown as { __lebahTest: LebahTestApi }).__lebahTest = api;
}
