<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/formulaReferences.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a64e5edabad04bd9f43dd6a1fbc11dce565bd9d61e625665922e2fe40ea4beae`.

<book:entry path="apps/web/src/lib/formulaReferences.test.ts">
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
</book:entry>

</book:doc>
