<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Launch profiles choose targets within the viewed project

For: Authors debugging generated code while navigating its document source.

Given a Debug gesture beside displayed source, Hickory resolves the latest
compatible atomic project before discovering launch profiles. The viewed file
remains the breakpoint coordinate space; a selected application's entry point
may be another file. Definition-only library modules are not automatically
applications. Discovery reads project inputs without running them.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-3b3668793103"></a>
## `apps/web/src/debug/plainDebugHosts.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Choose a launch without replacing the editor being read](../debug-controls-preserve-editor-space/contract.md), [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-3b3668793103-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-3b3668793103-expression_statement">
import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { publishPausedElsewhere, resetPausedElsewhere } from "../lib/pausedElsewhere";
import { IDLE_SESSION } from "./useDebugger";
import { isWorkspaceSource, resetPlainDebugHosts, useFollowingDebugSession, usePublishDebugSession } from "./plainDebugHosts";

afterEach(()=>{cleanup();resetPlainDebugHosts();resetPausedElsewhere();});
</storycode:copy>
<a id="f-3b3668793103-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-3b3668793103-expression_statement-2">
it("keeps execution controls in a callee while stepping and running, including document-owned sessions",()=>{
  const step=vi.fn(),stop=vi.fn(),jumpTo=vi.fn(),runTo=vi.fn(),setSourceBreakpoints=vi.fn();
  const debug={...IDLE_SESSION,status:"paused" as const,step,stop,jumpTo,runTo,setSourceBreakpoints,pausedLine:null,
    frames:[{id:1,name:"helper",source:"helpers.py",source_line:2,in_document:false,line:null}]};
  const callee={...IDLE_SESSION,breakpoints:[{line:2,state:"bound" as const}]};
  const hook=renderHook(({session})=>{
    usePublishDebugSession("project.md",session);
    usePublishDebugSession("helpers.py",callee);
    return useFollowingDebugSession("helpers.py");
  },{initialProps:{session:debug as typeof IDLE_SESSION}});
  act(()=>publishPausedElsewhere("project.md",{path:"helpers.py",line:2}));
  expect(hook.result.current?.status).toBe("paused");
  expect(hook.result.current?.pausedLine).toBe(2);
  hook.result.current?.step("in");expect(step).toHaveBeenCalledWith("in");
  expect(setSourceBreakpoints).toHaveBeenCalledWith("hick:///helpers.py",callee.breakpoints);
  hook.result.current?.toggleBreakpoint(4);
  expect(setSourceBreakpoints).toHaveBeenLastCalledWith("hick:///helpers.py",[{line:2,state:"bound"},{line:4,state:"pending"}]);
  hook.result.current?.jumpTo(2);hook.result.current?.runTo(2);
  expect(jumpTo).not.toHaveBeenCalled();expect(runTo).not.toHaveBeenCalled();
  act(()=>publishPausedElsewhere("project.md",null));
  hook.rerender({session:{...debug,status:"running"}});
  expect(hook.result.current?.status).toBe("running");
  hook.result.current?.stop();expect(stop).toHaveBeenCalledOnce();
  hook.rerender({session:{...debug,status:"finished"}});
  expect(hook.result.current?.status).toBe("idle");
});
</storycode:copy>
<a id="f-3b3668793103-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-3b3668793103-expression_statement-3">
it("never opens a captured URI as a workspace-relative filename",()=>{
  expect(isWorkspaceSource("helpers.py")).toBe(true);
  expect(isWorkspaceSource("hick-project:///capture/helpers.py")).toBe(false);
  expect(isWorkspaceSource("/tmp/helpers.py")).toBe(false);
});

</storycode:copy>
<a id="f-3b3668793103-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-3b3668793103-expression_statement-4">
it("includes cleared file sets when collecting saved breakpoints for another launch",()=>{
  const start=vi.fn();
  const root={...IDLE_SESSION,start,sourceBreakpoints:{"hick:///helpers.py":[{line:2,state:"bound" as const}]}};
  const helper={...IDLE_SESSION,breakpoints:[]};
  const hook=renderHook(()=>{
    usePublishDebugSession("app.py",root);usePublishDebugSession("helpers.py",helper);
    return useFollowingDebugSession("app.py");
  });
  act(()=>hook.result.current?.start());
  expect(start).toHaveBeenCalledWith(undefined,{"hick:///helpers.py":[]},undefined,undefined);
});

</storycode:copy>
<a id="f-3b3668793103-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-3b3668793103-expression_statement-5">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
it("carries standing library breakpoints into a different compatible capture and preserves clearing",async()=>{
  const {acquireCapturedContext,capturedSourceKey}=await import("../lib/capturedContexts");
  const {rememberBreakpointPreferences}=await import("./breakpointPreferences");
  // Registration is exercised separately by capturedContexts.test; here use
  // the API mock to supply two native identities for different complete trees.
  const client=await import("../api/client");
  let count=0;
  const spy=vi.spyOn(client,"request").mockImplementation(async method=>method==="POST"?{id:`prefs${++count}`,uri:`hick-project:///prefs${count}/`,workspace:"/preferences-test"}:undefined);
  const entries=(value:number)=>({"app.py":{bytes:[value]},"library.py":{bytes:[100,101,102]}});
  const first=await acquireCapturedContext(entries(1),"project"),second=await acquireCapturedContext(entries(2),"project");
  const key=capturedSourceKey(`${first.uri}library.py`);
  const start=vi.fn();
  const session={...IDLE_SESSION,start};
  const hook=renderHook(()=>{usePublishDebugSession(`${second.uri}app.py`,session);return useFollowingDebugSession(`${second.uri}app.py`);});
  try {
    act(()=>rememberBreakpointPreferences(key,[{line:1,state:"pending",condition:"x > 0"}]));
    act(()=>hook.result.current?.start(undefined,{},undefined,{id:"app",fingerprint:"f"}));
    expect(start).toHaveBeenLastCalledWith(undefined,{[`${second.uri}library.py`]:[{line:1,state:"pending",condition:"x > 0",hit_condition:undefined,log_message:undefined}]},undefined,{id:"app",fingerprint:"f"});
    act(()=>rememberBreakpointPreferences(key,[]));
    act(()=>hook.result.current?.start());
    expect(start).toHaveBeenLastCalledWith(undefined,{[`${second.uri}library.py`]:[]},undefined,undefined);
  } finally {hook.unmount();first.release();second.release();spy.mockRestore();}
});
</storycode:copy>

### Assemble `apps/web/src/debug/plainDebugHosts.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/plainDebugHosts.test.tsx"><storycode:paste select="#f-3b3668793103-expression_statement"/><storycode:paste select="#f-3b3668793103-expression_statement-2"/><storycode:paste select="#f-3b3668793103-expression_statement-3"/><storycode:paste select="#f-3b3668793103-expression_statement-4"/><storycode:paste select="#f-3b3668793103-expression_statement-5"/></storycode:file>

<a id="output-238f66c701b0"></a>
## `apps/web/src/debug/plainDebugHosts.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Has The Same Debugger A Document Has](../a-plain-file-has-the-same-debugger/contract.md), [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-238f66c701b0-isWorkspaceSource"></a>
### `isWorkspaceSource`

<storycode:copy id="f-238f66c701b0-isWorkspaceSource">
import { capturedFiles, capturedSourceKey } from "../lib/capturedContexts";
import { breakpointPreferences, rememberBreakpointPreferences, subscribeBreakpointPreferences } from "./breakpointPreferences";
import { LaunchProfiles } from "./launchProfiles";
// Where a plain file's debug session lives: above its pane.
//
// Only the active tab of a pane is rendered, so a plain file's pane
// unmounts the moment another tab is fronted — and a session held in that
// pane's own state would die with it. Found in a real browser: stepping
// from `app.py` into `helpers.py` opened the callee's tab, which unmounted
// the caller's pane, which stopped the session it had just stepped. A
// document does not have this problem because its session lives in the
// workspace's registry; this is the same move for plain files.
//
// A HOST is a component with no DOM that owns one `useWorkspaceDebugger`
// per path and publishes the session it holds into a store the pane reads.
// Hosts are mounted by the workspace (`<PlainDebugHosts />`) for every path
// a pane has asked for, and are never unmounted while the workspace lives:
// each is one hook, and a session that outlives its tab is the point.
//
// The host also publishes "paused elsewhere" (lib/pausedElsewhere.ts) —
// the fact belongs to the session, not to whichever pane happens to be on
// screen.

import { ProjectDebugSurface } from "./projectDebugSurface";
import { useEffect, useMemo, useRef, useSyncExternalStore } from "react";
import { clearPreferredDebugSurface, revealPreferredDebugLocation } from "./preferredDebugLocation";
import { isDebuggable } from "./languages";
import { languageFromPath } from "../editor/languages";

import { openLocation } from "../lib/revealLine";
import { openCapturedCode } from "../lib/capturedCode";
import { retainCapturedContext } from "../lib/capturedContexts";
import { publishPausedElsewhere, usePausedElsewhere } from "../lib/pausedElsewhere";
import type { DebugSession } from "./useDebugger";
import { useWorkspaceDebugger } from "./useDebugger";

/** Whether a frame's source is a file in the folder the app opened. */
export function isWorkspaceSource(source: string | null | undefined): source is string {
  return !!source && !/^[A-Za-z][A-Za-z0-9+.-]*:/.test(source) && !/^([A-Za-z]:)?[\\/]/.test(source);
}

</storycode:copy>
<a id="f-238f66c701b0-wanted"></a>
### `wanted`

The paths that want a host, and the session each host currently holds.

<storycode:copy id="f-238f66c701b0-wanted">
// The paths that want a host, and the session each host currently holds.
let wanted: readonly string[] = [];
</storycode:copy>
<a id="f-238f66c701b0-sessions"></a>
### `sessions`

<storycode:copy id="f-238f66c701b0-sessions">
const sessions = new Map<string, DebugSession>();
</storycode:copy>
<a id="f-238f66c701b0-listeners"></a>
### `listeners`

<storycode:copy id="f-238f66c701b0-listeners">
const listeners = new Set<() => void>();

</storycode:copy>
<a id="f-238f66c701b0-emit"></a>
### `emit`

<storycode:copy id="f-238f66c701b0-emit">
function emit() {
  for (const listener of listeners) listener();
}

