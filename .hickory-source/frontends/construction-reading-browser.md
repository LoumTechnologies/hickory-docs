<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read captured compiler constructions

For maintainers changing workspace services independently of the editor renderer.

The reading owns a replaceable read adapter and retires pending replies on close or a newer request. Check the captured entry and focus identity before accepting a response. The compiler owns contribution ranges and output changes; the frontend does not infer test execution from declarations or authorize source edits from these facts.

<ui:copy id="construction-reading-browser-model">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import type {SourceTestCandidate,TestSourcePoint} from '../api/testSources';
import {request} from '../api/client';
import {retainedBehaviorReading} from './BehaviorReading';
import {BehaviorReadingPanel} from './BehaviorReadingView';
import {readFrozenEntrySelection,type EntrySelection} from './WorkspaceEntrySelection';
export interface ConstructionInput {entry:string;sources:Record<string,string>;focus:string|null;entry_selection?:EntrySelection}
export interface ConstructedSnapshot {hash:string;kind:string;executable:boolean;size:number;preview:string|null;preview_truncated:boolean}
export interface ConstructionOutput {path:string;change:string;before:ConstructedSnapshot|null;after:ConstructedSnapshot|null;contributors:string[];origins_truncated:boolean;origins:{phase:string;path:string;source_bytes:[number,number];output_bytes:[number,number];exact:boolean;source_hash:string|null;source_point?:TestSourcePoint|null;fragment?:{name:string;kind:string;body:[number,number]}|null}[]}
export interface ConstructionReading {version:number;entry:string;focus:string|null;materialized:boolean;output_count:number;outputs_truncated:boolean;outputs:ConstructionOutput[];tests:SourceTestCandidate[];test_count:number;tests_truncated:boolean;test_relationship:string;evidence_status:string;notices:string[];unused_sources:string[]}
export class ConstructionReadingModel {
 value:ConstructionReading|null=null;notice='';loading=false;private ticket=0;private linkTicket=0;
 constructor(private load:(input:ConstructionInput)=>Promise<ConstructionReading>,private changed:()=>void=()=>{}){}
 sourceLink(value:ConstructionReading){const id=++this.linkTicket;return ()=>this.value===value&&id===this.linkTicket;}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(provided:ConstructionInput,current:()=>boolean=()=>true){const input={...provided,sources:{...provided.sources}};const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading construction…';this.changed();
 try{const value=await this.load(input);if(id!==this.ticket)return;if(!current())throw Error('Original construction selection changed. Refresh the reading.');if(value.version!==1||value.entry!==input.entry||value.focus!==input.focus||value.materialized!==false)throw new Error('Construction identity changed or unsupported response. Refresh the reading.');this.value=value;this.notice='Captured construction and draft consequences. Editor changes after capture are not included.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
</ui:copy>

## Compare captured outputs without resizing the editor

The change badge compares reconstructions from the saved entry and the captured drafts.
Use Files to inspect the working copy. The preview includes the digest of the
full reconstructed output so an abbreviated display can still identify its bytes.

Use the existing floating surface and menus. A blank entry field resolves the
current workspace entry through the engine. Capture its original proof and
reuse it for both readings and later verification. Typing an explicit path
keeps address-based reading. The two modes do not renew one another.

Capture retained authored drafts only when the reader requests analysis. Label the entry, focus, preview side, binary output and every truncation. Contributor byte ranges remain readable observations. Only service-supplied source points with matching acquired source hashes and valid scalar positions enable navigation. Newer links, replaced readings, closing and workspace switches retire pending navigation. Closing restores source focus and retires the request.

<ui:copy id="construction-reading-browser-view">
export function ConstructionReadingView({sources,retainedSources,capture,workspace,engineWorkspace,terminal,close,navigate}:{sources:()=>Record<string,string>;retainedSources?:()=>{sources:Record<string,string>;missing_sources:string[];unaddressed_sources:Record<string,string>;saved_guards:Record<string,string>};capture?:()=>()=>boolean;workspace?:string;engineWorkspace?:string;terminal?:(id:string)=>void;close:()=>void;navigate:(point:TestSourcePoint,current:()=>boolean)=>Promise<void>}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new ConstructionReadingModel(input=>request('POST','/api/construction/reading',input),paint),[]);
 const behavior=useMemo(()=>retainedBehaviorReading(workspace??'unknown',engineWorkspace??''),[workspace,engineWorkspace]);behavior.changed=paint;
 const [entry,setEntry]=useState(''),[focus,setFocus]=useState(''),[selected,setSelected]=useState(''),[phase,setPhase]=useState<'before'|'after'>('after');
 const [acquiring,setAcquiring]=useState(false),panel=useRef<HTMLElement>(null),reading=useRef(0);
 useEffect(()=>{panel.current?.focus();return()=>{reading.current++;model.close();behavior.close();behavior.changed=()=>{};};},[model,behavior]);

 async function readCaptured(){
  const intent=++reading.current,explicit=entry.trim()?entry:'',contributor=focus.trim()||null;
  const retained=retainedSources?.(),snapshot=retained?.sources??sources();
  const lease=capture?.()??(()=>JSON.stringify(sources())===JSON.stringify(snapshot));
  const current=()=>intent===reading.current&&lease();
  setSelected('');
  model.close();behavior.close();
  try{
   let address=explicit;
   let proof:EntrySelection|undefined;
   if(!address){
    if(!workspace||!engineWorkspace)throw Error('Refresh Files to read this workspace’s current construction entry.');
    setAcquiring(true);
    model.notice='Reading the current workspace entry…';paint();
    const acquired=await readFrozenEntrySelection(()=>request('GET','/api/workspace/entry'),
       {namespace:workspace,workspace:engineWorkspace,current});
    if(!acquired.entryPath)throw Error('This workspace has no default construction entry. Enter an authored path.');
    address=acquired.entryPath;
    proof=acquired.proof;
   }
   if(!current())throw Error('Original construction scope changed. Read again from its current source.');
   const input:ConstructionInput={entry:address,focus:contributor,sources:snapshot,...(proof?{entry_selection:proof}:{})};
   void model.read(input,current);
   void behavior.read({version:1,...input,missing_sources:retained?.missing_sources??[],
     unaddressed_sources:retained?.unaddressed_sources??{},saved_guards:retained?.saved_guards??{}},current);
  }catch(error){
   if(intent===reading.current){model.notice=String(error);paint();}
  }finally{if(intent===reading.current)setAcquiring(false);}
 }
 function retireReading(){
  reading.current++;
  setAcquiring(false);
  model.close();behavior.close();
  model.notice='';
 }
 const value=model.value,output=value?.outputs.find(item=>item.path===selected)??value?.outputs[0],snapshot=output?.[phase];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Construction reading" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Construction reading</strong><button onClick={close}>Close</button></header>
 <label>Construction entry <input value={entry} placeholder="Current workspace entry" onChange={event=>{retireReading();setEntry(event.target.value);}}/></label><label>Contributor or output (optional) <input value={focus} onChange={event=>{retireReading();setFocus(event.target.value);}}/></label>
 <button disabled={model.loading||acquiring} title={model.loading ? "Wait for the current reading to load." : acquiring ? "A source capture is in progress." : undefined} onClick={()=>void readCaptured()}>Read captured drafts</button><p role="status">{model.notice}</p>
 {value&&<><p>Captured entry: {value.entry}{value.focus?` · focus: ${value.focus}`:""}</p><p>Draft changes compare two reconstructions. Use Files to inspect the working copy.</p><p>{value.output_count} related outputs{value.outputs_truncated?' · first 300 shown':''}. {value.test_count} detected tests{value.tests_truncated?' · test list truncated':''}. Evidence: {value.evidence_status}. Outputs have not been written.</p><p>{value.test_relationship}</p><details><summary>Detected test declarations</summary>{value.tests.map(test=><p key={test.id}>{test.name} · {test.source?.path??test.target.path}:{test.source?.line??test.target.line}</p>)}</details>{value.notices.map((notice,index)=><p key={index}>{notice}</p>)}{value.unused_sources.length>0&&<p>{value.unused_sources.length} supplied drafts did not contribute to this entry.</p>}
 <label>Constructed output <select value={output?.path??''} onChange={event=>setSelected(event.target.value)}>{value.outputs.map(item=><option key={item.path} value={item.path}>{item.change} in draft · {item.path}</option>)}</select></label>
 {output&&<article><strong>{output.change} in draft · {output.path}</strong><p>Contributors: {output.contributors.join(', ')||'No compiler source origins reported.'}</p><button onClick={()=>setPhase('before')}>From saved entry</button><button onClick={()=>setPhase('after')}>From captured drafts</button>
 {snapshot?<><p>{snapshot.kind} · {snapshot.size} bytes · executable {String(snapshot.executable)}</p><p style={{overflowWrap:"anywhere"}}>SHA-256 <code>{snapshot.hash}</code></p><pre style={{whiteSpace:'pre-wrap'}}>{snapshot.preview??'Binary output; text preview unavailable.'}</pre>{snapshot.preview_truncated&&<p>Preview abbreviated.</p>}</>:<p>Output absent from this construction.</p>}
 <details><summary>Compiler contribution ranges</summary>{output.origins.map((origin,index)=><p key={index}>{origin.phase} · {origin.path} bytes {origin.source_bytes.join('–')} → output bytes {origin.output_bytes.join('–')} · {origin.exact?'equal byte slices':'reported origin; exact bytes unconfirmed'}{origin.fragment&&` · declared ${origin.fragment.kind} #${origin.fragment.name}`}{origin.source_point&&<button aria-label={`Open ${origin.phase} contributor ${origin.path}`} onClick={()=>{const current=model.sourceLink(value);void navigate(origin.source_point!,current).catch(error=>{if(current()){model.notice=String(error);paint();}});}}>Open contributor</button>}</p>)}{output.origins_truncated&&<p>First 100 origins shown.</p>}</details></article>}</>}
 <BehaviorReadingPanel model={behavior} terminal={terminal}/>
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
it('does not reflect a compiler reply after the original default selection retired',async()=>{
 const deferred:((reply:ConstructionReading)=>void)[]=[];
 const model=new ConstructionReadingModel(()=>new Promise(resolve=>deferred.push(resolve)));
 let current=true;
 const pending=model.read({entry:'moved/project.md',sources:{},focus:null},()=>current);
 current=false;
 deferred[0](value('moved/project.md'));
 await pending;
 expect(model.value).toBeNull();
 expect(model.notice).toContain('Original construction selection changed');
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
   driver.act("construction-open");text=driver.act("construction-read",entry="",focus="part.md");assert "modified" in text and "output.txt" in text and "not_queried" in text,text
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

## Keep optional relationships independent of readable compiler outputs

The connected reading uses its own asynchronous model. A failed metadata/provider read leaves compiler previews usable. A protocol-1 reading that omits verification availability remains readable, but only explicit Boolean true authorizes an admission attempt; present non-Boolean flags are rejected. Verification captures the retained namespace and exact buffer identities, consumes a request before transport, and preserves its request link even if the popover closes. Refresh observes only the selected retained process, never restarts it. Unsupported declared checks can use an explicitly chosen engine profile; the relationship is user-declared, not inferred coverage.

<ui:copy id="behavior-reading-browser-model">
import {request,ApiError} from '../api/client';
import {BehaviorSourceReading} from './BehaviorSourceReading';
import type {EntrySelection} from './WorkspaceEntrySelection';
export interface BehaviorContext {version:1;entry:string;sources:Record<string,string>;focus:string|null;missing_sources:string[];unaddressed_sources:Record<string,string>;saved_guards:Record<string,string>;entry_selection?:EntrySelection}
export interface BehaviorNode {id:string;kind:string;path:string;source_path?:string|null;fragment_id?:string|null;text_origin?:string|null;reading_notice?:string|null;label:string;status:string;hash:string|null;preview:string|null}
export interface BehaviorEdge {from:string;to:string;relationship:string;basis:string;reason:string;authority:string}
export interface BehaviorCheck {id:string;label:string;path:string;available:boolean;reason:string;candidate_id:string|null}
export interface BehaviorReading {version:1;behavior_protocol:1;namespace:string;engine_workspace_id:string;workspace_id:string;reading_fingerprint:string;context:BehaviorContext;graph:{nodes:Record<string,BehaviorNode>;edges:BehaviorEdge[];checks:BehaviorCheck[];notices:string[];truncated:boolean};impact:{basis:string;reason:string;nodes:string[];via:BehaviorEdge[];truncated?:boolean}|null;evidence:{status:string;records:Record<string,unknown>[];truncated?:boolean;notice?:string};evidence_limit:string;verification_available?:boolean;receipts?:{records:VerificationReceipt[];truncated:boolean}}
export interface VerificationReceipt {workspace_current?:boolean;observation_revision?:number;observation_engine_workspace_id?:string;version:1;request_id:string;engine_workspace_id?:string;origin?:string;workspace_id:string;reading_fingerprint:string;check_id:string;label:string;status:string;outcome:string;notice:string;error?:string;output?:string;output_truncated?:boolean;exit_code?:number;relationship?:string;profile?:{id:string;fingerprint:string}|null;freshness?:string;terminal?:{id:string}}
export interface ProfileChoice {id:string;name:string;fingerprint:string}
export type BehaviorProvider=(path:string,body?:unknown)=>Promise<unknown>;
const provider:BehaviorProvider=(path,body)=>request(body===undefined?'GET':'POST',path,body);
function behaviorTextPrefix(text:string,limit:number):string {let out='',count=0;for(const scalar of text){if(count++===limit)break;out+=scalar;}return out;}
function sameBehaviorContext(value:unknown,expected:BehaviorContext):boolean {if(!value||typeof value!=='object'||Array.isArray(value))return false;const row=value as BehaviorContext;if(Object.keys(row).length!==(expected.entry_selection===undefined?7:8)||canonical(row.entry_selection)!==canonical(expected.entry_selection)||row.version!==1||row.entry!==expected.entry||row.focus!==expected.focus||!Array.isArray(row.missing_sources)||row.missing_sources.length!==expected.missing_sources.length||!row.missing_sources.every((path,index)=>path===expected.missing_sources[index])||!row.sources||typeof row.sources!=='object'||Array.isArray(row.sources))return false;const same=(left:unknown,right:Record<string,string>)=>!!left&&typeof left==='object'&&!Array.isArray(left)&&Object.keys(left).length===Object.keys(right).length&&Object.entries(left).every(([path,text])=>typeof text==='string'&&right[path]===text);return same(row.sources,expected.sources)&&same(row.unaddressed_sources,expected.unaddressed_sources)&&same(row.saved_guards,expected.saved_guards);}
const canonical=(value:unknown):string=>JSON.stringify(value&&typeof value==='object'?Array.isArray(value)?value.map(item=>JSON.parse(canonical(item))):Object.fromEntries(Object.entries(value).sort(([a],[b])=>a.localeCompare(b)).map(([key,item])=>[key,JSON.parse(canonical(item))])):value);
const digest=(value:unknown):value is string=>typeof value==='string'&&/^[0-9a-f]{64}$/.test(value);
function immutable<T>(value:T):T {if(value&&typeof value==='object'){for(const child of Object.values(value))immutable(child);Object.freeze(value);}return value;}
export class BehaviorReadingModel {
 value:BehaviorReading|null=null;notice='';loading=false;profiles:ProfileChoice[]=[];receipts:VerificationReceipt[]=[];readonly source:BehaviorSourceReading;private ticket=0;private current:()=>boolean=()=>false;private consumed=new Map<string,{id:string;ticket:number}>();changed:()=>void=()=>{};
 constructor(private call:BehaviorProvider=provider,private workspace?:string,private engineWorkspace?:string){this.source=new BehaviorSourceReading(call,()=>this.changed());}
 close(){this.ticket++;this.current=()=>false;this.loading=false;this.profiles=[];this.value=null;this.source.close();this.notice='Captured reading retired. Verification requests and their process-only outcomes remain retained.';}
 async read(context:BehaviorContext,current:()=>boolean){const input=immutable({...context,sources:{...context.sources},unaddressed_sources:{...context.unaddressed_sources},saved_guards:{...context.saved_guards},missing_sources:[...context.missing_sources]}),ticket=++this.ticket;this.source.retire();this.current=()=>false;this.loading=true;this.notice='Reading connected behavior…';this.profiles=[];this.changed();try{const value=await this.call('/api/behavior/reading',input) as BehaviorReading;if(ticket!==this.ticket)return;if(value.version!==1||value.behavior_protocol!==1||value.verification_available!==undefined&&typeof value.verification_available!=='boolean'||(!digest(value.workspace_id)&&!(value.verification_available===false&&value.workspace_id===''))||typeof value.namespace!=='string'||!value.namespace||this.workspace!==undefined&&value.namespace!==this.workspace||typeof value.engine_workspace_id!=='string'||!/^[0-9a-f]{32}$/.test(value.engine_workspace_id)||this.engineWorkspace!==undefined&&value.engine_workspace_id!==this.engineWorkspace||!digest(value.reading_fingerprint)||!sameBehaviorContext(value.context,input)||!value.graph||!Array.isArray(value.graph.checks)||!Array.isArray(value.graph.edges)||value.graph.checks.some(check=>!check||typeof check.id!=='string'||typeof check.path!=='string'||typeof check.label!=='string'||typeof check.available!=='boolean'||typeof check.reason!=='string'))throw Error('Unsupported or foreign behavior reading. No verification was started.');this.source.close();this.value=immutable({...value,context:input,receipts:undefined});this.current=current;for(const receipt of [...value.receipts?.records??[]].reverse())if(receipt.version===1&&(receipt.workspace_id===value.workspace_id||receipt.origin==='unknown')&&digest(receipt.reading_fingerprint)&&typeof receipt.check_id==='string'&&typeof receipt.status==='string'&&typeof receipt.outcome==='string'&&typeof receipt.request_id==='string'&&/^[0-9a-f]{32}$/.test(receipt.request_id)){const owned=receipt.workspace_id===value.workspace_id&&receipt.origin!=='unknown',sameEngine=receipt.engine_workspace_id===value.engine_workspace_id;this.retain(immutable({...receipt,terminal:owned&&sameEngine?receipt.terminal:undefined,freshness:!owned?'origin-unknown':!sameEngine?'historical-engine':receipt.reading_fingerprint===value.reading_fingerprint?'current-reading':'historical-reading'}));if(owned&&!['process-finished','admission-refused','request-refused'].includes(receipt.status)){const key=JSON.stringify([receipt.reading_fingerprint,receipt.check_id,receipt.profile?.id,receipt.profile?.fingerprint]);if(this.consumed.size<256)this.consumed.set(key,{id:receipt.request_id,ticket:this.ticket});}}this.notice=current()?'Captured declared relationships and observed construction.':'Captured reading is historical: editor ownership or source changed.';}catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.loading=false;this.changed();}}}
 canVerify(){return !!this.value&&this.value.verification_available===true&&!this.loading&&this.current();}
 private admission(value:BehaviorReading){if(value.verification_available!==true)throw Error('This captured reading has no confirmed verification authority. Refresh/update the engine before verification.');if(this.value!==value||!this.current())throw Error('Source, editor ownership or workspace changed. Refresh the reading before verification.');return {engine_workspace_id:value.engine_workspace_id,workspace_id:value.workspace_id,reading_fingerprint:value.reading_fingerprint,context:value.context};}
 async chooseProfiles(){const value=this.value;if(!value)return;const ticket=this.ticket;this.notice='Reading explicit launch profiles…';this.changed();try{const body=this.admission(value),response=await this.call('/api/behavior/profiles',body) as {fingerprint:string;profiles:{id:string;name:string}[];notices?:string[]};if(ticket!==this.ticket)return;this.admission(value);if(!digest(response.fingerprint)||!Array.isArray(response.profiles))throw Error('Launch profile provider is unavailable.');this.profiles=response.profiles.map(profile=>({id:profile.id,name:profile.name,fingerprint:response.fingerprint}));this.notice=(this.profiles.length?'Choose an engine-discovered profile explicitly; its association with a check is user-declared.':'No engine-discovered profile is available; the declared check remains unsupported.')+(response.notices?.length?' '+response.notices.join(' · '):'');}catch(error){this.notice=String(error);}this.changed();}
 async run(checkId:string,profile?:ProfileChoice){const value=this.value;if(!value)return;let requestId='';try{const body=this.admission(value),check=value.graph.checks.find(check=>check.id===checkId);if(!check||!check.available&&!profile)throw Error(check?.reason||'Unknown captured check.');if(profile&&!this.profiles.some(choice=>choice.id===profile.id&&choice.fingerprint===profile.fingerprint))throw Error('Profile choice changed. Choose it again.');const key=JSON.stringify([value.reading_fingerprint,checkId,profile?.id,profile?.fingerprint]);const prior=this.consumed.get(key);if(prior&&!(prior.ticket!==this.ticket&&this.receipts.some(receipt=>receipt.request_id===prior.id&&['process-finished','admission-refused','request-refused'].includes(receipt.status)))){this.notice='This verification request is already consumed. Refresh its retained outcome; no second process was started.';this.changed();return;}requestId=crypto.randomUUID().replaceAll('-','');if(this.consumed.size>=256&&!prior)throw Error('Verification request capacity reached. Retained uncertain requests were not forgotten; inspect their outcomes.');this.consumed.set(key,{id:requestId,ticket:this.ticket});const pending:VerificationReceipt={version:1,request_id:requestId,engine_workspace_id:value.engine_workspace_id,workspace_id:value.workspace_id,reading_fingerprint:value.reading_fingerprint,check_id:checkId,label:check.label,profile:profile?{id:profile.id,fingerprint:profile.fingerprint}:null,status:'request-pending',outcome:'unknown',notice:'An uncertain response never permits an automatic retry.'};this.retain(pending);const raw=await this.call('/api/behavior/verify',{version:1,...body,check_id:checkId,request_id:requestId,...profile?{profile:{id:profile.id,fingerprint:profile.fingerprint}}:{}});this.accept(raw,pending);this.notice='Verification receipt retained. Process outcomes are not collected test verdicts.';}catch(error){if(requestId&&error instanceof ApiError&&error.status>=400&&error.status<500){const prior=this.receipts.find(receipt=>receipt.request_id===requestId);if(prior)this.retain({...prior,status:'request-refused',outcome:'not-started',error:String(error)});}this.notice=String(error)+(requestId?' Inspect the retained request; do not start a replacement after an uncertain response.':'');}this.changed();}
 private retain(receipt:VerificationReceipt){const text=typeof receipt.output==='string'?behaviorTextPrefix(receipt.output,4096):undefined;const row:VerificationReceipt=immutable({workspace_current:receipt.workspace_current,observation_revision:receipt.observation_revision,observation_engine_workspace_id:receipt.observation_engine_workspace_id,version:1,request_id:receipt.request_id,engine_workspace_id:receipt.engine_workspace_id,origin:receipt.origin,workspace_id:receipt.workspace_id,reading_fingerprint:receipt.reading_fingerprint,check_id:receipt.check_id,label:typeof receipt.label==='string'?receipt.label.slice(0,512):'Retained verification',status:receipt.status,outcome:receipt.outcome,notice:typeof receipt.notice==='string'?receipt.notice.slice(0,2000):'Retained process observation',error:typeof receipt.error==='string'?receipt.error.slice(0,2000):undefined,output:text,output_truncated:receipt.output_truncated||typeof receipt.output==='string'&&receipt.output!==text,exit_code:receipt.exit_code,relationship:typeof receipt.relationship==='string'?receipt.relationship.slice(0,512):undefined,profile:receipt.profile?{id:receipt.profile.id,fingerprint:receipt.profile.fingerprint}:null,freshness:receipt.freshness,terminal:typeof receipt.terminal?.id==='string'?{id:receipt.terminal.id}:undefined});this.receipts=[row,...this.receipts.filter(value=>value.request_id!==row.request_id)].slice(0,50);this.changed();}
 async readSource(nodeId:string,offset?:number){const value=this.value,node=value?.graph.nodes[nodeId];if(value&&node)await this.source.read(value,node,offset,()=>this.value===value);}
 private accept(raw:unknown,pending:VerificationReceipt){const value=raw as VerificationReceipt;if(!value||value.version!==1||value.request_id!==pending.request_id||value.engine_workspace_id!==pending.engine_workspace_id||value.workspace_id!==pending.workspace_id||value.reading_fingerprint!==pending.reading_fingerprint||value.check_id!==pending.check_id||canonical(value.profile??null)!==canonical(pending.profile??null)||typeof value.status!=='string'||typeof value.outcome!=='string')throw Error('Foreign or malformed verification receipt. Keep the original request ID for inspection.');if(value.workspace_current!==undefined&&typeof value.workspace_current!=='boolean'||value.observation_revision!==undefined&&(!Number.isSafeInteger(value.observation_revision)||value.observation_revision<0)||value.observation_engine_workspace_id!==undefined&&value.observation_engine_workspace_id!==pending.engine_workspace_id)throw Error('Malformed owned-process observation scope.');const prior=this.receipts.find(row=>row.request_id===pending.request_id);if(prior?.observation_revision!==undefined&&(value.observation_revision===undefined||value.observation_revision<prior.observation_revision)||prior?.status==='process-finished'&&value.status!=='process-finished')return;if(value.workspace_current===false)this.current=()=>false;this.retain(immutable(value));}
 async openProcess(receipt:VerificationReceipt,open:(id:string)=>void){try{this.accept(await this.call('/api/behavior/verification?id='+receipt.request_id),receipt);const current=this.receipts.find(row=>row.request_id===receipt.request_id);if(!current?.terminal?.id)throw Error('The retained process is no longer in this engine. Its historical outcome remains readable; no replacement was adopted.');open(current.terminal.id);}catch(error){this.notice=String(error);this.changed();}}
 async refresh(receipt:VerificationReceipt){try{this.accept(await this.call('/api/behavior/verification?id='+receipt.request_id),receipt);this.notice='Retained process observation refreshed; no check was restarted.';}catch(error){this.notice=String(error);}this.changed();}
}
const retained=new Map<string,BehaviorReadingModel>();
export function retainedBehaviorReading(workspace:string,engineWorkspace:string){const key=JSON.stringify([workspace,engineWorkspace]);let model=retained.get(key);if(!model){model=new BehaviorReadingModel(provider,workspace,engineWorkspace);retained.set(key,model);if(retained.size>16)retained.delete(retained.keys().next().value!);}return model;}
</ui:copy>

## Show why a check is related before running it

A compact section in the existing floating construction reading exposes each relationship's basis, explanation gaps and historical status. Captured snippets allow contract/explanation/implementation/test/evidence reading without replacing the source editor. The process pane and retained request remain separate from collector test verdicts.

<ui:copy id="behavior-reading-browser-view">
import {useState} from 'react';
import type {BehaviorReadingModel} from './BehaviorReading';
export function BehaviorReadingPanel({model,terminal}:{model:BehaviorReadingModel;terminal?:(id:string)=>void}){
 const [nodeId,setNodeId]=useState(''),[checkId,setCheckId]=useState(''),[profileId,setProfileId]=useState('');const value=model.value,nodes=Object.values(value?.graph.nodes??{}),node=nodes.find(node=>node.id===nodeId)??nodes[0],check=value?.graph.checks.find(check=>check.id===checkId)??value?.graph.checks[0];
 return <details open><summary>Connected behavior and relevant verification</summary><p role="status" aria-label="Connected reading status">{model.notice}</p>{value&&<><p>{value.evidence_limit}</p>{value.graph.notices.map((notice,index)=><p key={index}>{notice}</p>)}{value.graph.truncated&&<p>Relationship reading truncated; omitted relationships remain unknown.</p>}<label>Related reading <select aria-label="Related reading" value={node?.id??''} onChange={event=>setNodeId(event.target.value)}>{nodes.map(node=><option key={node.id} value={node.id}>{node.kind} · {node.source_path??node.path} · {node.label} · {node.status}</option>)}</select></label>{node&&<article><strong>{node.source_path??node.path}</strong>{node.fragment_id&&<p>Declared fragment: {node.fragment_id}</p>}<p>{node.status} · {node.text_origin??'origin unknown'} · captured hash {node.hash??'unknown'}</p>{model.source.page?.node_id===node.id?<><p>{model.source.page.origin} · {model.source.page.state} · {model.source.page.target_status}</p><pre aria-label="Captured full reading" style={{whiteSpace:'pre-wrap'}}>{model.source.page.text}</pre><p>Unicode scalars {model.source.page.offset}–{model.source.page.offset+Array.from(model.source.page.text).length} of {model.source.page.total}</p><button disabled={model.source.loading||model.source.page.offset===0} title={model.source.loading ? "Wait for the current reading to load." : model.source.page.offset===0 ? "You’re on the first page." : undefined} onClick={()=>void model.readSource(node.id,Math.max(0,model.source.page!.offset-4000))}>Previous source page</button><button disabled={model.source.loading||model.source.page.next===null} title={model.source.loading ? "Wait for the current reading to load." : model.source.page.next===null ? "You’re on the last page." : undefined} onClick={()=>void model.readSource(node.id,model.source.page!.next!)}>Next source page</button></>:<pre style={{whiteSpace:'pre-wrap'}}>{node.preview??'Readable preview unavailable; this relationship does not establish missing content.'}</pre>}<button disabled={model.source.loading||!node.hash} title={model.source.loading ? "Wait for the current reading to load." : !node.hash ? "This reading has no captured source hash." : undefined} onClick={()=>void model.readSource(node.id)}>Read full captured source</button>{node.reading_notice&&<p>{node.reading_notice}</p>}<p>Previews are abbreviated. Full readings remain bound to the captured node and source hash.</p>{model.source.notice&&<p role="status" aria-label="Source reading status">{model.source.notice}</p>}{value.graph.edges.filter(edge=>edge.from===node.id||edge.to===node.id).map((edge,index)=><p key={index}>{edge.relationship} · {edge.basis}: {edge.reason} Authority: {edge.authority}</p>)}</article>}{value.impact&&<details><summary>Why this change is related</summary><p>{value.impact.basis}: {value.impact.reason}</p>{value.impact.truncated&&<p>Impact traversal reached its depth limit; more relationships may be unknown.</p>}{value.impact.via.map((edge,index)=><p key={index}>{nodes.find(node=>node.id===edge.from)?.label} → {nodes.find(node=>node.id===edge.to)?.label}: {edge.reason}</p>)}</details>}<details><summary>Retained evidence · {value.evidence.status}</summary><p>{value.evidence.notice}</p>{value.evidence.records.map((record,index)=><pre key={index} style={{whiteSpace:'pre-wrap'}}>{JSON.stringify(record,null,2)}</pre>)}{value.evidence.truncated&&<p>Evidence projection truncated; omitted records remain unknown.</p>}</details><label>Related check <select aria-label="Related check" value={check?.id??''} onChange={event=>setCheckId(event.target.value)}>{value.graph.checks.map(check=><option key={check.id} value={check.id}>{check.label} · {check.available?'source candidate':'unsupported automatic provider'}</option>)}</select></label>{check?<><p>{check.reason}</p><button disabled={!check.available||!model.canVerify()} title={!check.available ? check.reason || "No automatic provider is available for this check." : !model.canVerify() ? "Verification requires a current captured reading and an idle verifier." : undefined} onClick={()=>void model.run(check.id)}>Run selected relevant test</button><button disabled={!model.canVerify()} title={!model.canVerify() ? "Verification requires a current captured reading and an idle verifier." : undefined} onClick={()=>void model.chooseProfiles()}>Choose profile for this check</button>{model.profiles.length>0&&<><label>Explicit verification profile <select aria-label="Explicit verification profile" value={profileId} onChange={event=>setProfileId(event.target.value)}><option value="">Choose a profile</option>{model.profiles.map(profile=><option key={profile.id} value={profile.id}>{profile.name}</option>)}</select></label><button disabled={!profileId||!model.canVerify()} title={!profileId ? "Choose a verification profile first." : !model.canVerify() ? "Verification requires a current captured reading and an idle verifier." : undefined} onClick={()=>{const profile=model.profiles.find(profile=>profile.id===profileId);if(profile)void model.run(check.id,profile);}}>Run user-declared profile for selected check</button></>}</>:<p>No checks have an explicit relationship to this reading. Undeclared test filenames are not relevance evidence.</p>}</>}{model.receipts.length>0&&<details open><summary>Retained verification requests</summary>{model.receipts.map(receipt=><article key={receipt.request_id}><strong>{receipt.label} · {receipt.status}</strong><p>{receipt.relationship} · {receipt.freshness??'captured request'} · {receipt.outcome}{receipt.exit_code!==undefined?` · process exit ${receipt.exit_code}`:''}</p><p>{receipt.notice} {receipt.error}</p><code>{receipt.request_id}</code><button onClick={()=>void model.refresh(receipt)}>Refresh retained outcome</button>{receipt.terminal?.id&&terminal&&<button onClick={()=>void model.openProcess(receipt,terminal)}>Open verification process</button>}{receipt.output&&<pre style={{whiteSpace:'pre-wrap'}}>{receipt.output}</pre>}{receipt.output_truncated&&<p>Process output abbreviated.</p>}</article>)}</details>}</details>;
}
</ui:copy>
<ui:file path="apps/web/src/simple/BehaviorReading.ts"><ui:paste select="#behavior-reading-browser-model"/></ui:file>
<ui:file path="apps/web/src/simple/BehaviorReadingView.tsx"><ui:paste select="#behavior-reading-browser-view"/></ui:file>


## Exercise refusal and retained-request lifetimes independently of rendering

These focused checks keep stale/foreign readings, transport uncertainty and unsupported declarations from becoming new processes. They also distinguish a retained process exit from a collected test result, and test raw-wire protocol refusal before duplicate keys or numeric lexemes are lost.

<ui:copy id="behavior-reading-browser-tests">
import {it,expect} from 'vitest';
import {createElement} from 'react';
import {render,screen,cleanup} from '@testing-library/react';
import {BehaviorReadingPanel} from './BehaviorReadingView';
import {BehaviorReadingModel,type BehaviorContext,type BehaviorReading} from './BehaviorReading';
import {parseProtocolJson} from '../api/client';
const context:BehaviorContext={version:1,entry:'project.md',focus:'a.py',sources:{'project.md':'source'},missing_sources:[],unaddressed_sources:{},saved_guards:{}};
const reading=():BehaviorReading=>({version:1,behavior_protocol:1,verification_available:true,namespace:'/workspace',engine_workspace_id:'e'.repeat(32),workspace_id:'a'.repeat(64),reading_fingerprint:'b'.repeat(64),context:structuredClone(context),graph:{nodes:{},edges:[],checks:[{id:'check',label:'selected',path:'test_a.py',available:true,reason:'explicit pair',candidate_id:'candidate'}],notices:[],truncated:false},impact:null,evidence:{status:'missing',records:[]},evidence_limit:'No verdict inferred'});
it('binds all retained source classes and original physical guards in the immutable context',async()=>{const input:BehaviorContext={...context,unaddressed_sources:{opaque:'notes'},saved_guards:{'project.md':'original-guard'}};let resolve!:(value:unknown)=>void;const model=new BehaviorReadingModel(()=>new Promise(done=>resolve=done));const pending=model.read(input,()=>true);input.unaddressed_sources.opaque='changed';input.saved_guards['project.md']='changed';resolve({...reading(),context:{...context,unaddressed_sources:{opaque:'notes'},saved_guards:{'project.md':'original-guard'}}});await pending;expect(model.value?.context.unaddressed_sources.opaque).toBe('notes');expect(model.value?.context.saved_guards['project.md']).toBe('original-guard');const prior=model.value;const invalid=new BehaviorReadingModel(async()=>({...reading(),context:{...context,unaddressed_sources:null}}));await invalid.read(context,()=>true);expect(invalid.value).toBeNull();expect(prior).not.toBeNull();});
it('keeps the readable previous graph after capability refusal and rejects foreign protocol/input',async()=>{let answer:unknown=reading();const model=new BehaviorReadingModel(async()=>answer);await model.read(context,()=>true);const previous=model.value;answer={...reading(),behavior_protocol:2};await model.read(context,()=>true);expect(model.value).toBe(previous);expect(model.notice).toContain('Unsupported');answer={...reading(),context:{...context,entry:'foreign.md'}};await model.read(context,()=>true);expect(model.value).toBe(previous);});
it('retires held reads on supersession and close, preserving retained outcomes',async()=>{const waits:((value:unknown)=>void)[]=[];const model=new BehaviorReadingModel(()=>new Promise(resolve=>waits.push(resolve)));const old=model.read(context,()=>true),latest=model.read(context,()=>true);waits[1](reading());await latest;const value=model.value;waits[0]({...reading(),reading_fingerprint:'c'.repeat(64)});await old;expect(model.value).toBe(value);const closed=model.read(context,()=>true);model.close();waits[2](reading());await closed;expect(model.value).toBeNull();await model.run('check');expect(model.receipts).toEqual([]);});
it('pins source/owner lifetime and consumes an uncertain request instead of fresh retry',async()=>{let current=true,starts=0;const model=new BehaviorReadingModel(async(path)=>{if(path.endsWith('reading'))return reading();starts++;throw Error('transport uncertain');});await model.read(context,()=>current);current=false;await model.run('check');expect(starts).toBe(0);current=true;await model.run('check');expect(starts).toBe(1);expect(model.receipts[0].status).toBe('request-pending');await model.run('check');expect(starts).toBe(1);expect(model.notice).toContain('already consumed');});
it('retains only an owned process after source retirement and rejects a held older observation',async()=>{let response:unknown;const model:BehaviorReadingModel=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return reading();if(path.endsWith('verify'))return {...body as object,label:'selected',status:'process-running',outcome:'unknown',notice:'process-only',observation_revision:1};return response;});await model.read(context,()=>true);await model.run('check');const original=model.receipts[0];response={...original,workspace_current:false,observation_revision:3,observation_engine_workspace_id:original.engine_workspace_id,status:'process-finished',exit_code:0,terminal:{id:'owned-process'}};await model.refresh(original);expect(model.canVerify()).toBe(false);expect(model.receipts[0].terminal?.id).toBe('owned-process');response={...original,observation_revision:2};await model.refresh(original);expect(model.receipts[0].status).toBe('process-finished');response={...model.receipts[0],workspace_current:'false',observation_revision:4};await model.refresh(original);expect(model.receipts[0].observation_revision).toBe(3);expect(model.notice).toMatch(/Malformed/);});
it('validates retained request binding and refreshes only the same process observation',async()=>{let starts=0;const model:BehaviorReadingModel=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return reading();if(path.endsWith('verify')){starts++;return {...body as object,label:'selected',status:'process-running',outcome:'unknown',notice:'process-only',terminal:{id:'terminal'}};}return {...model.receipts[0],status:'process-finished',outcome:'process-only',exit_code:0};});await model.read(context,()=>true);await model.run('check');model.close();await model.refresh(model.receipts[0]);expect(starts).toBe(1);expect(model.receipts[0].outcome).toBe('process-only');expect(model.receipts[0].exit_code).toBe(0);});
it('does not turn an unsupported declaration into a source Run; requires an actual explicit profile',async()=>{let starts=0;const value=reading();value.graph.checks[0].available=false;const model=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return value;if(path.endsWith('profiles'))return {fingerprint:'c'.repeat(64),profiles:[{id:'profile',name:'Manual check'}]};starts++;return {...body as object,label:'selected',status:'process-running',outcome:'unknown',notice:'process-only'};});await model.read(context,()=>true);await model.run('check');expect(starts).toBe(0);await model.chooseProfiles();await model.run('check',model.profiles[0]);expect(starts).toBe(1);});
it('keeps retained content readable but retires its execution lease during a held or failed newer read',async()=>{let answer:unknown=reading(),starts=0;const model=new BehaviorReadingModel(async(path)=>{if(path.endsWith('reading')){if(answer instanceof Error)throw answer;return answer;}starts++;throw Error('should not execute');});await model.read(context,()=>true);answer=Error('optional provider failed');await model.read({...context,sources:{'project.md':'new draft'}},()=>true);expect(model.value).not.toBeNull();await model.run('check');expect(starts).toBe(0);});
it('permits an explicit fresh reading after a known terminal outcome but never retries an uncertain one',async()=>{let starts=0;const model:BehaviorReadingModel=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return reading();if(path.endsWith('verify')){starts++;return {...body as object,label:'selected',status:'process-finished',outcome:'process-only',notice:'not a verdict'};}return model.receipts[0];});await model.read(context,()=>true);await model.run('check');await model.run('check');expect(starts).toBe(1);await model.read(context,()=>true);await model.run('check');expect(starts).toBe(2);});
it('names related reading/check/profile controls independently of their option labels',()=>{const model=new BehaviorReadingModel();model.value=reading();model.value.graph.nodes={source:{id:'source',kind:'source',path:'a.py',label:'source option',status:'captured',hash:null,preview:'value=41'}};model.profiles=[{id:'profile',name:'Profile option',fingerprint:'c'.repeat(64)}];render(createElement(BehaviorReadingPanel,{model}));expect((screen.getByLabelText('Related check',{exact:true}) as HTMLSelectElement).value).toBe('check');expect((screen.getByLabelText('Related reading',{exact:true}) as HTMLSelectElement).value).toBe('source');expect(screen.getByLabelText('Explicit verification profile',{exact:true})).toBeTruthy();cleanup();});
it('never opens a stale process link without revalidating its exact owned receipt',async()=>{const model:BehaviorReadingModel=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return reading();if(path.endsWith('verify'))return {...body as object,label:'selected',status:'process-running',outcome:'unknown',notice:'captured',terminal:{id:'owned'}};return {...model.receipts[0],terminal:undefined,status:'process-unavailable',outcome:'unknown'};});await model.read(context,()=>true);await model.run('check');const opened:string[]=[];await model.openProcess(model.receipts[0],id=>opened.push(id));expect(opened).toEqual([]);expect(model.notice).toContain('no longer');});
it('rejects present nonboolean verification availability instead of promoting it to supported',async()=>{for(const available of ['false',null,0]){const model=new BehaviorReadingModel(async()=>({...reading(),verification_available:available}));await model.read(context,()=>true);expect(model.value).toBeNull();expect(model.notice).toContain('Unsupported');}});
it('rejects duplicate and noncanonical behavior protocol lexemes before JSON loses them',()=>{expect(()=>parseProtocolJson('{"version":1,"behavior_protocol":1e0}')).toThrow('canonical');expect(()=>parseProtocolJson('{"version":1,"behavior_protocol":1,"behavior_protocol":1}')).toThrow();});
it('binds opaque physical identity to exact namespace and engine echoes instead of a path digest',async()=>{let value:BehaviorReading=reading(),starts=0;const model=new BehaviorReadingModel(async(path,body)=>{if(path.endsWith('reading'))return value;starts++;return {...body as object,label:'selected',status:'process-finished',outcome:'process-only',notice:'Not a test verdict'};},'/workspace','e'.repeat(32));await model.read(context,()=>true);expect(model.value?.workspace_id).toBe('a'.repeat(64));const accepted=model.value;for(const changed of [{namespace:'/foreign'},{engine_workspace_id:'f'.repeat(32)},{workspace_id:''}]){value={...reading(),...changed};await model.read(context,()=>true);expect(model.value).toBe(accepted);expect(model.canVerify()).toBe(false);await model.run('check');expect(starts).toBe(0);}});
it('keeps legacy path-only receipts readable without borrowing their live process identity',async()=>{const value=reading();value.receipts={truncated:false,records:[{version:1,request_id:'1'.repeat(32),workspace_id:'f'.repeat(64),reading_fingerprint:'c'.repeat(64),check_id:'check',label:'Legacy retained process',status:'process-finished',outcome:'process-only',notice:'Retained only',origin:'unknown',terminal:{id:'foreign-terminal'}}]};const model=new BehaviorReadingModel(async()=>value);await model.read(context,()=>true);expect(model.receipts[0].label).toBe('Legacy retained process');expect(model.receipts[0].freshness).toBe('origin-unknown');expect(model.receipts[0].terminal).toBeUndefined();});

