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
it("distinguishes center drops from each edge", () => {
  const rect = { x: 10, y: 20, w: 100, h: 200 };
  expect(dockHit(rect, 60, 120)).toBe("center");
  expect(dockHit(rect, 11, 120)).toBe("left");
  expect(dockHit(rect, 109, 120)).toBe("right");
  expect(dockHit(rect, 60, 21)).toBe("top");
  expect(dockHit(rect, 60, 219)).toBe("bottom");
});
