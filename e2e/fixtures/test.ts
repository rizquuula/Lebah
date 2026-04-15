import { test as base, type Page } from "@playwright/test";
import { TaskColumn, TaskStatus, type Task, type GitStatus, type ProjectConfig } from "./types";

type LebahTestApi = {
  reset(): void;
  getState(): unknown;
  seedTasks(tasks: Partial<Task>[]): Task[];
  seedProject(path: string, git?: GitStatus | null, config?: Partial<ProjectConfig>): void;
  setDialogReturn(path: string | null): void;
  setGitPushError(err: string | null): void;
  setGeneratedWorktreeName(name: string): void;
  emitClaudeOutput(id: string, payload: string | Record<string, unknown>): Promise<void>;
  emitClaudeResult(id: string, isError?: boolean): Promise<void>;
  emitAssistantText(id: string, text: string): Promise<void>;
  invoked(cmd?: string): Array<{ cmd: string; payload: unknown; at: number }>;
  lastInvoke(cmd: string): { cmd: string; payload: unknown; at: number } | undefined;
  clearInvoked(): void;
};

declare global {
  interface Window {
    __lebahTest: LebahTestApi;
  }
}

export interface MockHandle {
  reset(): Promise<void>;
  seedTasks(tasks: Partial<Task>[]): Promise<Task[]>;
  seedProject(path: string, git?: GitStatus | null, config?: Partial<ProjectConfig>): Promise<void>;
  setDialogReturn(path: string | null): Promise<void>;
  setGitPushError(err: string | null): Promise<void>;
  setGeneratedWorktreeName(name: string): Promise<void>;
  emitClaudeOutput(id: string, payload: string | Record<string, unknown>): Promise<void>;
  emitClaudeResult(id: string, isError?: boolean): Promise<void>;
  emitAssistantText(id: string, text: string): Promise<void>;
  invoked(cmd?: string): Promise<Array<{ cmd: string; payload: unknown; at: number }>>;
  lastInvoke(cmd: string): Promise<{ cmd: string; payload: unknown; at: number } | undefined>;
  clearInvoked(): Promise<void>;
  waitForInvoke(cmd: string, predicate?: (payload: unknown) => boolean, timeoutMs?: number): Promise<unknown>;
}

function makeMockHandle(page: Page): MockHandle {
  const wait = async <T>(fn: () => Promise<T>): Promise<T> => fn();
  return {
    async reset() {
      await page.evaluate(() => window.__lebahTest.reset());
    },
    async seedTasks(tasks) {
      return wait(() => page.evaluate((ts) => window.__lebahTest.seedTasks(ts), tasks));
    },
    async seedProject(path, git = null, config) {
      await page.evaluate(
        (args) =>
          window.__lebahTest.seedProject(
            args.path,
            args.git,
            args.config,
          ),
        { path, git: git ?? null, config: config ?? undefined },
      );
    },
    async setDialogReturn(path) {
      await page.evaluate((p) => window.__lebahTest.setDialogReturn(p), path);
    },
    async setGitPushError(err) {
      await page.evaluate((e) => window.__lebahTest.setGitPushError(e), err);
    },
    async setGeneratedWorktreeName(name) {
      await page.evaluate((n) => window.__lebahTest.setGeneratedWorktreeName(n), name);
    },
    async emitClaudeOutput(id, payload) {
      await page.evaluate(
        async (args) => {
          await window.__lebahTest.emitClaudeOutput(args.id, args.payload);
        },
        { id, payload },
      );
    },
    async emitClaudeResult(id, isError = false) {
      await page.evaluate(
        async (args) => {
          await window.__lebahTest.emitClaudeResult(args.id, args.isError);
        },
        { id, isError },
      );
    },
    async emitAssistantText(id, text) {
      await page.evaluate(
        async (args) => {
          await window.__lebahTest.emitAssistantText(args.id, args.text);
        },
        { id, text },
      );
    },
    async invoked(cmd) {
      return page.evaluate((c) => window.__lebahTest.invoked(c), cmd);
    },
    async lastInvoke(cmd) {
      return page.evaluate((c) => window.__lebahTest.lastInvoke(c), cmd);
    },
    async clearInvoked() {
      await page.evaluate(() => window.__lebahTest.clearInvoked());
    },
    async waitForInvoke(cmd, predicate, timeoutMs = 5000) {
      const result = await page.waitForFunction(
        (args) => {
          const list = window.__lebahTest.invoked(args.cmd);
          if (list.length === 0) return null;
          if (!args.predicateSrc) return list[list.length - 1];
          const fn = new Function("payload", `return (${args.predicateSrc})(payload);`);
          const match = list.reverse().find((r) => fn(r.payload));
          return match ?? null;
        },
        { cmd, predicateSrc: predicate ? predicate.toString() : null },
        { timeout: timeoutMs },
      );
      return result.jsonValue();
    },
  };
}

type Fixtures = {
  mock: MockHandle;
  appPage: Page;
};

export const test = base.extend<Fixtures>({
  appPage: async ({ page }, use) => {
    await page.goto("/?e2e=1");
    await page.waitForFunction(() => typeof window.__lebahTest !== "undefined");
    await use(page);
  },
  mock: async ({ appPage }, use) => {
    const handle = makeMockHandle(appPage);
    await handle.reset();
    await use(handle);
  },
});

export { expect } from "@playwright/test";
export { TaskColumn, TaskStatus };
export type { Task, GitStatus, ProjectConfig };
