<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/launchProfiles.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1a7a9a03aa8d"></a>
## `apps/web/src/debug/launchProfiles.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [Launch profiles choose targets within the viewed project](../../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).

<a id="f-1a7a9a03aa8d-LaunchSelection"></a>
### `LaunchSelection`

<storycode:copy id="f-1a7a9a03aa8d-LaunchSelection">
import { useEffect, useMemo, useRef, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { request } from "../api/client";
import { showTerminalRequest } from "../lib/revealLine";

export interface LaunchSelection { id: string; fingerprint: string; name?: string }
</storycode:copy>
<a id="f-1a7a9a03aa8d-LaunchQuery"></a>
### `LaunchQuery`

<storycode:copy id="f-1a7a9a03aa8d-LaunchQuery">
export interface LaunchQuery { uri: string; project?: string; program?: string; expected_source?: string; expected_document?: string }
</storycode:copy>
<a id="f-1a7a9a03aa8d-LaunchProfile"></a>
### `LaunchProfile`

<storycode:copy id="f-1a7a9a03aa8d-LaunchProfile">
export interface LaunchProfile {
  id: string; name: string; origin: string; program: string | null; command: string[];
  args: string[]; env: Record<string,string>; cwd: string; build: string[]; default: boolean; can_debug: boolean;
}
</storycode:copy>
<a id="f-1a7a9a03aa8d-Catalog"></a>
### `Catalog`

<storycode:copy id="f-1a7a9a03aa8d-Catalog">
interface Catalog { profiles: LaunchProfile[]; fingerprint: string; viewed: string; workspace: string; configuration: string }
</storycode:copy>
<a id="f-1a7a9a03aa8d-Choice"></a>
### `Choice`

<storycode:copy id="f-1a7a9a03aa8d-Choice">
type Choice = { profile: LaunchProfile; mode: "run" | "debug" };
</storycode:copy>
<a id="f-1a7a9a03aa8d-Picker"></a>
### `Picker`

<storycode:copy id="f-1a7a9a03aa8d-Picker">
interface Picker { notice?: string; contextLabel?: string; catalog?: Catalog; error?: string; settle: (value: Choice | null) => void; anchor: DOMRect | null }
</storycode:copy>
<a id="f-1a7a9a03aa8d-picker"></a>
### `picker`

<storycode:copy id="f-1a7a9a03aa8d-picker">
let picker: Picker | null = null;
</storycode:copy>
<a id="f-1a7a9a03aa8d-listeners"></a>
### `listeners`

<storycode:copy id="f-1a7a9a03aa8d-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-1a7a9a03aa8d-emit"></a>
### `emit`

<storycode:copy id="f-1a7a9a03aa8d-emit">
const emit = () => { for (const listener of listeners) listener(); };
</storycode:copy>
<a id="f-1a7a9a03aa8d-subscribe"></a>
### `subscribe`

<storycode:copy id="f-1a7a9a03aa8d-subscribe">
const subscribe = (listener: () => void) => { listeners.add(listener); return () => {listeners.delete(listener);}; };
</storycode:copy>
<a id="f-1a7a9a03aa8d-remembered"></a>
### `remembered`

<storycode:copy id="f-1a7a9a03aa8d-remembered">
const remembered = new Map<string,string>();
</storycode:copy>
<a id="f-1a7a9a03aa8d-pending"></a>
### `pending`

<storycode:copy id="f-1a7a9a03aa8d-pending">
const pending = new Set<string>();
</storycode:copy>
<a id="f-1a7a9a03aa8d-choose"></a>
### `choose`

<storycode:copy id="f-1a7a9a03aa8d-choose">
function choose(catalog?: Catalog, error?: string, contextLabel?: string, notice?:string): Promise<Choice | null> {
  picker?.settle(null);
  const active = document.activeElement;
  const anchor = active instanceof HTMLElement ? active.getBoundingClientRect() : null;
  return new Promise(resolve => {
    const current: Picker = {catalog,error,anchor,contextLabel,notice,settle:value=>{if(picker===current){picker=null;emit();}resolve(value);}};
    picker=current;emit();
  });
}

</storycode:copy>
<a id="f-1a7a9a03aa8d-launchWithProfiles"></a>
### `launchWithProfiles`

The only frontend Run/Debug profile resolution path. Remember the target, never its version.

<storycode:copy id="f-1a7a9a03aa8d-launchWithProfiles">
/** The only frontend Run/Debug profile resolution path. Remember the target, never its version. */
export async function launchWithProfiles(query: LaunchQuery, onDebug: (selection: LaunchSelection) => void,
  options: {choose?: boolean; isCurrent?: () => boolean; contextLabel?: string} = {}): Promise<void> {
  const key = JSON.stringify(query);
  if(pending.has(key))return;
  pending.add(key);
  try {
    const catalog = await request<Catalog>("POST","/api/launch-profiles",query);
    const memory = `${catalog.workspace}:${query.uri.replace(/hick-project:\/\/\/[^/]+\//,"captured/")}:${catalog.viewed}`;
    const candidates = catalog.profiles.filter(p=>p.can_debug);
    const last=remembered.get(memory), known=catalog.profiles.find(p=>p.id===last);
    const unavailable=!!last && !known;
    const preferred = known ?? catalog.profiles.find(p=>p.default);
    const automatic = preferred?.can_debug ? preferred : !preferred && candidates.length===1 ? candidates[0] : null;
    const choice = !options.choose && !unavailable && automatic ? {profile:automatic,mode:"debug" as const} : await choose(catalog,undefined,options.contextLabel,unavailable?"The previous profile is unavailable in this project version. Choose another target.":undefined);
    if(!choice)return;
    if(options.isCurrent && !options.isCurrent())throw new Error("The displayed code changed. Open Launch profiles again after saving or reloading it.");
    remembered.set(memory,choice.profile.id);
    const selection = {id:choice.profile.id,fingerprint:catalog.fingerprint};
    if(choice.mode==="debug")onDebug({...selection,name:choice.profile.name});
    else {
      const session=await request<{id:string;title:string}>("POST","/api/launch-profiles/run",{...query,profile:selection});
      showTerminalRequest(session.id,session.title);
    }
  } catch(e) { await choose(undefined,e instanceof Error?e.message:String(e)); }
  finally {pending.delete(key);}
}

</storycode:copy>
<a id="f-1a7a9a03aa8d-LaunchProfiles"></a>
### `LaunchProfiles`

Workspace overlay: it never inserts editor rows or creates a second debugger.

<storycode:copy id="f-1a7a9a03aa8d-LaunchProfiles">
/** Workspace overlay: it never inserts editor rows or creates a second debugger. */
export function LaunchProfiles() {
  const state = useSyncExternalStore(subscribe,()=>picker,()=>null);
  const panel=useRef<HTMLDivElement>(null);
  useEffect(()=>{ if(state)panel.current?.querySelector<HTMLButtonElement>("button")?.focus(); },[state]);
  useEffect(()=>()=>{picker?.settle(null);},[]);
  const capture = useMemo(() => crypto.randomUUID(), [state]);
  const revision = useUiRevision(() => capture);
  useUiReading("launch profile picker", state ? { notice: state.notice ?? null, error: state.error ?? null,
    viewed: state.catalog?.viewed ?? null, context: state.contextLabel ?? "Latest compatible project", configuration: state.catalog?.configuration ?? null,
    profiles: state.catalog?.profiles.map(profile => ({ id: profile.id, name: profile.name, origin: profile.origin, default: profile.default,
      displayed_command: profile.program ?? profile.command.join(" "), can_debug: profile.can_debug })) ?? [],
    scope: "Displayed picker metadata only. Selecting a profile requests launch; it does not attest process startup or debugging success." } : { open: false });
  useUiCommands(state ? [
    { key: "launch-picker:close", command: { label: "Close launch profiles", parent: "launch-profiles", subject: "launch-profiles:visibility", parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: true }), execute: async () => { state.settle(null); return { status: "completed", value: { dismissal_requested: true, launched: false } }; } } },
    ...(state.catalog?.profiles.flatMap(profile => (["run", "debug"] as const).map(mode => ({ key: `launch-picker:${profile.id}:${mode}`, command: {
      label: `${mode === "run" ? "Run" : "Debug"} ${profile.name}`, parent: "launch-profiles", subject: "launch-profiles:selection", parameters: {}, prerequisites: ["Launch resolves against the captured project and profile fingerprint"],
      read: () => ({ revision: revision(), enabled: mode === "run" || profile.can_debug, ...(!profile.can_debug && mode === "debug" ? { reason: "This command profile supports Run; choose an application profile to debug." } : {}) }),
      execute: async () => { state.settle({ profile, mode }); return { status: "completed" as const, value: { selection_requested: true, mode, profile: profile.id, launch_completed: false } }; },
    } }))) ?? []),
  ] : []);
  if(!state)return null;
  const left=Math.max(8,Math.min((state.anchor?.right ?? window.innerWidth)-400,window.innerWidth-416));
  const top=Math.max(8,Math.min(state.anchor?.top ?? 80,window.innerHeight-300));
  return createPortal(<div className="launch-profile-backdrop" onMouseDown={e=>{if(e.target===e.currentTarget)state.settle(null);}}>
    <div ref={panel} className="launch-profile-picker" role="dialog" aria-label="Launch profiles" style={{left,top}}
      onKeyDown={e=>{if(e.key==="Escape"){e.stopPropagation();state.settle(null);}}}>
      <header><strong>Launch profiles</strong><button type="button" aria-label="Close launch profiles" onClick={()=>state.settle(null)}>×</button></header>
      {state.notice && <p role="status">{state.notice}</p>}
      {state.error && <p role="alert">{state.error}</p>}
      {state.catalog && <>
        <p className="muted">For {state.catalog.viewed} · {state.contextLabel ?? "Latest compatible project"}</p>
        {!state.catalog.profiles.length && <p>No launch targets found. Add a profile in {state.catalog.configuration}.</p>}
        {state.catalog.profiles.map(profile=><section key={profile.id} className="launch-profile-option">
          <strong>{profile.name}</strong><span className="muted">{profile.origin}{profile.default?" · default":""}</span>
          <code>{profile.program ?? profile.command.join(" ")}</code>
          <span><button type="button" onClick={()=>state.settle({profile,mode:"run"})}>Run {profile.name}</button>
          <button type="button" disabled={!profile.can_debug} data-tip={!profile.can_debug?"This command profile supports Run; choose an application profile to debug.":undefined}
            onClick={()=>state.settle({profile,mode:"debug"})}>Debug {profile.name}</button></span>
        </section>)}
        <p className="muted">Manual profiles: {state.catalog.configuration} in this project. A declared document must construct this file in its project.</p>
      </>}
    </div>
  </div>,document.body);
}
</storycode:copy>

