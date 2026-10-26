<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Drawn Diagram Is Document Text

For: Authors editing executable documents and the files those documents produce.

Given a `<hick:diagram renderer="graph">` whose body is a JSON scene, when the
reader drags a node, connects or re-plugs an edge, adds, renames, resizes, or
deletes a node, restyles a selection from the palette, or applies auto-layout
in the app's graph editor, then the finished gesture is
written back into the document's own bytes as the scene's canonical
serialization (stable key order, one node/edge/layout entry per line, integer
positions) as a user edit — so undo undoes it, collaborators receive it, and a
diff of a diagram edit is a readable diff. When the document is woven, the
scene is drawn as an SVG output file (content-named, referenced as an image
from the woven markdown) carrying the author's positions, sizes, shapes and
colours, a body that does not parse weaves as its raw JSON rather than
nothing, and a scene string containing the literal `</hick:` is refused at
commit with a message saying why.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-drawn-diagram-is-document-text.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-e98d51bf9555"></a>
## `apps/web/e2e/uiGraph.spec.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Drawn Diagram Is Document Text](contract.md).

<a id="f-e98d51bf9555-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e98d51bf9555-expression_statement">
import { test, expect } from "@playwright/test";

// Guarantee: docs/guarantees/authoring/a-drawn-diagram-is-document-text.md
// Actual React Flow and native gestures; no paid provider, package or deployment witness.
test("native graph drag and label typing share the document with agent movement", async ({ page }) => {
  const unexpected: string[] = [];
  await page.route(url => url.pathname.startsWith("/api/"), route => { unexpected.push(route.request().url()); return route.abort(); });
  await page.goto("/embed.html?graph");
  const node = page.getByTestId("graph-node-api");
  await expect(node).toBeVisible();
  await node.dblclick();
  const label = node.getByRole("textbox");
  await expect(label).toBeVisible();
  await label.fill("Native API"); await label.press("Enter");
  await expect(node).toHaveText("Native API");
  const source = page.getByLabel("Graph document source");
  await expect(source).toContainText('"label": "Native API"');
  const before = await node.boundingBox();
  if (!before) throw Error("Node has no live canvas bounds");
  await page.mouse.move(before.x + before.width / 2, before.y + before.height / 2);
  await page.mouse.down(); await page.mouse.move(before.x + before.width / 2 + 70, before.y + before.height / 2 + 35, { steps: 8 }); await page.mouse.up();
  await expect.poll(async () => JSON.parse(await source.innerText()).layout.api.x).not.toBe(0);
  const dragged = await node.boundingBox();
  await page.getByRole("button", { name: "Agent move API" }).click();
  await expect(page.getByLabel("Graph action receipt")).toContainText('"commit_callback_requested":true');
  await expect.poll(async () => JSON.parse(await source.innerText()).layout.api).toMatchObject({ x: 300, y: 96 });
  await expect(node).toHaveText("Native API");
  const moved = await node.boundingBox();
  expect(moved?.x).not.toBe(dragged?.x);
  expect(unexpected).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/e2e/uiGraph.spec.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/e2e/uiGraph.spec.ts"><storycode:paste select="#f-e98d51bf9555-expression_statement"/></storycode:file>

<a id="output-b1b82d77ca55"></a>
## `apps/web/src/components/graph/GraphEditorPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Derived Diagram Keeps Your Layout](../a-derived-diagram-keeps-your-layout/contract.md), [A Drawn Diagram Is Document Text](contract.md).

<a id="f-b1b82d77ca55-GraphEditorPanelProps"></a>
### `GraphEditorPanelProps`

<storycode:copy id="f-b1b82d77ca55-GraphEditorPanelProps">
import { useWidgetEngagement } from "../useWidgetEngagement";
import type { UiActor } from "../../lib/uiActions";
// The interactive graph editor behind a `<hick:diagram renderer="graph">`.
//
// The document is the only store. Every committed gesture — a drag ended, an
// edge connected, a node added, renamed, or deleted, an auto-layout applied —
// serializes the scene canonically and writes it back into the document text,
// the exact contract the table grid has: the file stays the source of truth,
// and a diff of a diagram edit is a readable diff.
//
// Derived scenes (topology pasted from a generator's fragment) are TOPOLOGY
// READ-ONLY: drag and arrange all you like — commits rewrite only `layout` —
// but nodes and edges belong to the generator, and the next re-run keeps your
// layout precisely because it is keyed by semantic node id and stored apart.
//
// This file is loaded lazily, and the canvas library with it: the marketing
// site builds from this tree, and React Flow must never reach a page that
// shows no diagrams — the same reasoning as mermaid in DiagramPanel.

import { useCallback, useEffect, useRef, useState } from "react";
import {
  Background,
  ConnectionMode,
  MarkerType,
  ReactFlow,
  ReactFlowProvider,
  applyEdgeChanges,
  applyNodeChanges,
  useEdgesState,
  useNodesState,
  useReactFlow,
  useViewport,
} from "@xyflow/react";
import type { Connection, Edge, EdgeChange, Node, NodeChange, OnSelectionChangeParams } from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { DiagramAssertions } from "../DiagramPanel";
import type { DiagramPanelProps } from "../DiagramPanel";
import { SceneNodeView } from "./SceneNodeView";
import type { SceneNodeData } from "./SceneNodeView";
import { SceneEdgeView } from "./SceneEdgeView";
import { FILLS, SHAPES, STROKES } from "./palette";
import { autoLayout } from "./layout";
import { GraphInspectorLabel } from "./GraphInspectorLabel";
import { useGraphActions } from "./useGraphActions";
import {
  DEFAULT_H,
  DEFAULT_W,
  GRID,
  dropRef,
  keyedEdges,
  occupiedSlots,
  renumberSides,
  snap,
  parseSceneSource,
  placeMissing,
  serializeScene,
  uncommittableText,
  withResolvedTopology,
} from "./scene";
import type { NodeLayout, Scene, SceneEdge, SceneNode } from "./scene";

export interface GraphEditorPanelProps {
  /** The diagram body exactly as it stands in the document. */
  source: string;
  /** The server-resolved body, when the source holds a `<hick:paste>`. */
  resolved?: string | null;
  /** Ids named by `asserts`, with live pass/fail state. */
  assertions?: DiagramPanelProps["assertions"];
  /** Write the scene back into the document — the only save there is. */
  onCommit: (text: string, actor?: UiActor) => boolean | void;
  sourceCurrent?: () => boolean;
  sourceWritable?: () => boolean;
  sourceOwner?: object;
  sourceRevision?: () => object | null;
}

</storycode:copy>
<a id="f-b1b82d77ca55-NODE_TYPES"></a>
### `NODE_TYPES`

<storycode:copy id="f-b1b82d77ca55-NODE_TYPES">
const NODE_TYPES = { scene: SceneNodeView };
</storycode:copy>
<a id="f-b1b82d77ca55-EDGE_TYPES"></a>
### `EDGE_TYPES`

<storycode:copy id="f-b1b82d77ca55-EDGE_TYPES">
const EDGE_TYPES = { scene: SceneEdgeView };

</storycode:copy>
<a id="f-b1b82d77ca55-edgeKey"></a>
### `edgeKey`

The stable identity a React Flow edge shares with its scene edge.

<storycode:copy id="f-b1b82d77ca55-edgeKey">
/** The stable identity a React Flow edge shares with its scene edge. */
function edgeKey(edge: SceneEdge): string {
  return edge.id ?? `${edge.from}->${edge.to}`;
}

</storycode:copy>
<a id="f-b1b82d77ca55-flowNodes"></a>
### `flowNodes`

<storycode:copy id="f-b1b82d77ca55-flowNodes">
function flowNodes(
  scene: Scene,
  derived: boolean,
  onRename: (id: string, label: string, actor?: UiActor) => boolean | void,
  onResize: SceneNodeData["onResize"],
): Node[] {
  const layout = placeMissing(scene.nodes, scene.layout);
  const slots = occupiedSlots(scene.edges);
  return scene.nodes.map((node) => ({
    id: node.id,
    type: "scene",
    position: { x: layout[node.id].x, y: layout[node.id].y },
    // The wrapper is the ONE owner of a node's size — the resizer frame and
    // the connector positions align to it, so the visible box must never
    // out-size it with CSS minimums of its own.
    width: layout[node.id].w ?? DEFAULT_W,
    height: layout[node.id].h ?? DEFAULT_H,
    data: { node, derived, onRename, onResize, slots: slots[node.id] ?? {} } satisfies SceneNodeData,
  }));
}

</storycode:copy>
<a id="f-b1b82d77ca55-flowEdges"></a>
### `flowEdges`

<storycode:copy id="f-b1b82d77ca55-flowEdges">
function flowEdges(scene: Scene): Edge[] {
  return keyedEdges(scene.edges).map(({ key, edge }) => {
    const dashed = edge.style === "dashed" || edge.style === "dotted";
    const arrow = edge.arrow ?? "end";
    const marker = {
      type: MarkerType.ArrowClosed,
      width: 16,
      height: 16,
      ...(edge.color ? { color: edge.color } : {}),
    };
    const headAtEnd = arrow === "end" || arrow === "both";
    const headAtStart = arrow === "start" || arrow === "both";
    return {
      id: key,
      source: edge.from,
      target: edge.to,
      ...(edge.fromSide ? { sourceHandle: edge.fromSide } : {}),
      ...(edge.toSide ? { targetHandle: edge.toSide } : {}),
      type: "scene",
      ...(edge.label ? { label: edge.label } : {}),
      style: {
        ...(dashed ? { strokeDasharray: "6 4" } : {}),
        ...(edge.color ? { stroke: edge.color } : {}),
      },
      ...(headAtEnd ? { markerEnd: marker } : {}),
      ...(headAtStart ? { markerStart: marker } : {}),
      data: { edge },
    };
  });
}

</storycode:copy>
<a id="f-b1b82d77ca55-moveEdgeEnd"></a>
### `moveEdgeEnd`

The canvas height: the layout's own extent, within sane bounds — a
three-node sketch should not claim a screen, and a wide architecture
should not be a letterbox.
Move one end of an existing line to where a drag was dropped.
Exported for its own test: which end moves, and which end must not, is the
whole of the gesture's meaning, and it is a question about data rather than
about a canvas nobody can measure in jsdom.
The end that MOVES is the one that was picked up; the other keeps its node
and its slot, so an arrow points the way it pointed. The drop end is
whichever half of the connection is not the grabbed connector — the roles
React Flow assigns to `source`/`target` follow the direction of the DRAG,
which is the opposite of the line's own direction whenever the arrowhead is
what you grabbed.

<storycode:copy id="f-b1b82d77ca55-moveEdgeEnd">
/** The canvas height: the layout's own extent, within sane bounds — a
 * three-node sketch should not claim a screen, and a wide architecture
 * should not be a letterbox. */
/**
 * Move one end of an existing line to where a drag was dropped.
 *
 * Exported for its own test: which end moves, and which end must not, is the
 * whole of the gesture's meaning, and it is a question about data rather than
 * about a canvas nobody can measure in jsdom.
 *
 * The end that MOVES is the one that was picked up; the other keeps its node
 * and its slot, so an arrow points the way it pointed. The drop end is
 * whichever half of the connection is not the grabbed connector — the roles
 * React Flow assigns to `source`/`target` follow the direction of the DRAG,
 * which is the opposite of the line's own direction whenever the arrowhead is
 * what you grabbed.
 */
export function moveEdgeEnd(
  scene: Scene,
  moving: { key: string; grabbed: "from" | "to" },
  connection: Connection,
  commit: (next: Scene) => void,
): void {
  const keyed = keyedEdges(scene.edges);
  const held = keyed.find(({ key }) => key === moving.key);
  if (!held) return;
  const anchorNode = moving.grabbed === "from" ? held.edge.to : held.edge.from;
  const anchorSide = moving.grabbed === "from" ? held.edge.toSide : held.edge.fromSide;
  const ends = [
    { node: connection.source, handle: connection.sourceHandle },
    { node: connection.target, handle: connection.targetHandle },
  ];
  // The anchored end is still in the connection; the drop is the other one.
  // Compared on the SLOT, not just the node, so moving a line from one side
  // of a shape to another side of the SAME shape is a move like any other.
  const drop =
    ends.find((end) => !(end.node === anchorNode && end.handle === anchorSide)) ?? ends[1];
  if (!drop.node) return;

  // Occupancy WITHOUT the line being moved: its old seat is not taken for the
  // purpose of choosing its new one.
  const others = keyed.filter(({ key }) => key !== moving.key).map(({ edge }) => edge);
  const slots = occupiedSlots(others);
  const side = drop.handle?.split(".")[0];
  const seat = drop.handle
    ? dropRef(drop.handle, slots[drop.node]?.[side as keyof (typeof slots)[string]] ?? [])
    : undefined;

  commit({
    ...scene,
    edges: keyed.map(({ key, edge }) =>
      key === moving.key
        ? moving.grabbed === "from"
          ? { ...edge, from: drop.node!, fromSide: seat }
          : { ...edge, to: drop.node!, toSide: seat }
        : edge,
    ),
  });
}

</storycode:copy>
<a id="f-b1b82d77ca55-canvasHeight"></a>
### `canvasHeight`

<storycode:copy id="f-b1b82d77ca55-canvasHeight">
function canvasHeight(scene: Scene): number {
  const entries = Object.values(scene.layout);
  if (entries.length === 0) return 280;
  const bottom = Math.max(...entries.map((at) => at.y + (at.h ?? DEFAULT_H)));
  const top = Math.min(...entries.map((at) => at.y));
  return Math.min(640, Math.max(280, bottom - top + 140));
}

</storycode:copy>
<a id="f-b1b82d77ca55-GraphEditor"></a>
### `GraphEditor`

<storycode:copy id="f-b1b82d77ca55-GraphEditor">
function GraphEditor({ source, resolved, assertions, onCommit, sourceCurrent: hostCurrent, sourceWritable, sourceOwner, sourceRevision }: GraphEditorPanelProps) {
  // The last byte form THIS panel wrote. A source that comes back equal is
  // our own commit echoing through the registry; anything else is an external
  // edit — undo, a collaborator, the agent — and the document wins.
  const lastCommitted = useRef<string | null>(null);
  const sceneRef = useRef<Scene | null>(null);
  const currentInput = useRef({ source, resolved, hostCurrent, sourceWritable });
  currentInput.current = { source, resolved, hostCurrent, sourceWritable };
  const loadedInput = useRef<{ source: string; resolved: string | null | undefined } | null>(null);
  const sourceCurrent = () => {
    const loaded = loadedInput.current, live = currentInput.current;
    return (!live.hostCurrent || live.hostCurrent()) && !!loaded && live.resolved === loaded.resolved &&
      (live.source === loaded.source || live.source === lastCommitted.current);
  };
  const [parseError, setParseError] = useState<string | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);
  const [nodes, setNodes] = useNodesState<Node>([]);
  const [edges, setEdges] = useEdgesState<Edge>([]);
  const { fitView, setViewport } = useReactFlow();
  const viewport = useViewport();
  // The line whose label is being edited, right on the line (double-click).
  const [editingEdge, setEditingEdge] = useState<string | null>(null);
  // Node data closures are built when the scene loads; they read the live
  // handlers through these refs so a stale closure cannot commit through an
  // old scene.
  const renameRef = useRef<(id: string, label: string, actor?: UiActor) => boolean | void>(() => {});
  const resizeRef = useRef<SceneNodeData["onResize"]>(() => {});
  // Whether the reader is WORKING IN the canvas (focus is inside the panel).
  // The wheel belongs to whoever owns the moment: scrolling a document that
  // happens to contain a diagram must scroll the document, and only a canvas
  // you clicked into may turn the same gesture into zoom — the map-embedded-
  // in-a-page rule. Escape hands the wheel back.
  const engagement = useWidgetEngagement("graph canvas"), active = engagement.engaged;
  // What is selected on the canvas, for the inspector row. Ids only — the
  // scene stays the single source of everything else.
  const [selection, setSelection] = useState<{ nodes: string[]; edges: string[] }>({
    nodes: [],
    edges: [],
  });

  const commitScene = useCallback(
    (committed: Scene, actor?: UiActor) => {
      if (!sourceCurrent() || currentInput.current.sourceWritable?.() === false) {
        setRefusal("The document changed, closed or is read-only; wait for an editable current scene.");
        return false;
      }
      let scene = committed;
      const bad = uncommittableText(scene);
      if (bad !== null) {
        // The one string the no-escaping invariant cannot hold: written into
        // the body it would close the diagram from inside its own JSON.
        setRefusal(bad);
        return false;
      }
      setRefusal(null);
      // Keep every side's slots contiguous 0..k-1 whatever was dropped,
      // re-plugged, or deleted — position is a function of order, so this
      // never moves a line; it keeps the document's refs tidy. A derived
      // scene's edges are the fragment's and are left exactly as pasted.
      if (!scene.paste) scene = { ...scene, edges: renumberSides(scene.edges) };
      const text = `\n${serializeScene(scene)}`;
      const accepted = actor?.kind === "agent" ? onCommit(text, actor) : onCommit(text);
      if (accepted === false) { setRefusal("The document refused this graph edit; reload the current source."); return false; }
      sceneRef.current = scene;
      const nodeIds = new Set(scene.nodes.map(node => node.id));
      const edgeIds = new Set(keyedEdges(scene.edges).map(edge => edge.key));
      setSelection(current => ({ nodes: current.nodes.filter(id => nodeIds.has(id)), edges: current.edges.filter(id => edgeIds.has(id)) }));
      lastCommitted.current = text;
      // The rebuild must not cost the reader their selection: finishing a
      // resize (or applying a colour) is not "done with this node" — it is
      // usually the moment before the NEXT adjustment to the same node.
      setNodes((current) => {
        const kept = new Set(current.filter((n) => n.selected).map((n) => n.id));
        return flowNodes(scene, scene.paste !== null, renameRef.current, resizeRef.current).map(
          (n) => (kept.has(n.id) ? { ...n, selected: true } : n),
        );
      });
      setEdges((current) => {
        const kept = new Set(current.filter((e) => e.selected).map((e) => e.id));
        return flowEdges(scene).map((e) => (kept.has(e.id) ? { ...e, selected: true } : e));
      });
      return true;
    },
    [onCommit, setNodes, setEdges],
  );

  const rename = useCallback(
    (id: string, label: string, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !scene.nodes.some(node => node.id === id)) return;
      return commitScene({
        ...scene,
        nodes: scene.nodes.map((n) => (n.id === id ? { ...n, label } : n)),
      }, actor);
    },
    [commitScene],
  );
  renameRef.current = rename;
  const renameStable = useCallback((id: string, label: string, actor?: UiActor) => renameRef.current(id, label, actor), []);

  // A resize ended: snap the box to the grid — size and place both, so two
  // resized boxes line up without anyone squinting — and commit.
  const resize = useCallback(
    (id: string, at: { x: number; y: number; w: number; h: number }, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || !scene.nodes.some(node => node.id === id)) return;
      const snapped: NodeLayout = {
        x: snap(at.x),
        y: snap(at.y),
        w: Math.max(GRID * 6, snap(at.w)),
        h: Math.max(GRID * 3, snap(at.h)),
      };
      return commitScene({ ...scene, layout: { ...scene.layout, [id]: snapped } }, actor);
    },
    [commitScene],
  );
  resizeRef.current = resize;
  const resizeStable = useCallback<SceneNodeData["onResize"]>(
    (id, at) => resizeRef.current(id, at),
    [],
  );

  // Load from the document — on open, and whenever the document changed under
  // us in a way we did not write.
  useEffect(() => {
    const previous = loadedInput.current;
    loadedInput.current = { source, resolved };
    if (source === lastCommitted.current && previous?.resolved === resolved) return;
    const parsed = parseSceneSource(source);
    if (!parsed.ok) {
      setParseError(parsed.error);
      sceneRef.current = null;
      return;
    }
    setParseError(null);
    const scene = resolved
      ? withResolvedTopology(parsed.scene, resolved)
      : parsed.scene;
    sceneRef.current = scene;
    lastCommitted.current = source;
    setNodes(flowNodes(scene, scene.paste !== null, renameStable, resizeStable));
    setEdges(flowEdges(scene));
  }, [source, resolved, setNodes, setEdges, renameStable, resizeStable]);

  const derived = sceneRef.current?.paste != null;

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes((ns) => applyNodeChanges(changes, ns)),
    [setNodes],
  );
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((es) => applyEdgeChanges(changes, es)),
    [setEdges],
  );

  const moveNode = useCallback(
    (id: string, position: { x: number; y: number }, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || !scene.nodes.some(node => node.id === id)) return;
      const before = scene.layout[id];
      return commitScene({
        ...scene,
        layout: {
          ...scene.layout,
          [id]: {
            ...before,
            x: Math.round(position.x),
            y: Math.round(position.y),
          },
        },
      }, actor);
    },
    [commitScene],
  );

  const onNodeDragStop = useCallback((_event: unknown, node: Node) => moveNode(node.id, node.position), [moveNode]);

  // A drag that starts on an OCCUPIED connector moves that line rather than
  // drawing a second one from the same spot.
  //
  // A slot holds one line. Starting a new line from a taken slot could only
  // ever mean "put another line here", and there is already a gesture for
  // that — the flank handles, which make a new slot before or after the
  // occupants. So the drag with no other meaning gets the meaning people
  // expect: pick the line up by its end and put it somewhere else.
  //
  // Only when EXACTLY one line is attached. Two lines sharing a slot is not
  // something the editor produces, but a hand-written document can say it,
  // and silently moving one of two is worse than drawing a new one.
  const movingRef = useRef<{ key: string; grabbed: "from" | "to" } | null>(null);

  const onConnectStart = useCallback(
    (
      _event: unknown,
      { nodeId, handleId }: { nodeId: string | null; handleId: string | null },
    ) => {
      movingRef.current = null;
      const scene = sceneRef.current;
      if (!scene || scene.paste || !nodeId || !handleId) return;
      const attached = keyedEdges(scene.edges).filter(
        ({ edge }) =>
          (edge.from === nodeId && edge.fromSide === handleId) ||
          (edge.to === nodeId && edge.toSide === handleId),
      );
      if (attached.length !== 1) return;
      const { key, edge } = attached[0];
      movingRef.current = {
        key,
        // Which END was picked up decides which end moves — and the other
        // one stays put, so an arrow keeps pointing the way it pointed.
        grabbed: edge.from === nodeId && edge.fromSide === handleId ? "from" : "to",
      };
    },
    [],
  );

  // Always fires, including on a drag dropped over nothing — which must leave
  // the line exactly where it was. Nothing is committed until `onConnect`, so
  // forgetting the grab here is the whole of the cancel path.
  const onConnectEnd = useCallback(() => {
    movingRef.current = null;
  }, []);

  const connectNodes = useCallback(
    (connection: Connection, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !connection.source || !connection.target) return;
      // A flank handle becomes a slot that orders before or after the
      // side's occupants; commitScene compacts the numbers.
      const slots = occupiedSlots(scene.edges);
      const refFor = (nodeId: string, handle: string | null | undefined) => {
        if (!handle) return undefined;
        const side = handle.split(".")[0];
        return dropRef(handle, slots[nodeId]?.[side as keyof (typeof slots)[string]] ?? []);
      };
      const fromSide = refFor(connection.source, connection.sourceHandle);
      const toSide = refFor(connection.target, connection.targetHandle);
      const edge: SceneEdge = {
        from: connection.source,
        to: connection.target,
        ...(fromSide ? { fromSide } : {}),
        ...(toSide ? { toSide } : {}),
      };
      // More than one line between the same two shapes is legal; identity
      // has to say WHICH line, so a parallel newcomer gets its own id.
      const keys = new Set(keyedEdges(scene.edges).map(({ key }) => key));
      if (keys.has(edgeKey(edge))) {
        let n = 2;
        while (keys.has(`${edge.from}->${edge.to}#${n}`)) n += 1;
        edge.id = `${edge.from}->${edge.to}#${n}`;
      }
      return commitScene({ ...scene, edges: [...scene.edges, edge] }, actor);
    },
    [commitScene],
  );

  const onConnect = useCallback((connection: Connection) => {
    const scene = sceneRef.current;
    if (scene && !scene.paste && movingRef.current) moveEdgeEnd(scene, movingRef.current, connection, commitScene);
    else connectNodes(connection);
  }, [connectNodes, commitScene]);

  // Re-plugging a line: grab an edge near either END and drag it to another
  // node — or another side of the same node — instead of delete-and-redraw.
  // Topology for a derived scene belongs to its fragment, so there the
  // gesture is off (`edgesReconnectable` below).
  const reconnectEdge = useCallback(
    (id: string, connection: Connection, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !connection.source || !connection.target) return;
      // Occupancy WITHOUT the edge being moved: its old seat is not taken
      // for the purpose of choosing its new one.
      const others = keyedEdges(scene.edges)
        .filter(({ key }) => key !== id)
        .map(({ edge }) => edge);
      const slots = occupiedSlots(others);
      const refFor = (nodeId: string, handle: string | null | undefined) => {
        if (!handle) return undefined;
        const side = handle.split(".")[0];
        return dropRef(handle, slots[nodeId]?.[side as keyof (typeof slots)[string]] ?? []);
      };
      return commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map(({ key, edge }) =>
          key === id
            ? {
                ...edge,
                from: connection.source,
                to: connection.target,
                fromSide: refFor(connection.source, connection.sourceHandle),
                toSide: refFor(connection.target, connection.targetHandle),
              }
            : edge,
        ),
      }, actor);
    },
    [commitScene],
  );

  const onReconnect = useCallback((edge: Edge, connection: Connection) => reconnectEdge(edge.id, connection), [reconnectEdge]);

  const deleteNodes = useCallback(
    (ids: string[], actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      const gone = new Set(ids);
      return commitScene({
        ...scene,
        nodes: scene.nodes.filter((n) => !gone.has(n.id)),
        edges: scene.edges.filter((e) => !gone.has(e.from) && !gone.has(e.to)),
      }, actor);
    },
    [commitScene],
  );

  const onNodesDelete = useCallback((deleted: Node[]) => deleteNodes(deleted.map(node => node.id)), [deleteNodes]);

  const deleteEdges = useCallback(
    (ids: string[], actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      const gone = new Set(ids);
      return commitScene({
        ...scene,
        edges: keyedEdges(scene.edges)
          .filter(({ key }) => !gone.has(key))
          .map(({ edge }) => edge),
      }, actor);
    },
    [commitScene],
  );

  const onEdgesDelete = useCallback((deleted: Edge[]) => deleteEdges(deleted.map(edge => edge.id)), [deleteEdges]);

  const addNode = useCallback((actor?: UiActor) => {
    const scene = sceneRef.current;
    if (!scene || scene.paste) return;
    let n = scene.nodes.length + 1;
    while (scene.nodes.some((node) => node.id === `node-${n}`)) n += 1;
    const id = `node-${n}`;
    return commitScene({
      ...scene,
      nodes: [...scene.nodes, { id, label: id }],
    }, actor);
  }, [commitScene]);

  const applyAutoLayout = useCallback((actor?: UiActor) => {
    const scene = sceneRef.current;
    if (!scene) return;
    return commitScene({ ...scene, layout: autoLayout(scene) }, actor);
  }, [commitScene]);

  // Restyle every selected node. Styling lives on the node — topology — so a
  // derived scene refuses it (the inspector shows a note instead there).
  const patchNodes = useCallback(
    (ids: string[], patch: Partial<SceneNode>, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || ids.length === 0) return;
      const chosen = new Set(ids);
      return commitScene({
        ...scene,
        nodes: scene.nodes.map((n) => (chosen.has(n.id) ? { ...n, ...patch } : n)),
      }, actor);
    },
    [commitScene],
  );

  // Restyle every selected line — colour, dash, arrowheads, label.
  const styleNodes = (patch: Partial<SceneNode>) => patchNodes(selection.nodes, patch);
  const patchEdges = useCallback(
    (ids: string[], patch: Partial<SceneEdge>, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || ids.length === 0) return;
      const chosen = new Set(ids);
      return commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map(({ key, edge }) =>
          chosen.has(key) ? { ...edge, ...patch } : edge,
        ),
      }, actor);
    },
    [commitScene],
  );

  const styleEdges = (patch: Partial<SceneEdge>) => patchEdges(selection.edges, patch);

  // Commit one line's label from the on-the-line editor. By KEY, not by
  // selection: the double-clicked line need not be the selected one.
  const labelEdge = useCallback(
    (key: string, label: string, actor?: UiActor) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      const applied = commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map((entry) =>
          entry.key === key ? { ...entry.edge, label: label.trim() || undefined } : entry.edge,
        ),
      }, actor);
      if (applied) setEditingEdge(null);
      return applied;
    },
    [commitScene],
  );
  const labelEdgeRef = useRef(labelEdge);
  labelEdgeRef.current = labelEdge;
  const cancelLabelRef = useRef(() => setEditingEdge(null));

  const onEdgeDoubleClick = useCallback(
    (_event: unknown, edge: Edge) => {
      if (sceneRef.current?.paste) return;
      setEditingEdge(edge.id);
    },
    [],
  );

  // Editing travels to the one edge through its data, and the flag flips
  // without rebuilding the scene: the document has not changed yet.
  useEffect(() => {
    setEdges((current) =>
      current.map((e) => ({
        ...e,
        data: {
          ...e.data,
          editing: e.id === editingEdge,
          onLabel: (label: string, actor?: UiActor) => labelEdgeRef.current(e.id, label, actor),
          onCancel: () => cancelLabelRef.current(),
        },
      })),
    );
  }, [editingEdge, setEdges]);

  const onSelectionChange = useCallback(({ nodes, edges }: OnSelectionChangeParams) => {
    setSelection({
      nodes: nodes.map((n) => n.id),
      edges: edges.map((e) => e.id),
    });
  }, []);

  const invokeGraph = useGraphActions({
    scene: sceneRef.current, current: sourceCurrent, writable: () => currentInput.current.sourceWritable?.() !== false, source, resolved, selection, sourceOwner, sourceRevision,
    assertions, error: parseError ?? refusal, active, viewport, setViewport, commit: commitScene,
    rename, resize, move: moveNode, patchNodes, patchEdges, deleteNodes, deleteEdges,
    connect: connectNodes, reconnect: reconnectEdge, openEdgeLabel: setEditingEdge,
    select: (chosen) => {
      setSelection(chosen);
      setNodes(current => current.map(node => ({ ...node, selected: chosen.nodes.includes(node.id) })));
      setEdges(current => current.map(edge => ({ ...edge, selected: chosen.edges.includes(edge.id) })));
    },
    add: addNode, layout: applyAutoLayout, fit: () => fitView({ padding: 0.15, duration: 200 }),
  });

  if (parseError) {
    // Same posture as a mermaid diagram that does not parse: say why there is
    // no picture, and never touch the author's text.
    return (
      <div className="graph-editor graph-editor--broken">
        <p className="diagram-error" role="status">
          This diagram does not parse yet: {parseError}
        </p>
        <DiagramAssertions assertions={assertions} />
      </div>
    );
  }

  const height = sceneRef.current ? canvasHeight(sceneRef.current) : 280;
  // The inspector reads the FIRST selected thing for its current values;
  // its actions apply to the whole selection.
  const selectedNode = sceneRef.current?.nodes.find((n) => selection.nodes.includes(n.id));
  const selectedEdge = sceneRef.current
    ? keyedEdges(sceneRef.current.edges).find(({ key }) => selection.edges.includes(key))?.edge
    : undefined;
  return (
    <div
      className="graph-editor"
      data-testid="graph-editor"
      // The panel takes the focus when clicked. It lives portalled inside
      // the text editor's DOM, and clicking the canvas PANE moves focus
      // nowhere on its own — so the buffer kept it, and Delete pressed
      // while arranging the diagram erased document text at a caret nobody
      // was looking at. With focus in here, keys target the widget, which
      // both CodeMirror and the rendered-block guard already leave alone.
      tabIndex={-1}
      ref={engagement.root}
      {...engagement.handlers}
    >
      <div className="graph-editor__toolbar">
        {!derived && (
          <button type="button" onClick={() => invokeGraph("add")}>
            Add node
          </button>
        )}
        <button type="button" onClick={() => invokeGraph("layout")}>
          Auto-layout
        </button>
        <button
          type="button"
          data-tip="Bring the whole scene into view"
          onClick={() => invokeGraph("fit")}
        >
          Zoom to fit
        </button>
        {derived && (
          <span className="graph-editor__derived muted">
            Topology is derived — drag to arrange; nodes and edges come from
            the pasted fragment.
          </span>
        )}
        {refusal !== null && (
          <span className="graph-editor__refusal" role="status">
            Not saved: {refusal}
          </span>
        )}
      </div>
      {/* ALWAYS rendered, one fixed-height row: an inspector that appears
          only on selection changes the panel's height, and the panel's
          height IS the document's height there — so every click reflowed
          the prose below the diagram. The row is constant; only its
          contents follow the selection. */}
      <div className="graph-inspector">
        {!derived && selectedNode ? (
          <span className="graph-inspector__row" data-testid="node-inspector">
            <label className="graph-inspector__group">
              Shape
              <select
                aria-label="Shape"
                value={selectedNode.shape ?? "rect"}
                onChange={(e) =>
                  styleNodes({ shape: e.target.value === "rect" ? undefined : e.target.value })
                }
              >
                {SHAPES.map((shape) => (
                  <option key={shape} value={shape}>
                    {shape}
                  </option>
                ))}
              </select>
            </label>
            <Swatches label="Fill" colors={FILLS} onPick={(fill) => styleNodes({ fill })} />
            <Swatches label="Outline" colors={STROKES} onPick={(stroke) => styleNodes({ stroke })} />
            <Swatches label="Text" colors={STROKES} onPick={(text) => styleNodes({ text })} />
          </span>
        ) : !derived && selectedEdge ? (
          <span className="graph-inspector__row" data-testid="edge-inspector">
            <Swatches label="Line" colors={STROKES} onPick={(color) => styleEdges({ color })} />
            <button
              type="button"
              aria-label="Arrowheads"
              data-tip="Where the arrowheads go: one end, both, or none"
              onClick={() =>
                styleEdges({
                  arrow: NEXT_ARROW[selectedEdge.arrow ?? "end"],
                })
              }
            >
              {ARROW_GLYPH[selectedEdge.arrow ?? "end"]}
            </button>
            <button
              type="button"
              aria-label="Line style"
              onClick={() =>
                styleEdges({
                  style: NEXT_STYLE[selectedEdge.style ?? "solid"],
                })
              }
            >
              {selectedEdge.style ?? "solid"}
            </button>
            <GraphInspectorLabel key={edgeKey(selectedEdge)} identity={edgeKey(selectedEdge)}
              value={selectedEdge.label ?? ""}
              commit={(label, actor) => patchEdges([edgeKey(selectedEdge)], { label: label || undefined }, actor)} />
          </span>
        ) : (
          <span className="graph-inspector__hint muted">
            {derived
              ? "Sizes and positions are yours; shapes and colours come from the fragment."
              : "Select a box or a line to style it."}
          </span>
        )}
      </div>
      <div
        className={`graph-editor__canvas${active ? " graph-editor__canvas--active" : ""}`}
        style={{ height }}
      >
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={NODE_TYPES}
          edgeTypes={EDGE_TYPES}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeDragStop={onNodeDragStop}
          onConnect={onConnect}
          onConnectStart={onConnectStart}
          onConnectEnd={onConnectEnd}
          onReconnect={onReconnect}
          onNodesDelete={onNodesDelete}
          onEdgesDelete={onEdgesDelete}
          onEdgeDoubleClick={onEdgeDoubleClick}
          onSelectionChange={onSelectionChange}
          nodesConnectable={!derived}
          edgesReconnectable={!derived}
          deleteKeyCode={derived ? null : ["Backspace", "Delete"]}
          // A side is a place a line meets a box, not a polarity: any handle
          // accepts either end of a connection.
          connectionMode={ConnectionMode.Loose}
          // Grace. The cursor should be NEAR the thing, not on it: a drag
          // snaps to a handle from 36px out, an edge end is grabbable for
          // re-plugging from 24px, and an edge is clickable along a 24px
          // band rather than its one-pixel stroke.
          connectionRadius={36}
          reconnectRadius={24}
          defaultEdgeOptions={{ interactionWidth: 24 }}
          // Two gestures want the same pixels: a line ENDS exactly where a
          // node's connection dot sits, and the dot is on top. On an EMPTY
          // dot the drag draws a new line; on an OCCUPIED one it moves the
          // line already there (see `onConnectStart`), because a second line
          // from a taken slot is what the flank handles are for. Selection
          // still raises a line above the nodes, which is what makes its
          // end-grips reachable anywhere along the contested spot.
          elevateEdgesOnSelect
          // The wheel is the document's until the reader clicks into the
          // canvas (see the panel's focus handling above); zoom is what the
          // click buys, and Escape gives the wheel back.
          preventScrolling={active}
          zoomOnScroll={active}
          zoomOnPinch={active}
          // Boxes land ON the grid, not near it — dragging snaps, and the
          // resize commit snaps sizes to the same number the background
          // draws, so two boxes agree without anyone squinting.
          snapToGrid
          snapGrid={[GRID, GRID]}
          fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background gap={GRID} />
        </ReactFlow>
      </div>
      <DiagramAssertions assertions={assertions} />
    </div>
  );
}