it('keeps readable declared relationships when physical root authority is explicitly unavailable',async()=>{const value={...reading(),workspace_id:'',verification_available:false};const model=new BehaviorReadingModel(async()=>value,'/workspace','e'.repeat(32));await model.read(context,()=>true);expect(model.value?.graph.checks.length).toBe(1);expect(model.canVerify()).toBe(false);});

it('reconstructs consumed choices from a retained unavailable process instead of fresh-retrying after reload',async()=>{const value=reading();value.receipts={truncated:false,records:[{version:1,request_id:'2'.repeat(32),engine_workspace_id:'d'.repeat(32),workspace_id:value.workspace_id,reading_fingerprint:value.reading_fingerprint,check_id:'check',label:'Uncertain old process',status:'process-unavailable',outcome:'unknown',notice:'No current process proof',origin:'current-workspace'}]};let starts=0;const model=new BehaviorReadingModel(async path=>{if(path.endsWith('reading'))return value;starts++;throw Error('must not retry');});await model.read(context,()=>true);await model.run('check');expect(starts).toBe(0);expect(model.notice).toContain('already consumed');expect(model.receipts[0].freshness).toBe('historical-engine');});

</ui:copy>
<ui:file path="apps/web/src/simple/BehaviorReading.test.ts"><ui:paste select="#behavior-reading-browser-tests"/></ui:file>

