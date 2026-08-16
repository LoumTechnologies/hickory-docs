// Provenance ribbons, drawn between panes.
//
// The old Split view drew these between three fixed columns and carried its
// own file tree to do it. That made the chrome depend on the arrangement —
// a "generated files" list that existed in one layout and nowhere else — so
// the bands moved here, where they are an overlay over whatever panes happen
// to be open: a document on one side, a file it generated on the other.
//
// The middle anchor is gone with the tree. A band now runs from the source
// block straight to the text it produced, which is the relationship anyone
// was ever reading out of it.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";

import { structureOf } from "../editor/wysiwyg";
import { atLeast, clampBand, ribbonPath, thicknessFor } from "../lib/ribbonGeometry";
import { deriveRibbons, RIBBON_PALETTE_SIZE, type Ribbon } from "../lib/ribbons";
import type { OutputFile } from "../api/types";

/** One pane's editor, registered so the overlay can measure it. */
export interface RibbonSource {
  view: EditorView;
  /** The document's path, for matching provenance. */
  docPath: string;
  /** The document's text, as the server last wove it. */
  docSource: string;
}

export interface RibbonTarget {
  view: EditorView;
  file: OutputFile;
}

interface Shape {
  key: string;
  path: string;
  color: number;
  clamped: boolean;
}

export function RibbonOverlay({
  container,
  source,
  targets,
}: {
  container: HTMLElement | null;
  source: RibbonSource | null;
  targets: readonly RibbonTarget[];
}) {
  const [shapes, setShapes] = useState<Shape[]>([]);
  const frame = useRef<number | null>(null);

  const measure = useCallback(() => {
    if (!container || !source || targets.length === 0) {
      setShapes((current) => (current.length === 0 ? current : []));
      return;
    }
    const box = container.getBoundingClientRect();
    const left = source.view.scrollDOM.getBoundingClientRect();
    // The channel a ribbon is drawn through is the space between the two
    // TEXTS, not between the two panes: the gutters on either side belong to
    // it. Anchoring on the scrollers left four pixels of divider to draw in,
    // which is not a picture anyone can read.
    const leftText = source.view.contentDOM.getBoundingClientRect();
    const structure = structureOf(source.view.state);
    const sourceLength = source.view.state.doc.length;
    const out: Shape[] = [];
    let colour = 0;
    const colours = new Map<string, number>();

    for (const target of targets) {
      const right = target.view.scrollDOM.getBoundingClientRect();
      const rightText = target.view.contentDOM.getBoundingClientRect();
      // Panes can be in any order, so the curve is drawn from whichever edge
      // faces the other: a generated file to the LEFT of its document is an
      // arrangement someone is allowed to build.
      const forward = left.right <= right.left;
      const x0 = (forward ? leftText.right : leftText.left) - box.left;
      const x1 = (forward ? rightText.left : rightText.right) - box.left;
      const targetLength = target.view.state.doc.length;

      for (const ribbon of deriveRibbons(target.file, source.docPath, source.docSource)) {
        const band = sourceBand(source.view, structure, ribbon, sourceLength);
        if (!band) continue;
        const to = outputBand(target.view, ribbon, targetLength);
        if (!to) continue;

        const from = clampBand(band[0] - box.top, band[1] - box.top, left.top - box.top, left.bottom - box.top);
        const into = clampBand(to[0] - box.top, to[1] - box.top, right.top - box.top, right.bottom - box.top);
        const thickness = thicknessFor(ribbon.bytes, 1);
        const a = atLeast(from.yTop, from.yBot, Math.max(3, thickness));
        const b = atLeast(into.yTop, into.yBot, Math.max(3, thickness));

        const fragment = `${ribbon.sourceByteSpan[0]}:${ribbon.sourceByteSpan[1]}`;
        let assigned = colours.get(fragment);
        if (assigned === undefined) {
          assigned = colour % RIBBON_PALETTE_SIZE;
          colours.set(fragment, assigned);
          colour += 1;
        }

        out.push({
          key: `${target.file.path}:${ribbon.key}`,
          path: ribbonPath(x0, a.yTop, a.yBot, x1, b.yTop, b.yBot),
          color: assigned,
          clamped: from.clamped || into.clamped,
        });
      }
    }
    setShapes(out);
  }, [container, source, targets]);

  // Measurement follows the things that move: scrolling either pane, editing
  // either buffer, and the window changing shape. Throttled to a frame,
  // because all three can happen at once and the answer is the same.
  useEffect(() => {
    const schedule = () => {
      if (frame.current !== null) return;
      frame.current = requestAnimationFrame(() => {
        frame.current = null;
        measure();
      });
    };
    schedule();

    const views = [source?.view, ...targets.map((t) => t.view)].filter(Boolean) as EditorView[];
    const scrollers = views.map((view) => view.scrollDOM);
    for (const scroller of scrollers) scroller.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    const observer = new ResizeObserver(schedule);
    if (container) observer.observe(container);
    const timer = window.setInterval(schedule, 500);

    return () => {
      for (const scroller of scrollers) scroller.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      observer.disconnect();
      window.clearInterval(timer);
      if (frame.current !== null) cancelAnimationFrame(frame.current);
      frame.current = null;
    };
  }, [measure, container, source, targets]);

  if (shapes.length === 0) return null;
  return (
    <svg className="ribbon-layer" aria-hidden="true">
      {shapes.map((shape) => (
        <path
          key={shape.key}
          d={shape.path}
          className={`ribbon ribbon-c${shape.color}${shape.clamped ? " ribbon-clamped" : ""}`}
        />
      ))}
    </svg>
  );
}

