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

import { SceneEdgeView } from "./SceneEdgeView";

afterEach(cleanup);

type EdgeViewProps = React.ComponentProps<typeof SceneEdgeView>;

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
