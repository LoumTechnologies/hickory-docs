// The ruler across the top of an editor, and the margin marker you drag.
//
// The reference is the one every word processor has had since the eighties: a
// measuring stick above the text, with a marker showing where the right
// margin is, which you drag. People already know what it is and what dragging
// it does, which is worth more than any control we could invent.
//
// It measures against the editor's own content element rather than against
// its own box, so the ticks line up with the characters underneath even
// though the gutter's width changes with the line count and the pane's width
// changes when a divider moves. That measurement is read inside
// `requestMeasure`, where CodeMirror's height map and the DOM agree — the
// same discipline RightRail.tsx follows, for the same reason.
//
// It is the content element's PADDING box that column zero sits at, not its
// border box. `.cm-content` carries `--cm-pad-x` (2.5rem) of padding, and the
// dotted margin line in the editor is drawn from the inside of it — so a ruler
// measured from the border box put every tick, and the marker, 2.5rem to the
// left of the line the text actually wraps at.
//
// ## What the numbers are, which is the question it kept being asked
//
// Not inches. A CSS pixel is not a physical unit — the same ruler at the same
// zoom is a different width on two monitors — so an inch scale would be a
// number that looks authoritative and measures nothing.
//
// Not monospace columns either, because this editor sets prose in the
// system's proportional face (editor/chrome.ts): an `i` and an `m` are not
// the same width, so there is no column grid for a tick to fall on.
//
// What it measures is the TYPOGRAPHER'S measure — characters per line — which
// is the number five centuries of setting text actually cares about, and the
// only one that transfers between faces. It is an average by construction:
// CodeMirror's `defaultCharacterWidth` is the content element's real font
// measured over a sample, so "72" means "about 72 characters of this face,
// at this size, fit on a line", not "72 cells". A line of `l`s will hold
// more and a line of `M`s fewer, and that is a property of the question, not
// a defect in the answer.
//
// So the strip says so, in three ways that cost nothing: the unit is named at
// its left end, the comfortable band (45–75, the range every typography text
// gives for continuous reading) is shaded so the marker's position means
// something, and the tooltip says "about".
//
// Display-only except for the marker: the ticks take no pointer events, so a
// click near the ruler that was meant for the text is not stolen by it.
//
// ## Inside a table it names columns instead
//
// A measuring stick over a grid is measuring the wrong thing: a table's
// columns are not a character count, and the prose measure does not apply to a
// block that never wraps. So when the caret is inside a table the same strip
// becomes the table's column header — A, B, C over the columns they belong to
// — which is where a spreadsheet has always put them.
//
// It reads the widths out of the table's own header cells rather than being
// told them. The grid is React inside a CodeMirror widget and the ruler is
// React outside it; a prop would have to travel up through the editor and back
// down, and would be one render behind every column drag. One measurement, of
// the element that has the real geometry, is the same discipline the prose
// measure follows.

