<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/revealLine.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Find And Replace Is Exhaustive, And Refuses What It Would Undo](../../../../../contract.md)

Captured SHA-256: `bfb635393fde61c9f2ac994687381fe6a362a7b6d077b83fca09d1c775bb512d`.

<book:entry path="apps/web/src/lib/revealLine.test.ts">
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
</book:entry>

</book:doc>
