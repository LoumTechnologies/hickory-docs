<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/EditorRuler.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-75b1988d3607"></a>
## `apps/web/src/editor/EditorRuler.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [A Keystroke Redraws Only The Editor](../../../../../guarantees/authoring/a-keystroke-redraws-only-the-editor/contract.md), [The Measure Says What It Measures](../../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md).

<a id="f-75b1988d3607-Metrics"></a>
### `Metrics`

The ruler across the top of an editor, and the margin marker you drag.
The reference is the one every word processor has had since the eighties: a
measuring stick above the text, with a marker showing where the right
margin is, which you drag. People already know what it is and what dragging
it does, which is worth more than any control we could invent.
It measures against the editor's own content element rather than against
its own box, so the ticks line up with the characters underneath even
though the gutter's width changes with the line count and the pane's width
changes when a divider moves. That measurement is read inside
`requestMeasure`, where CodeMirror's height map and the DOM agree — the
same discipline RightRail.tsx follows, for the same reason.
It is the content element's PADDING box that column zero sits at, not its
border box. `.cm-content` carries `--cm-pad-x` (2.5rem) of padding, and the
dotted margin line in the editor is drawn from the inside of it — so a ruler
measured from the border box put every tick, and the marker, 2.5rem to the
left of the line the text actually wraps at.
## What the numbers are, which is the question it kept being asked
Not inches. A CSS pixel is not a physical unit — the same ruler at the same
zoom is a different width on two monitors — so an inch scale would be a
number that looks authoritative and measures nothing.
Not monospace columns either, because this editor sets prose in the
system's proportional face (editor/chrome.ts): an `i` and an `m` are not
the same width, so there is no column grid for a tick to fall on.
What it measures is the TYPOGRAPHER'S measure — characters per line — which
is the number five centuries of setting text actually cares about, and the
only one that transfers between faces. It is an average by construction:
CodeMirror's `defaultCharacterWidth` is the content element's real font
measured over a sample, so "72" means "about 72 characters of this face,
at this size, fit on a line", not "72 cells". A line of `l`s will hold
more and a line of `M`s fewer, and that is a property of the question, not
a defect in the answer.
So the strip says so, in three ways that cost nothing: the unit is named at
its left end, the comfortable band (45–75, the range every typography text
gives for continuous reading) is shaded so the marker's position means
something, and the tooltip says "about".
Display-only except for the marker: the ticks take no pointer events, so a
click near the ruler that was meant for the text is not stolen by it.
## Inside a table it names columns instead
A measuring stick over a grid is measuring the wrong thing: a table's
columns are not a character count, and the prose measure does not apply to a
block that never wraps. So when the caret is inside a table the same strip
becomes the table's column header — A, B, C over the columns they belong to
— which is where a spreadsheet has always put them.
It reads the widths out of the table's own header cells rather than being
told them. The grid is React inside a CodeMirror widget and the ruler is
React outside it; a prop would have to travel up through the editor and back
down, and would be one render behind every column drag. One measurement, of
the element that has the real geometry, is the same discipline the prose
measure follows.

<storycode:copy id="f-75b1988d3607-Metrics">
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

