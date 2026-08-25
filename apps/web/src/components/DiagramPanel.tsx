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

import { useEffect, useRef, useState } from "react";

export interface DiagramPanelProps {
  /** Renderer named by the tag; anything unknown stays source-only. */
  renderer: string;
  /** The diagram source, exactly as written between the tags. */
  source: string;
  /** Stable id for the render target — mermaid needs a unique DOM id. */
  domId: string;
  /** Ids named by `asserts`, with whether each one is currently passing. */
  assertions?: { id: string; state: "passing" | "failing" | "unknown" }[];
}

type Render =
  | { status: "idle" }
  | { status: "drawn"; svg: string }
  | { status: "failed"; message: string };

/** Only what this build can draw; everything else falls back to the source. */
const RENDERERS = ["mermaid"];

export function DiagramPanel({
  renderer,
  source,
  domId,
  assertions = [],
}: DiagramPanelProps) {
  const [render, setRender] = useState<Render>({ status: "idle" });
  // Guards against an out-of-order render: mermaid is async and someone
  // typing produces overlapping calls, of which only the last is current.
  const latest = useRef(0);

  useEffect(() => {
    if (!RENDERERS.includes(renderer)) return;
    const trimmed = source.trim();
    if (!trimmed) {
      setRender({ status: "idle" });
      return;
    }
    const ticket = ++latest.current;
    let cancelled = false;

    (async () => {
      try {
        const mermaid = (await import("mermaid")).default;
        mermaid.initialize({
          startOnLoad: false,
          // The editor is themed by the app; a diagram that ignored that
          // would be a bright rectangle in a dark document.
          theme: "base",
          securityLevel: "strict",
        });
        const { svg } = await mermaid.render(domId, trimmed);
        if (!cancelled && ticket === latest.current)
          setRender({ status: "drawn", svg });
      } catch (error) {
        // A half-typed diagram is the common case, not an exception. The
        // source stays visible below either way, so this never costs the
        // author their text — it only says why there is no picture yet.
        if (!cancelled && ticket === latest.current) {
          setRender({
            status: "failed",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [renderer, source, domId]);

  if (!RENDERERS.includes(renderer)) {
    return (
      <div className="diagram-panel diagram-panel-unknown">
        <p className="muted">
          No renderer for <span className="mono">{renderer}</span> in this build
          — the source below is unchanged and still weaves.
        </p>
      </div>
    );
  }

  return (
    <div className="diagram-panel" data-testid="diagram-panel">
      {render.status === "drawn" && (
        // eslint-disable-next-line react/no-danger -- mermaid's own output,
        // rendered with securityLevel "strict" (no scripts, no click handlers).
        <div
          className="diagram-figure"
          dangerouslySetInnerHTML={{ __html: render.svg }}
        />
      )}
      {render.status === "failed" && (
        <p className="diagram-error" role="status">
          This diagram does not parse yet: {render.message}
        </p>
      )}
      <DiagramAssertions assertions={assertions} />
    </div>
  );
}

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
        : `Checked by ${assertions.map((a) => `#${a.id}`).join(", ")}.`}
    </p>
  );
}
