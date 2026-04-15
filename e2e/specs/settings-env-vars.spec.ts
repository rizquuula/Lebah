import { test, expect } from "../fixtures/test";
import { openSettings, switchSettingsTab, addEnvVar } from "../helpers/settings";

test.describe("Settings – Environment variables", () => {
  test.beforeEach(async ({ mock }) => {
    await mock.seedProject("/tmp/e2e-project");
  });

  test("adds enabled + disabled vars and persists to config", async ({ appPage, mock }) => {
    await openSettings(appPage);
    await switchSettingsTab(appPage, "env");

    await addEnvVar(appPage, "ALPHA", "one", true);
    await addEnvVar(appPage, "BETA", "two", false);

    await appPage.getByTestId("settings-save-btn").click();

    const last = (await mock.lastInvoke("set_project_config")) as {
      payload: { config: { env_vars: Record<string, string>; disabled_env_var_keys: string[] | null } };
    };
    expect(last.payload.config.env_vars.ALPHA).toBe("one");
    expect(last.payload.config.env_vars.BETA).toBe("two");
    expect(last.payload.config.disabled_env_var_keys).toContain("BETA");
    expect(last.payload.config.disabled_env_var_keys).not.toContain("ALPHA");
  });
});
