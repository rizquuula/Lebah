export type TaskColumn = "Todo" | "InProgress" | "Review" | "Merge" | "Completed";
export type TaskStatus = "Idle" | "Running" | "Success" | "Failed" | "Warning";

export interface Task {
  id: string;
  description: string;
  column: TaskColumn;
  status: TaskStatus;
  use_plan: boolean;
  sort_order: number;
  created_at: string;
}

export const COLUMNS: { key: TaskColumn; label: string }[] = [
  { key: "Todo", label: "To-Do" },
  { key: "InProgress", label: "In Progress" },
  { key: "Review", label: "Review" },
  { key: "Merge", label: "Merge" },
  { key: "Completed", label: "Completed" },
];

export const STATUS_COLORS: Record<TaskStatus, string> = {
  Idle: "#6b7280",
  Running: "#eab308",
  Success: "#22c55e",
  Failed: "#ef4444",
  Warning: "#f97316",
};