</storycode:copy>
<a id="f-b1b82d77ca55-Swatches"></a>
### `Swatches`

One palette row: the colours, plus a "default" that clears back to the
theme's own.

<storycode:copy id="f-b1b82d77ca55-Swatches">
/** One palette row: the colours, plus a "default" that clears back to the
 * theme's own. */
function Swatches({
  label,
  colors,
  onPick,
}: {
  label: string;
  colors: { name: string; value: string }[];
  onPick: (value: string | undefined) => void;
}) {
  return (
    <span className="graph-inspector__group" role="group" aria-label={label}>
      {label}
      {colors.map((color) => (
        <button
          key={color.name}
          type="button"
          className="graph-swatch"
          style={{ background: color.value }}
          aria-label={`${label} ${color.name}`}
          onClick={() => onPick(color.value)}
        />
      ))}
      <button
        type="button"
        className="graph-swatch graph-swatch--none"
        aria-label={`${label} default`}
        data-tip="Back to the theme's own colour"
        onClick={() => onPick(undefined)}
      >
        ×
      </button>
    </span>
  );
}

</storycode:copy>
<a id="f-b1b82d77ca55-NEXT_ARROW"></a>
### `NEXT_ARROW`

<storycode:copy id="f-b1b82d77ca55-NEXT_ARROW">
const NEXT_ARROW: Record<string, string> = {
  end: "both",
  both: "start",
  start: "none",
  none: "end",
};
</storycode:copy>
<a id="f-b1b82d77ca55-ARROW_GLYPH"></a>
### `ARROW_GLYPH`

