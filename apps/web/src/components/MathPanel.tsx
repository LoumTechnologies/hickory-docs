// A `<hick:math>` block, typeset.
//
// The block twin of the inline maths in editor/mathRender.ts, and it shares
// that module's engine loader so KaTeX and its fonts are fetched once per
// page however the first equation arrived.
//
// Deliberately thin. A rendered card is something to READ — every verb an
// equation has ("show me the LaTeX") lives on the action rail beside it, in
// the one column that exists to be clicked. See
// docs/guarantees/authoring/the-gutters-never-skip-a-number.md.

import { useEffect, useRef, useState } from "react";

import { useUiReading } from "../lib/uiReading";
import { renderMathInto } from "../editor/mathRender";

export interface MathPanelProps {
  /** The LaTeX between the tags, exactly as written. */
  source: string;
}

export function MathPanel({ source }: MathPanelProps) {
  const [rendering, setRendering] = useState<{ source: string; status: string; error: string | null } | null>(null);
  useUiReading("math rendering", { source, display: "block", rendering: !source.trim() ? { status: "empty" } : rendering?.source === source ? rendering : { status: "pending" }, scope: "Authored LaTeX and actual renderer/fallback state; typesetting does not verify mathematical truth" });
  const hostRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    let cancelled = false;
    // The source stands in until the engine arrives: nothing flashes empty on
    // a cold load, and a build with no KaTeX still shows something true.
    host.textContent = source.trim();
    // Render offscreen; a superseded async completion cannot overwrite the
    // current equation or report old renderer state under new source.
    const target = document.createElement("div");
    void renderMathInto(target, source, true).then(() => {
      if (cancelled) return;
      const fallback = target.classList.contains("cm-math--unparsed");
      host.replaceChildren(...Array.from(target.childNodes));
      host.classList.toggle("cm-math--unparsed", fallback);
      if (fallback) host.dataset.tip = target.dataset.tip; else delete host.dataset.tip;
      setRendering({ source, status: fallback ? "source_fallback" : "rendered", error: fallback ? target.dataset.tip ?? null : null });
    });
    return () => {
      cancelled = true;
    };
  }, [source]);

  if (!source.trim()) {
    return (
      <div className="math-panel math-panel--empty">
        <p className="muted">An empty equation — nothing to typeset yet.</p>
      </div>
    );
  }

  return (
    <div className="math-panel" data-testid="math-panel">
      <div className="math-figure" ref={hostRef} />
    </div>
  );
}
