<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/plainDebugHosts.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Plain File Has The Same Debugger A Document Has](../../../../../../a-plain-file-has-the-same-debugger/contract.md)
- [Launch profiles choose targets within the viewed project](../../../../../contract.md)

Captured SHA-256: `fbd0ce1f92f8ad4cc0d4afeb24ea6dd865c7fa5bcc7f2957fb00609213ce6412`.

<book:entry path="apps/web/src/debug/plainDebugHosts.tsx">
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

// The paths that want a host, and the session each host currently holds.
let wanted: readonly string[] = [];
const sessions = new Map<string, DebugSession>();
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Ask for a host for `path`. Idempotent; hosts are never removed. */
export function ensurePlainDebugHost(path: string): void {
  if (wanted.includes(path)) return;
  wanted = [...wanted, path];
  emit();
}

function wantedPaths(): readonly string[] {
  return wanted;
}

/** The session for `path`, or null until its host has mounted. */
export function usePlainDebugSession(path: string): DebugSession | null {
  useEffect(() => { if (path) ensurePlainDebugHost(path); }, [path]);
  return useSyncExternalStore(
    subscribe,
    () => sessions.get(path) ?? null,
    () => null,
  );
}

/** Test seam. */
export function resetPlainDebugHosts(): void {
  wanted = [];
  sessions.clear();
  emit();
}

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

/** Let document-owned sessions participate in the same cross-file controls. */
export function usePublishDebugSession(owner: string, debug: DebugSession) {
  useEffect(() => { if (owner) { sessions.set(owner, debug); emit(); } }, [owner, debug]);
  useEffect(() => () => { if (owner) { sessions.delete(owner); emit(); } }, [owner]);
}

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
</book:entry>

</book:doc>
