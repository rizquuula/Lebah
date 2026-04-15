import { test, expect, TaskColumn, TaskStatus } from "../fixtures/test";
import { cardInColumn } from "../helpers/task";

test.describe("Merge queue", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("second merge task waits while first is running", async ({ appPage, mock }) => {
    const [first, second] = await mock.seedTasks([
      { description: "Merge first", column: TaskColumn.Merge, has_run: true, auto: true },
      { description: "Merge second", column: TaskColumn.Merge, has_run: true, auto: true },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const firstCard = cardInColumn(appPage, TaskColumn.Merge, "Merge first");
    const secondCard = cardInColumn(appPage, TaskColumn.Merge, "Merge second");

    await firstCard.getByTestId("task-run-btn").click();
    await expect(firstCard).toHaveAttribute("data-task-status", TaskStatus.Running);

    await secondCard.getByTestId("task-run-btn").click();
    await expect(secondCard).toHaveAttribute("data-task-status", TaskStatus.Waiting);

    await mock.emitClaudeResult(first.id, false);
    await expect(cardInColumn(appPage, TaskColumn.Completed, "Merge first")).toBeVisible();
    await expect(secondCard).toHaveAttribute("data-task-status", TaskStatus.Running);

    await mock.emitClaudeResult(second.id, false);
    await expect(cardInColumn(appPage, TaskColumn.Completed, "Merge second")).toBeVisible();
  });
});
