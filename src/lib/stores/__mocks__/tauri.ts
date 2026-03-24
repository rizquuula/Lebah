// Shared mock helpers for Tauri APIs
import { vi } from "vitest";

export const mockInvoke = vi.fn();
export const mockListen = vi.fn().mockResolvedValue(() => {});
export const mockEmit = vi.fn();
