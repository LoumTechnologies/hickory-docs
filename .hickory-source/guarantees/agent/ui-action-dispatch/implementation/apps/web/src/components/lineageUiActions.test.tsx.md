<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/lineageUiActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b841ae76cd46"></a>
## `apps/web/src/components/lineageUiActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-b841ae76cd46-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b841ae76cd46-expression_statement">
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { LineageSettings } from "./LineageSettings";
import { loadRibbonStyle } from "../lib/ribbonStyle";
afterEach(() => { cleanup(); localStorage.clear(); });
</storycode:copy>
<a id="f-b841ae76cd46-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b841ae76cd46-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("mounts preference commands only after the real feature-settings disclosure", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><LineageSettings /></UiActionContext.Provider>);
  expect(registry.groups().groups.some(group => group.startsWith("lineage-settings:"))).toBe(false);
  let n = 0;
  const invoke = async (parent: string, label: string, arguments_: Record<string, string | boolean>) => {
    const action = registry.observe(parent).actions.find(action => action.label === label)!;
    await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) }); });
  };
  await invoke("feature-settings", "Set Lineage settings visibility", { open: true });
  const parent = registry.groups().groups.find(group => group.startsWith("lineage-settings:"))!;
  await invoke(parent, "Set lineage drawing style", { style: "bands" });
  expect(loadRibbonStyle()).toBe("bands"); expect((screen.getByLabelText("Lineage style") as HTMLSelectElement).value).toBe("bands");
  await invoke("feature-settings", "Set Lineage settings visibility", { open: false });
  expect(registry.observe(parent).actions).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/components/lineageUiActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/lineageUiActions.test.tsx"><storycode:paste select="#f-b841ae76cd46-expression_statement"/><storycode:paste select="#f-b841ae76cd46-expression_statement-2"/></storycode:file>

</storycode:doc>