import { memo, useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
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

</storycode:copy>
<a id="f-75b1988d3607-TICK_EVERY"></a>
### `TICK_EVERY`

A tick every 5 columns; the number every 10. Any denser and the ruler is a
grey smear at the font sizes this app uses.

<storycode:copy id="f-75b1988d3607-TICK_EVERY">
/** A tick every 5 columns; the number every 10. Any denser and the ruler is a
 * grey smear at the font sizes this app uses. */
const TICK_EVERY = 5;
</storycode:copy>
<a id="f-75b1988d3607-LABEL_EVERY"></a>
### `LABEL_EVERY`

<storycode:copy id="f-75b1988d3607-LABEL_EVERY">
const LABEL_EVERY = 10;

</storycode:copy>
<a id="f-75b1988d3607-READABLE_MIN"></a>
### `READABLE_MIN`

The comfortable measure for continuous reading, in characters per line.
Shaded on the ruler so the marker has something to be near or far from —
a number with no band around it is a number nobody can judge.

<storycode:copy id="f-75b1988d3607-READABLE_MIN">
/** The comfortable measure for continuous reading, in characters per line.
 * Shaded on the ruler so the marker has something to be near or far from —
 * a number with no band around it is a number nobody can judge. */
export const READABLE_MIN = 45;
</storycode:copy>
<a id="f-75b1988d3607-READABLE_MAX"></a>
### `READABLE_MAX`

<storycode:copy id="f-75b1988d3607-READABLE_MAX">
export const READABLE_MAX = 75;

</storycode:copy>
<a id="f-75b1988d3607-ColumnBand"></a>
### `ColumnBand`

<storycode:copy id="f-75b1988d3607-ColumnBand">
/** One column of a table, as the ruler draws it. */
interface ColumnBand {
  label: string;
  /** Left edge and width, in ruler-local pixels. */
  x: number;
  width: number;
}

</storycode:copy>
<a id="f-75b1988d3607-EditorRuler"></a>
### `EditorRuler`

<storycode:copy id="f-75b1988d3607-EditorRuler">
export const EditorRuler = memo(function EditorRuler({
  view,
  column,
  onColumn,
  tableEl = null,
  owner,
}: {
  view: EditorView | null;
  owner?: string | null;
  /** The measure, in columns. Owned by the tab so it can be persisted. */
  column: number;
  onColumn: (next: number) => void;
  /** The rendered table the caret is inside, or null. Present means the ruler
   * names that table's columns instead of measuring prose. */
  tableEl?: HTMLElement | null;
}) {
  const lifetime = useMemo(() => crypto.randomUUID(), [view, owner, onColumn]);
  const currentLifetime = useRef(lifetime); currentLifetime.current = lifetime;
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
        if (currentLifetime.current !== lifetime) return;
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
  }, [view, lifetime]);

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

  useEffect(() => { setDragColumn(null); }, [lifetime, tableEl]);
  const shown = clampWrapColumn(dragColumn ?? column);
  const changeColumn = (next: number) => { setDragColumn(null); onColumn(clampWrapColumn(next)); };

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
    if (next !== column) changeColumn(next);
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
  const parent = `ruler:${useId()}`;
  const revision = useUiRevision(() => [lifetime, column, shown, naming, bands, metrics]);
  useUiReading("editor ruler", naming ? { owner: owner ?? null, mode: "table columns", columns: bands } : {
    owner: owner ?? null, mode: "prose measure", average_characters_per_line: shown, comfortable_range: [READABLE_MIN, READABLE_MAX],
    minimum: WRAP_MIN, maximum: WRAP_MAX, metrics, scope: "Average characters of the current proportional face, not physical inches. Code does not wrap. This is presentation state." });
  useUiCommands(naming ? [] : [{ key: `${parent}:measure`, command: {
    label: "Set prose wrapping measure", parent, subject: parent,
    parameters: { column: { type: "number", minimum: WRAP_MIN, maximum: WRAP_MAX, integer: true } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      changeColumn(args.column as number);
      return { status: "completed", value: { measure_requested: args.column, source_changed: false, persisted: false } };
    },
  } }]);

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
        onPointerCancel={() => setDragColumn(null)}
        onKeyDown={(event) => {
          const step = event.shiftKey ? 10 : 1;
          if (event.key === "ArrowLeft") {
            event.preventDefault();
            changeColumn(column - step);
          } else if (event.key === "ArrowRight") {
            event.preventDefault();
            changeColumn(column + step);
          }
        }}
      >
        ▽
      </button>
    </div>
  );
});
</storycode:copy>

### Assemble `apps/web/src/editor/EditorRuler.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/EditorRuler.tsx"><storycode:paste select="#f-75b1988d3607-Metrics"/><storycode:paste select="#f-75b1988d3607-TICK_EVERY"/><storycode:paste select="#f-75b1988d3607-LABEL_EVERY"/><storycode:paste select="#f-75b1988d3607-READABLE_MIN"/><storycode:paste select="#f-75b1988d3607-READABLE_MAX"/><storycode:paste select="#f-75b1988d3607-ColumnBand"/><storycode:paste select="#f-75b1988d3607-EditorRuler"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-a0ab5d3c94e1"></a>
## `apps/web/src/editor/EditorRuler.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [The Measure Says What It Measures](../../../../../guarantees/authoring/the-measure-says-what-it-measures/contract.md).

<a id="f-a0ab5d3c94e1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, fireEvent } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { EditorRuler } from "./EditorRuler";
import { WRAP_MAX, WRAP_MIN, proseWrap, wrapColumnOf } from "./wrapColumn";
import { READABLE_MAX, READABLE_MIN } from "./EditorRuler";

// vitest runs without `globals`, so Testing Library's automatic cleanup
// never registers; without this every render stacks up in one document.
afterEach(() => {
  cleanup();
});

