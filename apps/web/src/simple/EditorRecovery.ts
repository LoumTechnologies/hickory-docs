import {historySourceHash} from './SourceHistory';
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
  admit(await historySourceHash(value.material)===value.snapshot_hash,'The complete recovery material fingerprint changed.');const snapshot:unknown=JSON.parse(value.material);validateEditorRecoverySnapshot(snapshot);admit(snapshot.host===row.host&&snapshot.buffers.length===row.buffers);
  admit(Array.isArray(value.authority)&&value.authority.length===snapshot.buffers.length&&snapshot.buffers.every(b=>value.authority.filter(a=>a.buffer_id===b.id&&a.write_permission===false&&typeof a.baseline_current==='boolean').length===1));
  if(ticket!==this.ticket)return;admit(lease(),'Open editor state changed while reading recovery. Preview again.');this.reading={envelope:value,snapshot,current:lease};this.notice='Complete checkpoint verified. Baseline observations grant no write permission; recovered sources require guarded Save As.';
 }catch(e){if(ticket===this.ticket)this.notice=String(e);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}}
 restore(apply:(snapshot:EditorRecoverySnapshot)=>void){const held=this.reading;if(this.busy||!held?.current()){this.notice='The host editor lease changed. Preview again; open drafts were preserved.';this.changed();return false;}try{apply(structuredClone(held.snapshot));this.reading=null;this.notice='Recovered draft copies and layout. Existing editors and saved source files were retained.';this.changed();return true;}catch(e){this.notice=String(e);this.changed();return false;}}
}
