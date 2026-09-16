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
