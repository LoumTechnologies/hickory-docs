<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Recover browser editor sessions without publishing source

For maintainers following automatic private checkpoints into an explicit session recovery.

## Retain one exact checkpoint across an uncertain reply

A checkpoint is private editor state, never a source save. Bind it to the current engine workspace and window slot. Keep the complete request after a lost reply and retry only that request before recording a newer snapshot. A later capture must not silently change an already consumed request. Source editing remains available while storage is slow or unavailable.

<ui:copy id="editor-recovery-browser-model">
import {historySourceHash} from './SourceHistory';
import {parseProtocolJson} from '../api/client';
export type RecoveryAddress={kind:'source';path:string}|{kind:'unnamed';name:string}|{kind:'generated';path:string;owner:string}|{kind:'detached';name:string;original_path:string|null};
export interface EditorRecoveryBuffer {id:string;address:RecoveryAddress;text:string;baseline:{text:string;save_guard:string|null}|null;view:{selection:{anchor:number;head:number};scroll_x:number;scroll_y:number}}
export interface EditorRecoverySnapshot {host:'browser'|'macos'|'ios'|'android';buffers:EditorRecoveryBuffer[];layout:unknown}
export interface EditorRecoverySession {client_id:string;window_slot:string;workspace_id:string;namespace:string;current_workspace:boolean;revision:number;request_id:string;snapshot_hash:string;host:string;buffers:number;source_effects:false}
export interface EditorRecoveryCatalogue {version:number;recovery_protocol:number;engine_workspace_id:string;workspace_id:string;namespace:string;window_slot:string;current_window_slot:string;window_slots:string[];sessions:EditorRecoverySession[];notices:string[];next:string|null;source_effects:false}
export interface EditorRecoveryEnvelope {version:number;recovery_protocol:number;engine_workspace_id:string;client_id:string;window_slot:string;workspace_id:string;namespace:string;revision:number;request_id:string;snapshot_hash:string;material:string;source_effects:false;source_saved:false;current_workspace:boolean;authority:{buffer_id:string;baseline_current:boolean;write_permission:false;reason:string|null}[]}
type Transport=<T>(method:'GET'|'POST',path:string,body?:unknown)=>Promise<T>;
const hex=(v:unknown,n:number):v is string=>typeof v==='string'&&new RegExp('^[a-f0-9]{'+n+'}$').test(v);
const slot=(v:unknown):v is string=>typeof v==='string'&&/^(0|[1-9][0-9]{0,19})$/.test(v)&&BigInt(v)<=18446744073709551615n;
const scalar=(text:string,n:unknown):n is number=>typeof n==='number'&&Number.isSafeInteger(n)&&n>=0&&n<=text.length&&!(n>0&&n<text.length&&/[\uD800-\uDBFF]/.test(text[n-1])&&/[\uDC00-\uDFFF]/.test(text[n]));
function admit(ok:unknown,message='Unsupported or changed editor recovery receipt.'):asserts ok {if(!ok)throw Error(message);}
export function validateEditorRecoverySnapshot(raw:unknown):asserts raw is EditorRecoverySnapshot {
 admit(!!raw&&typeof raw==='object');const value=raw as EditorRecoverySnapshot;
 admit(['browser','macos','ios','android'].includes(value.host)&&Array.isArray(value.buffers)&&value.buffers.length<=256);
 const ids=new Set<string>();let bytes=0;
 for(const b of value.buffers){admit(hex(b.id,32)&&!ids.has(b.id)&&typeof b.text==='string');ids.add(b.id);const a=b.address;admit(!!a&&['source','unnamed','generated','detached'].includes(a.kind));
  const label=a.kind==='source'||a.kind==='generated'?a.path:a.name;admit(typeof label==='string'&&label.length>0&&label.length<=16384&&!label.includes('\0'));
  if(a.kind==='generated')admit(typeof a.owner==='string');if(a.kind==='detached')admit(a.original_path===null||typeof a.original_path==='string');
  admit(b.baseline===null||typeof b.baseline?.text==='string'&&(b.baseline.save_guard===null||typeof b.baseline.save_guard==='string'&&b.baseline.save_guard.length<=16384));
  admit(!!b.view&&scalar(b.text,b.view.selection?.anchor)&&scalar(b.text,b.view.selection?.head)&&[b.view.scroll_x,b.view.scroll_y].every(n=>Number.isFinite(n)&&n>=0&&n<=1e9));
  for(const ch of b.text)admit(ch.length!==1||ch.charCodeAt(0)<0xD800||ch.charCodeAt(0)>0xDFFF,'Recovery contains invalid Unicode source.');
  bytes+=new TextEncoder().encode(b.text).length+new TextEncoder().encode(b.baseline?.text??'').length;admit(bytes<=32*1024*1024);
 }admit(new TextEncoder().encode(JSON.stringify(value.layout)).length<=512*1024);
}
export class EditorRecoveryModel {
 readonly client=crypto.randomUUID().replaceAll('-','');revision=0;notice='';busy=false;catalogue:EditorRecoveryCatalogue|null=null;reading:{envelope:EditorRecoveryEnvelope;snapshot:EditorRecoverySnapshot;current:()=>boolean}|null=null;
 private scope='';private pending:{body:unknown;serialized:string;workspace:string;namespace:string;physical:string;slot:string;request:string;base:number}|null=null;private last='';private ticket=0;private writing=false;
 constructor(private send:Transport,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.busy=false;this.reading=null;}
 async list(workspace:string,namespace:string,windowSlot?:string){const query=windowSlot===undefined?'':'?window_slot='+encodeURIComponent(windowSlot);const value=await this.send<EditorRecoveryCatalogue>('GET','/api/workspace/recovery'+query);
  admit(value.version===1&&value.recovery_protocol===1&&value.engine_workspace_id===workspace&&value.namespace===namespace&&hex(value.workspace_id,64)&&slot(value.window_slot)&&slot(value.current_window_slot)&&value.source_effects===false&&Array.isArray(value.sessions)&&value.sessions.length<=32&&Array.isArray(value.window_slots)&&value.window_slots.every(slot)&&Array.isArray(value.notices));
  for(const row of value.sessions)admit(hex(row.client_id,32)&&hex(row.snapshot_hash,64)&&hex(row.request_id,32)&&row.window_slot===value.window_slot&&Number.isSafeInteger(row.revision)&&row.revision>0&&row.source_effects===false&&hex(row.workspace_id,64)&&typeof row.namespace==='string'&&typeof row.current_workspace==='boolean'&&['browser','macos','ios','android'].includes(row.host)&&Number.isInteger(row.buffers)&&row.buffers>=0&&row.buffers<=256);
  return value;
 }
 async checkpoint(workspace:string,namespace:string,snapshot:EditorRecoverySnapshot,current:()=>boolean){if(this.writing||!workspace||!namespace)return;this.writing=true;
  try {const captured=structuredClone(snapshot);validateEditorRecoverySnapshot(captured);const serialized=JSON.stringify(captured);if(this.scope&&this.scope!==workspace+'\0'+namespace){this.notice='The original checkpoint belongs to another connection. Its retained record remains available; this writer was retired.';return;}
   if(!this.scope)this.scope=workspace+'\0'+namespace;
   if(!this.pending&&serialized===this.last)return;
   if(!this.pending){const meta=await this.list(workspace,namespace);if(!current())return;this.pending={workspace,namespace,physical:meta.workspace_id,slot:meta.current_window_slot,request:crypto.randomUUID().replaceAll('-',''),base:this.revision,serialized,body:null};this.pending.body={version:1,workspace,window_slot:this.pending.slot,client_id:this.client,request_id:this.pending.request,base_revision:this.revision,snapshot:captured};}
   const held=this.pending;if(held.workspace!==workspace||held.namespace!==namespace||!current())return;
   const result=await this.send<{version:number;recovery_protocol:number;client_id:string;window_slot:string;namespace:string;workspace_id:string;request_id:string;revision:number;snapshot_hash:string;source_effects:boolean;source_saved:boolean}>('POST','/api/workspace/recovery/checkpoint',held.body);
   admit(result.version===1&&result.recovery_protocol===1&&result.client_id===this.client&&result.window_slot===held.slot&&result.namespace===held.namespace&&result.workspace_id===held.physical&&result.request_id===held.request&&result.revision===held.base+1&&hex(result.snapshot_hash,64)&&result.source_effects===false&&result.source_saved===false);
   this.revision=result.revision;this.last=held.serialized;this.pending=null;this.notice='Private editor checkpoint retained · source files unchanged.';
  }catch(e){this.notice=String(e)+' Readable drafts are retained; no source save was claimed.';}finally{this.writing=false;this.changed();}
 }
 async open(workspace:string,namespace:string,windowSlot?:string){const ticket=++this.ticket;this.busy=true;this.reading=null;this.changed();try{const value=await this.list(workspace,namespace,windowSlot);if(ticket!==this.ticket)return;this.catalogue=value;this.notice='Preview a private session. Recovery adds drafts; it never replaces open editors or saves source.';}catch(e){if(ticket===this.ticket)this.notice=String(e);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}}
 async read(row:EditorRecoverySession,workspace:string,namespace:string,lease:()=>boolean){const ticket=++this.ticket;this.busy=true;this.reading=null;this.changed();try{
  admit(!row.current_workspace||row.namespace===namespace);const value=await this.send<EditorRecoveryEnvelope>('GET','/api/workspace/recovery/'+row.client_id+'?revision='+row.revision+'&window_slot='+encodeURIComponent(row.window_slot));
  admit(value.version===1&&value.recovery_protocol===1&&value.engine_workspace_id===workspace&&value.namespace===row.namespace&&value.workspace_id===row.workspace_id&&value.client_id===row.client_id&&value.window_slot===row.window_slot&&value.revision===row.revision&&value.request_id===row.request_id&&value.snapshot_hash===row.snapshot_hash&&value.source_effects===false&&value.source_saved===false&&typeof value.material==='string'&&value.material.length<=64*1024*1024);
  admit(await historySourceHash(value.material)===value.snapshot_hash,'The complete recovery material fingerprint changed.');const snapshot:unknown=parseProtocolJson(value.material,{bytes:64*1024*1024,depth:64,containers:65536});validateEditorRecoverySnapshot(snapshot);admit(snapshot.host===row.host&&snapshot.buffers.length===row.buffers);
  admit(Array.isArray(value.authority)&&value.authority.length===snapshot.buffers.length&&snapshot.buffers.every(b=>value.authority.filter(a=>a.buffer_id===b.id&&a.write_permission===false&&typeof a.baseline_current==='boolean').length===1));
  if(ticket!==this.ticket)return;admit(lease(),'Open editor state changed while reading recovery. Preview again.');this.reading={envelope:value,snapshot,current:lease};this.notice='Complete checkpoint verified. Baseline observations grant no write permission; recovered sources require guarded Save As.';
 }catch(e){if(ticket===this.ticket)this.notice=String(e);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}}
 restore(apply:(snapshot:EditorRecoverySnapshot)=>void){const held=this.reading;if(this.busy||!held?.current()){this.notice='The host editor lease changed. Preview again; open drafts were preserved.';this.changed();return false;}try{apply(structuredClone(held.snapshot));this.reading=null;this.notice='Recovered draft copies and layout. Existing editors and saved source files were retained.';this.changed();return true;}catch(e){this.notice=String(e);this.changed();return false;}}
}
</ui:copy>

