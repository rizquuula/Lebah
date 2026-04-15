import { test, expect } from "../fixtures/test";

test.describe("Git status header", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject(
      "/tmp/e2e-project",
      { branch: "main", ahead: 2, behind: 1, changed_files: 3 },
    );
  });

  test("renders branch, ahead, behind, and changed badges", async ({ appPage }) => {
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    await expect(appPage.getByTestId("git-branch")).toHaveText("main");
    await expect(appPage.getByTestId("git-ahead")).toContainText("2");
    await expect(appPage.getByTestId("git-behind")).toContainText("1");
    await expect(appPage.getByTestId("git-changes")).toContainText("3");
  });

  test("push button opens confirm dialog and invokes git_push on confirm", async ({ appPage, mock }) => {
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    await appPage.getByTestId("git-push-btn").click();
    const dialog = appPage.getByTestId("confirm-dialog");
    await expect(dialog).toBeVisible();
    await dialog.getByTestId("confirm-ok-btn").click();
    await expect(dialog).toBeHidden();

    const invoked = await mock.invoked("git_push");
    expect(invoked.length).toBe(1);
  });

  test("push error surfaces in error banner", async ({ appPage, mock }) => {
    await mock.setGitPushError("remote rejected");
    await appPage.reload();
    await appPage.waitForFunction(() => typeof window.__lebahTest !== "undefined");

    await appPage.getByTestId("git-push-btn").click();
    await appPage.getByTestId("confirm-dialog").getByTestId("confirm-ok-btn").click();

    const banner = appPage.getByTestId("error-banner");
    await expect(banner).toBeVisible();
    await expect(banner).toContainText("Push failed");
  });
});