## Drive the compact reading against owned source and process receipts

This fixture declares its relationships explicitly, preserves an unrelated test as unlinked, and runs only its selected test or an explicitly chosen profile. The fixture owns every process and receipt it inspects; it never queries the checkout regression monitor.

<ui:copy id="behavior-reading-browser-real-engine">
"""Owned repository: connected readings, relevant verification and retained receipts."""
import argparse,hashlib,importlib.util,json,os,shutil,signal,socket,subprocess,tempfile,threading,time,urllib.error,urllib.request,urllib.parse,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def prepare(root):
 """Require an empty owned directory; never inspect checkout execution artifacts."""
 root.mkdir(parents=True,exist_ok=True);root=root.resolve()
 if any(root.iterdir()):raise ValueError('Behavior fixture requires an empty owned workspace')
 (root/'hickory-dogfood').mkdir();sibling=root/'sibling-ran.txt';manual=root/'manual-ran.txt';stale_manual=root/'stale-manual-ran.txt'
 implementation='def answer():\n    return 41\n';unrelated='def test_unrelated():\n    pass\n'
 test=f'from a import answer\nfrom pathlib import Path\ndef test_selected():\n    assert answer() == 41\ndef test_sibling():\n    Path({str(sibling)!r}).write_text("unwanted")\n'
 held_ready=root.parent/'held-ready.txt';held_release=root.parent/'held-release.txt'
 held_loop=f'while not Path({str(held_release)!r}).exists(): time.sleep(.05)'
 held_code=f'from pathlib import Path; import time; Path({str(held_ready)!r}).write_text("ready"); exec({held_loop!r}); print("owned process complete",flush=True)'
 launch={'version':1,'profiles':[{'id':'held-manual-check','name':'Held manual check','command':['python3','-c',held_code]},{'id':'manual-check','name':'Explicit manual check','command':['python3','-c',f'from pathlib import Path; Path({str(manual)!r}).write_text("ran")'],'default':True},{'id':'stale-manual-check','name':'Stale manual check','command':['python3','-c',f'from pathlib import Path; Path({str(stale_manual)!r}).write_text("unwanted")']}]}
 entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:project sources="source.md"/>\n</h:doc>\n'
 marker='Later explanation for the declared answer'
 source=('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="a.py"><h:paste select="#answer"/></h:file>\n<h:file path="test_a.py">\n'+test+'</h:file>\n<h:file path="test_undeclared.py">\n'+unrelated+'</h:file>\n<h:file path=".hick-launch.json">\n'+json.dumps(launch)+'\n</h:file>\n<h:file path="pyproject.toml">\n[tool.uv]\n</h:file>\n'+('Earlier retained source context 😀.\n'*220)+'\n## '+marker+'\n\nOnly the explicit compiler and metadata relationships authorize this reading.\n<h:copy id="answer">\n'+implementation+'</h:copy>\n</h:doc>\n')
 files={'project.md':entry,'source.md':source,'a.py':implementation,'test_a.py':test,'test_undeclared.py':unrelated,'.hick-launch.json':json.dumps(launch)+'\n','pyproject.toml':'[tool.uv]\n','behavior.md':'# Answer behavior\nReturn the declared answer.\n','.hick-tests.json':json.dumps({'groups':{'selected':{'selectors':['test_a.py::test_selected'],'labels':['owned-behavior']}}})}
 for path,text in files.items():(root/path).write_text(text)
 (root/'.venv').symlink_to(ROOT/'target/thread-debug-env',target_is_directory=True)
 metadata={'format':1,'contracts':{'answer':{'page':'behavior.md','title':'Answer behavior','disposition':'retained-requirement','references':['a.py'],'tests':[{'runner':'pytest','file':'test_a.py','name':'test_selected'},{'runner':'unconfigured-check','file':'manual.check','name':'Explicit unsupported check'}]}},'constructors':{'a.py':{'source':'source.md'}}}
 narrative={'format':1,'paired_test_modules':[{'implementation':'a.py','tests':'test_a.py','page':'source.md'}],'fragments':[{'id':'answer','page':'source.md','output':'a.py','symbol':'answer','basis':'needs-explanation'}]}
 (root/'hickory-dogfood/behavior-map.json').write_text(json.dumps(metadata));(root/'hickory-dogfood/narrative-map.json').write_text(json.dumps(narrative));(root/'.hickory-native-behavior-fixture').write_text('owned-native-behavior-v1\n');subprocess.run(['git','init','-q',str(root)],check=True)
 return {'workspace':str(root),'entry':'project.md','focus':'a.py','source_file':'source.md','source_text':source,'implementation_text':implementation,'selected':'test_selected','unsupported':'Explicit unsupported check','profile':'Explicit manual check','sibling':str(sibling),'manual':str(manual),'stale_manual':str(stale_manual),'later_marker':marker,'fragment':'answer','held_ready':str(held_ready),'held_release':str(held_release)}