</storycode:copy>
<a id="f-238f66c701b0-subscribe"></a>
### `subscribe`

<storycode:copy id="f-238f66c701b0-subscribe">
function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

</storycode:copy>
<a id="f-238f66c701b0-ensurePlainDebugHost"></a>
### `ensurePlainDebugHost`

Ask for a host for `path`. Idempotent; hosts are never removed.

<storycode:copy id="f-238f66c701b0-ensurePlainDebugHost">
/** Ask for a host for `path`. Idempotent; hosts are never removed. */
export function ensurePlainDebugHost(path: string): void {
  if (wanted.includes(path)) return;
  wanted = [...wanted, path];
  emit();
}

</storycode:copy>
<a id="f-238f66c701b0-wantedPaths"></a>
### `wantedPaths`

<storycode:copy id="f-238f66c701b0-wantedPaths">
function wantedPaths(): readonly string[] {
  return wanted;
}

</storycode:copy>
<a id="f-238f66c701b0-usePlainDebugSession"></a>
### `usePlainDebugSession`

The session for `path`, or null until its host has mounted.

<storycode:copy id="f-238f66c701b0-usePlainDebugSession">
/** The session for `path`, or null until its host has mounted. */
export function usePlainDebugSession(path: string): DebugSession | null {
  useEffect(() => { if (path) ensurePlainDebugHost(path); }, [path]);
  return useSyncExternalStore(
    subscribe,
    () => sessions.get(path) ?? null,
    () => null,
  );
}

</storycode:copy>
<a id="f-238f66c701b0-resetPlainDebugHosts"></a>
### `resetPlainDebugHosts`

<storycode:copy id="f-238f66c701b0-resetPlainDebugHosts">
/** Test seam. */
export function resetPlainDebugHosts(): void {
  wanted = [];
  sessions.clear();
  emit();
}

</storycode:copy>
<a id="f-238f66c701b0-Host"></a>
### `Host`

<storycode:copy id="f-238f66c701b0-Host">
function Host({ path }: { path: string }) {
  const debug = useWorkspaceDebugger(path);
  useEffect(() => {
    sessions.set(path, debug);
    emit();
  }, [path, debug]);

  const active = ["starting", "running", "paused"].includes(debug.status);
  useEffect(() => {
    if (active) return retainCapturedContext(path);
  }, [path, active]);

  // Where this session is paused when that is another file of the folder:
  // say so, and open that file, the way stepping into a callee in another
  // file opens it in any IDE. Cleared the moment the program moves on or
  // the session ends; only this owner can clear what it published.
  useEffect(() => {
    const top = debug.frames[0];
    if (["idle", "finished", "failed"].includes(debug.status)) clearPreferredDebugSurface(path);
    if (debug.status === "paused" && top && revealPreferredDebugLocation(path, top)) {
      publishPausedElsewhere(path, top.source && typeof top.source_line === "number" ? {path:top.source,line:top.source_line} : null);
      return;
    }
    if (
      debug.status === "paused" &&
      top &&
      !top.in_document &&
      (top.source?.startsWith("hick-project:///") || isWorkspaceSource(top.source)) &&
      typeof top.source_line === "number"
    ) {
      publishPausedElsewhere(path, { path: top.source!, line: top.source_line });
      if (top.source!.startsWith("hick-project:///")) {
        void openCapturedCode({uri:top.source!,range:{start:{line:top.source_line,character:0},end:{line:top.source_line,character:0}}}, path).catch(() => {});
      } else openLocation(top.source!, top.source_line + 1);
    } else {
      publishPausedElsewhere(path, null);
    }
  }, [debug.status, debug.frames, path]);
  return <ProjectDebugSurface owner={path} debug={debug}/>;
}

</storycode:copy>
<a id="f-238f66c701b0-PlainDebugHosts"></a>
### `PlainDebugHosts`

Mount once, in the workspace: one host per path a pane has asked for.

<storycode:copy id="f-238f66c701b0-PlainDebugHosts">
/** Mount once, in the workspace: one host per path a pane has asked for. */
export function PlainDebugHosts() {
  const paths = useSyncExternalStore(subscribe, wantedPaths, wantedPaths);
  return (
    <>
      <LaunchProfiles />
      {paths.map((path) => (
        <Host key={path} path={path} />
      ))}
    </>
  );
}

</storycode:copy>
<a id="f-238f66c701b0-usePublishDebugSession"></a>
### `usePublishDebugSession`

Let document-owned sessions participate in the same cross-file controls.

<storycode:copy id="f-238f66c701b0-usePublishDebugSession">
/** Let document-owned sessions participate in the same cross-file controls. */
export function usePublishDebugSession(owner: string, debug: DebugSession) {
  useEffect(() => { if (owner) { sessions.set(owner, debug); emit(); } }, [owner, debug]);
  useEffect(() => () => { if (owner) { sessions.delete(owner); emit(); } }, [owner]);
}

</storycode:copy>
<a id="f-238f66c701b0-useFollowingDebugSession"></a>
### `useFollowingDebugSession`

Borrow execution controls from the session that opened this callee.

<storycode:copy id="f-238f66c701b0-useFollowingDebugSession">
/** Borrow execution controls from the session that opened this callee. */
export function useFollowingDebugSession(path: string, explicitOwner?: string): DebugSession | null {
  const own = usePlainDebugSession(path);
  const key=capturedSourceKey(path);
  const saved=useSyncExternalStore(subscribeBreakpointPreferences,()=>breakpointPreferences(key),()=>undefined);
  const ownRef=useRef(own);ownRef.current=own;
  const ownActive=!!own && ["starting","paused","running"].includes(own.status);
  useEffect(()=>{if(saved && ownRef.current && !ownActive)ownRef.current.replaceBreakpoints(saved);},[saved,key,!!own,ownActive]);
  const elsewhere = usePausedElsewhere();
  const followed = useRef({path, owner:explicitOwner});
  if (followed.current.path !== path) followed.current = {path, owner:explicitOwner};
  if (explicitOwner) followed.current.owner = explicitOwner;
  if (elsewhere?.path === path) followed.current.owner = elsewhere.owner;
  const borrowed = useSyncExternalStore(subscribe, () => sessions.get(followed.current.owner ?? "") ?? null, () => null);
  const source = path.startsWith("hick-project:///") ? path : `hick:///${path}`;
  const active = borrowed && ["paused","running"].includes(borrowed.status) && followed.current.owner !== path;
  useEffect(()=> {
    if (!active || !own || !borrowed) return;
    const held = borrowed.sourceBreakpoints[source];
    if (held && held !== own.breakpoints) own.replaceBreakpoints(held);
    else if (!held && own.breakpoints.length) borrowed.setSourceBreakpoints(source,own.breakpoints);
  },[active,borrowed,source,own]);
  const direct = useMemo(()=>own ? {...own,start:(program?:string, sources: Record<string, import("./useDebugger").HeldBreakpoint[]> = {}, project?: string, profile?: import("./launchProfiles").LaunchSelection)=> {
    const captured = /^hick-project:\/\/\/[^/]+\//.exec(path)?.[0];
    const initial = Object.fromEntries([...sessions].filter(([other])=>other!==path &&
      isDebuggable(languageFromPath(other)) && (captured ? other.startsWith(captured) : isWorkspaceSource(other)))
      .map(([other,session])=>[other.startsWith("hick-project:///")?other:`hick:///${other}`,session.breakpoints]));
    if(captured)for(const file of capturedFiles(path)) {
      const uri=`${captured}${file}`, held=breakpointPreferences(capturedSourceKey(uri));
      if(uri!==path && held)initial[uri]=held;
    }
    own.start(program,{...initial,...sources},project,profile);
  }} : null,[own,path]);
  if (!borrowed || !["starting", "running", "paused"].includes(borrowed.status) || followed.current.owner === path) return preferences(direct,key);
  const frame = borrowed.frames.find(f => f.source === path);
  const pausedLine = frame?.source_line ?? (elsewhere?.path === path ? elsewhere.line : null);
  const breakpoints = borrowed.sourceBreakpoints[source] ?? own?.breakpoints ?? [];
  return preferences({ ...borrowed, pausedLine, program: borrowed.program ?? followed.current.owner ?? null,
    // These protocol verbs address the launch file, not the viewed callee.
    // Never send a callee's line number as a line in the launch file.
    capabilities: borrowed.capabilities ? {...borrowed.capabilities, goto_targets:false} : null,
    jumpTo: () => {}, runTo: () => {},
    frames: borrowed.frames.map(f => ({ ...f, in_document: f.source === path, line: f.source === path ? (f.source_line ?? null) : null })),
    breakpoints,
    start: own?.start ?? (() => {}),
    toggleBreakpoint: line => borrowed.setSourceBreakpoints(source,breakpoints.some(bp=>bp.line===line)
      ? breakpoints.filter(bp=>bp.line!==line) : [...breakpoints,{line,state:"pending"}]),
    setBreakpointCondition: (line,patch) => borrowed.setSourceBreakpoints(source,breakpoints.map(bp=>bp.line===line?{...bp,...patch}:bp)),
  },key);
}

</storycode:copy>
<a id="f-238f66c701b0-preferences"></a>
### `preferences`

<storycode:copy id="f-238f66c701b0-preferences">
function preferences(debug:DebugSession|null,key:string|null):DebugSession|null {
  if(!debug || !key)return debug;
  return {...debug,toggleBreakpoint:line=>{
    const next=debug.breakpoints.some(bp=>bp.line===line)?debug.breakpoints.filter(bp=>bp.line!==line):[...debug.breakpoints,{line,state:"pending" as const}];
    rememberBreakpointPreferences(key,next);debug.toggleBreakpoint(line);
  },setBreakpointCondition:(line,patch)=>{
    rememberBreakpointPreferences(key,debug.breakpoints.map(bp=>bp.line===line?{...bp,...patch}:bp));
    debug.setBreakpointCondition(line,patch);
  }};
}
</storycode:copy>

