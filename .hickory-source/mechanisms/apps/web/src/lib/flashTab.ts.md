<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/flashTab.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-67742d1b612b"></a>
## `apps/web/src/lib/flashTab.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-67742d1b612b-FLASH_TAB_EVENT"></a>
### `FLASH_TAB_EVENT`

"It's already open — HERE." Opening a file that is already open activates
its tab, but activation alone is invisible when the tab was already active
or sits in a busy strip: the flash is what moves the eye to it. One event,
answered by the shell, works for every place a tab renders — the top
strip, the side tab tree, and a collapsed pane's icon strip — because they
all carry the same data attributes the ribbon overlay measures by.

<storycode:copy id="f-67742d1b612b-FLASH_TAB_EVENT">
// "It's already open — HERE." Opening a file that is already open activates
// its tab, but activation alone is invisible when the tab was already active
// or sits in a busy strip: the flash is what moves the eye to it. One event,
// answered by the shell, works for every place a tab renders — the top
// strip, the side tab tree, and a collapsed pane's icon strip — because they
// all carry the same data attributes the ribbon overlay measures by.

export const FLASH_TAB_EVENT = "hickory:flash-tab";
</storycode:copy>
<a id="f-67742d1b612b-FLASH_CLASS"></a>
### `FLASH_CLASS`

<storycode:copy id="f-67742d1b612b-FLASH_CLASS">
export const FLASH_CLASS = "shell-tab--flash";

</storycode:copy>
<a id="f-67742d1b612b-flashTab"></a>
### `flashTab`

Ask the shell to pulse the tab showing `target`.

<storycode:copy id="f-67742d1b612b-flashTab">
/** Ask the shell to pulse the tab showing `target`. */
export function flashTab(kind: "document" | "generated" | "file" | "tree", target: string): void {
  window.dispatchEvent(new CustomEvent(FLASH_TAB_EVENT, { detail: { kind, target } }));
}

</storycode:copy>
<a id="f-67742d1b612b-attr"></a>
### `attr`

A value quoted into an attribute selector. Paths may hold anything.

<storycode:copy id="f-67742d1b612b-attr">
/** A value quoted into an attribute selector. Paths may hold anything. */
function attr(value: string): string {
  return `"${value.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

</storycode:copy>
<a id="f-67742d1b612b-attachFlashListener"></a>
### `attachFlashListener`

Listen for flash requests and pulse the matching tab under `root`.
The lookup runs a frame later on purpose: the request is usually
dispatched in the same tick as the activation that re-renders (and may
re-parent) the tab element, and a class added to the old element would
vanish with it.

<storycode:copy id="f-67742d1b612b-attachFlashListener">
/**
 * Listen for flash requests and pulse the matching tab under `root`.
 * The lookup runs a frame later on purpose: the request is usually
 * dispatched in the same tick as the activation that re-renders (and may
 * re-parent) the tab element, and a class added to the old element would
 * vanish with it.
 */
export function attachFlashListener(root: HTMLElement): () => void {
  const frames = new Set<number>();
  const onFlash = (e: Event) => {
    const detail = (e as CustomEvent<{ kind?: string; target?: string }>).detail;
    if (!detail?.kind || typeof detail.target !== "string") return;
    // `let` + assign, not `const`: a synchronous rAF (tests) runs the
    // callback during the call, and a `const` frame would still be in its
    // temporal dead zone there.
    let frame = 0;
    frame = requestAnimationFrame(() => {
      frames.delete(frame);
      const el = root.querySelector(
        `[data-shell-tab-kind=${attr(detail.kind!)}][data-shell-tab-target=${attr(detail.target!)}]`,
      );
      if (!(el instanceof HTMLElement)) return;
      // Restarting an in-flight flash needs the class re-applied from
      // scratch, or the animation never re-fires.
      el.classList.remove(FLASH_CLASS);
      void el.offsetWidth;
      el.classList.add(FLASH_CLASS);
      el.addEventListener("animationend", () => el.classList.remove(FLASH_CLASS), {
        once: true,
      });
    });
    frames.add(frame);
  };
  window.addEventListener(FLASH_TAB_EVENT, onFlash);
  return () => {
    window.removeEventListener(FLASH_TAB_EVENT, onFlash);
    for (const frame of frames) cancelAnimationFrame(frame);
  };
}
</storycode:copy>

### Assemble `apps/web/src/lib/flashTab.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/flashTab.ts"><storycode:paste select="#f-67742d1b612b-FLASH_TAB_EVENT"/><storycode:paste select="#f-67742d1b612b-FLASH_CLASS"/><storycode:paste select="#f-67742d1b612b-flashTab"/><storycode:paste select="#f-67742d1b612b-attr"/><storycode:paste select="#f-67742d1b612b-attachFlashListener"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-6a5410256dea"></a>
## `apps/web/src/lib/flashTab.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6a5410256dea-tab"></a>
### `tab`

Opening an already-open file flashes its tab — the activation alone is
invisible when the tab was already front-and-centre.

<storycode:copy id="f-6a5410256dea-tab">
// Opening an already-open file flashes its tab — the activation alone is
// invisible when the tab was already front-and-centre.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { attachFlashListener, FLASH_CLASS, flashTab } from "./flashTab";

function tab(root: HTMLElement, kind: string, target: string): HTMLElement {
  const el = document.createElement("button");
  el.setAttribute("data-shell-tab-kind", kind);
  el.setAttribute("data-shell-tab-target", target);
  root.appendChild(el);
  return el;
}

</storycode:copy>
<a id="f-6a5410256dea-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-6a5410256dea-expression_statement">
describe("flashTab", () => {
  let root: HTMLElement;
  let detach: () => void;

  beforeEach(() => {
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
      cb(0);
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", () => undefined);
    root = document.createElement("div");
    document.body.appendChild(root);
    detach = attachFlashListener(root);
  });

  afterEach(() => {
    detach();
    root.remove();
    vi.unstubAllGlobals();
  });

  it("pulses the matching tab and clears the class when the animation ends", () => {
    const el = tab(root, "document", "notes.md");
    flashTab("document", "notes.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
    el.dispatchEvent(new Event("animationend"));
    expect(el.classList.contains(FLASH_CLASS)).toBe(false);
  });

  it("matches kind AND target — a generated file's path never flashes a doc tab", () => {
    const doc = tab(root, "document", "orders.py");
    const gen = tab(root, "generated", "orders.py");
    flashTab("generated", "orders.py");
    expect(doc.classList.contains(FLASH_CLASS)).toBe(false);
    expect(gen.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("survives targets with quotes and backslashes", () => {
    const el = tab(root, "generated", 'we"ird\\name.py');
    flashTab("generated", 'we"ird\\name.py');
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("re-fires an in-flight flash from scratch", () => {
    const el = tab(root, "document", "a.md");
    flashTab("document", "a.md");
    flashTab("document", "a.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("does nothing after detach", () => {
    const el = tab(root, "document", "a.md");
    detach();
    flashTab("document", "a.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(false);
    detach = () => undefined;
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/flashTab.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/flashTab.test.ts"><storycode:paste select="#f-6a5410256dea-tab"/><storycode:paste select="#f-6a5410256dea-expression_statement"/></storycode:file>


</storycode:doc>