### Assemble `apps/web/src/debug/launchProfiles.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/launchProfiles.tsx"><storycode:paste select="#f-1a7a9a03aa8d-LaunchSelection"/><storycode:paste select="#f-1a7a9a03aa8d-LaunchQuery"/><storycode:paste select="#f-1a7a9a03aa8d-LaunchProfile"/><storycode:paste select="#f-1a7a9a03aa8d-Catalog"/><storycode:paste select="#f-1a7a9a03aa8d-Choice"/><storycode:paste select="#f-1a7a9a03aa8d-Picker"/><storycode:paste select="#f-1a7a9a03aa8d-picker"/><storycode:paste select="#f-1a7a9a03aa8d-listeners"/><storycode:paste select="#f-1a7a9a03aa8d-emit"/><storycode:paste select="#f-1a7a9a03aa8d-subscribe"/><storycode:paste select="#f-1a7a9a03aa8d-remembered"/><storycode:paste select="#f-1a7a9a03aa8d-pending"/><storycode:paste select="#f-1a7a9a03aa8d-choose"/><storycode:paste select="#f-1a7a9a03aa8d-launchWithProfiles"/><storycode:paste select="#f-1a7a9a03aa8d-LaunchProfiles"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-af7d3cd605b8"></a>
## `apps/web/src/debug/launchProfiles.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md), [Launch profiles choose targets within the viewed project](../../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).

