<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/executionEvidence.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A run result is distinct from what was checked](../../../../../contract.md)

Captured SHA-256: `b816c12c112ead58615af6bb998d464fce740a5bde51b1dac057f32525ccf064`.

<book:entry path="apps/web/src/lib/executionEvidence.test.ts">
import { describe, expect, it } from "vitest";
import { executionEvidence } from "./executionEvidence";

// Guarantee: docs/guarantees/execution/a-run-result-is-distinct-from-what-was-checked.md
describe("execution evidence", () => {
  it("does not call a successful exit verified", () => {
    const success = executionEvidence("ok");
    expect(success.label).toContain("run succeeded");
    expect(success.label).not.toContain("verified");
    expect(success.detail).toContain("reported separately");
  });
  it("distinguishes a stale recording, no recording, and a live run", () => {
    expect(executionEvidence("stale").label).toBe("Stale recording");
    expect(executionEvidence().label).toContain("Unrecorded");
    expect(executionEvidence("ok", true).detail).toContain("no verdict yet");
  });
});
</book:entry>

</book:doc>
