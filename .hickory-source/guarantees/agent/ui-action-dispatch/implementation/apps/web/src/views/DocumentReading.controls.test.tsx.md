<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/DocumentReading.controls.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c7ae8a3155cc"></a>
## `apps/web/src/views/DocumentReading.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-c7ae8a3155cc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c7ae8a3155cc-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { forgetReading, publishReading } from "../lib/readingViews";
vi.mock("../components/ComparisonEditor", () => ({ ComparisonEditor: () => <p>Source comparison</p>, CodeEditor: () => null }));
</storycode:copy>
<a id="f-c7ae8a3155cc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c7ae8a3155cc-expression_statement-2">
vi.mock("./CommitReading", () => ({ CommitReading: () => null }));
</storycode:copy>
<a id="f-c7ae8a3155cc-import_statement"></a>
### `import_statement`

<storycode:copy id="f-c7ae8a3155cc-import_statement">
import { DocumentReading } from "./DocumentReading";
</storycode:copy>
<a id="f-c7ae8a3155cc-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c7ae8a3155cc-expression_statement-3">
afterEach(() => { cleanup(); forgetReading("parity-review"); });
</storycode:copy>
<a id="f-c7ae8a3155cc-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-c7ae8a3155cc-expression_statement-4">
it("returns an addressable human review request and preserves actual decision failure", async () => {
  const decide = vi.fn().mockResolvedValue({ status:"unavailable", reason:"Source changed before review acceptance" });
  publishReading("parity-review", { title:"Proposed change", path:"doc.md", before:"before", after:"after", status:"pending", decide });
  const registry = new UiActions({workspace:"work",window:"review"},()=>true,async()=>{});
  render(<UiActionContext.Provider value={registry}><DocumentReading id="parity-review" /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group=>registry.observe(group,64).actions);
  const action = actions().find(action=>action.label === "Accept change")!;
  let result: unknown;
  await act(async()=> { result = await registry.act(registry.scope,{kind:"agent",id:"proposer"},{handle:action.handle,arguments:{},request_id:"request-review"}); });
  expect(result).toMatchObject({outcome:{status:"requires_input",request:"edit-review:parity-review"}});
  expect(decide).not.toHaveBeenCalled();
  const current = actions().find(action=>action.label === "Accept change")!;
  await act(async()=> { result=await registry.act(registry.scope,{kind:"human",id:"reviewer"},{handle:current.handle,arguments:{},request_id:"human-review"}); });
  expect(result).toMatchObject({outcome:{status:"unavailable",reason:"Source changed before review acceptance"}});
  expect(decide).toHaveBeenCalledWith(true,undefined);
  fireEvent.click(screen.getByRole("button",{name:"Reject change"}));
  await waitFor(()=>expect(decide).toHaveBeenCalledWith(false,undefined));
  act(()=> publishReading("parity-review", {title:"Proposed change",path:"doc.md",before:"before",after:"after",status:"accepted",decide}));
  expect(actions().some(action=>action.label === "Accept change")).toBe(false);
});
</storycode:copy>

### Assemble `apps/web/src/views/DocumentReading.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/DocumentReading.controls.test.tsx"><storycode:paste select="#f-c7ae8a3155cc-expression_statement"/><storycode:paste select="#f-c7ae8a3155cc-expression_statement-2"/><storycode:paste select="#f-c7ae8a3155cc-import_statement"/><storycode:paste select="#f-c7ae8a3155cc-expression_statement-3"/><storycode:paste select="#f-c7ae8a3155cc-expression_statement-4"/></storycode:file>

</storycode:doc>
