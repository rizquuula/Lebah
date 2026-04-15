import { expect, type Page } from "@playwright/test";
import type { MockHandle } from "../fixtures/test";
import { cardByDescription } from "./task";
import { TaskStatus } from "../fixtures/types";

export async function startSession(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-run-btn").click();
  await expect(card).toHaveAttribute("data-task-status", TaskStatus.Running);
}

export async function stopSession(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-run-btn").click();
}

export async function emitAssistantText(
  mock: MockHandle,
  taskId: string,
  text: string,
): Promise<void> {
  await mock.emitAssistantText(taskId, text);
}

export async function emitClaudeToolUse(
  mock: MockHandle,
  taskId: string,
  name: string,
  input: Record<string, unknown>,
): Promise<void> {
  await mock.emitClaudeOutput(taskId, {
    type: "assistant",
    message: {
      content: [{ type: "tool_use", id: "tu-1", name, input }],
      usage: { output_tokens: 1 },
    },
  });
}

export async function emitSuccessResult(mock: MockHandle, taskId: string): Promise<void> {
  await mock.emitClaudeResult(taskId, false);
}

export async function emitErrorResult(mock: MockHandle, taskId: string): Promise<void> {
  await mock.emitClaudeResult(taskId, true);
}

export async function openTerminal(page: Page, description: string): Promise<void> {
  const card = cardByDescription(page, description);
  await card.getByTestId("task-chat-btn").click();
  await expect(page.getByTestId("terminal-modal")).toBeVisible();
}

export async function closeTerminal(page: Page): Promise<void> {
  await page.getByTestId("terminal-close-btn").click();
  await expect(page.getByTestId("terminal-modal")).toBeHidden();
}

export async function waitForAssistantMessage(page: Page, text: string): Promise<void> {
  await expect(page.getByTestId("terminal-messages").getByText(text, { exact: false })).toBeVisible();
}

export async function sendTerminalInput(page: Page, text: string): Promise<void> {
  await page.getByTestId("terminal-input").fill(text);
  await page.getByTestId("terminal-send-btn").click();
}
