export enum TaskColumn {
  Todo = "Todo",
  InProgress = "InProgress",
  Review = "Review",
  Merge = "Merge",
  Completed = "Completed",
}

export enum TaskStatus {
  Idle = "Idle",
  Running = "Running",
  Success = "Success",
  Failed = "Failed",
  Warning = "Warning",
  Waiting = "Waiting",
  Canceled = "Canceled",
}

export interface Task {
  id: string;
  description: string;
  column: TaskColumn;
  status: TaskStatus;
  use_plan: boolean;
  yolo: boolean;
  auto: boolean;
  sort_order: number;
  created_at: string;
  completed_at: string | null;
  claude_path: string | null;
  worktree: string | null;
  has_run: boolean;
  model: string | null;
  lines_added: number | null;
  lines_removed: number | null;
}

export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  changed_files: number;
}

export const COLUMNS: { key: TaskColumn; label: string }[] = [
  { key: TaskColumn.Todo, label: "To-Do" },
  { key: TaskColumn.InProgress, label: "In Progress" },
  { key: TaskColumn.Review, label: "Review" },
  { key: TaskColumn.Merge, label: "Merge" },
  { key: TaskColumn.Completed, label: "Completed" },
];

export const COLUMN_COLORS: Record<TaskColumn, string> = {
  [TaskColumn.Todo]: "#89b4fa",
  [TaskColumn.InProgress]: "#f9e2af",
  [TaskColumn.Review]: "#cba6f7",
  [TaskColumn.Merge]: "#a6e3a1",
  [TaskColumn.Completed]: "#94e2d5",
};

export interface ProjectConfig {
  review_template: string | null;
  merge_template: string | null;
  inprogress_template: string | null;
  claude_path: string | null;
  worktree_model: string | null;
  default_use_plan: boolean | null;
  default_yolo: boolean | null;
  default_auto: boolean | null;
  env_vars: Record<string, string> | null;
  disabled_env_var_keys: string[] | null;
}

export const DEFAULT_REVIEW_TEMPLATE = "Help me to check for test, lint and build error if we not yet do it. Do git commit in the worktree, no need for reading changed files to commit, just use knowledge in session and commit all changes.";
export const DEFAULT_MERGE_TEMPLATE = "Pull from main branch in the local repository and check for conflict. Fix the conflict gracefully and run build after conflict resolution. Then merge this worktree to the main local branch. Make comprehensive tasks first before executing.";
export const DEFAULT_INPROGRESS_TEMPLATE = "Help me do this task:\n\n <TASK_DESCRIPTION> \n\nUse all best practices, no need to ask me. Make comprehensive tasks first before executing.";

export interface UsageInfo {
  input_tokens: number;
  output_tokens: number;
  cache_read_input_tokens: number;
  cache_creation_input_tokens: number;
}

export type ChatEntry =
  | { kind: "user"; text: string }
  | { kind: "assistant"; text: string }
  | { kind: "tool_use"; name: string; input: string }
  | { kind: "usage"; input: number; output: number; cacheRead: number; cacheCreate: number }
  | { kind: "file_output"; path: string; content: string }
  | { kind: "result"; success: boolean; cost: number; duration_ms: number; usage: UsageInfo }
  | { kind: "system"; text: string };

export const STATUS_COLORS: Record<TaskStatus, string> = {
  [TaskStatus.Idle]: "#6b7280",
  [TaskStatus.Running]: "#eab308",
  [TaskStatus.Success]: "#22c55e",
  [TaskStatus.Failed]: "#ef4444",
  [TaskStatus.Warning]: "#f97316",
  [TaskStatus.Waiting]: "#3b82f6",
  [TaskStatus.Canceled]: "#6b7280",
};