### Assemble `apps/web/src/debug/plainDebugHosts.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/plainDebugHosts.tsx"><storycode:paste select="#f-238f66c701b0-isWorkspaceSource"/><storycode:paste select="#f-238f66c701b0-wanted"/><storycode:paste select="#f-238f66c701b0-sessions"/><storycode:paste select="#f-238f66c701b0-listeners"/><storycode:paste select="#f-238f66c701b0-emit"/><storycode:paste select="#f-238f66c701b0-subscribe"/><storycode:paste select="#f-238f66c701b0-ensurePlainDebugHost"/><storycode:paste select="#f-238f66c701b0-wantedPaths"/><storycode:paste select="#f-238f66c701b0-usePlainDebugSession"/><storycode:paste select="#f-238f66c701b0-resetPlainDebugHosts"/><storycode:paste select="#f-238f66c701b0-Host"/><storycode:paste select="#f-238f66c701b0-PlainDebugHosts"/><storycode:paste select="#f-238f66c701b0-usePublishDebugSession"/><storycode:paste select="#f-238f66c701b0-useFollowingDebugSession"/><storycode:paste select="#f-238f66c701b0-preferences"/></storycode:file>

<a id="output-75fab5522700"></a>
## `apps/web/src/debug/projectDebugSurface.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-75fab5522700-ComparisonSurface"></a>
### `ComparisonSurface`

<storycode:copy id="f-75fab5522700-ComparisonSurface">
import { useId, useRef } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { useEffect, useState, useSyncExternalStore } from "react";
import { EditorView } from "@codemirror/view";
import type { ProjectEntry } from "../editor/hickLang";
import type { Frame } from "./client";
import type { DebugSession } from "./useDebugger";
import { preferProjectDebugSurface, revealPreferredDebugLocation } from "./preferredDebugLocation";
import { projectFrameOffset } from "./projectFrameOffset";
import { parseHickDoc } from "../editor/hickDoc";
import { isRendered } from "../editor/rendered";
import { revealLine, setPausedLine } from "./cmDebug";
import { openLocation } from "../lib/revealLine";
import { openCapturedCode } from "../lib/capturedCode";
import { PromptPanel, usePrompt } from "../components/PromptPanel";
import { DebugStrip } from "./DebugStrip";

interface ComparisonSurface {view:EditorView;source:string;outer?:EditorView}
</storycode:copy>
<a id="f-75fab5522700-comparisons"></a>
### `comparisons`

<storycode:copy id="f-75fab5522700-comparisons">
const comparisons=new Map<string,Set<ComparisonSurface>>();
</storycode:copy>
<a id="f-75fab5522700-registerCapturedComparison"></a>
### `registerCapturedComparison`

<storycode:copy id="f-75fab5522700-registerCapturedComparison">
export function registerCapturedComparison(uri:string,view:EditorView,source:string,outer?:EditorView):()=>void {
  const entry={view,source,outer}, all=comparisons.get(uri) ?? new Set<ComparisonSurface>();
  all.add(entry);comparisons.set(uri,all);
  return ()=>{all.delete(entry);if(!all.size)comparisons.delete(uri);};
}
</storycode:copy>
<a id="f-75fab5522700-revealComparison"></a>
### `revealComparison`

<storycode:copy id="f-75fab5522700-revealComparison">
function revealComparison(frame:Frame,outer:EditorView):boolean {
  if(!frame.source || !Number.isInteger(frame.source_line) || frame.source_line!<0)return false;
  for(const surface of comparisons.get(frame.source) ?? []) {
    const view=surface.view;
    if(surface.outer!==outer || !view.dom.isConnected || !view.dom.getClientRects().length
        || view.state.doc.toString()!==surface.source || frame.source_line!>=view.state.doc.lines)continue;
    view.dispatch({effects:setPausedLine.of(frame.source_line!)});revealLine(view,frame.source_line!);return true;
  }
  return false;
}

</storycode:copy>
<a id="f-75fab5522700-Surface"></a>
### `Surface`

<storycode:copy id="f-75fab5522700-Surface">
interface Surface { view: EditorView; entry: string; source: string; path?: string; codeView?: EditorView | null; codeUri?: string; codeText?: string; frame?: Frame }
</storycode:copy>
<a id="f-75fab5522700-locations"></a>
### `locations`

<storycode:copy id="f-75fab5522700-locations">
const locations = new Map<string, Surface>();
</storycode:copy>
<a id="f-75fab5522700-listeners"></a>
### `listeners`

<storycode:copy id="f-75fab5522700-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-75fab5522700-version"></a>
### `version`

<storycode:copy id="f-75fab5522700-version">
let version = 0;
</storycode:copy>
<a id="f-75fab5522700-subscribe"></a>
### `subscribe`

<storycode:copy id="f-75fab5522700-subscribe">
const subscribe = (fn: () => void) => { listeners.add(fn); return () => { listeners.delete(fn); }; };
</storycode:copy>
<a id="f-75fab5522700-emit"></a>
### `emit`

<storycode:copy id="f-75fab5522700-emit">
const emit = () => { version++; for (const listener of listeners) listener(); };
</storycode:copy>
<a id="f-75fab5522700-useSurface"></a>
### `useSurface`

<storycode:copy id="f-75fab5522700-useSurface">
function useSurface(owner: string) {
  useSyncExternalStore(subscribe, () => version, () => 0);
  return locations.get(owner);
}
</storycode:copy>
<a id="f-75fab5522700-preferCapturedLiterateSurface"></a>
### `preferCapturedLiterateSurface`

<storycode:copy id="f-75fab5522700-preferCapturedLiterateSurface">
export function preferCapturedLiterateSurface(owner: string, prefix: string, entries: ProjectEntry[], surface: Surface) {
  preferProjectDebugSurface(owner, frame => {
    surface.frame = frame;
    const outer = surface.view;
    const code = surface.codeView;
    if (code?.dom.isConnected && code.dom.getClientRects().length && frame.source===surface.codeUri
        && code.state.doc.toString()===surface.codeText && typeof frame.source_line==="number") {
      code.dispatch({effects:setPausedLine.of(frame.source_line)});
      revealLine(code,frame.source_line);
      return true;
    }
    if(revealComparison(frame,outer))return true;
    const at = projectFrameOffset(prefix, entries, surface.entry, surface.source, frame);
    if (at !== null && (!outer.dom.isConnected || !outer.dom.getClientRects().length) && surface.path) {
      openLocation(surface.path, surface.source.slice(0,at).split("\n").length);
      if (!outer.dom.isConnected) return true;
    }
    if (!outer.dom.isConnected || outer.state.doc.toString() !== surface.source) return false;
    if (at === null) { outer.dispatch({effects:setPausedLine.of(null)}); return false; }
    const block = parseHickDoc(surface.source).blocks.filter(b => b.from <= at && at < b.to).sort((a,b) => (a.to-a.from)-(b.to-b.from))[0];
    if (block && isRendered(outer.state, block.from)) {
      outer.dispatch({effects:[setPausedLine.of(null), EditorView.scrollIntoView(block.from,{y:"center"})]});
      return true;
    }
    const line = outer.state.doc.lineAt(at).number - 1;
    outer.dispatch({effects:setPausedLine.of(line)});
    revealLine(outer, line);
    return true;
  }, () => {
    if (surface.view.dom.isConnected) surface.view.dispatch({effects:setPausedLine.of(null)});
    locations.delete(owner); emit();
  });
  locations.set(owner, surface); emit();
}

</storycode:copy>
<a id="f-75fab5522700-bindProjectDebugView"></a>
### `bindProjectDebugView`

Rebind a returning note to the still-live captured debugger.

<storycode:copy id="f-75fab5522700-bindProjectDebugView">
/** Rebind a returning note to the still-live captured debugger. */
export function bindProjectDebugView(path: string, view: EditorView) {
  for (const [owner,surface] of locations) if (surface.path === path && view.state.doc.toString() === surface.source) {
    surface.view = view;
    if (surface.frame) revealPreferredDebugLocation(owner,surface.frame);
  }
  emit();
}

</storycode:copy>
<a id="f-75fab5522700-ProjectDebugLinks"></a>
### `ProjectDebugLinks`

Both locations address one paused frame and keep the same live controls.

<storycode:copy id="f-75fab5522700-ProjectDebugLinks">
/** Both locations address one paused frame and keep the same live controls. */
export function ProjectDebugLinks({owner, debug}: {owner:string; debug:DebugSession}) {
  const parent = `project-debug-links:${useId()}`;
  const [error,setError] = useState("");
  const surface = useSurface(owner);
  const frame = debug.frames.find(f => f.id === debug.selectedFrame) ?? debug.frames[0];
  const revision = useUiRevision(() => [owner, version, debug.status, debug.reading?.scope, frame, surface?.source, surface?.view.state.doc.toString(), surface?.view.dom.isConnected]);
  const latestRevision = useRef(revision); latestRevision.current = revision;
  const current = () => !!surface && locations.get(owner) === surface && surface.view.dom.isConnected && surface.view.state.doc.toString() === surface.source && debug.status === "paused" && !!frame;
  const active = !!surface && debug.status === "paused" && !!frame;
  const hasCode = !!frame?.source?.startsWith("hick-project:///") && Number.isInteger(frame.source_line) && (frame.source_line ?? -1) >= 0;
  useUiReading("captured debugger location links", active ? { owner, frame, current: current(), error } : null);
  const invoke = useUiCommands(active ? (["literate", "generated"] as const).map(kind => ({ key: `${parent}:${kind}`, command: {
    label: kind === "literate" ? "Show captured debugger literate source" : "Show captured debugger generated code", parent, subject: parent, parameters: {}, prerequisites: ["Current paused frame and captured project/editor owner"],
    read: () => ({ revision: revision(), enabled: current() && (kind === "literate" || hasCode), reason: "Wait for a paused captured project frame and current source" }),
    execute: async (): Promise<UiOutcome> => {
      if (!current() || !frame) return { status: "unavailable", reason: "The captured debugger frame or editor changed" };
      if (kind === "literate") {
        if (!revealPreferredDebugLocation(owner, frame)) return { status: "unavailable", reason: "The current frame has no exact visible literate source mapping" };
        return { status: "completed", value: { source_selection_requested: true, frame: frame.id } };
      }
      if (!hasCode || !frame.source) return { status: "unavailable", reason: "This frame has no captured project code" };
      const captured = revision();
      await openCapturedCode({ uri: frame.source, range: { start: { line: frame.source_line ?? 0, character: 0 }, end: { line: frame.source_line ?? 0, character: 0 } } }, owner, () => current() && latestRevision.current() === captured);
      return { status: "completed", value: { captured_reading_open_requested: true, frame: frame.id, executed: false } };
    },
  } })) : []);
  if (!active) return null;
  const act = (kind: string) => { void invoke(`${parent}:${kind}`).catch(cause => setError(cause instanceof Error ? cause.message : String(cause))); };
  return <span className="project-debug-links">
    <button className="debug-icon" type="button" aria-label="Show literate source" data-tip="Show literate source" onClick={() => act("literate")}>¶</button>
    <button className="debug-icon" type="button" aria-label="Show generated code" data-tip="Show generated code" disabled={!hasCode} onClick={() => act("generated")}>{"{}"}</button>
    {error && <span role="alert">Could not open generated code: {error}</span>}
  </span>;
}

