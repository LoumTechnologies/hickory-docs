<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/scene.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3f46decc2b79"></a>
## `apps/web/src/components/graph/scene.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Derived Diagram Keeps Your Layout](../../../../../../guarantees/authoring/a-derived-diagram-keeps-your-layout/contract.md), [A Drawn Diagram Is Document Text](../../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md).

<a id="f-3f46decc2b79-SceneNode"></a>
### `SceneNode`

The graph scene: the TypeScript mirror of crates/hick-literate/src/scene.rs.
One split does all the work here: TOPOLOGY (nodes/edges — what a generator
can deduce) versus LAYOUT (where a person put things — what only a human
decides). A hand-drawn scene holds both inline; a DERIVED scene's topology
arrives through a `<hick:paste>` from the fragment a generator wrote, and
the editor may then rewrite only the layout.
The serializer is CANONICAL: stable key order, one node/edge/layout entry
per line, integer positions. Line-per-entity is what keeps git diffs and
CRDT merges of a diagram edit local to the thing that moved, and a stable
byte form is what makes "did my own commit come back?" a string compare.
The schema is written fresh; its shape is informed by the pure-data format
grafly documents (grafly's code is AGPL and none of it was read or used —
see docs/specs/freeform/a-diagram-you-can-drag.md).

<storycode:copy id="f-3f46decc2b79-SceneNode">
// The graph scene: the TypeScript mirror of crates/hick-literate/src/scene.rs.
//
// One split does all the work here: TOPOLOGY (nodes/edges — what a generator
// can deduce) versus LAYOUT (where a person put things — what only a human
// decides). A hand-drawn scene holds both inline; a DERIVED scene's topology
// arrives through a `<hick:paste>` from the fragment a generator wrote, and
// the editor may then rewrite only the layout.
//
// The serializer is CANONICAL: stable key order, one node/edge/layout entry
// per line, integer positions. Line-per-entity is what keeps git diffs and
// CRDT merges of a diagram edit local to the thing that moved, and a stable
// byte form is what makes "did my own commit come back?" a string compare.
//
// The schema is written fresh; its shape is informed by the pure-data format
// grafly documents (grafly's code is AGPL and none of it was read or used —
// see docs/specs/freeform/a-diagram-you-can-drag.md).

export interface SceneNode {
  id: string;
  label?: string;
  /** rect (default), round, pill, circle, diamond, hexagon, cylinder. */
  shape?: string;
  fill?: string;
  stroke?: string;
  /** Text colour. Absent means the theme's own. */
  text?: string;
}

</storycode:copy>
<a id="f-3f46decc2b79-SceneEdge"></a>
### `SceneEdge`

<storycode:copy id="f-3f46decc2b79-SceneEdge">
export interface SceneEdge {
  id?: string;
  from: string;
  to: string;
  /** Which side of the node each end attaches to: top, right, bottom, left.
   * Absent means "wherever the renderer likes" — a generator never sets
   * these; they are recorded when a person draws or re-drags an end. */
  fromSide?: string;
  toSide?: string;
  label?: string;
  /** solid (default), dashed, dotted. */
  style?: string;
  /** Which end(s) wear an arrowhead: end (default), start, both, none. */
  arrow?: string;
  /** Line colour, arrowheads included. Absent means the theme's own. */
  color?: string;
}

</storycode:copy>
<a id="f-3f46decc2b79-NodeLayout"></a>
### `NodeLayout`

<storycode:copy id="f-3f46decc2b79-NodeLayout">
export interface NodeLayout {
  x: number;
  y: number;
  w?: number;
  h?: number;
}

</storycode:copy>
<a id="f-3f46decc2b79-Scene"></a>
### `Scene`

<storycode:copy id="f-3f46decc2b79-Scene">
export interface Scene {
  nodes: SceneNode[];
  edges: SceneEdge[];
  layout: Record<string, NodeLayout>;
  /** The literal `<hick:paste …/>` text, when the topology is derived. The
   * commit must write it back byte-for-byte — it is the document's record of
   * where the topology comes from. */
  paste: string | null;
}

</storycode:copy>
<a id="f-3f46decc2b79-ParsedScene"></a>
### `ParsedScene`

<storycode:copy id="f-3f46decc2b79-ParsedScene">
export type ParsedScene =
  | { ok: true; scene: Scene }
  | { ok: false; error: string };

