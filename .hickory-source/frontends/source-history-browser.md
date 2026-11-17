<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Restore retained source history into the existing browser editor

For maintainers adding history without granting historical text filesystem authority.

## Bind a readable history to its exact saved and draft contexts

History entries distinguish prepared intention, confirmed publication, failure and
unknown outcome. Verify retained bytes and all restore echoes, then recheck the
captured editor after every await. A stale selection, source, workspace or review
never becomes permission to overwrite a draft. An observed Missing source is
distinct from an empty file: its exact absence receipt may restore confirmed
historical File text into an unmaterialized draft. It never creates the path or
acquires a new save guard; a separate guarded Save As publishes that text.

<ui:copy id="source-history-browser-model">
export type SavedHistorySource={kind:'file';text:string;sha256:string;readonly:boolean}|{kind:'missing'};
export interface SourceHistoryEntry {id:string;at:number;outcome:'prepared'|'published'|'failed'|'unknown';fingerprint:string;before:SavedHistorySource;after:SavedHistorySource;available:boolean;reason?:string}
export interface SourceHistoryReading {version:number;history_protocol:number;workspace:string;namespace:string;path:string;current:SavedHistorySource;current_identity:string;restore_target:"current-source-draft"|"missing-source-draft";entries:SourceHistoryEntry[];notices:string[]}
export interface SourceHistoryDraft {version:number;history_protocol:number;workspace:string;namespace:string;path:string;id:string;fingerprint:string;current_identity:string;current_hash:string;draft_hash:string;side:'before'|'after';source:string;restore_target:'current-source-draft'|'missing-source-draft';source_effects:false;consequences:string;notice:string}
export interface SourceHistoryContext {namespace:string;path:string;source:string;readCurrent?:()=>boolean;current:()=>boolean;apply:(source:string,missing:boolean)=>void}
export type SourceHistoryTransport=<T>(method:'GET'|'POST',path:string,body?:unknown)=>Promise<T>;
export async function historySourceHash(source:string):Promise<string>{return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');}
function hex(value:unknown,count:number):value is string{return typeof value==='string'&&new RegExp('^[0-9a-f]{'+count+'}$').test(value);}
export async function validateSourceHistory(value:SourceHistoryReading,path:string,namespace:string):Promise<void>{
 if(value.version!==1||value.history_protocol!==1||value.path!==path||!namespace||value.namespace!==namespace||!value.workspace||!value.current_identity||!Array.isArray(value.entries)||value.entries.length>1000||!Array.isArray(value.notices)||!['file','missing'].includes(value.current.kind)||value.restore_target!==(value.current.kind==='missing'?'missing-source-draft':'current-source-draft'))throw Error('Unsupported or changed local history identity.');
 const ids=new Set<string>();
 for(const entry of value.entries){if(!hex(entry.id,32)||ids.has(entry.id)||!hex(entry.fingerprint,64)||!Number.isSafeInteger(entry.at)||entry.at<0||entry.at>8640000000000000||!['prepared','published','failed','unknown'].includes(entry.outcome)||typeof entry.available!=='boolean'||entry.available&&entry.outcome!=='published')throw Error('Local history contains ambiguous outcome evidence.');ids.add(entry.id);}
 for(const saved of [value.current,...value.entries.flatMap(entry=>[entry.before,entry.after])])if(saved.kind==='file'){if(typeof saved.text!=='string'||!hex(saved.sha256,64)||await historySourceHash(saved.text)!==saved.sha256)throw Error('Local history source fingerprint changed.');}else if(saved.kind!=='missing')throw Error('Unsupported history source state.');
}
export class SourceHistoryModel {
 reading:SourceHistoryReading|null=null;selected:SourceHistoryEntry|null=null;side:'before'|'after'='before';notice='';busy=false;private ticket=0;private context:SourceHistoryContext|null=null;
 constructor(private load:SourceHistoryTransport,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.context=null;this.selected=null;this.busy=false;}
 async open(context:SourceHistoryContext){this.close();this.context=context;this.reading=null;const ticket=++this.ticket;this.busy=true;this.notice='Reading retained local source history…';this.changed();
  try{const value=structuredClone(await this.load<SourceHistoryReading>('GET','/api/source-history?path='+encodeURIComponent(context.path)));await validateSourceHistory(value,context.path,context.namespace);if(ticket!==this.ticket)return;if(!(context.readCurrent?.()??context.current()))throw Error('Source editor changed while reading history. Open it again.');this.reading=value;this.notice='Compare retained text before restoring to this draft. Saved files are unchanged.';}catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
 choose(id:string,side:'before'|'after'){if(this.busy)return;const entry=this.reading?.entries.find(entry=>entry.id===id);if(!entry)return;this.selected=entry;this.side=side;this.notice=entry.reason??'Preview only; current unsaved text is retained.';this.changed();}
 get preview(){const saved=this.selected?.[this.side];return saved?.kind==='file'?saved.text:null;}
 current(){return this.context?.current()===true&&!!this.reading&&(this.reading.current.kind==='missing'||this.reading.current.readonly===false);}
 async restore():Promise<boolean>{const context=this.context,reading=this.reading,entry=this.selected,side=this.side,source=this.preview;if(this.busy||!context?.current()||!reading||!entry?.available||source===null||(reading.current.kind==='file'&&reading.current.readonly!==false)){this.notice='This history or editor is stale or unavailable. Current draft is retained.';this.changed();return false;}
  const ticket=++this.ticket;this.busy=true;this.selected=null;this.changed();
  try{const draftHash=await historySourceHash(context.source);if(ticket!==this.ticket||!context.current())return false;const body={version:1,path:context.path,id:entry.id,fingerprint:entry.fingerprint,workspace:reading.workspace,namespace:context.namespace,current_identity:reading.current_identity,current_hash:reading.current.kind==='missing'?'missing':reading.current.sha256,draft_hash:draftHash,side};const value=await this.load<SourceHistoryDraft>('POST','/api/source-history/restore-draft',body);if(ticket!==this.ticket)return false;
   if(!context.current()||value.version!==1||value.history_protocol!==1||value.workspace!==body.workspace||value.namespace!==body.namespace||value.path!==body.path||value.id!==body.id||value.fingerprint!==body.fingerprint||value.current_identity!==body.current_identity||value.current_hash!==body.current_hash||value.draft_hash!==body.draft_hash||value.side!==side||value.source!==source||value.restore_target!==reading.restore_target||value.source_effects!==false||value.consequences!=='unknown')throw Error('History authority or editor changed. No draft was replaced.');context.apply(source,reading.current.kind==='missing');this.context=null;this.notice=value.notice;return true;
  }catch(error){if(ticket===this.ticket)this.notice=String(error);return false;}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
}
</ui:copy>

## Keep comparison floating and application coordinated

The dialog pages long text inside the existing floating surface. Capture the
actual attached editor, selection and draft revision before reading. Apply through
the same workspace change mechanism and retained undo history as refactoring;
no source save or generated publication happens here. Native physical input and
assistive technology acceptance are separate from controller and browser tests.

<ui:copy id="source-history-browser-view">
import {useEffect,useMemo,useReducer,useRef,useState,useSyncExternalStore} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import {editorForBuffer} from './Document';
import {EditorView} from '@codemirror/view';
import {WorkspaceChangeHistory,invertChangePlan,type ChangePlan} from './ChangePlan';
import {applyWorkspacePlan} from './WorkspaceChanges';
import {SourceHistoryModel} from './SourceHistory';
export function SourceHistoryView({buffer,buffers,history,alive,close}:{buffer:Buffer;buffers:Buffers;history:WorkspaceChangeHistory;alive:()=>boolean;close:()=>void}){
 useSyncExternalStore(buffer.subscribe,buffer.snapshot);
 const [,paint]=useReducer((value:number)=>value+1,0),model=useMemo(()=>new SourceHistoryModel((method,path,body)=>request(method,path,body),paint),[]),panel=useRef<HTMLElement>(null),[page,setPage]=useState(0);
 useEffect(()=>{const view=editorForBuffer(buffer)?.view,source=buffer.text,path=buffer.file.path,file=buffer.file,namespace=buffers.workspace,revision=buffer.sourceRevision,selection=view?.state.selection.toJSON(),stamp=history.stamp();let living=true;
 const readCurrent=()=>living&&alive()&&!!view&&view.dom.isConnected&&editorForBuffer(buffer)?.view===view&&view.state.doc.toString()===source&&buffer.text===source&&buffer.file===file&&buffer.file.path===path&&buffers.workspace===namespace&&buffer.namespace===namespace&&!!namespace&&buffers.values.get(path)===buffer&&buffer.sourceRevision===revision&&history.stamp()===stamp;
 const current=()=>readCurrent()&&!!view&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable)&&view.state.selection.ranges.length===1&&JSON.stringify(view.state.selection.toJSON())===JSON.stringify(selection)&&!buffer.readonly&&!buffer.transient&&!buffer.saving&&!buffer.resourceDeleted;
 void model.open({namespace,path,source,readCurrent,current,apply:(after,missing)=>{if(!current())throw Error('Source editor changed before draft restoration.');const plan:ChangePlan={title:'Restore local history to draft',patches:[{path,before:source,after,edits:[{from:0,to:source.length,text:after}]}]};applyWorkspacePlan(buffers,plan);history.record({kind:'draft',plan:invertChangePlan(plan)});if(missing){buffer.resourceDeleted=true;buffer.saveGuard=undefined;buffer.saveGuardReason='Restored missing-source draft: use guarded Save As to publish to an unused path.';buffer.publish();buffers.changed();}view!.focus();}});panel.current?.focus();return()=>{living=false;model.close();};},[buffer,buffers,history,model]);
 const selected=model.selected,text=model.preview;
 return createPortal(<section role="dialog" aria-label="Local source history" tabIndex={-1} ref={panel} className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Local source history</strong><button onClick={close}>Close</button></header><p role="status">{model.notice}</p>{model.reading?.notices.map((notice,index)=><p key={index}>{notice}</p>)}
 <label>Retained save <select aria-label="Retained save" value={selected?.id??''} disabled={model.busy} onChange={event=>{model.choose(event.target.value,model.side);setPage(0);}}><option value="">Choose a save</option>{model.reading?.entries.map(entry=><option key={entry.id} value={entry.id}>{entry.outcome} · {new Date(entry.at).toISOString()} · {entry.id}</option>)}</select></label>
 {selected&&<><label>Snapshot <select aria-label="Snapshot" value={model.side} onChange={event=>{model.choose(selected.id,event.target.value as 'before'|'after');setPage(0);}}><option value="before">Before save</option><option value="after">Intended saved after</option></select></label><p>{selected.outcome} · {selected.fingerprint}</p><strong>Retained text</strong><pre style={{whiteSpace:'pre-wrap'}}>{text?.slice(page*16000,(page+1)*16000)??'Source absent; absence cannot replace an editor draft.'}</pre><strong>Current unsaved draft</strong><pre style={{whiteSpace:'pre-wrap'}}>{buffer.text.slice(page*16000,(page+1)*16000)}</pre><p>Text page {page+1} · 16,000 characters per side</p><button disabled={!page} onClick={()=>setPage(page-1)}>Previous text</button><button disabled={Math.max(text?.length??0,buffer.text.length)<=(page+1)*16000} onClick={()=>setPage(page+1)}>Next text</button><button disabled={model.busy||!selected.available||text===null||!model.current()} onClick={()=>void model.restore()}>Restore selected text to draft</button></>}
 {!model.busy&&model.reading?.entries.length===0&&<p>No ordinary saves are retained for this source.</p>}
 </section>,document.body);
}
</ui:copy>

## Refuse stale context and forged historical authority

These focused model cases exercise exact echoed restoration, tampered hashes,
ambiguous outcomes, obsolete reads and stale editor context. They do not replace
actual floating-dialog, editor undo/redo or physical accessibility acceptance.

<ui:copy id="source-history-browser-tests">
import {beforeAll,expect,it} from 'vitest';
import {SourceHistoryModel,historySourceHash,validateSourceHistory,type SourceHistoryReading,type SourceHistoryDraft} from './SourceHistory';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
async function reading():Promise<SourceHistoryReading>{const file=async(text:string)=>({kind:'file' as const,text,sha256:await historySourceHash(text),readonly:false});return {version:1,history_protocol:1,workspace:'workspace',namespace:'/workspace',path:'literal %?#.py',current:await file('saved'),current_identity:'physical-stamp',restore_target:'current-source-draft',entries:[{id:'a'.repeat(32),at:1,outcome:'published',fingerprint:'b'.repeat(64),before:await file('before 😀'),after:await file('saved'),available:true}],notices:[]};}
it('previews without applying and restores only the exact echoed draft through its port',async()=>{const value=await reading();let applied='',requests=0;const model=new SourceHistoryModel(async<T>(method:string,_path:string,body?:unknown)=>{if(method==='GET')return value as T;requests++;return {...body as object,history_protocol:1,source:'before 😀',restore_target:'current-source-draft',source_effects:false,consequences:'unknown',notice:'Draft restored'} as T;});await model.open({namespace:'/workspace',path:value.path,source:'unsaved',current:()=>true,apply:source=>{applied=source;}});model.choose(value.entries[0].id,'before');expect(model.preview).toBe('before 😀');expect(applied).toBe('');expect(await model.restore()).toBe(true);expect(applied).toBe('before 😀');expect(requests).toBe(1);expect(await model.restore()).toBe(false);expect(requests).toBe(1);});
it('refuses ambiguous or tampered retained evidence',async()=>{for(const mode of ['protocol','namespace','duplicate','hash','outcome','availability']){const value=await reading();if(mode==='protocol')value.history_protocol=2;if(mode==='namespace')value.namespace='/foreign';if(mode==='duplicate')value.entries.push(value.entries[0]);if(mode==='hash'&&value.entries[0].before.kind==='file')value.entries[0].before.text='tampered';if(mode==='outcome'){value.entries[0].outcome='unknown';}if(mode==='availability'){(value.entries[0] as unknown as {available:unknown}).available='true';}await expect(validateSourceHistory(value,value.path,'/workspace')).rejects.toThrow();}});
it('retains the draft when source or restore receipt changes',async()=>{for(const mode of ['current','path','workspace','namespace','source','draft_hash','current_identity','side','consequences','restore_target','source_effects']){const value=await reading();let current=true,applies=0;const model=new SourceHistoryModel(async<T>(method:string,_path:string,body?:unknown)=>{if(method==='GET')return value as T;const result={...body as object,history_protocol:1,source:'before 😀',restore_target:'current-source-draft',source_effects:false,consequences:'unknown',notice:'draft'} as SourceHistoryDraft;if(mode==='current')current=false;else (result as unknown as Record<string,unknown>)[mode]='foreign';return result as T;});await model.open({namespace:'/workspace',path:value.path,source:'unsaved',current:()=>current,apply:()=>{applies++;}});model.choose(value.entries[0].id,'before');expect(await model.restore()).toBe(false);expect(applies).toBe(0);}});
it('retires delayed reads and restores after close without applying obsolete replies',async()=>{const value=await reading();let resolve!:(value:unknown)=>void,applies=0;const model=new SourceHistoryModel(<T>()=>new Promise<T>(done=>{resolve=done as (value:unknown)=>void;}));const pending=model.open({namespace:'/workspace',path:value.path,source:'draft',current:()=>true,apply:()=>{applies++;}});model.close();resolve(value);await pending;expect(model.reading).toBeNull();expect(applies).toBe(0);});
it('keeps failed and unknown outcomes readable while refusing restore authority',async()=>{const value=await reading();value.entries[0].outcome='unknown';value.entries[0].available=false;let requests=0;const model=new SourceHistoryModel(async<T>()=>{requests++;return value as T;});await model.open({namespace:'/workspace',path:value.path,source:'draft',current:()=>true,apply:()=>{throw Error('must not apply');}});model.choose(value.entries[0].id,'before');expect(model.preview).toBe('before 😀');expect(await model.restore()).toBe(false);expect(requests).toBe(1);});
it('keeps readable history after write capability refusal without granting draft restore',async()=>{const value=await reading();let requests=0;const model=new SourceHistoryModel(async<T>()=>{requests++;return value as T;});await model.open({namespace:'/workspace',path:value.path,source:'readonly source',readCurrent:()=>true,current:()=>false,apply:()=>{throw Error('must not apply');}});expect(model.reading).not.toBeNull();model.choose(value.entries[0].id,'before');expect(model.preview).toBe('before 😀');expect(await model.restore()).toBe(false);expect(requests).toBe(1);});
it('keeps readonly or unknown saved capability readable without offering restoration',async()=>{for(const flag of [true,undefined,'false']){const value=await reading();(value.current as unknown as {readonly:unknown}).readonly=flag;let calls=0;const model=new SourceHistoryModel(async<T>()=>{calls++;return value as T;});await model.open({namespace:'/workspace',path:value.path,source:'draft',current:()=>true,apply:()=>{throw Error('must not edit');}});model.choose(value.entries[0].id,'before');expect(model.preview).toBe('before 😀');expect(model.current()).toBe(false);expect(await model.restore()).toBe(false);expect(calls).toBe(1);}});

it('restores confirmed File text under exact Missing authority without treating absence as empty text',async()=>{const value=await reading();value.current={kind:'missing'};value.restore_target='missing-source-draft';let applied:unknown;const model=new SourceHistoryModel(async<T>(method:string,_path:string,body?:unknown)=>{if(method==='GET')return value as T;expect((body as {current_hash:string}).current_hash).toBe('missing');return {...body as object,history_protocol:1,source:'before 😀',restore_target:'missing-source-draft',source_effects:false,consequences:'unknown',notice:'Unmaterialized draft'} as T;});await model.open({namespace:'/workspace',path:value.path,source:'retained draft',current:()=>true,apply:(text,missing)=>{applied={text,missing};}});model.choose(value.entries[0].id,'before');expect(model.current()).toBe(true);expect(await model.restore()).toBe(true);expect(applied).toEqual({text:'before 😀',missing:true});});
</ui:copy>
<ui:file path="apps/web/src/simple/SourceHistory.test.ts"><ui:paste select="#source-history-browser-tests"/></ui:file>

## Advance saved state only after a strict identity receipt

The focused save cases check lost-reply identity reuse, absent guards, old-route
refusal, forged receipts, source-owner changes and edits continuing during a save.
History availability is independent from a confirmed write. An acknowledged
external exact-file save may lack history without inventing a retained record.

<ui:copy id="source-history-browser-save-tests">
import {afterEach,beforeAll,expect,it,vi} from 'vitest';
import {Buffer,Buffers} from './buffers';
import {historySourceHash} from './SourceHistory';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
afterEach(()=>vi.unstubAllGlobals());
function buffer(doc=false){const value=new Buffer({path:'literal %?#.py',name:'literal %?#.py',dir:false,...(doc?{doc_id:'doc-id'}:{})},'saved','old-hash',()=>{},false,'/workspace');value.saveGuard='loaded-physical-token';value.edit('submitted 😀');return value;}
async function reply(body:Record<string,unknown>,path:string){return {version:1,history_protocol:1,request_id:body.request_id,path,source_hash:await historySourceHash(String(body.source??body.content)),save_guard:'new-physical-token',hash:'new-hash',history:{available:true,outcome:'published'}};}
it('uses strict routes and advances only the exactly confirmed submitted baseline',async()=>{for(const doc of [false,true]){const value=buffer(doc),calls:{path:string;body:Record<string,unknown>}[]=[];vi.stubGlobal('fetch',vi.fn(async(path:string,init:RequestInit)=>{const body=JSON.parse(String(init.body));calls.push({path,body});value.edit('later unsaved');return new Response(JSON.stringify(await reply(body,value.file.path)));}));await value.save();expect(calls[0].path).toBe(doc?'/api/source-history/docs/doc-id':'/api/source-history/file');expect(calls[0].body.save_guard).toBe('loaded-physical-token');expect(value.base).toBe('submitted 😀');expect(value.text).toBe('later unsaved');expect(value.saveGuard).toBe('new-physical-token');expect(value.error).toBe('');}});
it('retains the exact consumed request after a lost reply without legacy fallback',async()=>{const value=buffer(),bodies:string[]=[],paths:string[]=[];let lost=true;vi.stubGlobal('fetch',vi.fn(async(path:string,init:RequestInit)=>{paths.push(path);bodies.push(String(init.body));if(lost){lost=false;throw Error('lost reply');}return new Response(JSON.stringify(await reply(JSON.parse(String(init.body)),value.file.path)));}));await value.save();expect(value.base).toBe('saved');expect(value.saveGuard).toBe('loaded-physical-token');await value.save();expect(bodies[0]).toBe(bodies[1]);expect(paths).toEqual(['/api/source-history/file','/api/source-history/file']);expect(value.base).toBe('submitted 😀');});
it('refuses absent baseline guards, unsupported routes and forged outcome echoes',async()=>{for(const mode of ['missing','404','version','history_protocol','path','request_id','source_hash','save_guard','history']){const value=buffer();if(mode==='missing')value.saveGuard=undefined;let calls=0;vi.stubGlobal('fetch',vi.fn(async(_path:string,init:RequestInit)=>{calls++;if(mode==='404')return new Response(JSON.stringify({error:'unavailable'}),{status:404});const result=await reply(JSON.parse(String(init.body)),value.file.path);(result as Record<string,unknown>)[mode]=mode==='save_guard'?'':mode==='history'?{available:true,outcome:'unknown'}:'foreign';return new Response(JSON.stringify(result));}));await value.save();expect(value.base).toBe('saved');expect(value.text).toBe('submitted 😀');expect(value.saveGuard).toBe(mode==='missing'?undefined:'loaded-physical-token');expect(value.error).not.toBe('');expect(calls).toBe(mode==='missing'?0:1);}});
it('refuses adopting a receipt after source identity or baseline changed',async()=>{for(const mode of ['file','namespace','guard','base','deleted']){const value=buffer();vi.stubGlobal('fetch',vi.fn(async(_path:string,init:RequestInit)=>{const result=await reply(JSON.parse(String(init.body)),value.file.path);if(mode==='file')value.file={...value.file};if(mode==='namespace')value.namespace='/replacement';if(mode==='guard')value.saveGuard='replacement';if(mode==='base')value.base='foreign baseline';if(mode==='deleted')value.resourceDeleted=true;return new Response(JSON.stringify(result));}));await value.save();expect(value.base).toBe(mode==='base'?'foreign baseline':'saved');expect(value.error).toContain('not confirmed');}});
it('keeps history availability distinct from a confirmed source save',async()=>{const value=buffer();vi.stubGlobal('fetch',vi.fn(async(_path:string,init:RequestInit)=>{const result=await reply(JSON.parse(String(init.body)),value.file.path);result.history={available:false,outcome:'published'};return new Response(JSON.stringify(result));}));await value.save();expect(value.base).toBe('submitted 😀');expect(value.error).toContain('Saved; local history is unavailable');});
it('retains source guards only from the original saved read',async()=>{vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({path:'a.py',language:'python',content:'read source',hash:'hash',save_guard:'read-token'}))));const values=new Buffers(()=>{});values.workspace='/workspace';const value=await values.open({path:'a.py',name:'a.py',dir:false});expect(value.saveGuard).toBe('read-token');expect(value.base).toBe('read source');});
</ui:copy>
<ui:file path="apps/web/src/simple/SourceHistorySave.test.ts"><ui:paste select="#source-history-browser-save-tests"/></ui:file>

<ui:file path="apps/web/src/simple/SourceHistory.ts"><ui:paste select="#source-history-browser-model"/></ui:file>
<ui:file path="apps/web/src/simple/SourceHistoryView.tsx"><ui:paste select="#source-history-browser-view"/></ui:file>
## Browser acceptance and limits

The maintained owned-workspace browser witness exercises real strict Save,
full before/after history previews, unsaved draft restoration, both native and
workspace undo/redo, stale-selection refusal and unchanged editor geometry.
Seven history model cases and six strict Save cases passed on 2026-11-16.
Read-only or unknown saved capability stays readable and never enables restore.
Physical input and accessibility acceptance remain unexecuted here.

</ui:doc>
