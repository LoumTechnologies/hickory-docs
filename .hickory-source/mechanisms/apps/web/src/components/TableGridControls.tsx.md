<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TableGridControls.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5603c9496242"></a>
## `apps/web/src/components/TableGridControls.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-5603c9496242-DRAG_THRESHOLD"></a>
### `DRAG_THRESHOLD`

<storycode:copy id="f-5603c9496242-DRAG_THRESHOLD">
import { useRef } from "react";

const DRAG_THRESHOLD = 3;
</storycode:copy>
<a id="f-5603c9496242-MIN_GRID_HEIGHT"></a>
### `MIN_GRID_HEIGHT`

<storycode:copy id="f-5603c9496242-MIN_GRID_HEIGHT">
const MIN_GRID_HEIGHT = 64;

</storycode:copy>
<a id="f-5603c9496242-SelectAll"></a>
### `SelectAll`

The box above the row numbers: click it for the whole table, as every
spreadsheet's corner does.

<storycode:copy id="f-5603c9496242-SelectAll">
/** The box above the row numbers: click it for the whole table, as every
 * spreadsheet's corner does. */
export function SelectAll({
  onSelect,
  lane = false,
}: {
  onSelect: () => void;
  lane?: boolean;
}) {
  return (
    <th
      className={`table-panel__corner${lane ? " table-panel__corner--lane" : ""}`}
      scope="col"
      // The lane's corner is the same box a second time, at the other end of
      // the same row. It keeps the click, because a target that looks like
      // one should behave like one — and loses the name, because two things
      // called "Select the whole table" is a screen reader reading the same
      // control twice.
      aria-label={lane ? undefined : "Select the whole table"}
      aria-hidden={lane ? true : undefined}
      data-tip="Select the whole table"
      onMouseDown={(event) => {
        event.preventDefault();
        onSelect();
      }}
      onClick={onSelect}
    />
  );
}

</storycode:copy>
<a id="f-5603c9496242-Resizer"></a>
### `Resizer`

The drag handle on a grid line.
Every line in the table is one — the right edge of a column and the bottom
edge of a row, in the header, in the row numbers, and in every cell. A
spreadsheet only puts them in its headers, which is fine when the headers
are always on screen and irritating when the line you want to move is the
one your pointer is already next to.
Two things make that affordable rather than annoying:
- **A press that does not move is a click.** The strip is five pixels
along the edge of a cell people also want to select, so it hands the
press back rather than swallowing it. Without this the bottom of every
row would be unselectable.
- **The measure is a number this component owns**, not one it reads off
the screen — the same reason the widths are declared (see the note at
the top). A handle in a cell has no idea how wide its column is; it is
told, and it reports a new number back.
Sizes live in the component and not in the document: how wide a column
looks is not something the CSV says, and writing it there would put
presentation into a file whose whole justification is that a script can
read it.

<storycode:copy id="f-5603c9496242-Resizer">
/**
 * The drag handle on a grid line.
 *
 * Every line in the table is one — the right edge of a column and the bottom
 * edge of a row, in the header, in the row numbers, and in every cell. A
 * spreadsheet only puts them in its headers, which is fine when the headers
 * are always on screen and irritating when the line you want to move is the
 * one your pointer is already next to.
 *
 * Two things make that affordable rather than annoying:
 *
 *   - **A press that does not move is a click.** The strip is five pixels
 *     along the edge of a cell people also want to select, so it hands the
 *     press back rather than swallowing it. Without this the bottom of every
 *     row would be unselectable.
 *   - **The measure is a number this component owns**, not one it reads off
 *     the screen — the same reason the widths are declared (see the note at
 *     the top). A handle in a cell has no idea how wide its column is; it is
 *     told, and it reports a new number back.
 *
 * Sizes live in the component and not in the document: how wide a column
 * looks is not something the CSV says, and writing it there would put
 * presentation into a file whose whole justification is that a script can
 * read it.
 */
