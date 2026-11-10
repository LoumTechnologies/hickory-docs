<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/cardRail.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-996aefac1e60"></a>
## `apps/web/src/lib/cardRail.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md).

<a id="f-996aefac1e60-ICON_SIZE"></a>
### `ICON_SIZE`

Where the action rail's icons sit.
Each icon wants to be level with the line its card is about. Two cards on
nearby lines want the same pixels, so the rail stacks them: an icon never
rises above its own line, and never overlaps the one before it. The result
is monotonic, which is what keeps "further down the rail" meaning "further
down the document" even where cards crowd together.
Pure arithmetic, separate from the component, because the crowding cases
are exactly the ones that are tedious to produce in a browser and trivial
to state as numbers.
Icon box, in pixels. Kept here so the geometry and the CSS agree.

<storycode:copy id="f-996aefac1e60-ICON_SIZE">
// Where the action rail's icons sit.
//
// Each group starts level with its source line. It may stack into available
// space below that line, but cannot displace a later populated line. Reserve
// the last available slot for overflow and show excess actions in a grid.
//
// Pure arithmetic, separate from the component, because the crowding cases
// are exactly the ones that are tedious to produce in a browser and trivial
// to state as numbers.

/** Icon box, in pixels. Kept here so the geometry and the CSS agree. */
export const ICON_SIZE = 20;
</storycode:copy>
<a id="f-996aefac1e60-ICON_GAP"></a>
### `ICON_GAP`

<storycode:copy id="f-996aefac1e60-ICON_GAP">
/** Smallest gap between two icon boxes. */
export const ICON_GAP = 2;

</storycode:copy>
<a id="f-996aefac1e60-stackIcons"></a>
### Preserve source rows while grouping icons

Measure groups in source order and reserve room for the next group before placing icons. This is the same slot rule used by CodeMirror’s left and right gutters. A hidden icon receives no rail position; its overflow trigger retains the group’s anchor.

Stack icon tops so none overlaps the previous one.
`wanted` must be in document order (the card list is). An icon is placed at
its wanted top, or just under its predecessor when that would collide —
pushed DOWN rather than up, so an icon is never drawn above the line it
belongs to and a reader scanning from a line to the rail never has to look
backwards.

<storycode:copy id="f-996aefac1e60-stackIcons">
/** Reserve overflow before the next populated line; never move that line. */
export function gutterSlots(count: number, available: number, size = ICON_SIZE, columns = 1) {
  const slots = Math.max(1, Math.floor((available + .5) / size)) * Math.max(1, columns);
  return count <= slots ? { visible: count, overflow: 0 } : { visible: slots - 1, overflow: count - slots + 1 };
}

export function groupRailIcons(wanted: readonly number[], bottom: number, pitch = ICON_SIZE + ICON_GAP) {
  const rows: { top: number; indices: number[] }[] = [];
  wanted.forEach((top, index) => {
    const previous = rows[rows.length - 1];
    if (previous && Math.abs(previous.top - top) < .5) previous.indices.push(index);
    else rows.push({ top, indices: [index] });
  });
  const tops: (number | null)[] = wanted.map(() => null);
  const overflow: { top: number; indices: number[] }[] = [];
  rows.forEach((row, index) => {
    const room = (rows[index + 1]?.top ?? bottom) - row.top;
    const slots = gutterSlots(row.indices.length, room, pitch);
    row.indices.slice(0, slots.visible).forEach((icon, i) => tops[icon] = row.top + i * pitch);
    if (slots.overflow) overflow.push({ top: row.top + slots.visible * pitch, indices: row.indices.slice(slots.visible) });
  });
  return { tops, overflow };
}
</storycode:copy>
<a id="f-996aefac1e60-iconVisible"></a>
### `iconVisible`

Whether an icon at `top` is worth drawing, given the scroller's visible
band. One icon-height of slack either side so an icon scrolling into view
is already placed rather than appearing mid-motion.

<storycode:copy id="f-996aefac1e60-iconVisible">

/**
 * Whether an icon at `top` is worth drawing, given the scroller's visible
 * band. One icon-height of slack either side so an icon scrolling into view
 * is already placed rather than appearing mid-motion.
 */
export function iconVisible(
  top: number,
  band: { top: number; bottom: number },
  size: number = ICON_SIZE,
): boolean {
  return top + size >= band.top - size && top <= band.bottom + size;
}

</storycode:copy>
<a id="f-996aefac1e60-popoverTop"></a>
### `popoverTop`

Where a popover opened from an icon should sit vertically.
It opens level with its icon, then slides up only as far as it must to fit
— never above the top of the editor, and never past the bottom. A popover
that hangs off the viewport is a popover with its buttons missing.

<storycode:copy id="f-996aefac1e60-popoverTop">
/**
 * Where a popover opened from an icon should sit vertically.
 *
 * It opens level with its icon, then slides up only as far as it must to fit
 * — never above the top of the editor, and never past the bottom. A popover
 * that hangs off the viewport is a popover with its buttons missing.
 */
export function popoverTop(
  iconTop: number,
  popoverHeight: number,
  box: { top: number; height: number },
  margin = 8,
): number {
  const highest = box.top + margin;
  const lowest = box.top + box.height - popoverHeight - margin;
  // A popover taller than the box pins to the top and scrolls inside itself.
  if (lowest <= highest) return highest;
  return Math.max(highest, Math.min(iconTop, lowest));
}
</storycode:copy>

### Assemble `apps/web/src/lib/cardRail.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/cardRail.ts"><storycode:paste select="#f-996aefac1e60-ICON_SIZE"/><storycode:paste select="#f-996aefac1e60-ICON_GAP"/><storycode:paste select="#f-996aefac1e60-stackIcons"/><storycode:paste select="#f-996aefac1e60-iconVisible"/><storycode:paste select="#f-996aefac1e60-popoverTop"/></storycode:file>

</storycode:doc>
