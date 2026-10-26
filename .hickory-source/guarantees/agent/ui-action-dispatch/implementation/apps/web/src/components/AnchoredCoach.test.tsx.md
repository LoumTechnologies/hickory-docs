<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AnchoredCoach.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1457fcd4dbfb"></a>
## `apps/web/src/components/AnchoredCoach.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-1457fcd4dbfb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1457fcd4dbfb-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AnchoredCoach } from "./AnchoredCoach";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-1457fcd4dbfb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1457fcd4dbfb-expression_statement-2">
it("preserves target focus and dismisses with Escape only inside its nonmodal popup", () => {
  const target = document.createElement("button"); target.id = "coach-target"; target.textContent = "Real control"; document.body.append(target);
  vi.spyOn(target, "getClientRects").mockReturnValue([{ x: 20, y: 20, width: 100, height: 30 }] as unknown as DOMRectList);
  vi.spyOn(target, "getBoundingClientRect").mockReturnValue({ left: 20, right: 120, top: 20, bottom: 50, width: 100, height: 30 } as DOMRect);
  target.scrollIntoView = vi.fn(); target.focus();
  const dismiss = vi.fn();
  try {
    render(<AnchoredCoach target="#coach-target" title="Investigation" onDismiss={dismiss}><p>Use the actual control</p></AnchoredCoach>);
    expect(document.activeElement).toBe(target);
    expect(screen.getByRole("region", { name: "Investigation" }).getAttribute("aria-modal")).toBeNull();
    fireEvent.keyDown(window, { key: "Escape" }); expect(dismiss).not.toHaveBeenCalled();
    const locate = screen.getByRole("button", { name: "Locate target" }); locate.focus(); fireEvent.click(locate);
    expect(target.scrollIntoView).toHaveBeenCalled(); expect(document.activeElement).toBe(target);
    screen.getByRole("button", { name: "Dismiss coach" }).focus(); fireEvent.keyDown(window, { key: "Escape" }); expect(dismiss).toHaveBeenCalledOnce();
  } finally { target.remove(); }
});
</storycode:copy>
<a id="f-1457fcd4dbfb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1457fcd4dbfb-expression_statement-3">
it("keeps recovery available when the normal target pane is absent", () => {
  render(<AnchoredCoach target="#closed-pane" title="Inspect evidence" onDismiss={() => {}}><button>Skip unestablished</button></AnchoredCoach>);
  expect(screen.getByRole("status").textContent).toContain("closed, hidden or unavailable");
  expect(screen.getByRole("button", { name: "Locate target" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Skip unestablished" })).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/components/AnchoredCoach.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AnchoredCoach.test.tsx"><storycode:paste select="#f-1457fcd4dbfb-expression_statement"/><storycode:paste select="#f-1457fcd4dbfb-expression_statement-2"/><storycode:paste select="#f-1457fcd4dbfb-expression_statement-3"/></storycode:file>

</storycode:doc>
