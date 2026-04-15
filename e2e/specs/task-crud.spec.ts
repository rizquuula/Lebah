import { test, expect, TaskColumn } from "../fixtures/test";
import { cardInColumn, createTask, deleteTask } from "../helpers/task";

test.describe("Task CRUD", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("creates a task in the Todo column", async ({ appPage, mock }) => {
    await createTask(appPage, { description: "Write login form" });
    const state = (await mock.invoked("create_task")) as Array<{ payload: { description: string } }>;
    expect(state.length).toBe(1);
    expect(state[0].payload.description).toBe("Write login form");
  });

  test("edits a task agent and model", async ({ appPage, mock }) => {
    await createTask(appPage, { description: "Refactor auth" });
    const card = cardInColumn(appPage, TaskColumn.Todo, "Refactor auth");
    await card.getByTestId("task-edit-btn").click();
    const modal = appPage.getByTestId("task-modal");
    await expect(modal).toBeVisible();
    await modal.getByTestId("task-model-select").selectOption("opus");
    await modal.getByTestId("task-save-btn").click();
    await expect(modal).toBeHidden();
    const last = (await mock.lastInvoke("update_task")) as { payload: { task: { model: string } } };
    expect(last.payload.task.model).toBe("opus");
  });

  test("deletes a task after confirmation", async ({ appPage, mock }) => {
    await createTask(appPage, { description: "Delete me" });
    await deleteTask(appPage, "Delete me");
    const last = (await mock.lastInvoke("delete_task")) as { payload: { id: string } };
    expect(last.payload.id).toBeTruthy();
    await expect(cardInColumn(appPage, TaskColumn.Todo, "Delete me")).toHaveCount(0);
  });

  test("generates worktree name via button", async ({ appPage, mock }) => {
    await mock.setGeneratedWorktreeName("feat/ai-generated");
    const col = appPage.getByTestId(`column-${TaskColumn.Todo}`);
    await col.getByTestId("add-task-btn").click();
    const modal = appPage.getByTestId("task-modal");
    await modal.getByTestId("task-desc-input").fill("Something");
    await modal.getByTestId("task-generate-worktree-btn").click();
    await expect(modal.getByTestId("task-worktree-input")).toHaveValue("feat/ai-generated");
  });
});
