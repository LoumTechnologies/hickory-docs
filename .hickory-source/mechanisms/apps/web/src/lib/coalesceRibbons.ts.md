<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/coalesceRibbons.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8e259ab2001c"></a>
## `apps/web/src/lib/coalesceRibbons.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Show the requested connections without changing their meaning](../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md).

<a id="f-8e259ab2001c-coalesceRibbons"></a>
### `coalesceRibbons`

<storycode:copy id="f-8e259ab2001c-coalesceRibbons">
import type { Ribbon } from "./ribbons";

/** Presentation only: never alter the byte-precise provenance used for edits. */
export function coalesceRibbons(ribbons: readonly Ribbon[], source: string, output: string, terminal = false): Ribbon[] {
  const groups: Ribbon[] = [];
  for (const ribbon of [...ribbons].sort((a, b) => a.sourceSpan[0] - b.sourceSpan[0])) {
    const previous = [...groups].reverse().find(candidate => candidate.kind === ribbon.kind
      && candidate.whitespaceOnly === ribbon.whitespaceOnly
      && candidate.sourceSpan[1] <= ribbon.sourceSpan[0]
      && source.slice(candidate.sourceSpan[1], ribbon.sourceSpan[0]).trim() === ""
      && (terminal || (candidate.outputRange[1] <= ribbon.outputRange[0]
        && output.slice(candidate.outputRange[1], ribbon.outputRange[0]).trim() === "")
        || candidate.outputRange.every((offset, i) => offset === ribbon.outputRange[i])));
    if (!previous) { groups.push({ ...ribbon }); continue; }
    previous.sourceSpan = [previous.sourceSpan[0], ribbon.sourceSpan[1]];
    previous.sourceByteSpan = [previous.sourceByteSpan[0], ribbon.sourceByteSpan[1]];
    if (!terminal) previous.outputRange = [previous.outputRange[0], ribbon.outputRange[1]];
    previous.bytes += ribbon.bytes;
  }
  return groups;
}
</storycode:copy>

### Assemble `apps/web/src/lib/coalesceRibbons.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/coalesceRibbons.ts"><storycode:paste select="#f-8e259ab2001c-coalesceRibbons"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-b6705bff27f9"></a>
## `apps/web/src/lib/coalesceRibbons.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b6705bff27f9-ribbon"></a>
### `ribbon`

<storycode:copy id="f-b6705bff27f9-ribbon">
import { expect, it } from "vitest";
import { coalesceRibbons } from "./coalesceRibbons";
import type { Ribbon } from "./ribbons";
const ribbon = (start: number, end: number, outputRange: [number, number] = [start, end]): Ribbon => ({
  key: String(start), kind: "literal", sourceSpan: [start, end], sourceByteSpan: [start, end],
  outputRange, bytes: end - start, color: 0, whitespaceOnly: false,
});
</storycode:copy>
<a id="f-b6705bff27f9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b6705bff27f9-expression_statement">
it("combines a whitespace-separated chain without mutating attribution", () => {
  const original = [ribbon(0, 1), ribbon(3, 4), ribbon(5, 6)];
  const grouped = coalesceRibbons(original, "a\n b\tc", "a\n b\tc");
  expect(grouped).toHaveLength(1);
  expect(grouped[0].sourceSpan).toEqual([0, 6]);
  expect(grouped[0].outputRange).toEqual([0, 6]);
  expect(grouped[0].bytes).toBe(3);
  expect(original[0].sourceSpan).toEqual([0, 1]);
});
</storycode:copy>
<a id="f-b6705bff27f9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b6705bff27f9-expression_statement-2">
it("preserves intervening prose, different origin kinds, and repeated sources", () => {
  expect(coalesceRibbons([ribbon(0, 1), ribbon(3, 4)], "ax b", "a  b")).toHaveLength(2);
  expect(coalesceRibbons([ribbon(0, 1), { ...ribbon(3, 4), kind: "paste" }], "a  b", "a  b")).toHaveLength(2);
  expect(coalesceRibbons([ribbon(0, 1), ribbon(0, 1, [1, 2])], "a", "aa")).toHaveLength(2);
});
</storycode:copy>
<a id="f-b6705bff27f9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b6705bff27f9-expression_statement-3">
it("does not span unrelated output text, but groups connections to the same file terminal", () => {
  const ribbons = [ribbon(0, 1), ribbon(3, 4)];
  expect(coalesceRibbons(ribbons, "a  b", "axxb")).toHaveLength(2);
  expect(coalesceRibbons(ribbons, "a  b", "axxb", true)).toHaveLength(1);
});
</storycode:copy>
<a id="f-b6705bff27f9-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-b6705bff27f9-expression_statement-4">
it("combines adjacent sources that point to the exact same passage", () => {
  expect(coalesceRibbons([ribbon(0, 1, [0, 1]), ribbon(3, 4, [0, 1])], "a  b", "x")).toHaveLength(1);
});
</storycode:copy>

### Assemble `apps/web/src/lib/coalesceRibbons.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/coalesceRibbons.test.ts"><storycode:paste select="#f-b6705bff27f9-ribbon"/><storycode:paste select="#f-b6705bff27f9-expression_statement"/><storycode:paste select="#f-b6705bff27f9-expression_statement-2"/><storycode:paste select="#f-b6705bff27f9-expression_statement-3"/><storycode:paste select="#f-b6705bff27f9-expression_statement-4"/></storycode:file>


</storycode:doc>
