<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read workspace analysis without moving code

For maintainers changing workspace services independently of the editor renderer.

Expose acquisition through Edit. The label shows progress while acquisition runs. The report floats over code and has bounded, filterable pages; starting it never activates a source pane. It names generated and unclassified exclusions and keeps zero-symbol reads distinct from a clean-project claim. Closing the report leaves background work alive; Stop drains admitted reads without starting more files. Opening source remains a separate action through the existing Files or navigation surfaces.

<ui:copy id="simple-workspace-analysis-browser-view">
import {useEffect,useRef,useState,useSyncExternalStore} from "react";
import {createPortal} from "react-dom";
import type {FileNode} from "../api/types";
import type {Buffers} from "./buffers";
import {WorkspaceAnalysis} from "./WorkspaceAnalysis";
import {useWorkspaceAnalysisRunner} from "./WorkspaceLanguages";
const SHOW="hickory:simple:workspace-analysis";
export function requestWorkspaceAnalysis(){window.dispatchEvent(new Event(SHOW));}
export function WorkspaceAnalysisView({buffers,files,model,focusCurrent}:{buffers:Buffers;files:FileNode[];model:WorkspaceAnalysis;focusCurrent:()=>void}) {
  useSyncExternalStore(model.subscribe,model.snapshot);
  const [visible,setVisible]=useState(false),[query,setQuery]=useState(""),[page,setPage]=useState(0);const panel=useRef<HTMLDivElement>(null);
  const run=useWorkspaceAnalysisRunner(model,buffers,files);
  useEffect(()=>{const show=()=>setVisible(true);window.addEventListener(SHOW,show);return()=>{window.removeEventListener(SHOW,show);model.stop();};},[model]);
  useEffect(()=>{if(visible)panel.current?.focus();},[visible]);
  const close=()=>{setVisible(false);focusCurrent();};
  const reports=[...model.knowledge.values()].filter(item=>`${item.path} ${item.notice}`.toLowerCase().includes(query.toLowerCase())).sort((a,b)=>a.path.localeCompare(b.path));
  const last=Math.max(0,Math.ceil(reports.length/100)-1),selected=Math.min(page,last);
  return <><button type="button" onClick={()=>visible?close():setVisible(true)}>{model.running?`Workspace analysis · ${model.completed}/${model.scope.files.length}`:"Workspace analysis"}</button>{visible&&createPortal(
    <div ref={panel} tabIndex={-1} className="simple-workspace-problems" role="dialog" aria-label="Workspace analysis" onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();close();}}}>
      <button aria-label="Close analysis" onClick={close}>×</button><strong>Workspace analysis</strong>
      <p>{model.completed}/{model.scope.files.length} attempted · {[...model.knowledge.values()].filter(item=>item.source!==undefined).length} sources read · {model.scope.generated} generated and {model.scope.unclassified} unclassified excluded.</p>
      <p>Provider symbols and diagnostics arrive independently. Zero symbols may mean no declarations, unsupported analysis or indexing. This is not verification.</p>
      <button disabled={model.running} title={model.running ? "Workspace analysis is in progress." : undefined} onClick={()=>{void run();}}>Read workspace</button>
      <button disabled={!model.running||model.stopping} title={!model.running ? "There is no workspace analysis to stop." : model.stopping ? "A stop request is in progress." : undefined} onClick={()=>model.stop()}>{model.stopping?"Stopping…":"Stop"}</button>
      <input aria-label="Filter analysis" value={query} onChange={event=>{setQuery(event.target.value);setPage(0);}}/>
      {reports.slice(selected*100,(selected+1)*100).map(item=><div key={item.path}><strong>{item.path}</strong><p>{item.source !== undefined && buffers.values.get(item.path)?.text !== item.source ? "stale" : item.state} · {item.locations.length} symbols · {item.notice}</p></div>)}
      {last>0&&<div><button disabled={!selected} title={!selected ? "You’re on the first page." : undefined} onClick={()=>setPage(selected-1)}>Previous results</button>{selected+1}/{last+1}<button disabled={selected===last} title={selected===last ? "You’re on the last page." : undefined} onClick={()=>setPage(selected+1)}>Next results</button></div>}
    </div>,document.body)}</>;
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceAnalysisView.tsx"><ui:paste select="#simple-workspace-analysis-browser-view"/></ui:file>
</ui:doc>
