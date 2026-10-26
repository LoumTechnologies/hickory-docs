<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/graph/GraphEditorPanel.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b7404652ca7f"></a>
## `apps/web/src/components/graph/GraphEditorPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [A Derived Diagram Keeps Your Layout](../../../../../../guarantees/authoring/a-derived-diagram-keeps-your-layout/contract.md), [A Drawn Diagram Is Document Text](../../../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md).

<a id="f-b7404652ca7f-flowProps"></a>
### `flowProps`

@vitest-environment jsdom
The canvas library is mocked the way mermaid is in DiagramPanel.test.tsx:
it wants a live browser to measure anything, and what these tests are about
is the panel's contract with the DOCUMENT — what a gesture commits, what an
external edit resets, and what a derived scene refuses.

<storycode:copy id="f-b7404652ca7f-flowProps">
// @vitest-environment jsdom
// The canvas library is mocked the way mermaid is in DiagramPanel.test.tsx:
// it wants a live browser to measure anything, and what these tests are about
// is the panel's contract with the DOCUMENT — what a gesture commits, what an
// external edit resets, and what a derived scene refuses.
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";

let flowProps: Record<string, unknown> = {};
</storycode:copy>
<a id="f-b7404652ca7f-fitViewMock"></a>
### `fitViewMock`

<storycode:copy id="f-b7404652ca7f-fitViewMock">
const fitViewMock = vi.fn();
</storycode:copy>
<a id="f-b7404652ca7f-setViewportMock"></a>
### `setViewportMock`

<storycode:copy id="f-b7404652ca7f-setViewportMock">
const setViewportMock = vi.fn().mockResolvedValue(true);
</storycode:copy>
<a id="f-b7404652ca7f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement">
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
  useReactFlow: () => ({ fitView: fitViewMock, setViewport: setViewportMock }),
  useViewport: () => ({ x: 0, y: 0, zoom: 1 }),
  useEdgesState: (init: unknown) => useState(init),
}));

</storycode:copy>
<a id="f-b7404652ca7f-import_statement"></a>
### `import_statement`

<storycode:copy id="f-b7404652ca7f-import_statement">
import { GraphEditorPanel, moveEdgeEnd } from "./GraphEditorPanel";
</storycode:copy>
<a id="f-b7404652ca7f-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-b7404652ca7f-import_statement-2">
import { parseSceneSource } from "./scene";

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-2">
afterEach(cleanup);
</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-3">
beforeEach(() => {
  flowProps = {};
  fitViewMock.mockReset();
  setViewportMock.mockClear();
});

</storycode:copy>
<a id="f-b7404652ca7f-HAND_DRAWN"></a>
### `HAND_DRAWN`

<storycode:copy id="f-b7404652ca7f-HAND_DRAWN">
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

</storycode:copy>
<a id="f-b7404652ca7f-DERIVED"></a>
### `DERIVED`

<storycode:copy id="f-b7404652ca7f-DERIVED">
const DERIVED =
  '{\n  "topology": <hick:paste select="#arch" />,\n  "layout": {\n    "api": {"x": 0, "y": 0}\n  }\n}\n';
</storycode:copy>
<a id="f-b7404652ca7f-RESOLVED"></a>
### `RESOLVED`

<storycode:copy id="f-b7404652ca7f-RESOLVED">
const RESOLVED = '{"nodes": [{"id": "api"}, {"id": "db"}], "edges": [{"from": "api", "to": "db"}]}';

</storycode:copy>
<a id="f-b7404652ca7f-DragStop"></a>
### `DragStop`

<storycode:copy id="f-b7404652ca7f-DragStop">
type DragStop = (event: unknown, node: { id: string; position: { x: number; y: number } }) => void;
</storycode:copy>
<a id="f-b7404652ca7f-Connect"></a>
### `Connect`