</storycode:copy>
<a id="f-3f46decc2b79-PASTE_RE"></a>
### `PASTE_RE`

<storycode:copy id="f-3f46decc2b79-PASTE_RE">
const PASTE_RE = /<hick:paste(?:\s+(?:"[^"]*"|'[^']*'|[^<>"'])*)?\/>/;

</storycode:copy>
<a id="f-3f46decc2b79-parseSceneSource"></a>
### `parseSceneSource`

Parse a diagram body as it stands in the DOCUMENT. A derived body is not
JSON — it holds a paste tag where its topology goes — so the tag is lifted
out, remembered verbatim, and its place taken by `null` for the parse.

<storycode:copy id="f-3f46decc2b79-parseSceneSource">
/**
 * Parse a diagram body as it stands in the DOCUMENT. A derived body is not
 * JSON — it holds a paste tag where its topology goes — so the tag is lifted
 * out, remembered verbatim, and its place taken by `null` for the parse.
 */
export function parseSceneSource(raw: string): ParsedScene {
  const paste = PASTE_RE.exec(raw)?.[0] ?? null;
  const text = paste ? raw.replace(paste, "null") : raw;
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error) };
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    return { ok: false, error: "a scene is a JSON object" };
  }
  const body = parsed as {
    nodes?: SceneNode[];
    edges?: SceneEdge[];
    topology?: { nodes?: SceneNode[]; edges?: SceneEdge[] } | null;
    layout?: Record<string, NodeLayout>;
  };
  if (paste && (body.nodes?.length || body.edges?.length)) {
    return {
      ok: false,
      error:
        "a scene carries either inline nodes/edges or a derived topology, never both",
    };
  }
  const topology = body.topology ?? undefined;
  return {
    ok: true,
    scene: {
      nodes: topology?.nodes ?? body.nodes ?? [],
      edges: topology?.edges ?? body.edges ?? [],
      layout: body.layout ?? {},
      paste,
    },
  };
}

</storycode:copy>
<a id="f-3f46decc2b79-withResolvedTopology"></a>
### `withResolvedTopology`

Fold the server-resolved body's topology into a derived scene. The raw
source only says WHERE the topology comes from; the resolved body says what
it currently is.

<storycode:copy id="f-3f46decc2b79-withResolvedTopology">
/** Fold the server-resolved body's topology into a derived scene. The raw
 * source only says WHERE the topology comes from; the resolved body says what
 * it currently is. */
export function withResolvedTopology(scene: Scene, resolved: string): Scene {
  if (!scene.paste) return scene;
  const parsed = parseSceneSource(resolved);
  if (!parsed.ok) return scene;
  return { ...scene, nodes: parsed.scene.nodes, edges: parsed.scene.edges };
}

</storycode:copy>
<a id="f-3f46decc2b79-uncommittableText"></a>
### `uncommittableText`

The one string a document body cannot contain (no-escaping invariant): a
label carrying `</hick:` would close the diagram from inside its own JSON.

<storycode:copy id="f-3f46decc2b79-uncommittableText">
/** The one string a document body cannot contain (no-escaping invariant): a
 * label carrying `</hick:` would close the diagram from inside its own JSON. */
export function uncommittableText(scene: Scene): string | null {
  const strings = [
    ...scene.nodes.flatMap((n) => [n.id, n.label ?? "", n.shape ?? "", n.fill ?? "", n.stroke ?? ""]),
    ...scene.edges.flatMap((e) => [e.id ?? "", e.from, e.to, e.label ?? "", e.style ?? "", e.arrow ?? ""]),
  ];
  return strings.find((s) => s.includes("</hick:")) ?? null;
}

</storycode:copy>
<a id="f-3f46decc2b79-jsonField"></a>
### `jsonField`

<storycode:copy id="f-3f46decc2b79-jsonField">
function jsonField(key: string, value: string | undefined): string | null {
  return value === undefined || value === "" ? null : `${JSON.stringify(key)}: ${JSON.stringify(value)}`;
}

</storycode:copy>
<a id="f-3f46decc2b79-nodeLine"></a>
### `nodeLine`

<storycode:copy id="f-3f46decc2b79-nodeLine">
function nodeLine(node: SceneNode): string {
  const fields = [
    jsonField("id", node.id),
    jsonField("label", node.label),
    jsonField("shape", node.shape),
    jsonField("fill", node.fill),
    jsonField("stroke", node.stroke),
    jsonField("text", node.text),
  ].filter(Boolean);
  return `{${fields.join(", ")}}`;
}

