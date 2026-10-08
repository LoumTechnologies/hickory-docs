// Guarantee: docs/guarantees/authoring/a-fill-handle-extends-formulas.md
import { describe, expect, it } from "vitest";
import { copyFormulaReferences } from "./formulaReferences";
import { parseCsv } from "./csv";
import { fillSelection, fillTable } from "./tableFill";
import { boundsOf, cellSelection, type Selection } from "./tableSelection";

it("copies relative, absolute and mixed references and both range endpoints", () => {
  expect(copyFormulaReferences("=sum(A1:B3) + $A1 + B$2 + $C$4", 2, 3))
    .toBe("=sum(D3:E5) + $A3 + E$2 + $C$4");
  expect(copyFormulaReferences("=C4 + $C4 + C$4 + $C$4", -2, -1)).toBe("=B2 + $C2 + B$4 + $C$4");
});
it("leaves language strings, comments and identifiers intact", () => {
  expect(copyFormulaReferences('=A1 + len("A1") + myA1 # B2', 1, 0, "python")).toBe('=A2 + len("A1") + myA1 # B2');
  expect(copyFormulaReferences("=A1 + 'B2'.length // C3", 0, 1)).toBe("=B1 + 'B2'.length // C3");
  expect(copyFormulaReferences("=A1 + `B2` /* C3 */", 1, 0)).toBe("=A2 + `B2` /* C3 */");
  expect(copyFormulaReferences("=len('''A1\nB2''') + C3", 1, 0)).toBe("=len('''A1\nB2''') + C4");
});
it("marks references moved before A1 invalid instead of wrapping them", () => {
  expect(copyFormulaReferences("=A1 + $A$1", -1, -1)).toBe("=#REF! + $A$1");
});

describe("four directions", () => {
  const seed = cellSelection(2, 2);
  const table = parseCsv("0,0,0,0,0\n0,0,0,0,0\n0,0,=B3 * 2,0,0\n0,0,0,0,0\n0,0,0,0,0");
  it.each([
    [{ row: 4, column: 2 }, 4, 2, "=B5 * 2"],
    [{ row: 0, column: 2 }, 0, 2, "=B1 * 2"],
    [{ row: 2, column: 4 }, 2, 4, "=D3 * 2"],
    [{ row: 2, column: 1 }, 2, 1, "=A3 * 2"],
  ] as const)("fills toward %j", (at, row, column, expected) => {
    const filled = fillTable(table, seed, fillSelection(seed, at, 5, 5), true);
    expect(filled.rows[row][column]).toBe(expected);
    expect(filled.rows[2][2]).toBe("=B3 * 2");
    expect(table.rows[row][column]).toBe("0");
  });
});
it("tiles a selected block upward and downward from the frozen seed, carrying formats", () => {
  const table = { ...parseCsv("0\n0\n=A3 * 2\n=A4 * 3\n0\n0"), formats: { A3: "USD", A4: "percent", A1: "number" } as const };
  const seed: Selection = { anchor: { row: 3, column: 0 }, focus: { row: 2, column: 0 }, kind: "cells" };
  const up = fillTable(table, seed, fillSelection(seed, { row: 0, column: 0 }, 6, 1), true);
  expect(up.rows.slice(0, 4)).toEqual([["=A1 * 2"], ["=A2 * 3"], ["=A3 * 2"], ["=A4 * 3"]]);
  expect(up.formats?.A1).toBe("USD");
  expect(up.formats?.A2).toBe("percent");
  const down = fillTable(table, seed, fillSelection(seed, { row: 5, column: 0 }, 6, 1), true);
  expect(down.rows.slice(4)).toEqual([["=A5 * 2"], ["=A6 * 3"]]);
});
it("repeats literals and treats equals as text when no language is declared", () => {
  const table = parseCsv("=A1,hello\n0,0");
  const seed: Selection = { anchor: { row: 0, column: 0 }, focus: { row: 0, column: 1 }, kind: "cells" };
  expect(fillTable(table, seed, fillSelection(seed, { row: 1, column: 1 }, 2, 2), false).rows[1]).toEqual(["=A1", "hello"]);
});
it("grows within the grid budget and selects only one axis on a diagonal", () => {
  const seed = cellSelection(0, 0);
  expect(boundsOf(fillSelection(seed, { row: 100_000, column: 0 }, 1, 2))).toEqual({ top: 0, bottom: 9999, left: 0, right: 0 });
  expect(boundsOf(fillSelection(seed, { row: 0, column: 100_000 }, 2, 1))).toEqual({ top: 0, bottom: 0, left: 0, right: 9999 });
  const target = fillSelection(seed, { row: 3, column: 2 }, 1, 1);
  expect(boundsOf(target)).toEqual({ top: 0, bottom: 3, left: 0, right: 0 });
  expect(fillTable(parseCsv("=B1"), seed, target, true).rows).toEqual([["=B1"], ["=B2"], ["=B3"], ["=B4"]]);
});

it('keeps Python floor division while moving its operand references', () => {
  expect(copyFormulaReferences('=A1 // B1 # C1', 1, 0, 'python')).toBe('=A2 // B2 # C1');
});