<storycode:copy id="f-b7404652ca7f-Connect">
type Connect = (connection: { source: string; target: string }) => void;

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-4">
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
    fireEvent.click(screen.getByLabelText("Text black"));
    const text = onCommit.mock.calls.at(-1)![0] as string;
    expect(text).toContain('"shape": "cylinder"');
    expect(text).toContain('"fill": "#00A9FF26"');
    expect(text).toContain('"stroke": "#F8766D"');
    expect(text).toContain('"text": "#000000"');
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
    expect(onCommit.mock.calls.at(-1)![0] as string).toContain('"color": "#7CAE00"');
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

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-5"></a>
### `expression_statement`

Dragging from a connector that already has a line MOVES that line.
A slot holds one line, so a drag from a taken slot has only one sensible
meaning; a second line from the same spot is what the flank handles are
for. What the gesture must never do is swap the arrow around.

<storycode:copy id="f-b7404652ca7f-expression_statement-5">
// Dragging from a connector that already has a line MOVES that line.
//
// Guarantee: docs/guarantees/authoring/a-drawn-diagram-is-document-text.md
//
// A slot holds one line, so a drag from a taken slot has only one sensible
// meaning; a second line from the same spot is what the flank handles are
// for. What the gesture must never do is swap the arrow around.
describe("moving a line by its end", () => {
  const scene = {
    nodes: [{ id: "api" }, { id: "db" }, { id: "cache" }],
    edges: [{ from: "api", to: "db", fromSide: "right.0", toSide: "left.0" }],
    layout: {},
  } as never as Parameters<typeof moveEdgeEnd>[0];

  it("moves the arrowhead and leaves the tail where it was", () => {
    const commit = vi.fn();
    // Grabbed the `to` end (on db) and dropped it on cache.
    moveEdgeEnd(
      scene,
      { key: "api->db", grabbed: "to" },
      {
        source: "api",
        sourceHandle: "right.0",
        target: "cache",
        targetHandle: "left.0",
      },
      commit,
    );
    const next = commit.mock.calls[0][0];
    expect(next.edges).toHaveLength(1);
    expect(next.edges[0]).toMatchObject({ from: "api", to: "cache", fromSide: "right.0" });
  });

  it("moves the tail and leaves the arrowhead where it was", () => {
    const commit = vi.fn();
    // Grabbed the `from` end (on api). The drag's own direction makes api the
    // connection's SOURCE, which is the opposite of the line's direction —
    // the arrow must still end at db.
    moveEdgeEnd(
      scene,
      { key: "api->db", grabbed: "from" },
      {
        source: "cache",
        sourceHandle: "right.0",
        target: "db",
        targetHandle: "left.0",
      },
      commit,
    );
    const next = commit.mock.calls[0][0];
    expect(next.edges[0]).toMatchObject({ from: "cache", to: "db", toSide: "left.0" });
  });

  it("moves a line to another side of the SAME shape", () => {
    const commit = vi.fn();
    moveEdgeEnd(
      scene,
      { key: "api->db", grabbed: "to" },
      {
        source: "api",
        sourceHandle: "right.0",
        target: "db",
        targetHandle: "top.0",
      },
      commit,
    );
    expect(commit.mock.calls[0][0].edges[0]).toMatchObject({
      from: "api",
      to: "db",
      toSide: "top.0",
    });
  });

  it("leaves every other line alone", () => {
    const commit = vi.fn();
    const two = {
      ...scene,
      edges: [...scene.edges, { from: "db", to: "cache" }],
    } as never as Parameters<typeof moveEdgeEnd>[0];
    moveEdgeEnd(
      two,
      { key: "api->db", grabbed: "to" },
      { source: "api", sourceHandle: "right.0", target: "cache", targetHandle: "left.1" },
      commit,
    );
    const next = commit.mock.calls[0][0];
    expect(next.edges).toHaveLength(2);
    expect(next.edges[1]).toMatchObject({ from: "db", to: "cache" });
  });

  it("commits nothing when the line it was told to move is gone", () => {
    const commit = vi.fn();
    moveEdgeEnd(
      scene,
      { key: "nope->gone", grabbed: "to" },
      { source: "api", sourceHandle: "right.0", target: "cache", targetHandle: "left.0" },
      commit,
    );
    expect(commit).not.toHaveBeenCalled();
  });
});

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-6">
it("shares graph node mutations and refuses derived topology while retaining layout controls", async () => {
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onCommit = vi.fn();
  const view = render(<UiActionContext.Provider value={registry}><GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
      handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id),
    }); });
    return result;
  };
  await waitFor(() => expect(actions().some(a => a.label === "Rename graph node api")).toBe(true));
  expect(await run("Rename graph node api", { label: "Service" })).toMatchObject({ status: "result", outcome: { value: { commit_callback_requested: true, saved: false } } });
  expect(onCommit.mock.lastCall![0]).toContain('"label": "Service"');
  expect(onCommit.mock.lastCall![1]).toEqual({ id: "python:turn", kind: "agent" });
  await run("Resize graph node api", { x: 19, y: 35, w: 100, h: 50 });
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { layout: { api: { x: 16, y: 32, w: 96, h: 48 } } } });
  await run("Move graph node api", { x: 40.2, y: 22.8 });
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { layout: { api: { x: 40, y: 23, w: 96, h: 48 } } } });
  const stale = actions().find(a => a.label === "Rename graph node api")!.handle;
  view.rerender(<UiActionContext.Provider value={registry}><GraphEditorPanel source={DERIVED} resolved={RESOLVED} onCommit={onCommit} /></UiActionContext.Provider>);
  await waitFor(() => expect(actions().find(a => a.label === "Add graph node")?.enabled).toBe(false));
  expect(await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, { handle: stale, arguments: { label: "stale" }, request_id: "stale" })).not.toMatchObject({ status: "result" });
  expect(await run("Rename graph node api", { label: "forbidden" })).toMatchObject({ status: "disabled" });
  await run("Move graph node api", { x: 80, y: 96 });
  expect(onCommit.mock.lastCall![0]).toContain('<hick:paste select="#arch" />');
  expect(onCommit.mock.lastCall![0]).not.toContain('"nodes"');
  view.unmount();
  expect(actions()).toHaveLength(0);
});

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-7">
it("reloads changed derived topology when authored source is unchanged", async () => {
  const onCommit = vi.fn();
  const view = render(<GraphEditorPanel source={DERIVED} resolved={RESOLVED} onCommit={onCommit} />);
  await waitFor(() => expect((flowProps.nodes as { id: string }[]).map(n => n.id)).toEqual(["api", "db"]));
  view.rerender(<GraphEditorPanel source={DERIVED} resolved='{"nodes":[{"id":"new"}],"edges":[]}' onCommit={onCommit} />);
  await waitFor(() => expect((flowProps.nodes as { id: string }[]).map(n => n.id)).toEqual(["new"]));
  expect(onCommit).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-8">
it("shares graph selection, finite inspector palettes and validated connection lifecycle", async () => {
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onCommit = vi.fn();
  render(<UiActionContext.Provider value={registry}><GraphEditorPanel source={HAND_DRAWN} onCommit={onCommit} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
      handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id),
    }); }); return result;
  };
  await waitFor(() => expect(actions().some(a => a.label === "Select graph objects")).toBe(true));
  expect(await run("Set graph viewport", { x: 120, y: -20, zoom: 1.5 })).toMatchObject({ status: "result", outcome: { value: { viewport_changed: true } } });
  expect(setViewportMock).toHaveBeenCalledWith({ x: 120, y: -20, zoom: 1.5 }, { duration: 0 });
  await run("Set graph canvas engagement", { engaged: true });
  expect(document.activeElement).toBe(screen.getByTestId("graph-editor"));
  expect(flowProps.zoomOnScroll).toBe(true);
  await run("Set graph canvas engagement", { engaged: false });
  expect(flowProps.zoomOnScroll).toBe(false);
  expect(onCommit).not.toHaveBeenCalled();
  await run("Select graph objects", { nodes: ["api"], edges: [] });
  expect(screen.getByTestId("node-inspector")).toBeTruthy();
  expect((flowProps.nodes as { id: string; selected: boolean }[]).find(n => n.id === "api")?.selected).toBe(true);
  await run("Set graph node api shape", { value: "diamond" });
  expect(onCommit.mock.lastCall![1]).toEqual({ id: "acp:turn", kind: "agent" });
  expect((screen.getByRole("combobox", { name: "Shape" }) as HTMLSelectElement).value).toBe("diamond");
  fireEvent.change(screen.getByRole("combobox", { name: "Shape" }), { target: { value: "circle" } });
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { nodes: [{ id: "api", shape: "circle" }, { id: "db" }] } });
  const count = onCommit.mock.calls.length;
  expect(await run("Set graph node api fill", { value: "url(unsafe)" })).toMatchObject({ status: "invalid" });
  expect(await run("Connect graph nodes", { from: "api", to: "db", from_handle: "not-a-side", to_handle: "" })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(onCommit).toHaveBeenCalledTimes(count);
  await run("Connect graph nodes", { from: "api", to: "db", from_handle: "", to_handle: "" });
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { edges: [{ from: "api", to: "db" }, { id: "api->db#2" }] } });
  await run("Reconnect graph edge api->db#2", { from: "db", to: "api", from_handle: "", to_handle: "" });
  await run("Set graph edge api->db#2 label", { label: "response" });
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { edges: [{ from: "api", to: "db" }, { id: "api->db#2", from: "db", to: "api", label: "response" }] } });
  expect(onCommit.mock.lastCall![1]).toEqual({ id: "acp:turn", kind: "agent" });
  await run("Delete graph node api");
  expect(parseSceneSource(onCommit.mock.lastCall![0])).toMatchObject({ ok: true, scene: { nodes: [{ id: "db" }], edges: [] } });
});