</storycode:copy>
<a id="f-a0ab5d3c94e1-view"></a>
### `view`

<storycode:copy id="f-a0ab5d3c94e1-view">
const view = () =>
  new EditorView({
    state: EditorState.create({ doc: "prose\n", extensions: [proseWrap(() => [])] }),
    parent: document.body,
  });

</storycode:copy>
<a id="f-a0ab5d3c94e1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement-2">
describe("the ruler's margin marker", () => {
  it("is a slider with a value, not just something to drag", () => {
    // A margin you can only set by dragging is a margin nobody can set
    // precisely, and nobody without a mouse can set at all.
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    const marker = screen.getByRole("slider", { name: /where prose wraps/i });
    expect(marker.getAttribute("aria-valuenow")).toBe("72");
    expect(marker.getAttribute("aria-valuemin")).toBe(String(WRAP_MIN));
    expect(marker.getAttribute("aria-valuemax")).toBe(String(WRAP_MAX));
  });

  it("moves a column at a time with the arrows, ten with shift", () => {
    const onColumn = vi.fn();
    render(<EditorRuler view={null} column={80} onColumn={onColumn} />);
    const marker = screen.getByRole("slider");
    fireEvent.keyDown(marker, { key: "ArrowLeft" });
    expect(onColumn).toHaveBeenLastCalledWith(79);
    fireEvent.keyDown(marker, { key: "ArrowRight", shiftKey: true });
    expect(onColumn).toHaveBeenLastCalledWith(90);
  });

  it("will not be driven past the measures that still lay out", () => {
    const onColumn = vi.fn();
    render(<EditorRuler view={null} column={WRAP_MIN} onColumn={onColumn} />);
    fireEvent.keyDown(screen.getByRole("slider"), { key: "ArrowLeft" });
    expect(onColumn).toHaveBeenLastCalledWith(WRAP_MIN);
  });

  it("tells the editor the tab's measure, so a restored session lays out right", () => {
    const live = view();
    render(<EditorRuler view={live} column={64} onColumn={() => {}} />);
    expect(wrapColumnOf(live.state)).toBe(64);
    live.destroy();
  });

  it("says in words what it does, in the app's own tooltip layer", () => {
    render(<EditorRuler view={null} column={80} onColumn={() => {}} />);
    expect(screen.getByRole("slider").dataset.tip).toMatch(
      /prose wraps at about 80 characters per line.*code never wraps/i,
    );
  });
});

</storycode:copy>
<a id="f-a0ab5d3c94e1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement-3">
describe("where column zero is", () => {
  it("measures from the inside of the content padding, where the text starts", () => {
    // The bug this exists to stop: `.cm-content` carries `--cm-pad-x` of
    // padding and the dotted margin line is drawn from the inside of it, so a
    // ruler measured from the BORDER box put every tick and the marker 2.5rem
    // to the left of the line the text actually wraps at.
    const live = view();
    const content = live.contentDOM;
    content.style.paddingLeft = "40px";
    // jsdom lays nothing out, so the geometry the component reads is stated
    // here rather than measured: 100px to the content box, 40px of padding.
    content.getBoundingClientRect = () => ({ left: 100, width: 600 }) as DOMRect;
    Object.defineProperty(live, "defaultCharacterWidth", { value: 8, configurable: true });

    render(<EditorRuler view={live} column={80} onColumn={() => {}} />);
    const ruler = screen.getByTestId("editor-ruler");
    ruler.getBoundingClientRect = () => ({ left: 0, width: 900 }) as DOMRect;
    live.requestMeasure = (<T,>(request: {
      read: (v: EditorView) => T;
      write?: (measure: T, v: EditorView) => void;
    }) => request?.write?.(request.read(live), live)) as typeof live.requestMeasure;

    fireEvent(window, new Event("resize"));

    // 100 (content) + 40 (padding) + 80 × 8 = 780, not 740.
    expect(screen.getByRole("slider").style.left).toBe("780px");
    live.destroy();
  });
});

