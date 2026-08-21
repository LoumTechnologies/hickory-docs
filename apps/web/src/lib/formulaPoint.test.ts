// Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
// — rule 7, "a click is how a reference gets written".

import { describe, expect, it } from "vitest";

import { acceptsReference, pointAt } from "./formulaPoint";

describe("where a reference could go", () => {
  it("accepts one after the `=` that starts a formula", () => {
    expect(acceptsReference("=", 1)).toBe(true);
  });

  it("accepts one after an operator, an opener, or a separator", () => {
    for (const text of ["=1+", "=sum(", "=sum([A1,", "=A1 * ", "=f(x, "]) {
      expect(acceptsReference(text, text.length)).toBe(true);
    }
  });

  it("refuses one where the expression already has its operand", () => {
    // After `42` or after a word, a click is a click — the formula is not
    // asking for anything.
    for (const text of ["=42", "=sum(A1)", "=total"]) {
      expect(acceptsReference(text, text.length)).toBe(false);
    }
  });

  it("refuses one in an empty field, which is not a formula at all", () => {
    expect(acceptsReference("", 0)).toBe(false);
  });
});

describe("writing the reference in", () => {
  it("puts it at the caret", () => {
    const out = pointAt("=1+", 3, "B2")!;
    expect(out.text).toBe("=1+B2");
    expect(out.caret).toBe(5);
    expect(out.pointed).toEqual({ start: 3, end: 5 });
  });

  it("writes into the middle when that is where the caret is", () => {
    const out = pointAt("=(+1)", 2, "A1")!;
    expect(out.text).toBe("=(A1+1)");
  });

  it("REPLACES the reference the last click put there", () => {
    // Clicking around to find the right cell should leave one reference, not
    // five.
    const first = pointAt("=", 1, "B2")!;
    const second = pointAt(first.text, first.caret, "C7", first.pointed)!;
    expect(second.text).toBe("=C7");
    expect(second.pointed).toEqual({ start: 1, end: 3 });
    const third = pointAt(second.text, second.caret, "AA10", second.pointed)!;
    expect(third.text).toBe("=AA10");
  });

  it("stops replacing once the caret has moved off the reference", () => {
    // The person typed `+` after it; the next click is a new operand.
    const first = pointAt("=", 1, "B2")!;
    const typed = `${first.text}+`;
    const next = pointAt(typed, typed.length, "C7", first.pointed)!;
    expect(next.text).toBe("=B2+C7");
  });

  it("says no when the formula is not asking for an operand", () => {
    // The caller then treats the click as the ordinary click it was.
    expect(pointAt("=42", 3, "B2")).toBeNull();
  });
});
