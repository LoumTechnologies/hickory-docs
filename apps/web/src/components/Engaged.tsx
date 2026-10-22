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

import type { ReactNode } from "react";
import { useWidgetEngagement } from "./useWidgetEngagement";

export function Engaged({ className, children }: { className?: string; children: ReactNode }) {
  const engagement = useWidgetEngagement("embedded widget");
  return <div ref={engagement.root} {...engagement.handlers} tabIndex={-1}
    className={`engaged-gate${engagement.engaged ? " engaged-gate--on" : ""}${className ? ` ${className}` : ""}`}>
    {children}
  </div>;
}
