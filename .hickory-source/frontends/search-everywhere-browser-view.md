<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Search within the existing browser shell

For maintainers changing workspace services independently of the editor renderer.

Keep Search Everywhere as a floating consumer of workspace facts. The input ranks files, source-current symbols and the same commands used by menus. Opening the panel preserves the mounted editor and does not allocate code space. Keyboard selection, Escape and composition handling belong to this adapter; source proof and command eligibility remain in the existing models. Read workspace reuses the bounded acquisition adapter, and selecting a symbol carries its captured source to authoritative reveal.

<ui:copy id="simple-search-everywhere-browser-view">
import {useEffect,useRef,useState,useSyncExternalStore} from "react";
import {createPortal} from "react-dom";
import type {FileNode} from "../api/types";
import type {Buffers} from "./buffers";
import type {WorkspaceAnalysis} from "./WorkspaceAnalysis";
import type {WorkspaceCommand} from "./WorkspaceCommands";
import {useWorkspaceAnalysisRunner} from "./WorkspaceLanguages";
import {everywhereTargets,searchEverywhere,type SearchKind,type SearchTarget} from "./SearchEverywhere";
import {openLocation} from "../lib/revealLine";
export function SearchEverywhereView({visible,setVisible,buffers,files,analysis,commands,openFile,focusCurrent}:{visible:boolean;setVisible:(value:boolean)=>void;buffers:Buffers;files:FileNode[];analysis:WorkspaceAnalysis;commands:WorkspaceCommand[];openFile:(path:string)=>void;focusCurrent:()=>void}) {
  useSyncExternalStore(analysis.subscribe,analysis.snapshot);
  const [query,setQuery]=useState(""),[kind,setKind]=useState<SearchKind|"all">("all"),[selected,setSelected]=useState(0),[notice,setNotice]=useState("");const input=useRef<HTMLInputElement>(null);
  const run=useWorkspaceAnalysisRunner(analysis,buffers,files);
  useEffect(()=>{if(visible)input.current?.focus();},[visible]);
  useEffect(()=>{const key=(event:KeyboardEvent)=>{const primary=navigator.platform.includes("Mac")?event.metaKey:event.ctrlKey;if(primary&&!event.shiftKey&&!event.altKey&&event.key.toLowerCase()==="p"&&!event.isComposing){event.preventDefault();event.stopPropagation();setVisible(true);}};window.addEventListener("keydown",key);return()=>window.removeEventListener("keydown",key);},[setVisible]);
  useEffect(()=>{if(visible)document.getElementById(`simple-everywhere-${selected}`)?.scrollIntoView?.({block:"nearest"});},[selected,visible]);
  if(!visible)return null;
  const targets=everywhereTargets([...files,...[...buffers.values.values()].map(buffer=>buffer.file)],[...analysis.knowledge.values()],path=>buffers.values.get(path)?.text,commands);
  const result=searchEverywhere(targets,query,kind),index=Math.min(selected,Math.max(0,result.items.length-1));
  const close=()=>{setVisible(false);focusCurrent();};
  const invoke=(target:SearchTarget)=>{
    if(target.commandID){const command=commands.find(item=>item.id===target.commandID);if(!command||command.reason){setNotice(command?.reason??"Command unavailable. Refresh Search Everywhere.");return;}setVisible(false);command.run();return;}
    if(!target.path)return;
    if(target.location&&target.source!==undefined){if(!analysis.current(target.path,target.source)||buffers.values.get(target.path)?.text!==target.source){setNotice("Symbol source changed. Read the workspace again.");return;}
      openLocation(target.path,target.location.range.start.line+1,target.location.range.start.character,true,target.source);
    }else openFile(target.path);
    setVisible(false);
  };
  return createPortal(<div className="simple-workspace-problems simple-search-everywhere" role="dialog" aria-label="Search Everywhere" onKeyDown={event=>{
    if(event.nativeEvent.isComposing)return;
    if(event.key==="Escape"){event.preventDefault();event.stopPropagation();close();}
    if(event.target===input.current&&["ArrowDown","ArrowUp"].includes(event.key)){event.preventDefault();setSelected(Math.max(0,Math.min(result.items.length-1,index+(event.key==="ArrowDown"?1:-1))));}
    if(event.target===input.current&&event.key==="Enter"){event.preventDefault();const target=result.items[index];if(target&&!target.reason)invoke(target);}
  }}><button aria-label="Close Search Everywhere" onClick={close}>×</button><strong>Search Everywhere</strong>
    <input ref={input} aria-label="Search files, types, members and actions" value={query} role="combobox" aria-expanded="true" aria-autocomplete="list" aria-activedescendant={result.items.length?`simple-everywhere-${index}`:undefined} aria-controls="simple-everywhere-results" onChange={event=>{setQuery(event.target.value);setSelected(0);setNotice("");}}/>
    <select aria-label="Search scope" value={kind} onChange={event=>{setKind(event.target.value as SearchKind|"all");setSelected(0);}}>{["all","file","type","member","symbol","action"].map(value=><option key={value} value={value}>{value}</option>)}</select>
    <p>{result.items.length} of {result.total} matches <button aria-label="Read workspace" disabled={analysis.running} title={analysis.running ? "Workspace analysis is in progress." : undefined} onClick={()=>{void run();}}>{analysis.running?"Reading…":"Read workspace"}</button></p>
    <p>Symbols reflect source-current provider reads; missing results may need analysis or language support. Files and actions work independently.</p>{notice&&<p role="status">{notice}</p>}
    {!result.items.length&&<p>No matches in current knowledge. Try another query or Read workspace.</p>}
    <div id="simple-everywhere-results" role="listbox" aria-label="Search results">{result.items.map((target,at)=><div key={target.id}><button type="button" role="option" id={`simple-everywhere-${at}`} aria-selected={at===index} disabled={!!target.reason} title={!!target.reason ? target.reason : undefined} onFocus={()=>setSelected(at)} onClick={()=>invoke(target)}>{target.label}<small>{target.kind} · {target.detail}</small></button>{target.reason&&<p>{target.reason}</p>}</div>)}</div>
  </div>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/SearchEverywhereView.tsx"><ui:paste select="#simple-search-everywhere-browser-view"/></ui:file>
</ui:doc>
