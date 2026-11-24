<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/DiagramPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.


<a id="output-80582538ea4e"></a>
## `apps/web/src/components/DiagramPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Diagram Names What Proves It](../../../../../guarantees/authoring/a-diagram-names-what-proves-it/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md).

<a id="f-80582538ea4e-DiagramPanelProps"></a>
### `DiagramPanelProps`

The rendered picture above a `<hick:diagram>` block.
Why a picture belongs in a document at all: understanding a codebase by
reading every line does not scale, so people draw. What they draw is wrong
within a quarter, because a drawing is the one artifact nothing re-checks.
Here the cells named in `asserts` re-check it, so this panel shows two
things at once — the diagram, and whether the claims behind it still hold.
Renderer staging (docs/specs/freeform/diagrams-that-fail-when-they-lie.md):
mermaid now, d3 when interaction becomes the point, first-party SVG maybe
never. `renderer` on the tag is what makes that a value rather than a
rewrite.
Mermaid is imported LAZILY and on purpose. It is the largest dependency in
this app by a wide margin, and the marketing site builds from the same
source tree — a static import would put a diagramming engine into the
bundle of a page that shows no diagrams.

<storycode:copy id="f-80582538ea4e-DiagramPanelProps">
// The rendered picture above a `<hick:diagram>` block.
//
// Why a picture belongs in a document at all: understanding a codebase by
// reading every line does not scale, so people draw. What they draw is wrong
// within a quarter, because a drawing is the one artifact nothing re-checks.
// Here the cells named in `asserts` re-check it, so this panel shows two
// things at once — the diagram, and whether the claims behind it still hold.
//
// Renderer staging (docs/specs/freeform/diagrams-that-fail-when-they-lie.md):
// mermaid now, d3 when interaction becomes the point, first-party SVG maybe
// never. `renderer` on the tag is what makes that a value rather than a
// rewrite.
//
// Mermaid is imported LAZILY and on purpose. It is the largest dependency in
// this app by a wide margin, and the marketing site builds from the same
// source tree — a static import would put a diagramming engine into the
// bundle of a page that shows no diagrams.

import { openDiagramLocation } from "../lib/revealLine";
import { useUiReading } from "../lib/uiReading";
import { useEffect, useRef, useState } from "react";

export interface DiagramPanelProps {
  /** Renderer named by the tag; anything unknown stays source-only. */
  renderer: string;
  path?: string;
  openLink?: (href: string) => void;
  /** The diagram source, exactly as written between the tags. */
  source: string;
  /** Stable id for the render target — mermaid needs a unique DOM id. */
  domId: string;
  systemAppearance?: boolean;
  /** Ids named by `asserts`, with whether each one is currently passing. */
  assertions?: { id: string; state: "passing" | "failing" | "unknown" }[];
}

</storycode:copy>
<a id="f-80582538ea4e-Render"></a>
### `Render`

<storycode:copy id="f-80582538ea4e-Render">
type Render =
  | { status: "idle" }
  | { status: "drawn"; svg: string; input: { source: string; renderer: string; domId: string } }
  | { status: "failed"; message: string; input: { source: string; renderer: string; domId: string } };

</storycode:copy>
<a id="f-80582538ea4e-RENDERERS"></a>
### `RENDERERS`

Only what this build can draw; everything else falls back to the source.

<storycode:copy id="f-80582538ea4e-RENDERERS">
/** Only what this build can draw; everything else falls back to the source. */
const RENDERERS = ["mermaid"];

</storycode:copy>
<a id="f-80582538ea4e-DiagramPanel"></a>
### `DiagramPanel`

Keep Mermaid in strict mode. On double-click, match only a rendered flowchart node to its quoted file-link declaration; resolve a local path through workspace navigation rather than enabling Mermaid callbacks or browser navigation.

Keep the last picture in place while the Simple workspace changes appearance.
Subscribe within the diagram and reject obsolete asynchronous replies with the
same render generation used for source changes. The editor owns its selection
and history throughout.

Handle empty and oversize source before loading Mermaid. Explain the failure
without leaving an earlier drawing on screen for that source.

