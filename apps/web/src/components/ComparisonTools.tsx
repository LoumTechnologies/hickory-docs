import { revealPreferredDebugLocation } from "../debug/preferredDebugLocation";
import { preferCapturedLiterateSurface, ProjectDebugLinks } from "../debug/projectDebugSurface";
import { PromptPanel, usePrompt } from "./PromptPanel";
import { completionSource, completionSources } from "../lsp/completion";
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { api } from "../api/client";
import { projectPosition, type ProjectEntry } from "../editor/hickLang";
import type { ComparisonSource } from "../lib/projectComparison";
import { openCapturedCode } from "../lib/capturedCode";
import { acquireCapturedContext, retainCapturedContext } from "../lib/capturedContexts";
import { useFollowingDebugSession } from "../debug/plainDebugHosts";
import { IDLE_SESSION } from "../debug/useDebugger";
import { DebugLaunchAction } from "../debug/DebugRail";
import { revealLine } from "../debug/cmDebug";
import { isDebuggable } from "../debug/languages";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport, setLspDiagnostics, diagnosticRanges } from "../lsp/cmLsp";
import { debugEditor, debugStateEffects } from "../debug/cmDebug";
import { DebugStrip } from "../debug/DebugStrip";
import { languageFromPath } from "../editor/languages";

const decode = (entry: ProjectEntry) => new TextDecoder("utf-8", {fatal:true}).decode(new Uint8Array(entry.bytes));
function offset(text: string, line: number, column: number): number | null {
  const lines = text.split("\n");
  if (line >= lines.length || column > lines[line].length) return null;
  return lines.slice(0,line).reduce((sum,s) => sum+s.length+1,0)+column;
}
function position(bytes: number[], byte: number) {
  const prefix = new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(bytes.slice(0,byte)));
  const lines = prefix.split("\n"); return {line:lines.length-1,character:lines.at(-1)!.length};
}
export { openCapturedCode } from "../lib/capturedCode";

