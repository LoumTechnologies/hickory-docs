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
