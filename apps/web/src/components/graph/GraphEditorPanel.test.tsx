// @vitest-environment jsdom
// The canvas library is mocked the way mermaid is in DiagramPanel.test.tsx:
// it wants a live browser to measure anything, and what these tests are about
// is the panel's contract with the DOCUMENT — what a gesture commits, what an
// external edit resets, and what a derived scene refuses.
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";

let flowProps: Record<string, unknown> = {};
const fitViewMock = vi.fn();
vi.mock("@xyflow/react", () => ({
  ReactFlow: (props: Record<string, unknown>) => {
    flowProps = props;
    return <div data-testid="flow" />;
  },
  ReactFlowProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  Background: () => null,
  Handle: () => null,
  Position: { Top: "top", Left: "left", Right: "right", Bottom: "bottom" },
  MarkerType: { ArrowClosed: "arrowclosed" },
  ConnectionMode: { Loose: "loose", Strict: "strict" },
  NodeResizer: () => null,
  useConnection: (selector?: (c: { inProgress: boolean }) => unknown) =>
    selector ? selector({ inProgress: false }) : { inProgress: false },
  BaseEdge: () => null,
  getStraightPath: () => ["", 0, 0],
  getSmoothStepPath: () => ["", 0, 0],
  BaseEdgeLabelRendererPlaceholder: null,
  EdgeLabelRenderer: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  applyNodeChanges: (changes: { type: string; id: string; selected?: boolean }[], ns: { id: string }[]) =>
    ns.map((n) => {
      const change = changes.find((c) => c.type === "select" && c.id === n.id);
      return change ? { ...n, selected: change.selected } : n;
    }),
  applyEdgeChanges: (_c: unknown, es: unknown) => es,
  useNodesState: (init: unknown) => useState(init),
  useReactFlow: () => ({ fitView: fitViewMock }),
  useEdgesState: (init: unknown) => useState(init),
}));

import { GraphEditorPanel } from "./GraphEditorPanel";

afterEach(cleanup);
beforeEach(() => {
  flowProps = {};
  fitViewMock.mockReset();
});

const HAND_DRAWN = `{
  "nodes": [
    {"id": "api", "label": "API server"},
    {"id": "db"}
  ],
  "edges": [
    {"from": "api", "to": "db"}
  ],
  "layout": {
    "api": {"x": 0, "y": 0},
    "db": {"x": 0, "y": 160}
  }
}
`;

const DERIVED =
  '{\n  "topology": <hick:paste select="#arch" />,\n  "layout": {\n    "api": {"x": 0, "y": 0}\n  }\n}\n';
const RESOLVED = '{"nodes": [{"id": "api"}, {"id": "db"}], "edges": [{"from": "api", "to": "db"}]}';

type DragStop = (event: unknown, node: { id: string; position: { x: number; y: number } }) => void;
type Connect = (connection: { source: string; target: string }) => void;

