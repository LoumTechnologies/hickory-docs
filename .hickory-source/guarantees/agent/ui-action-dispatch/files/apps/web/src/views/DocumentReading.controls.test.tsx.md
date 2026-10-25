<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/DocumentReading.controls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `62104ea33c0f4cb7b541166748a19955df4df756b5bb8fb0d91f558fbf2e32f3`.

<book:entry path="apps/web/src/views/DocumentReading.controls.test.tsx">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { forgetReading, publishReading } from "../lib/readingViews";
vi.mock("../components/ComparisonEditor", () => ({ ComparisonEditor: () => <p>Source comparison</p>, CodeEditor: () => null }));
vi.mock("./CommitReading", () => ({ CommitReading: () => null }));
import { DocumentReading } from "./DocumentReading";
afterEach(() => { cleanup(); forgetReading("parity-review"); });
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
</book:entry>

</book:doc>