</storycode:copy>
<a id="f-3f46decc2b79-edgeLine"></a>
### `edgeLine`

<storycode:copy id="f-3f46decc2b79-edgeLine">
function edgeLine(edge: SceneEdge): string {
  const fields = [
    jsonField("id", edge.id),
    jsonField("from", edge.from),
    jsonField("fromSide", edge.fromSide),
    jsonField("to", edge.to),
    jsonField("toSide", edge.toSide),
    jsonField("label", edge.label),
    jsonField("style", edge.style),
    jsonField("arrow", edge.arrow),
    jsonField("color", edge.color),
  ].filter(Boolean);
  return `{${fields.join(", ")}}`;
}

</storycode:copy>
<a id="f-3f46decc2b79-layoutLine"></a>
### `layoutLine`

<storycode:copy id="f-3f46decc2b79-layoutLine">
function layoutLine(id: string, at: NodeLayout): string {
  const fields = [
    `"x": ${Math.round(at.x)}`,
    `"y": ${Math.round(at.y)}`,
    ...(at.w !== undefined ? [`"w": ${Math.round(at.w)}`] : []),
    ...(at.h !== undefined ? [`"h": ${Math.round(at.h)}`] : []),
  ];
  return `${JSON.stringify(id)}: {${fields.join(", ")}}`;
}

</storycode:copy>
<a id="f-3f46decc2b79-serializeScene"></a>
### `serializeScene`

The canonical byte form a commit writes into the document.
Layout entries are emitted only for nodes the topology still has — an
orphaned entry is a claim about a node that no longer exists, and dropping
it here is what "orphans are cleaned on the next commit" means. Order is
the topology's own for nodes and edges (a generator's order is meaningful),
and node order for layout.

<storycode:copy id="f-3f46decc2b79-serializeScene">
/**
 * The canonical byte form a commit writes into the document.
 *
 * Layout entries are emitted only for nodes the topology still has — an
 * orphaned entry is a claim about a node that no longer exists, and dropping
 * it here is what "orphans are cleaned on the next commit" means. Order is
 * the topology's own for nodes and edges (a generator's order is meaningful),
 * and node order for layout.
 */
export function serializeScene(scene: Scene): string {
  const ids = scene.nodes.map((n) => n.id);
  const layoutLines = ids
    .filter((id) => scene.layout[id])
    .map((id) => `    ${layoutLine(id, scene.layout[id])}`);
  const layoutBlock = layoutLines.length
    ? `{\n${layoutLines.join(",\n")}\n  }`
    : "{}";
  if (scene.paste) {
    return `{\n  "topology": ${scene.paste},\n  "layout": ${layoutBlock}\n}\n`;
  }
  const nodesBlock = scene.nodes.length
    ? `[\n${scene.nodes.map((n) => `    ${nodeLine(n)}`).join(",\n")}\n  ]`
    : "[]";
  const edgesBlock = scene.edges.length
    ? `[\n${scene.edges.map((e) => `    ${edgeLine(e)}`).join(",\n")}\n  ]`
    : "[]";
  return `{\n  "nodes": ${nodesBlock},\n  "edges": ${edgesBlock},\n  "layout": ${layoutBlock}\n}\n`;
}

</storycode:copy>
<a id="f-3f46decc2b79-placeMissing"></a>
### `placeMissing`

Where a node without a recorded place goes: to the right of everything
placed so far, stacked downward — visibly "not yet arranged", never on top
of somebody's careful layout. Full auto-layout is a deliberate verb on the
toolbar, not something a re-run does to you.

<storycode:copy id="f-3f46decc2b79-placeMissing">
/** Where a node without a recorded place goes: to the right of everything
 * placed so far, stacked downward — visibly "not yet arranged", never on top
 * of somebody's careful layout. Full auto-layout is a deliberate verb on the
 * toolbar, not something a re-run does to you. */
