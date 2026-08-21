import { describe, expect, it } from "vitest";
import {
  SEVERITY_HINT,
  SEVERITY_INFO,
  SEVERITY_WARNING,
  countProblems,
  problemsLabel,
  severityOf,
  totalProblems,
} from "./problems";

const d = (severity?: number) => ({ severity });

describe("what counts as a problem", () => {
  it("counts errors and warnings", () => {
    expect(countProblems([d(1), d(1), d(SEVERITY_WARNING)])).toEqual({
      errors: 2,
      warnings: 1,
    });
  });

  it("leaves information and hints out of the tally", () => {
    // A "this could be simplified" in the warning count is how a status bar
    // reaches 400 and stops being read. They still show in the editor.
    expect(countProblems([d(SEVERITY_INFO), d(SEVERITY_HINT)])).toEqual({
      errors: 0,
      warnings: 0,
    });
  });

  it("reads a missing severity as an error", () => {
    // The specification leaves it to the client. A server that omits it is
    // far more often reporting a compile failure than a style hint, and
    // undercounting a real error is the worse mistake.
    expect(severityOf(d(undefined))).toBe(1);
    expect(countProblems([d(undefined)])).toEqual({ errors: 1, warnings: 0 });
  });

  it("reads a nonsense severity as an error too", () => {
    expect(severityOf(d(0))).toBe(1);
    expect(severityOf(d(99))).toBe(1);
    expect(severityOf(d(Number.NaN))).toBe(1);
  });

  it("counts nothing for nothing", () => {
    expect(countProblems([])).toEqual({ errors: 0, warnings: 0 });
  });
});

describe("adding up several documents", () => {
  it("sums them", () => {
    expect(
      totalProblems([[d(1), d(2)], [d(2)], []]),
    ).toEqual({ errors: 1, warnings: 2 });
  });
});

describe("what the status bar says out loud", () => {
  it("says the quiet case plainly", () => {
    expect(problemsLabel({ errors: 0, warnings: 0 })).toBe("No problems");
  });

  it("gets the plurals right", () => {
    expect(problemsLabel({ errors: 1, warnings: 0 })).toBe("1 error");
    expect(problemsLabel({ errors: 2, warnings: 1 })).toBe("2 errors, 1 warning");
  });

  it("leaves out the half that is zero", () => {
    expect(problemsLabel({ errors: 0, warnings: 3 })).toBe("3 warnings");
  });
});
