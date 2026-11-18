export type EntryStamp=Readonly<{version:1;identity:string;generation:number;path:string|null}>;
export type EntrySelection=Readonly<{version:1;entry_protocol:1;workspace:string;namespace:string;entry:EntryStamp;guard:string;source_guard?:string;source_hash?:string;movable:boolean;source_effects:false}>;
export type FrozenEntrySelection=Readonly<{entryPath:string|null;proof:EntrySelection;current:()=>boolean}>;
export type EntryReadingLease={workspace:string;namespace:string;current:()=>boolean};
function object(value:unknown):Record<string,unknown>{if(!value||typeof value!=="object"||Array.isArray(value))throw new Error("Workspace entry response is malformed.");return value as Record<string,unknown>;}
function hex(value:unknown,size:number):value is string{return typeof value==="string"&&value.length===size&&/^[0-9a-f]+$/.test(value);}
function entryStamp(value:unknown):EntryStamp {
 const entry=object(value),path=entry.path;
 if(Object.keys(entry).some(key=>!["version","identity","generation","path"].includes(key))||entry.version!==1||!hex(entry.identity,32)||typeof entry.generation!=="number"||!Number.isInteger(entry.generation)||entry.generation<0||entry.generation>0xffffffff)throw new Error('Workspace entry stamp is malformed.');
 if(path!==null&&(typeof path!=="string"||!path||path.includes('\\')||path.includes(':')||path.split('/').some(part=>!part||part==='.'||part==='..')||/[\u0000-\u001f\u007f]/.test(path)))throw new Error("Workspace entry path is outside the supported source namespace.");
 return Object.freeze({version:1,identity:entry.identity,generation:entry.generation,path});
}
function sameEntry(left:EntryStamp,right:EntryStamp):boolean {
 return left.identity===right.identity&&left.generation===right.generation&&left.path===right.path;
}
export function parseEntrySelection(value:unknown):EntrySelection{
 const body=object(value),stamp=entryStamp(body.entry),path=stamp.path;
 if(Object.keys(body).some(key=>!["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].includes(key)))throw new Error("Workspace entry response has unsupported fields.");
 if(body.version!==1||body.entry_protocol!==1||!hex(body.workspace,32)||typeof body.namespace!=="string"||!body.namespace||!hex(body.guard,64)||body.source_effects!==false||typeof body.movable!=="boolean")throw new Error("Workspace entry response has no supported exact authority.");
 if(path===null){if(body.source_guard!==undefined||body.source_hash!==undefined||body.movable)throw new Error("An absent entry cannot carry source authority.");}
 else if(typeof body.source_guard!=="string"||!body.source_guard||new TextEncoder().encode(body.source_guard).length>16384||typeof body.source_hash!=="string"||!/^sha256:[0-9a-f]{64}$/.test(body.source_hash))throw new Error("Workspace entry lacks an original physical source proof.");
 return Object.freeze({version:1 as const,entry_protocol:1 as const,workspace:body.workspace,namespace:body.namespace,entry:stamp,guard:body.guard,...(path===null?{}:{source_guard:body.source_guard as string,source_hash:body.source_hash as string}),movable:body.movable,source_effects:false as const});
}
export async function readFrozenEntrySelection(load:()=>Promise<unknown>,lease:EntryReadingLease):Promise<FrozenEntrySelection>{
 if(!lease.current())throw new Error("Original source context changed before entry acquisition.");
 const proof=parseEntrySelection(await load());
 if(!lease.current()||proof.workspace!==lease.workspace||proof.namespace!==lease.namespace)throw new Error("Workspace entry response belongs to an obsolete source context.");
 return Object.freeze({entryPath:proof.entry.path,proof,current:()=>lease.current()});
}

export type EntryOperation =
 | {kind:'move';destination:string}
 | {kind:'restoration';recovery_id:string;recovery_fingerprint:string};
export type EntryIntent = Readonly<{context:string;token:string;title:string;edit:unknown;before:EntryStamp;after:EntryStamp}>;

function parseEntryIntent(value:unknown,proof:EntrySelection,operation:EntryOperation,expectedPath:string|null):EntryIntent {
 const body=object(value),before=entryStamp(body.before_entry),after=entryStamp(body.after_entry);
 if(body.version!==1||body.workspace!==proof.workspace||body.namespace!==proof.namespace||
    body.kind!==operation.kind||body.source_files_modified!==false||!hex(body.token,64)||body.context!==body.token||
    !sameEntry(before,proof.entry)||after.identity!==before.identity||after.generation!==before.generation+1||
    typeof body.title!=='string'||!body.title||after.path!==expectedPath) {
  throw new Error('The entry review did not retain the requested construction and address.');
 }
 const edit=JSON.stringify(object(body.edit));
 return Object.freeze({context:body.token,token:body.token,title:body.title,before,after,get edit(){return JSON.parse(edit);}});
}

export async function prepareEntryIntent(selection:FrozenEntrySelection,operation:EntryOperation,load:(body:unknown)=>Promise<unknown>,expectedPath:string|null,current:()=>boolean):Promise<EntryIntent> {
 const check=()=>{if(!selection.current()||!current())throw new Error('The original workspace entry or source review changed. Open the move again.');};
 check();
 const reply=await load({version:1,entry_selection:selection.proof,operation});
 check();
 return parseEntryIntent(reply,selection.proof,operation,expectedPath);
}

export function readEntryTransition(value:unknown,recovery:string):{before:EntryStamp;after:EntryStamp}|null {
 if(value===null||value===undefined)return null;
 const body=object(value);
 if(body.version===1&&body.state==='unavailable'&&typeof body.notice==='string'&&body.notice)throw new Error(body.notice);
 if(body.version!==1||body.state!=='retained'||body.request_id!==recovery||!hex(body.manifest_sha256,64)||
    typeof body.kind!=='string'||!['move','restoration','create','repair'].includes(body.kind))throw new Error('This recovery record lacks a supported entry transition.');
 const before=entryStamp(body.before_entry),after=entryStamp(body.after_entry);
 if(before.identity!==after.identity||before.generation+1!==after.generation)throw new Error('The retained entry transition is inconsistent.');
 return {before,after};
}

export function requireEntryTransitionCurrent(selection:FrozenEntrySelection,after:EntryStamp):void {
 if(!sameEntry(selection.proof.entry,after))throw new Error('This recovery belongs to an earlier main construction generation. Restore the latest transition first.');
}

export function confirmEntryOutcome(value:unknown,request:string,intent:EntryIntent|undefined):void {
 if(!intent){if(value!==undefined)throw new Error('Unexpected main construction publication.');return;}
 const body=object(value);
 if(body.version!==1||body.state!=='confirmed'||body.request_id!==request||!sameEntry(entryStamp(body.before_entry),intent.before)||
    !sameEntry(entryStamp(body.after_entry),intent.after)||!sameEntry(entryStamp(body.current_entry),intent.after))throw new Error('The original main construction publication is unconfirmed.');
}
