<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/ribbonClickThrough.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8100f88c461ba2d9608943a028225fced49ee78c1e96dfc53c1bba02a8e55a88`.

<book:entry path="apps/web/src/lib/ribbonClickThrough.ts">
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
</book:entry>

</book:doc>
