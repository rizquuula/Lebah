import { expect, type Page, type Locator } from "@playwright/test";
import { TaskColumn } from "../fixtures/types";

export function columnLocator(page: Page, column: TaskColumn): Locator {
  return page.getByTestId(`column-${column}`);
}

export function cardByDescription(page: Page, description: string): Locator {
  return page
    .getByTestId("task-card")
    .filter({ has: page.getByTestId("task-description").getByText(description, { exact: true }) });
}

export function cardInColumn(page: Page, column: TaskColumn, description: string): Locator {
  return columnLocator(page, column)
    .getByTestId("task-card")
    .filter({ has: page.getByTestId("task-description").getByText(description, { exact: true }) });
}

export async function createTask(
  page: Page,
  opts: {
    description: string;
    column?: TaskColumn;
    agent?: "claude" | "opencode";
    model?: string;
    worktree?: string;
  },
): Promise<void> {
  const col = opts.column ?? TaskColumn.Todo;
  await columnLocator(page, col).getByTestId("add-task-btn").click();
  const modal = page.getByTestId("task-modal");
  await expect(modal).toBeVisible();
  await modal.getByTestId("task-desc-input").fill(opts.description);
  if (opts.agent) {
    await modal.getByTestId("task-agent-select").selectOption(opts.agent);
  }
  if (opts.model) {
    const sel = modal.getByTestId("task-model-select");
    if (await sel.count()) await sel.selectOption(opts.model);
    else await modal.getByTestId("task-model-input").fill(opts.model);
  }
  if (opts.worktree !== undefined) {
    await modal.getByTestId("task-worktree-input").fill(opts.worktree);
  }
  await modal.getByTestId("task-save-btn").click();
  await expect(modal).toBeHidden();
  await expect(cardInColumn(page, col, opts.description)).toBeVisible();
}

export async function deleteTask(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-delete-btn").click();
  const dialog = page.getByTestId("confirm-dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByTestId("confirm-ok-btn").click();
  await expect(dialog).toBeHidden();
}

export async function moveTaskToInProgress(page: Page, description: string): Promise<void> {
  const card = cardInColumn(page, TaskColumn.Todo, description);
  await card.getByTestId("task-move-next-btn").click();
  await expect(cardInColumn(page, TaskColumn.InProgress, description)).toBeVisible();
}

export async function openTaskDetail(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-detail-btn").click();
}

export async function runTask(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-run-btn").click();
}

export async function toggleAuto(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("toggle-auto").click({ force: true });
}
