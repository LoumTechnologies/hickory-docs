import { describe, expect, it } from "vitest";

import { nextInQueue, stillWaiting } from "./attentionCursor";

describe("nextInQueue — where ⌘J lands", () => {
  it("starts at the front when nothing is selected", () => {
    expect(nextInQueue(["a", "b", "c"], null)).toBe("a");
  });

  it("walks in the server's order", () => {
    expect(nextInQueue(["a", "b", "c"], "a")).toBe("b");
    expect(nextInQueue(["a", "b", "c"], "b")).toBe("c");
  });

  it("wraps, so the queue can be walked round without hunting for the end", () => {
    expect(nextInQueue(["a", "b", "c"], "c")).toBe("a");
  });

  it("returns to the front when the session it was on has left the queue", () => {
    // Answered between two presses: an index would have pointed at whoever
    // shuffled into that slot.
    expect(nextInQueue(["b", "c"], "a")).toBe("b");
  });

  it("says nothing is waiting rather than pretending to move", () => {
    expect(nextInQueue([], "a")).toBeNull();
    expect(nextInQueue([], null)).toBeNull();
  });
});

describe("stillWaiting", () => {
  it("is false once the session has been answered or closed", () => {
    expect(stillWaiting(["a", "b"], "a")).toBe(true);
    expect(stillWaiting(["b"], "a")).toBe(false);
    expect(stillWaiting(["a"], null)).toBe(false);
  });
});
