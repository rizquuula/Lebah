import { test, expect, TaskColumn } from "../fixtures/test";
import { editColumnTemplate } from "../helpers/settings";
import { cardInColumn } from "../helpers/task";

test.describe("Column templates", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("saving a Review template persists and is used by auto-advance", async ({ appPage, mock }) => {
    await editColumnTemplate(appPage, TaskColumn.Review, "CUSTOM_REVIEW_TEMPLATE");

    const last = (await mock.lastInvoke("set_project_config")) as {
      payload: { config: { review_template: string } };
    };
    expect(last.payload.config.review_template).toBe("CUSTOM_REVIEW_TEMPLATE");

    const [task] = await mock.seedTasks([
      { description: "Uses tpl", column: TaskColumn.InProgress, auto: true },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Uses tpl");
    await card.getByTestId("task-run-btn").click();
    await mock.emitClaudeResult(task.id, false);

    await expect(cardInColumn(appPage, TaskColumn.Review, "Uses tpl")).toBeVisible();
    const send = (await mock.lastInvoke("send_input")) as { payload: { input: string } };
    expect(send.payload.input).toBe("CUSTOM_REVIEW_TEMPLATE");
  });
});
