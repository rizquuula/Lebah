import { test, expect, TaskColumn, TaskStatus } from "../fixtures/test";
import { cardInColumn } from "../helpers/task";

test.describe("Error handling", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("error banner can be dismissed", async ({ appPage, mock }) => {
    await mock.setGitPushError("boom");
    await mock.seedProject("/tmp/e2e-project", {
      branch: "main",
      ahead: 1,
      behind: 0,
      changed_files: 0,
    });
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    await appPage.getByTestId("git-push-btn").click();
    await appPage.getByTestId("confirm-dialog").getByTestId("confirm-ok-btn").click();

    const banner = appPage.getByTestId("error-banner");
    await expect(banner).toBeVisible();
    await banner.click();
    await expect(banner).toBeHidden();
  });

  test("session failure is reflected as Failed status", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Error task", column: TaskColumn.InProgress },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Error task");
    await card.getByTestId("task-run-btn").click();
    await mock.emitClaudeResult(task.id, true);
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Failed);
  });
});
