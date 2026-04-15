import { test, expect, TaskColumn, TaskStatus } from "../fixtures/test";
import { cardByDescription, cardInColumn } from "../helpers/task";

test.describe("Auto-advance flow", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("InProgress success auto-advances to Review and sends review_template", async ({
    appPage,
    mock,
  }) => {
    const [task] = await mock.seedTasks([
      {
        description: "Auto advance",
        column: TaskColumn.InProgress,
        auto: true,
      },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Auto advance");
    await card.getByTestId("task-run-btn").click();
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Running);

    await mock.emitClaudeResult(task.id, false);

    const reviewCard = cardInColumn(appPage, TaskColumn.Review, "Auto advance");
    await expect(reviewCard).toBeVisible();

    const send = (await mock.lastInvoke("send_input")) as { payload: { id: string; input: string } };
    expect(send.payload.id).toBe(task.id);
    expect(send.payload.input).toContain("check for test");
  });
});
