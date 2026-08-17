// A ribbon is a huge, forgiving click target drawn over small, precise ones.
// When both want the same click, the precise one wins: the ribbon can be hit
// anywhere along its length, but the tab-close button, breakpoint gutter, or
// "open here" port underneath can only be hit exactly where it is. So a
// click that lands on a ribbon first asks what is beneath the pointer, and
// yields to anything interactive before acting itself.

/**
 * Controls that must win a click even when a ribbon covers them. Text
 * content is deliberately absent: a click on plain code through a ribbon is
 * the one case where the ribbon is the more plausible intent, and the caret
 * is one un-ribboned pixel away.
 */
export const PRECISE_TARGETS = [
  "button",
  "a[href]",
  "select",
  "input",
  "textarea",
  "summary",
  '[role="button"]',
  "[data-ribbon-port]",
  "[data-shell-tab-kind]",
  ".cm-gutterElement",
].join(", ");

/**
 * The precise control a click at this point would have reached were the
 * ribbon not in the way, from a top-to-bottom element stack (the shape
 * `document.elementsFromPoint` returns). Only the topmost element outside
 * the ribbon layer is consulted — anything deeper is visually behind
 * something opaque and was never clickable.
 */
export function preciseTargetInStack(
  stack: readonly Element[],
  ribbonLayer: Element | null,
): HTMLElement | null {
  for (const el of stack) {
    if (ribbonLayer?.contains(el)) continue;
    const hit = el.closest(PRECISE_TARGETS);
    return hit instanceof HTMLElement ? hit : null;
  }
  return null;
}

/** The DOM-backed wrapper the ribbon layer calls. */
export function preciseTargetBeneath(
  x: number,
  y: number,
  ribbonLayer: Element | null,
): HTMLElement | null {
  return preciseTargetInStack(document.elementsFromPoint(x, y), ribbonLayer);
}
