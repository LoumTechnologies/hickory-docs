import { describe, expect, it } from "vitest";
import { emptyPane, panes, tab, type Layout } from "./layout";
import { clearSpatialSlot, emptySpatialState, intersects, parallaxAnchor, readSpatialState, resizeSpatialPin, spatialize, spatialKey, type Rect } from "./spatialLayout";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
describe("spatial rectangles", () => {
  const rect = (y: number, h = 100): Rect => ({ x: 0, y, w: 300, h });
  it("keeps ordinary and resized slots clear of every pinned rectangle", () => {
    const pins = [rect(100, 200), rect(430, 120)];
    for (let y = -100; y < 800; y += 7) for (const h of [36, 190, 253]) {
      const slot = clearSpatialSlot(rect(y, h), pins);
      expect(slot).not.toBeNull();
      for (const pin of pins) expect(intersects(slot!, pin, 7)).toBe(false);
    }
  });
  it("uses free space before translating a neighbor, in both directions", () => {
    const lane = rect(0, 600), views = { own: { side: "left" as const, pin: rect(100) }, next: { side: "left" as const, pin: rect(250) } };
    const bottom = resizeSpatialPin(views, "own", 100, "bottom", lane);
    expect(bottom.own.pin).toEqual(rect(100, 200));
    expect(bottom.next.pin).toEqual(rect(308));
    const top = resizeSpatialPin({ own: { side: "left", pin: rect(300) }, previous: { side: "left", pin: rect(150) } }, "own", -100, "top", lane);
    expect(top.own.pin).toEqual(rect(200, 200));
    expect(top.previous.pin).toEqual(rect(92));
    expect(views.next.pin).toEqual(rect(250));
  });
  it("shrinks a neighbor only after its free space is exhausted", () => {
    const lane = rect(0, 400);
    const next = resizeSpatialPin({ a: { side: "left", pin: rect(100) }, b: { side: "left", pin: rect(208, 192) } }, "a", 100, "bottom", lane);
    expect(next.a.pin).toEqual(rect(100, 200));
    expect(next.b.pin).toEqual(rect(308, 92));
    expect(intersects(next.a.pin!, next.b.pin!)).toBe(false);
  });
  it("has no parallax displacement through the central alignment band", () => {
    for (const y of [310, 400, 500, 600, 690]) expect(parallaxAnchor(y, 500, 900)).toBe(y);
    expect(parallaxAnchor(900, 500, 900)).toBeLessThan(900);
  });
  it("keeps every tab identity, gives every view its own pane and supplies two tree buffers", () => {
    const one = tab("document", "notes.md"), two = tab("chat", "agent"), tree = tab("tree", "folder"), p = emptyPane();
    const input: Layout = { root: { ...p, tabs: [one, two], active: 0 }, focus: p.id };
    const tiled = spatialize(input);
    expect(panes(tiled.root).flatMap(p => p.tabs.map(t => t.id))).toEqual([one.id, two.id]);
    expect(spatialize(tiled)).toBe(tiled);
    const trees = spatialize({ ...input, root: { type: "split", id: "s", direction: "row", sizes: [.2, .8], children: [{ ...emptyPane(), tabs: [tree], active: 0 }, input.root] } });
    expect(panes(trees.root).flatMap(p => p.tabs.filter(t => t.kind === "tree"))).toHaveLength(2);
    expect(spatialKey(tab("untitled", "untitled"))).not.toBe(spatialKey(tab("untitled", "untitled")));
  });
  it("treats corrupt presentation storage as optional", () => {
    expect(readSpatialState("invalid")).toEqual(emptySpatialState());
    expect(readSpatialState(JSON.stringify({ version: 1, views: { file: { side: "left", pin: { x: 0, y: 20, w: 200, h: -20 } } } })).views.file.pin).toBeUndefined();
  });
});
