import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
  emit: vi.fn(),
}));
// tasks store is imported transitively; mock it to avoid full initialisation
vi.mock("./tasks", () => ({ loadTasks: vi.fn().mockResolvedValue(undefined) }));

import { projectPath, getRecentProjects, switchProject, openProject, removeRecentProject } from "./project";

beforeEach(() => {
  mockInvoke.mockReset();
  projectPath.set(null);
});

describe("getRecentProjects", () => {
  it("returns list from backend", async () => {
    mockInvoke.mockResolvedValue(["/a", "/b", "/c"]);
    const result = await getRecentProjects();
    expect(mockInvoke).toHaveBeenCalledWith("get_recent_projects", { maxCount: 10 });
    expect(result).toEqual(["/a", "/b", "/c"]);
  });

  it("propagates backend errors", async () => {
    mockInvoke.mockRejectedValue(new Error("backend error"));
    await expect(getRecentProjects()).rejects.toThrow("backend error");
  });
});

describe("openProject", () => {
  it("calls set_project_path and updates projectPath store", async () => {
    mockInvoke.mockResolvedValue(null); // set_project_path, get_git_status
    await openProject("/my/project");
    expect(mockInvoke).toHaveBeenCalledWith("set_project_path", { path: "/my/project" });
    expect(get(projectPath)).toBe("/my/project");
  });
});

describe("switchProject", () => {
  it("delegates to openProject and updates store", async () => {
    mockInvoke.mockResolvedValue(null);
    await switchProject("/switched/path");
    expect(get(projectPath)).toBe("/switched/path");
  });
});

describe("removeRecentProject", () => {
  it("calls remove_recent_project with the path", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await removeRecentProject("/old/project");
    expect(mockInvoke).toHaveBeenCalledWith("remove_recent_project", { path: "/old/project" });
  });

  it("propagates backend errors", async () => {
    mockInvoke.mockRejectedValue(new Error("backend error"));
    await expect(removeRecentProject("/some/path")).rejects.toThrow("backend error");
  });
});
