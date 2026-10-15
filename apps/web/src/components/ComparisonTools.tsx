import { useEffect, useMemo, useRef, useState } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { api, request } from "../api/client";
import { projectPosition, type ProjectEntry } from "../editor/hickLang";
import type { ComparisonSource } from "../lib/projectComparison";
import { publishReading, openReading } from "../lib/readingViews";
import { useWorkspaceLsp } from "../lsp/useLsp";
import { lspSupport, setLspDiagnostics, diagnosticRanges, type LspNavigationTarget } from "../lsp/cmLsp";
import { useWorkspaceDebugger } from "../debug/useDebugger";
import { debugEditor, debugStateEffects } from "../debug/cmDebug";
import { DebugStrip } from "../debug/DebugStrip";
import { languageFromPath } from "../editor/languages";
import { MarginActions } from "./MarginActions";

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
export async function openCapturedCode(target: LspNavigationTarget) {
  const file = await request<{path:string;bytes:number[];files:ProjectEntry[]}>("GET",`/api/project-contexts/file?uri=${encodeURIComponent(target.uri)}`);
  const text = new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(file.bytes));
  const id = `captured:${target.uri}`;
  publishReading(id,{title:`Captured code · ${file.path}:${target.range.start.line+1}`,path:file.path,before:text,after:text,status:"immutable project endpoint",project:file.files});
  openReading(id,file.path);
}

/** Rendering starts nothing. The explicit action registers an isolated value;
 * only didOpen/debug then materializes it for a native tool. */
