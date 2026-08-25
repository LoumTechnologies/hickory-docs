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
  /** end (default), none, both. */
  arrow?: string;
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
export const DEFAULT_H = 56;
