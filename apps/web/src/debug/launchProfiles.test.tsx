// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import {act,cleanup,fireEvent,render,screen,waitFor} from "@testing-library/react";
import {afterEach,beforeEach,expect,it,vi} from "vitest";
import {request} from "../api/client";
import {showTerminalRequest} from "../lib/revealLine";
import {LaunchProfiles,launchWithProfiles,type LaunchProfile} from "./launchProfiles";
vi.mock("../api/client",()=>({request:vi.fn()}));
vi.mock("../lib/revealLine",()=>({showTerminalRequest:vi.fn()}));
const profile=(id:string,debug=true):LaunchProfile=>({id,name:id,origin:"application",program:"app.py",command:[],args:[],env:{},cwd:".",build:[],default:false,can_debug:debug});
let workspace=0;
const catalog=(profiles:LaunchProfile[],fingerprint="one")=>({profiles,fingerprint,viewed:"library.py",workspace:`/test/${workspace}`,configuration:".hick-launch.json"});
beforeEach(()=>{workspace++;vi.clearAllMocks();});
afterEach(()=>{cleanup();});
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("uses the one discovered app while keeping the viewed library as the debug source",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("greeting")]));
  const start=vi.fn();
  await launchWithProfiles({uri:"hick-project:///a/library.py"},start);
  expect(start).toHaveBeenCalledWith({id:"greeting",fingerprint:"one",name:"greeting"});
  expect(request).toHaveBeenCalledWith("POST","/api/launch-profiles",{uri:"hick-project:///a/library.py"});
});
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("shows an overlay choice for ambiguous targets and remembers the profile without its old project version",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("a"),profile("b")]));
  render(<LaunchProfiles/>);
  const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick-project:///old/library.py"},start);});
  const button=await screen.findByRole("button",{name:"Debug b"});
  expect(screen.getByRole("dialog").closest(".cm-content")).toBeNull();
  expect(start).not.toHaveBeenCalled();
  fireEvent.click(button);await pending;
  expect(start).toHaveBeenLastCalledWith({id:"b",fingerprint:"one",name:"b"});
  vi.mocked(request).mockResolvedValue(catalog([profile("a"),profile("b")],"new-project"));
  await launchWithProfiles({uri:"hick-project:///new/library.py"},start);
  expect(start).toHaveBeenLastCalledWith({id:"b",fingerprint:"new-project",name:"b"});
});
it("runs command profiles through the terminal route and does not pretend they support Debug",async()=>{
  vi.mocked(request).mockImplementation(async(_method,path)=>path==="/api/launch-profiles"?catalog([profile("tests",false)]):{id:"term1",title:"Run tests"});
  render(<LaunchProfiles/>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///test_app.py"},start,{choose:true});});
  const button=await screen.findByRole("button",{name:"Run tests"});
  expect((screen.getByRole("button",{name:"Debug tests"}) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(button);await pending;
  expect(start).not.toHaveBeenCalled();
  expect(request).toHaveBeenCalledWith("POST","/api/launch-profiles/run",{uri:"hick:///test_app.py",profile:{id:"tests",fingerprint:"one"}});
  expect(showTerminalRequest).toHaveBeenCalledWith("term1","Run tests");
});
it("cancels on Escape and preserves the normal source editor",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("a"),profile("b")]));
  render(<><div data-testid="source">source text</div><LaunchProfiles/></>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start);});
  const dialog=await screen.findByRole("dialog");
  fireEvent.keyDown(dialog,{key:"Escape"});await pending;
  expect(start).not.toHaveBeenCalled();expect(screen.getByTestId("source").textContent).toBe("source text");
});
it("refuses code changed while choosing and offers a useful recovery message",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("a")]));
  render(<LaunchProfiles/>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start,{isCurrent:()=>false});});
  await waitFor(()=>expect(screen.getByRole("alert").textContent).toContain("displayed code changed"));
  fireEvent.click(screen.getByRole("button",{name:"Close launch profiles"}));await pending;
  expect(start).not.toHaveBeenCalled();
});
it("explains discovery/configuration failures without starting anything",async()=>{
  vi.mocked(request).mockRejectedValue(new Error("Fix .hick-launch.json: unknown field"));
  render(<LaunchProfiles/>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start);});
  await waitFor(()=>expect(screen.getByRole("alert").textContent).toContain("Fix .hick-launch.json"));
  fireEvent.click(screen.getByRole("button",{name:"Close launch profiles"}));await pending;
  expect(start).not.toHaveBeenCalled();
});
it("shares captured profile selection and retires it when the picker is replaced", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.mocked(request).mockResolvedValue(catalog([profile("tests", false), profile("app")]));
  render(<UiActionContext.Provider value={registry}><LaunchProfiles /></UiActionContext.Provider>);
  const start = vi.fn(); let first!: Promise<void>, second!: Promise<void>;
  await act(async () => { first = launchWithProfiles({ uri: "hick:///first.py" }, start, { choose: true }); });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const handle = (label: string) => actions().find(action => action.label === label)!.handle;
  const run = async (target: string) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: target, arguments: {}, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await run(handle("Debug tests"))).toMatchObject({ status: "disabled" });
  expect(start).not.toHaveBeenCalled();
  const stale = handle("Debug app");
  await act(async () => { second = launchWithProfiles({ uri: "hick:///second.py" }, start, { choose: true }); });
  await first;
  expect(await run(stale)).not.toMatchObject({ status: "result" });
  expect(start).not.toHaveBeenCalled();
  expect(await run(handle("Debug app"))).toMatchObject({ status: "result", outcome: { value: { selection_requested: true, launch_completed: false } } });
  await second;
  expect(start).toHaveBeenCalledExactlyOnceWith({ id: "app", fingerprint: "one", name: "app" });
  expect(actions().some(action => action.label === "Debug app")).toBe(false);
});
