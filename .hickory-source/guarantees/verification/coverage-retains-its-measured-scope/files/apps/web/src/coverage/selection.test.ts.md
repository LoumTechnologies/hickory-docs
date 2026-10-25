<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/coverage/selection.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../../contract.md)

Captured SHA-256: `fc8b1bd445b5097089dabab5b0e526dcb1b43ea3158a0fa46e7a3f906e6185e2`.

<book:entry path="apps/web/src/coverage/selection.test.ts">
import { describe, it, expect } from "vitest";
import { coveredLocations } from "./selection";
import type { CoverageRun } from "./types";
// Guarantee: docs/guarantees/verification/coverage-retains-its-measured-scope.md
describe("coverage source navigation", () => {
  it("maps only executable lines and preserves hit counts and original coordinates", () => {
    const run = { evidence: { report: { files: { "app.py": { lines: { 1: 1, 3: 0 }, line_counts: false } } } }, mappings: { "app.py": { 1: { path: "app.md", line: 5 }, 3: { path: "app.md", line: 7 } } } } as unknown as CoverageRun;
    expect(coveredLocations(run, "app.md")).toEqual([
      { line: 5, count: 1, countsAvailable: false, originalPath: "app.py", originalLine: 1 },
      { line: 7, count: 0, countsAvailable: false, originalPath: "app.py", originalLine: 3 },
    ]);
    expect(coveredLocations(run, "other.py")).toEqual([]);
  });
});
</book:entry>

</book:doc>
