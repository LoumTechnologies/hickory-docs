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

import { openCapturedLocation } from "../lib/revealLine";
import { openCapturedCode } from "../lib/capturedCode";
import { retainCapturedContext } from "../lib/capturedContexts";
import { publishPausedElsewhere, usePausedElsewhere } from "../lib/pausedElsewhere";
import type { DebugSession } from "./useDebugger";
import { useWorkspaceDebugger } from "./useDebugger";
import {savedDebugSources,type DebugSourceCapability} from "./SourceGuards";
import {documentUri,capturedDocumentUri} from "../lsp/documentUri";

/** Whether a frame's source is a file in the folder the app opened. */
export function isWorkspaceSource(source: string | null | undefined): source is string {
  return !!source && !/^[A-Za-z][A-Za-z0-9+.-]*:/.test(source) && !/^([A-Za-z]:)?[\\/]/.test(source);
}

// The paths that want a host, and the session each host currently holds.
interface PlainHost {readonly id:string;readonly path:string;readonly identity?:object;launchCurrent?:()=>boolean;source?:DebugSourceCapability}
let wanted: readonly PlainHost[] = [];
let bufferHosts=new WeakMap<object,PlainHost>();
const liveDebug=(debug:DebugSession|undefined|null)=>!!debug&&(debug.isActive?.()??["starting","running","paused","unconfirmed","stopping"].includes(debug.status));
/** The retained buffer owns its running launch, even when its current address changes. */
export function plainDebugOwner(path:string,identity?:object):PlainHost {
 if(!identity)return {id:path,path};
 const held=bufferHosts.get(identity);
 if(held&&(held.path===path||liveDebug(sessions.get(held.id))))return held;
 const next={id:`plain-buffer:${crypto.randomUUID()}`,path,identity};bufferHosts.set(identity,next);return next;
}
export function hasActivePlainDebugSession(identity:object):boolean {const owner=bufferHosts.get(identity);return !!owner&&liveDebug(sessions.get(owner.id));}
/** Explicit copied-source retirement revokes its idle UI actor, never a live launch. */
export function retirePlainDebugIdentity(identity:object):boolean {
 const ids=new Set(wanted.filter(host=>host.identity===identity).map(host=>host.id)),owner=bufferHosts.get(identity);if(owner)ids.add(owner.id);
 if([...ids].some(id=>liveDebug(sessions.get(id))))return false;
 bufferHosts.delete(identity);wanted=wanted.filter(host=>!ids.has(host.id));for(const id of ids){sessions.delete(id);publishPausedElsewhere(id,null);clearPreferredDebugSurface(id);}emit();return true;
}
const sessions = new Map<string, DebugSession>();
const listeners = new Set<() => void>();
let sessionRevision=0;