## Project a restored session into new draft identities

The original editor roster is never a replacement target. Allocate all recovered buffers and validate the complete layout before mutating the workspace. Restored source text receives a fresh unnamed identity and no physical save guard, even when an old baseline still matches. Generated text remains read-only. Retain the old baseline only as comparison material. Saved source positions and proportions are projected into a new pane subtree; existing utility instances stay in their current panes rather than attaching one terminal twice.

<ui:copy id="editor-recovery-browser-adapter">
import {Buffer,Buffers} from './buffers';
import {editorForBuffer} from './Document';
import {groups,isGroup,sourceTab,tabIdentity,type Layout} from './layout';
import {validateEditorRecoverySnapshot,type EditorRecoverySnapshot,type EditorRecoveryBuffer} from './EditorRecovery';
export function captureEditorRecovery(buffers:Buffers,tree:Layout,focused:string,extra:Iterable<Buffer>=[],resolve?:(tab:string)=>Buffer|undefined):EditorRecoverySnapshot {
 const rows=[...new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra])];
 const captured:EditorRecoveryBuffer[]=rows.map(b=>{const view=editorForBuffer(b)?.view;if(view?.composing||view&&view.state.doc.toString()!==b.text)throw Error('Finish source attachment and composition before the next private editor checkpoint.');const named=buffers.values.get(b.file.path)===b&&!b.resourceDeleted&&!b.isUntitledDraft&&!b.transient;
  const address:EditorRecoveryBuffer['address']=b.readonly?{kind:'generated',path:b.file.path,owner:b.file.generated_by!}:named?{kind:'source',path:b.file.path}:b.isUntitledDraft?{kind:'unnamed',name:b.file.name}:{kind:'detached',name:b.file.name,original_path:b.file.path};
  const selection=view?.state.selection.main??b.recoveryView?.selection??{anchor:0,head:0};
  return {id:b.sourceId,address,text:b.text,baseline:(b.isUntitledDraft||b.transient||b.resourceDeleted)?b.recoveryBaseline??{text:b.base,save_guard:b.saveGuard??null}:{text:b.base,save_guard:b.saveGuard??null},view:{selection:{anchor:selection.anchor,head:selection.head},scroll_x:view?.scrollDOM.scrollLeft??b.recoveryView?.scroll_x??0,scroll_y:view?.scrollDOM.scrollTop??b.recoveryView?.scroll_y??0}};
 });
 const tabs:Record<string,string>={};for(const group of groups(tree))for(const tab of group.tabs){const id=tabIdentity(tab);if(id.kind==='source'){const b=resolve?.(tab)??buffers.values.get(id.key)??rows.find(b=>b.sourceId===id.key);if(b)tabs[tab]=b.sourceId;}}
 return {host:'browser',buffers:captured,layout:{tree,focused,tabs}};
}
export function editorRecoveryLease(buffers:Buffers,tree:()=>Layout,focused:()=>string,extra:()=>Iterable<Buffer>=()=>[]):()=>boolean {
 const workspace=buffers.workspace,runtime=buffers.workspaceId,layout=tree(),focus=focused(),rows=[...new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra()])];
 const held=rows.map(b=>{const editor=editorForBuffer(b);return {b,file:b.file,text:b.text,base:b.base,guard:b.saveGuard,revision:b.sourceRevision,view:editor?.view,selection:editor?.view.state.selection.toJSON(),interaction:editor?.interaction};});const addresses=[...buffers.values];
 return ()=>buffers.workspace===workspace&&buffers.workspaceId===runtime&&tree()===layout&&focused()===focus&&buffers.values.size===addresses.length&&addresses.every(([path,b])=>buffers.values.get(path)===b)&&new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra()]).size===rows.length&&rows.every(b=>[...buffers.values.values(),...buffers.retainedResources.values(),...extra()].includes(b))&&held.every(h=>{const editor=editorForBuffer(h.b);return h.b.file===h.file&&h.b.text===h.text&&h.b.base===h.base&&h.b.saveGuard===h.guard&&h.b.sourceRevision===h.revision&&!h.b.saving&&editor?.view===h.view&&editor?.interaction===h.interaction&&!editor?.view.composing&&JSON.stringify(editor?.view.state.selection.toJSON())===JSON.stringify(h.selection);});
}
export function restoreEditorRecovery(buffers:Buffers,snapshot:EditorRecoverySnapshot,existing:Layout):{tree:Layout;focused:string} {
 validateEditorRecoverySnapshot(snapshot);const prefix='recovered-'+crypto.randomUUID(),added=new Map<string,Buffer>(),ids=new Map<string,string>();
 for(const row of snapshot.buffers){const path=prefix+'/'+row.id,name='Recovered '+(row.address.kind==='source'||row.address.kind==='generated'?row.address.path:row.address.name);if(buffers.values.has(path))throw Error('Recovery destination identity was reused. Preview again.');const generated=row.address.kind==='generated';const b=new Buffer({path,name,dir:false,...(row.address.kind==='generated'?{generated_by:row.address.owner}:{})},row.text,'',buffers.changed,generated,buffers.workspace);b.isUntitledDraft=!generated;b.base='';b.recoveryView=structuredClone(row.view);b.recoveryBaseline=row.baseline?structuredClone(row.baseline):null;b.saveGuardReason='Recovered private session: choose guarded Save As; no source save authority was restored.';added.set(path,b);ids.set(row.id,path);}
 let tree:Layout={id:prefix+'-sources',tabs:[...added.keys()].map(sourceTab),active:added.size?sourceTab([...added.keys()][0]):null},focused=tree.id;
 if(snapshot.host==='browser'){
  const data=snapshot.layout as {tree?:Layout;focused?:string;tabs?:Record<string,string>};let count=0;const seen=new Set<string>();
  function project(node:Layout,depth=0):Layout {if(!node||typeof node!=='object'||depth>32||++count>512||typeof node.id!=='string'||seen.has(node.id))throw Error('Malformed captured pane tree. Open drafts were preserved.');seen.add(node.id);const id=prefix+'-'+node.id;
   if(isGroup(node)){if(!Array.isArray(node.tabs)||node.tabs.length>512||node.tabs.some(t=>typeof t!=='string'))throw Error('Malformed captured pane tabs.');const tabs=node.tabs.flatMap(t=>{const path=ids.get(data.tabs?.[t]??'');return path?[sourceTab(path)]:[]});return {id,tabs:[...new Set(tabs)],active:node.active&&data.tabs?.[node.active]&&ids.has(data.tabs[node.active])?sourceTab(ids.get(data.tabs[node.active])!):tabs[0]??null};}
   if(!['horizontal','vertical'].includes(node.axis)||!Number.isFinite(node.ratio)||node.ratio<.1||node.ratio>.9)throw Error('Malformed captured pane proportions.');return {id,axis:node.axis,ratio:node.ratio,first:project(node.first,depth+1),second:project(node.second,depth+1)};
  }
  if(data?.tree){tree=project(data.tree);const placed=new Set(groups(tree).flatMap(g=>g.tabs)),missing=[...added.keys()].map(sourceTab).filter(tab=>!placed.has(tab));if(missing.length){tree={id:prefix+'-unplaced-split',axis:'horizontal',ratio:.75,first:tree,second:{id:prefix+'-unplaced',tabs:missing,active:missing[0]}};}focused=data.focused&&seen.has(data.focused)?prefix+'-'+data.focused:groups(tree).find(g=>g.tabs.length)?.id??groups(tree)[0].id;}
 }
 // Commit only after every buffer and layout node has been prepared successfully.
 for(const [path,b] of added)buffers.values.set(path,b);buffers.changed();return {tree:{id:prefix+'-session',axis:'horizontal',ratio:.5,first:existing,second:tree},focused};
}
</ui:copy>

