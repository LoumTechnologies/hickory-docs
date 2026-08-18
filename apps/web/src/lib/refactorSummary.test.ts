// The refactor badge's words. Protects the reporting half of
// docs/guarantees/authoring/a-refactor-baseline-reports-drift.md.

import { describe, expect, it } from "vitest";

import { refactorDetail, refactorSummary } from "./refactorSummary";

describe("refactorSummary", () => {
  it("says nothing while no baseline is pinned", () => {
    expect(refactorSummary(null)).toBeNull();
    expect(refactorSummary({ active: false })).toBeNull();
  });

  it("reports a clean baseline and counts the outputs that moved", () => {
    expect(
      refactorSummary({ active: true, started_at: "t", clean: true, diffs: [] }),
    ).toBe("Outputs match baseline");
    expect(
      refactorSummary({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [{ path: "a.py", kind: "changed" }],
      }),
    ).toBe("1 output differs");
    expect(
      refactorSummary({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [
          { path: "a.py", kind: "changed" },
          { path: "b.txt", kind: "removed" },
        ],
      }),
    ).toBe("2 outputs differ");
  });

  it("details name each output and how it moved", () => {
    expect(refactorDetail({ active: true, started_at: "t", clean: true, diffs: [] })).toBe("");
    expect(
      refactorDetail({
        active: true,
        started_at: "t",
        clean: false,
        diffs: [
          { path: "a.py", kind: "changed" },
          { path: "new.txt", kind: "added" },
        ],
      }),
    ).toBe("a.py (changed), new.txt (added)");
  });
});