<storycode:copy id="f-b1b82d77ca55-ARROW_GLYPH">
const ARROW_GLYPH: Record<string, string> = {
  end: "\u2192",
  both: "\u2194",
  start: "\u2190",
  none: "\u2014",
};
</storycode:copy>
<a id="f-b1b82d77ca55-NEXT_STYLE"></a>
### `NEXT_STYLE`

<storycode:copy id="f-b1b82d77ca55-NEXT_STYLE">
const NEXT_STYLE: Record<string, string> = {
  solid: "dashed",
  dashed: "dotted",
  dotted: "solid",
};

</storycode:copy>
<a id="f-b1b82d77ca55-GraphEditorPanel"></a>
### `GraphEditorPanel`

Each mounted panel is its own React Flow store: N diagrams in one
document are N providers in N portals, and none can reach another.

<storycode:copy id="f-b1b82d77ca55-GraphEditorPanel">
/** Each mounted panel is its own React Flow store: N diagrams in one
 * document are N providers in N portals, and none can reach another. */
export function GraphEditorPanel(props: GraphEditorPanelProps) {
  return (
    <ReactFlowProvider>
      <GraphEditor {...props} />
    </ReactFlowProvider>
  );
}

</storycode:copy>
<a id="f-b1b82d77ca55-export_statement"></a>
### `export_statement`

