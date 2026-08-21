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
// Display-only except for the marker: the ticks take no pointer events, so a
// click near the ruler that was meant for the text is not stolen by it.

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
  /** Left edge of the text, relative to the ruler's own left edge. */
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

export function EditorRuler({
  view,
  column,
  onColumn,
}: {
  view: EditorView | null;
  /** The measure, in columns. Owned by the tab so it can be persisted. */
  column: number;
  onColumn: (next: number) => void;
}) {
  const rulerRef = useRef<HTMLDivElement | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
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
        return {
          originX: content.left - box.left,
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

  return (
    <div className="editor-ruler" ref={rulerRef} data-testid="editor-ruler">
      <div className="editor-ruler__ticks" aria-hidden="true">
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
        className={`editor-ruler__marker${dragColumn !== null ? " editor-ruler__marker--dragging" : ""}`}
        style={{ left: `${markerX}px` }}
        aria-label="Where prose wraps"
        aria-valuemin={WRAP_MIN}
        aria-valuemax={WRAP_MAX}
        aria-valuenow={shown}
        aria-valuetext={`${shown} columns`}
        data-tip={`Prose wraps at ${shown} columns — drag to move it. Code never wraps.`}
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
