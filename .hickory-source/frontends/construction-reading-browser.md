<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read captured compiler constructions

For maintainers changing workspace services independently of the editor renderer.

The reading owns a replaceable read adapter and retires pending replies on close or a newer request. Check the captured entry and focus identity before accepting a response. The compiler owns contribution ranges and output changes; the frontend does not infer test execution from declarations or authorize source edits from these facts.

<ui:copy id="construction-reading-browser-model">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import type {SourceTestCandidate,TestSourcePoint} from '../api/testSources';
import {request} from '../api/client';
export interface ConstructionInput {entry:string;sources:Record<string,string>;focus:string|null}
export interface ConstructedSnapshot {hash:string;kind:string;executable:boolean;size:number;preview:string|null;preview_truncated:boolean}
export interface ConstructionOutput {path:string;change:string;before:ConstructedSnapshot|null;after:ConstructedSnapshot|null;contributors:string[];origins_truncated:boolean;origins:{phase:string;path:string;source_bytes:[number,number];output_bytes:[number,number];exact:boolean;source_hash:string|null;source_point?:TestSourcePoint|null;fragment?:{name:string;kind:string;body:[number,number]}|null}[]}
export interface ConstructionReading {version:number;entry:string;focus:string|null;materialized:boolean;output_count:number;outputs_truncated:boolean;outputs:ConstructionOutput[];tests:SourceTestCandidate[];test_count:number;tests_truncated:boolean;test_relationship:string;evidence_status:string;notices:string[];unused_sources:string[]}
export class ConstructionReadingModel {
 value:ConstructionReading|null=null;notice='';loading=false;private ticket=0;private linkTicket=0;
 constructor(private load:(input:ConstructionInput)=>Promise<ConstructionReading>,private changed:()=>void=()=>{}){}
 sourceLink(value:ConstructionReading){const id=++this.linkTicket;return ()=>this.value===value&&id===this.linkTicket;}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(provided:ConstructionInput){const input={...provided,sources:{...provided.sources}};const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading construction…';this.changed();
 try{const value=await this.load(input);if(id!==this.ticket)return;if(value.version!==1||value.entry!==input.entry||value.focus!==input.focus||value.materialized!==false)throw new Error('Construction identity changed or unsupported response. Refresh the reading.');this.value=value;this.notice='Captured construction and draft consequences. Editor changes after capture are not included.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
</ui:copy>

## Compare captured outputs without resizing the editor

Use the existing floating surface and menus. Capture retained authored drafts only when the reader requests analysis. Label the entry, focus, preview side, binary output and every truncation. Contributor byte ranges remain readable observations. Only service-supplied source points with matching acquired source hashes and valid scalar positions enable navigation. Newer links, replaced readings, closing and workspace switches retire pending navigation. Closing restores source focus and retires the request.

<ui:copy id="construction-reading-browser-view">
export function ConstructionReadingView({sources,close,navigate}:{sources:()=>Record<string,string>;close:()=>void;navigate:(point:TestSourcePoint,current:()=>boolean)=>Promise<void>}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new ConstructionReadingModel(input=>request('POST','/api/construction/reading',input),paint),[]);
 const [entry,setEntry]=useState('project.md'),[focus,setFocus]=useState(''),[selected,setSelected]=useState(''),[phase,setPhase]=useState<'before'|'after'>('after');const panel=useRef<HTMLElement>(null);
 useEffect(()=>{panel.current?.focus();return()=>model.close();},[model]);
 const value=model.value,output=value?.outputs.find(item=>item.path===selected)??value?.outputs[0],snapshot=output?.[phase];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Construction reading" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Construction reading</strong><button onClick={close}>Close</button></header>
 <label>Construction entry <input value={entry} onChange={event=>setEntry(event.target.value)}/></label><label>Contributor or output (optional) <input value={focus} onChange={event=>setFocus(event.target.value)}/></label>
 <button disabled={model.loading||!entry.trim()} onClick={()=>{setSelected('');void model.read({entry:entry.trim(),focus:focus.trim()||null,sources:sources()});}}>Read captured drafts</button><p role="status">{model.notice}</p>
 {value&&<><p>Captured entry: {value.entry}{value.focus?` · focus: ${value.focus}`:""}</p><p>{value.output_count} related outputs{value.outputs_truncated?' · first 300 shown':''}. {value.test_count} detected tests{value.tests_truncated?' · test list truncated':''}. Evidence: {value.evidence_status}. Outputs have not been written.</p><p>{value.test_relationship}</p><details><summary>Detected test declarations</summary>{value.tests.map(test=><p key={test.id}>{test.name} · {test.source?.path??test.target.path}:{test.source?.line??test.target.line}</p>)}</details>{value.notices.map((notice,index)=><p key={index}>{notice}</p>)}{value.unused_sources.length>0&&<p>{value.unused_sources.length} supplied drafts did not contribute to this entry.</p>}
 <label>Constructed output <select value={output?.path??''} onChange={event=>setSelected(event.target.value)}>{value.outputs.map(item=><option key={item.path} value={item.path}>{item.change} · {item.path}</option>)}</select></label>
 {output&&<article><strong>{output.change} · {output.path}</strong><p>Contributors: {output.contributors.join(', ')||'No compiler source origins reported.'}</p><button onClick={()=>setPhase('before')}>Saved construction</button><button onClick={()=>setPhase('after')}>Draft construction</button>
 {snapshot?<><p>{snapshot.kind} · {snapshot.size} bytes · executable {String(snapshot.executable)}</p><pre style={{whiteSpace:'pre-wrap'}}>{snapshot.preview??'Binary output; text preview unavailable.'}</pre>{snapshot.preview_truncated&&<p>Preview abbreviated.</p>}</>:<p>Output absent from this construction.</p>}
 <details><summary>Compiler contribution ranges</summary>{output.origins.map((origin,index)=><p key={index}>{origin.phase} · {origin.path} bytes {origin.source_bytes.join('–')} → output bytes {origin.output_bytes.join('–')} · {origin.exact?'equal byte slices':'reported origin; exact bytes unconfirmed'}{origin.fragment&&` · declared ${origin.fragment.kind} #${origin.fragment.name}`}{origin.source_point&&<button aria-label={`Open ${origin.phase} contributor ${origin.path}`} onClick={()=>{const current=model.sourceLink(value);void navigate(origin.source_point!,current).catch(error=>{if(current()){model.notice=String(error);paint();}});}}>Open contributor</button>}</p>)}{output.origins_truncated&&<p>First 100 origins shown.</p>}</details></article>}</>}
 </section>,document.body);
}
</ui:copy>

## Validate a captured point before moving the editor

A source link needs a matching SHA-256 value computed over the acquired text. Require a positive line, an in-range UTF-16 column and a scalar boundary; never clamp a stale point into the document. File identity, workspace lifetime and captured-reading lifetime are checked separately by the navigation adapter.

<ui:copy id="construction-reading-browser-source-point">
export function constructionPointOffset(source:string,point:TestSourcePoint,actualHash:string):number|null {
 if(!/^[0-9a-f]{64}$/.test(point.hash)||point.hash!==actualHash||!Number.isSafeInteger(point.line)||!Number.isSafeInteger(point.character)||point.line<1||point.character<0)return null;
 const rows=source.split('\n');if(point.line>rows.length||point.character>rows[point.line-1].length)return null;
 const offset=rows.slice(0,point.line-1).reduce((sum,row)=>sum+row.length+1,0)+point.character;
 if(offset>0&&offset<source.length&&source.charCodeAt(offset-1)>=0xd800&&source.charCodeAt(offset-1)<=0xdbff&&source.charCodeAt(offset)>=0xdc00&&source.charCodeAt(offset)<=0xdfff)return null;
 return offset;
}
</ui:copy>

<ui:file path="apps/web/src/simple/ConstructionReading.tsx"><ui:paste select="#construction-reading-browser-model"/><ui:paste select="#construction-reading-browser-view"/><ui:paste select="#construction-reading-browser-source-point"/></ui:file>
## Retire replaced and closed readings

Release held compiler replies after a newer reading or after closing. Neither may replace the retained result. A different entry refuses instead of appearing under the selected construction. Source-point cases also require exact hashes and Unicode scalar boundaries, and a newer link retires an older link. These service witnesses do not establish native physical input or assistive technology acceptance.

<ui:copy id="construction-reading-browser-lifetime-witness">
import {it,expect} from 'vitest';
import {ConstructionReadingModel,constructionPointOffset,type ConstructionReading} from './ConstructionReading';
const value=(entry='project.md'):ConstructionReading=>({version:1,entry,focus:null,materialized:false,outputs:[],output_count:0,outputs_truncated:false,tests:[],test_count:0,tests_truncated:false,test_relationship:'declaration',evidence_status:'not_queried',notices:[],unused_sources:[]});
it('retires superseded and closed compiler readings and rejects foreign identities',async()=>{
 const pending:((value:ConstructionReading)=>void)[]=[];const model=new ConstructionReadingModel(()=>new Promise(resolve=>pending.push(resolve)));const input={entry:'project.md',focus:null,sources:{}};
 const old=model.read(input),latest=model.read(input);pending[1]({...value(),notices:['new']});await latest;pending[0]({...value(),notices:['old']});await old;expect(model.value?.notices).toEqual(['new']);
 const closed=model.read(input);model.close();pending[2](value());await closed;expect(model.value).toBeNull();
 const foreign=model.read(input);pending[3](value('other.md'));await foreign;expect(model.value).toBeNull();expect(model.notice).toContain('identity changed');
 const written=model.read(input);pending[4]({...value(),materialized:true});await written;expect(model.value).toBeNull();
});
it('captures request identity before callers change their request object',async()=>{
 let finish!:(value:ConstructionReading)=>void;const model=new ConstructionReadingModel(()=>new Promise(resolve=>finish=resolve));const input={entry:'project.md',focus:null,sources:{'a.md':'one'}};const read=model.read(input);input.entry='other.md';input.sources['a.md']='two';finish(value());await read;expect(model.value?.entry).toBe('project.md');
});

it('requires exact source hashes and scalar UTF-16 boundaries and supersedes older links',()=>{
 const source='😀e\u0301\né😀code',hash='a'.repeat(64),point={path:'part.md',line:2,character:3,hash};
 expect(constructionPointOffset(source,point,hash)).toBe(source.indexOf('code'));
 expect(constructionPointOffset(source,{...point,character:2},hash)).toBeNull();
 expect(constructionPointOffset(source,{...point,line:0},hash)).toBeNull();
 expect(constructionPointOffset(source,{...point,character:99},hash)).toBeNull();
 expect(constructionPointOffset(source,point,'b'.repeat(64))).toBeNull();
 const model=new ConstructionReadingModel(async()=>value());model.value=value();const old=model.sourceLink(model.value),latest=model.sourceLink(model.value);expect(old()).toBe(false);expect(latest()).toBe(true);model.close();expect(latest()).toBe(false);
});
</ui:copy>

<ui:file path="apps/web/src/simple/ConstructionReading.test.ts"><ui:paste select="#construction-reading-browser-lifetime-witness"/></ui:file>
## Read draft consequences through the actual menu

Drive a retained authored draft through the View menu against a temporary real engine. Compare saved and draft construction bytes, refuse a stale saved-source link, and reveal the matching draft contributor at its exact position. Close and verify editor identity, geometry, text and undo/redo. Saved sources remain unchanged and generated files remain absent. This focused witness does not establish native pointer or device acceptance.

<ui:copy id="construction-reading-browser-real-engine">
"""Drive the existing View menu against a real engine and retained authored draft."""
import importlib.util
import os
from pathlib import Path
import socket
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-construction-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir()
  entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>'
  source='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:copy id="body">one</h:copy><h:file path="output.txt"><h:paste select="#body"/></h:file></h:doc>'
  (root/"project.md").write_bytes(entry.encode());(root/"part.md").write_bytes(source.encode());draft=source.replace('>one<','>two<')
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="part.md");driver.act("remember-editor",path="part.md");size=driver.act("editor-size",path="part.md")
   driver.act("construction-set-source",path="part.md",source=draft);actual=driver.act("construction-source",path="part.md");assert actual==draft,(draft,actual)
   driver.act("construction-open");text=driver.act("construction-read",entry="project.md",focus="part.md");assert "modified" in text and "output.txt" in text and "not_queried" in text,text
   assert driver.act("construction-side",after=False)=="one";assert driver.act("construction-side",after=True)=="two"
   stale=driver.act("construction-origin",phase="before",path="part.md",stale=True);assert "Source changed" in stale and "declared copy #body" in stale,stale
   driver.act("construction-origin",phase="after",path="part.md",stale=False)
   selected=driver.act("construction-source",path="part.md",selection=True);assert selected["anchor"]==draft.index(">two<")+1,selected
   driver.act("construction-open");driver.act("construction-close")
   assert driver.act("editor-is-retained",path="part.md"), "Editor identity changed"
   after_size=driver.act("editor-size",path="part.md");assert after_size==size,(size,after_size)
   after_source=driver.act("construction-source",path="part.md");assert after_source==draft,(draft,after_source)
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path="part.md")==source
   driver.act("construction-history",redo=True);assert driver.act("construction-source",path="part.md")==draft
   assert (root/"part.md").read_bytes()==source.encode();assert (root/"project.md").read_bytes()==entry.encode();assert not (root/"output.txt").exists()
   print("PASS: existing View menu, captured retained draft, compiler contribution/impact reading, saved/draft previews, stale-link refusal and exact matching-source reveal, retained editor/dimensions/undo/redo, unchanged source and no generated materialization")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-simple-construction-browser.py"><ui:paste select="#construction-reading-browser-real-engine"/></ui:file>
</ui:doc>
