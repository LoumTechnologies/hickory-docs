<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tableFit.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d3546ba989096af3747567d67ef3416a0ad6dababa3d77b638beb4e55a092288`.

<book:entry path="apps/web/src/lib/tableFit.ts">
import { rowsIn, type Selection } from "./tableSelection";

export function intrinsic(
  cells: Iterable<HTMLElement>,
  axis: "width" | "height",
): number {
  let most = 0;
  for (const cell of cells) {
    const held = cell.style[axis];
    cell.style[axis] = "max-content";
    most = Math.max(most, cell.getBoundingClientRect()[axis]);
    // Restored immediately, so a fit never leaves a cell laid out differently
    // from the ones beside it.
    cell.style[axis] = held;
  }
  return Math.ceil(most);
}

/** Measure wrapped text at the existing column width, even when shrinking. */
export function fittedRows(
  grid: HTMLTableElement | null,
  row: number,
  selection: Selection | null = null,
): Record<string, number> {
  const heights: Record<string, number> = {};
  for (const index of selection ? rowsIn(selection) : [row]) {
    const cells = grid?.querySelectorAll<HTMLElement>(`[data-row="${index}"]`);
    if (!cells?.length) continue;
    for (const cell of cells) {
      const held = cell.style.whiteSpace;
      cell.style.whiteSpace = "pre-wrap";
      heights[index] = Math.max(heights[index] ?? 16, intrinsic([cell], "height") + 1);
      cell.style.whiteSpace = held;
    }
  }
  return heights;
}
</book:entry>

</book:doc>