</storycode:copy>
<a id="f-75fab5522700-ProjectDebugSurface"></a>
### `ProjectDebugSurface`

<storycode:copy id="f-75fab5522700-ProjectDebugSurface">
export function ProjectDebugSurface({owner, debug}: {owner:string; debug:DebugSession}) {
  const prompt = usePrompt();
  const surface = useSurface(owner);
  useEffect(() => {
    if (surface && debug.status !== "paused") {
      surface.frame = undefined;
      if (surface.view.dom.isConnected) surface.view.dispatch({effects:setPausedLine.of(null)});
    }
  },[surface,debug.status]);
  if (!surface?.view.dom.isConnected || !["running","paused"].includes(debug.status)) return null;
  return <>
    <DebugStrip {...debug} locationActions={<ProjectDebugLinks owner={owner} debug={debug}/>} capabilities={debug.capabilities ? {...debug.capabilities, goto_targets:false} : null} editorView={surface.view} pausedLine={null}
      onSelectFrame={id => {debug.selectFrame(id); const frame=debug.frames.find(f=>f.id===id); if(frame)revealPreferredDebugLocation(owner,frame);}}
      onStep={debug.step} onStart={() => debug.start()} onStop={debug.stop} onJumpHere={() => {}}
      onAddWatch={() => { void prompt.askText("Expression to watch:", "", { agent: true }).then(expression => { if(expression) debug.addWatch(expression); }); }} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle}/>
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/debug/projectDebugSurface.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/projectDebugSurface.tsx"><storycode:paste select="#f-75fab5522700-ComparisonSurface"/><storycode:paste select="#f-75fab5522700-comparisons"/><storycode:paste select="#f-75fab5522700-registerCapturedComparison"/><storycode:paste select="#f-75fab5522700-revealComparison"/><storycode:paste select="#f-75fab5522700-Surface"/><storycode:paste select="#f-75fab5522700-locations"/><storycode:paste select="#f-75fab5522700-listeners"/><storycode:paste select="#f-75fab5522700-version"/><storycode:paste select="#f-75fab5522700-subscribe"/><storycode:paste select="#f-75fab5522700-emit"/><storycode:paste select="#f-75fab5522700-useSurface"/><storycode:paste select="#f-75fab5522700-preferCapturedLiterateSurface"/><storycode:paste select="#f-75fab5522700-bindProjectDebugView"/><storycode:paste select="#f-75fab5522700-ProjectDebugLinks"/><storycode:paste select="#f-75fab5522700-ProjectDebugSurface"/></storycode:file>

## Test the behavior: `atomic_scope_never_exposes_a_new_caller_with_the_old_library`