export function placeMissing(
  nodes: SceneNode[],
  layout: Record<string, NodeLayout>,
): Record<string, NodeLayout> {
  const placed = nodes.filter((n) => layout[n.id]);
  const missing = nodes.filter((n) => !layout[n.id]);
  if (missing.length === 0) return layout;
  const right = placed.length
    ? Math.max(...placed.map((n) => layout[n.id].x + (layout[n.id].w ?? DEFAULT_W)))
    : 0;
  const top = placed.length ? Math.min(...placed.map((n) => layout[n.id].y)) : 0;
  const next = { ...layout };
  missing.forEach((node, i) => {
    next[node.id] = { x: right + 60, y: top + i * (DEFAULT_H + 24) };
  });
  return next;
}

</storycode:copy>
<a id="f-3f46decc2b79-DEFAULT_W"></a>
### `DEFAULT_W`

<storycode:copy id="f-3f46decc2b79-DEFAULT_W">
export const DEFAULT_W = 160;
</storycode:copy>
<a id="f-3f46decc2b79-DEFAULT_H"></a>
### `DEFAULT_H`

<storycode:copy id="f-3f46decc2b79-DEFAULT_H">
export const DEFAULT_H = 64;

</storycode:copy>
<a id="f-3f46decc2b79-GRID"></a>
### `GRID`

The canvas grid, in pixels: positions and sizes snap to it, and the dot
background draws it. One number so they can never disagree.

<storycode:copy id="f-3f46decc2b79-GRID">
/** The canvas grid, in pixels: positions and sizes snap to it, and the dot
 * background draws it. One number so they can never disagree. */
export const GRID = 16;

</storycode:copy>
<a id="f-3f46decc2b79-snap"></a>
### `snap`

<storycode:copy id="f-3f46decc2b79-snap">
/** Snap a coordinate or size to the grid. */
export function snap(value: number): number {
  return Math.round(value / GRID) * GRID;
}

</storycode:copy>
<a id="f-3f46decc2b79-Side"></a>
### `Side`

Connection slots: the points a side offers.

<storycode:copy id="f-3f46decc2b79-Side">
// ---------------------------------------------------------------------------
// Connection slots: the points a side offers.
// ---------------------------------------------------------------------------

export type Side = "top" | "right" | "bottom" | "left";
</storycode:copy>
<a id="f-3f46decc2b79-SIDES_ALL"></a>
### `SIDES_ALL`

<storycode:copy id="f-3f46decc2b79-SIDES_ALL">
export const SIDES_ALL: Side[] = ["top", "right", "bottom", "left"];

</storycode:copy>
<a id="f-3f46decc2b79-parseSideRef"></a>
### `parseSideRef`

A recorded side is `left` or `left.N` — side plus which slot on it.

<storycode:copy id="f-3f46decc2b79-parseSideRef">
/** A recorded side is `left` or `left.N` — side plus which slot on it. */
export function parseSideRef(value: string): { side: Side; slot: number } | null {
  const [side, slot] = value.split(".");
  if (!SIDES_ALL.includes(side as Side)) return null;
  const n = slot === undefined ? 0 : Number(slot);
  return Number.isInteger(n) && n >= 0 ? { side: side as Side, slot: n } : null;
}

</storycode:copy>
<a id="f-3f46decc2b79-sideRef"></a>
### `sideRef`

The handle id a slot renders as — slot 0 keeps the bare side name, so
every scene recorded before slots existed still means what it meant.

<storycode:copy id="f-3f46decc2b79-sideRef">
/** The handle id a slot renders as — slot 0 keeps the bare side name, so
 * every scene recorded before slots existed still means what it meant. */
export function sideRef(side: Side, slot: number): string {
  return slot === 0 ? side : `${side}.${slot}`;
}

</storycode:copy>
<a id="f-3f46decc2b79-occupiedSlots"></a>
### `occupiedSlots`

Which slots each side of each node has lines attached to.

<storycode:copy id="f-3f46decc2b79-occupiedSlots">
/** Which slots each side of each node has lines attached to. */
export function occupiedSlots(
  edges: SceneEdge[],
): Record<string, Partial<Record<Side, number[]>>> {
  const out: Record<string, Partial<Record<Side, number[]>>> = {};
  const add = (nodeId: string, ref: string | undefined) => {
    if (!ref) return;
    const parsed = parseSideRef(ref);
    if (!parsed) return;
    const sides = (out[nodeId] ??= {});
    const slots = (sides[parsed.side] ??= []);
    if (!slots.includes(parsed.slot)) slots.push(parsed.slot);
  };
  for (const edge of edges) {
    add(edge.from, edge.fromSide);
    add(edge.to, edge.toSide);
  }
  return out;
}