<storycode:copy id="f-80582538ea4e-DiagramPanel">
export function DiagramPanel({
  renderer,
  source,
  domId,
  assertions = [],
  path,
  openLink,
  systemAppearance = false,
}: DiagramPanelProps) {
  const [render, setRender] = useState<Render>({ status: "idle" });
  const unavailable = source.length > 200_000 ? "Diagram is too large to preview."
    : !source.trim() ? "Diagram source is empty." : null;
  // Guards against an out-of-order render: mermaid is async and someone
  // typing produces overlapping calls, of which only the last is current.
  const latest = useRef(0);

  useEffect(() => {
    if (!RENDERERS.includes(renderer) || unavailable) return;
    const trimmed = source.trim();
    let cancelled = false;

    const appearance = systemAppearance ? window.matchMedia?.("(prefers-color-scheme: dark)") : undefined;
    const draw = async () => {
      const generation = ++latest.current;
      try {
        const mermaid = (await import("mermaid")).default;
        if (cancelled || generation !== latest.current) return;
        mermaid.initialize({
          startOnLoad: false,
          // The editor is themed by the app; a diagram that ignored that
          // would be a bright rectangle in a dark document.
          theme: appearance?.matches ? "dark" : "base",
          securityLevel: "strict",
        });
        const { svg } = await mermaid.render(domId, trimmed);
        if (!cancelled && generation === latest.current)
          setRender({ status: "drawn", svg, input: { source, renderer, domId } });
      } catch (error) {
        // A half-typed diagram is the common case, not an exception. The
        // source stays visible below either way, so this never costs the
        // author their text — it only says why there is no picture yet.
        if (!cancelled && generation === latest.current) {
          setRender({
            status: "failed",
            input: { source, renderer, domId },
            message: error instanceof Error ? error.message : String(error),
          });
        }
      }
    };
    appearance?.addEventListener("change", draw);
    void draw();
    return () => {
      cancelled = true;
      appearance?.removeEventListener("change", draw);
    };
  }, [renderer, source, domId, systemAppearance, unavailable]);

  const stale = !unavailable && render.status !== "idle" && (render.input.source !== source || render.input.renderer !== renderer || render.input.domId !== domId);
  const reportedRender = !RENDERERS.includes(renderer) ? { status: "unsupported" }
    : unavailable ? { status: "failed", message: unavailable } : render;
  useUiReading("diagram rendering", { renderer, source, assertions, render: reportedRender, stale, scope: "Actual renderer result and reported assertion states; diagram rendering does not independently establish assertions" });
  if (!RENDERERS.includes(renderer)) {
    return (
      <div className="diagram-panel diagram-panel-unknown">
        <p className="muted" role="status">
          No renderer for <span className="mono">{renderer}</span> in this build
          — the source below is unchanged and still weaves.
        </p>
      </div>
    );
  }

  return (
    <div className="diagram-panel" data-testid="diagram-panel">
      {unavailable && <p className="diagram-error" role="status">{unavailable} Use the diagram gutter to edit its source.</p>}
      {stale && <p role="status">Diagram preview is from previous source; current rendering is pending.</p>}
      {!unavailable && render.status === "drawn" && (
        // eslint-disable-next-line react/no-danger -- mermaid's own output,
        // rendered with securityLevel "strict" (no scripts, no click handlers).
        <div
          className="diagram-figure"
          onClick={event => { if ((event.target as Element).closest("a")) event.preventDefault(); }}
          onDoubleClick={event => {
            const node = (event.target as Element).closest("g.node");
            if (!node || !path || stale) return;
            for (const match of source.matchAll(/(?:^|[;\n])\s*click\s+([\w-]+)\s+(?:href\s+)?"([^"]+)"/g)) {
              if (node.id.match(/(?:^|-)flowchart-(.+)-\d+$/)?.[1] !== match[1]) continue;
              if (openLink) openLink(match[2]);
              else openDiagramLocation(match[2], path, () => true);
            }
          }}
          dangerouslySetInnerHTML={{ __html: render.svg }}
        />
      )}
      {!unavailable && !stale && render.status === "failed" && (
        <p className="diagram-error" role="status">
          This diagram does not parse yet: {render.message}
        </p>
      )}
      <DiagramAssertions assertions={assertions} />
    </div>
  );
}

</storycode:copy>
<a id="f-80582538ea4e-DiagramAssertions"></a>
### `DiagramAssertions`

The line that makes the picture worth trusting — or worth doubting.
A diagram with no assertions is not an error, but it must not look the same
as one that is proved: "nothing checks this" is the most useful thing this
panel can say about an unverified drawing.

<storycode:copy id="f-80582538ea4e-DiagramAssertions">
/**
 * The line that makes the picture worth trusting — or worth doubting.
 *
 * A diagram with no assertions is not an error, but it must not look the same
 * as one that is proved: "nothing checks this" is the most useful thing this
 * panel can say about an unverified drawing.
 */
