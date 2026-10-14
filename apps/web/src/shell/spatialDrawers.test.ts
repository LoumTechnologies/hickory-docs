import { expect, it } from "vitest";
import { drawerContent, workspaceRegions, type DrawerPresentation } from "./spatialDrawers";
import { emptySpatialState, readSpatialState } from "./spatialLayout";

it("counts relationships, pending reviews and explicit pins without counting unrelated open tabs", () => {
  const items = ["main", "related", "review", "pinned", "other"].map(path => ({ key: path, path: `${path}.md`, side: "left" as const }));
  const content = drawerContent(items, items[0], [{ key: "link", fromPath: "main.md", toPath: "related.md" }],
    { pinned: { side: "left", pin: { x: 0, y: 0, w: 100, h: 100 } } }, new Set(["review"]));
  expect(content.map(item => item.key)).toEqual(["related", "review", "pinned"]);
  expect(drawerContent(items, items[0], [], {}, new Set())).toEqual([]);
});

const drawers = (): DrawerPresentation => ({ available: { left: true, right: true }, visible: { left: false, right: false },
  files: { left: true, right: true }, contexts: { left: true, right: true }, pinned: { left: false, right: false }, keys: [] });
it("reserves no space for collapsed or overlay drawers and reserves space only when pinned", () => {
  for (const width of [500, 1200]) {
    const state = { ...emptySpatialState(), drawers: drawers() };
    expect(workspaceRegions(width, 800, state).center).toEqual({ x: 0, y: 0, w: width, h: 800 });
    state.drawers.visible.left = true;
    const overlay = workspaceRegions(width, 800, state);
    expect(overlay.center.w).toBe(width); expect(overlay.left.w).toBeGreaterThan(0);
    state.drawers.pinned.left = true;
    const pinned = workspaceRegions(width, 800, state);
    expect(pinned.center.w).toBeLessThan(width); if (pinned.filesBelow.left) {
      expect(pinned.leftRail.x + pinned.leftRail.w).toBe(pinned.center.x);
      expect(pinned.left.x + pinned.left.w).toBeLessThan(pinned.center.x);
    } else expect(pinned.leftRail.x + pinned.leftRail.w + pinned.left.w).toBe(pinned.center.x);
    state.drawers.visible.right = true; state.drawers.pinned.right = true;
    const both = workspaceRegions(width, 800, state);
    expect(both.center.w).toBeGreaterThan(0);
    expect(both.center.x + both.center.w).toBe(both.rightRail.x);
  }
});

it("gives a files-only drawer its whole width rather than reserving an empty context lane", () => {
  const drawer = drawers(); drawer.visible.left = true; drawer.contexts.left = false;
  const regions = workspaceRegions(1200, 800, { ...emptySpatialState(), drawers: drawer });
  expect(regions.left.w).toBe(0); expect(regions.leftRail.w).toBeGreaterThan(0);
});

it("restores drawer pins, treats open state as transient, and migrates existing view pins", () => {
  const current = readSpatialState(JSON.stringify({ version: 1, drawerPins: { left: true, right: false }, drawerOpen: { left: true } }));
  expect(current.drawerPins).toEqual({ left: true, right: false }); expect(current.drawers).toBeUndefined();
  const previous = readSpatialState(JSON.stringify({ version: 1, views: { file: { side: "left", pin: { x: 0, y: 20, w: 200, h: 250 } } } }));
  expect(previous.drawerPins).toEqual({ left: true, right: false });
});

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
it("translates the whole Files surface, clamps it on screen and restores offsets", () => {
  const drawer = drawers(); drawer.visible.left = true;
  const state = { ...emptySpatialState(), drawers: drawer };
  const before = workspaceRegions(1200, 800, state);
  const offsets = { left: { x: 90, y: 60 } };
  const after = workspaceRegions(1200, 800, { ...state, drawerOffsets: offsets });
  for (const key of ["left", "leftRail"] as const) {
    expect(after[key].x).toBe(before[key].x + 90);
    expect(after[key].y).toBe(before[key].y + 60);
  }
  expect(after.center).toEqual(before.center);
  expect(readSpatialState(JSON.stringify({ version: 1, drawerOffsets: offsets })).drawerOffsets).toEqual(offsets);
  const edge = workspaceRegions(500, 400, { ...state, drawerOffsets: { left: { x: 9000, y: 9000 } } });
  if (!edge.drawers) throw new Error("Expected drawer geometry");
  expect(edge.drawers.left.x + edge.drawers.left.w).toBeLessThanOrEqual(500);
  expect(edge.drawers.left.y + edge.drawers.left.h).toBeLessThanOrEqual(400);
  expect(edge.drawers.left.h).toBeGreaterThanOrEqual(160);
});

it("keeps context for a ribbon source in a second center tile", () => {
  const items = ["main", "tile", "related"].map(path => ({ key: path, path: `${path}.md`, side: "left" as const }));
  const relation = { key: "tile-link", fromPath: "tile.md", toPath: "related.md" };
  expect(drawerContent(items, items[0], [relation], {}, new Set(), items.slice(0, 2)).map(item => item.key)).toEqual(["related"]);
});


// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
it("switches each Files pane by its own width, with previews below or to the right", () => {
  const drawer = drawers(); drawer.visible = { left: true, right: true };
  for (const side of ["left", "right"] as const) {
    const index = side === "left" ? 0 : 1;
    const state = { ...emptySpatialState(), drawers: drawer };
    state.columns[index] = .16;
    const narrow = workspaceRegions(1200, 800, state);
    expect(narrow.filesBelow[side]).toBe(true);
    expect(narrow[side].x).toBe(narrow[`${side}Rail`].x + 8);
    expect(narrow[side].w).toBe(narrow[`${side}Rail`].w - 16);
    state.columns[index] = .30;
    const wide = workspaceRegions(1200, 800, state);
    expect(wide.filesBelow[side]).toBe(false);
    expect(wide[side].x).toBe(wide[`${side}Rail`].x + wide[`${side}Rail`].w);
    expect(wide[side].w).toBeGreaterThanOrEqual(200);
  }
});

it("keeps files-only and preview-only panes in one full-width column", () => {
  const drawer = drawers(); drawer.visible.left = true;
  const state = { ...emptySpatialState(), drawers: drawer };
  drawer.contexts.left = false;
  const files = workspaceRegions(1200, 800, state);
  expect(files.filesBelow.left).toBe(false); expect(files.left.w).toBe(0);
  drawer.contexts.left = true; drawer.files.left = false;
  const previews = workspaceRegions(1200, 800, state);
  expect(previews.filesBelow.left).toBe(false); expect(previews.leftRail.w).toBe(0);
  expect(previews.left.w).toBe(files.leftRail.w);
});
