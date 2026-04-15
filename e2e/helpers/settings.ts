import { expect, type Page } from "@playwright/test";
import { TaskColumn } from "../fixtures/types";
import { columnLocator } from "./task";

export async function openSettings(page: Page): Promise<void> {
  await page.getByTestId("settings-btn").click();
  await expect(page.getByTestId("settings-modal")).toBeVisible();
}

export async function closeSettings(page: Page, save: boolean = true): Promise<void> {
  if (save) {
    await page.getByTestId("settings-save-btn").click();
  } else {
    await page.getByTestId("settings-cancel-btn").click();
  }
  await expect(page.getByTestId("settings-modal")).toBeHidden();
}

export async function switchSettingsTab(
  page: Page,
  tab: "general" | "env" | "links",
): Promise<void> {
  await page.getByTestId(`settings-tab-${tab}`).click();
}

export async function addEnvVar(
  page: Page,
  key: string,
  value: string,
  enabled: boolean = true,
): Promise<void> {
  await page.getByTestId("env-add-btn").click();
  const rows = page.getByTestId("env-row");
  const last = rows.last();
  await last.getByTestId("env-key-input").fill(key);
  await last.getByTestId("env-value-input").fill(value);
  if (!enabled) {
    await last.getByTestId("env-toggle-btn").click();
  }
}

export async function removeEnvVarAt(page: Page, index: number): Promise<void> {
  const rows = page.getByTestId("env-row");
  await rows.nth(index).getByTestId("env-remove-btn").click();
}

export async function editColumnTemplate(
  page: Page,
  column: TaskColumn,
  template: string,
): Promise<void> {
  const col = columnLocator(page, column);
  await col.getByTestId("template-editor-btn").click();
  const popover = col.getByTestId("template-popover");
  await expect(popover).toBeVisible();
  await popover.getByTestId("template-textarea").fill(template);
  await popover.getByTestId("template-save-btn").click();
  await expect(popover).toBeHidden();
}