</storycode:copy>
<a id="f-3f46decc2b79-keyedEdges"></a>
### `keyedEdges`

Every edge with a key that is UNIQUE in this scene. An edge's own `id`
(or `from->to`) is the base; parallel edges without ids — hand-written,
or drawn before the editor minted ids for them — get `#2`, `#3` by
position, so each of three identical-looking lines stays individually
clickable, stylable, and editable instead of two of them shadowing the
third behind one identity.

<storycode:copy id="f-3f46decc2b79-keyedEdges">
/**
 * Every edge with a key that is UNIQUE in this scene. An edge's own `id`
 * (or `from->to`) is the base; parallel edges without ids — hand-written,
 * or drawn before the editor minted ids for them — get `#2`, `#3` by
 * position, so each of three identical-looking lines stays individually
 * clickable, stylable, and editable instead of two of them shadowing the
 * third behind one identity.
 */
export function keyedEdges(edges: SceneEdge[]): { key: string; edge: SceneEdge }[] {
  const seen = new Map<string, number>();
  return edges.map((edge) => {
    const base = edge.id ?? `${edge.from}->${edge.to}`;
    const n = (seen.get(base) ?? 0) + 1;
    seen.set(base, n);
    return { key: n === 1 ? base : `${base}#${n}`, edge };
  });
}

</storycode:copy>
<a id="f-3f46decc2b79-SLOT_SPACING"></a>
### `SLOT_SPACING`

<storycode:copy id="f-3f46decc2b79-SLOT_SPACING">
/** Distance between two bubbles on one side. */
export const SLOT_SPACING = 18;

</storycode:copy>
<a id="f-3f46decc2b79-STRAIGHT_BELOW"></a>
### `STRAIGHT_BELOW`

Below this distance between endpoints, a connector is drawn straight.

<storycode:copy id="f-3f46decc2b79-STRAIGHT_BELOW">
/** Below this distance between endpoints, a connector is drawn straight. */
export const STRAIGHT_BELOW = 96;
</storycode:copy>
<a id="f-3f46decc2b79-NEAR_AXIS"></a>
### `NEAR_AXIS`

Below this LATERAL shift, an elbowed connector cannot show its rounded
corners in full — it degenerates into a clipped little jog — so it is
drawn straight instead. The guarantee: every corner you see is a whole
corner. Just above one slot's spacing, so a line meeting the neighbouring
slot of an otherwise-aligned node stays clean.

<storycode:copy id="f-3f46decc2b79-NEAR_AXIS">
/** Below this LATERAL shift, an elbowed connector cannot show its rounded
 * corners in full — it degenerates into a clipped little jog — so it is
 * drawn straight instead. The guarantee: every corner you see is a whole
 * corner. Just above one slot's spacing, so a line meeting the neighbouring
 * slot of an otherwise-aligned node stays clean. */
export const NEAR_AXIS = 24;

</storycode:copy>
<a id="f-3f46decc2b79-straightConnector"></a>
### `straightConnector`

Whether a connector between these endpoints should be a straight line
rather than a rounded elbow: short lines always (between adjacent boxes
there is no room for corners), and opposite-facing lines whose lateral
offset is too small for the corners to render whole.

<storycode:copy id="f-3f46decc2b79-straightConnector">
/**
 * Whether a connector between these endpoints should be a straight line
 * rather than a rounded elbow: short lines always (between adjacent boxes
 * there is no room for corners), and opposite-facing lines whose lateral
 * offset is too small for the corners to render whole.
 */
export function straightConnector(
  sourceSide: string,
  targetSide: string,
  dx: number,
  dy: number,
): boolean {
  if (Math.hypot(dx, dy) < STRAIGHT_BELOW) return true;
  const h = (side: string) => side === "left" || side === "right";
  const v = (side: string) => side === "top" || side === "bottom";
  if (h(sourceSide) && h(targetSide) && Math.abs(dy) < NEAR_AXIS) return true;
  if (v(sourceSide) && v(targetSide) && Math.abs(dx) < NEAR_AXIS) return true;
  return false;
}

</storycode:copy>
<a id="f-3f46decc2b79-EXTRA_BEFORE"></a>
### `EXTRA_BEFORE`

The flanking free handles' id suffixes — never stored in the document:
a line dropped on one is renumbered into a real slot on commit.