/**
 * The vertical extent of a ribbon's source, in client coordinates.
 *
 * The whole enclosing block, not the fragment: a band that covers the copy
 * block says "this block became that text", which is the claim. A band over
 * three characters of it says nothing anyone can read at a glance.
 */
function sourceBand(
  view: EditorView,
  structure: ReturnType<typeof structureOf>,
  ribbon: Ribbon,
  length: number,
): [number, number] | null {
  let from = Math.min(ribbon.sourceSpan[0], length);
  let to = Math.min(ribbon.sourceSpan[1], length);
  for (const block of structure.blocks) {
    if (
      (block.name === "copy" || block.name === "cut" || block.name === "file") &&
      ribbon.sourceSpan[0] >= block.from &&
      ribbon.sourceSpan[1] <= block.to
    ) {
      from = Math.min(block.from, length);
      to = Math.min(block.to, length);
      break;
    }
  }
  return bandBetween(view, from, to);
}

function outputBand(view: EditorView, ribbon: Ribbon, length: number): [number, number] | null {
  const from = Math.min(ribbon.outputRange[0], length);
  const to = Math.min(ribbon.outputRange[1], length);
  return bandBetween(view, from, to);
}

/**
 * Screen extent of a range, including one scrolled out of sight.
 *
 * `coordsAtPos` answers for rendered text only — everything outside the
 * viewport returns null, and a ribbon that cannot be measured is a ribbon
 * that is not drawn. Six relationships showed one. The height map knows where
 * every line is whether or not it is on screen, and the content box moves
 * with the scroll, so the two together give an answer for any position; the
 * caller clamps it to the visible pane.
 */
function bandBetween(view: EditorView, from: number, to: number): [number, number] | null {
  const content = view.contentDOM.getBoundingClientRect();
  const first = view.lineBlockAt(Math.max(0, from));
  const last = view.lineBlockAt(Math.max(0, to));
  const start = { top: content.top + first.top, bottom: content.top + first.bottom };
  const end = { top: content.top + last.top, bottom: content.top + last.bottom };
  return [Math.min(start.top, end.top), Math.max(start.bottom, end.bottom)];
}
