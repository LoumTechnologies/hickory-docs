<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/SamplePicker.controls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Sample Shows Generated Lines Without Storing Them](../../../../../contract.md)

Captured SHA-256: `ef00469eff180a11bd93a8a51c6ea43b97123c46be4931fb59b74933cdd6fde5`.

<book:entry path="apps/web/src/shell/SamplePicker.controls.test.tsx">
// Guarantee: docs/guarantees/authoring/a-sample-shows-generated-lines-without-storing-them.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { SamplePicker } from "./SamplePicker";
import { api } from "../api/client";
import * as reviews from "../lib/reviews";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("shares caption and source-bound dispatch, refuses a retired hash capture and leaves new selection status untouched", async () => {
  const create = vi.spyOn(api,"createSample").mockResolvedValue({doc_id:"doc",doc_path:"doc.md",path:"out.txt",from:1,to:2});
  const hash = vi.spyOn(reviews,"sha256").mockResolvedValue("a".repeat(64));
  const registry = new UiActions({workspace:"work",window:"sample"},()=>true,async()=>{});
  const surface = (contents:string) => <UiActionContext.Provider value={registry}><SamplePicker path="out.txt" range={{from:1,to:2}} contents={contents} /></UiActionContext.Provider>;
  const view=render(surface("selected bytes"));
  const actions=()=>registry.groups().groups.flatMap(group=>registry.observe(group,64).actions);
  const caption=actions().find(action=>action.label === "Set caption for this sample")!;
  await act(async()=>{await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:caption.handle,arguments:{caption:"Illustration"},request_id:"caption"});});
  expect(screen.getByLabelText("Caption for this sample")).toHaveProperty("value","Illustration");
  const add=actions().find(action=>action.label === "Show selected generated lines in document")!;
  await act(async()=>{await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:add.handle,arguments:{},request_id:"sample"});});
  expect(create).toHaveBeenCalledWith("out.txt",1,2,"Illustration","a".repeat(64));
  expect(screen.getByRole("status").textContent).toContain("doc.md");
  let finishHash!: (value:string)=>void;
  hash.mockImplementationOnce(()=>new Promise(resolve=>{finishHash=resolve;}));
  fireEvent.click(screen.getByRole("button",{name:"Show in document"}));
  view.rerender(surface("replacement bytes"));
  await act(async()=>{finishHash("b".repeat(64));});
  expect(create).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("status")).toBeNull();
  expect(await registry.act(registry.scope,{kind:"agent",id:"sampler"},{handle:add.handle,arguments:{},request_id:"retired"})).toMatchObject({status:"stale"});
});
</book:entry>

</book:doc>
