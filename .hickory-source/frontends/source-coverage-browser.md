<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect source-bound coverage in the existing gutter

For maintainers changing workspace services independently of the editor renderer.

A selected report projects captured runtime and authored owners separately. Only a matching background source digest grants a line mark. Edits retire it immediately, missing evidence remains unknown, and mixed measurements keep their own state and count availability. Gutter actions open the retained report through the existing floating surface without executing a cached action.

A retained deleted editor remains readable for its captured debugger, but its old spelling can already belong to another source. Reject deleted handles before producing or invoking path-based inspection hints; restoring the exact handle re-enables its ordinary current-source capabilities.

A gutter mark requires exact authored-source equality with the buffer. Display normalization is a coordinate mechanism, not evidence that different CRLF and LF bytes are current.

<ui:copy id="simple-source-coverage-browser">
import {authoredSource} from './SourceText';
import {createContext,useContext,useEffect,useMemo,useSyncExternalStore,type ReactNode} from 'react';
import type {EditorView} from '@codemirror/view';
import type {Buffer} from './buffers';
import {SourceDigests} from './SourceDigests';
import {coverageProjection,type CoverageProjectedLine} from '../coverage/CoverageProjection';
import {onCoverageSelection,selectedCoverage} from '../coverage/selection';
import type {CoverageRun} from '../coverage/types';
export const SOURCE_COVERAGE_INSPECTION='hickory:simple:inspect-coverage';
const Context=createContext<{digests:SourceDigests;run:CoverageRun|null;rows:Record<string,CoverageProjectedLine[]>}|null>(null);
const idle=()=>()=>{},zero=()=>0;
export function SourceCoverageProvider({workspace,children}:{workspace:string;children:ReactNode}){
 const digests=useMemo(()=>new SourceDigests(),[workspace]);useEffect(()=>()=>digests.stop(),[digests]);
 const run=useSyncExternalStore(onCoverageSelection,selectedCoverage);const rows=useMemo(()=>run?coverageProjection(run):{},[run]);
 return <Context.Provider value={{digests,run,rows}}>{children}</Context.Provider>;
}
export function useCoverageLineActions(buffer:Buffer,view:EditorView|null){
 const model=useContext(Context);useSyncExternalStore(model?.digests.subscribe??idle,model?.digests.snapshot??zero);
 useEffect(()=>{if(!model?.rows[buffer.file.path]||buffer.transient||buffer.resourceDeleted)return;const read=()=>model.digests.enqueue(buffer.file.path,buffer.text,()=>buffer.text);read();return buffer.subscribe(read);},[model?.digests,model?.rows,buffer,buffer.resourceDeleted]);
 if(buffer.resourceDeleted||!model?.run||!view||authoredSource(view.state)!==buffer.text)return [];
 const path=buffer.file.path,source=buffer.text,run=model.run;
 if(model.digests.current(path,source)!==run.identity.source_hashes[path]?.toLowerCase())return [];
 return (model.rows[path]??[]).filter(row=>row.line<view.state.doc.lines).map(row=>({at:view.state.doc.line(row.line+1).from,glyph:row.state==='covered'?'●':row.state==='uncovered'?'○':'◐',tone:row.state==='covered'?'#248047':row.state==='uncovered'?'#b94a48':'#a66b00',label:`Coverage ${row.state} · ${row.count===null?'count unavailable':`count ${row.count}`} · ${row.origins.length} measured location(s)`,run:()=>{
  if(buffer.resourceDeleted||selectedCoverage()?.id!==run.id||model.digests.current(path,buffer.text)!==run.identity.source_hashes[path]?.toLowerCase())return;
  window.dispatchEvent(new CustomEvent(SOURCE_COVERAGE_INSPECTION,{detail:{path,id:run.id}}));
 }}));
}
</ui:copy>

<ui:file path="apps/web/src/simple/SourceCoverage.tsx"><ui:paste select="#simple-source-coverage-browser"/></ui:file>
</ui:doc>
