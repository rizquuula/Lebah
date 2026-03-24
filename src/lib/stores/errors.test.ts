import { describe, it, expect } from "vitest";
import { get } from "svelte/store";
import { lastError, setError, clearError } from "./errors";

describe("errors store", () => {
  it("starts as null", () => {
    expect(get(lastError)).toBeNull();
  });

  it("setError sets the message", () => {
    setError("something went wrong");
    expect(get(lastError)).toBe("something went wrong");
  });

  it("clearError resets to null", () => {
    setError("oops");
    clearError();
    expect(get(lastError)).toBeNull();
  });
});