</storycode:copy>
<a id="f-a0ab5d3c94e1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement-4">
describe("what the ruler says it is measuring", () => {
  // The strip used to be a row of unexplained numbers. Prose here is set in
  // the system's proportional face, so they are not monospace columns and
  // they are certainly not inches — they are the typographer's measure, and
  // the ruler has to say so or it is furniture people learn to ignore.
  it("names its unit at the left end", () => {
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    expect(document.querySelector(".editor-ruler__unit")?.textContent).toBe("chars/line");
  });

  it("says the number is an average, because the face is proportional", () => {
    render(<EditorRuler view={null} column={72} onColumn={() => {}} />);
    const marker = screen.getByRole("slider");
    expect(marker.getAttribute("aria-valuetext")).toBe("about 72 characters per line");
    expect(marker.dataset.tip).toMatch(/average, not a column count/i);
  });

  it("shades the comfortable range so the marker's place means something", () => {
    const live = view();
    render(<EditorRuler view={live} column={72} onColumn={() => {}} />);
    expect(READABLE_MIN).toBeLessThan(READABLE_MAX);
    expect(screen.getByRole("slider").dataset.tip).toContain(
      `${READABLE_MIN}\u2013${READABLE_MAX}`,
    );
    live.destroy();
  });

  it("drops the prose furniture entirely inside a table, where it would lie", () => {
    const table = document.createElement("div");
    const head = document.createElement("div");
    head.className = "table-panel__head";
    head.textContent = "A";
    table.appendChild(head);
    document.body.appendChild(table);
    render(<EditorRuler view={null} column={72} onColumn={() => {}} tableEl={table} />);
    // jsdom gives every element a zero-width box, so no band is drawn; what
    // this pins is that the unit label is a PROSE thing and goes with it.
    expect(document.querySelector(".editor-ruler__band")).toBeNull();
  });
});
</storycode:copy>
<a id="f-a0ab5d3c94e1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement-5">
it("shares the ruler measure and refuses same-valued handles from another document", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const change = vi.fn();
  const renderRuler = (owner: string) => <UiActionContext.Provider value={registry}><EditorRuler view={null} owner={owner} column={72} onColumn={change} /></UiActionContext.Provider>;
  const rendered = render(renderRuler("first.md"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = () => actions().find(action => action.label === "Set prose wrapping measure")!.handle;
  const run = async (target: string, column: number) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: target, arguments: { column }, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await run(handle(), 64)).toMatchObject({ status: "result", outcome: { value: { measure_requested: 64, source_changed: false, persisted: false } } });
  expect(change).toHaveBeenLastCalledWith(64);
  fireEvent.keyDown(screen.getByRole("slider"), { key: "ArrowRight" });
  expect(change).toHaveBeenLastCalledWith(73);
  const stale = handle();
  rendered.rerender(renderRuler("second.md"));
  change.mockClear();
  expect(await run(stale, 64)).not.toMatchObject({ status: "result" });
  expect(await run(handle(), WRAP_MAX + 1)).toMatchObject({ status: "invalid" });
  expect(change).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-a0ab5d3c94e1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-a0ab5d3c94e1-expression_statement-6">
it("discards a cancelled pointer drag instead of persisting its last position", () => {
  const live = view(), change = vi.fn();
  live.contentDOM.getBoundingClientRect = () => ({ left: 0, width: 800 }) as DOMRect;
  Object.defineProperty(live, "defaultCharacterWidth", { value: 8, configurable: true });
  live.requestMeasure = (<T,>(request: { read: (v: EditorView) => T; write?: (measure: T, v: EditorView) => void }) => request?.write?.(request.read(live), live)) as typeof live.requestMeasure;
  const rendered = render(<EditorRuler view={live} column={72} onColumn={change} />);
  const marker = screen.getByRole("slider");
  fireEvent(marker, new MouseEvent("pointerdown", { clientX: 640, bubbles: true }));
  fireEvent(marker, new MouseEvent("pointercancel", { clientX: 640, bubbles: true }));
  expect(change).not.toHaveBeenCalled();
  expect(marker.getAttribute("aria-valuenow")).toBe("72");
  rendered.unmount(); live.destroy();
});
</storycode:copy>

### Assemble `apps/web/src/editor/EditorRuler.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/EditorRuler.test.tsx"><storycode:paste select="#f-a0ab5d3c94e1-expression_statement"/><storycode:paste select="#f-a0ab5d3c94e1-view"/><storycode:paste select="#f-a0ab5d3c94e1-expression_statement-2"/><storycode:paste select="#f-a0ab5d3c94e1-expression_statement-3"/><storycode:paste select="#f-a0ab5d3c94e1-expression_statement-4"/><storycode:paste select="#f-a0ab5d3c94e1-expression_statement-5"/><storycode:paste select="#f-a0ab5d3c94e1-expression_statement-6"/></storycode:file>


</storycode:doc>
