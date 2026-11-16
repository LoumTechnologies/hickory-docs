export interface AdapterSourceScope {session:string;stop:number;thread:number;frame:number;reference:number}
export interface AdapterSourceEvent extends AdapterSourceScope {event:'adapter_source';request_id:string;content:string;source_hash:string;mime_type:string|null}
interface PendingSource {scope:AdapterSourceScope;request:string;current:()=>boolean}
export class AdapterSourceReading {
 value:AdapterSourceEvent|null=null;notice='';busy=false;private pending:PendingSource|null=null;private heldCurrent:()=>boolean=()=>false;
 constructor(private changed:()=>void=()=>{}){}
 close(){this.pending=null;this.busy=false;this.value=null;this.notice='';this.changed();}
 retire(){if(this.pending){this.pending=null;this.busy=false;this.notice='Adapter source request retired with its stopped context.';this.changed();}}
 current(){return this.value!==null&&this.heldCurrent();}
 read(scope:AdapterSourceScope,current:()=>boolean,send:(scope:AdapterSourceScope,request:string)=>void){
  if(!scope.session||!Number.isSafeInteger(scope.stop)||scope.stop<0||![scope.thread,scope.frame].every(value=>Number.isInteger(value)&&value>=-2147483648&&value<=2147483647)||!Number.isInteger(scope.reference)||scope.reference<=0||scope.reference>2147483647||!current()){this.notice='Select a current stopped frame with adapter-owned source.';this.changed();return;}
  const pending={scope:{...scope},request:crypto.randomUUID(),current};this.pending=pending;this.busy=true;this.notice='Reading literal adapter-owned source…';this.changed();
  try{send(pending.scope,pending.request);}catch(error){if(this.pending===pending){this.pending=null;this.busy=false;this.notice=String(error);this.changed();}}
 }
 async accept(input:AdapterSourceEvent){const event={...input};const pending=this.pending;if(!pending||event.request_id!==pending.request||!(['session','stop','thread','frame','reference'] as const).every(key=>event[key]===pending.scope[key]))return;
  try{if(typeof event.content!=='string'||new TextEncoder().encode(event.content).length>8*1024*1024||!/^[a-f0-9]{64}$/.test(event.source_hash)||event.mime_type!==null&&(typeof event.mime_type!=='string'||new TextEncoder().encode(event.mime_type).length>256||/[\u0000-\u001f\u007f]/.test(event.mime_type)))throw Error('Adapter source receipt is malformed or exceeds its reading budget.');
   const hash=[...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(event.content)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');
   if(this.pending!==pending)return;if(!pending.current())throw Error('Stopped frame changed before adapter source arrived.');if(hash!==event.source_hash)throw Error('Adapter source content fingerprint changed.');this.value=Object.freeze({...event});this.heldCurrent=pending.current;this.notice='Adapter-reported content only. No workspace source, edit or breakpoint authority is implied.';
  }catch(error){if(this.pending===pending)this.notice=String(error);}finally{if(this.pending===pending){this.pending=null;this.busy=false;this.changed();}}
 }
 failed(session:string|null|undefined,request:string|null|undefined,message:string){if(!this.pending||this.pending.scope.session!==session||this.pending.request!==request)return false;this.pending=null;this.busy=false;this.notice=message;this.changed();return true;}
}
