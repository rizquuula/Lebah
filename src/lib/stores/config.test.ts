import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));

const NULL_CONFIG = {
  review_template: null,
  merge_template: null,
  inprogress_template: null,
  claude_path: null,
  opencode_path: null,
  worktree_model: null,
  default_use_plan: null,
  default_yolo: null,
  default_auto: null,
  env_vars: null,
  disabled_env_var_keys: null,
  worktree_links: null,
};

describe("config store", () => {
  beforeEach(() => {
    vi.resetModules();
    mockInvoke.mockReset();
  });

  it("loadProjectConfig sets store from backend", async () => {
    const cfg = { ...NULL_CONFIG, review_template: "my review" };
    mockInvoke.mockResolvedValueOnce(cfg);

    const { loadProjectConfig, projectConfig } = await import("./config");
    await loadProjectConfig();

    expect(get(projectConfig).review_template).toBe("my review");
  });

  it("loadProjectConfig falls back to null config on error", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("backend error"));

    const { loadProjectConfig, projectConfig } = await import("./config");
    await loadProjectConfig();

    expect(get(projectConfig)).toEqual(NULL_CONFIG);
  });

  it("saveProjectConfig invokes backend and updates store", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const cfg = { ...NULL_CONFIG, merge_template: "merge!" };

    const { saveProjectConfig, projectConfig } = await import("./config");
    await saveProjectConfig(cfg);

    expect(mockInvoke).toHaveBeenCalledWith("set_project_config", { config: cfg });
    expect(get(projectConfig).merge_template).toBe("merge!");
  });

  it("loadProjectConfig preserves worktree_links from backend", async () => {
    const cfg = { ...NULL_CONFIG, worktree_links: ["node_modules", ".env"] };
    mockInvoke.mockResolvedValueOnce(cfg);

    const { loadProjectConfig, projectConfig } = await import("./config");
    await loadProjectConfig();

    expect(get(projectConfig).worktree_links).toEqual(["node_modules", ".env"]);
  });

  it("saveProjectConfig passes worktree_links to backend", async () => {
    mockInvoke.mockResolvedValue(undefined);
    const cfg = { ...NULL_CONFIG, worktree_links: ["target", ".env"] };

    const { saveProjectConfig, projectConfig } = await import("./config");
    await saveProjectConfig(cfg);

    expect(mockInvoke).toHaveBeenCalledWith("set_project_config", { config: cfg });
    expect(get(projectConfig).worktree_links).toEqual(["target", ".env"]);
  });

  it("NULL_CONFIG has worktree_links as null", () => {
    expect(NULL_CONFIG.worktree_links).toBeNull();
  });
});