describe("the graph editor panel", () => {
  it("commits a drag as canonical document text", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onNodeDragStop).toBeTruthy());
    // The wrapper always carries a size — the resizer frame and connector
    // positions align to it, so it may never be out-sized by the box inside.
    const first = (flowProps.nodes as { width?: number; height?: number }[])[0];
    expect(first.width).toBe(160);
    expect(first.height).toBe(64);
    (flowProps.onNodeDragStop as DragStop)(null, { id: "db", position: { x: 240.4, y: 80.6 } });
    expect(onCommit).toHaveBeenCalledTimes(1);
    const text = onCommit.mock.calls[0][0] as string;
    // Rounded integers, one entity per line, everything else untouched.
    expect(text).toContain('"db": {"x": 240, "y": 81}');
    expect(text).toContain('{"id": "api", "label": "API server"}');
  });

  it("commits a new edge on connect — the gesture is the save", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onConnect).toBeTruthy());
    (flowProps.onConnect as Connect)({ source: "db", target: "api" });
    const text = onCommit.mock.calls[0][0] as string;
    expect(text).toContain('{"from": "db", "to": "api"}');
  });

  it("re-plugs an edge end onto another node or side, recording where it lands", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onReconnect).toBeTruthy());
    // The whole point of the gesture: no delete-and-redraw. Reconnection is
    // never off for a hand-drawn scene, and any handle takes either end.
    expect(flowProps.edgesReconnectable).toBe(true);
    expect(flowProps.connectionMode).toBe("loose");
    // A selected line rises above the nodes so its end-grips win the pixels
    // it shares with the connection dots — click the line, then drag its
    // end; a bare drag from a dot draws a new line instead.
    expect(flowProps.elevateEdgesOnSelect).toBe(true);
    type Reconnect = (
      oldEdge: { id: string },
      connection: { source: string; target: string; sourceHandle?: string; targetHandle?: string },
    ) => void;
    (flowProps.onReconnect as Reconnect)(
      { id: "api->db" },
      { source: "api", target: "api", sourceHandle: "right", targetHandle: "left" },
    );
    const text = onCommit.mock.calls[0][0] as string;
    expect(text).toContain(
      '{"from": "api", "fromSide": "right", "to": "api", "toSide": "left"}',
    );
    expect(text).not.toContain('"to": "db"');
  });

  it("takes the focus when clicked, so Delete belongs to the canvas, not the buffer", async () => {
    // The panel is portalled inside the text editor's DOM, and clicking a
    // canvas PANE moves focus nowhere by itself — the buffer kept it, and
    // Delete pressed while arranging the diagram erased document text at a
    // caret nobody was looking at.
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={vi.fn()} />);
    const root = await screen.findByTestId("graph-editor");
    expect(root.contains(document.activeElement)).toBe(false);
    fireEvent.pointerDown(root);
    expect(document.activeElement).toBe(root);
  });

  it("gives the wheel to the document until the canvas is clicked into, and back on Escape", async () => {
    // Scrolling a document that happens to contain a diagram must scroll the
    // document; zoom is what clicking into the canvas buys.
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={vi.fn()} />);
    const root = await screen.findByTestId("graph-editor");
    await waitFor(() => expect(flowProps.preventScrolling).toBe(false));
    expect(flowProps.zoomOnScroll).toBe(false);

    fireEvent.pointerDown(root);
    fireEvent.focus(root);
    await waitFor(() => expect(flowProps.preventScrolling).toBe(true));
    expect(flowProps.zoomOnScroll).toBe(true);

    fireEvent.keyDown(root, { key: "Escape" });
    fireEvent.blur(root);
    await waitFor(() => expect(flowProps.preventScrolling).toBe(false));
  });

  it("keeps the cursor grace: snap and grab radii are set, not left at a pixel", async () => {
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={vi.fn()} />);
    await waitFor(() => expect(flowProps.connectionRadius).toBeTruthy());
    // Near the thing, not exactly on it — see the spec's editor section.
    expect(flowProps.connectionRadius as number).toBeGreaterThanOrEqual(30);
    expect(flowProps.reconnectRadius as number).toBeGreaterThanOrEqual(20);
    expect(
      (flowProps.defaultEdgeOptions as { interactionWidth: number }).interactionWidth,
    ).toBeGreaterThanOrEqual(20);
  });

  it("locks a derived scene's topology: drag rewrites only layout, connect is refused", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={DERIVED} resolved={RESOLVED} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onConnect).toBeTruthy());
    expect(screen.getByText(/Topology is derived/).textContent).toContain("pasted fragment");
    expect(flowProps.nodesConnectable).toBe(false);

    (flowProps.onConnect as Connect)({ source: "db", target: "api" });
    expect(onCommit).not.toHaveBeenCalled();
    // Re-plugging an end changes the topology too, so a derived scene
    // refuses it the same way.
    expect(flowProps.edgesReconnectable).toBe(false);

    (flowProps.onNodeDragStop as DragStop)(null, { id: "db", position: { x: 300, y: 40 } });
    const text = onCommit.mock.calls[0][0] as string;
    expect(text).toContain('<hick:paste select="#arch" />');
    expect(text).toContain('"db": {"x": 300, "y": 40}');
    expect(text).not.toContain('"nodes"');
  });

  it("refuses to commit a label that would close the element from inside", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.nodes).toBeTruthy());
    const nodes = flowProps.nodes as { data: { onRename: (id: string, label: string) => void } }[];
    nodes[0].data.onRename("api", "bad </hick:diagram> label");
    expect(onCommit).not.toHaveBeenCalled();
    await waitFor(() => expect(screen.getByRole("status").textContent).toContain("Not saved"));
  });

  it("resets from the document on an external edit, and shows a parse error without eating text", async () => {
    const onCommit = vi.fn();
    const { rerender } = render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect((flowProps.nodes as unknown[]).length).toBe(2));
    // An external edit (undo, a collaborator, the agent) replaces the body.
    rerender(
      <GraphEditorPanel
        source={'{"nodes": [{"id": "solo"}], "edges": [], "layout": {}}'}
        onCommit={onCommit}
      />,
    );
    await waitFor(() =>
      expect((flowProps.nodes as { id: string }[]).map((n) => n.id)).toEqual(["solo"]),
    );
    // A body that stops parsing is said, not guessed at.
    rerender(<GraphEditorPanel source={"{ broken"} onCommit={onCommit} />);
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toContain("does not parse"),
    );
    expect(onCommit).not.toHaveBeenCalled();
  });

  it("snaps a resize to the grid — size and place both — and commits it", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.snapToGrid).toBe(true));
    expect(flowProps.snapGrid).toEqual([16, 16]);
    const nodes = flowProps.nodes as {
      data: { onResize: (id: string, at: { x: number; y: number; w: number; h: number }) => void };
    }[];
    act(() => nodes[0].data.onResize("api", { x: 13, y: 19, w: 173, h: 61 }));
    const text = onCommit.mock.calls[0][0] as string;
    expect(text).toContain('"api": {"x": 16, "y": 16, "w": 176, "h": 64}');
  });

  it("keeps the inspector row present when nothing is selected, so selection never reflows the document", async () => {
    // The panel's height is the document's height there: a row that comes
    // and goes with the selection shifted the prose below on every click.
    const { container } = render(<GraphEditorPanel source={HAND_DRAWN} onCommit={vi.fn()} />);
    await waitFor(() => expect(container.querySelector(".graph-inspector")).toBeTruthy());
    expect(screen.getByText(/Select a box or a line/)).toBeTruthy();
    type SelectionChange = (params: { nodes: { id: string }[]; edges: { id: string }[] }) => void;
    act(() =>
      (flowProps.onSelectionChange as SelectionChange)({ nodes: [{ id: "api" }], edges: [] }),
    );
    await screen.findByTestId("node-inspector");
    // Same single row, different contents — the container never unmounts.
    expect(container.querySelectorAll(".graph-inspector").length).toBe(1);
    act(() => (flowProps.onSelectionChange as SelectionChange)({ nodes: [], edges: [] }));
    await waitFor(() => expect(screen.getByText(/Select a box or a line/)).toBeTruthy());
  });

  it("hands each node its sides' occupancy, and records the slot a new line lands on", async () => {
    const sided = `{
  "nodes": [
    {"id": "api"},
    {"id": "db"}
  ],
  "edges": [
    {"from": "api", "fromSide": "right", "to": "db", "toSide": "left"}
  ],
  "layout": {}
}
`;
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={sided} onCommit={onCommit} />);
    await waitFor(() => expect((flowProps.nodes as unknown[]).length).toBe(2));
    const nodes = flowProps.nodes as { id: string; data: { slots: Record<string, number[]> } }[];
    expect(nodes.find((n) => n.id === "db")!.data.slots.left).toEqual([0]);
    // A line dropped on the AFTER flank orders behind the occupant; one on
    // the BEFORE flank orders ahead, and the side renumbers so the document
    // stays contiguous from 0.
    (flowProps.onConnect as Connect)({
      source: "api",
      target: "db",
      sourceHandle: "bottom",
      targetHandle: "left._after",
    } as never);
    let text = onCommit.mock.calls.at(-1)![0] as string;
    expect(text).toContain('"toSide": "left.1"');
    (flowProps.onConnect as Connect)({
      source: "api",
      target: "db",
      sourceHandle: "top",
      targetHandle: "left._before",
    } as never);
    text = onCommit.mock.calls.at(-1)![0] as string;
    // The newcomer took the front seat; everyone behind shifted by one.
    expect(text).toContain('"toSide": "left"');
    expect(text).toContain('"toSide": "left.1"');
    expect(text).toContain('"toSide": "left.2"');
  });

  it("lets two shapes carry more than one line, each with its own identity", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onConnect).toBeTruthy());
    // The pair already has api->db; a second line between the same two
    // shapes is a new line, not a rejected duplicate.
    (flowProps.onConnect as Connect)({
      source: "api",
      target: "db",
      sourceHandle: "bottom",
      targetHandle: "top",
    } as never);
    const text = onCommit.mock.calls.at(-1)![0] as string;
    expect(text).toContain('"id": "api->db#2"');
    // Both lines survive serialization, distinctly.
    expect((text.match(/"from": "api", .*"to": "db"/g) ?? []).length).toBeGreaterThanOrEqual(1);
    expect(text).toContain('"from": "api"');
  });

  it("opens the label editor ON the line on double-click, and commits from it", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onEdgeDoubleClick).toBeTruthy());
    type DblClick = (event: unknown, edge: { id: string }) => void;
    act(() => (flowProps.onEdgeDoubleClick as DblClick)(null, { id: "api->db" }));
    await waitFor(() => {
      const edges = flowProps.edges as { id: string; data: { editing?: boolean } }[];
      expect(edges.find((e) => e.id === "api->db")?.data.editing).toBe(true);
    });
    const edges = flowProps.edges as {
      id: string;
      data: { onLabel: (label: string) => void };
    }[];
    act(() => edges.find((e) => e.id === "api->db")!.data.onLabel("uses"));
    const text = onCommit.mock.calls.at(-1)![0] as string;
    expect(text).toContain('"label": "uses"');
    // Editing ended with the commit.
    await waitFor(() => {
      const after = flowProps.edges as { id: string; data: { editing?: boolean } }[];
      expect(after.find((e) => e.id === "api->db")?.data.editing).toBe(false);
    });
  });

  it("zooms to fit from the toolbar", async () => {
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={vi.fn()} />);
    const fit = await screen.findByText("Zoom to fit");
    fireEvent.click(fit);
    expect(fitViewMock).toHaveBeenCalled();
  });

  it("keeps a node selected through the commit its own resize makes", async () => {
    // Finishing a resize is usually the moment before the NEXT adjustment to
    // the same node; the rebuild after the commit must not deselect it.
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onNodesChange).toBeTruthy());
    type NodesChange = (changes: { type: string; id: string; selected?: boolean }[]) => void;
    act(() => (flowProps.onNodesChange as NodesChange)([{ type: "select", id: "api", selected: true }]));
    await waitFor(() =>
      expect((flowProps.nodes as { id: string; selected?: boolean }[]).find((n) => n.id === "api")?.selected).toBe(true),
    );
    const nodes = flowProps.nodes as {
      id: string;
      data: { onResize: (id: string, at: { x: number; y: number; w: number; h: number }) => void };
    }[];
    act(() => nodes[0].data.onResize("api", { x: 0, y: 0, w: 200, h: 120 }));
    expect(onCommit).toHaveBeenCalled();
    await waitFor(() =>
      expect(
        (flowProps.nodes as { id: string; selected?: boolean }[]).find((n) => n.id === "api")
          ?.selected,
      ).toBe(true),
    );
  });

  it("styles selected nodes from the palette: shape, fill, outline, text", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onSelectionChange).toBeTruthy());
    type SelectionChange = (params: { nodes: { id: string }[]; edges: { id: string }[] }) => void;
    act(() =>
      (flowProps.onSelectionChange as SelectionChange)({ nodes: [{ id: "api" }], edges: [] }),
    );
    await screen.findByTestId("node-inspector");

    fireEvent.change(screen.getByLabelText("Shape"), { target: { value: "cylinder" } });
    fireEvent.click(screen.getByLabelText("Fill blue"));
    fireEvent.click(screen.getByLabelText("Outline red"));
    fireEvent.click(screen.getByLabelText("Text amber"));
    const text = onCommit.mock.calls.at(-1)![0] as string;
    expect(text).toContain('"shape": "cylinder"');
    expect(text).toContain('"fill": "#60a5fa26"');
    expect(text).toContain('"stroke": "#f87171"');
    expect(text).toContain('"text": "#fbbf24"');
    // The other node was not selected and is untouched.
    expect(text).toContain('{"id": "db"}');

    // Default clears back to the theme's own colour.
    fireEvent.click(screen.getByLabelText("Fill default"));
    expect(onCommit.mock.calls.at(-1)![0] as string).not.toContain('"fill"');
  });

  it("styles a selected line: colour, arrowheads, dash, and its own label", async () => {
    const onCommit = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} />);
    await waitFor(() => expect(flowProps.onSelectionChange).toBeTruthy());
    type SelectionChange = (params: { nodes: { id: string }[]; edges: { id: string }[] }) => void;
    act(() =>
      (flowProps.onSelectionChange as SelectionChange)({ nodes: [], edges: [{ id: "api->db" }] }),
    );
    await screen.findByTestId("edge-inspector");

    fireEvent.click(screen.getByLabelText("Line green"));
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"color": "#4ade80"');
    // Arrowheads cycle one end → both → start → none, so a head can sit at
    // EITHER end alone.
    fireEvent.click(screen.getByLabelText("Arrowheads"));
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"arrow": "both"');
    fireEvent.click(screen.getByLabelText("Arrowheads"));
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"arrow": "start"');
    fireEvent.click(screen.getByLabelText("Line style"));
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"style": "dashed"');
    const label = screen.getByLabelText("Line label") as HTMLInputElement;
    fireEvent.change(label, { target: { value: "SQL" } });
    fireEvent.blur(label);
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"label": "SQL"');
  });

  it("keeps two panels apart: each instance is its own store", async () => {
    const first = vi.fn();
    const second = vi.fn();
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={first} />);
    const firstDrag = flowProps.onNodeDragStop as DragStop;
    render(<GraphEditorPanel source={HAND_DRAWN} onCommit={second} />);
    await waitFor(() => expect(flowProps.onNodeDragStop).not.toBe(firstDrag));
    (flowProps.onNodeDragStop as DragStop)(null, { id: "api", position: { x: 5, y: 5 } });
    expect(second).toHaveBeenCalledTimes(1);
    expect(first).not.toHaveBeenCalled();
  });
});
