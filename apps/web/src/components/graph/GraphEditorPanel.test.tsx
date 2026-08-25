// @vitest-environment jsdom
// The canvas library is mocked the way mermaid is in DiagramPanel.test.tsx:
// it wants a live browser to measure anything, and what these tests are about
// is the panel's contract with the DOCUMENT — what a gesture commits, what an
// external edit resets, and what a derived scene refuses.
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useState } from "react";

let flowProps: Record<string, unknown> = {};
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
  applyNodeChanges: (_c: unknown, ns: unknown) => ns,
  applyEdgeChanges: (_c: unknown, es: unknown) => es,
  useNodesState: (init: unknown) => useState(init),
  useEdgesState: (init: unknown) => useState(init),
}));

import { GraphEditorPanel } from "./GraphEditorPanel";

afterEach(cleanup);
beforeEach(() => {
  flowProps = {};
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