def api(origin,path,body=None,expected=200):
 request=urllib.request.Request(origin+path,data=None if body is None else json.dumps(body).encode(),headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(request,timeout=30) as response:value=json.load(response)
  assert expected==200,(expected,value);return value
 except urllib.error.HTTPError as error:
  value=error.read().decode();assert error.code==expected,(error.code,value);return value

def verify_receipts(root):
 directory=root/'.hick-cache/behavior-verification';return set(directory.glob('*.json')) if directory.exists() else set()
def coverage_receipts(origin,root,source,implementation):
 # Real pytest-cov/JUnit collection, deliberately restricted to one selected case.
 record=api(origin,'/api/tests/coverage',{'action':'run','path':'.','language':'python','group':'selected'});deadline=time.monotonic()+20
 while time.monotonic()<deadline:
  value=api(origin,'/api/tests/coverage',{'action':'get','id':record['id']})
  if value['status'] not in ('running','collecting'):break
  time.sleep(.1)
 assert value['status']=='completed' and value['exit_code']==0,value
 assert any(test['status']=='passed' and 'test_selected' in test['id'] for test in value['tests']),value['tests']
 assert not (root/'sibling-ran.txt').exists()
 identity={'collector':'owned-manual-import','version':'1','configuration':'Explicit fixture import; not automated execution','scope':'.','source_hashes':{'source.md':hashlib.sha256(source.encode()).hexdigest(),'a.py':hashlib.sha256(implementation.encode()).hexdigest()}}
 raw='SF:a.py\nDA:1,1\nDA:2,1\nend_of_record\n'
 current=api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':identity})
 stale_identity={**identity,'source_hashes':{'a.py':'0'*64,'source.md':'0'*64}}
 api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':stale_identity})
 legacy=api(origin,'/api/tests/coverage',{'action':'import','path':'.','format':'lcov','raw':raw,'identity':identity})
 # Explicit retained legacy-input fixture, still labelled import: no fabricated automated record.
 path=root/'.hick-cache/coverage'/f"{legacy['id']}.json";data=json.loads(path.read_text());data.pop('workspace_id');path.write_text(json.dumps(data))
 return current['id']
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--serve-only',action='store_true');options=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-behavior-reading-') as temporary:
  folder=Path(temporary);details=prepare(folder/'workspace');root=Path(details['workspace']);source=details['source_text'];implementation=details['implementation_text'];sibling=Path(details['sibling']);manual=Path(details['manual']);processes=[];driver=None
  environment={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   if options.serve_only:
    stopped=threading.Event()
    for event in (signal.SIGTERM,signal.SIGINT):signal.signal(event,lambda *_:stopped.set())
    print(json.dumps({'origin':backend,**details}),flush=True);stopped.wait();return
   opened=api(backend,'/api/open-file',{'path':'source.md'});loaded=api(backend,'/api/docs/'+urllib.parse.quote(opened['doc_id'],safe=''))
   context={'version':1,'entry':'project.md','focus':'a.py','sources':{'source.md':source},'missing_sources':[],'unaddressed_sources':{},'saved_guards':{'source.md':loaded['save_guard']}};reading=api(backend,'/api/behavior/reading',context)
   # Revoke a captured profile before any editor is opened. Restored bytes do not
   # resurrect its old physical guard; the later editor receives its initial baseline explicitly.
   profile_scope={key:reading[key] for key in ['engine_workspace_id','workspace_id','reading_fingerprint']};profile_scope['context']=context
   profiles=api(backend,'/api/behavior/profiles',profile_scope);stale_choice=next(profile for profile in profiles['profiles'] if profile['id']=='stale-manual-check');stale_check=next(check for check in reading['graph']['checks'] if check['label']=='Explicit unsupported check')
   (root/'source.md').write_text(source.replace('Stale manual check','Changed stale profile'))
   api(backend,'/api/behavior/verify',{'version':1,**profile_scope,'check_id':stale_check['id'],'request_id':uuid.uuid4().hex,'profile':{'id':stale_choice['id'],'fingerprint':profiles['fingerprint']}},expected=409)
   (root/'source.md').write_text(source);assert not Path(details['stale_manual']).exists()
   loaded=api(backend,'/api/docs/'+urllib.parse.quote(opened['doc_id'],safe=''));context={**context,'saved_guards':{'source.md':loaded['save_guard']}};reading=api(backend,'/api/behavior/reading',context)
   assert reading['verification_available'] is True,reading['graph']['notices']
   # Readability survives missing loaded authority and anonymous construction ownership.
   for changed_context in [{**context,'saved_guards':{}},{**context,'unaddressed_sources':{'opaque-retained':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="other.py">value=99</h:file></h:doc>'}}]:
    unavailable=api(backend,'/api/behavior/reading',changed_context);assert unavailable['verification_available'] is False and any(node['label']=='Answer behavior' for node in unavailable['graph']['nodes'].values()),unavailable
    selected_check=next(check for check in unavailable['graph']['checks'] if check['available']);api(backend,'/api/behavior/verify',{'version':1,'engine_workspace_id':unavailable['engine_workspace_id'],'workspace_id':unavailable['workspace_id'],'reading_fingerprint':unavailable['reading_fingerprint'],'check_id':selected_check['id'],'request_id':uuid.uuid4().hex,'context':changed_context},expected=409)
   unrelated=root/'retained-notes.txt';unrelated.write_text('ordinary retained notes');note=api(backend,'/api/file?path=retained-notes.txt');guarded_context={**context,'sources':{**context['sources'],'retained-notes.txt':'ordinary retained notes'},'saved_guards':{**context['saved_guards'],'retained-notes.txt':note['save_guard']}}
   assert api(backend,'/api/behavior/reading',guarded_context)['verification_available'] is True
   dirty_named={**guarded_context,'sources':{**guarded_context['sources'],'retained-notes.txt':'unsaved retained notes'}};assert api(backend,'/api/behavior/reading',dirty_named)['verification_available'] is False
   replacement=root/'replacement-notes.txt';replacement.write_text(unrelated.read_text());replacement.replace(unrelated)
   stale_guard=api(backend,'/api/behavior/reading',guarded_context);assert stale_guard['verification_available'] is False and stale_guard['graph']['checks'],stale_guard
   output=next(output for output in reading['construction']['outputs'] if output['path']=='a.py');assert output['after']['hash']==hashlib.sha256(implementation.encode()).hexdigest(),output
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**environment,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='source.md');driver.act('remember-editor',path='source.md');size=driver.act('editor-size',path='source.md');before=driver.act('construction-source',path='source.md',selection=True)
   driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');text=driver.act('behavior-ready');assert 'Answer behavior' in text and 'needs-explanation' in text,text[-3000:]
   assert not any('test_unrelated' in option for option in driver.act('behavior-check-options'))
   page=driver.act('behavior-source',kind='explanation',label='answer');assert details['later_marker'] in page['text'] and 'observed-fragment-context' in page['state'],page
   assert driver.act('construction-source',path='source.md',selection=True)==before
   driver.act('construction-close');driver.act('open',path='retained-notes.txt');driver.act('remember-editor',path='retained-notes.txt');driver.act('editor-position',path='retained-notes.txt',anchor=0);driver.act('type-source-selection',path='retained-notes.txt',text='unsaved ');dirty_notes=driver.act('construction-source',path='retained-notes.txt',selection=True)
   driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');unavailable=driver.act('behavior-ready');assert 'unsaved' in unavailable.lower() and not driver.act('behavior-run-enabled'),unavailable[-3000:];assert driver.act('construction-source',path='retained-notes.txt',selection=True)==dirty_notes;assert driver.act('editor-is-retained',path='retained-notes.txt');assert unrelated.read_text()=='ordinary retained notes'
   driver.act('construction-close');driver.act('construction-history',redo=False);driver.act('open',path='source.md');assert driver.act('editor-is-retained',path='source.md');assert driver.act('construction-source',path='source.md',selection=True)==before;driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready')
   surface=driver.act('behavior-surface');assert surface['position']=='fixed' and surface['width']<=450 and surface['bottom']<=surface['viewport'],surface
   driver.act('behavior-check',label='test_selected');driver.act('behavior-run');text=driver.act('behavior-finished');assert 'process-finished' in text and 'process-only' in text,text[-3000:];assert not sibling.exists()
   driver.act('behavior-check',label='Explicit unsupported check');driver.act('behavior-profiles',label='Explicit manual check');driver.act('behavior-profile-run');text=driver.act('behavior-finished');assert 'user-declared profile' in text,text[-3000:];assert manual.read_text()=='ran'
   # Same request ID can inspect only the exact typed intent, including selected profile.
   body=driver.act('behavior-last-request');api(backend,'/api/behavior/verify',body);changed={**body,'profile':{**body['profile'],'id':'stale-manual-check'}};api(backend,'/api/behavior/verify',changed,expected=409)
   selected=next(check for check in reading['graph']['checks'] if check['label']=='test_selected' and check['available']);proof={'version':1,'engine_workspace_id':reading['engine_workspace_id'],'workspace_id':reading['workspace_id'],'reading_fingerprint':reading['reading_fingerprint'],'check_id':selected['id'],'context':context}
   existing=verify_receipts(root);api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex,'workspace_id':'f'*64},expected=409);assert verify_receipts(root)==existing
   (root/'a.py').write_text(implementation.replace('41','99'));api(backend,'/api/behavior/verify',{**proof,'request_id':uuid.uuid4().hex},expected=409);assert verify_receipts(root)==existing;(root/'a.py').write_text(implementation)
   assert verify_receipts(root)==existing
   coverage_receipts(backend,root,source,implementation)
   captured=api(backend,'/api/behavior/reading',context);rows=captured['evidence']['records'];assert any(row['evidence_kind']=='automated-collector' and row['freshness']=='current-captured-bytes' for row in rows),rows;assert any(row['freshness']=='stale' for row in rows),rows;assert any(row['origin']=='unknown' for row in rows),rows;assert any(row['evidence_kind']=='manual-or-imported' for row in rows),rows
   driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready');evidence=driver.act('behavior-evidence');assert 'automated-collector' in evidence and 'manual-or-imported' in evidence and 'origin-unknown' in evidence and 'stale' in evidence,evidence[-4000:]
   # Capability reading preserves a live unsaved draft and cannot run a superseded review.
   driver.act('source-tests-edit',path='source.md',text='\n# retained unsaved review draft\n');dirty=driver.act('construction-source',path='source.md',selection=True);count=driver.act('behavior-request-count');driver.act('behavior-check',label='test_selected');assert not driver.act('behavior-run-enabled');assert driver.act('behavior-request-count')==count
   driver.act('behavior-source',kind='explanation',label='answer');assert driver.act('construction-source',path='source.md',selection=True)==dirty
   driver.act('construction-close');driver.act('construction-open');assert 'process-finished' in driver.act('behavior-ready',retained=True);driver.act('construction-close')
   assert driver.act('editor-is-retained',path='source.md');assert driver.act('editor-size',path='source.md')==size
   driver.act('construction-history',redo=False);assert driver.act('construction-source',path='source.md')==source;driver.act('construction-history',redo=True);assert driver.act('construction-source',path='source.md')==dirty['source']
   assert (root/'source.md').read_text()==source and (root/'a.py').read_text()==implementation;assert not sibling.exists();assert not driver.act('debug-diagnostics')['errors']
   # The process belongs to the admitted engine, independently of its source directory.
   driver.act('construction-history',redo=False);driver.act('construction-open');driver.act('construction-read',entry='project.md',focus='a.py');driver.act('behavior-ready');driver.act('behavior-check',label='Explicit unsupported check');driver.act('behavior-profiles',label='Held manual check');driver.act('behavior-profile-run')
   deadline=time.monotonic()+10
   while time.monotonic()<deadline and not Path(details['held_ready']).exists():time.sleep(.05)
   assert Path(details['held_ready']).read_text()=='ready'
   held=driver.act('behavior-last-request');owned=api(backend,'/api/behavior/verification?id='+held['request_id']);assert owned['terminal']['id'];known_terminal=owned['terminal']['id']
   # Identical bytes cannot rebind an old reading to a replacement directory.
   retired=root.with_name('retained-original');root.rename(retired);shutil.copytree(retired,root,symlinks=True)
   cache_before={path.name:path.read_bytes() for path in verify_receipts(root)}
   observation=api(backend,'/api/behavior/verification?id='+held['request_id']);assert observation['workspace_current'] is False and observation['terminal']['id']==known_terminal,observation
   replay=api(backend,'/api/behavior/verify',held);assert replay['terminal']['id']==known_terminal and replay['workspace_current'] is False
   Path(details['held_release']).write_text('release');text=driver.act('behavior-finished');assert 'process-finished' in text and 'Workspace physical identity changed' in text,text[-3000:]
   final=api(backend,'/api/behavior/verification?id='+held['request_id']);assert final['terminal']['id']==known_terminal and final['status']=='process-finished' and final['exit_code']==0,final
   assert {path.name:path.read_bytes() for path in verify_receipts(root)}==cache_before
   assert driver.act('editor-is-retained',path='source.md') and driver.act('editor-size',path='source.md')==size
   identity=uuid.uuid4().hex;api(backend,'/api/behavior/verify',{**proof,'request_id':identity},expected=409);assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   fixture.stop(engine)
   restarted=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env={**environment,'HICKORY_STATE_DIR':str(folder/'restarted-state')});processes.append(restarted);new_origin=fixture.origin(restarted)
   fresh=api(new_origin,'/api/behavior/reading',context);assert fresh['workspace_id']!=reading['workspace_id'];assert fresh['engine_workspace_id']!=reading['engine_workspace_id']
   api(new_origin,'/api/behavior/verify',{**proof,'request_id':identity},expected=409)
   api(new_origin,'/api/behavior/verify',{**proof,'engine_workspace_id':fresh['engine_workspace_id'],'request_id':identity},expected=409)
   assert not (root/'.hick-cache/behavior-verification'/f'{identity}.json').exists()
   print('PASS: later-fragment full captured reading; explicit contract/explanation/compiler/test edges; unrelated-test exclusion; selected pytest and exact user profile; full-intent replay/stale output/inventory/foreign refusal; genuine pytest-cov/JUnit plus manual/stale/legacy distinctions; retained process-only links, editor/selection/geometry/unsaved/native undo/redo, unchanged source bytes; identical-byte root replacement, independently owned post-effect process retention without foreign cache writes, and old-reading/new-engine refusal')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-behavior-readings-browser.py"><ui:paste select="#behavior-reading-browser-real-engine"/></ui:file>