/** Opening the view registers its isolated snapshot; native tools use only that captured project. */
export function ComparisonTools({path,source,view,children}: {path:string;source:ComparisonSource;view:EditorView|null;children?: (debug: typeof IDLE_SESSION | null) => ReactNode}) {
  const prompt = usePrompt();
  const [registration,setContext] = useState<{id:string;uri:string;project:ProjectEntry[]}|null>(null);
  const [references,setReferences] = useState<import("../lsp/client").LspLocation[] | null>(null);
  const [error,setError] = useState("");
  const [busy,setBusy] = useState(false);
  const [retry,setRetry] = useState(0);
  const tools = useMemo(() => new Compartment(),[]);
  const local = source.local ?? source.project;
  const selected = source.project;
  const displayed = local?.find(e => e.path === path);
  let activePath = selected?.find(e => e.path === path)?.path;
  if (!activePath && displayed?.origins?.length && selected && local) {
    try { activePath = projectPosition(local,selected,path,displayed.origins[0].output[0])?.[0]; } catch { /* explicit absent-code status */ }
  }
  const active = selected?.find(e => e.path === activePath && e.kind === "file");
  const displayMatches = !!displayed && decode(displayed) === source.text;
  const context = registration?.project === selected && displayMatches ? registration : null;
  const uri = context && active ? `${context.uri}${active.path}` : "";
  const lsp = useWorkspaceLsp(uri,active ? decode(active) : "");
  const debug = useFollowingDebugSession(uri, source.debugOwner) ?? IDLE_SESSION;
  const live = useRef({source,local,selected,context,debug}); live.current={source,local,selected,context,debug};
  const exact = !!active && !!displayed && active.path === path && decode(active) === source.text;
  useEffect(() => {
    let live = true;
    let release: (() => void) | undefined;
    setContext(null); setError("");
    if (!selected || !active || !displayMatches) { setBusy(false); return; }
    if (source.codeUri) {
      const match = /^hick-project:\/\/\/([^/]+)\//.exec(source.codeUri);
      if (match) {
        release = retainCapturedContext(source.codeUri);
        setContext({id:match[1],uri:match[0],project:selected}); setBusy(false);
        return release;
      }
    }
    setBusy(true);
    const entries = Object.fromEntries(selected.map(e => [e.path, { bytes: e.bytes, kind: e.kind === "file" ? "File" : "Symlink", executable: e.executable, origins: e.origins ?? [] }]));
    void acquireCapturedContext(entries).then(next => {
      const dispose = next.release;
      if (!live) { dispose(); return; }
      release = dispose;
      setContext({ ...next, project: selected });
    }, e => { if (live) setError(e instanceof Error ? e.message : String(e)); })
      .finally(() => { if (live) setBusy(false); });
    return () => { live = false; release?.(); };
  }, [selected, active, displayMatches, retry, source.codeUri]);
  useEffect(() => {
    if (!view || !context || !local || !selected || !displayMatches) return;
    const targetAt = (at: number, v: EditorView) => {
        const current=live.current;
        if (v.state.doc.toString() !== current.source.text || !current.local || !current.selected || !current.context) return null;
        try {
          const target=projectPosition(current.local,current.selected,path,new TextEncoder().encode(current.source.text.slice(0,at)).length);
          if (!target) return null;
          return {uri:`${current.context.uri}${target[0]}`,position:position(current.selected.find(e=>e.path===target[0])!.bytes,target[1])};
        } catch { return null; }
      };
    const extensions = [
      ...lspSupport({client:lsp.client,uri:lsp.uri,positionAt:() => null,targetAt,
      onNavigate:target => {
        if (target.uri === uri && active) {
          const at = offset(decode(active),target.range.start.line,target.range.start.character);
          if (at !== null) {
            try {
              const localTarget = projectPosition(selected,local,active.path,new TextEncoder().encode(decode(active).slice(0,at)).length);
              if (localTarget?.[0] === path) {
                const anchor = new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(displayed!.bytes.slice(0,localTarget[1]))).length;
                view.dispatch({selection:{anchor},scrollIntoView:true}); view.focus(); return;
              }
            } catch { /* Open the exact captured source when no local mapping exists. */ }
          }
        }
        void openCapturedCode(target, source.debugOwner ?? uri).catch(e=>setError(String(e)));
      },
      onReferences:locations => setReferences(locations),
      runtimeValue:word => exact ? live.current.debug.valueAt(word) : Promise.resolve(null),
    }),
      completionSources.of(async completion => {
        if (!lsp.client || completion.state !== view.state) return null;
        const target = targetAt(completion.pos, view);
        if (!target) return null;
        const captured = live.current.context;
        const result = await completionSource({ lsp: { client: lsp.client, uri: target.uri, positionAt: () => target.position } })(completion);
        return captured === live.current.context && completion.state === view.state ? result : null;
      }),
    ];
    if (exact) extensions.push(...debugEditor({language:languageFromPath(path),onToggleBreakpoint:line=>live.current.debug.toggleBreakpoint(line),onSelectFrame:id=>live.current.debug.selectFrame(id),onEvaluate:expression=>live.current.debug.query(expression),onAddWatch:expression=>live.current.debug.addWatch(expression),onSetBreakpointCondition:(line,patch)=>live.current.debug.setBreakpointCondition(line,patch)}));
    view.dispatch({effects:tools.get(view.state) === undefined ? StateEffect.appendConfig.of(tools.of(extensions)) : tools.reconfigure(extensions)});
    return () => { if (view.dom.isConnected) view.dispatch({effects:tools.reconfigure([])}); };
  },[view,context,lsp.client,lsp.uri,path,exact,tools,local,selected,displayMatches]);
  useEffect(() => { if (view && context && exact) view.dispatch({effects:debugStateEffects(debug)}); },[view,context,exact,debug]);
  useEffect(() => {
    if (!view || !active || !local || !selected || !context || !displayMatches) { view?.dispatch({effects:setLspDiagnostics.of([])}); return; }
    const spans=diagnosticRanges(lsp.diagnostics,p=> {
      const at=offset(decode(active),p.line,p.character); if (at === null) return null;
      try {
        const target=projectPosition(selected,local,active.path,new TextEncoder().encode(decode(active).slice(0,at)).length);
        return target?.[0] === path ? new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(displayed!.bytes.slice(0,target[1]))).length : null;
      } catch { return null; }
    });
    view.dispatch({effects:setLspDiagnostics.of(spans)});
  },[view,lsp.diagnostics,context,active,local,selected,path,displayed,displayMatches]);
  const launch = () => {
    const surface = source.debugSurface;
    if (surface && selected && context) {
      preferCapturedLiterateSurface(uri, context.uri, selected, surface);
    }
    debug.start();
    if (!exact) void openCapturedCode({uri,range:{start:{line:0,character:0},end:{line:0,character:0}}}, uri).catch(e=>setError(String(e)));
  };
  const canDebug = !!active && isDebuggable(languageFromPath(active.path));
  const selectedFrame = debug.frames.find(frame => frame.id === debug.selectedFrame);
  useSelectedFrame(view, exact ? (selectedFrame?.line ?? debug.pausedLine) : null);
  return <>
    {children?.(context && exact && canDebug ? debug : null)}
    {canDebug && <DebugLaunchAction view={view} label={`Debug ${path} · ${source.label}`} disabled={!context || busy || !displayMatches} onStart={launch} />}
    <DebugStrip {...debug} locationActions={<ProjectDebugLinks owner={source.debugOwner ?? uri} debug={debug}/>} editorView={view} pausedLine={exact ? debug.pausedLine : null} program={path}
      onSelectFrame={id=> {
        debug.selectFrame(id); const frame=debug.frames.find(f=>f.id===id);
        if (frame && revealPreferredDebugLocation(source.debugOwner ?? uri, frame)) return;
        if (frame?.source?.startsWith("hick-project:///") && frame.source!==uri)
          void openCapturedCode({uri:frame.source,range:{start:{line:frame.source_line ?? 0,character:0},end:{line:frame.source_line ?? 0,character:0}}},source.debugOwner ?? uri).catch(e=>setError(String(e)));
        else if (view && frame?.line !== null && frame?.line !== undefined) revealLine(view,frame.line);
      }} onStep={debug.step} onStart={launch} onStop={debug.stop}
      onJumpHere={()=> {if(view && exact) debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number-1);}}
      onAddWatch={()=>{void prompt.askText("Expression to watch:", "").then(expression=>{if(expression)debug.addWatch(expression);});}}
      onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch}
      onInstall={async offer=> {await api.installTool(offer.kind,offer.language);}} />
    {error && <div className="comparison-tools">
      <p role="alert">Could not start code tools: {error}</p>
      <button className="btn" onClick={() => setRetry(n => n + 1)}>Retry code tools</button>
    </div>}
    <PromptPanel prompt={prompt.prompt} onSettle={prompt.settle} />
    {references && <div className="code-references" role="dialog" aria-label="References" onKeyDown={event=>{if(event.key==="Escape")setReferences(null);}}>
      <button type="button" aria-label="Close references" onClick={()=>setReferences(null)}>×</button>
      {references.length ? references.map((target,index)=><button type="button" key={index} onClick={()=>{
        void openCapturedCode(target,source.debugOwner ?? uri).then(()=>setReferences(null),e=>setError(String(e)));
      }}>{decodeURIComponent(target.uri.split("/").slice(4).join("/"))}:{target.range.start.line+1}</button>) : <p>No references found.</p>}
    </div>}
  </>;
}

function useSelectedFrame(view: EditorView | null, line: number | null | undefined) {
  useEffect(()=> { if (view && line !== null && line !== undefined) revealLine(view,line); },[view,line]);
}
