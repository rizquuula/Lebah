import { describe, it, expect, vi, beforeEach } from "vitest";
import { copyToClipboard } from "./clipboard";

describe("copyToClipboard", () => {
  beforeEach(() => {
    Object.assign(globalThis, {
      navigator: {
        clipboard: {
          writeText: vi.fn(),
        },
      },
    });
  });

  it("writes text to clipboard and returns true on success", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    (globalThis as any).navigator = { clipboard: { writeText } };

    const result = await copyToClipboard("/home/user/my-project");

    expect(writeText).toHaveBeenCalledWith("/home/user/my-project");
    expect(result).toBe(true);
  });

  it("returns false when clipboard write fails", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("Permission denied"));
    (globalThis as any).navigator = { clipboard: { writeText } };

    const result = await copyToClipboard("/some/path");

    expect(result).toBe(false);
  });

  it("returns false when navigator.clipboard is undefined", async () => {
    (globalThis as any).navigator = {};

    const result = await copyToClipboard("/some/path");

    expect(result).toBe(false);
  });
});
