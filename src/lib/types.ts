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
}

export interface Task {
  id: string;
  description: string;
  column: TaskColumn;
  status: TaskStatus;
  use_plan: boolean;
  yolo: boolean;
  sort_order: number;
  created_at: string;
  completed_at: string | null;
  claude_path: string | null;
  worktree: string | null;
  has_run: boolean;
  model: string | null;
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
}

export const DEFAULT_REVIEW_TEMPLATE = "Help me to check for test, lint and build error if you not yet do it. Then do commit in the worktree.";
export const DEFAULT_MERGE_TEMPLATE = "Pull from main branch and check for conflict. Fix the conflict. Then merge this worktree to main.";
export const DEFAULT_INPROGRESS_TEMPLATE = "Help me do this task: <TASK_DESCRIPTION>. Make comprehensive tasks first before executing.";

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
};