function emit() {
  sessionRevision++;
  for (const listener of listeners) listener();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Reading adapters retain an owner id, never look up a later session by spelling. */
export const subscribePlainDebugSessions=(listener:()=>void)=>subscribe(listener);
export const plainDebugSessionsSnapshot=()=>sessionRevision;
export function readPlainDebugOwner(owner:string):DebugSession|null{return sessions.get(owner)??null;}
export function readPlainDebugBuffer(identity:object):{owner:string;debug:DebugSession}|null {const host=bufferHosts.get(identity),debug=host&&sessions.get(host.id);return host&&debug?{owner:host.id,debug}:null;}

/** Ask for a host for `path`. Idempotent; hosts are never removed. */
export function ensurePlainDebugHost(value: string|PlainHost): void {
  const host=typeof value==='string'?{id:value,path:value}:value;
  if (wanted.some(held=>held.id===host.id)) return;
  wanted = [...wanted, host];
  emit();
}

function wantedPaths(): readonly PlainHost[] {
  return wanted;
}

/** The session for `path`, or null until its host has mounted. */
export function usePlainDebugSession(path: string,identity?:object,launchCurrent?:()=>boolean,source?:DebugSourceCapability): DebugSession | null {
  const owner=plainDebugOwner(path,identity);
  if(source)owner.source=source;
  if(identity)owner.launchCurrent=()=>owner.path===path&&(launchCurrent?.()??true);
  useEffect(() => { if (path) ensurePlainDebugHost(owner); }, [owner.id,owner.path]);
  return useSyncExternalStore(
    subscribe,
    () => sessions.get(owner.id) ?? null,
    () => null,
  );
}

/** Test seam. */
export function resetPlainDebugHosts(): void {
  wanted = [];bufferHosts=new WeakMap();
  sessions.clear();
  emit();
}

function Host({ host }: {host:PlainHost}) {
  const {path,id}=host;
  const access=useMemo<DebugSourceCapability>(()=>({read:uri=>{
    if(uri===documentUri(path)&&host.source)return host.source.read(uri);
    const held=wanted.find(item=>documentUri(item.path)===uri&&item.source);return held?.source?.read(uri)??savedDebugSources.read(uri);
  },subscribe:listener=>host.source?.subscribe?.(listener)??(()=>{})}),[host,path]);
  const debug = useWorkspaceDebugger(path,id,()=>host.launchCurrent?.()??true,access);
  useEffect(() => {
    sessions.set(id, debug);
    emit();
  }, [id, debug]);

  const active = ["starting", "running", "paused", "unconfirmed", "stopping"].includes(debug.status);
  useEffect(() => {
    if (active) return retainCapturedContext(path);
  }, [path, active]);

  // Where this session is paused when that is another file of the folder:
  // say so, and open that file, the way stepping into a callee in another
  // file opens it in any IDE. Cleared the moment the program moves on or
  // the session ends; only this owner can clear what it published.
  useEffect(() => {
    const top = debug.frames[0],captured=top&&debug.captureFrame?.(top.id);
    if (["idle", "finished", "failed"].includes(debug.status)) clearPreferredDebugSurface(id);
    if (debug.status === "paused" && top && captured?.current() && revealPreferredDebugLocation(id, top)) {
      publishPausedElsewhere(id, top.source && typeof top.source_line === "number" ? {path:top.source,line:top.source_line} : null);
      return;
    }
    if (
      debug.status === "paused" &&
      top &&captured?.current()&&
      !top.in_document &&
      (top.source?.startsWith("hick-project:///") || isWorkspaceSource(top.source)) &&
      typeof top.source_line === "number"
    ) {
      publishPausedElsewhere(id, { path: top.source!, line: top.source_line });
      if (top.source!.startsWith("hick-project:///")) {
        void openCapturedCode({uri:top.source!,range:{start:{line:top.source_line,character:0},end:{line:top.source_line,character:0}}}, id,captured.current).catch(() => {});
      } else openCapturedLocation(top.source!,top.source_line+1,top.source_hash??null,captured.current,top.source_reference);
    } else {
      publishPausedElsewhere(id, null);
    }
  }, [debug.status, debug.frames, path,id]);
  return <ProjectDebugSurface owner={id} debug={debug}/>;
}

/** Mount once, in the workspace: one host per path a pane has asked for. */
export function PlainDebugHosts() {
  const paths = useSyncExternalStore(subscribe, wantedPaths, wantedPaths);
  return (
    <>
      <LaunchProfiles />
      {paths.map((host) => (
        <Host key={host.id} host={host} />
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
export function useFollowingDebugSession(path: string, explicitOwner?: string,identity?:object,launchCurrent?:()=>boolean,sourceCapability?:DebugSourceCapability): DebugSession | null {
  const own = usePlainDebugSession(path,identity,launchCurrent,sourceCapability);
  const ownId=plainDebugOwner(path,identity).id;
  const key=capturedSourceKey(path);
  const saved=useSyncExternalStore(subscribeBreakpointPreferences,()=>breakpointPreferences(key),()=>undefined);
  const ownRef=useRef(own);ownRef.current=own;
  const ownActive=liveDebug(own);
  useEffect(()=>{if(saved && ownRef.current && !ownActive)ownRef.current.replaceBreakpoints(saved,key??undefined);},[saved,key,!!own,ownActive]);
  const elsewhere = usePausedElsewhere();
  const followed = useRef({path, owner:explicitOwner});
  if (followed.current.path !== path) followed.current = {path, owner:explicitOwner};
  if (explicitOwner) followed.current.owner = explicitOwner;
  if (!ownActive&&elsewhere?.path === path) followed.current.owner = elsewhere.owner;
  const borrowed = useSyncExternalStore(subscribe, () => sessions.get(followed.current.owner ?? "") ?? null, () => null);
  const source = documentUri(path);
  const active = borrowed && ["paused","running"].includes(borrowed.status) && followed.current.owner !== path;
  useEffect(()=> {
    if (!active || !own || !borrowed) return;
    const held = borrowed.sourceBreakpoints[source];
    if (held && held !== own.breakpoints) own.replaceBreakpoints(held);
    else if (!held && own.breakpoints.length) borrowed.setSourceBreakpoints(source,own.breakpoints);
  },[active,borrowed,source,own]);
  const direct = useMemo(()=>own ? {...own,start:(program?:string, sources: Record<string, import("./useDebugger").HeldBreakpoint[]> = {}, project?: string, profile?: import("./launchProfiles").LaunchSelection)=> {
    const captured = /^hick-project:\/\/\/[^/]+\//.exec(path)?.[0];
    const initial = Object.fromEntries([...sessions].filter(([owner])=>owner!==ownId).map(([owner,session])=>[wanted.find(host=>host.id===owner)?.path??owner,session] as const).filter(([other])=>other!==path &&
      isDebuggable(languageFromPath(other)) && (captured ? other.startsWith(captured) : isWorkspaceSource(other)))
      .map(([other,session])=>[documentUri(other),session.breakpoints]));
    if(captured)for(const file of capturedFiles(path)) {
      const uri=capturedDocumentUri(captured,file), held=breakpointPreferences(capturedSourceKey(uri));
      if(uri!==path && held)initial[uri]=held;
    }
    // Only model-captured intentions can cross controller boundaries. A legacy
    // path/line preference remains readable but cannot borrow this launch's bytes.
    const origins=Object.assign({},...[...sessions].filter(([owner])=>owner!==ownId).map(([,session])=>session.capturedBreakpointIntents?.()??{}));
    own.start(program,{...initial,...sources},project,profile,origins);
  }} : null,[own,path,ownId]);
  if (ownActive||!borrowed || !["starting", "running", "paused", "unconfirmed", "stopping"].includes(borrowed.status) || followed.current.owner === ownId) return preferences(direct,key);
  const frame = borrowed.frames.find(f => f.source === path);
  const pausedLine = frame?.source_line ?? (elsewhere?.path === path ? elsewhere.line : null);
  const breakpoints = borrowed.sourceBreakpoints[source] ?? own?.breakpoints ?? [];
  return preferences({ ...borrowed, pausedLine, program: borrowed.program ?? followed.current.owner ?? null,
    // These protocol verbs address the launch file, not the viewed callee.
    // Never send a callee's line number as a line in the launch file.
    capabilities: borrowed.capabilities ? {...borrowed.capabilities, goto_targets:false,run_to:false} : null,
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
