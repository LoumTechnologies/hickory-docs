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

import { diagramFilePath, openLocation } from "../lib/revealLine";
import { useUiReading } from "../lib/uiReading";
import { useEffect, useRef, useState } from "react";

export interface DiagramPanelProps {
  /** Renderer named by the tag; anything unknown stays source-only. */
  renderer: string;
  path?: string;
  /** The diagram source, exactly as written between the tags. */
  source: string;
  /** Stable id for the render target — mermaid needs a unique DOM id. */
  domId: string;
  systemAppearance?: boolean;
  /** Ids named by `asserts`, with whether each one is currently passing. */
  assertions?: { id: string; state: "passing" | "failing" | "unknown" }[];
}

type Render =
  | { status: "idle" }
  | { status: "drawn"; svg: string; input: { source: string; renderer: string; domId: string } }
  | { status: "failed"; message: string; input: { source: string; renderer: string; domId: string } };

/** Only what this build can draw; everything else falls back to the source. */
const RENDERERS = ["mermaid"];

export function DiagramPanel({
  renderer,
  source,
  domId,
  assertions = [],
  path,
  systemAppearance = false,
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
  }, [renderer, source, domId, systemAppearance]);

  const stale = render.status !== "idle" && (render.input.source !== source || render.input.renderer !== renderer || render.input.domId !== domId);
  useUiReading("diagram rendering", { renderer, source, assertions, render: RENDERERS.includes(renderer) ? render : { status: "unsupported" }, stale, scope: "Actual renderer result and reported assertion states; diagram rendering does not independently establish assertions" });
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
      {stale && <p role="status">Diagram preview is from previous source; current rendering is pending.</p>}
      {render.status === "drawn" && (
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
              const target = diagramFilePath(match[2], path); if (target) openLocation(target, 1);
            }
          }}
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
        : assertions.every(a => a.state === "passing")
          ? `Assertions passed: ${assertions.map(a => `#${a.id}`).join(", ")}.`
          : `Assertions not yet verified: ${assertions.filter(a => a.state !== "passing").map(a => `#${a.id}`).join(", ")}.`}
    </p>
  );
}