<storycode:copy id="f-3f46decc2b79-EXTRA_BEFORE">
/** The flanking free handles' id suffixes — never stored in the document:
 * a line dropped on one is renumbered into a real slot on commit. */
export const EXTRA_BEFORE = "_before";
</storycode:copy>
<a id="f-3f46decc2b79-EXTRA_AFTER"></a>
### `EXTRA_AFTER`

<storycode:copy id="f-3f46decc2b79-EXTRA_AFTER">
export const EXTRA_AFTER = "_after";

</storycode:copy>
<a id="f-3f46decc2b79-sideHandles"></a>
### `sideHandles`

What one side draws. The OCCUPIED slots are centred on the side's midpoint
— the lines in use stay balanced, never pushed aside by a vacancy — and
one free handle flanks them on EACH end, symmetric, so taking either
flank keeps the group centred too. An empty side is one centred point.
Offsets are from the side's midpoint, in pixels.

<storycode:copy id="f-3f46decc2b79-sideHandles">
/**
 * What one side draws. The OCCUPIED slots are centred on the side's midpoint
 * — the lines in use stay balanced, never pushed aside by a vacancy — and
 * one free handle flanks them on EACH end, symmetric, so taking either
 * flank keeps the group centred too. An empty side is one centred point.
 * Offsets are from the side's midpoint, in pixels.
 */
export function sideHandles(
  side: Side,
  occupied: number[],
): { id: string; offset: number; extra: boolean }[] {
  if (occupied.length === 0) return [{ id: side, offset: 0, extra: false }];
  const slots = [...occupied].sort((a, b) => a - b);
  const k = slots.length;
  const taken = slots.map((slot, i) => ({
    id: sideRef(side, slot),
    offset: (i - (k - 1) / 2) * SLOT_SPACING,
    extra: false,
  }));
  const flank = ((k - 1) / 2 + 1) * SLOT_SPACING;
  return [
    { id: `${side}.${EXTRA_BEFORE}`, offset: -flank, extra: true },
    ...taken,
    { id: `${side}.${EXTRA_AFTER}`, offset: flank, extra: true },
  ];
}

</storycode:copy>
<a id="f-3f46decc2b79-dropRef"></a>
### `dropRef`

The stored ref for the handle a connection landed on. A real slot passes
through; a flank becomes a slot that ORDERS before or after everything
already there — temporarily negative on the before-flank, compacted by
[`renumberSides`] in the same commit.

<storycode:copy id="f-3f46decc2b79-dropRef">
/**
 * The stored ref for the handle a connection landed on. A real slot passes
 * through; a flank becomes a slot that ORDERS before or after everything
 * already there — temporarily negative on the before-flank, compacted by
 * [`renumberSides`] in the same commit.
 */
export function dropRef(handleId: string, occupied: number[]): string {
  const [side, suffix] = handleId.split(".");
  if (suffix === EXTRA_BEFORE) {
    return `${side}.${(occupied.length ? Math.min(...occupied) : 0) - 1}`;
  }
  if (suffix === EXTRA_AFTER) {
    return `${side}.${(occupied.length ? Math.max(...occupied) : -1) + 1}`;
  }
  return handleId;
}

</storycode:copy>
<a id="f-3f46decc2b79-SideEnd"></a>
### `SideEnd`

<storycode:copy id="f-3f46decc2b79-SideEnd">
type SideEnd = "fromSide" | "toSide";

</storycode:copy>
<a id="f-3f46decc2b79-renumberSides"></a>
### `renumberSides`

Compact every side's slots to 0..k-1, keeping their order. Position is a
function of ORDER, so this changes nothing on screen — it keeps the
document's refs contiguous and non-negative whatever was dropped, moved,
or deleted.

<storycode:copy id="f-3f46decc2b79-renumberSides">
/**
 * Compact every side's slots to 0..k-1, keeping their order. Position is a
 * function of ORDER, so this changes nothing on screen — it keeps the
 * document's refs contiguous and non-negative whatever was dropped, moved,
 * or deleted.
 */