<a id="f-af7d3cd605b8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import {act,cleanup,fireEvent,render,screen,waitFor} from "@testing-library/react";
import {afterEach,beforeEach,expect,it,vi} from "vitest";
import {request} from "../api/client";
import {showTerminalRequest} from "../lib/revealLine";
import {LaunchProfiles,launchWithProfiles,type LaunchProfile} from "./launchProfiles";
vi.mock("../api/client",()=>({request:vi.fn()}));
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-2">
vi.mock("../lib/revealLine",()=>({showTerminalRequest:vi.fn()}));
</storycode:copy>
<a id="f-af7d3cd605b8-profile"></a>
### `profile`

<storycode:copy id="f-af7d3cd605b8-profile">
const profile=(id:string,debug=true):LaunchProfile=>({id,name:id,origin:"application",program:"app.py",command:[],args:[],env:{},cwd:".",build:[],default:false,can_debug:debug});
</storycode:copy>
<a id="f-af7d3cd605b8-workspace"></a>
### `workspace`

<storycode:copy id="f-af7d3cd605b8-workspace">
let workspace=0;
</storycode:copy>
<a id="f-af7d3cd605b8-catalog"></a>
### `catalog`

<storycode:copy id="f-af7d3cd605b8-catalog">
const catalog=(profiles:LaunchProfile[],fingerprint="one")=>({profiles,fingerprint,viewed:"library.py",workspace:`/test/${workspace}`,configuration:".hick-launch.json"});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-3">
beforeEach(()=>{workspace++;vi.clearAllMocks();});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-4">
afterEach(()=>{cleanup();});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-5">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("uses the one discovered app while keeping the viewed library as the debug source",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("greeting")]));
  const start=vi.fn();
  await launchWithProfiles({uri:"hick-project:///a/library.py"},start);
  expect(start).toHaveBeenCalledWith({id:"greeting",fingerprint:"one",name:"greeting"});
  expect(request).toHaveBeenCalledWith("POST","/api/launch-profiles",{uri:"hick-project:///a/library.py"});
});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-6">
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
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-7">
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
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-8">
it("cancels on Escape and preserves the normal source editor",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("a"),profile("b")]));
  render(<><div data-testid="source">source text</div><LaunchProfiles/></>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start);});
  const dialog=await screen.findByRole("dialog");
  fireEvent.keyDown(dialog,{key:"Escape"});await pending;
  expect(start).not.toHaveBeenCalled();expect(screen.getByTestId("source").textContent).toBe("source text");
});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-9">
it("refuses code changed while choosing and offers a useful recovery message",async()=>{
  vi.mocked(request).mockResolvedValue(catalog([profile("a")]));
  render(<LaunchProfiles/>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start,{isCurrent:()=>false});});
  await waitFor(()=>expect(screen.getByRole("alert").textContent).toContain("displayed code changed"));
  fireEvent.click(screen.getByRole("button",{name:"Close launch profiles"}));await pending;
  expect(start).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-10">