The canonical owned browser witness also exercises a real explicitly selected long-running profile. After dispatch returns its exact terminal identity, the fixture replaces the workspace directory with identical bytes. Refresh and exact-request replay retain only that engine-owned process, observe its eventual process exit zero, leave the replacement's cache unchanged, and preserve the editor and geometry. A separately isolated engine refuses the older physical reading. This is process lifecycle evidence, not a collected test verdict or proof that execution inputs were frozen. The final shared suite passed 20 focused cases, including Attach-profile Run unavailability and unsaved unselected plain-source admission refusal. The seven-field context binds all retained authored named sources, typed anonymous drafts and original saved guards. The maintained real-engine/browser witness also edits a retained plain source while keeping its valid original guard: captured readings and its editor remain readable, saved verification becomes unavailable, and disk bytes stay unchanged. The proposal integration’s focused browser suite and TypeScript checks also passed. Physical input and accessibility acceptance remain unexecuted here.

## Page through the exact declared source rather than stopping at an abbreviation

The source reader is a separate observation model. It echoes the chosen node,
raw source path, fragment, full-text hash and graph identity; scalar page bounds
and the page's SHA-256 are checked after every asynchronous boundary. A failed
or stale page leaves already readable text intact. Nothing in this model can
replace a source editor or authorize an edit.

