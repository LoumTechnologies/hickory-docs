<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/tableFit.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e3496da2400b"></a>
## `apps/web/src/lib/tableFit.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-e3496da2400b-intrinsic"></a>
### `intrinsic`

<storycode:copy id="f-e3496da2400b-intrinsic">
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

</storycode:copy>
<a id="f-e3496da2400b-fittedRows"></a>
### `fittedRows`

Measure wrapped text at the existing column width, even when shrinking.

<storycode:copy id="f-e3496da2400b-fittedRows">
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
</storycode:copy>

### Assemble `apps/web/src/lib/tableFit.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tableFit.ts"><storycode:paste select="#f-e3496da2400b-intrinsic"/><storycode:paste select="#f-e3496da2400b-fittedRows"/></storycode:file>

</storycode:doc>