it("explains discovery/configuration failures without starting anything",async()=>{
  vi.mocked(request).mockRejectedValue(new Error("Fix .hick-launch.json: unknown field"));
  render(<LaunchProfiles/>);const start=vi.fn();let pending!:Promise<void>;
  act(()=>{pending=launchWithProfiles({uri:"hick:///app.py"},start);});
  await waitFor(()=>expect(screen.getByRole("alert").textContent).toContain("Fix .hick-launch.json"));
  fireEvent.click(screen.getByRole("button",{name:"Close launch profiles"}));await pending;
  expect(start).not.toHaveBeenCalled();
});
</storycode:copy>
<a id="f-af7d3cd605b8-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-af7d3cd605b8-expression_statement-11">
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
</storycode:copy>

### Assemble `apps/web/src/debug/launchProfiles.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/launchProfiles.test.tsx"><storycode:paste select="#f-af7d3cd605b8-expression_statement"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-2"/><storycode:paste select="#f-af7d3cd605b8-profile"/><storycode:paste select="#f-af7d3cd605b8-workspace"/><storycode:paste select="#f-af7d3cd605b8-catalog"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-3"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-4"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-5"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-6"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-7"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-8"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-9"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-10"/><storycode:paste select="#f-af7d3cd605b8-expression_statement-11"/></storycode:file>


</storycode:doc>