## Read and restore through a compact floating review

The review exposes retained window slots, session identity, complete text pages and explicit restoration. A changed editor lease retires Restore. Reading and automatic recording run independently; neither blocks opening source or invokes providers.

<ui:copy id="editor-recovery-browser-view">
import {useEffect,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import {EditorRecoveryModel,type EditorRecoverySnapshot} from './EditorRecovery';
export function EditorRecoveryView({model,workspace,namespace,lease,restore,close}:{model:EditorRecoveryModel;workspace:string;namespace:string;lease:()=>()=>boolean;restore:(snapshot:EditorRecoverySnapshot)=>void;close:()=>void}){
 const [,paint]=useReducer(n=>n+1,0),[page,setPage]=useState(0),[index,setIndex]=useState(0),panel=useRef<HTMLElement>(null);
 useEffect(()=>{void model.open(workspace,namespace).then(paint);panel.current?.focus();return()=>model.close();},[model,workspace,namespace]);
 const reading=model.reading,row=reading?.snapshot.buffers[index],text=row?.text??'',count=Math.max(1,Math.ceil(text.length/16000)),shown=Math.min(page,count-1);
 async function refresh(slot?:string){await model.open(workspace,namespace,slot);paint();}
 return createPortal(<section role="dialog" aria-label="Editor session recovery" tabIndex={-1} ref={panel} className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}><header><strong>Editor session recovery</strong><button onClick={close}>Close</button></header><p role="status">{model.notice}</p><p>Private checkpoints are draft recovery, not source saves. Restoration adds copies beside existing editors. Utility panes remain in place.</p><button disabled={model.busy} onClick={()=>void refresh(model.catalogue?.window_slot)}>Refresh sessions</button>{model.catalogue&&<label>Retained window<select aria-label="Retained window" value={model.catalogue.window_slot} disabled={model.busy} onChange={e=>void refresh(e.target.value)}>{model.catalogue.window_slots.map(slot=><option key={slot} value={slot}>{slot}</option>)}</select></label>}{model.catalogue?.notices.map((notice,i)=><p key={i}>{notice}</p>)}{model.catalogue?.sessions.map(session=><button key={session.client_id} disabled={model.busy} onClick={()=>{setIndex(0);setPage(0);void model.read(session,workspace,namespace,lease()).then(paint);}}>Read {session.host} · {session.buffers} buffers · {session.client_id} · revision {session.revision}</button>)}{reading&&<><label>Captured buffer<select aria-label="Captured buffer" value={index} onChange={e=>{setIndex(Number(e.target.value));setPage(0);}}>{reading.snapshot.buffers.map((b,i)=><option key={b.id} value={i}>{b.address.kind} · {'path' in b.address?b.address.path:b.address.name}</option>)}</select></label><pre style={{whiteSpace:'pre-wrap'}}>{text.slice(shown*16000,(shown+1)*16000)}</pre><p>Text page {shown+1}/{count} · {row?.address.kind==='generated'?'read-only generated text':'unnamed recovered draft; guarded Save As required'}</p><button disabled={!shown} onClick={()=>setPage(shown-1)}>Previous text</button><button disabled={shown+1>=count} onClick={()=>setPage(shown+1)}>Next text</button><button disabled={model.busy||!reading.current()} onClick={()=>{model.restore(restore);paint();}}>Recover complete session as draft copies</button></>}</section>,document.body);
}
export function editorRecoveryTransport<T>(method:'GET'|'POST',path:string,body?:unknown):Promise<T>{return request<T>(method,path,body);}
</ui:copy>

