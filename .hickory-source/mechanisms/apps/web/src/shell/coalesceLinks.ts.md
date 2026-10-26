<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/coalesceLinks.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f87ad5795296"></a>
## `apps/web/src/shell/coalesceLinks.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Show the requested connections without changing their meaning](../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md).

<a id="f-f87ad5795296-coalesceLinks"></a>
### `coalesceLinks`

<storycode:copy id="f-f87ad5795296-coalesceLinks">
import type { RibbonLink, RibbonSource } from "./Ribbons";
import { samePath } from "../lib/paths";

/** Adjacent document passages citing the same destination share one shape. */
export function coalesceLinks(links: readonly RibbonLink[], sources: readonly RibbonSource[]): RibbonLink[] {
  const groups: RibbonLink[] = [];
  for (const link of [...links].sort((a, b) => a.from.lines[0] - b.from.lines[0])) {
    const source = sources.find(s => samePath(s.docPath, link.from.path));
    const text = source?.view?.state.doc.toString() ?? source?.docSource;
    const previous = text === undefined || link.anchor || link.family === "execution" ? undefined : [...groups].reverse().find(candidate =>
      !candidate.anchor && candidate.family === link.family && candidate.alwaysVisible === link.alwaysVisible
      && samePath(candidate.from.path, link.from.path) && samePath(candidate.to.path, link.to.path)
      && candidate.to.kind === link.to.kind && String(candidate.to.lines) === String(link.to.lines)
      && candidate.from.lines[1] <= link.from.lines[0]
      && text.split("\n").slice(candidate.from.lines[1], link.from.lines[0] - 1).join("\n").trim() === "");
    if (previous) previous.from = { ...previous.from, lines: [previous.from.lines[0], link.from.lines[1]] };
    else groups.push({ ...link, from: { ...link.from } });
  }
  return groups;
}
</storycode:copy>

### Assemble `apps/web/src/shell/coalesceLinks.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/coalesceLinks.ts"><storycode:paste select="#f-f87ad5795296-coalesceLinks"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-2bf5b6b77218"></a>
## `apps/web/src/shell/coalesceLinks.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-2bf5b6b77218-link"></a>
### `link`

<storycode:copy id="f-2bf5b6b77218-link">
import { expect, it } from "vitest";
import { coalesceLinks } from "./coalesceLinks";
import type { RibbonLink } from "./Ribbons";
const link = (line: number): RibbonLink => ({ key: String(line), family: "declared", title: "citation",
  from: { path: "d.md", lines: [line, line] }, to: { path: "target.md", lines: [2, 3] } });
</storycode:copy>
<a id="f-2bf5b6b77218-sources"></a>
### `sources`

<storycode:copy id="f-2bf5b6b77218-sources">
const sources = [{ docPath: "d.md", docSource: "first\n \t\nsecond\n\nthird" }];
</storycode:copy>
<a id="f-2bf5b6b77218-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2bf5b6b77218-expression_statement">
it("groups whitespace-separated citations to the same passage", () => {
  const links = [link(1), link(3), link(5)];
  expect(coalesceLinks(links, sources).map(l => l.from.lines)).toEqual([[1, 5]]);
  expect(links[0].from.lines).toEqual([1, 1]);
});
</storycode:copy>
<a id="f-2bf5b6b77218-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2bf5b6b77218-expression_statement-2">
it("preserves prose, destinations, families, and anchored cards", () => {
  expect(coalesceLinks([link(1), link(5)], sources)).toHaveLength(2);
  for (const changed of [{ to: { path: "other.md" } }, { to: { path: "target.md", lines: [4, 5] as [number, number] } },
    { family: "context" as const }, { anchor: {} as HTMLElement }]) {
    expect(coalesceLinks([link(1), { ...link(3), ...changed }], sources)).toHaveLength(2);
  }
});
</storycode:copy>

### Assemble `apps/web/src/shell/coalesceLinks.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/coalesceLinks.test.ts"><storycode:paste select="#f-2bf5b6b77218-link"/><storycode:paste select="#f-2bf5b6b77218-sources"/><storycode:paste select="#f-2bf5b6b77218-expression_statement"/><storycode:paste select="#f-2bf5b6b77218-expression_statement-2"/></storycode:file>


</storycode:doc>
