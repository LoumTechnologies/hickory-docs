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
/** Smallest gap between two icon boxes. */
export const ICON_GAP = 2;

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
