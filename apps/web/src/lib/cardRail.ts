// Where the action rail's icons sit.
//
// Each icon wants to be level with the line its card is about. Two cards on
// nearby lines want the same pixels, so the rail stacks them: an icon never
// rises above its own line, and never overlaps the one before it. The result
// is monotonic, which is what keeps "further down the rail" meaning "further
// down the document" even where cards crowd together.
//
// Pure arithmetic, separate from the component, because the crowding cases
// are exactly the ones that are tedious to produce in a browser and trivial
// to state as numbers.

/** Icon box, in pixels. Kept here so the geometry and the CSS agree. */
export const ICON_SIZE = 20;
/** Smallest gap between two icon boxes. */
export const ICON_GAP = 2;

/**
 * Stack icon tops so none overlaps the previous one.
 *
 * `wanted` must be in document order (the card list is). An icon is placed at
 * its wanted top, or just under its predecessor when that would collide —
 * pushed DOWN rather than up, so an icon is never drawn above the line it
 * belongs to and a reader scanning from a line to the rail never has to look
 * backwards.
 */
export function stackIcons(
  wanted: readonly number[],
  pitch: number = ICON_SIZE + ICON_GAP,
): number[] {
  const tops: number[] = [];
  let floor = Number.NEGATIVE_INFINITY;
  for (const want of wanted) {
    const top = Math.max(want, floor);
    tops.push(top);
    floor = top + pitch;
  }
  return tops;
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