export function DiagramAssertions({
  assertions,
}: {
  assertions: DiagramPanelProps["assertions"];
}) {
  if (!assertions || assertions.length === 0) {
    return (
      <p className="diagram-assertions diagram-assertions-none muted">
        Nothing checks this diagram — it will not fail when it stops being true.
      </p>
    );
  }
  const failing = assertions.filter((a) => a.state === "failing");
  return (
    <p
      className={`diagram-assertions${failing.length ? " diagram-assertions-failing" : ""}`}
      role="status"
    >
      {failing.length > 0
        ? `This diagram is out of date: ${failing.map((a) => `#${a.id}`).join(", ")} ${
            failing.length === 1 ? "no longer holds" : "no longer hold"
          }.`
        : assertions.every(a => a.state === "passing")
          ? `Assertions passed: ${assertions.map(a => `#${a.id}`).join(", ")}.`
          : `Assertions not yet verified: ${assertions.filter(a => a.state !== "passing").map(a => `#${a.id}`).join(", ")}.`}
    </p>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/DiagramPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DiagramPanel.tsx"><storycode:paste select="#f-80582538ea4e-DiagramPanelProps"/><storycode:paste select="#f-80582538ea4e-Render"/><storycode:paste select="#f-80582538ea4e-RENDERERS"/><storycode:paste select="#f-80582538ea4e-DiagramPanel"/><storycode:paste select="#f-80582538ea4e-DiagramAssertions"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-56af562b717e"></a>
## `apps/web/src/components/DiagramPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Diagram Names What Proves It](../../../../../guarantees/authoring/a-diagram-names-what-proves-it/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md).

<a id="f-56af562b717e-renderMock"></a>
### `renderMock`

<storycode:copy id="f-56af562b717e-renderMock">
// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import mermaid from "mermaid";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { DiagramPanel } from "./DiagramPanel";

// The real engine is a megabyte of layout code that wants a live browser.
// What this file is about is the panel's behaviour around it: draw when it
// can, say why when it cannot, and never lose the author's source.
const renderMock = vi.fn();
</storycode:copy>
<a id="f-56af562b717e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-56af562b717e-expression_statement">
vi.mock("mermaid", () => ({
  default: {
    initialize: vi.fn(),
    render: (...args: unknown[]) => renderMock(...args),
  },
}));

</storycode:copy>
<a id="f-56af562b717e-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-56af562b717e-expression_statement-2">
afterEach(cleanup);

</storycode:copy>
<a id="f-56af562b717e-expression_statement-3"></a>
### `expression_statement`

Change the media query while a diagram is mounted and require both the new
palette and a fresh picture. The deferred source test then checks that an older
asynchronous result cannot take its place.

<storycode:copy id="f-56af562b717e-expression_statement-3">
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
      "Assertions passed: #no-back-edges",
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

  it("reports empty and oversize source without retaining the picture or calling Mermaid", async () => {
    renderMock.mockResolvedValue({ svg: "<svg data-testid='drawn'></svg>" });
    const { rerender } = render(<DiagramPanel renderer="mermaid" source="flowchart TD" domId="bounded" />);
    await screen.findByTestId("drawn");
    rerender(<DiagramPanel renderer="mermaid" source="" domId="bounded" />);
    expect(screen.getByRole("status").textContent).toContain("Diagram source is empty");
    expect(screen.queryByTestId("drawn")).toBeNull();
    rerender(<DiagramPanel renderer="mermaid" source={" ".repeat(200_001)} domId="bounded" />);
    expect(screen.getByRole("status").textContent).toContain("Diagram is too large to preview");
    expect(screen.getByRole("status").textContent).toContain("Use the diagram gutter");
    expect(renderMock).toHaveBeenCalledTimes(1);
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

  it("repaints an open Simple diagram when the system appearance changes", async () => {
    const preference = new EventTarget();
    let dark = false;
    Object.defineProperty(preference, "matches", { get: () => dark });
    vi.stubGlobal("matchMedia", () => preference);
    try {
      renderMock.mockResolvedValueOnce({ svg: "<svg data-testid='light'></svg>" })
        .mockResolvedValueOnce({ svg: "<svg data-testid='dark'></svg>" });
      render(<DiagramPanel renderer="mermaid" source="flowchart LR" domId="appearance" systemAppearance />);
      await screen.findByTestId("light");
      dark = true;
      preference.dispatchEvent(new Event("change"));
      await screen.findByTestId("dark");
      expect(mermaid.initialize).toHaveBeenLastCalledWith(expect.objectContaining({ theme: "dark", securityLevel: "strict" }));
      expect(screen.queryByTestId("light")).toBeNull();
    } finally {
      vi.unstubAllGlobals();
    }
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
</storycode:copy>

### Assemble `apps/web/src/components/DiagramPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/DiagramPanel.test.tsx"><storycode:paste select="#f-56af562b717e-renderMock"/><storycode:paste select="#f-56af562b717e-expression_statement"/><storycode:paste select="#f-56af562b717e-expression_statement-2"/><storycode:paste select="#f-56af562b717e-expression_statement-3"/></storycode:file>


</storycode:doc>
