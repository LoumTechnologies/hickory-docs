<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/WelcomePane.controls.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a6400780f2db"></a>
## `apps/web/src/views/WelcomePane.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-a6400780f2db-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a6400780f2db-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { WelcomePane } from "./WelcomePane";
import { WELCOME_KEY } from "../lib/welcomePref";
import { SurfaceInfo } from "../components/SurfaceInfo";
afterEach(() => { cleanup(); localStorage.removeItem(WELCOME_KEY); });
</storycode:copy>
<a id="f-a6400780f2db-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-a6400780f2db-expression_statement-2">
it("shares welcome callbacks and the saved preference without disclosing hidden recent rows", async () => {
  const run = vi.fn(), open = vi.fn();
  const registry = new UiActions({ workspace: "workspace", window: "welcome" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><WelcomePane actions={[{ id:"new", label:"New document", hint:"Create a document", run }]} recent={Array.from({length:9}, (_,i) => ({id:String(i),path:`document-${i}.md`}))} onOpenRecent={open} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group,64).actions);
  let serial = 0;
  const invoke = async (label: string, args = {}) => {
    const action = actions().find(action => action.label === label)!;
    await act(async () => { await registry.act(registry.scope,{kind:"agent",id:"welcome-agent"},{handle:action.handle,arguments:args,request_id:`welcome-${serial++}`}); });
  };
  expect(actions().some(action => action.label.includes("document-8"))).toBe(false);
  await invoke("New document"); expect(run).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button",{name:"New document"})); expect(run).toHaveBeenCalledTimes(2);
  await invoke("Open recent document document-3.md"); expect(open).toHaveBeenCalledWith("3");
  await invoke("Show this page when a folder opens", {show:true});
  expect(localStorage.getItem(WELCOME_KEY)).toBe("1");
  expect(screen.getByLabelText("Show this page when a folder opens")).toHaveProperty("checked",true);
});
</storycode:copy>
<a id="f-a6400780f2db-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-a6400780f2db-expression_statement-3">
it("shares surface disclosure and removes retired controls", async () => {
  const registry = new UiActions({workspace:"workspace",window:"info"},()=>true,async()=>{});
  const view = render(<UiActionContext.Provider value={registry}><SurfaceInfo name="Generated" editable={false} destination="Review source to change this output" /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group,64).actions);
  const action = actions().find(action => action.label === "Set information visibility for Generated")!;
  expect(screen.getByLabelText("About this view")).toHaveProperty("open",false);
  await act(async()=> { await registry.act(registry.scope,{kind:"agent",id:"reader"},{handle:action.handle,arguments:{open:true},request_id:"show"}); });
  expect(screen.getByLabelText("About this view")).toHaveProperty("open",true);
  const details = screen.getByLabelText("About this view");
  (details as HTMLDetailsElement).open=false; fireEvent(details,new Event("toggle"));
  expect(details).toHaveProperty("open",false);
  view.unmount();
  expect(actions()).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/views/WelcomePane.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/WelcomePane.controls.test.tsx"><storycode:paste select="#f-a6400780f2db-expression_statement"/><storycode:paste select="#f-a6400780f2db-expression_statement-2"/><storycode:paste select="#f-a6400780f2db-expression_statement-3"/></storycode:file>

</storycode:doc>
