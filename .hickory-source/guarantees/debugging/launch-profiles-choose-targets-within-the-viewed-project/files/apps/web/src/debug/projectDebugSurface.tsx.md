<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/projectDebugSurface.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Launch profiles choose targets within the viewed project](../../../../../contract.md)

Captured SHA-256: `f6699b7b305f0c95d86bf697b85879290512f3016bba8f72444bee7bc15c2d55`.

<book:entry path="apps/web/src/debug/projectDebugSurface.tsx">
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
</book:entry>

</book:doc>