## Check lost replies and stale restoration independently

These checks exercise the exact request retry, material admission and host lease. They do not establish crash/reload or rendered geometry acceptance; the connected browser witness must cover those separately.

<ui:copy id="editor-recovery-browser-tests">
import {describe,it,expect} from 'vitest';
import {EditorRecoveryModel,validateEditorRecoverySnapshot,type EditorRecoverySnapshot,type EditorRecoverySession} from './EditorRecovery';
import {historySourceHash} from './SourceHistory';
import {Buffers,Buffer} from './buffers';
import {restoreEditorRecovery,editorRecoveryLease} from './EditorRecoveryAdapter';
import {initialLayout} from './layout';
const workspace='e'.repeat(32),namespace='/owned';
const snapshot=():EditorRecoverySnapshot=>({host:'browser',buffers:[{id:'b'.repeat(32),address:{kind:'source',path:'files'},text:'draft 😀',baseline:{text:'saved',save_guard:'original'},view:{selection:{anchor:2,head:3},scroll_x:0,scroll_y:42}}],layout:{}});
const meta=()=>({version:1,recovery_protocol:1,engine_workspace_id:workspace,workspace_id:'a'.repeat(64),namespace,window_slot:'0',current_window_slot:'0',window_slots:['0','1'],sessions:[],notices:[],next:null,source_effects:false});
async function fixture(){const value=snapshot(),material=JSON.stringify(value),hash=await historySourceHash(material);const row:EditorRecoverySession={client_id:'c'.repeat(32),window_slot:'0',workspace_id:'a'.repeat(64),namespace,current_workspace:true,revision:1,request_id:'d'.repeat(32),snapshot_hash:hash,host:'browser',buffers:1,source_effects:false};return {value,row,envelope:{...row,version:1,recovery_protocol:1,engine_workspace_id:workspace,material,source_saved:false,authority:[{buffer_id:'b'.repeat(32),baseline_current:true,write_permission:false,reason:null}]}};}
describe('private editor recovery',()=>{
 it('prepares recovered drafts without guards and keeps generated output read-only',()=>{const buffers=new Buffers(()=>{});buffers.workspace=namespace;buffers.workspaceId=workspace;const original=new Buffer({path:'files',name:'files',dir:false},'independent','hash',()=>{},false,namespace);buffers.values.set('files',original);const value=snapshot();value.buffers.push({...structuredClone(value.buffers[0]),id:'f'.repeat(32),address:{kind:'generated',path:'output.py',owner:'project.md'}});const result=restoreEditorRecovery(buffers,value,initialLayout());expect(buffers.values.get('files')).toBe(original);const added=[...buffers.values.values()].filter(b=>b!==original);expect(added).toHaveLength(2);expect(added.every(b=>b.saveGuard===undefined)).toBe(true);expect(added.find(b=>!b.readonly)?.isUntitledDraft).toBe(true);expect(added.find(b=>b.readonly)?.text).toBe('draft 😀');expect(result.tree).not.toEqual(initialLayout());expect(original.text).toBe('independent');});
 it('does not import any buffer if a later layout node is malformed',()=>{const buffers=new Buffers(()=>{}),value=snapshot();value.layout={tree:{id:'bad',axis:'horizontal',ratio:1,first:{},second:{}},tabs:{}};expect(()=>restoreEditorRecovery(buffers,value,initialLayout())).toThrow();expect(buffers.values.size).toBe(0);});
 it('retires a same-sized replacement of a retained owner',()=>{const buffers=new Buffers(()=>{}),old=new Buffer({path:'a',name:'a',dir:false},'draft','',()=>{}),tree=initialLayout();buffers.retainedResources.set('old',old);const held=editorRecoveryLease(buffers,()=>tree,()=> 'document-group');expect(held()).toBe(true);buffers.retainedResources.clear();buffers.retainedResources.set('new',new Buffer(old.file,old.text,'',()=>{}));expect(held()).toBe(false);});

 it('retries the same uncertain checkpoint before recording a newer capture',async()=>{const sent:unknown[]=[];let lose=true;const model:EditorRecoveryModel=new EditorRecoveryModel(async<T>(method:'GET'|'POST',_path:string,raw?:unknown):Promise<T>=>{if(method==='GET')return meta() as T;const body=raw as {request_id:string;base_revision:number};sent.push(structuredClone(raw));if(lose){lose=false;throw Error('lost reply');}return {version:1,recovery_protocol:1,client_id:model.client,window_slot:'0',namespace,workspace_id:'a'.repeat(64),request_id:body.request_id,revision:body.base_revision+1,snapshot_hash:'f'.repeat(64),source_effects:false,source_saved:false} as T;});await model.checkpoint(workspace,namespace,snapshot(),()=>true);const next=snapshot();next.buffers[0].text='newer';await model.checkpoint(workspace,namespace,next,()=>true);expect(sent[1]).toEqual(sent[0]);expect(model.revision).toBe(1);await model.checkpoint(workspace,namespace,next,()=>true);expect(sent[2]).not.toEqual(sent[0]);expect(model.revision).toBe(2);});
 it('does not send a capture after the connection lease retires',async()=>{let current=true,posts=0;const model=new EditorRecoveryModel(async<T>(method:'GET'|'POST')=>{if(method==='GET'){current=false;return meta() as T;}posts++;throw Error('unexpected');});await model.checkpoint(workspace,namespace,snapshot(),()=>current);expect(posts).toBe(0);});
 it('admits exact material then restores only under the captured host lease',async()=>{const f=await fixture();let current=true,applied=0;const model=new EditorRecoveryModel(async<T>()=>f.envelope as T);await model.read(f.row,workspace,namespace,()=>current);expect(model.reading?.snapshot).toEqual(f.value);current=false;expect(model.restore(()=>applied++)).toBe(false);expect(applied).toBe(0);current=true;expect(model.restore(value=>{expect(value).toEqual(f.value);applied++;})).toBe(true);expect(applied).toBe(1);expect(model.restore(()=>applied++)).toBe(false);});
 it('rejects changed complete material and forged publication authority',async()=>{for(const mode of ['material','authority','revision','namespace','source_saved']){const f=await fixture();if(mode==='material')f.envelope.material+=' ';if(mode==='authority')(f.envelope.authority[0] as {write_permission:boolean}).write_permission=true;if(mode==='revision')f.envelope.revision++;if(mode==='namespace')f.envelope.namespace='/foreign';if(mode==='source_saved')(f.envelope as {source_saved:boolean}).source_saved=true;const model=new EditorRecoveryModel(async<T>()=>f.envelope as T);await model.read(f.row,workspace,namespace,()=>true);expect(model.reading,mode).toBeNull();}});
 it('rejects duplicate inner keys even when their complete digest matches',async()=>{const f=await fixture();f.envelope.material=f.envelope.material.replace('"host":"browser"','"host":"browser","host":"browser"');const hash=await historySourceHash(f.envelope.material);f.row.snapshot_hash=hash;f.envelope.snapshot_hash=hash;const model=new EditorRecoveryModel(async<T>()=>f.envelope as T);await model.read(f.row,workspace,namespace,()=>true);expect(model.reading).toBeNull();expect(model.notice).toContain('Duplicate');});
 it('refuses duplicate identities, scalar splits and invalid scroll before storage',()=>{for(const mode of ['duplicate','scalar','scroll']){const value=snapshot();if(mode==='duplicate')value.buffers.push(value.buffers[0]);if(mode==='scalar')value.buffers[0].view.selection.head=7;if(mode==='scroll')value.buffers[0].view.scroll_y=NaN;expect(()=>validateEditorRecoverySnapshot(value),mode).toThrow();}});
});
</ui:copy>