<ui:copy id="behavior-source-reading-browser-model">
import type {BehaviorReading,BehaviorNode,BehaviorProvider} from './BehaviorReading';
export interface BehaviorSourcePage {version:1;behavior_protocol:1;workspace_id:string;reading_fingerprint:string;node_id:string;source_hash:string;page_hash:string;source_path:string|null;fragment_id:string|null;origin:'constructed'|'captured-draft'|'saved-source'|'saved-baseline'|'retained-summary';state:'captured'|'historical-deleted-baseline';target_status:'whole-source'|'observed-fragment-context'|'missing-declared-fragment'|'ambiguous-declared-fragment';units:'unicode-scalars';offset:number;total:number;next:number|null;text:string;notice:string}
const hex=(value:unknown):value is string=>typeof value==='string'&&/^[0-9a-f]{64}$/.test(value);
const integer=(value:unknown):value is number=>Number.isSafeInteger(value)&&Number(value)>=0;
export async function behaviorPageHash(text:string):Promise<string>{return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');}
export class BehaviorSourceReading {
 page:Readonly<BehaviorSourcePage>|null=null;notice='';loading=false;private ticket=0;
 constructor(private call:BehaviorProvider,private changed:()=>void=()=>{}){}
 retire(){this.ticket++;this.loading=false;}
 close(){this.retire();this.page=null;this.notice='';}
 async read(reading:BehaviorReading,node:BehaviorNode,offset:number|undefined,current:()=>boolean){const ticket=++this.ticket;this.loading=true;this.notice='Reading exact captured source…';this.changed();
  try{if(!current()||reading.graph.nodes[node.id]!==node||!hex(node.hash)||offset!==undefined&&!integer(offset))throw Error('Captured node or page is unavailable. Existing text was retained.');const raw=await this.call('/api/behavior/source',{version:1,workspace_id:reading.workspace_id,reading_fingerprint:reading.reading_fingerprint,node_id:node.id,source_hash:node.hash,context:reading.context,...offset===undefined?{}:{offset}}) as BehaviorSourcePage;
   if(ticket!==this.ticket||!current())return;
   if(!raw||raw.version!==1||raw.behavior_protocol!==1||raw.workspace_id!==reading.workspace_id||raw.reading_fingerprint!==reading.reading_fingerprint||raw.node_id!==node.id||raw.source_hash!==node.hash||raw.source_path!==(node.source_path??null)||raw.fragment_id!==(node.fragment_id??null)||raw.units!=='unicode-scalars'||!hex(raw.page_hash)||!integer(raw.offset)||!integer(raw.total)||raw.offset>raw.total||offset!==undefined&&raw.offset!==offset||typeof raw.text!=='string'||typeof raw.notice!=='string'||!['constructed','captured-draft','saved-source','saved-baseline','retained-summary'].includes(raw.origin)||!['captured','historical-deleted-baseline'].includes(raw.state)||!['whole-source','observed-fragment-context','missing-declared-fragment','ambiguous-declared-fragment'].includes(raw.target_status))throw Error('Foreign or malformed captured source page. Existing text was retained.');
   if(raw.text.length>8000)throw Error('Captured source page exceeds its scalar budget. Existing text was retained.');const scalars=Array.from(raw.text);if(raw.offset<raw.total&&scalars.length===0||scalars.length>4000||scalars.some(character=>{const point=character.codePointAt(0)!;return point>=0xd800&&point<=0xdfff;})||raw.offset+scalars.length>raw.total||raw.next!==(raw.offset+scalars.length<raw.total?raw.offset+scalars.length:null)||raw.state==='historical-deleted-baseline'&&raw.origin!=='saved-baseline'||raw.origin==='retained-summary'&&raw.source_path!==null)throw Error('Captured source scalar bounds or origin changed. Existing text was retained.');
   const hash=await behaviorPageHash(raw.text);if(ticket!==this.ticket||!current())return;if(hash!==raw.page_hash)throw Error('Captured source page bytes changed. Existing text was retained.');this.page=Object.freeze({...raw});this.notice=raw.notice;
  }catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.loading=false;this.changed();}}
 }
}
</ui:copy>

