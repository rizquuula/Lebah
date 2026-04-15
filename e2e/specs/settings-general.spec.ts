import { test, expect } from "../fixtures/test";
import { openSettings } from "../helpers/settings";

test.describe("Settings – General tab", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("saves claude path, worktree model and default toggles", async ({ appPage, mock }) => {
    await openSettings(appPage);

    await appPage.getByTestId("settings-claude-path").fill("/custom/claude");
    await appPage.getByTestId("settings-worktree-model").selectOption("sonnet");
    for (const tid of ["settings-default-plan", "settings-default-auto"]) {
      await appPage.getByTestId(tid).evaluate((el: HTMLInputElement) => {
        el.checked = true;
        el.dispatchEvent(new Event("change", { bubbles: true }));
      });
    }

    await appPage.getByTestId("settings-save-btn").click();

    const last = (await mock.lastInvoke("set_project_config")) as {
      payload: { config: { claude_path: string; worktree_model: string; default_use_plan: boolean; default_auto: boolean } };
    };
    expect(last.payload.config.claude_path).toBe("/custom/claude");
    expect(last.payload.config.worktree_model).toBe("sonnet");
    expect(last.payload.config.default_use_plan).toBe(true);
    expect(last.payload.config.default_auto).toBe(true);
  });
});