<storycode:copy id="f-b1b82d77ca55-export_statement">
export default GraphEditorPanel;
</storycode:copy>

### Assemble `apps/web/src/components/graph/GraphEditorPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/GraphEditorPanel.tsx"><storycode:paste select="#f-b1b82d77ca55-GraphEditorPanelProps"/><storycode:paste select="#f-b1b82d77ca55-NODE_TYPES"/><storycode:paste select="#f-b1b82d77ca55-EDGE_TYPES"/><storycode:paste select="#f-b1b82d77ca55-edgeKey"/><storycode:paste select="#f-b1b82d77ca55-flowNodes"/><storycode:paste select="#f-b1b82d77ca55-flowEdges"/><storycode:paste select="#f-b1b82d77ca55-moveEdgeEnd"/><storycode:paste select="#f-b1b82d77ca55-canvasHeight"/><storycode:paste select="#f-b1b82d77ca55-GraphEditor"/><storycode:paste select="#f-b1b82d77ca55-Swatches"/><storycode:paste select="#f-b1b82d77ca55-NEXT_ARROW"/><storycode:paste select="#f-b1b82d77ca55-ARROW_GLYPH"/><storycode:paste select="#f-b1b82d77ca55-NEXT_STYLE"/><storycode:paste select="#f-b1b82d77ca55-GraphEditorPanel"/><storycode:paste select="#f-b1b82d77ca55-export_statement"/></storycode:file>

