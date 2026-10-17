import { useEffect, useRef, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { request } from "../api/client";
import { showTerminalRequest } from "../lib/revealLine";

export interface LaunchSelection { id: string; fingerprint: string; name?: string }
export interface LaunchQuery { uri: string; project?: string; program?: string; expected_source?: string; expected_document?: string }
export interface LaunchProfile {
  id: string; name: string; origin: string; program: string | null; command: string[];
  args: string[]; env: Record<string,string>; cwd: string; build: string[]; default: boolean; can_debug: boolean;
}
interface Catalog { profiles: LaunchProfile[]; fingerprint: string; viewed: string; workspace: string; configuration: string }
type Choice = { profile: LaunchProfile; mode: "run" | "debug" };
interface Picker { notice?: string; contextLabel?: string; catalog?: Catalog; error?: string; settle: (value: Choice | null) => void; anchor: DOMRect | null }
let picker: Picker | null = null;
const listeners = new Set<() => void>();
const emit = () => { for (const listener of listeners) listener(); };
const subscribe = (listener: () => void) => { listeners.add(listener); return () => {listeners.delete(listener);}; };
const remembered = new Map<string,string>();
const pending = new Set<string>();
function choose(catalog?: Catalog, error?: string, contextLabel?: string, notice?:string): Promise<Choice | null> {
  picker?.settle(null);
  const active = document.activeElement;
  const anchor = active instanceof HTMLElement ? active.getBoundingClientRect() : null;
  return new Promise(resolve => {
    const current: Picker = {catalog,error,anchor,contextLabel,notice,settle:value=>{if(picker===current){picker=null;emit();}resolve(value);}};
    picker=current;emit();
  });
}

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

/** Workspace overlay: it never inserts editor rows or creates a second debugger. */
export function LaunchProfiles() {
  const state = useSyncExternalStore(subscribe,()=>picker,()=>null);
  const panel=useRef<HTMLDivElement>(null);
  useEffect(()=>{ if(state)panel.current?.querySelector<HTMLButtonElement>("button")?.focus(); },[state]);
  useEffect(()=>()=>{picker?.settle(null);},[]);
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
