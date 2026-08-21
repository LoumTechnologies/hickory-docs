import { afterEach, describe, expect, it, vi } from "vitest";
import { claimReveal, onRevealLine, resetReveals, revealLine } from "./revealLine";

afterEach(() => {
  resetReveals();
});

describe("asking for a file at a line", () => {
  it("remembers the request for a pane that does not exist yet", () => {
    // A find hit opens a file; the editor for it is built a tick or two
    // later. An event alone would be shouted into an empty room.
    revealLine("src/main.rs", 42);
    expect(claimReveal("src/main.rs")).toBe(42);
  });

  it("is consumed by the first claim, so it does not fire twice", () => {
    // Re-answering on the next re-render would fight the reader for the
    // cursor.
    revealLine("a.txt", 3);
    expect(claimReveal("a.txt")).toBe(3);
    expect(claimReveal("a.txt")).toBeNull();
  });

  it("keeps the newest request for a file, because that is the one clicked", () => {
    revealLine("a.txt", 3);
    revealLine("a.txt", 90);
    expect(claimReveal("a.txt")).toBe(90);
  });

  it("answers nothing for a file nobody asked about", () => {
    expect(claimReveal("never.txt")).toBeNull();
  });

  it("clamps a line number that could not be scrolled to", () => {
    revealLine("a.txt", 0);
    expect(claimReveal("a.txt")).toBe(1);
    revealLine("b.txt", -5);
    expect(claimReveal("b.txt")).toBe(1);
  });

  it("tells listeners which path was asked for", () => {
    const heard = vi.fn();
    const off = onRevealLine(heard);
    revealLine("src/lib.rs", 7);
    expect(heard).toHaveBeenCalledWith("src/lib.rs");
    off();
    revealLine("src/lib.rs", 8);
    expect(heard).toHaveBeenCalledTimes(1);
  });
});
