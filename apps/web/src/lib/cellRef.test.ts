import { describe, expect, it } from "vitest";
import { cellLabel, isFormula } from "./cellRef";

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