<a id="output-12ab5bc1217b"></a>
## `apps/web/src/components/graph/SceneEdgeView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Drawn Diagram Is Document Text](contract.md).

<a id="f-12ab5bc1217b-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
The line gets out of the way of its own label editor.
Found by dogfooding: a line running through the middle of the text box you
are typing in reads as a strikethrough — "deleted" said about the words
being written. The canvas library is mocked the way it is in
GraphEditorPanel.test.tsx: it wants a live browser to measure anything, and
what this test is about is which SVG the component asks for.

<storycode:copy id="f-12ab5bc1217b-expression_statement">
// @vitest-environment jsdom
// The line gets out of the way of its own label editor.
//
// Guarantee: docs/guarantees/authoring/a-drawn-diagram-is-document-text.md
//
// Found by dogfooding: a line running through the middle of the text box you
// are typing in reads as a strikethrough — "deleted" said about the words
// being written. The canvas library is mocked the way it is in
// GraphEditorPanel.test.tsx: it wants a live browser to measure anything, and
// what this test is about is which SVG the component asks for.
import { cleanup, render } from "@testing-library/react";
import { createPortal } from "react-dom";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("@xyflow/react", () => ({
  // The real BaseEdge spreads its extra props onto the `<path>`; the mock does
  // the same, because forwarding `mask` is exactly what is under test.
  BaseEdge: ({ path, mask, id }: { path: string; mask?: string; id: string }) => (
    <path data-testid="edge-path" data-edge={id} d={path} mask={mask} />
  ),
  // The real one portals the label OUT of the SVG, which matters here: an
  // <input> rendered inside an <svg> is an SVG element in jsdom, and the
  // editor's `select()` on mount would not exist on it.
  EdgeLabelRenderer: ({ children }: { children: React.ReactNode }) =>
    createPortal(children, document.body),
  getStraightPath: () => ["M0,0 L10,0", 50, 60],
  getSmoothStepPath: () => ["M0,0 L10,0", 50, 60],
  Position: { Top: "top", Left: "left", Right: "right", Bottom: "bottom" },
}));

