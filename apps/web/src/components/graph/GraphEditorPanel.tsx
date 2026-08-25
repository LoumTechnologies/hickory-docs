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
  GRID,
  occupiedSlots,
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
    ...(layout[node.id].w !== undefined ? { width: layout[node.id].w } : {}),
    ...(layout[node.id].h !== undefined ? { height: layout[node.id].h } : {}),
    data: { node, derived, onRename, onResize, slots: slots[node.id] ?? {} } satisfies SceneNodeData,
  }));
}

function flowEdges(scene: Scene): Edge[] {
  return scene.edges.map((edge) => {
    const dashed = edge.style === "dashed" || edge.style === "dotted";
    const arrow = edge.arrow ?? "end";
    const marker = {
      type: MarkerType.ArrowClosed,
      width: 16,
      height: 16,
      ...(edge.color ? { color: edge.color } : {}),
    };
    return {
      id: edgeKey(edge),
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
      ...(arrow !== "none" ? { markerEnd: marker } : {}),
      ...(arrow === "both" ? { markerStart: marker } : {}),
      data: { edge },
    };
  });
}

/** The canvas height: the layout's own extent, within sane bounds — a
 * three-node sketch should not claim a screen, and a wide architecture
 * should not be a letterbox. */
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
  // Node data closures are built when the scene loads; they read the live
  // handlers through these refs so a stale closure cannot commit through an
  // old scene.
  const renameRef = useRef<(id: string, label: string) => void>(() => {});
  const resizeRef = useRef<SceneNodeData["onResize"]>(() => {});
  // What is selected on the canvas, for the inspector row. Ids only — the
  // scene stays the single source of everything else.
  const [selection, setSelection] = useState<{ nodes: string[]; edges: string[] }>({
    nodes: [],
    edges: [],
  });

  const commitScene = useCallback(
    (scene: Scene) => {
      const bad = uncommittableText(scene);
      if (bad !== null) {
        // The one string the no-escaping invariant cannot hold: written into
        // the body it would close the diagram from inside its own JSON.
        setRefusal(bad);
        return;
      }
      setRefusal(null);
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

  const onConnect = useCallback(
    (connection: Connection) => {
      const scene = sceneRef.current;
      if (!scene || scene.paste || !connection.source || !connection.target) return;
      const edge: SceneEdge = {
        from: connection.source,
        to: connection.target,
        ...(connection.sourceHandle ? { fromSide: connection.sourceHandle } : {}),
        ...(connection.targetHandle ? { toSide: connection.targetHandle } : {}),
      };
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
      commitScene({
        ...scene,
        edges: scene.edges.map((edge) =>
          (edge.id ?? `${edge.from}->${edge.to}`) === oldEdge.id
            ? {
                ...edge,
                from: connection.source,
                to: connection.target,
                ...(connection.sourceHandle
                  ? { fromSide: connection.sourceHandle }
                  : { fromSide: undefined }),
                ...(connection.targetHandle
                  ? { toSide: connection.targetHandle }
                  : { toSide: undefined }),
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
        edges: scene.edges.filter((e) => !gone.has(e.id ?? `${e.from}->${e.to}`)),
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
        edges: scene.edges.map((e) => (chosen.has(edgeKey(e)) ? { ...e, ...patch } : e)),
      });
    },
    [commitScene, selection],
  );

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
  const selectedEdge = sceneRef.current?.edges.find((e) =>
    selection.edges.includes(edgeKey(e)),
  );
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
      <div className="graph-editor__canvas" style={{ height }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={NODE_TYPES}
          edgeTypes={EDGE_TYPES}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeDragStop={onNodeDragStop}
          onConnect={onConnect}
          onReconnect={onReconnect}
          onNodesDelete={onNodesDelete}
          onEdgesDelete={onEdgesDelete}
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
          // node's connection dot sits, and the dot is on top, so a bare
          // drag there draws a NEW line. Selection is the disambiguator:
          // click the line first and it is raised above the nodes, so its
          // end-grips win the contested spot and the same drag RE-PLUGS it.
          elevateEdgesOnSelect
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

const NEXT_ARROW: Record<string, string> = { end: "both", both: "none", none: "end" };
const ARROW_GLYPH: Record<string, string> = { end: "→", both: "↔", none: "—" };
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