import { useCallback, useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";

import {
  WRAP_MAX,
  WRAP_MIN,
  clampWrapColumn,
  setWrapColumn,
  wrapColumnOf,
} from "./wrapColumn";

/** Where the text starts and how wide a character is, in ruler-local pixels. */
interface Metrics {
  /** Column zero — the inside of the content element's left padding, relative
   * to the ruler's own left edge. The same edge the dotted margin line is
   * drawn from, which is the point. */
  originX: number;
  /** One character. */
  charWidth: number;
  /** How much room the ruler has to draw in. */
  width: number;
}

/** A tick every 5 columns; the number every 10. Any denser and the ruler is a
 * grey smear at the font sizes this app uses. */
const TICK_EVERY = 5;
const LABEL_EVERY = 10;

/** The comfortable measure for continuous reading, in characters per line.
 * Shaded on the ruler so the marker has something to be near or far from —
 * a number with no band around it is a number nobody can judge. */
export const READABLE_MIN = 45;
export const READABLE_MAX = 75;

/** One column of a table, as the ruler draws it. */
interface ColumnBand {
  label: string;
  /** Left edge and width, in ruler-local pixels. */
  x: number;
  width: number;
}

export function EditorRuler({
  view,
  column,
  onColumn,
  tableEl = null,
}: {
  view: EditorView | null;
  /** The measure, in columns. Owned by the tab so it can be persisted. */
  column: number;
  onColumn: (next: number) => void;
  /** The rendered table the caret is inside, or null. Present means the ruler
   * names that table's columns instead of measuring prose. */
  tableEl?: HTMLElement | null;
}) {
  const rulerRef = useRef<HTMLDivElement | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [bands, setBands] = useState<ColumnBand[]>([]);
  // Held during a drag so the marker follows the pointer without a round trip
  // through the tab's state on every pointermove.
  const [dragColumn, setDragColumn] = useState<number | null>(null);

  const measure = useCallback(() => {
    const ruler = rulerRef.current;
    if (!view || !ruler) return;
    view.requestMeasure({
      read: (v) => {
        const content = v.contentDOM.getBoundingClientRect();
        const box = ruler.getBoundingClientRect();
        // `--cm-pad-x`, read from the element that has it rather than
        // hardcoded here: two places stating the same inset is how they come
        // to disagree.
        const padLeft = parseFloat(getComputedStyle(v.contentDOM).paddingLeft) || 0;
        return {
          originX: content.left - box.left + padLeft,
          charWidth: v.defaultCharacterWidth,
          width: box.width,
        };
      },
      write: (next: Metrics) => {
        setMetrics((current) =>
          current &&
          current.originX === next.originX &&
          current.charWidth === next.charWidth &&
          current.width === next.width
            ? current
            : next,
        );
      },
    });
  }, [view]);

  useEffect(() => {
    if (!view) return;
    measure();
    window.addEventListener("resize", measure);
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measure);
    observer?.observe(view.dom);
    return () => {
      window.removeEventListener("resize", measure);
      observer?.disconnect();
    };
  }, [view, measure]);

  // A column drag, the grid scrolling sideways, and a pane divider moving all
  // change where the columns are, and none of them is a signal this component
  // otherwise hears about — so the table's own box is observed.
  const measureBands = useCallback(() => {
    const ruler = rulerRef.current;
    if (!tableEl || !ruler) {
      setBands((current) => (current.length === 0 ? current : []));
      return;
    }
    const box = ruler.getBoundingClientRect();
    const next: ColumnBand[] = [];
    for (const head of tableEl.querySelectorAll(".table-panel__head")) {
      const rect = head.getBoundingClientRect();
      // A column scrolled out of the grid's own scroller must not be drawn
      // over the prose beside it.
      if (rect.width <= 0) continue;
      next.push({ label: head.textContent ?? "", x: rect.left - box.left, width: rect.width });
    }
    setBands((current) =>
      current.length === next.length &&
      current.every((b, i) => b.label === next[i].label && b.x === next[i].x && b.width === next[i].width)
        ? current
        : next,
    );
  }, [tableEl]);

  useEffect(() => {
    measureBands();
    if (!tableEl) return;
    const scroller = tableEl.querySelector(".table-panel__scroll");
    scroller?.addEventListener("scroll", measureBands);
    window.addEventListener("resize", measureBands);
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measureBands);
    observer?.observe(tableEl);
    return () => {
      scroller?.removeEventListener("scroll", measureBands);
      window.removeEventListener("resize", measureBands);
      observer?.disconnect();
    };
  }, [tableEl, measureBands]);

  // The document's own idea of the measure is the source of truth; this keeps
  // the editor in step when the tab's value arrives from somewhere else (a
  // restored session, the settings pane).
  useEffect(() => {
    if (!view) return;
    if (wrapColumnOf(view.state) !== clampWrapColumn(column)) {
      view.dispatch({ effects: setWrapColumn.of(clampWrapColumn(column)) });
    }
  }, [view, column]);

  const shown = clampWrapColumn(dragColumn ?? column);

  const columnAt = (clientX: number): number => {
    const ruler = rulerRef.current;
    if (!ruler || !metrics || metrics.charWidth <= 0) return shown;
    const x = clientX - ruler.getBoundingClientRect().left - metrics.originX;
    return clampWrapColumn(Math.round(x / metrics.charWidth));
  };

  const onPointerDown = (event: React.PointerEvent) => {
    event.preventDefault();
    (event.target as HTMLElement).setPointerCapture?.(event.pointerId);
    setDragColumn(columnAt(event.clientX));
  };

  const onPointerMove = (event: React.PointerEvent) => {
    if (dragColumn === null) return;
    setDragColumn(columnAt(event.clientX));
  };

  const endDrag = (event: React.PointerEvent) => {
    if (dragColumn === null) return;
    const next = columnAt(event.clientX);
    setDragColumn(null);
    // Reported once, at the end: a tab that persisted its measure on every
    // pointermove would write a hundred times per drag.
    if (next !== column) onColumn(next);
  };

  const ticks: { column: number; x: number; label: boolean }[] = [];
  if (metrics && metrics.charWidth > 0) {
    const last = Math.min(
      WRAP_MAX,
      Math.floor((metrics.width - metrics.originX) / metrics.charWidth),
    );
    for (let c = 0; c <= last; c += TICK_EVERY) {
      ticks.push({
        column: c,
        x: metrics.originX + c * metrics.charWidth,
        label: c > 0 && c % LABEL_EVERY === 0,
      });
    }
  }

  const markerX = metrics ? metrics.originX + shown * metrics.charWidth : 0;
  const band = metrics
    ? {
        left: metrics.originX + READABLE_MIN * metrics.charWidth,
        width: (READABLE_MAX - READABLE_MIN) * metrics.charWidth,
      }
    : null;

  const naming = bands.length > 0;

  return (
    <div
      className={`editor-ruler${naming ? " editor-ruler--columns" : ""}`}
      ref={rulerRef}
      data-testid="editor-ruler"
    >
      {naming && (
        <div className="editor-ruler__columns" aria-hidden="true">
          {bands.map((band) => (
            <span
              key={band.label + band.x}
              className="editor-ruler__column"
              style={{ left: `${band.x}px`, width: `${band.width}px` }}
            >
              {band.label}
            </span>
          ))}
        </div>
      )}
      {/* The unit, at the left end, in the lane the gutter occupies below.
          A measuring stick whose unit is not written on it is furniture
          people learn to ignore. */}
      {!naming && (
        <span className="editor-ruler__unit" aria-hidden="true">
          chars/line
        </span>
      )}
      {band && !naming && (
        <span
          className="editor-ruler__band"
          aria-hidden="true"
          style={{ left: `${band.left}px`, width: `${band.width}px` }}
        />
      )}
      <div className="editor-ruler__ticks" aria-hidden="true" hidden={naming}>
        {ticks.map((tick) => (
          <span
            key={tick.column}
            className={`editor-ruler__tick${tick.label ? " editor-ruler__tick--labelled" : ""}`}
            style={{ left: `${tick.x}px` }}
          >
            {tick.label ? tick.column : ""}
          </span>
        ))}
      </div>
      {/* The marker is a real control: it has a name, a value, and arrow keys,
          because a margin you can only set by dragging is a margin nobody can
          set precisely or without a mouse. */}
      <button
        type="button"
        role="slider"
        hidden={naming}
        className={`editor-ruler__marker${dragColumn !== null ? " editor-ruler__marker--dragging" : ""}`}
        style={{ left: `${markerX}px` }}
        aria-label="Where prose wraps"
        aria-valuemin={WRAP_MIN}
        aria-valuemax={WRAP_MAX}
        aria-valuenow={shown}
        aria-valuetext={`about ${shown} characters per line`}
        data-tip={
          `Prose wraps at about ${shown} characters per line — drag to move it. ` +
          `The face is proportional, so this is an average, not a column count. ` +
          `${READABLE_MIN}\u2013${READABLE_MAX} (shaded) is the comfortable range for ` +
          `reading. Code never wraps.`
        }
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onKeyDown={(event) => {
          const step = event.shiftKey ? 10 : 1;
          if (event.key === "ArrowLeft") {
            event.preventDefault();
            onColumn(clampWrapColumn(column - step));
          } else if (event.key === "ArrowRight") {
            event.preventDefault();
            onColumn(clampWrapColumn(column + step));
          }
        }}
      >
        ▽
      </button>
    </div>
  );
}
