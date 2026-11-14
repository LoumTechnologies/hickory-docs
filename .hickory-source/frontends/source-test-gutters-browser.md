<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect source tests from the existing gutter on browser

For maintainers changing workspace services independently of the editor renderer.

The editor supplies text immediately; a separate bounded source index reads test declarations in the background. A source-bound hint uses the existing right-hand action column and overflow grid. Clicking it activates the existing buffer, then refreshes the full floating test view with current drafts before offering execution. Cached gutter facts never authorize a run. Other saved construction contributors can differ from retained drafts, so this fresh inspection is required even when the primary text still matches.

<ui:copy id="source-test-gutters-browser">
import {createContext,useContext,useEffect,useMemo,useSyncExternalStore,type ReactNode} from "react";
import type {EditorView} from "@codemirror/view";
import {api} from "../api/client";
import type {Buffer} from "./buffers";
import {SourceTestIndex,indexedTestRows} from "./SourceTestIndex";
export const SOURCE_TEST_INSPECTION="hickory:simple:inspect-source-tests";
export interface SourceTestInspection {path:string;name:string;serial:number}
const Context=createContext<SourceTestIndex|null>(null);
const idleSubscribe=()=>()=>{},zero=()=>0;
export function SourceTestGutterProvider({workspace,children}:{workspace:string;children:ReactNode}) {
  const index=useMemo(()=>new SourceTestIndex((path,source)=>api.sourceTestCandidates({path,source})),[workspace]);
  useEffect(()=>()=>index.stop(),[index]);
  return <Context.Provider value={index}>{children}</Context.Provider>;
}
/** A gutter hint opens a fresh inspection; it never executes its cached candidate. */
export function useSourceTestLineActions(buffer:Buffer,view:EditorView|null) {
  const index=useContext(Context);useSyncExternalStore(index?.subscribe??idleSubscribe,index?.snapshot??zero);
  useEffect(()=>{
    if(!index||buffer.transient)return;
    const read=()=>index.enqueue(buffer.file.path,buffer.text,()=>buffer.text);
    read();return buffer.subscribe(read);
  },[index,buffer]);
  const record=index?.current(buffer.file.path,buffer.text);
  if(!view||view.state.doc.toString()!==buffer.text.replace(/\r\n/g,"\n"))return [];
  const grouped=new Map<number,string[]>();
  for(const row of indexedTestRows(record)){const names=grouped.get(row.line)??[];names.push(row.candidate.name);grouped.set(row.line,names);}
  return [...grouped].filter(([line])=>line<view.state.doc.lines).map(([line,names])=>({at:view.state.doc.line(line+1).from,label:`Inspect source tests: ${names.join(", ")}`,run:()=>{
    if(!index?.current(buffer.file.path,buffer.text))return;
    window.dispatchEvent(new CustomEvent(SOURCE_TEST_INSPECTION,{detail:{path:buffer.file.path,name:names.length===1?names[0]:"",serial:Date.now()}}));
  }}));
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceTestGutters.tsx"><ui:paste select="#source-test-gutters-browser"/></ui:file>
</ui:doc>
