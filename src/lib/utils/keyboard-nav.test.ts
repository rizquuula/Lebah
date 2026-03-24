import { describe, it, expect } from "vitest";
import { cycleNext, movePrev, moveNext, initialIndex } from "./keyboard-nav";

describe("cycleNext", () => {
  it("returns 0 when starting from -1", () => {
    expect(cycleNext(-1, 3)).toBe(0);
  });

  it("advances to next index", () => {
    expect(cycleNext(0, 3)).toBe(1);
    expect(cycleNext(1, 3)).toBe(2);
  });

  it("wraps around from last to first", () => {
    expect(cycleNext(2, 3)).toBe(0);
  });

  it("returns -1 for empty list", () => {
    expect(cycleNext(-1, 0)).toBe(-1);
    expect(cycleNext(0, 0)).toBe(-1);
  });

  it("works with single-item list", () => {
    expect(cycleNext(-1, 1)).toBe(0);
    expect(cycleNext(0, 1)).toBe(0);
  });
});

describe("moveNext", () => {
  it("advances index", () => {
    expect(moveNext(0, 3)).toBe(1);
    expect(moveNext(1, 3)).toBe(2);
  });

  it("clamps at last index", () => {
    expect(moveNext(2, 3)).toBe(2);
  });

  it("returns -1 for empty list", () => {
    expect(moveNext(0, 0)).toBe(-1);
  });
});

describe("movePrev", () => {
  it("decrements index", () => {
    expect(movePrev(2, 3)).toBe(1);
    expect(movePrev(1, 3)).toBe(0);
  });

  it("clamps at 0", () => {
    expect(movePrev(0, 3)).toBe(0);
  });
});

describe("initialIndex", () => {
  it("returns 0 for non-empty list", () => {
    expect(initialIndex(1)).toBe(0);
    expect(initialIndex(5)).toBe(0);
  });

  it("returns -1 for empty list", () => {
    expect(initialIndex(0)).toBe(-1);
  });
});
