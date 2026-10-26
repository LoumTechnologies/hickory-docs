<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/formulaReferences.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-50f8e2e2b561"></a>
## `apps/web/src/lib/formulaReferences.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-50f8e2e2b561-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-50f8e2e2b561-expression_statement">
import { describe, expect, it } from "vitest";

import { moveFormulaReferences, shiftFormulaReferences } from "./formulaReferences";

describe("formula references through structural edits", () => {
  it("follows inserted rows and columns, including range endpoints", () => {
    expect(shiftFormulaReferences("=sum(A1:B2)", "row", 1)).toBe("=sum(A1:B3)");
    expect(shiftFormulaReferences("=sum(A1:B2)", "column", 1)).toBe("=sum(A1:C2)");
  });

  it("follows a cut-and-paste rectangle to its new cells", () => {
    expect(
      moveFormulaReferences("=A1+B2", { row: 0, column: 0 }, { rows: 2, columns: 1 }, { row: 3, column: 2 }),
    ).toBe("=C4+B2");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/formulaReferences.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/formulaReferences.test.ts"><storycode:paste select="#f-50f8e2e2b561-expression_statement"/></storycode:file>

</storycode:doc>