## Exercise held replies, literal names, page budgets and historical states

These focused checks keep late source reads from replacing a newer reading and
keep missing/deleted/historical text distinct. The actual engine witness below
must additionally locate a later explanation from its observed parser span.

<ui:copy id="behavior-source-reading-browser-tests">
import {beforeAll,it,expect} from 'vitest';
import {BehaviorSourceReading,behaviorPageHash,type BehaviorSourcePage} from './BehaviorSourceReading';
import type {BehaviorReading,BehaviorNode} from './BehaviorReading';
beforeAll(async()=>{const module='node:'+'crypto';const {webcrypto}=await import(module);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const node:BehaviorNode={id:'named',kind:'explanation',path:'literal#name.md#later',source_path:'literal#name.md',fragment_id:'later',label:'later',status:'needs-explanation',hash:'a'.repeat(64),preview:'abbreviation'};
const reading:BehaviorReading={version:1,behavior_protocol:1,namespace:'/workspace',engine_workspace_id:'e'.repeat(32),workspace_id:'b'.repeat(64),reading_fingerprint:'c'.repeat(64),context:{version:1,entry:'project.md',focus:'a.py',sources:{},missing_sources:[],unaddressed_sources:{},saved_guards:{}},graph:{nodes:{named:node},edges:[],checks:[],notices:[],truncated:false},impact:null,evidence:{status:'missing',records:[]},evidence_limit:'Unknown'};
async function page(text='😀 later explanation'):Promise<BehaviorSourcePage>{return {version:1,behavior_protocol:1,workspace_id:reading.workspace_id,reading_fingerprint:reading.reading_fingerprint,node_id:node.id,source_hash:node.hash!,page_hash:await behaviorPageHash(text),source_path:node.source_path!,fragment_id:'later',origin:'saved-source',state:'captured',target_status:'observed-fragment-context',units:'unicode-scalars',offset:5000,total:5000+Array.from(text).length,next:null,text,notice:'Captured only'};}
it('reads exact literal source/fragment and Unicode scalar pages without an editor application port',async()=>{let body:unknown;const model=new BehaviorSourceReading(async(_path,input)=>{body=input;return page();});await model.read(reading,node,undefined,()=>true);expect(model.page?.text).toContain('😀 later');expect(model.page?.offset).toBe(5000);expect((body as {source_hash:string}).source_hash).toBe(node.hash);expect(model.page?.source_path).toBe('literal#name.md');});
it('rejects foreign identity, changed hash, malformed scalar ranges and forged page bytes while preserving readable text',async()=>{let response=await page();const model=new BehaviorSourceReading(async()=>response);await model.read(reading,node,undefined,()=>true);const captured=model.page;for(const field of ['workspace_id','reading_fingerprint','node_id','source_hash','source_path','fragment_id','page_hash','units','offset','next','origin'] as const){response={...await page(),[field]:field==='offset'?-1:field==='next'?1:'foreign'} as BehaviorSourcePage;await model.read(reading,node,undefined,()=>true);expect(model.page).toBe(captured);}response=await page('x'.repeat(4001));await model.read(reading,node,undefined,()=>true);expect(model.page).toBe(captured);});
it('retires held older, changed-reading and closed source pages',async()=>{const waits:((value:unknown)=>void)[]=[];let current=true;const model=new BehaviorSourceReading(()=>new Promise(resolve=>waits.push(resolve)));const old=model.read(reading,node,undefined,()=>current),latest=model.read(reading,node,undefined,()=>current);waits[1](await page('latest'));await latest;waits[0](await page('old'));await old;expect(model.page?.text).toBe('latest');const obsolete=model.read(reading,node,undefined,()=>current);current=false;waits[2](await page('changed'));await obsolete;expect(model.page?.text).toBe('latest');current=true;const closed=model.read(reading,node,undefined,()=>current);model.close();waits[3](await page('closed'));await closed;expect(model.page).toBeNull();});
it('refuses zero-progress and oversized source pages before retaining them',async()=>{for(const mode of ['empty','oversized']){const value=await page(mode==='empty'?'':'x'.repeat(8001));if(mode==='empty'){value.total=value.offset+1;value.next=value.offset;}const model=new BehaviorSourceReading(async()=>value);await model.read(reading,node,undefined,()=>true);expect(model.page).toBeNull();}});
it('retains missing-fragment and historical/deleted-baseline status instead of inventing a current target',async()=>{const model=new BehaviorSourceReading(async()=>({...await page(),origin:'saved-baseline',state:'historical-deleted-baseline',target_status:'missing-declared-fragment'}));await model.read(reading,node,undefined,()=>true);expect(model.page?.state).toBe('historical-deleted-baseline');expect(model.page?.target_status).toBe('missing-declared-fragment');});
</ui:copy>
<ui:file path="apps/web/src/simple/BehaviorSourceReading.ts"><ui:paste select="#behavior-source-reading-browser-model"/></ui:file>
<ui:file path="apps/web/src/simple/BehaviorSourceReading.test.ts"><ui:paste select="#behavior-source-reading-browser-tests"/></ui:file>
## Connected reading execution evidence

On 2026-11-16 the canonical browser suite passed 31 focused cases across the
behavior model, full-source paging, construction model and API decoder;
TypeScript validation passed. The maintained real-engine owned-fixture witness
passed complete later-fragment reading, explicit metadata/compiler/test edges,
unrelated-test exclusion, selected pytest and user-selected exact profile
execution, full-intent replay and stale output/inventory/foreign refusal. It
also collected genuine pytest-cov/JUnit evidence and kept current, stale,
manual/imported and legacy origin-unknown records distinct. Retained process-only
links, editor identity, selection, geometry, unsaved text, native undo/redo and
unchanged source bytes were checked. This establishes neither physical input nor
accessibility acceptance. A process result is never a fabricated collected test
verdict or proof that captured mapping bytes were frozen during execution.

</ui:doc>
