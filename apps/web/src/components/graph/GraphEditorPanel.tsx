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
  onCommit: (text: string) => void;
}

const NODE_TYPES = { scene: SceneNodeView };
const EDGE_TYPES = { scene: SceneEdgeView };

/** The stable identity a React Flow edge shares with its scene edge. */
function edgeKey(edge: SceneEdge): string {
  return edge.id ?? `${edge.from}->${edge.to}`;
}

function flowNodes(
  scene: Scene,
  derived: boolean,
  onRename: (id: string, label: string) => void,
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

function canvasHeight(scene: Scene): number {
  const entries = Object.values(scene.layout);
  if (entries.length === 0) return 280;
  const bottom = Math.max(...entries.map((at) => at.y + (at.h ?? DEFAULT_H)));
  const top = Math.min(...entries.map((at) => at.y));
  return Math.min(640, Math.max(280, bottom - top + 140));
}

function GraphEditor({ source, resolved, assertions, onCommit }: GraphEditorPanelProps) {
  // The last byte form THIS panel wrote. A source that comes back equal is
  // our own commit echoing through the registry; anything else is an external
  // edit — undo, a collaborator, the agent — and the document wins.
  const lastCommitted = useRef<string | null>(null);
  const sceneRef = useRef<Scene | null>(null);
  const [parseError, setParseError] = useState<string | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);
  const [nodes, setNodes] = useNodesState<Node>([]);
  const [edges, setEdges] = useEdgesState<Edge>([]);
  const { fitView } = useReactFlow();
  // The line whose label is being edited, right on the line (double-click).
  const [editingEdge, setEditingEdge] = useState<string | null>(null);
  // Node data closures are built when the scene loads; they read the live
  // handlers through these refs so a stale closure cannot commit through an
  // old scene.
  const renameRef = useRef<(id: string, label: string) => void>(() => {});
  const resizeRef = useRef<SceneNodeData["onResize"]>(() => {});
  // Whether the reader is WORKING IN the canvas (focus is inside the panel).
  // The wheel belongs to whoever owns the moment: scrolling a document that
  // happens to contain a diagram must scroll the document, and only a canvas
  // you clicked into may turn the same gesture into zoom — the map-embedded-
  // in-a-page rule. Escape hands the wheel back.
  const [active, setActive] = useState(false);
  // What is selected on the canvas, for the inspector row. Ids only — the
  // scene stays the single source of everything else.
  const [selection, setSelection] = useState<{ nodes: string[]; edges: string[] }>({
    nodes: [],
    edges: [],
  });

  const commitScene = useCallback(
    (committed: Scene) => {
      let scene = committed;
      const bad = uncommittableText(scene);
      if (bad !== null) {
        // The one string the no-escaping invariant cannot hold: written into
        // the body it would close the diagram from inside its own JSON.
        setRefusal(bad);
        return;
      }
      setRefusal(null);
      // Keep every side's slots contiguous 0..k-1 whatever was dropped,
      // re-plugged, or deleted — position is a function of order, so this
      // never moves a line; it keeps the document's refs tidy. A derived
      // scene's edges are the fragment's and are left exactly as pasted.
      if (!scene.paste) scene = { ...scene, edges: renumberSides(scene.edges) };
      sceneRef.current = scene;
      const text = `\n${serializeScene(scene)}`;
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
      onCommit(text);
    },
    [onCommit, setNodes, setEdges],
  );

  const rename = useCallback(
    (id: string, label: string) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      commitScene({
        ...scene,
        nodes: scene.nodes.map((n) => (n.id === id ? { ...n, label } : n)),
      });
    },
    [commitScene],
  );
  renameRef.current = rename;
  const renameStable = useCallback((id: string, label: string) => renameRef.current(id, label), []);

  // A resize ended: snap the box to the grid — size and place both, so two
  // resized boxes line up without anyone squinting — and commit.
  const resize = useCallback(
    (id: string, at: { x: number; y: number; w: number; h: number }) => {
      const scene = sceneRef.current;
      if (!scene) return;
      const snapped: NodeLayout = {
        x: snap(at.x),
        y: snap(at.y),
        w: Math.max(GRID * 6, snap(at.w)),
        h: Math.max(GRID * 3, snap(at.h)),
      };
      commitScene({ ...scene, layout: { ...scene.layout, [id]: snapped } });
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
    if (source === lastCommitted.current) return;
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

  const onNodeDragStop = useCallback(
    (_event: unknown, node: Node) => {
      const scene = sceneRef.current;
      if (!scene) return;
      const before = scene.layout[node.id];
      commitScene({
        ...scene,
        layout: {
          ...scene.layout,
          [node.id]: {
            ...before,
            x: Math.round(node.position.x),
            y: Math.round(node.position.y),
          },
        },
      });
    },
    [commitScene],
  );

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

  const onConnect = useCallback(
    (connection: Connection) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !connection.source || !connection.target) return;
      const moving = movingRef.current;
      if (moving) {
        moveEdgeEnd(scene, moving, connection, commitScene);
        return;
      }
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
      commitScene({ ...scene, edges: [...scene.edges, edge] });
    },
    [commitScene],
  );

  // Re-plugging a line: grab an edge near either END and drag it to another
  // node — or another side of the same node — instead of delete-and-redraw.
  // Topology for a derived scene belongs to its fragment, so there the
  // gesture is off (`edgesReconnectable` below).
  const onReconnect = useCallback(
    (oldEdge: Edge, connection: Connection) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !connection.source || !connection.target) return;
      // Occupancy WITHOUT the edge being moved: its old seat is not taken
      // for the purpose of choosing its new one.
      const others = keyedEdges(scene.edges)
        .filter(({ key }) => key !== oldEdge.id)
        .map(({ edge }) => edge);
      const slots = occupiedSlots(others);
      const refFor = (nodeId: string, handle: string | null | undefined) => {
        if (!handle) return undefined;
        const side = handle.split(".")[0];
        return dropRef(handle, slots[nodeId]?.[side as keyof (typeof slots)[string]] ?? []);
      };
      commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map(({ key, edge }) =>
          key === oldEdge.id
            ? {
                ...edge,
                from: connection.source,
                to: connection.target,
                fromSide: refFor(connection.source, connection.sourceHandle),
                toSide: refFor(connection.target, connection.targetHandle),
              }
            : edge,
        ),
      });
    },
    [commitScene],
  );

  const onNodesDelete = useCallback(
    (deleted: Node[]) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      const gone = new Set(deleted.map((n) => n.id));
      commitScene({
        ...scene,
        nodes: scene.nodes.filter((n) => !gone.has(n.id)),
        edges: scene.edges.filter((e) => !gone.has(e.from) && !gone.has(e.to)),
      });
    },
    [commitScene],
  );

  const onEdgesDelete = useCallback(
    (deleted: Edge[]) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      const gone = new Set(deleted.map((e) => e.id));
      commitScene({
        ...scene,
        edges: keyedEdges(scene.edges)
          .filter(({ key }) => !gone.has(key))
          .map(({ edge }) => edge),
      });
    },
    [commitScene],
  );

  const addNode = useCallback(() => {
    const scene = sceneRef.current;
    if (!scene || scene.paste) return;
    let n = scene.nodes.length + 1;
    while (scene.nodes.some((node) => node.id === `node-${n}`)) n += 1;
    const id = `node-${n}`;
    commitScene({
      ...scene,
      nodes: [...scene.nodes, { id, label: id }],
    });
  }, [commitScene]);

  const applyAutoLayout = useCallback(() => {
    const scene = sceneRef.current;
    if (!scene) return;
    commitScene({ ...scene, layout: autoLayout(scene) });
  }, [commitScene]);

  // Restyle every selected node. Styling lives on the node — topology — so a
  // derived scene refuses it (the inspector shows a note instead there).
  const styleNodes = useCallback(
    (patch: Partial<SceneNode>) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || selection.nodes.length === 0) return;
      const chosen = new Set(selection.nodes);
      commitScene({
        ...scene,
        nodes: scene.nodes.map((n) => (chosen.has(n.id) ? { ...n, ...patch } : n)),
      });
    },
    [commitScene, selection],
  );

  // Restyle every selected line — colour, dash, arrowheads, label.
  const styleEdges = useCallback(
    (patch: Partial<SceneEdge>) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || selection.edges.length === 0) return;
      const chosen = new Set(selection.edges);
      commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map(({ key, edge }) =>
          chosen.has(key) ? { ...edge, ...patch } : edge,
        ),
      });
    },
    [commitScene, selection],
  );

  // Commit one line's label from the on-the-line editor. By KEY, not by
  // selection: the double-clicked line need not be the selected one.
  const labelEdge = useCallback(
    (key: string, label: string) => {
      setEditingEdge(null);
      const scene = sceneRef.current;
      if (!scene || scene.paste) return;
      commitScene({
        ...scene,
        edges: keyedEdges(scene.edges).map((entry) =>
          entry.key === key ? { ...entry.edge, label: label.trim() || undefined } : entry.edge,
        ),
      });
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
          onLabel: (label: string) => labelEdgeRef.current(e.id, label),
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
      onFocus={() => setActive(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Element | null)) {
          setActive(false);
        }
      }}
      onKeyDown={(event) => {
        // Escape is "I am done in here": the wheel scrolls the document
        // again. But not when a field inside is using Escape for its own
        // cancel (the rename input): that Escape means "undo my typing".
        const target = event.target as HTMLElement;
        if (event.key === "Escape" && !target.closest("input, textarea")) {
          (event.currentTarget as HTMLElement).blur();
        }
      }}
      onPointerDownCapture={(event) => {
        const root = event.currentTarget;
        const target = event.target as HTMLElement;
        // Never steal from a field someone is typing in (the rename input).
        if (target.closest("input, textarea, [contenteditable]")) return;
        if (!root.contains(document.activeElement)) root.focus({ preventScroll: true });
      }}
    >
      <div className="graph-editor__toolbar">
        {!derived && (
          <button type="button" onClick={addNode}>
            Add node
          </button>
        )}
        <button type="button" onClick={applyAutoLayout}>
          Auto-layout
        </button>
        <button
          type="button"
          data-tip="Bring the whole scene into view"
          onClick={() => void fitView({ padding: 0.15, duration: 200 })}
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
            Not saved: a diagram may not contain the text “&lt;/hick:” — it
            would end the element from inside it.
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
            <input
              aria-label="Line label"
              className="graph-inspector__label"
              placeholder="label…"
              key={edgeKey(selectedEdge)}
              defaultValue={selectedEdge.label ?? ""}
              onBlur={(e) => styleEdges({ label: e.target.value.trim() || undefined })}
              onKeyDown={(e) => {
                if (e.key === "Enter") (e.target as HTMLInputElement).blur();
              }}
            />
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

const NEXT_ARROW: Record<string, string> = {
  end: "both",
  both: "start",
  start: "none",
  none: "end",
};
const ARROW_GLYPH: Record<string, string> = {
  end: "\u2192",
  both: "\u2194",
  start: "\u2190",
  none: "\u2014",
};
const NEXT_STYLE: Record<string, string> = {
  solid: "dashed",
  dashed: "dotted",
  dotted: "solid",
};

/** Each mounted panel is its own React Flow store: N diagrams in one
 * document are N providers in N portals, and none can reach another. */
export function GraphEditorPanel(props: GraphEditorPanelProps) {
  return (
    <ReactFlowProvider>
      <GraphEditor {...props} />
    </ReactFlowProvider>
  );
}

export default GraphEditorPanel;
