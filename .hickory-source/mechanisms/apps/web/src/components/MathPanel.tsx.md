<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MathPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-79c748e2d9c5"></a>
## `apps/web/src/components/MathPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An Equation Renders And Shows Its Source](../../../../../guarantees/authoring/an-equation-renders-and-shows-its-source/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md).

<a id="f-79c748e2d9c5-MathPanelProps"></a>
### `MathPanelProps`

A `<hick:math>` block, typeset.
The block twin of the inline maths in editor/mathRender.ts, and it shares
that module's engine loader so KaTeX and its fonts are fetched once per
page however the first equation arrived.
Deliberately thin. A rendered card is something to READ — every verb an
equation has ("show me the LaTeX") lives on the action rail beside it, in
the one column that exists to be clicked. See

<storycode:copy id="f-79c748e2d9c5-MathPanelProps">
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

</storycode:copy>
<a id="f-79c748e2d9c5-MathPanel"></a>
### `MathPanel`

<storycode:copy id="f-79c748e2d9c5-MathPanel">
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
</storycode:copy>

### Assemble `apps/web/src/components/MathPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MathPanel.tsx"><storycode:paste select="#f-79c748e2d9c5-MathPanelProps"/><storycode:paste select="#f-79c748e2d9c5-MathPanel"/></storycode:file>

</storycode:doc>
