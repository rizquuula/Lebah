import { test, expect, TaskColumn, TaskStatus } from "../fixtures/test";
import { cardInColumn } from "../helpers/task";

test.describe("Agent session lifecycle", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("run → success transitions card to Success", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Run this", column: TaskColumn.InProgress },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Run this");
    await card.getByTestId("task-run-btn").click();

    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Running);
    const runInvoke = (await mock.lastInvoke("run_agent_session")) as { payload: { id: string } };
    expect(runInvoke.payload.id).toBe(task.id);

    await mock.emitClaudeResult(task.id, false);
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Success);
  });

  test("run → error marks card as Failed", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Will fail", column: TaskColumn.InProgress },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Will fail");
    await card.getByTestId("task-run-btn").click();
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Running);

    await mock.emitClaudeResult(task.id, true);
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Failed);
  });

  test("stop while running invokes stop_claude_session", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Stop me", column: TaskColumn.InProgress },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Stop me");
    await card.getByTestId("task-run-btn").click();
    await expect(card).toHaveAttribute("data-task-status", TaskStatus.Running);

    await card.getByTestId("task-run-btn").click();
    const last = (await mock.lastInvoke("stop_claude_session")) as { payload: { id: string } };
    expect(last.payload.id).toBe(task.id);
  });
});
