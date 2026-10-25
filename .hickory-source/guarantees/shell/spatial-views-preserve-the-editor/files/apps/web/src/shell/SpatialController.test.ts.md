<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/SpatialController.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../contract.md)

Captured SHA-256: `9f1bb726d2e2b0e9fa34cd35e940ef8772bda618cddacdaeba570f8c4da6cb6c`.

<book:entry path="apps/web/src/shell/SpatialController.test.ts">
import { afterEach, expect, it } from "vitest";
import { SpatialController } from "./SpatialController";
import { emptySpatialState, parallaxAnchor, type SpatialState } from "./spatialLayout";
import { PRIMARY } from "./spatialTiles";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
const box = (x: number, y: number, width: number, height: number) => new DOMRect(x, y, width, height);
afterEach(() => document.body.replaceChildren());
it.each([false, true])("follows the source tile with parallax enabled=%s when Files floats away", (parallax) => {
  const root = document.createElement("div"); document.body.append(root);
  Object.defineProperties(root, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  root.getBoundingClientRect = () => box(20, 100, 1200, 800);
  const main = document.createElement("section"), second = document.createElement("section"), target = document.createElement("section"), anchor = document.createElement("span");
  main.dataset.spatialKey = "main"; main.dataset.spatialCenter = "true";
  second.dataset.spatialKey = "second"; second.dataset.spatialTiled = "true";
  target.dataset.spatialKey = "target";
  main.getBoundingClientRect = () => box(20, 100, 1200, 397);
  second.getBoundingClientRect = () => box(20, 503, 1200, 397);
  let sourceY = 860;
  anchor.getBoundingClientRect = () => box(40, sourceY - 2, 10, 4);
  second.append(anchor); root.append(main, second, target);
  let state: SpatialState = { ...emptySpatialState(), center: "main", tiles: { axis: "column", first: PRIMARY, second: "second" },
    drawers: { available: { left: true, right: false }, visible: { left: true, right: false },
      files: { left: false, right: false }, contexts: { left: true, right: false }, pinned: { left: false, right: false }, keys: ["target"] } };
  const items = ["main", "second", "target"].map(key => ({ key, path: `${key}.md`, side: "left" as const }));
  const controller = new SpatialController({ root: () => root, state: () => state, items: () => items,
    parallax: () => parallax,
    relations: () => [{ key: "receipt", fromPath: "second.md", toPath: "target.md", anchor }], update: make => { state = make(state); } });
  controller.paint();
  expect(controller.frames.get("target")!.y).toBeCloseTo(parallaxAnchor(760, 601.5, 397, parallax ? .75 : 1) - 18);
  const before = controller.frames.get("target")!;
  state = { ...state, drawerOffsets: { left: { x: 80, y: 40 } } };
  controller.paint();
  expect(controller.frames.get("target")!.x).toBe(before.x + 80);
  expect(controller.frames.get("target")!.y).toBe(before.y);
  sourceY -= 120;
  controller.paint();
  expect(controller.frames.get("target")!.y).toBeCloseTo(parallaxAnchor(640, 601.5, 397, parallax ? .75 : 1) - 18);
  expect(controller.frames.get("target")!.y).toBeLessThan(before.y);
});

it("can drag back from the screen edge and cancel without changing the original placement", () => {
  const root = document.createElement("div"), title = document.createElement("strong"); root.append(title); document.body.append(root);
  Object.defineProperties(root, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  let state: SpatialState = { ...emptySpatialState(), drawers: {
    available: { left: true, right: false }, visible: { left: true, right: false },
    files: { left: true, right: false }, contexts: { left: true, right: false }, pinned: { left: false, right: false }, keys: [] },
    views: { pin: { side: "left", pin: { x: 160, y: 200, w: 272, h: 253 } } } };
  const controller = new SpatialController({ root: () => root, state: () => state, items: () => [], relations: () => [], update: make => { state = make(state); } });
  const event = (x: number, y: number) => ({ button: 0, pointerId: 1, clientX: x, clientY: y, target: title, currentTarget: root, preventDefault() {} }) as unknown as React.PointerEvent;
  controller.beginFiles("left", event(0, 0));
  controller.move({ pointerId: 1, clientX: 10000, clientY: 0 }); controller.finish();
  const edge = state.drawerOffsets!.left!.x, pin = { ...state.views.pin.pin! };
  expect(edge).toBeLessThan(1200);
  controller.beginFiles("left", event(10000, 0));
  controller.move({ pointerId: 1, clientX: 9970, clientY: 0 });
  expect(state.drawerOffsets!.left!.x).toBe(edge - 30);
  expect(state.views.pin.pin!.x).toBe(pin.x - 30);
  controller.finish(true);
  expect(state.drawerOffsets!.left!.x).toBe(edge);
  expect(state.views.pin.pin).toEqual(pin);
});
</book:entry>

</book:doc>
