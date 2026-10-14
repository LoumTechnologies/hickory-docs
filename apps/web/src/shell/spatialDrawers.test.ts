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
    expect(pinned.center.w).toBeLessThan(width); expect(pinned.leftRail.x + pinned.leftRail.w + pinned.left.w).toBe(pinned.center.x);
    state.drawers.visible.right = true; state.drawers.pinned.right = true;
    const both = workspaceRegions(width, 800, state);
    expect(both.center.w).toBeGreaterThan(0);
    expect(both.center.x + both.center.w).toBe(both.right.x);
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
