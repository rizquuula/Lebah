import { describe, it, expect } from "vitest";
import { CHAT_BUTTON_TITLE, CHAT_BUTTON_ACTIVE_CLASS } from "./task-card-constants";

describe("task card chat button constants", () => {
  it("has the correct tooltip title for the chat button", () => {
    expect(CHAT_BUTTON_TITLE).toBe("Agent Chat");
  });

  it("has the correct active CSS class for the chat button", () => {
    expect(CHAT_BUTTON_ACTIVE_CLASS).toBe("chat-btn");
  });
});
