<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/cellRef.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-73b9f3351e52"></a>
## `apps/web/src/lib/cellRef.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-73b9f3351e52-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-73b9f3351e52-expression_statement">
import { describe, expect, it } from "vitest";
import { cellLabel, isFormula, parseCellLabel } from "./cellRef";

describe("A1 labels", () => {
  it("names the first cell A1", () => {
    expect(cellLabel(0, 0)).toBe("A1");
  });

  it("is bijective base-26, so Z is followed by AA", () => {
    // There is no zero digit; plain base-26 would put AA at 27 only by
    // accident.
    expect(cellLabel(25, 0)).toBe("Z1");
    expect(cellLabel(26, 0)).toBe("AA1");
    expect(cellLabel(27, 0)).toBe("AB1");
    expect(cellLabel(701, 0)).toBe("ZZ1");
  });

  it("counts rows from one, the way a person does", () => {
    expect(cellLabel(1, 99)).toBe("B100");
  });
});

</storycode:copy>
<a id="f-73b9f3351e52-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-73b9f3351e52-expression_statement-2">
describe("what counts as a formula", () => {
  it("is a leading equals, as in every spreadsheet", () => {
    expect(isFormula("=A1+1")).toBe(true);
    expect(isFormula("   =A1")).toBe(true);
  });

  it("is not a value that merely contains one", () => {
    expect(isFormula("a=b")).toBe(false);
    expect(isFormula("42")).toBe(false);
    expect(isFormula("")).toBe(false);
  });
});

</storycode:copy>
<a id="f-73b9f3351e52-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-73b9f3351e52-expression_statement-3">
describe("finding the cell a label names", () => {
  it("round trips every label the grid can draw", () => {
    for (const [column, row] of [
      [0, 0],
      [1, 2],
      [25, 9],
      [26, 0],
      [701, 99],
    ]) {
      expect(parseCellLabel(cellLabel(column, row))).toEqual({ column, row });
    }
  });

  it("refuses what is not a label, rather than guessing", () => {
    for (const text of ["", "A", "1", "A0", "1A", "A1.5", "$B$4"]) {
      expect(parseCellLabel(text)).toBeNull();
    }
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/cellRef.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/cellRef.test.ts"><storycode:paste select="#f-73b9f3351e52-expression_statement"/><storycode:paste select="#f-73b9f3351e52-expression_statement-2"/><storycode:paste select="#f-73b9f3351e52-expression_statement-3"/></storycode:file>

</storycode:doc>
