import { test, expect, TaskColumn } from "../fixtures/test";
import { cardInColumn, moveTaskToInProgress } from "../helpers/task";

test.describe("Drag-drop / move across columns", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("moves a Todo task to InProgress via arrow button", async ({ appPage, mock }) => {
    await mock.seedTasks([{ description: "Arrow move" }]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");
    await expect(cardInColumn(appPage, TaskColumn.Todo, "Arrow move")).toBeVisible();

    await moveTaskToInProgress(appPage, "Arrow move");

    const last = (await mock.lastInvoke("move_task")) as {
      payload: { id: string; column: TaskColumn; sortOrder: number };
    };
    expect(last.payload.column).toBe(TaskColumn.InProgress);
    expect(last.payload.sortOrder).toBe(0);
  });

  test("moves InProgress->Review via arrow when task succeeded", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Review me", column: TaskColumn.InProgress, has_run: true, status: "Success" as never },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");
    const card = cardInColumn(appPage, TaskColumn.InProgress, "Review me");
    await card.getByTestId("task-move-review-btn").click();
    const last = (await mock.lastInvoke("move_task")) as {
      payload: { id: string; column: TaskColumn };
    };
    expect(last.payload.id).toBe(task.id);
    expect(last.payload.column).toBe(TaskColumn.Review);
  });
});
