<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/refactorSummary.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Refactor Baseline Reports The Moment An Output Would Move](../../../../../contract.md)

Captured SHA-256: `1938c3b7211c028d95dbb63f26e7d1da4744e0fac116fc8ca626f56d203f43fb`.

<book:entry path="apps/web/src/lib/refactorSummary.test.ts">
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
</book:entry>

</book:doc>
