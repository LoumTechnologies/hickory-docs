// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { WelcomePane } from "./WelcomePane";
import { WELCOME_KEY } from "../lib/welcomePref";
import { SurfaceInfo } from "../components/SurfaceInfo";
afterEach(() => { cleanup(); localStorage.removeItem(WELCOME_KEY); });
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
