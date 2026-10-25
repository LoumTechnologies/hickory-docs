<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tableFormats.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Cell formatting keeps numbers numeric](../../../../../contract.md)

Captured SHA-256: `a2dd8adb0b98d4cfe323d808687be091ae5efae45b6b70896501772c1d0aa13b`.

<book:entry path="apps/web/src/lib/tableFormats.test.ts">
// Guarantee: docs/guarantees/authoring/cell-formatting-keeps-numbers-numeric.md
import { describe, expect, it } from "vitest";
import { currencyInput, displayNumber, readFormats, writeFormats } from "./tableFormats";
import { insertColumn, insertRow, parseCsv, removeColumns, removeRows, resizeTable } from "./csv";

describe("currency input", () => {
  it.each([
    ["$1,234.50", "1234.50", "USD"], ["-$12.00", "-12.00", "USD"],
    ["($1,234.50)", "-1234.50", "USD"], ["€25.99", "25.99", "EUR"],
    ["25.99 €", "25.99", "EUR"], ["£-2.50", "-2.50", "GBP"], ["¥300", "300", "JPY"],
  ])("reads %s without storing its symbol", (raw, value, format) => {
    expect(currencyInput(raw)).toEqual({ value, format });
  });
  it.each(["$12,34", "€12,50", "$1,23,456", "cost $5", "=A1 + 2", "", "$-+2", "(-$2)"])("leaves ambiguous or nonnumeric %s alone", raw => {
    expect(currencyInput(raw)).toBeNull();
  });
});

it("formats numbers without rounding the stored value or inventing a numeric value for text", () => {
  expect(displayNumber("1234.567", "USD")).toBe("$1,234.57");
  expect(displayNumber("0.125", "percent")).toBe("12.5%");
  expect(displayNumber("1234.5", "number")).toBe("1,234.50");
  expect(displayNumber("", "USD")).toBe("");
  expect(displayNumber("=A1", "USD")).toBe("=A1");
  expect(displayNumber("label", "USD")).toBe("label");
});

it("round-trips per-cell formats, including explicit General, through structural edits", () => {
  const table = { ...parseCsv("name,amount\na,1234.50\nb,5\n"), formats: readFormats("B2:USD B3:general") };
  expect(writeFormats(insertColumn(insertRow(table, 1), 0).formats)).toBe("C3:USD C4:general");
  expect(writeFormats(removeRows(table, [1]).formats)).toBe("B2:general");
  expect(writeFormats(removeColumns(table, [1]).formats)).toBe("");
  expect(writeFormats(resizeTable(table, 2, 2).formats)).toBe("B2:USD");
  expect(readFormats("B2:USD garbage Z0:USD C4:unknown")).toEqual({ B2: "USD" });
});
</book:entry>

</book:doc>