</storycode:copy>
<a id="f-12ab5bc1217b-import_statement"></a>
### `import_statement`

<storycode:copy id="f-12ab5bc1217b-import_statement">
import { SceneEdgeView } from "./SceneEdgeView";

</storycode:copy>
<a id="f-12ab5bc1217b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-12ab5bc1217b-expression_statement-2">
afterEach(cleanup);

</storycode:copy>
<a id="f-12ab5bc1217b-EdgeViewProps"></a>
### `EdgeViewProps`

<storycode:copy id="f-12ab5bc1217b-EdgeViewProps">
type EdgeViewProps = React.ComponentProps<typeof SceneEdgeView>;

</storycode:copy>
<a id="f-12ab5bc1217b-props"></a>
### `props`

<storycode:copy id="f-12ab5bc1217b-props">
const props = (editing: boolean): EdgeViewProps =>
  ({
    id: "api->db",
    sourceX: 0,
    sourceY: 0,
    targetX: 100,
    targetY: 0,
    sourcePosition: "right",
    targetPosition: "left",
    data: { edge: { from: "api", to: "db", label: "writes" }, editing },
  }) as unknown as EdgeViewProps;

</storycode:copy>
<a id="f-12ab5bc1217b-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-12ab5bc1217b-expression_statement-3">
describe("a line being labelled", () => {
  it("opens a gap where the editor sits, instead of running through it", () => {
    const { container } = render(
      <svg>
        <SceneEdgeView {...props(true)} />
      </svg>,
    );
    const mask = container.querySelector("mask");
    expect(mask, "the gap is a mask, not a patch of background colour").not.toBeNull();
    // Two rects: everything shows, the label's box does not.
    const rects = mask!.querySelectorAll("rect");
    expect(rects).toHaveLength(2);
    expect(rects[0].getAttribute("fill")).toBe("white");
    expect(rects[1].getAttribute("fill")).toBe("black");
    // The hole is centred on the label position the path handed back.
    expect(Number(rects[1].getAttribute("x")) + Number(rects[1].getAttribute("width")) / 2).toBe(50);
    expect(Number(rects[1].getAttribute("y")) + Number(rects[1].getAttribute("height")) / 2).toBe(60);
    // And the path actually uses it — a mask nothing references is decoration.
    expect(container.querySelector("[data-testid=edge-path]")?.getAttribute("mask")).toBe(
      `url(#${mask!.id})`,
    );
  });

  it("leaves the line whole when nothing is being edited", () => {
    const { container } = render(
      <svg>
        <SceneEdgeView {...props(false)} />
      </svg>,
    );
    expect(container.querySelector("mask")).toBeNull();
    expect(container.querySelector("[data-testid=edge-path]")?.getAttribute("mask")).toBeNull();
  });
});