</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-9">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("refuses parent retirement before React updates and does not accept a refused commit locally", async () => {
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "guard" }, () => true, async () => {});
  let editable = true, writable = true;
  const onCommit = vi.fn(() => false);
  render(<UiActionContext.Provider value={registry}><GraphEditorPanel source={HAND_DRAWN} sourceCurrent={() => editable} sourceWritable={() => writable} onCommit={onCommit} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await waitFor(() => expect(actions().some(action => action.label === "Rename graph node api")).toBe(true));
  const command = actions().find(action => action.label === "Rename graph node api")!;
  let result;
  await act(async () => { result = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: command.handle, arguments: { label: "Refused" }, request_id: "refused" }); });
  expect(result).toMatchObject({ outcome: { status: "unavailable" } });
  expect((flowProps.nodes as { data: { node: { label: string } } }[])[0].data.node.label).not.toBe("Refused");
  onCommit.mockClear(); writable = false;
  expect(actions().find(action => action.label === "Rename graph node api")!.enabled).toBe(false);
  const viewport = actions().find(action => action.label === "Set graph viewport")!;
  expect(viewport.enabled).toBe(true);
  await act(async () => { result = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: viewport.handle, arguments: { x: 0, y: 0, zoom: 1 }, request_id: "readonly-view" }); });
  expect(result).toMatchObject({ outcome: { status: "completed" } });
  expect(onCommit).not.toHaveBeenCalled();
  editable = false;
  const latest = actions().find(action => action.label === "Rename graph node api")!;
  await act(async () => { result = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: latest.handle, arguments: { label: "Stale" }, request_id: "stale-owner" }); });
  expect(result).not.toMatchObject({ outcome: { status: "completed" } });
  expect(onCommit).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-b7404652ca7f-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-b7404652ca7f-expression_statement-10">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("writes graph actions through the actual document block with agent attribution and live revision guards", async () => {
  const { UiActions } = await import("../../lib/uiActions");
  const { UiActionContext } = await import("../../lib/uiActionBridge");
  const { EditorState, Transaction } = await import("@codemirror/state");
  const { EditorView } = await import("@codemirror/view");
  const { diagramView } = await import("../../elements/diagram/view");
  const { structureOf } = await import("../../editor/wysiwyg");
  const { replaceCapturedBlock } = await import("../../editor/replaceCapturedBlock");
  const registry = new UiActions({ workspace: "doc", window: "graph" }, () => true, async () => {});
  const parent = document.body.appendChild(document.createElement("div")), events: string[] = [];
  const editor = new EditorView({ parent, state: EditorState.create({ doc: `# Intro\n\n<hick:diagram renderer="graph">\n${HAND_DRAWN}</hick:diagram>\n`, extensions: [EditorView.updateListener.of(update => { for (const transaction of update.transactions) { const event = transaction.annotation(Transaction.userEvent); if (event) events.push(event); } })] }) });
  try {
    const block = structureOf(editor.state).blocks.find(block => block.name === "diagram")!;
    const slot = { at: block.from, text: editor.state.doc.sliceString(block.contentFrom, block.contentTo), renderer: "graph", asserts: [], index: 0 } as unknown as import("../../editor/rendered").RenderedSlot;
    render(<UiActionContext.Provider value={registry}>{diagramView.render(slot, { view: editor, path: "doc.md", execBlocks: [], diagramBlocks: [], runningCells: new Set(), replaying: [], replaceBlockContent: (slot, text, event) => replaceCapturedBlock(editor, slot, text, event ?? "input") })}</UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
    await waitFor(() => expect(actions().some(action => action.label === "Rename graph node api")).toBe(true));
    const old = actions().find(action => action.label === "Rename graph node api")!;
    // An unrelated prose edit expires old handles but needs no widget rerender.
    act(() => editor.dispatch({ changes: { from: 2, to: 3, insert: "Z" } }));
    let result;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:doc", kind: "agent" }, { handle: old.handle, arguments: { label: "Stale" }, request_id: "old" }); });
    expect(result).not.toMatchObject({ outcome: { status: "completed" } });
    const current = actions().find(action => action.label === "Rename graph node api")!;
    expect(current.enabled).toBe(true);
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:doc", kind: "agent" }, { handle: current.handle, arguments: { label: "Agent approved source" }, request_id: "current" }); });
    expect(result).toMatchObject({ outcome: { status: "completed" } });
    expect(editor.state.doc.toString()).toContain('"label": "Agent approved source"');
    expect(editor.state.doc.toString()).not.toContain('"label": "Stale"');
    expect(events).toEqual(["input.uiAgentDiagram"]);
  } finally { editor.destroy(); parent.remove(); }
});
</storycode:copy>