export function renumberSides(edges: SceneEdge[]): SceneEdge[] {
  const next = edges.map((edge) => ({ ...edge }));
  const groups = new Map<string, { edge: SceneEdge; end: SideEnd; slot: number }[]>();
  for (const edge of next) {
    for (const end of ["fromSide", "toSide"] as SideEnd[]) {
      const ref = edge[end];
      if (!ref) continue;
      const [side, slotText] = ref.split(".");
      const slot = slotText === undefined ? 0 : Number(slotText);
      if (!Number.isInteger(slot)) continue;
      const key = `${end === "fromSide" ? edge.from : edge.to} ${side}`;
      const group = groups.get(key) ?? [];
      group.push({ edge, end, slot });
      groups.set(key, group);
    }
  }
  for (const group of groups.values()) {
    group.sort((a, b) => a.slot - b.slot);
    group.forEach((entry, i) => {
      const side = (entry.edge[entry.end] as string).split(".")[0] as Side;
      entry.edge[entry.end] = sideRef(side, i);
    });
  }
  return next;
}
</storycode:copy>

### Assemble `apps/web/src/components/graph/scene.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/scene.ts"><storycode:paste select="#f-3f46decc2b79-SceneNode"/><storycode:paste select="#f-3f46decc2b79-SceneEdge"/><storycode:paste select="#f-3f46decc2b79-NodeLayout"/><storycode:paste select="#f-3f46decc2b79-Scene"/><storycode:paste select="#f-3f46decc2b79-ParsedScene"/><storycode:paste select="#f-3f46decc2b79-PASTE_RE"/><storycode:paste select="#f-3f46decc2b79-parseSceneSource"/><storycode:paste select="#f-3f46decc2b79-withResolvedTopology"/><storycode:paste select="#f-3f46decc2b79-uncommittableText"/><storycode:paste select="#f-3f46decc2b79-jsonField"/><storycode:paste select="#f-3f46decc2b79-nodeLine"/><storycode:paste select="#f-3f46decc2b79-edgeLine"/><storycode:paste select="#f-3f46decc2b79-layoutLine"/><storycode:paste select="#f-3f46decc2b79-serializeScene"/><storycode:paste select="#f-3f46decc2b79-placeMissing"/><storycode:paste select="#f-3f46decc2b79-DEFAULT_W"/><storycode:paste select="#f-3f46decc2b79-DEFAULT_H"/><storycode:paste select="#f-3f46decc2b79-GRID"/><storycode:paste select="#f-3f46decc2b79-snap"/><storycode:paste select="#f-3f46decc2b79-Side"/><storycode:paste select="#f-3f46decc2b79-SIDES_ALL"/><storycode:paste select="#f-3f46decc2b79-parseSideRef"/><storycode:paste select="#f-3f46decc2b79-sideRef"/><storycode:paste select="#f-3f46decc2b79-occupiedSlots"/><storycode:paste select="#f-3f46decc2b79-keyedEdges"/><storycode:paste select="#f-3f46decc2b79-SLOT_SPACING"/><storycode:paste select="#f-3f46decc2b79-STRAIGHT_BELOW"/><storycode:paste select="#f-3f46decc2b79-NEAR_AXIS"/><storycode:paste select="#f-3f46decc2b79-straightConnector"/><storycode:paste select="#f-3f46decc2b79-EXTRA_BEFORE"/><storycode:paste select="#f-3f46decc2b79-EXTRA_AFTER"/><storycode:paste select="#f-3f46decc2b79-sideHandles"/><storycode:paste select="#f-3f46decc2b79-dropRef"/><storycode:paste select="#f-3f46decc2b79-SideEnd"/><storycode:paste select="#f-3f46decc2b79-renumberSides"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-9d6e99c5706d"></a>
## `apps/web/src/components/graph/scene.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Derived Diagram Keeps Your Layout](../../../../../../guarantees/authoring/a-derived-diagram-keeps-your-layout/contract.md), [A Drawn Diagram Is Document Text](../../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md).

<a id="f-9d6e99c5706d-HAND_DRAWN"></a>
### `HAND_DRAWN`

Protects docs/guarantees/authoring/a-drawn-diagram-is-document-text.md and
scene's byte form is the document, so its parse and its serialization are
the contract everything else stands on.

<storycode:copy id="f-9d6e99c5706d-HAND_DRAWN">
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

</storycode:copy>
<a id="f-9d6e99c5706d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9d6e99c5706d-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/components/graph/scene.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/scene.test.ts"><storycode:paste select="#f-9d6e99c5706d-HAND_DRAWN"/><storycode:paste select="#f-9d6e99c5706d-expression_statement"/></storycode:file>


</storycode:doc>
