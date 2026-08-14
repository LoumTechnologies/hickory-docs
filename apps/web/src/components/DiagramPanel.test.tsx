// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { DiagramPanel } from "./DiagramPanel";

// The real engine is a megabyte of layout code that wants a live browser.
// What this file is about is the panel's behaviour around it: draw when it
// can, say why when it cannot, and never lose the author's source.
const renderMock = vi.fn();
vi.mock("mermaid", () => ({
  default: {
    initialize: vi.fn(),
    render: (...args: unknown[]) => renderMock(...args),
  },
}));

afterEach(cleanup);

describe("the diagram panel", () => {
  beforeEach(() => {
    renderMock.mockReset();
  });

  it("draws what the renderer returns", async () => {
    renderMock.mockResolvedValue({ svg: "<svg data-testid='drawn'></svg>" });
    render(
      <DiagramPanel
        renderer="mermaid"
        source="flowchart TD\n  a --> b"
        domId="d1"
        assertions={[{ id: "no-back-edges", state: "passing" }]}
      />,
    );
    await waitFor(() => expect(screen.getByTestId("drawn")).toBeTruthy());
    expect(screen.getByRole("status").textContent).toContain(
      "Checked by #no-back-edges",
    );
  });

  it("says why there is no picture instead of throwing", async () => {
    // Half-typed diagrams are the common case while someone is writing one.
    renderMock.mockRejectedValue(new Error("Parse error on line 2"));
    render(
      <DiagramPanel
        renderer="mermaid"
        source="flowchart TD\n  a -->"
        domId="d2"
      />,
    );
    await waitFor(() =>
      expect(screen.getByText(/does not parse yet/).textContent).toContain(
        "Parse error on line 2",
      ),
    );
  });

  it("names an unchecked diagram as unchecked", async () => {
    renderMock.mockResolvedValue({ svg: "<svg></svg>" });
    render(
      <DiagramPanel renderer="mermaid" source="flowchart TD" domId="d3" />,
    );
    await waitFor(() =>
      expect(screen.getByText(/Nothing checks this diagram/)).toBeTruthy(),
    );
  });

  it("says the diagram is out of date when an assertion it names is failing", async () => {
    // The whole point: the drawing and the code have parted company, and the
    // reader finds out here rather than three weeks later.
    renderMock.mockResolvedValue({ svg: "<svg></svg>" });
    render(
      <DiagramPanel
        renderer="mermaid"
        source="flowchart TD"
        domId="d4"
        assertions={[
          { id: "no-back-edges", state: "failing" },
          { id: "layers", state: "passing" },
        ]}
      />,
    );
    await waitFor(() => {
      const status = screen.getByRole("status").textContent ?? "";
      expect(status).toContain("out of date");
      expect(status).toContain("#no-back-edges");
      expect(status).not.toContain("#layers no longer");
    });
  });

  it("leaves an unknown renderer alone rather than guessing", async () => {
    render(<DiagramPanel renderer="d3" source="{}" domId="d5" />);
    await waitFor(() =>
      expect(screen.getByText(/No renderer for/)).toBeTruthy(),
    );
    expect(renderMock).not.toHaveBeenCalled();
  });

  it("ignores an answer that arrived after the source moved on", async () => {
    // Typing produces overlapping async renders. The danger is the SLOW one
    // landing last and painting a picture of text that is no longer there,
    // so this resolves them deliberately out of order.
    const pending: { source: string; resolve: (v: { svg: string }) => void }[] =
      [];
    renderMock.mockImplementation(
      (_id: string, source: string) =>
        new Promise((resolve) => pending.push({ source, resolve })),
    );

    const { rerender } = render(
      <DiagramPanel renderer="mermaid" source="flowchart TD" domId="d6" />,
    );
    await waitFor(() => expect(pending).toHaveLength(1));
    rerender(
      <DiagramPanel renderer="mermaid" source="flowchart LR" domId="d6" />,
    );
    await waitFor(() => expect(pending).toHaveLength(2));

    const first = pending.find((p) => p.source === "flowchart TD")!;
    const second = pending.find((p) => p.source === "flowchart LR")!;
    second.resolve({ svg: "<svg data-testid='second'></svg>" });
    await waitFor(() => expect(screen.getByTestId("second")).toBeTruthy());

    first.resolve({ svg: "<svg data-testid='stale'></svg>" });
    await new Promise((r) => setTimeout(r, 0));
    expect(screen.queryByTestId("stale")).toBeNull();
    expect(screen.getByTestId("second")).toBeTruthy();
  });
});