</storycode:copy>
<a id="f-12ab5bc1217b-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-12ab5bc1217b-expression_statement-4">
it("preserves an inline edge draft when its committed label changes", async () => {
  const { act, fireEvent, screen } = await import("@testing-library/react");
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onLabel = vi.fn();
  const mount = (label: string) => <UiActionContext.Provider value={registry}><svg><SceneEdgeView {...props(true)}
    data={{ edge: { from: "api", to: "db", label }, editing: true, onLabel }} /></svg></UiActionContext.Provider>;
  const view = render(mount("writes"));
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "draft" } });
  view.rerender(mount("external"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(a => a.label === "Commit graph label draft edge:api->db")!.handle, arguments: {}, request_id: "commit",
  }); });
  expect(result).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect((screen.getByRole("textbox") as HTMLInputElement).value).toBe("draft");
  expect(onLabel).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/components/graph/SceneEdgeView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/SceneEdgeView.test.tsx"><storycode:paste select="#f-12ab5bc1217b-expression_statement"/><storycode:paste select="#f-12ab5bc1217b-import_statement"/><storycode:paste select="#f-12ab5bc1217b-expression_statement-2"/><storycode:paste select="#f-12ab5bc1217b-EdgeViewProps"/><storycode:paste select="#f-12ab5bc1217b-props"/><storycode:paste select="#f-12ab5bc1217b-expression_statement-3"/><storycode:paste select="#f-12ab5bc1217b-expression_statement-4"/></storycode:file>

<a id="output-c95cccbf8dbc"></a>
## `apps/web/src/components/graph/SceneEdgeView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Drawn Diagram Is Document Text](contract.md).

<a id="f-c95cccbf8dbc-SceneEdgeData"></a>
### `SceneEdgeData`

<storycode:copy id="f-c95cccbf8dbc-SceneEdgeData">
import type { UiActor } from "../../lib/uiActions";
// One line on the canvas: colour, label, path — and, on double-click, the
// label editor right on the line.
//
// Rounded elbows read well across open space and badly between neighbours: a
// smoothstep between two boxes sitting next to each other folds its corner
// radii into an S in the little gap between them. So a SHORT line is drawn
// straight — the distance between two adjacent boxes' rims is small by
// definition — and everything longer keeps its rounded corners.

import { useEffect, useRef } from "react";
import { BaseEdge, EdgeLabelRenderer, getSmoothStepPath, getStraightPath } from "@xyflow/react";
import type { EdgeProps } from "@xyflow/react";

import { useGraphLabelDraft } from "./useGraphLabelDraft";
import { straightConnector } from "./scene";
import type { SceneEdge } from "./scene";

interface SceneEdgeData {
  edge?: SceneEdge;
  /** This line's label is being edited (double-click put it there). */
  editing?: boolean;
  onLabel?: (label: string, actor?: UiActor) => boolean | void;
  onCancel?: () => void;
}

</storycode:copy>
<a id="f-c95cccbf8dbc-SceneEdgeView"></a>
### `SceneEdgeView`

