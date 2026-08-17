// The RIGHT line-number rail: a slim display-only column on the far edge of
// an editor, mirroring the numbers CodeMirror's left gutter shows. Lineage
// ribbons anchor on the OUTER edge of whichever rail faces the channel, so a
// reader sees exactly which line numbers a ribbon spans — on both sides.
//
// Kept in sync the way the ribbon overlay is: the editor's own height map is
// the source of truth (viewportLineBlocks, read inside requestMeasure so the
// geometry is post-layout), redrawn on scroll, on size changes of the
// scroller or the content, and on the RAIL_SYNC_EVENT a highlight dispatch
// fires. Only the viewport's lines are ever rendered, so a 10,000-line file
// costs what its visible screen costs.
//
// Display only, on purpose: pointer-events pass through it, it never affects
// editing, wrapping, or search, and the debugger's breakpoint gutter stays a
// left-gutter concern.

import { useEffect, useRef } from "react";
import { BlockType } from "@codemirror/view";
import type { EditorView } from "@codemirror/view";

import { railLines, railWidthCh, textExtent } from "../lib/rightRail";
import { lineHighlightField, RAIL_SYNC_EVENT } from "./lineHighlight";

/** Redraw the rail's entries from a measured view. Imperative DOM: the rail
 * repaints on every scrolled frame, and a React render per frame is a cost
 * with no benefit for a list of numbers nothing interacts with. */
function redraw(rail: HTMLElement, view: EditorView): void {
  const doc = view.state.doc;
  const railTop = rail.getBoundingClientRect().top;
  // Document-relative tops become rail-relative through the document's own
  // screen position — the same trick Ribbons.tsx uses, so the two never
  // disagree about where a line is.
  const offset = view.documentTop - railTop;
  const lines = railLines(
    view.viewportLineBlocks.map((b) => {
      // A composite block (line + block widgets) reports the widgets'
      // rows in its extent; the number belongs beside the TEXT only.
      const children = Array.isArray(b.type)
        ? b.type.map((c) => ({
            text: c.type === BlockType.Text,
            top: c.top,
            height: c.height,
          }))
        : null;
      const extent = textExtent(b, children);
      return { from: b.from, top: extent.top, height: extent.height };
    }),
    (pos) => doc.lineAt(pos).number,
  );
  const hl = view.state.field(lineHighlightField, false) ?? null;
  const clamp = (pos: number) => Math.max(0, Math.min(pos, doc.length));
  const tint = hl
    ? {
        from: doc.lineAt(clamp(hl.from)).number,
        to: doc.lineAt(clamp(Math.max(hl.from, hl.to))).number,
        color: hl.color,
      }
    : null;

  rail.style.width = `${railWidthCh(doc.lines)}ch`;
  while (rail.children.length > lines.length) rail.lastElementChild?.remove();
  while (rail.children.length < lines.length) {
    rail.appendChild(document.createElement("div"));
  }
  lines.forEach((line, i) => {
    const el = rail.children[i] as HTMLElement;
    const tinted = tint && line.line >= tint.from && line.line <= tint.to;
    el.className = `cm-right-rail__line${tinted ? ` cm-linehl cm-linehl-c${tint.color}` : ""}`;
    el.textContent = String(line.line);
    el.style.top = `${line.top + offset}px`;
    el.style.height = `${line.height}px`;
  });
}

export function RightRail({ view }: { view: EditorView | null }) {
  const railRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const rail = railRef.current;
    if (!view || !rail) return;
    let frame: number | null = null;
    const schedule = () => {
      if (frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        // Read inside the measure phase: the height map and the DOM agree
        // there, where a direct read mid-update could see either half.
        view.requestMeasure({ read: (v) => redraw(rail, v) });
      });
    };
    schedule();

    view.scrollDOM.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    window.addEventListener(RAIL_SYNC_EVENT, schedule);
    // jsdom has no ResizeObserver; the interval below still covers tests.
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    observer?.observe(view.scrollDOM);
    // Content height is what changes when lines are added, removed, or
    // re-wrapped — the edits a scroll listener never sees.
    observer?.observe(view.contentDOM);
    // The same last-resort tick the ribbon overlay keeps: geometry that
    // changed without an event (a font loading, a widget settling).
    const timer = window.setInterval(schedule, 500);

    return () => {
      view.scrollDOM.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      window.removeEventListener(RAIL_SYNC_EVENT, schedule);
      observer?.disconnect();
      window.clearInterval(timer);
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [view]);

  return <div ref={railRef} className="cm-right-rail" aria-hidden="true" />;
}
