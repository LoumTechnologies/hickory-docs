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
const comparisons=new Map<string,Set<ComparisonSurface>>();
export function registerCapturedComparison(uri:string,view:EditorView,source:string,outer?:EditorView):()=>void {
  const entry={view,source,outer}, all=comparisons.get(uri) ?? new Set<ComparisonSurface>();
  all.add(entry);comparisons.set(uri,all);
  return ()=>{all.delete(entry);if(!all.size)comparisons.delete(uri);};
}
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

interface Surface { view: EditorView; entry: string; source: string; path?: string; codeView?: EditorView | null; codeUri?: string; codeText?: string; frame?: Frame }
const locations = new Map<string, Surface>();
const listeners = new Set<() => void>();
let version = 0;
const subscribe = (fn: () => void) => { listeners.add(fn); return () => { listeners.delete(fn); }; };
const emit = () => { version++; for (const listener of listeners) listener(); };
function useSurface(owner: string) {
  useSyncExternalStore(subscribe, () => version, () => 0);
  return locations.get(owner);
}
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

/** Rebind a returning note to the still-live captured debugger. */
export function bindProjectDebugView(path: string, view: EditorView) {
  for (const [owner,surface] of locations) if (surface.path === path && view.state.doc.toString() === surface.source) {
    surface.view = view;
    if (surface.frame) revealPreferredDebugLocation(owner,surface.frame);
  }
  emit();
}

/** Both locations address one paused frame and keep the same live controls. */
export function ProjectDebugLinks({owner, debug}: {owner:string; debug:DebugSession}) {
  const [error,setError] = useState("");
  const surface = useSurface(owner);
  const frame = debug.frames.find(f => f.id === debug.selectedFrame) ?? debug.frames[0];
  if (!surface || debug.status !== "paused" || !frame) return null;
  return <span className="project-debug-links">
    <button className="debug-icon" type="button" aria-label="Show literate source" data-tip="Show literate source" onClick={() => revealPreferredDebugLocation(owner,frame)}>¶</button>
    <button className="debug-icon" type="button" aria-label="Show generated code" data-tip="Show generated code" onClick={() => {
      if (frame.source?.startsWith("hick-project:///")) void openCapturedCode({uri:frame.source,range:{start:{line:frame.source_line ?? 0,character:0},end:{line:frame.source_line ?? 0,character:0}}},owner).catch(e => setError(String(e)));
    }}>{"{}"}</button>
    {error && <span role="alert">Could not open generated code: {error}</span>}
  </span>;
}

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
