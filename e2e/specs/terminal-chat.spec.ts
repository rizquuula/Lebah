import { test, expect, TaskColumn } from "../fixtures/test";
import { cardInColumn } from "../helpers/task";

test.describe("Terminal chat modal", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("shows assistant text streamed from events", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Chat task", column: TaskColumn.InProgress, has_run: true },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Chat task");
    await card.getByTestId("task-chat-btn").click();
    await expect(appPage.getByTestId("terminal-modal")).toBeVisible();

    await mock.emitAssistantText(task.id, "Hello from the agent");
    await expect(
      appPage.getByTestId("terminal-messages").getByText("Hello from the agent"),
    ).toBeVisible();
  });

  test("sending input calls send_input and closes modal", async ({ appPage, mock }) => {
    const [task] = await mock.seedTasks([
      { description: "Send input", column: TaskColumn.InProgress, has_run: true },
    ]);
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    const card = cardInColumn(appPage, TaskColumn.InProgress, "Send input");
    await card.getByTestId("task-chat-btn").click();
    await expect(appPage.getByTestId("terminal-modal")).toBeVisible();

    await appPage.getByTestId("terminal-input").fill("follow up please");
    await appPage.getByTestId("terminal-send-btn").click();

    const last = (await mock.lastInvoke("send_input")) as {
      payload: { id: string; input: string };
    };
    expect(last.payload.id).toBe(task.id);
    expect(last.payload.input).toBe("follow up please");

    await appPage.getByTestId("terminal-close-btn").click();
    await expect(appPage.getByTestId("terminal-modal")).toBeHidden();
  });
});