### Assemble `apps/web/src/components/graph/GraphEditorPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/graph/GraphEditorPanel.test.tsx"><storycode:paste select="#f-b7404652ca7f-flowProps"/><storycode:paste select="#f-b7404652ca7f-fitViewMock"/><storycode:paste select="#f-b7404652ca7f-setViewportMock"/><storycode:paste select="#f-b7404652ca7f-expression_statement"/><storycode:paste select="#f-b7404652ca7f-import_statement"/><storycode:paste select="#f-b7404652ca7f-import_statement-2"/><storycode:paste select="#f-b7404652ca7f-expression_statement-2"/><storycode:paste select="#f-b7404652ca7f-expression_statement-3"/><storycode:paste select="#f-b7404652ca7f-HAND_DRAWN"/><storycode:paste select="#f-b7404652ca7f-DERIVED"/><storycode:paste select="#f-b7404652ca7f-RESOLVED"/><storycode:paste select="#f-b7404652ca7f-DragStop"/><storycode:paste select="#f-b7404652ca7f-Connect"/><storycode:paste select="#f-b7404652ca7f-expression_statement-4"/><storycode:paste select="#f-b7404652ca7f-expression_statement-5"/><storycode:paste select="#f-b7404652ca7f-expression_statement-6"/><storycode:paste select="#f-b7404652ca7f-expression_statement-7"/><storycode:paste select="#f-b7404652ca7f-expression_statement-8"/><storycode:paste select="#f-b7404652ca7f-expression_statement-9"/><storycode:paste select="#f-b7404652ca7f-expression_statement-10"/></storycode:file>

</storycode:doc>
