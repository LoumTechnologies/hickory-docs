<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/coalesceRibbons.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `54ab204da2ddfe803444d6d6745dbc8d3aa49249d5cadd1d5bfdf37bad8a4112`.

<book:entry path="apps/web/src/lib/coalesceRibbons.test.ts">
import { expect, it } from "vitest";
import { coalesceRibbons } from "./coalesceRibbons";
import type { Ribbon } from "./ribbons";
const ribbon = (start: number, end: number, outputRange: [number, number] = [start, end]): Ribbon => ({
  key: String(start), kind: "literal", sourceSpan: [start, end], sourceByteSpan: [start, end],
  outputRange, bytes: end - start, color: 0, whitespaceOnly: false,
});
it("combines a whitespace-separated chain without mutating attribution", () => {
  const original = [ribbon(0, 1), ribbon(3, 4), ribbon(5, 6)];
  const grouped = coalesceRibbons(original, "a\n b\tc", "a\n b\tc");
  expect(grouped).toHaveLength(1);
  expect(grouped[0].sourceSpan).toEqual([0, 6]);
  expect(grouped[0].outputRange).toEqual([0, 6]);
  expect(grouped[0].bytes).toBe(3);
  expect(original[0].sourceSpan).toEqual([0, 1]);
});
it("preserves intervening prose, different origin kinds, and repeated sources", () => {
  expect(coalesceRibbons([ribbon(0, 1), ribbon(3, 4)], "ax b", "a  b")).toHaveLength(2);
  expect(coalesceRibbons([ribbon(0, 1), { ...ribbon(3, 4), kind: "paste" }], "a  b", "a  b")).toHaveLength(2);
  expect(coalesceRibbons([ribbon(0, 1), ribbon(0, 1, [1, 2])], "a", "aa")).toHaveLength(2);
});
it("does not span unrelated output text, but groups connections to the same file terminal", () => {
  const ribbons = [ribbon(0, 1), ribbon(3, 4)];
  expect(coalesceRibbons(ribbons, "a  b", "axxb")).toHaveLength(2);
  expect(coalesceRibbons(ribbons, "a  b", "axxb", true)).toHaveLength(1);
});
it("combines adjacent sources that point to the exact same passage", () => {
  expect(coalesceRibbons([ribbon(0, 1, [0, 1]), ribbon(3, 4, [0, 1])], "a  b", "x")).toHaveLength(1);
});
</book:entry>

</book:doc>
