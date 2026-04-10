import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { get } from "svelte/store";

describe("notifications store", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.resetModules();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts as null", async () => {
    const { notification } = await import("./notifications");
    expect(get(notification)).toBeNull();
  });

  it("showNotification sets the message", async () => {
    const { notification, showNotification } = await import("./notifications");
    showNotification("Settings saved.");
    expect(get(notification)).toBe("Settings saved.");
  });

  it("auto-dismisses after the given duration", async () => {
    const { notification, showNotification } = await import("./notifications");
    showNotification("Settings saved.", 3000);
    expect(get(notification)).toBe("Settings saved.");

    vi.advanceTimersByTime(3000);
    expect(get(notification)).toBeNull();
  });

  it("does not dismiss before the duration elapses", async () => {
    const { notification, showNotification } = await import("./notifications");
    showNotification("Settings saved.", 3000);

    vi.advanceTimersByTime(2999);
    expect(get(notification)).toBe("Settings saved.");
  });

  it("clearNotification resets to null immediately", async () => {
    const { notification, showNotification, clearNotification } = await import("./notifications");
    showNotification("Settings saved.");
    clearNotification();
    expect(get(notification)).toBeNull();
  });

  it("clearNotification cancels the auto-dismiss timer", async () => {
    const { notification, showNotification, clearNotification } = await import("./notifications");
    showNotification("Settings saved.", 3000);
    clearNotification();

    // After the original duration, nothing changes (already null, no error)
    vi.advanceTimersByTime(3000);
    expect(get(notification)).toBeNull();
  });

  it("calling showNotification again resets the timer", async () => {
    const { notification, showNotification } = await import("./notifications");
    showNotification("First", 3000);

    vi.advanceTimersByTime(2000);
    showNotification("Second", 3000);

    // At t=4000ms since first call (2000 past "Second"), still visible
    vi.advanceTimersByTime(2000);
    expect(get(notification)).toBe("Second");

    // At t=5000ms since first call (3000 past "Second"), should be gone
    vi.advanceTimersByTime(1000);
    expect(get(notification)).toBeNull();
  });

  it("uses 3000ms as the default duration", async () => {
    const { notification, showNotification } = await import("./notifications");
    showNotification("Settings saved.");

    vi.advanceTimersByTime(2999);
    expect(get(notification)).toBe("Settings saved.");

    vi.advanceTimersByTime(1);
    expect(get(notification)).toBeNull();
  });
});
