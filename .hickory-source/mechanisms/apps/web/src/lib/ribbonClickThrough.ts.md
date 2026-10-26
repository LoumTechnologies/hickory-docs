<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/ribbonClickThrough.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-246b9cc5576a"></a>
## `apps/web/src/lib/ribbonClickThrough.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-246b9cc5576a-PRECISE_TARGETS"></a>
### `PRECISE_TARGETS`

A ribbon is a huge, forgiving click target drawn over small, precise ones.
When both want the same click, the precise one wins: the ribbon can be hit
anywhere along its length, but the tab-close button, breakpoint gutter, or
"open here" port underneath can only be hit exactly where it is. So a
click that lands on a ribbon first asks what is beneath the pointer, and
yields to anything interactive before acting itself.
Controls that must win a click even when a ribbon covers them. Text
content is deliberately absent: a click on plain code through a ribbon is
the one case where the ribbon is the more plausible intent, and the caret
is one un-ribboned pixel away.

<storycode:copy id="f-246b9cc5576a-PRECISE_TARGETS">
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

</storycode:copy>
<a id="f-246b9cc5576a-preciseTargetInStack"></a>
### `preciseTargetInStack`

The precise control a click at this point would have reached were the
ribbon not in the way, from a top-to-bottom element stack (the shape
`document.elementsFromPoint` returns). Only the topmost element outside
the ribbon layer is consulted — anything deeper is visually behind
something opaque and was never clickable.

<storycode:copy id="f-246b9cc5576a-preciseTargetInStack">
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

</storycode:copy>
<a id="f-246b9cc5576a-preciseTargetBeneath"></a>
### `preciseTargetBeneath`

The DOM-backed wrapper the ribbon layer calls.

<storycode:copy id="f-246b9cc5576a-preciseTargetBeneath">
/** The DOM-backed wrapper the ribbon layer calls. */
export function preciseTargetBeneath(
  x: number,
  y: number,
  ribbonLayer: Element | null,
): HTMLElement | null {
  return preciseTargetInStack(document.elementsFromPoint(x, y), ribbonLayer);
}
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonClickThrough.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonClickThrough.ts"><storycode:paste select="#f-246b9cc5576a-PRECISE_TARGETS"/><storycode:paste select="#f-246b9cc5576a-preciseTargetInStack"/><storycode:paste select="#f-246b9cc5576a-preciseTargetBeneath"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c5e864506093"></a>
## `apps/web/src/lib/ribbonClickThrough.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c5e864506093-build"></a>
### `build`

A ribbon yields its click to any precise control beneath it — the ribbon
can be hit anywhere, the button underneath only exactly where it is.

<storycode:copy id="f-c5e864506093-build">
// A ribbon yields its click to any precise control beneath it — the ribbon
// can be hit anywhere, the button underneath only exactly where it is.
import { describe, expect, it } from "vitest";

import { preciseTargetInStack } from "./ribbonClickThrough";

function build(html: string): HTMLElement {
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.appendChild(host);
  return host;
}

</storycode:copy>
<a id="f-c5e864506093-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c5e864506093-expression_statement">
describe("preciseTargetInStack", () => {
  it("yields to a button under the ribbon layer", () => {
    const host = build(
      `<svg class="ribbon-layer"><g><path /></g></svg>
       <button class="tab-close">×</button>`,
    );
    const layer = host.querySelector("svg")!;
    const path = host.querySelector("path")!;
    const button = host.querySelector("button")!;
    expect(preciseTargetInStack([path, layer, button, host], layer)).toBe(button);
  });

  it("resolves a click landing inside a control to the control itself", () => {
    const host = build(`<button><span class="icon">▸</span></button>`);
    const span = host.querySelector("span")!;
    expect(preciseTargetInStack([span, host], null)).toBe(host.querySelector("button"));
  });

  it("keeps the click when only plain content is beneath", () => {
    // Code text: the caret is one un-ribboned pixel away, the ribbon is the
    // more plausible intent — the ribbon acts.
    const host = build(`<div class="cm-line">let x = 1;</div>`);
    const line = host.querySelector(".cm-line")!;
    expect(preciseTargetInStack([line, host], null)).toBeNull();
  });

  it("only consults the topmost non-ribbon element", () => {
    // The button here is BEHIND an opaque pane: it was never clickable, so
    // the ribbon must not resurrect it.
    const host = build(
      `<div class="pane"><div class="cm-line">text</div></div><button>hidden</button>`,
    );
    const line = host.querySelector(".cm-line")!;
    const button = host.querySelector("button")!;
    expect(preciseTargetInStack([line, host, button], null)).toBeNull();
  });

  it("recognises tabs, ports, and gutter elements as precise", () => {
    const host = build(
      `<div data-shell-tab-kind="generated">tab</div>
       <div data-ribbon-port="generated:a.py">port</div>
       <div class="cm-gutterElement">12</div>`,
    );
    for (const el of Array.from(host.children)) {
      expect(preciseTargetInStack([el], null)).toBe(el);
    }
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/ribbonClickThrough.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/ribbonClickThrough.test.ts"><storycode:paste select="#f-c5e864506093-build"/><storycode:paste select="#f-c5e864506093-expression_statement"/></storycode:file>


</storycode:doc>
