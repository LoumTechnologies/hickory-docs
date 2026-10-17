import { launchWithProfiles } from "./launchProfiles";
import { useEffect, useMemo, useRef } from "react";
import { Compartment, StateEffect } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";
import { parseHickDoc } from "../editor/hickDoc";
import type { OutputFile } from "../api/types";
import { api } from "../api/client";
import { provToChars } from "../components/OutputEditorPane";
import { sourcePositionAt } from "../lsp/outputMapping";
import { DebugLaunchAction } from "./DebugRail";
import { DebugStrip } from "./DebugStrip";
import { debugEditor, debugStateEffects, revealLine } from "./cmDebug";
import { isDebuggable } from "./languages";
import { languageFromPath } from "../editor/languages";
import type { DebugSession } from "./useDebugger";

/** Exact line correspondence; duplicate source occurrences remain ambiguous. */
export function generatedLineMap(file: OutputFile, docPath: string, source: string): Map<number, number> {
  const result = new Map<number, number>(), provenance = provToChars(file);
  let at = 0;
  file.content.split("\n").forEach((text,line) => {
    const position = sourcePositionAt(at, provenance, docPath, source);
    if (position) result.set(line, position.line);
    at += text.length + 1;
  });
  return result;
}
export function GeneratedDebugTools({view,file,debug,docPath,source,askText}: {
  view: EditorView | null; file: OutputFile; debug: DebugSession; docPath: string; source: string;
  askText: (title:string,initial:string)=>Promise<string|null>;
}) {
  const tools = useMemo(()=>new Compartment(),[]);
  const mapping = useMemo(()=>generatedLineMap(file,docPath,source),[file,docPath,source]);
  const outputLine = (line:number|null) => {
    const candidates = [...mapping].filter(([,sourceLine])=>sourceLine===line);
    return candidates.length===1 ? candidates[0][0] : null;
  };
  const current = useRef({debug,mapping,file}); current.current={debug,mapping,file};
  useEffect(()=> {
    if (!view || !isDebuggable(languageFromPath(file.path))) return;
    const sourceLine = (line:number) => view.state.doc.toString()===current.current.file.content ? current.current.mapping.get(line) : undefined;
    const extensions = debugEditor({language:languageFromPath(file.path),lineNumbers:false,
      onToggleBreakpoint:line=>{const at=sourceLine(line);if(at!==undefined)current.current.debug.toggleBreakpoint(at);},
      onSetBreakpointCondition:(line,patch)=>{const at=sourceLine(line);if(at!==undefined)current.current.debug.setBreakpointCondition(at,patch);},
      onSelectFrame:id=>current.current.debug.selectFrame(id),
      onEvaluate:expression=>current.current.debug.query(expression),onAddWatch:expression=>current.current.debug.addWatch(expression),
    });
    view.dispatch({effects:tools.get(view.state)===undefined?StateEffect.appendConfig.of(tools.of(extensions)):tools.reconfigure(extensions)});
    return ()=>{if(view.dom.isConnected)view.dispatch({effects:tools.reconfigure([])});};
  },[view,file.path,tools]);
  const activeHere = debug.program === file.path;
  const pausedLine = activeHere ? outputLine(debug.frames.find(frame=>frame.id===debug.selectedFrame)?.line ?? debug.pausedLine) : null;
  useEffect(()=> {
    if (!view || !isDebuggable(languageFromPath(file.path))) return;
    const exact = view.state.doc.toString()===file.content;
    view.dispatch({effects:debugStateEffects({...debug,pausedLine:exact?pausedLine:null,
      breakpoints:debug.breakpoints.flatMap(bp=>{const line=outputLine(bp.line);return line===null?[]:[{...bp,line}];}),
      frames:activeHere ? debug.frames.map(frame=>({...frame,line:outputLine(frame.line)})) : [],
      variables:activeHere?debug.variables:[], watches:activeHere?debug.watches:[],
    })});
    if(exact && pausedLine!==null)revealLine(view,pausedLine);
  },[view,file,debug,pausedLine,mapping,activeHere]);
  if(!isDebuggable(languageFromPath(file.path)))return null;
  const start=(choose=false)=>{
    if(view?.state.doc.toString()!==file.content)return;
    const project=parseHickDoc(source).blocks.some(b=>b.name==="project")?`latest-code:${JSON.stringify([file.path,file.content,source])}`:"new";
    void launchWithProfiles({uri:`hick:///${docPath.replace(/^\/+/,"")}`,program:file.path,project,expected_document:source,expected_source:file.content},
      profile=>debug.start(file.path,{},project,profile),{choose,isCurrent:()=>view.state.doc.toString()===file.content});
  };
  return <>
    <DebugLaunchAction view={view} label={`Debug ${file.path}`} onStart={()=>start()} onProfiles={()=>start(true)} disabled={!view || view.state.doc.toString()!==file.content} hint={view?.state.doc.toString()!==file.content?"Waiting for your edits to save before debugging":undefined}/>
    <DebugStrip {...debug} status={activeHere?debug.status:"idle"} editorView={view} pausedLine={pausedLine} program={file.path}
      onStep={debug.step} onStart={()=>start()} onStop={debug.stop} onSelectFrame={debug.selectFrame}
      onJumpHere={()=>{if(view){const line=mapping.get(view.state.doc.lineAt(view.state.selection.main.head).number-1);if(line!==undefined)debug.jumpTo(line);}}}
      onAddWatch={()=>{void askText("Expression to watch:","").then(expression=>{if(expression)debug.addWatch(expression);});}}
      onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch}
      onInstall={offer=>api.installTool(offer.kind,offer.language).then(()=>undefined)}/>
  </>;
}
