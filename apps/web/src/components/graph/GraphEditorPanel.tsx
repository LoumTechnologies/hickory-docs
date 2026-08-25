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
  MarkerType,
  ReactFlow,
  ReactFlowProvider,
  applyEdgeChanges,
  applyNodeChanges,
  useEdgesState,
  useNodesState,
} from "@xyflow/react";
import type { Connection, Edge, EdgeChange, Node, NodeChange } from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import { api } from "../../api/client";
import { DiagramAssertions } from "../DiagramPanel";
import type { DiagramPanelProps } from "../DiagramPanel";
import { SceneNodeView } from "./SceneNodeView";
import type { SceneNodeData } from "./SceneNodeView";
import { autoLayout } from "./layout";
import {
  DEFAULT_H,
  parseSceneSource,
  placeMissing,
  serializeScene,
  uncommittableText,
  withResolvedTopology,
} from "./scene";
import type { Scene, SceneEdge } from "./scene";

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

function flowNodes(
  scene: Scene,
  derived: boolean,
  onRename: (id: string, label: string) => void,
): Node[] {
  const layout = placeMissing(scene.nodes, scene.layout);
  return scene.nodes.map((node) => ({
    id: node.id,
    type: "scene",
    position: { x: layout[node.id].x, y: layout[node.id].y },
    ...(layout[node.id].w !== undefined ? { width: layout[node.id].w } : {}),
    ...(layout[node.id].h !== undefined ? { height: layout[node.id].h } : {}),
    data: { node, derived, onRename } satisfies SceneNodeData,
  }));
}

function flowEdges(scene: Scene): Edge[] {
  return scene.edges.map((edge) => {
    const dashed = edge.style === "dashed" || edge.style === "dotted";
    const arrow = edge.arrow ?? "end";
    return {
      id: edge.id ?? `${edge.from}->${edge.to}`,
      source: edge.from,
      target: edge.to,
      type: "smoothstep",
      ...(edge.label ? { label: edge.label } : {}),
      ...(dashed ? { style: { strokeDasharray: "6 4" } } : {}),
      ...(arrow !== "none" ? { markerEnd: { type: MarkerType.ArrowClosed } } : {}),
      ...(arrow === "both" ? { markerStart: { type: MarkerType.ArrowClosed } } : {}),
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
  // rename handler through this ref so a stale closure cannot commit through
  // an old scene.
  const renameRef = useRef<(id: string, label: string) => void>(() => {});

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
      setNodes(flowNodes(scene, scene.paste !== null, renameRef.current));
      setEdges(flowEdges(scene));
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
    setNodes(flowNodes(scene, scene.paste !== null, renameStable));
    setEdges(flowEdges(scene));
  }, [source, resolved, setNodes, setEdges, renameStable]);

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
      const edge: SceneEdge = { from: connection.source, to: connection.target };
      commitScene({ ...scene, edges: [...scene.edges, edge] });
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

  // The deterministic generator: replace this scene's topology with what the
  // server deduces from the folder's code. Layout survives by construction —
  // it is keyed by id, surviving ids keep their places, and orphans are
  // dropped by the serializer. Hand-drawn scenes only: a derived scene's
  // topology already has an owner (its pasted fragment).
  const [generating, setGenerating] = useState(false);
  const generateFromCode = useCallback(async () => {
    const scene = sceneRef.current;
    if (!scene || scene.paste) return;
    setGenerating(true);
    try {
      const answer = await api.diagramTopology("dir");
      const current = sceneRef.current;
      if (!current || current.paste) return;
      commitScene({
        ...current,
        nodes: answer.topology.nodes,
        edges: answer.topology.edges,
      });
    } catch {
      // The generator is a convenience over an offline-capable local server;
      // a failure leaves the scene exactly as it was.
    } finally {
      setGenerating(false);
    }
  }, [commitScene]);

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
  return (
    <div className="graph-editor" data-testid="graph-editor">
      <div className="graph-editor__toolbar">
        {!derived && (
          <button type="button" onClick={addNode}>
            Add node
          </button>
        )}
        <button type="button" onClick={applyAutoLayout}>
          Auto-layout
        </button>
        {!derived && (
          <button type="button" onClick={generateFromCode} disabled={generating}>
            {generating ? "Reading the code…" : "Generate from code"}
          </button>
        )}
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
      <div className="graph-editor__canvas" style={{ height }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={NODE_TYPES}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeDragStop={onNodeDragStop}
          onConnect={onConnect}
          onNodesDelete={onNodesDelete}
          onEdgesDelete={onEdgesDelete}
          nodesConnectable={!derived}
          deleteKeyCode={derived ? null : ["Backspace", "Delete"]}
          fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background gap={16} />
        </ReactFlow>
      </div>
      <DiagramAssertions assertions={assertions} />
    </div>
  );
}

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
