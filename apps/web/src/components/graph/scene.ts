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

export interface NodeLayout {
  x: number;
  y: number;
  w?: number;
  h?: number;
}

export interface Scene {
  nodes: SceneNode[];
  edges: SceneEdge[];
  layout: Record<string, NodeLayout>;
  /** The literal `<hick:paste …/>` text, when the topology is derived. The
   * commit must write it back byte-for-byte — it is the document's record of
   * where the topology comes from. */
  paste: string | null;
}

export type ParsedScene =
  | { ok: true; scene: Scene }
  | { ok: false; error: string };

const PASTE_RE = /<hick:paste(?:\s+(?:"[^"]*"|'[^']*'|[^<>"'])*)?\/>/;

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

/** Fold the server-resolved body's topology into a derived scene. The raw
 * source only says WHERE the topology comes from; the resolved body says what
 * it currently is. */
export function withResolvedTopology(scene: Scene, resolved: string): Scene {
  if (!scene.paste) return scene;
  const parsed = parseSceneSource(resolved);
  if (!parsed.ok) return scene;
  return { ...scene, nodes: parsed.scene.nodes, edges: parsed.scene.edges };
}

/** The one string a document body cannot contain (no-escaping invariant): a
 * label carrying `</hick:` would close the diagram from inside its own JSON. */
export function uncommittableText(scene: Scene): string | null {
  const strings = [
    ...scene.nodes.flatMap((n) => [n.id, n.label ?? "", n.shape ?? "", n.fill ?? "", n.stroke ?? ""]),
    ...scene.edges.flatMap((e) => [e.id ?? "", e.from, e.to, e.label ?? "", e.style ?? "", e.arrow ?? ""]),
  ];
  return strings.find((s) => s.includes("</hick:")) ?? null;
}

function jsonField(key: string, value: string | undefined): string | null {
  return value === undefined || value === "" ? null : `${JSON.stringify(key)}: ${JSON.stringify(value)}`;
}

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

function layoutLine(id: string, at: NodeLayout): string {
  const fields = [
    `"x": ${Math.round(at.x)}`,
    `"y": ${Math.round(at.y)}`,
    ...(at.w !== undefined ? [`"w": ${Math.round(at.w)}`] : []),
    ...(at.h !== undefined ? [`"h": ${Math.round(at.h)}`] : []),
  ];
  return `${JSON.stringify(id)}: {${fields.join(", ")}}`;
}

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

export const DEFAULT_W = 160;
export const DEFAULT_H = 64;

/** The canvas grid, in pixels: positions and sizes snap to it, and the dot
 * background draws it. One number so they can never disagree. */
export const GRID = 16;

/** Snap a coordinate or size to the grid. */
export function snap(value: number): number {
  return Math.round(value / GRID) * GRID;
}

// ---------------------------------------------------------------------------
// Connection slots: the points a side offers.
// ---------------------------------------------------------------------------

export type Side = "top" | "right" | "bottom" | "left";
export const SIDES_ALL: Side[] = ["top", "right", "bottom", "left"];

/** A recorded side is `left` or `left.N` — side plus which slot on it. */
export function parseSideRef(value: string): { side: Side; slot: number } | null {
  const [side, slot] = value.split(".");
  if (!SIDES_ALL.includes(side as Side)) return null;
  const n = slot === undefined ? 0 : Number(slot);
  return Number.isInteger(n) && n >= 0 ? { side: side as Side, slot: n } : null;
}

/** The handle id a slot renders as — slot 0 keeps the bare side name, so
 * every scene recorded before slots existed still means what it meant. */
export function sideRef(side: Side, slot: number): string {
  return slot === 0 ? side : `${side}.${slot}`;
}

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

/** Distance between two bubbles on one side. */
export const SLOT_SPACING = 18;

/** Below this distance between endpoints, a connector is drawn straight. */
export const STRAIGHT_BELOW = 96;
/** Below this LATERAL shift, an elbowed connector cannot show its rounded
 * corners in full — it degenerates into a clipped little jog — so it is
 * drawn straight instead. The guarantee: every corner you see is a whole
 * corner. Just above one slot's spacing, so a line meeting the neighbouring
 * slot of an otherwise-aligned node stays clean. */
export const NEAR_AXIS = 24;

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

/** The flanking free handles' id suffixes — never stored in the document:
 * a line dropped on one is renumbered into a real slot on commit. */
export const EXTRA_BEFORE = "_before";
export const EXTRA_AFTER = "_after";

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

type SideEnd = "fromSide" | "toSide";

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
