<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MathPanel.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a7d8fefd10d5"></a>
## `apps/web/src/components/MathPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-a7d8fefd10d5-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a7d8fefd10d5-expression_statement">
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { renderMathInto } from "../editor/mathRender";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { MathPanel } from "./MathPanel";
vi.mock("../editor/mathRender", () => ({ renderMathInto: vi.fn() }));
</storycode:copy>
<a id="f-a7d8fefd10d5-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a7d8fefd10d5-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-a7d8fefd10d5-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-a7d8fefd10d5-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("promotes only the latest rendered equation and exposes actual source/fallback status explicitly", async () => {
  const completions = new Map<string, () => void>();
  vi.mocked(renderMathInto).mockImplementation((target, source) => new Promise(resolve => { completions.set(source, () => { target.textContent = `Rendered ${source}`; if (source === "new λ") { target.classList.add("cm-math--unparsed"); target.dataset.tip = "Engine unavailable"; } resolve(); }); }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const content = (source: string) => <UiActionContext.Provider value={registry}><MathPanel source={source} /></UiActionContext.Provider>;
  const view = render(content("old private equation"));
  view.rerender(content("new λ"));
  await act(async () => { completions.get("new λ")!(); });
  expect(screen.getByTestId("math-panel").textContent).toBe("Rendered new λ");
  await act(async () => { completions.get("old private equation")!(); });
  expect(screen.getByTestId("math-panel").textContent).toBe("Rendered new λ");
  const parent = registry.groups().groups[0];
  const action = registry.observe(parent).actions[0];
  expect(JSON.stringify(action)).not.toContain("new λ");
  const reading = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes)))).toMatchObject({ source: "new λ", rendering: { status: "source_fallback", error: "Engine unavailable" } });
});
</storycode:copy>

### Assemble `apps/web/src/components/MathPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MathPanel.test.tsx"><storycode:paste select="#f-a7d8fefd10d5-expression_statement"/><storycode:paste select="#f-a7d8fefd10d5-expression_statement-2"/><storycode:paste select="#f-a7d8fefd10d5-expression_statement-3"/></storycode:file>

</storycode:doc>