export function ComparisonTools({path,source,view,compact=false}: {path:string;source:ComparisonSource;view:EditorView|null;compact?:boolean}) {
  const [context,setContext] = useState<{id:string;uri:string}|null>(null);
  const [error,setError] = useState("");
  const [busy,setBusy] = useState(false);
  const tools = useMemo(() => new Compartment(),[]);
  const local = source.local ?? source.project;
  const selected = source.project;
  const displayed = local?.find(e => e.path === path);
  let activePath = selected?.find(e => e.path === path)?.path;
  if (!activePath && displayed?.origins?.length && selected && local) {
    try { activePath = projectPosition(local,selected,path,displayed.origins[0].output[0])?.[0]; } catch { /* explicit absent-code status */ }
  }
  const active = selected?.find(e => e.path === activePath && e.kind === "file");
  const uri = context && active ? `${context.uri}${active.path}` : "";
  const lsp = useWorkspaceLsp(uri,active ? decode(active) : "");
  const debug = useWorkspaceDebugger(uri);
  const live = useRef({source,local,selected,context,debug}); live.current={source,local,selected,context,debug};
  const exact = !!active && !!displayed && active.path === path && decode(active) === source.text;
  useEffect(() => { setContext(null); setError(""); },[selected]);
  useEffect(() => () => { if (context) void request("DELETE",`/api/project-contexts/${context.id}`).catch(() => {}); },[context]);
  useEffect(() => {
    if (!view || !context || !lsp.client || !local || !selected) return;
    const extensions = lspSupport({client:lsp.client,uri:lsp.uri,positionAt:() => null,
      targetAt:(at,v) => {
        const current=live.current;
        if (v.state.doc.toString() !== current.source.text || !current.local || !current.selected || !current.context) return null;
        try {
          const target=projectPosition(current.local,current.selected,path,new TextEncoder().encode(current.source.text.slice(0,at)).length);
          if (!target) return null;
          return {uri:`${current.context.uri}${target[0]}`,position:position(current.selected.find(e=>e.path===target[0])!.bytes,target[1])};
        } catch { return null; }
      },onNavigate:target => { void openCapturedCode(target).catch(e=>setError(String(e))); },
      onReferences:locations => { if (locations[0]) void openCapturedCode(locations[0]).catch(e=>setError(String(e))); else setError("No references in this captured project."); },
      runtimeValue:word => exact ? live.current.debug.valueAt(word) : Promise.resolve(null),
    });
    if (exact) extensions.push(...debugEditor({language:languageFromPath(path),onToggleBreakpoint:line=>live.current.debug.toggleBreakpoint(line),onSelectFrame:id=>live.current.debug.selectFrame(id),onEvaluate:expression=>live.current.debug.query(expression),onAddWatch:expression=>live.current.debug.addWatch(expression),onSetBreakpointCondition:(line,patch)=>live.current.debug.setBreakpointCondition(line,patch)}));
    view.dispatch({effects:tools.get(view.state) === undefined ? StateEffect.appendConfig.of(tools.of(extensions)) : tools.reconfigure(extensions)});
    return () => { if (view.dom.isConnected) view.dispatch({effects:tools.reconfigure([])}); };
  },[view,context,lsp.client,lsp.uri,path,exact,tools,local,selected]);
  useEffect(() => { if (view && context && exact) view.dispatch({effects:debugStateEffects(debug)}); },[view,context,exact,debug]);
  useEffect(() => {
    if (!view || !active || !local || !selected || !context) return;
    const spans=diagnosticRanges(lsp.diagnostics,p=> {
      const at=offset(decode(active),p.line,p.character); if (at === null) return null;
      try {
        const target=projectPosition(selected,local,active.path,new TextEncoder().encode(decode(active).slice(0,at)).length);
        return target?.[0] === path ? new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(displayed!.bytes.slice(0,target[1]))).length : null;
      } catch { return null; }
    });
    view.dispatch({effects:setLspDiagnostics.of(spans)});
  },[view,lsp.diagnostics,context,active,local,selected,path,displayed]);
  const enable=async()=> {
    setBusy(true);setError("");
    try {
      if (!selected || !displayed || decode(displayed) !== source.text) throw new Error("Apply or discard this draft before selecting a native project context.");
      const entries=Object.fromEntries(selected.map(e=>[e.path,{bytes:e.bytes,kind:e.kind === "file" ? "File" : "Symlink",executable:e.executable,origins:e.origins ?? []}]));
      setContext(await request("POST","/api/project-contexts",{snapshot:{entries}}));
    } catch(e) {setError(String(e));} finally {setBusy(false);}
  };
  if (!selected) return null;
  const controls = <div className="comparison-tools">
    <p>{source.label}</p>
    <p className="muted">Code completion, navigation and debugging use the complete project for this side.{compact && (source.id === "old" ? " Before uses the project’s input." : " After uses its final output, including later patches.")}</p>
    {!context && <button className="btn" disabled={busy || !active} onClick={()=>void enable()}>Enable code completion and navigation</button>}
    {!active && <span>Code absent from {source.label}; choose another scope endpoint.</span>}
    {context && <><span>Code tools enabled</span>
      {!exact && <button className="btn" onClick={()=>void openCapturedCode({uri,range:{start:{line:0,character:0},end:{line:0,character:0}}})}>Open completed code to debug</button>}
      {exact && <><button className="btn" onClick={()=>debug.start()}>Debug · {source.label}</button>
        <DebugStrip {...debug} onSelectFrame={id=> {debug.selectFrame(id);const frame=debug.frames.find(f=>f.id===id); if (frame?.source?.startsWith("hick-project:///") && frame.source!==uri) void openCapturedCode({uri:frame.source,range:{start:{line:frame.source_line ?? 0,character:0},end:{line:frame.source_line ?? 0,character:0}}});}}
          onStep={debug.step} onStart={()=>debug.start()} onStop={debug.stop} onJumpHere={()=> {if(view)debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number-1);}}
          onAddWatch={()=>{const expression=window.prompt("Expression to watch:");if(expression)debug.addWatch(expression);}}
          onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch}
          onInstall={async offer=> {await api.installTool(offer.kind,offer.language);debug.start();}} />
      </>}
    </>}
    {error && <p role="alert">{error}</p>}
  </div>;
  return compact ? <MarginActions label={`Code tools · ${source.label}`} glyph="{}">{controls}</MarginActions> : controls;
}
