// The map-in-a-page rule, as a wrapper any embedded widget can stand in.
//
// A widget portalled into the document competes with it for the wheel: a
// table that scrolls internally, a canvas that zooms, a terminal that pages
// all turn "scrolling past" into "trapped inside". The rule that fixes every
// case the same way: **the wheel belongs to whoever owns the moment.** At
// rest the widget is scenery and the wheel scrolls the document; clicking
// into the widget engages it (focus is the signal, exactly as it is for
// keys — see GraphEditorPanel, which implements the same rule inline because
// it must also feed the state to React Flow); Escape, or clicking away,
// hands the wheel back.
//
// What "engaged" gates is the widget's own business, keyed off the
// `engaged-gate--on` class — the table hides its internal overflow until
// engaged, the canvas turns zoom-on-scroll on. This wrapper only owns the
// signal.

import { useState } from "react";
import type { ReactNode } from "react";

export function Engaged({
  className,
  children,
}: {
  className?: string;
  children: ReactNode;
}) {
  const [on, setOn] = useState(false);
  return (
    <div
      className={`engaged-gate${on ? " engaged-gate--on" : ""}${className ? ` ${className}` : ""}`}
      tabIndex={-1}
      onFocus={() => setOn(true)}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget as Element | null)) {
          setOn(false);
        }
      }}
      onKeyDown={(event) => {
        // Escape disengages — but not when a field inside is using Escape
        // for its own cancel (a cell edit, a rename): that Escape means
        // "undo my typing", not "I am done in here".
        const target = event.target as HTMLElement;
        if (event.key === "Escape" && !target.closest("input, textarea")) {
          (event.currentTarget as HTMLElement).blur();
        }
      }}
      onPointerDownCapture={(event) => {
        const root = event.currentTarget;
        const target = event.target as HTMLElement;
        if (target.closest("input, textarea, [contenteditable]")) return;
        if (!root.contains(document.activeElement)) root.focus({ preventScroll: true });
      }}
    >
      {children}
    </div>
  );
}