## Drive a real renderer reload against private engine storage

Wait for a checkpoint whose exact text and view match the live editors before reloading. Recover beside an independently edited source in the new renderer, then check retained text, selection, scroll, Undo and unchanged disk. This is controlled browser acceptance; physical input and accessibility still need their own acceptance.

<ui:copy id="editor-recovery-browser-witness">
"""Owned browser renderer-reload and private session recovery acceptance."""
import importlib.util,json,os,socket,tempfile,time,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-editor-session-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();saved=''.join(f'line {i} = {i}\n' for i in range(150));(root/'files').write_text(saved);(root/'note.py').write_text('value=1\n');processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
   driver.act('open',path='files');driver.act('source-tests-edit',path='files',text='\nunsaved λ😀');draft=driver.act('construction-source',path='files');position=driver.act('editor-position',path='files',anchor=9,head=12,y=180)
   note=driver.act('session-new-note',text='anonymous retained note');driver.act('split');driver.act('open',path='files')
   def get(path):
    with urllib.request.urlopen(backend+path,timeout=5) as response:return json.load(response)
   deadline=time.monotonic()+25;captured=None
   while time.monotonic()<deadline:
    for row in get('/api/workspace/recovery')['sessions']:
     value=get('/api/workspace/recovery/'+row['client_id']+'?revision='+str(row['revision'])+'&window_slot='+row['window_slot']);snapshot=json.loads(value['material']);named=next((b for b in snapshot['buffers'] if b['address']=={'kind':'source','path':'files'}),None)
     if named and named['text']==draft and named['view']['selection']=={'anchor':position['anchor'],'head':position['head']} and any(b['address']['kind']=='unnamed' and b['text']=='anonymous retained note' for b in snapshot['buffers']):captured=(row,value,snapshot);break
    if captured:break
    time.sleep(.1)
   assert captured,'Automatic checkpoint did not retain the source, unnamed note and view';row,value,snapshot=captured;assert (root/'files').read_text()==saved
   driver.act('session-reload');driver.act('open',path='note.py');driver.act('source-tests-edit',path='note.py',text='# independent new draft');independent=driver.act('construction-source',path='note.py');driver.act('remember-editor',path='note.py');selection=driver.act('editor-position',path='note.py',anchor=2,head=4)
   driver.act('session-open');preview=driver.act('session-read',client=row['client_id']);assert 'Complete checkpoint verified' in preview,preview;driver.act('editor-position',path='note.py',anchor=3);stale=driver.act('session-restore',stale=True);assert 'lease changed' in stale,stale;assert driver.act('construction-source',path='note.py')==independent;driver.act('session-close');selection=driver.act('editor-position',path='note.py',anchor=2,head=4);driver.act('session-open');driver.act('session-read',client=row['client_id']);editors=driver.act('session-restore');assert any(e['text']==draft and e['anchor']==position['anchor'] and e['head']==position['head'] and abs(e['y']-next(b for b in snapshot['buffers'] if b['address'].get('path')=='files')['view']['scroll_y'])<=1 for e in editors),editors
   assert driver.act('editor-is-retained',path='note.py');assert driver.act('construction-source',path='note.py')==independent;assert driver.act('editor-position',path='note.py')['anchor']==selection['anchor'];assert (root/'files').read_text()==saved and (root/'note.py').read_text()=='value=1\n'
   driver.act('undo',path='note.py');assert driver.act('construction-source',path='note.py')=='value=1\n';assert not driver.act('debug-diagnostics')['errors']
   print('PASS owned renderer reload: automatic named/unnamed checkpoints; complete-seal preview; stale lease refusal; draft/layout/scroll recovery beside independent unsaved editor; retained identity/selection/Undo; saved source unchanged')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>

<ui:file path="apps/web/src/simple/EditorRecovery.ts"><ui:paste select="#editor-recovery-browser-model"/></ui:file>
<ui:file path="apps/web/src/simple/EditorRecoveryAdapter.ts"><ui:paste select="#editor-recovery-browser-adapter"/></ui:file>
<ui:file path="apps/web/src/simple/EditorRecoveryView.tsx"><ui:paste select="#editor-recovery-browser-view"/></ui:file>
<ui:file path="apps/web/src/simple/EditorRecovery.test.ts"><ui:paste select="#editor-recovery-browser-tests"/></ui:file>
<ui:file path="scripts/test-editor-session-browser.py"><ui:paste select="#editor-recovery-browser-witness"/></ui:file>
</ui:doc>
