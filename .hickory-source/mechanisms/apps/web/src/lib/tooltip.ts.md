<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/tooltip.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d5b928a2f7eb"></a>
## `apps/web/src/lib/tooltip.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Every Hover Hint Is Drawn In The App Theme](../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md).

<a id="f-d5b928a2f7eb-TIP_ATTR"></a>
### `TIP_ATTR`

Where a themed tooltip goes, given what it is pointing at.
The browser's own `title=` tooltip is drawn by the operating system: white
card, system font, no relation to the three token sets in styles.css. On a
dark editor that reads as a foreign object, so the app draws its own — and
the placement, which is the only part with a real failure mode (a tooltip
off the top of the window, or clipped at the right edge), lives here where
jsdom-free arithmetic can be tested.
The attribute that carries a tooltip's text. Hover any element with it.

<storycode:copy id="f-d5b928a2f7eb-TIP_ATTR">
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

</storycode:copy>
<a id="f-d5b928a2f7eb-Box"></a>
### `Box`

<storycode:copy id="f-d5b928a2f7eb-Box">
export interface Box {
  left: number;
  top: number;
  width: number;
  height: number;
}

</storycode:copy>
<a id="f-d5b928a2f7eb-Size"></a>
### `Size`

<storycode:copy id="f-d5b928a2f7eb-Size">
export interface Size {
  width: number;
  height: number;
}

</storycode:copy>
<a id="f-d5b928a2f7eb-Placement"></a>
### `Placement`

<storycode:copy id="f-d5b928a2f7eb-Placement">
export interface Placement {
  left: number;
  top: number;
  /** Which side of the anchor the tooltip ended up on. */
  side: "above" | "below";
}

</storycode:copy>
<a id="f-d5b928a2f7eb-placeTip"></a>
### `placeTip`

Place `tip` against `anchor` inside `viewport`, in viewport coordinates.
Above by preference — a tooltip below the pointer covers the next row, and
the next row is usually the thing you were about to hover. It flips below
only when there is no room above, and is always clamped inside the viewport
horizontally, because a tooltip that leaves the window says nothing.

<storycode:copy id="f-d5b928a2f7eb-placeTip">
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

</storycode:copy>
<a id="f-d5b928a2f7eb-tipTargetOf"></a>
### `tipTargetOf`

The nearest ancestor (or self) carrying tooltip text, and that text.
Delegated from the document, so what the pointer is actually over is often
a `<span>` inside the button that owns the tooltip.

<storycode:copy id="f-d5b928a2f7eb-tipTargetOf">
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
</storycode:copy>

### Assemble `apps/web/src/lib/tooltip.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/tooltip.ts"><storycode:paste select="#f-d5b928a2f7eb-TIP_ATTR"/><storycode:paste select="#f-d5b928a2f7eb-Box"/><storycode:paste select="#f-d5b928a2f7eb-Size"/><storycode:paste select="#f-d5b928a2f7eb-Placement"/><storycode:paste select="#f-d5b928a2f7eb-placeTip"/><storycode:paste select="#f-d5b928a2f7eb-tipTargetOf"/></storycode:file>

</storycode:doc>
