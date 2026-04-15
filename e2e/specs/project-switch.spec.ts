import { test, expect } from "../fixtures/test";

test.describe("Project switching", () => {
  test("clicking Open Project invokes set_project_path and reloads tasks", async ({
    appPage,
    mock,
  }) => {
    await mock.setDialogReturn("/tmp/another-project");
    await appPage.getByTestId("open-project-btn").click();

    const setPath = (await mock.lastInvoke("set_project_path")) as { payload: { path: string } };
    expect(setPath.payload.path).toBe("/tmp/another-project");

    await expect(appPage.getByTestId("project-path-display")).toContainText("another-project");

    const getTasks = await mock.invoked("get_tasks");
    expect(getTasks.length).toBeGreaterThanOrEqual(1);
  });
});