<storycode:copy id="f-c95cccbf8dbc-SceneEdgeView">
export function SceneEdgeView(props: EdgeProps) {
  const {
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    markerStart,
    markerEnd,
    style,
    label,
    interactionWidth,
    data,
  } = props;
  const { edge, editing, onLabel, onCancel } = (data ?? {}) as SceneEdgeData;
  const straight = straightConnector(
    sourcePosition ?? "right",
    targetPosition ?? "left",
    targetX - sourceX,
    targetY - sourceY,
  );
  const [path, labelX, labelY] = straight
    ? getStraightPath({ sourceX, sourceY, targetX, targetY })
    : getSmoothStepPath({
        sourceX,
        sourceY,
        targetX,
        targetY,
        sourcePosition,
        targetPosition,
        borderRadius: 8,
      });
  // While the label is being edited the line STOPS at the editor and picks up
  // again on the other side, instead of running behind it. The input is opaque,
  // so this is not about seeing through it — a line crossing the middle of a
  // text box reads as a strikethrough, which says "deleted" about the words you
  // are typing. A gap says "this label belongs to this line" instead.
  //
  // A mask rather than a patch of background colour: the canvas has its own
  // pattern behind it, and a filled rectangle would sit on it as a visible
  // hole. The mask travels with the path, so it is right at every zoom.
  const gapId = `edge-label-gap-${id}`;
  return (
    <>
      {editing && (
        <mask id={gapId} maskUnits="userSpaceOnUse">
          {/* White shows the path, black hides it. The cover is deliberately
              enormous: a mask's default region is the object's bounding box,
              and a path that is a horizontal line has zero height there. */}
          <rect x={-100000} y={-100000} width={200000} height={200000} fill="white" />
          <rect
            x={labelX - LABEL_GAP_WIDTH / 2}
            y={labelY - LABEL_GAP_HEIGHT / 2}
            width={LABEL_GAP_WIDTH}
            height={LABEL_GAP_HEIGHT}
            rx={4}
            fill="black"
          />
        </mask>
      )}
      <BaseEdge
        id={id}
        path={path}
        mask={editing ? `url(#${gapId})` : undefined}
        markerStart={markerStart}
        markerEnd={markerEnd}
        style={style}
        interactionWidth={interactionWidth}
        label={editing ? undefined : label}
        labelX={labelX}
        labelY={labelY}
        labelStyle={{
          fill: edge?.color ?? "var(--fg)",
          fontSize: 11,
        }}
        labelBgStyle={{ fill: "var(--bg-sunken)", fillOpacity: 0.85 }}
        labelBgPadding={[4, 2]}
        labelBgBorderRadius={3}
      />
      {editing && (
        <EdgeLabelRenderer>
          {/* The editor sits ON the line, where the words will live —
              not in a toolbar the eye has to travel to. */}
          <LabelInput
            identity={`edge:${id}`}
            x={labelX}
            y={labelY}
            initial={edge?.label ?? ""}
            onLabel={onLabel}
            onCancel={onCancel}
          />
        </EdgeLabelRenderer>
      )}
    </>
  );
}

</storycode:copy>
<a id="f-c95cccbf8dbc-LABEL_GAP_WIDTH"></a>
### `LABEL_GAP_WIDTH`

The hole the label editor gets in the line, in flow units.
Sized from `.graph-edge__label-input` — `width: 8rem` (128px, border-box)
and a ~17px tall box — plus a little clearance so the line does not kiss
the border. The label layer is transformed by the same viewport the edges
are, so these units track the input at any zoom.

<storycode:copy id="f-c95cccbf8dbc-LABEL_GAP_WIDTH">
/** The hole the label editor gets in the line, in flow units.
 *
 * Sized from `.graph-edge__label-input` — `width: 8rem` (128px, border-box)
 * and a ~17px tall box — plus a little clearance so the line does not kiss
 * the border. The label layer is transformed by the same viewport the edges
 * are, so these units track the input at any zoom. */
const LABEL_GAP_WIDTH = 136;
</storycode:copy>
<a id="f-c95cccbf8dbc-LABEL_GAP_HEIGHT"></a>
### `LABEL_GAP_HEIGHT`

<storycode:copy id="f-c95cccbf8dbc-LABEL_GAP_HEIGHT">
const LABEL_GAP_HEIGHT = 24;

</storycode:copy>
<a id="f-c95cccbf8dbc-LabelInput"></a>
### `LabelInput`

<storycode:copy id="f-c95cccbf8dbc-LabelInput">
function LabelInput({
  identity,
  x,
  y,
  initial,
  onLabel,
  onCancel,
}: {
  identity: string;
  x: number;
  y: number;
  initial: string;
  onLabel?: (label: string, actor?: UiActor) => boolean | void;
  onCancel?: () => void;
}) {
  const label = useGraphLabelDraft({ identity, value: initial, initiallyOpen: true,
    commit: (value, actor) => onLabel?.(value, actor), close: onCancel });
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    ref.current?.select();
  }, []);
  return (
    <>
    {label.error && <span role="alert">{label.error}</span>}
    <input
      ref={ref}
      className="graph-edge__label-input nodrag nopan"
      style={{
        position: "absolute",
        transform: `translate(-50%, -50%) translate(${x}px, ${y}px)`,
        pointerEvents: "all",
      }}
      aria-label="Line label"
      placeholder="label…"
      value={label.value}
      onChange={e => label.change(e.target.value)}
      autoFocus
      onBlur={() => label.finish(true)}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        if (e.key === "Escape") {
          e.stopPropagation();
          label.finish(false);
        }
      }}
    />
    </>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/graph/SceneEdgeView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/SceneEdgeView.tsx"><storycode:paste select="#f-c95cccbf8dbc-SceneEdgeData"/><storycode:paste select="#f-c95cccbf8dbc-SceneEdgeView"/><storycode:paste select="#f-c95cccbf8dbc-LABEL_GAP_WIDTH"/><storycode:paste select="#f-c95cccbf8dbc-LABEL_GAP_HEIGHT"/><storycode:paste select="#f-c95cccbf8dbc-LabelInput"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/graph/GraphEditorPanel.tsx](contract.md#output-b1b82d77ca55)
- [apps/web/src/components/graph/SceneEdgeView.tsx](contract.md#output-c95cccbf8dbc)
- [apps/web/src/components/graph/scene.ts](../../../mechanisms/apps/web/src/components/graph/scene.ts.md#output-3f46decc2b79)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/vite.site.config.ts](../../../mechanisms/apps/web/vite.site.config.ts.md#output-2cea88e563a6)
- [crates/hick-literate/src/scene.rs](../../../mechanisms/crates/hick-literate/src/scene.rs.md#output-34790cc193bc)
- [crates/hick-literate/src/weave.rs](../../../mechanisms/crates/hick-literate/src/weave.rs.md#output-7fd582622cb2)
- [crates/hickory-cli/src/lib.rs](../../../mechanisms/crates/hickory-cli/src/lib.rs.md#output-056f6a618871)

### Tests and fixtures

- [apps/web/e2e/uiGraph.spec.ts](contract.md#output-e98d51bf9555)
- [apps/web/src/components/graph/GraphEditorPanel.test.tsx](../../../mechanisms/apps/web/src/components/graph/GraphEditorPanel.test.tsx.md#output-b7404652ca7f)
- [apps/web/src/components/graph/SceneEdgeView.test.tsx](contract.md#output-12ab5bc1217b)
- [apps/web/src/components/graph/SceneNodeView.test.tsx](../../../mechanisms/apps/web/src/components/graph/SceneNodeView.tsx.md#output-d7bdcc63ee38)
- [apps/web/src/components/graph/scene.test.ts](../../../mechanisms/apps/web/src/components/graph/scene.ts.md#output-9d6e99c5706d)
- [crates/hickory-cli/tests/diagrams.rs](../../../mechanisms/crates/hickory-cli/tests/diagrams.rs.md#output-60028fe1d41a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