export function Resizer({
  axis,
  size,
  least,
  onResize,
  onTap,
  onFit,
}: {
  axis: "column" | "row";
  /** What it is now, in pixels — the number the drag starts from. */
  size: number | (() => number);
  least: number;
  onResize: (size: number) => void;
  /** A press that never became a drag. Absent means such a press does
   * nothing, which is what a read-only table wants. */
  onTap?: () => void;
  /** A double-click: the smallest measure that still fits the content. The
   * gesture every spreadsheet has on this exact target. */
  onFit?: () => void;
}) {
  const along = (event: { clientX: number; clientY: number }) =>
    axis === "column" ? event.clientX : event.clientY;

  const onMouseDown = (event: React.MouseEvent) => {
    // Without these the press also selects the cell underneath, so a drag
    // that started on the line would move the selection as a side effect.
    event.preventDefault();
    event.stopPropagation();
    const from = along(event);
    const initialSize = typeof size === "function" ? size() : size;
    let dragged = false;
    const move = (e: MouseEvent) => {
      const to = along(e);
      if (!dragged && Math.abs(to - from) < DRAG_THRESHOLD) return;
      dragged = true;
      onResize(Math.max(least, Math.round(initialSize + (to - from))));
    };
    const up = (e: MouseEvent) => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
      // `detail` is the click count, so the second press of a double-click is
      // 2 and does not tap. The FIRST one still does — nothing can know a
      // second is coming — which is why the fit gives the selection back.
      if (!dragged && e.detail <= 1) onTap?.();
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };

  return (
    <span
      className={`table-panel__resizer table-panel__resizer--${axis}`}
      role="presentation"
      data-testid={`${axis}-resizer`}
      onMouseDown={onMouseDown}
      onClick={(event) => event.stopPropagation()}
      onDoubleClick={(event) => {
        // Stopped so the second click does not also reach the cell behind
        // this strip, where a double-click means "open this for editing".
        event.stopPropagation();
        onFit?.();
      }}
    />
  );
}

</storycode:copy>
<a id="f-5603c9496242-HeightResizer"></a>
### `HeightResizer`

The drag bar along the bottom of the grid.
Measured against the scroller it sits under rather than against a remembered
number, so the first drag starts from whatever height the table currently
has instead of jumping to a default.

<storycode:copy id="f-5603c9496242-HeightResizer">
/**
 * The drag bar along the bottom of the grid.
 *
 * Measured against the scroller it sits under rather than against a remembered
 * number, so the first drag starts from whatever height the table currently
 * has instead of jumping to a default.
 */
export function HeightResizer({
  height,
  onResize,
}: {
  height?: number;
  onResize: (height: number) => void;
}) {
  const bar = useRef<HTMLDivElement | null>(null);

  // Mouse events rather than pointer events, matching `Resizer` beside
  // it: two drag handles on one component that behave differently is a thing
  // somebody eventually trips over.
  const onMouseDown = (event: React.MouseEvent) => {
    event.preventDefault();
    const scroller = bar.current?.previousElementSibling as HTMLElement | null;
    const startHeight =
      height ?? scroller?.getBoundingClientRect().height ?? MIN_GRID_HEIGHT;
    const startY = event.clientY;
    const move = (e: MouseEvent) =>
      onResize(
        Math.max(
          MIN_GRID_HEIGHT,
          Math.round(startHeight + (e.clientY - startY)),
        ),
      );
    const up = () => {
      window.removeEventListener("mousemove", move);
      window.removeEventListener("mouseup", up);
    };
    window.addEventListener("mousemove", move);
    window.addEventListener("mouseup", up);
  };

  return (
    <div
      ref={bar}
      className="table-panel__height"
      role="separator"
      aria-label="Table height"
      aria-orientation="horizontal"
      data-testid="height-resizer"
      data-tip="Drag to see more rows at a time, or fewer"
      onMouseDown={onMouseDown}
    />
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/TableGridControls.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableGridControls.tsx"><storycode:paste select="#f-5603c9496242-DRAG_THRESHOLD"/><storycode:paste select="#f-5603c9496242-MIN_GRID_HEIGHT"/><storycode:paste select="#f-5603c9496242-SelectAll"/><storycode:paste select="#f-5603c9496242-Resizer"/><storycode:paste select="#f-5603c9496242-HeightResizer"/></storycode:file>

</storycode:doc>
