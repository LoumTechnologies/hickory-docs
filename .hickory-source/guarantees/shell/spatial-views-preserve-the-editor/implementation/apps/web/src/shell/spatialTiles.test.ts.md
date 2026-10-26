<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/spatialTiles.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ba8df0717d23"></a>
## `apps/web/src/shell/spatialTiles.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../contract.md).

<a id="f-ba8df0717d23-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ba8df0717d23-expression_statement">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
import { expect, it } from "vitest";
import { PRIMARY, dockTile, dockHit, pruneTiles, readTiles, tileRects } from "./spatialTiles";

it("docks on either axis, moves an existing tile and reclaims closed tiles", () => {
  const split = dockTile(PRIMARY, "a", PRIMARY, "right");
  const nested = dockTile(split, "b", "a", "bottom");
  const rects = tileRects(nested, { x: 0, y: 0, w: 1006, h: 806 });
  expect(rects.get(PRIMARY)).toEqual({ x: 0, y: 0, w: 500, h: 806 });
  expect(rects.get("b")).toEqual({ x: 506, y: 406, w: 500, h: 400 });
  const moved = dockTile(nested, "b", PRIMARY, "left");
  expect([...tileRects(moved, { x: 0, y: 0, w: 1006, h: 806 }).keys()]).toEqual(["b", PRIMARY, "a"]);
  expect(pruneTiles(moved, key => key === PRIMARY)).toBe(PRIMARY);
  expect(readTiles(nested)).toEqual(nested);
  expect(readTiles({ axis: "diagonal", first: PRIMARY, second: "a" })).toBeUndefined();
});
</storycode:copy>
<a id="f-ba8df0717d23-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ba8df0717d23-expression_statement-2">
it("distinguishes center drops from each edge", () => {
  const rect = { x: 10, y: 20, w: 100, h: 200 };
  expect(dockHit(rect, 60, 120)).toBe("center");
  expect(dockHit(rect, 11, 120)).toBe("left");
  expect(dockHit(rect, 109, 120)).toBe("right");
  expect(dockHit(rect, 60, 21)).toBe("top");
  expect(dockHit(rect, 60, 219)).toBe("bottom");
});
</storycode:copy>

### Assemble `apps/web/src/shell/spatialTiles.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/spatialTiles.test.ts"><storycode:paste select="#f-ba8df0717d23-expression_statement"/><storycode:paste select="#f-ba8df0717d23-expression_statement-2"/></storycode:file>

</storycode:doc>
