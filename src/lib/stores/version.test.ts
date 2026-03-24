import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: mockInvoke }));

describe("version store", () => {
  beforeEach(() => {
    vi.resetModules();
    mockInvoke.mockReset();
  });

  it("starts as empty string", async () => {
    const { appVersion } = await import("./version");
    expect(get(appVersion)).toBe("");
  });

  it("loadAppVersion sets version from backend", async () => {
    mockInvoke.mockResolvedValueOnce("0.1.0");

    const { loadAppVersion, appVersion } = await import("./version");
    await loadAppVersion();

    expect(get(appVersion)).toBe("0.1.0");
    expect(mockInvoke).toHaveBeenCalledWith("get_app_version");
  });

  it("loadAppVersion falls back to empty string on error", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("tauri error"));

    const { loadAppVersion, appVersion } = await import("./version");
    await loadAppVersion();

    expect(get(appVersion)).toBe("");
  });
});
