// Protects docs/guarantees/authoring/a-drawn-diagram-is-document-text.md and
// docs/guarantees/authoring/a-derived-diagram-keeps-your-layout.md — the
// scene's byte form is the document, so its parse and its serialization are
// the contract everything else stands on.
import { describe, expect, it } from "vitest";

import {
  dropRef,
  keyedEdges,
  straightConnector,
  occupiedSlots,
  parseSideRef,
  renumberSides,
  sideHandles,
  sideRef,
  parseSceneSource,
  placeMissing,
  serializeScene,
  uncommittableText,
  withResolvedTopology,
} from "./scene";

const HAND_DRAWN = `{
  "nodes": [
    {"id": "api", "label": "API server"},
    {"id": "db", "shape": "cylinder"}
  ],
  "edges": [
    {"from": "api", "to": "db", "label": "SQL"}
  ],
  "layout": {
    "api": {"x": 40, "y": 30, "w": 160, "h": 64},
    "db": {"x": 40, "y": 190}
  }
}
`;

describe("the scene's byte form", () => {
  it("round-trips canonically: parse then serialize is a fixed point", () => {
    const parsed = parseSceneSource(HAND_DRAWN);
    expect(parsed.ok).toBe(true);
    if (!parsed.ok) return;
    const once = serializeScene(parsed.scene);
    const twice = serializeScene((parseSceneSource(once) as { ok: true; scene: never }).scene);
    expect(twice).toBe(once);
    // One entity per line: a moved node is a one-line diff.
    expect(once).toContain('    {"id": "api", "label": "API server"},');
    expect(once).toContain('    "api": {"x": 40, "y": 30, "w": 160, "h": 64},');
  });

  it("keeps a derived body's paste tag byte-for-byte and rewrites only layout", () => {
    const derived =
      '{\n  "topology": <hick:paste select="#arch-topology" />,\n  "layout": {"api": {"x": 1, "y": 2}}\n}\n';
    const parsed = parseSceneSource(derived);
    expect(parsed.ok).toBe(true);
    if (!parsed.ok) return;
    expect(parsed.scene.paste).toBe('<hick:paste select="#arch-topology" />');
    // Topology arrives from the server's resolved body.
    const scene = withResolvedTopology(
      parsed.scene,
      '{"nodes": [{"id": "api"}, {"id": "db"}], "edges": [{"from": "api", "to": "db"}]}',
    );
    expect(scene.nodes.map((n) => n.id)).toEqual(["api", "db"]);
    const out = serializeScene({ ...scene, layout: { ...scene.layout, db: { x: 9, y: 9 } } });
    expect(out).toContain('<hick:paste select="#arch-topology" />');
    expect(out).toContain('"db": {"x": 9, "y": 9}');
    expect(out).not.toContain('"nodes"');
  });

  it("drops layout entries for nodes the topology no longer has", () => {
    const parsed = parseSceneSource(HAND_DRAWN);
    if (!parsed.ok) throw new Error(parsed.error);
    const scene = {
      ...parsed.scene,
      layout: { ...parsed.scene.layout, ghost: { x: 0, y: 0 } },
    };
    expect(serializeScene(scene)).not.toContain("ghost");
  });

  it("refuses the one string the no-escaping invariant cannot hold", () => {
    const parsed = parseSceneSource(HAND_DRAWN);
    if (!parsed.ok) throw new Error(parsed.error);
    expect(uncommittableText(parsed.scene)).toBeNull();
    const poisoned = {
      ...parsed.scene,
      nodes: [{ id: "a", label: "closes </hick:diagram> early" }],
    };
    expect(uncommittableText(poisoned)).toContain("</hick:");
  });

  it("gives an unplaced node a spot beside the layout, never on top of it", () => {
    const nodes = [{ id: "api" }, { id: "new-1" }, { id: "new-2" }];
    const layout = placeMissing(nodes, { api: { x: 0, y: 0, w: 160 } });
    expect(layout.api).toEqual({ x: 0, y: 0, w: 160 });
    expect(layout["new-1"].x).toBeGreaterThan(160);
    expect(layout["new-2"].y).toBeGreaterThan(layout["new-1"].y);
  });

  it("keeps the lines in use centred, with one free flank on each end", () => {
    // An empty side is one centred point. Occupied slots are balanced on
    // the midline — a vacancy never pushes a line off centre — and the two
    // flanks sit symmetrically outside them.
    expect(sideHandles("left", [])).toEqual([{ id: "left", offset: 0, extra: false }]);
    expect(sideHandles("left", [0])).toEqual([
      { id: "left._before", offset: -18, extra: true },
      { id: "left", offset: 0, extra: false },
      { id: "left._after", offset: 18, extra: true },
    ]);
    expect(sideHandles("left", [0, 1])).toEqual([
      { id: "left._before", offset: -27, extra: true },
      { id: "left", offset: -9, extra: false },
      { id: "left.1", offset: 9, extra: false },
      { id: "left._after", offset: 27, extra: true },
    ]);
  });

  it("keeps three identical-looking parallel lines individually addressable", () => {
    // Hand-written (or pre-id-era) parallel edges carry no ids; identity
    // still has to answer "which line?" or two of them shadow the third —
    // the bug where double-click opened the label editor on SOME lines.
    const keys = keyedEdges([
      { from: "a", to: "b" },
      { from: "a", to: "b" },
      { from: "a", to: "b" },
      { from: "a", to: "b", id: "named" },
    ]).map((e) => e.key);
    expect(keys).toEqual(["a->b", "a->b#2", "a->b#3", "named"]);
    expect(new Set(keys).size).toBe(4);
  });

  it("orders a flank drop before or after the occupants, then compacts the numbers", () => {
    expect(dropRef("left._after", [0])).toBe("left.1");
    expect(dropRef("left._before", [0])).toBe("left.-1");
    expect(dropRef("left.1", [0])).toBe("left.1");
    const compacted = renumberSides([
      { from: "a", to: "b", toSide: "left.-1" },
      { from: "c", to: "b", toSide: "left.4" },
    ]);
    expect(compacted.map((e) => e.toSide)).toEqual(["left", "left.1"]);
  });

  it("reads slot occupancy from recorded sides, bare and suffixed alike", () => {
    const slots = occupiedSlots([
      { from: "a", to: "b", fromSide: "right", toSide: "left" },
      { from: "c", to: "b", toSide: "left.1" },
      { from: "c", to: "b" }, // side-less: counts nowhere
    ]);
    expect(slots.b.left).toEqual([0, 1]);
    expect(slots.a.right).toEqual([0]);
    // Slot 0 keeps the bare side name, so old scenes mean what they meant.
    expect(sideRef("left", 0)).toBe("left");
    expect(sideRef("left", 1)).toBe("left.1");
    expect(parseSideRef("left.1")).toEqual({ side: "left", slot: 1 });
    expect(parseSideRef("corner")).toBeNull();
  });

  it("never draws an elbow whose corners cannot render whole", () => {
    // A 9px jog between near-aligned boxes was a clipped little S — every
    // corner you see must be a whole corner, so below the threshold the
    // line is straight.
    expect(straightConnector("right", "left", 300, 9)).toBe(true);
    expect(straightConnector("right", "left", 300, 18)).toBe(true);
    expect(straightConnector("bottom", "top", 12, 300)).toBe(true);
    // Real lateral distance earns real corners…
    expect(straightConnector("right", "left", 300, 120)).toBe(false);
    // …and between adjacent boxes there is no room for corners at all.
    expect(straightConnector("right", "top", 40, 40)).toBe(true);
  });

  it("reports a parse failure instead of throwing or guessing", () => {
    const parsed = parseSceneSource("{ not json");
    expect(parsed.ok).toBe(false);
  });
});