This is the maintained test fragment from `crates/hick-lang/tests/project_debug.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3d2aa0516f40-atomic_scope_never_exposes_a_new_caller_with_the_old_library"></a>
### `atomic_scope_never_exposes_a_new_caller_with_the_old_library`

<storycode:copy id="f-3d2aa0516f40-atomic_scope_never_exposes_a_new_caller_with_the_old_library">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn atomic_scope_never_exposes_a_new_caller_with_the_old_library() {
    let source = "<hick:project/><hick:file id=\"app\" path=\"app.py\">from library import greet\nprint(greet())\n</hick:file><hick:file id=\"library\" path=\"library.py\">def greet():\n    return 'Hello'\n</hick:file><hick:scope id=\"greeting\" after=\"app library\"><hick:patch id=\"caller\">\n--- a/app.py\n+++ b/app.py\n@@ -1,2 +1,2 @@\n-from library import greet\n-print(greet())\n+from library import welcome\n+print(welcome('Hickory'))\n</hick:patch><hick:patch id=\"complete\" after=\"caller\">\n--- a/library.py\n+++ b/library.py\n@@ -1,2 +1,2 @@\n-def greet():\n-    return 'Hello'\n+def welcome(name):\n+    return 'Hello ' + name\n</hick:patch></hick:scope>";
    let project = evaluate_project(
        "p.md",
        &BTreeMap::from([("p.md".into(), source.replace("><hick:", ">\n<hick:"))]),
        &ProjectTree::default(),
    )
    .unwrap();
    let old = latest_debug_project(
        &project.versions,
        &project.local_states["complete"].0,
        "library.py",
    )
    .unwrap()
    .0;
    assert_eq!(
        project.versions[old].get("app.py").unwrap().bytes.as_ref(),
        b"from library import greet\nprint(greet())\n"
    );
    let old_caller = latest_debug_project(
        &project.versions,
        &project.local_states["caller"].0,
        "app.py",
    )
    .unwrap()
    .0;
    assert_eq!(old, old_caller);
    let new = latest_debug_project(
        &project.versions,
        &project.local_states["caller"].1,
        "app.py",
    )
    .unwrap()
    .0;
    assert_eq!(
        project.versions[new]
            .get("library.py")
            .unwrap()
            .bytes
            .as_ref(),
        b"def welcome(name):\n    return 'Hello ' + name\n"
    );
    assert_eq!(new, project.versions.len() - 1);
}
</storycode:copy>

<a id="output-05e2104e1cc1"></a>
## `crates/hickory-cli/src/debug_profiles.rs`

Related contracts: [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-05e2104e1cc1-ProfileSelection"></a>
### `ProfileSelection`

Resolve a profile independently from the source whose breakpoints are primary.

<storycode:copy id="f-05e2104e1cc1-ProfileSelection">
//! Resolve a profile independently from the source whose breakpoints are primary.
use super::*;
use crate::launch_profiles::{self, Profile};
use hick_lang::{ProjectEntry, ProjectTree};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSelection {
    pub id: String,
    pub fingerprint: String,
}

</storycode:copy>
<a id="f-05e2104e1cc1-LaunchContext"></a>
### `LaunchContext`

<storycode:copy id="f-05e2104e1cc1-LaunchContext">
pub struct LaunchContext {
    pub tree: ProjectTree,
    pub root: PathBuf,
    pub viewed: String,
    identity: Identity,
}
</storycode:copy>
<a id="f-05e2104e1cc1-Identity"></a>
### `Identity`

<storycode:copy id="f-05e2104e1cc1-Identity">
enum Identity {
    Captured(String),
    Document(PathBuf, Box<hick_lsp::document::HickDocumentState>, String),
    Plain(PathBuf),
}
</storycode:copy>
<a id="f-05e2104e1cc1-impl-LaunchContext"></a>
### `impl-LaunchContext`

<storycode:copy id="f-05e2104e1cc1-impl-LaunchContext">
impl LaunchContext {
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        for (path, entry) in self.tree.entries() {
            h.update((path.len() as u64).to_le_bytes());
            h.update(path.as_bytes());
            h.update((entry.bytes.len() as u64).to_le_bytes());
            h.update(&entry.bytes);
            h.update([
                entry.executable as u8,
                matches!(entry.kind, hick_lang::EntryKind::File) as u8,
            ]);
            for origin in &entry.origins {
                h.update(serde_json::to_vec(origin).expect("byte origins are serializable"));
            }
        }
        if let Identity::Document(_, _, raw_source) = &self.identity {
            h.update(raw_source.as_bytes());
        }
        format!("{:x}", h.finalize())
    }

    pub fn check_expected(&self, code: Option<&str>, document: Option<&str>) -> Result<()> {
        if let Some(code) = code {
            anyhow::ensure!(
                self.tree
                    .get(&self.viewed)
                    .is_some_and(|e| e.bytes.as_ref() == code.as_bytes()),
                "The saved file differs from the displayed code. Save or reload it before choosing a launch profile."
            );
        }
        if let (Some(expected), Identity::Document(_, _, raw_source)) = (document, &self.identity) {
            anyhow::ensure!(
                raw_source == expected,
                "The document changed. Save or reload it before choosing a launch profile."
            );
        }
        Ok(())
    }
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        launch_profiles::discover(&self.tree).map_err(anyhow::Error::msg)
    }
    pub fn selected(&self, selection: &ProfileSelection) -> Result<Profile> {
        anyhow::ensure!(
            self.fingerprint() == selection.fingerprint,
            "The project changed after choosing a launch profile. Open Launch profiles and choose again."
        );
        launch_profiles::select(&self.tree, Some(&selection.id)).map_err(anyhow::Error::msg)
    }
    pub fn prepare(&self) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
        if matches!(self.identity, Identity::Plain(_)) {
            return Ok((self.root.clone(), None));
        }
        let scratch = tempfile::tempdir()?;
        hick_lsp::write_tree(scratch.path(), &self.tree).map_err(anyhow::Error::msg)?;
        Ok((scratch.path().to_path_buf(), Some(scratch)))
    }
    fn mapping(&self, root: &Path) -> Result<Mapping> {
        match &self.identity {
            Identity::Captured(id) => Mapping::captured(&self.tree, root, id, &self.viewed),
            Identity::Document(path, state, raw_source) => {
                Ok(Mapping::for_state(path, state, root)?.with_observation_source(raw_source))
            }
            Identity::Plain(path) => Ok(Mapping::identity(path)),
        }
    }
}
</storycode:copy>
<a id="f-05e2104e1cc1-impl-Registry"></a>
### `impl-Registry`

<storycode:copy id="f-05e2104e1cc1-impl-Registry">
impl Registry {
    pub fn launch_context(
        &self,
        root: &Path,
        uri: &str,
        selection: &str,
        viewed: Option<&str>,
    ) -> Result<LaunchContext> {
        if uri.starts_with("hick-project:///") {
            let (id, path, capture) = self
                .projects
                .get(&tower_lsp::lsp_types::Url::parse(uri)?)
                .map_err(anyhow::Error::msg)?;
            return Ok(LaunchContext {
                tree: capture.tree.clone(),
                root: root.into(),
                viewed: path,
                identity: Identity::Captured(id),
            });
        }
        let relative = uri.strip_prefix("hick:///").unwrap_or(uri);
        hick_lang::validate_project_path(relative).map_err(anyhow::Error::msg)?;
        let path = root.join(relative).canonicalize()?;
        anyhow::ensure!(
            path.starts_with(root.canonicalize()?),
            "Launch source is outside the open folder. Choose a file inside this workspace."
        );
        if is_document(&path) {
            let source = std::fs::read_to_string(&path)?;
            let endpoint =
                hick_lsp::ProjectEndpoint::parse(selection).map_err(anyhow::Error::msg)?;
            let state = crate::project_documents::state_at(&path, &source, &endpoint)?;
            let mut tree = state.selected_project.clone().unwrap_or_default();
            if state.selected_project.is_none() {
                for file in &state.virtual_files {
                    tree.insert(
                        &file.path,
                        ProjectEntry::file(file.written_content().into_bytes()),
                    )
                    .map_err(anyhow::Error::msg)?;
                }
            }
            let target = if matches!(
                endpoint,
                hick_lsp::ProjectEndpoint::LatestFile { .. }
                    | hick_lsp::ProjectEndpoint::LatestCode { .. }
            ) {
                hick_lsp::debug_selection(
                    state
                        .evaluated_project
                        .as_ref()
                        .context("Missing project")?,
                    &endpoint,
                )
                .map_err(anyhow::Error::msg)?
                .1
            } else {
                viewed
                    .context("Choose a file block before selecting a launch profile")?
                    .into()
            };
            return Ok(LaunchContext {
                tree,
                root: path.parent().unwrap_or(root).into(),
                viewed: target,
                identity: Identity::Document(path, Box::new(state), source),
            });
        }
        // Workspace discovery respects gitignore, never follows directory symlinks,
        // and reads source/manifests only. It does not materialize a second workspace.
        let mut tree = ProjectTree::default();
        for item in ignore::WalkBuilder::new(root)
            .hidden(false)
            .follow_links(false)
            .build()
        {
            let item = item?;
            if !item.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let relative = item
                .path()
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            let relevant = [".py", ".js", ".mjs", ".cjs", ".rs", ".go", ".cs", ".csproj"]
                .iter()
                .any(|e| relative.ends_with(e))
                || [
                    launch_profiles::CONFIG,
                    "package.json",
                    "Cargo.toml",
                    "go.mod",
                    "pyproject.toml",
                    "pytest.ini",
                ]
                .contains(&item.file_name().to_string_lossy().as_ref());
            if relevant {
                tree.insert(&relative, ProjectEntry::file(std::fs::read(item.path())?))
                    .map_err(anyhow::Error::msg)?;
            }
        }
        // Hidden config remains available even if the user's ignore rules hide it.
        let config = root.join(launch_profiles::CONFIG);
        if config.exists() {
            anyhow::ensure!(
                std::fs::symlink_metadata(&config)?.is_file(),
                "{} must be a regular file",
                launch_profiles::CONFIG
            );
            tree.insert(
                launch_profiles::CONFIG,
                ProjectEntry::file(std::fs::read(config)?),
            )
            .map_err(anyhow::Error::msg)?;
        }
        let viewed = path
            .strip_prefix(root.canonicalize()?)?
            .to_string_lossy()
            .replace('\\', "/");
        tree.insert(&viewed, ProjectEntry::file(std::fs::read(&path)?))
            .map_err(anyhow::Error::msg)?;
        Ok(LaunchContext {
            tree,
            root: root.into(),
            viewed,
            identity: Identity::Plain(path),
        })
    }

    pub async fn start_profile(
        &self,
        context: LaunchContext,
        selection: &ProfileSelection,
        breakpoints: &[Breakpoint],
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        let profile = context.selected(selection)?;
        anyhow::ensure!(
            profile.can_debug,
            "{} is a Run-only profile. Choose Run, or an application profile with a debug adapter.",
            profile.definition.name
        );
        let (root, scratch) = context.prepare()?;
        let cwd = safe_cwd(&root, &profile.definition.cwd)?;
        run_build(&profile, &cwd, on_build).await?;
        let entry = root.join(
            profile
                .definition
                .program
                .as_ref()
                .context("Profile has no program")?,
        );
        let adapter =
            hick_dap::adapter_for(&entry, self.host_root.as_deref().unwrap_or(&context.root))?;
        let program = if let Some(artifact) = &profile.definition.artifact {
            let artifact = root.join(artifact).canonicalize().context(
                "The configured debug artifact does not exist. Fix the build or artifact path.",
            )?;
            anyhow::ensure!(
                artifact.starts_with(root.canonicalize()?),
                "The debug artifact escapes the project"
            );
            artifact
        } else if scratch.is_some() {
            hick_dap::build(&entry, &root, &context.root, on_build).await?
        } else {
            hick_dap::build_plain(&entry, &root, on_build).await?
        };
        let mut extra =
            serde_json::json!({"args":profile.definition.args,"env":profile.definition.env});
        if hick_dap::language_of(&entry) == Some("python")
            && let Some(python) = hick_project_env::python_interpreter(&context.root, &context.root)
        {
            extra["python"] = serde_json::json!(python);
        }
        on_build(BuildOutput::Note(format!(
            "Debug {} · source {} · project {}",
            profile.definition.name,
            context.viewed,
            context.fingerprint()
        )));
        let mapping = context.mapping(&root)?;
        let (session, statuses) = Session::start_with_sources(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd,
                extra,
            },
            Arc::new(mapping),
            breakpoints,
            sources,
        )
        .await?;
        Ok(self.register(session, scratch, statuses).await)
    }
}

</storycode:copy>
<a id="f-05e2104e1cc1-safe_cwd"></a>
### `safe_cwd`

<storycode:copy id="f-05e2104e1cc1-safe_cwd">
pub(crate) fn safe_cwd(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = root.join(relative).canonicalize().with_context(|| {
        format!(
            "Working directory {relative} does not exist in this project. Fix the launch profile."
        )
    })?;
    anyhow::ensure!(
        path.starts_with(root.canonicalize()?),
        "Launch working directory escapes this project. Fix the launch profile."
    );
    Ok(path)
}
</storycode:copy>
<a id="f-05e2104e1cc1-run_build"></a>
### `run_build`

<storycode:copy id="f-05e2104e1cc1-run_build">
async fn run_build(
    profile: &Profile,
    cwd: &Path,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<()> {
    let argv = &profile.definition.build;
    if argv.is_empty() {
        return Ok(());
    }
    on_build(BuildOutput::Cmd(format!("{argv:?}")));
    let mut child = tokio::process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .envs(&profile.definition.env)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut out = BufReader::new(child.stdout.take().context("Missing build stdout")?).lines();
    let mut err = BufReader::new(child.stderr.take().context("Missing build stderr")?).lines();
    let (mut stdout, mut stderr) = (true, true);
    while stdout || stderr {
        tokio::select! {
            line = out.next_line(), if stdout => match line? {Some(line)=>on_build(BuildOutput::Out(line)),None=>stdout=false},
            line = err.next_line(), if stderr => match line? {Some(line)=>on_build(BuildOutput::Err(line)),None=>stderr=false},
        }
    }
    let status = child.wait().await?;
    on_build(BuildOutput::Exit(status.code().unwrap_or(-1)));
    anyhow::ensure!(
        status.success(),
        "Build for {} failed; see its output and fix the profile or project.",
        profile.definition.name
    );
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/debug_profiles.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/debug_profiles.rs"><storycode:paste select="#f-05e2104e1cc1-ProfileSelection"/><storycode:paste select="#f-05e2104e1cc1-LaunchContext"/><storycode:paste select="#f-05e2104e1cc1-Identity"/><storycode:paste select="#f-05e2104e1cc1-impl-LaunchContext"/><storycode:paste select="#f-05e2104e1cc1-impl-Registry"/><storycode:paste select="#f-05e2104e1cc1-safe_cwd"/><storycode:paste select="#f-05e2104e1cc1-run_build"/></storycode:file>

<a id="output-01e001976095"></a>
## `crates/hickory-cli/src/launch_profiles.rs`

Related contracts: [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-01e001976095-CONFIG"></a>
### `CONFIG`

Launch discovery is read-only. Providers and manual profiles produce one model.

<storycode:copy id="f-01e001976095-CONFIG">
//! Launch discovery is read-only. Providers and manual profiles produce one model.
use hick_lang::{EntryKind, ProjectTree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONFIG: &str = ".hick-launch.json";

</storycode:copy>
<a id="f-01e001976095-Definition"></a>
### `Definition`

<storycode:copy id="f-01e001976095-Definition">
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub command: Vec<String>,
    #[serde(default)]
    pub artifact: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default = "root")]
    pub cwd: String,
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub default: bool,
}
</storycode:copy>
<a id="f-01e001976095-root"></a>
### `root`

<storycode:copy id="f-01e001976095-root">
fn root() -> String {
    ".".into()
}
</storycode:copy>
<a id="f-01e001976095-Profile"></a>
### `Profile`

<storycode:copy id="f-01e001976095-Profile">
#[derive(Clone, Debug, Serialize)]
pub struct Profile {
    #[serde(flatten)]
    pub definition: Definition,
    pub origin: String,
    pub can_debug: bool,
}
</storycode:copy>
<a id="f-01e001976095-Config"></a>
### `Config`

<storycode:copy id="f-01e001976095-Config">
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    version: u32,
    profiles: Vec<Definition>,
}

</storycode:copy>
<a id="f-01e001976095-Provider"></a>
### `Provider`

Each provider receives the same immutable project, without starting tools.

<storycode:copy id="f-01e001976095-Provider">
/// Each provider receives the same immutable project, without starting tools.
pub trait Provider {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String>;
}
</storycode:copy>
<a id="f-01e001976095-Applications"></a>
### `Applications`

<storycode:copy id="f-01e001976095-Applications">
pub struct Applications;
</storycode:copy>
<a id="f-01e001976095-PackageScripts"></a>
### `PackageScripts`

<storycode:copy id="f-01e001976095-PackageScripts">
pub struct PackageScripts;
</storycode:copy>
<a id="f-01e001976095-PythonTests"></a>
### `PythonTests`

<storycode:copy id="f-01e001976095-PythonTests">
pub struct PythonTests;

</storycode:copy>
<a id="f-01e001976095-profile"></a>
### `profile`

<storycode:copy id="f-01e001976095-profile">
fn profile(
    id: String,
    name: String,
    program: Option<String>,
    command: Vec<String>,
    cwd: String,
    origin: &str,
) -> Profile {
    let can_debug = program
        .as_ref()
        .is_some_and(|p| hick_dap::language_of(std::path::Path::new(p)).is_some());
    Profile {
        definition: Definition {
            id,
            name,
            program,
            command,
            artifact: None,
            cwd,
            args: vec![],
            env: BTreeMap::new(),
            build: vec![],
            default: false,
        },
        origin: origin.into(),
        can_debug,
    }
}
</storycode:copy>
<a id="f-01e001976095-command_profile"></a>
### `command_profile`

Existing test gutters and command providers share the same launch model.

<storycode:copy id="f-01e001976095-command_profile">
/// Existing test gutters and command providers share the same launch model.
pub fn command_profile(
    id: String,
    name: String,
    command: Vec<String>,
    cwd: String,
    origin: &str,
) -> Profile {
    profile(id, name, None, command, cwd, origin)
}
</storycode:copy>
<a id="f-01e001976095-text"></a>
### `text`

<storycode:copy id="f-01e001976095-text">
fn text<'a>(tree: &'a ProjectTree, path: &str) -> Option<&'a str> {
    let entry = tree.get(path)?;
    (entry.kind == EntryKind::File)
        .then(|| std::str::from_utf8(&entry.bytes).ok())
        .flatten()
}
</storycode:copy>
<a id="f-01e001976095-impl-Provider-for-Applications"></a>
### `impl-Provider for Applications`

Conservative syntax-shaped discovery, not a claim about call-graph reachability.

<storycode:copy id="f-01e001976095-impl-Provider-for-Applications">
/// Conservative syntax-shaped discovery, not a claim about call-graph reachability.
impl Provider for Applications {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree.entries().keys() {
            let Some(source) = text(tree, path) else {
                continue;
            };
            let is_test = path
                .rsplit('/')
                .next()
                .is_some_and(|p| p.starts_with("test_") || p.ends_with("_test.py"));
            let runnable = if path.ends_with(".py") && !is_test {
                source.lines().any(|line| {
                    if line.starts_with(char::is_whitespace) {
                        return false;
                    }
                    let line = line.trim();
                    line.starts_with("if __name__")
                        || line.starts_with("print(")
                        || line.starts_with("asyncio.run(")
                })
            } else if path.ends_with(".js") || path.ends_with(".mjs") || path.ends_with(".cjs") {
                source.contains("console.")
                    || source.contains(".listen(")
                    || path.ends_with("/index.js")
                    || path == "index.js"
            } else if path.ends_with(".rs") {
                (path.ends_with("main.rs") || path.contains("/src/bin/"))
                    && source.contains("fn main(")
            } else if path.ends_with(".go") {
                source.contains("func main(")
            } else if path.ends_with(".cs") {
                path.ends_with("Program.cs")
                    || source.contains("static void Main(")
                    || source.contains("static async Task Main(")
            } else {
                false
            };
            if runnable {
                let manifest = if path.ends_with(".rs") {
                    Some("Cargo.toml")
                } else if path.ends_with(".cs") {
                    Some(".csproj")
                } else {
                    None
                };
                let cwd = manifest
                    .and_then(|name| {
                        tree.entries()
                            .keys()
                            .filter(|p| p.ends_with(name))
                            .filter_map(|p| {
                                let dir = p.rsplit_once('/').map_or("", |(d, _)| d);
                                (dir.is_empty() || path.starts_with(&format!("{dir}/")))
                                    .then_some(dir)
                            })
                            .max_by_key(|p| p.len())
                    })
                    .filter(|p| !p.is_empty())
                    .unwrap_or(".");
                found.push(profile(
                    format!("app:{path}"),
                    path.clone(),
                    Some(path.clone()),
                    vec![],
                    cwd.into(),
                    "application",
                ));
            }
        }
        Ok(found)
    }
}
</storycode:copy>
<a id="f-01e001976095-impl-Provider-for-PackageScripts"></a>
### `impl-Provider for PackageScripts`

<storycode:copy id="f-01e001976095-impl-Provider-for-PackageScripts">
impl Provider for PackageScripts {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree
            .entries()
            .keys()
            .filter(|p| p.as_str() == "package.json" || p.ends_with("/package.json"))
        {
            let Some(source) = text(tree, path) else {
                continue;
            };
            let manifest: serde_json::Value =
                serde_json::from_str(source).map_err(|e| format!("Cannot read {path}: {e}"))?;
            let cwd = path.rsplit_once('/').map_or(".", |(p, _)| p);
            if let Some(scripts) = manifest["scripts"].as_object() {
                for name in scripts.keys() {
                    found.push(profile(
                        format!("npm:{path}:{name}"),
                        format!("{cwd} · npm run {name}"),
                        None,
                        vec!["npm".into(), "run".into(), name.clone()],
                        cwd.into(),
                        "package script",
                    ));
                }
            }
        }
        Ok(found)
    }
}
</storycode:copy>
<a id="f-01e001976095-impl-Provider-for-PythonTests"></a>
### `impl-Provider for PythonTests`

<storycode:copy id="f-01e001976095-impl-Provider-for-PythonTests">
impl Provider for PythonTests {
    fn discover(&self, tree: &ProjectTree) -> Result<Vec<Profile>, String> {
        let mut found = vec![];
        for path in tree.entries().keys().filter(|p| p.ends_with(".py")) {
            let Some(source) = text(tree, path) else {
                continue;
            };
            for line in source.lines() {
                let Some(name) = line
                    .strip_prefix("def ")
                    .or_else(|| line.strip_prefix("async def "))
                    .and_then(|s| s.split_once('(').map(|s| s.0))
                else {
                    continue;
                };
                if name.starts_with("test_")
                    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    found.push(profile(
                        format!("pytest:{path}:{name}"),
                        format!("{path}::{name}"),
                        None,
                        vec![
                            "python3".into(),
                            "-m".into(),
                            "pytest".into(),
                            format!("{path}::{name}"),
                        ],
                        root(),
                        "test",
                    ));
                }
            }
        }
        Ok(found)
    }
}

</storycode:copy>
<a id="f-01e001976095-discover_with"></a>
### `discover_with`

<storycode:copy id="f-01e001976095-discover_with">
pub fn discover_with(
    tree: &ProjectTree,
    providers: &[&dyn Provider],
) -> Result<Vec<Profile>, String> {
    let mut found = BTreeMap::new();
    for provider in providers {
        for profile in provider.discover(tree)? {
            validate(&profile.definition, tree, &profile.origin)?;
            found.insert(profile.definition.id.clone(), profile);
        }
    }
    if let Some(source) = text(tree, CONFIG) {
        let config: Config =
            serde_json::from_str(source).map_err(|e| format!("Fix {CONFIG}: {e}"))?;
        if config.version != 1 {
            return Err(format!(
                "{CONFIG} needs version 1; found {}",
                config.version
            ));
        }
        let mut ids = BTreeSet::new();
        for definition in config.profiles {
            if !ids.insert(definition.id.clone()) {
                return Err(format!(
                    "Duplicate launch profile {} in {CONFIG}; give each profile a unique id",
                    definition.id
                ));
            }
            validate(&definition, tree, CONFIG)?;
            let can_debug = definition
                .program
                .as_ref()
                .is_some_and(|p| hick_dap::language_of(std::path::Path::new(p)).is_some());
            found.insert(
                definition.id.clone(),
                Profile {
                    definition,
                    origin: CONFIG.into(),
                    can_debug,
                },
            );
        }
    } else if tree.get(CONFIG).is_some() {
        return Err(format!(
            "{CONFIG} must be a UTF-8 regular file. Replace its symlink or binary contents."
        ));
    }
    let result: Vec<_> = found.into_values().collect();
    if result.iter().filter(|p| p.definition.default).count() > 1 {
        return Err(format!("Choose only one default profile in {CONFIG}"));
    }
    Ok(result)
}
</storycode:copy>
<a id="f-01e001976095-discover"></a>
### `discover`

<storycode:copy id="f-01e001976095-discover">
pub fn discover(tree: &ProjectTree) -> Result<Vec<Profile>, String> {
    discover_with(tree, &[&Applications, &PackageScripts, &PythonTests])
}
</storycode:copy>
<a id="f-01e001976095-select"></a>
### `select`

<storycode:copy id="f-01e001976095-select">
pub fn select(tree: &ProjectTree, id: Option<&str>) -> Result<Profile, String> {
    let profiles = discover(tree)?;
    if let Some(id) = id {
        return profiles.into_iter().find(|p| p.definition.id == id).ok_or_else(|| format!("Launch profile {id} is absent from this project version. Choose another profile."));
    }
    if let Some(default) = profiles.iter().find(|p| p.definition.default) {
        return Ok(default.clone());
    }
    let apps: Vec<_> = profiles.iter().filter(|p| p.can_debug).collect();
    if apps.len() == 1 {
        return Ok(apps[0].clone());
    }
    Err(format!(
        "{} launch targets found. Choose a launch profile, or add one in {CONFIG}.",
        profiles.len()
    ))
}
</storycode:copy>
<a id="f-01e001976095-validate"></a>
### `validate`

<storycode:copy id="f-01e001976095-validate">
fn validate(p: &Definition, tree: &ProjectTree, origin: &str) -> Result<(), String> {
    let fail = |message: &str| format!("Launch profile {} from {origin}: {message}", p.id);
    if p.id.trim().is_empty() || p.name.trim().is_empty() {
        return Err(fail("id and name cannot be empty"));
    }
    if p.program.is_some() != p.command.is_empty() {
        return Err(fail("specify exactly one of program or command"));
    }
    if let Some(path) = &p.program {
        hick_lang::validate_project_path(path).map_err(|e| fail(&e))?;
        if tree.get(path).is_none_or(|e| e.kind != EntryKind::File) {
            return Err(fail(
                "program must name a regular file in this project version",
            ));
        }
    }
    if let Some(path) = &p.artifact {
        hick_lang::validate_project_path(path).map_err(|e| fail(&e))?;
    }
    if p.cwd != "." {
        hick_lang::validate_project_path(&p.cwd).map_err(|e| fail(&e))?;
    }
    if p.command.first().is_some_and(|s| s.is_empty())
        || p.build.first().is_some_and(|s| s.is_empty())
    {
        return Err(fail("command/build executable cannot be empty"));
    }
    if p.args
        .iter()
        .chain(p.command.iter())
        .chain(p.build.iter())
        .any(|s| s.contains('\0'))
    {
        return Err(fail("arguments cannot contain NUL bytes"));
    }
    if p.env.iter().any(|(k, v)| {
        k.is_empty()
            || k.starts_with(char::is_numeric)
            || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || v.contains('\0')
    }) {
        return Err(fail("environment names/values are invalid"));
    }
    Ok(())
}

</storycode:copy>
<a id="f-01e001976095-impl-Profile"></a>
### `impl-Profile`

<storycode:copy id="f-01e001976095-impl-Profile">
impl Profile {
    pub fn run_argv(&self, root: &std::path::Path) -> Result<Vec<String>, String> {
        let d = &self.definition;
        let mut argv = if let Some(artifact) = &d.artifact {
            let artifact = root.join(artifact).to_string_lossy().into_owned();
            if d.program.as_ref().is_some_and(|p| p.ends_with(".cs")) {
                vec!["dotnet".into(), artifact]
            } else {
                vec![artifact]
            }
        } else if let Some(path) = &d.program {
            let absolute = root.join(path).to_string_lossy().into_owned();
            match hick_dap::language_of(std::path::Path::new(path)) {
                Some("python") => vec!["python3".into(), absolute.clone()],
                Some("javascript") => vec!["node".into(), absolute.clone()],
                Some("rust") => vec!["cargo".into(), "run".into(), "--".into()],
                Some("go") => vec!["go".into(), "run".into(), absolute.clone()],
                Some("csharp") => vec!["dotnet".into(), "run".into(), "--".into()],
                _ => {
                    return Err(format!(
                        "No runner for {path}. Configure command in {CONFIG}."
                    ));
                }
            }
        } else {
            d.command.clone()
        };
        argv.extend(d.args.clone());
        Ok(argv)
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/launch_profiles.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/launch_profiles.rs"><storycode:paste select="#f-01e001976095-CONFIG"/><storycode:paste select="#f-01e001976095-Definition"/><storycode:paste select="#f-01e001976095-root"/><storycode:paste select="#f-01e001976095-Profile"/><storycode:paste select="#f-01e001976095-Config"/><storycode:paste select="#f-01e001976095-Provider"/><storycode:paste select="#f-01e001976095-Applications"/><storycode:paste select="#f-01e001976095-PackageScripts"/><storycode:paste select="#f-01e001976095-PythonTests"/><storycode:paste select="#f-01e001976095-profile"/><storycode:paste select="#f-01e001976095-command_profile"/><storycode:paste select="#f-01e001976095-text"/><storycode:paste select="#f-01e001976095-impl-Provider-for-Applications"/><storycode:paste select="#f-01e001976095-impl-Provider-for-PackageScripts"/><storycode:paste select="#f-01e001976095-impl-Provider-for-PythonTests"/><storycode:paste select="#f-01e001976095-discover_with"/><storycode:paste select="#f-01e001976095-discover"/><storycode:paste select="#f-01e001976095-select"/><storycode:paste select="#f-01e001976095-validate"/><storycode:paste select="#f-01e001976095-impl-Profile"/></storycode:file>

<a id="output-97a426cc0869"></a>
## `crates/hickory-cli/src/serve/launch_profiles.rs`

Related contracts: [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-97a426cc0869-Query"></a>
### `Query`

Discovery starts no process. Run uses the same resolved profile as Debug.

<storycode:copy id="f-97a426cc0869-Query">
//! Discovery starts no process. Run uses the same resolved profile as Debug.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use crate::debug_sessions::{ProfileSelection, Registry};
use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Query {
    pub uri: String,
    #[serde(default = "current")]
    pub project: String,
    pub program: Option<String>,
    pub expected_source: Option<String>,
    pub expected_document: Option<String>,
}
</storycode:copy>
<a id="f-97a426cc0869-current"></a>
### `current`

<storycode:copy id="f-97a426cc0869-current">
fn current() -> String {
    "new".into()
}
</storycode:copy>
<a id="f-97a426cc0869-context"></a>
### `context`

<storycode:copy id="f-97a426cc0869-context">
fn context(
    state: &LocalState,
    query: &Query,
) -> anyhow::Result<crate::debug_sessions::LaunchContext> {
    let context = Registry::with_projects(state.lsp.projects(), state.index.root().into())
        .launch_context(
            state.index.root(),
            &query.uri,
            &query.project,
            query.program.as_deref(),
        )?;
    context.check_expected(
        query.expected_source.as_deref(),
        query.expected_document.as_deref(),
    )?;
    Ok(context)
}
</storycode:copy>
<a id="f-97a426cc0869-list"></a>
### `list`

<storycode:copy id="f-97a426cc0869-list">
pub async fn list(
    State(state): State<LocalState>,
    Json(query): Json<Query>,
) -> ApiResult<Json<Value>> {
    let context = context(&state, &query).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    let profiles = context
        .profiles()
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(
        json!({"profiles":profiles,"fingerprint":context.fingerprint(),"viewed":context.viewed,"workspace":state.index.root(),"configuration":crate::launch_profiles::CONFIG}),
    ))
}
</storycode:copy>
<a id="f-97a426cc0869-Run"></a>
### `Run`

<storycode:copy id="f-97a426cc0869-Run">
#[derive(Deserialize)]
pub struct Run {
    #[serde(flatten)]
    pub context: Query,
    pub profile: ProfileSelection,
}
</storycode:copy>
<a id="f-97a426cc0869-run"></a>
### `run`

<storycode:copy id="f-97a426cc0869-run">
pub async fn run(State(state): State<LocalState>, Json(body): Json<Run>) -> ApiResult<Json<Value>> {
    let result = (|| -> anyhow::Result<_> {
        let context = context(&state, &body.context)?;
        let profile = context.selected(&body.profile)?;
        let (root, scratch) = context.prepare()?;
        open_run(&state, &profile, &context.root, &root, scratch, None)
    })();
    result
        .map(|s| Json(json!(s)))
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}

</storycode:copy>
<a id="f-97a426cc0869-open_run"></a>
### `open_run`

A terminal is the single Run mechanism for both profiles and test gutters.

<storycode:copy id="f-97a426cc0869-open_run">
/// A terminal is the single Run mechanism for both profiles and test gutters.
pub(super) fn open_run(
    state: &LocalState,
    profile: &crate::launch_profiles::Profile,
    workspace: &std::path::Path,
    root: &std::path::Path,
    scratch: Option<tempfile::TempDir>,
    title: Option<String>,
) -> anyhow::Result<hick_term::SessionSummary> {
    let cwd = crate::debug_sessions::safe_cwd(root, &profile.definition.cwd)?;
    let mut argv = profile.run_argv(root).map_err(anyhow::Error::msg)?;
    if argv.first().is_some_and(|s| s == "python3")
        && let Some(python) = hick_project_env::python_interpreter(workspace, workspace)
    {
        argv[0] = python.to_string_lossy().into_owned();
    }
    let command = terminal_command(profile, &argv);
    let session = state.terminals.open(hick_term::SessionSpec {
        title: title.unwrap_or_else(|| format!("Run {}", profile.definition.name)),
        cwd,
        argv: command,
        monitor: false,
    })?;
    let summary = session.summary();
    // The captured tree lives until the actual terminal child exits, even
    // if the pane or its originating comparison is closed.
    tokio::spawn(async move {
        let _scratch = scratch;
        loop {
            if session.exit_code().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    Ok(summary)
}

</storycode:copy>
<a id="f-97a426cc0869-terminal_command"></a>
### `terminal_command`

<storycode:copy id="f-97a426cc0869-terminal_command">
fn terminal_command(profile: &crate::launch_profiles::Profile, argv: &[String]) -> Vec<String> {
    let d = &profile.definition;
    #[cfg(not(windows))]
    {
        let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        let command = |args: &[String]| args.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ");
        let mut script = "set -e\n".to_owned();
        for (key, value) in &d.env {
            script.push_str(&format!("export {key}={}\n", quote(value)));
        }
        if !d.build.is_empty() {
            script.push_str(&format!("{}\n", command(&d.build)));
        }
        script.push_str(&format!("exec {}", command(argv)));
        vec!["sh".into(), "-c".into(), script]
    }
    #[cfg(windows)]
    {
        let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
        let command = |args: &[String]| {
            format!(
                "& {}",
                args.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ")
            )
        };
        let mut script = "$ErrorActionPreference='Stop';\n".to_owned();
        for (key, value) in &d.env {
            script.push_str(&format!("$env:{key}={};\n", quote(value)));
        }
        if !d.build.is_empty() {
            script.push_str(&format!(
                "{}; if ($LASTEXITCODE -ne 0) {{ exit $LASTEXITCODE }};\n",
                command(&d.build)
            ));
        }
        script.push_str(&format!("{}; exit $LASTEXITCODE", command(argv)));
        vec![
            "powershell.exe".into(),
            "-NoProfile".into(),
            "-Command".into(),
            script,
        ]
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/launch_profiles.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/launch_profiles.rs"><storycode:paste select="#f-97a426cc0869-Query"/><storycode:paste select="#f-97a426cc0869-current"/><storycode:paste select="#f-97a426cc0869-context"/><storycode:paste select="#f-97a426cc0869-list"/><storycode:paste select="#f-97a426cc0869-Run"/><storycode:paste select="#f-97a426cc0869-run"/><storycode:paste select="#f-97a426cc0869-open_run"/><storycode:paste select="#f-97a426cc0869-terminal_command"/></storycode:file>

<a id="output-7149d905428b"></a>
## `crates/hickory-cli/tests/launch_profiles.rs`

Related contracts: [Launch profiles choose targets within the viewed project](contract.md).

<a id="f-7149d905428b-tree"></a>
### `tree`

<storycode:copy id="f-7149d905428b-tree">
use hick_lang::{ProjectEntry, ProjectTree};
use hickory_cli::launch_profiles::{self, Profile, Provider};

fn tree(files: &[(&str, &str)]) -> ProjectTree {
    let mut tree = ProjectTree::default();
    for (path, text) in files {
        tree.insert(path, ProjectEntry::file(text.as_bytes()))
            .unwrap();
    }
    tree
}
</storycode:copy>
<a id="f-7149d905428b-library_discovery_selects_the_calling_application_without_running_project_code"></a>
### `library_discovery_selects_the_calling_application_without_running_project_code`

<storycode:copy id="f-7149d905428b-library_discovery_selects_the_calling_application_without_running_project_code">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn library_discovery_selects_the_calling_application_without_running_project_code() {
    let tree = tree(&[
        ("app.py", "from library import greet\nprint(greet())\n"),
        ("library.py", "def greet():\n    return 'Hello'\n"),
    ]);
    let profiles = launch_profiles::discover(&tree).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(
        launch_profiles::select(&tree, None)
            .unwrap()
            .definition
            .program
            .as_deref(),
        Some("app.py")
    );
}
</storycode:copy>
<a id="f-7149d905428b-manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries"></a>
### `manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries`

<storycode:copy id="f-7149d905428b-manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries() {
    let source = r#"{"version":1,"profiles":[{"id":"app:app.py","name":"Greeting Ada","program":"app.py","args":["Ada Lovelace","$(touch no)","'quoted'"],"env":{"GREETING":"Hello"},"default":true}]}"#;
    let tree = tree(&[
        ("app.py", "print('hello')"),
        (launch_profiles::CONFIG, source),
    ]);
    let profile = launch_profiles::select(&tree, None).unwrap();
    assert_eq!(launch_profiles::discover(&tree).unwrap().len(), 1);
    assert_eq!(profile.origin, launch_profiles::CONFIG);
    assert_eq!(
        profile.run_argv(std::path::Path::new("/project")).unwrap(),
        vec![
            "python3",
            "/project/app.py",
            "Ada Lovelace",
            "$(touch no)",
            "'quoted'"
        ]
    );
    assert_eq!(profile.definition.env["GREETING"], "Hello");
}
</storycode:copy>
<a id="f-7149d905428b-ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored"></a>
### `ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored`

<storycode:copy id="f-7149d905428b-ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored() {
    let mut tree = tree(&[("a.py", "print(1)"), ("b.py", "print(2)")]);
    assert!(
        launch_profiles::select(&tree, None)
            .unwrap_err()
            .contains("Choose a launch profile")
    );
    assert_eq!(
        launch_profiles::select(&tree, Some("app:b.py"))
            .unwrap()
            .definition
            .program
            .as_deref(),
        Some("b.py")
    );
    tree.insert(
        launch_profiles::CONFIG,
        ProjectEntry::file(
            br#"{"version":1,"profiles":[{"id":"x","name":"bad","program":"../outside.py"}]}"#
                .as_slice(),
        ),
    )
    .unwrap();
    assert!(
        launch_profiles::discover(&tree)
            .unwrap_err()
            .contains(launch_profiles::CONFIG)
    );
    tree.insert(
        launch_profiles::CONFIG,
        ProjectEntry::file(br#"{"version":1,"profiles":[],"unknown":true}"#.as_slice()),
    )
    .unwrap();
    assert!(
        launch_profiles::discover(&tree)
            .unwrap_err()
            .contains("unknown")
    );
}
</storycode:copy>
<a id="f-7149d905428b-manifest_scripts_and_test_definitions_become_run_only_targets"></a>
### `manifest_scripts_and_test_definitions_become_run_only_targets`

<storycode:copy id="f-7149d905428b-manifest_scripts_and_test_definitions_become_run_only_targets">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn manifest_scripts_and_test_definitions_become_run_only_targets() {
    let tree = tree(&[
        (
            "web/package.json",
            r#"{"scripts":{"dev":"vite","test":"vitest"}}"#,
        ),
        ("test_greeting.py", "def test_greet():\n    assert True\n"),
    ]);
    let profiles = launch_profiles::discover(&tree).unwrap();
    assert_eq!(profiles.len(), 3);
    assert!(profiles.iter().all(|p| !p.can_debug));
    assert_eq!(
        profiles
            .iter()
            .find(|p| p.definition.id == "npm:web/package.json:dev")
            .unwrap()
            .definition
            .cwd,
        "web"
    );
    assert!(profiles.iter().any(
        |p| p.definition.command == ["python3", "-m", "pytest", "test_greeting.py::test_greet"]
    ));
}
</storycode:copy>
<a id="f-7149d905428b-ExampleProvider"></a>
### `ExampleProvider`

<storycode:copy id="f-7149d905428b-ExampleProvider">
struct ExampleProvider;
</storycode:copy>
<a id="f-7149d905428b-impl-Provider-for-ExampleProvider"></a>
### `impl-Provider for ExampleProvider`

<storycode:copy id="f-7149d905428b-impl-Provider-for-ExampleProvider">
impl Provider for ExampleProvider {
    fn discover(&self, _: &ProjectTree) -> Result<Vec<Profile>, String> {
        Ok(vec![launch_profiles::command_profile(
            "custom".into(),
            "Provider target".into(),
            vec!["tool".into(), "test".into()],
            ".".into(),
            "example provider",
        )])
    }
}
</storycode:copy>
<a id="f-7149d905428b-provider_extension_uses_the_same_validated_manual_merge"></a>
### `provider_extension_uses_the_same_validated_manual_merge`

<storycode:copy id="f-7149d905428b-provider_extension_uses_the_same_validated_manual_merge">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn provider_extension_uses_the_same_validated_manual_merge() {
    let discovered =
        launch_profiles::discover_with(&ProjectTree::default(), &[&ExampleProvider]).unwrap();
    assert_eq!(discovered[0].definition.command, ["tool", "test"]);
    let configured = tree(&[(
        ".hick-launch.json",
        r#"{"version":1,"profiles":[{"id":"custom","name":"Manual target","command":["other"]}]}"#,
    )]);
    let merged = launch_profiles::discover_with(&configured, &[&ExampleProvider]).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].definition.name, "Manual target");
    assert_eq!(merged[0].definition.command, ["other"]);
}
</storycode:copy>
<a id="f-7149d905428b-launch_selection_is_frozen_to_the_listed_project"></a>
### `launch_selection_is_frozen_to_the_listed_project`

<storycode:copy id="f-7149d905428b-launch_selection_is_frozen_to_the_listed_project">
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn launch_selection_is_frozen_to_the_listed_project() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("app.py"), "print(1)\n").unwrap();
    let registry = hickory_cli::debug_sessions::Registry::new();
    let context = registry
        .launch_context(temp.path(), "hick:///app.py", "new", None)
        .unwrap();
    let selection = hickory_cli::debug_sessions::ProfileSelection {
        id: "app:app.py".into(),
        fingerprint: context.fingerprint(),
    };
    assert_eq!(
        context.selected(&selection).unwrap().definition.name,
        "app.py"
    );
    std::fs::write(temp.path().join("app.py"), "print(2)\n").unwrap();
    let changed = registry
        .launch_context(temp.path(), "hick:///app.py", "new", None)
        .unwrap();
    assert!(
        changed
            .selected(&selection)
            .unwrap_err()
            .to_string()
            .contains("project changed")
    );
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/launch_profiles.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/launch_profiles.rs"><storycode:paste select="#f-7149d905428b-tree"/><storycode:paste select="#f-7149d905428b-library_discovery_selects_the_calling_application_without_running_project_code"/><storycode:paste select="#f-7149d905428b-manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries"/><storycode:paste select="#f-7149d905428b-ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored"/><storycode:paste select="#f-7149d905428b-manifest_scripts_and_test_definitions_become_run_only_targets"/><storycode:paste select="#f-7149d905428b-ExampleProvider"/><storycode:paste select="#f-7149d905428b-impl-Provider-for-ExampleProvider"/><storycode:paste select="#f-7149d905428b-provider_extension_uses_the_same_validated_manual_merge"/><storycode:paste select="#f-7149d905428b-launch_selection_is_frozen_to_the_listed_project"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/breakpointPreferences.ts](../../../mechanisms/apps/web/src/debug/breakpointPreferences.ts.md#output-9ef53203ebd0)
- [apps/web/src/debug/launchProfiles.tsx](../../../mechanisms/apps/web/src/debug/launchProfiles.tsx.md#output-1a7a9a03aa8d)
- [apps/web/src/debug/plainDebugHosts.tsx](contract.md#output-238f66c701b0)
- [apps/web/src/debug/projectDebugSurface.tsx](contract.md#output-75fab5522700)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [crates/hick-lang/src/project/evaluate.rs](../../../mechanisms/crates/hick-lang/src/project/evaluate.rs.md#output-4957ad4b2f9a)
- [crates/hickory-cli/src/debug_profiles.rs](contract.md#output-05e2104e1cc1)
- [crates/hickory-cli/src/launch_profiles.rs](contract.md#output-01e001976095)
- [crates/hickory-cli/src/serve/launch_profiles.rs](contract.md#output-97a426cc0869)

### Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../mechanisms/apps/web/e2e/literate-editor.spec.ts.md#output-648693644a28)
- [apps/web/src/debug/launchProfiles.test.tsx](../../../mechanisms/apps/web/src/debug/launchProfiles.tsx.md#output-af7d3cd605b8)
- [apps/web/src/debug/plainDebugHosts.test.tsx](contract.md#output-3b3668793103)
- [apps/web/src/debug/projectDebugSurface.test.tsx](../../../mechanisms/apps/web/src/debug/projectDebugSurface.test.tsx.md#output-777a3021cd65)
- [crates/hick-lang/tests/project_debug.rs](../../../mechanisms/crates/hick-lang/tests/project_debug.rs.md#output-3d2aa0516f40)
- [crates/hickory-cli/tests/launch_profiles.rs](contract.md#output-7149d905428b)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
