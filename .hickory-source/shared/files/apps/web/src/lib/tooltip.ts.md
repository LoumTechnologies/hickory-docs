<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/tooltip.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Every Hover Hint Is Drawn In The App Theme](../../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md)

Captured SHA-256: `e468bdcbccc63c620b6040a9919fa267ee98a9e20f5939885432081ccbd27490`.

<book:entry path="apps/web/src/lib/tooltip.ts">
// Where a themed tooltip goes, given what it is pointing at.
//
// The browser's own `title=` tooltip is drawn by the operating system: white
// card, system font, no relation to the three token sets in styles.css. On a
// dark editor that reads as a foreign object, so the app draws its own — and
// the placement, which is the only part with a real failure mode (a tooltip
// off the top of the window, or clipped at the right edge), lives here where
// jsdom-free arithmetic can be tested.

/** The attribute that carries a tooltip's text. Hover any element with it. */
export const TIP_ATTR = "data-tip";

export interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface Placement {
  left: number;
  top: number;
  /** Which side of the anchor the tooltip ended up on. */
  side: "above" | "below";
}

/**
 * Place `tip` against `anchor` inside `viewport`, in viewport coordinates.
 *
 * Above by preference — a tooltip below the pointer covers the next row, and
 * the next row is usually the thing you were about to hover. It flips below
 * only when there is no room above, and is always clamped inside the viewport
 * horizontally, because a tooltip that leaves the window says nothing.
 */
export function placeTip(
  anchor: Box,
  tip: Size,
  viewport: Size,
  gap = 8,
  margin = 6,
): Placement {
  const above = anchor.top - gap - tip.height;
  const side: Placement["side"] = above >= margin ? "above" : "below";
  const top = side === "above" ? above : anchor.top + anchor.height + gap;
  const centred = anchor.left + anchor.width / 2 - tip.width / 2;
  const rightmost = Math.max(margin, viewport.width - tip.width - margin);
  return {
    left: Math.round(Math.min(Math.max(centred, margin), rightmost)),
    // A tooltip taller than the space below it is still better on-screen than
    // hanging off the bottom edge.
    top: Math.round(Math.min(top, Math.max(margin, viewport.height - tip.height - margin))),
    side,
  };
}

/**
 * The nearest ancestor (or self) carrying tooltip text, and that text.
 *
 * Delegated from the document, so what the pointer is actually over is often
 * a `<span>` inside the button that owns the tooltip.
 */
export function tipTargetOf(node: EventTarget | null): { el: HTMLElement; text: string } | null {
  if (!(node instanceof Element)) return null;
  const el = node.closest<HTMLElement>(`[${TIP_ATTR}]`);
  const text = el?.getAttribute(TIP_ATTR)?.trim();
  return el && text ? { el, text } : null;
}
</book:entry>

</book:doc>
