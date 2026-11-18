export type EntryStamp=Readonly<{version:1;identity:string;generation:number;path:string|null}>;
export type EntrySelection=Readonly<{version:1;entry_protocol:1;workspace:string;namespace:string;entry:EntryStamp;guard:string;source_guard?:string;source_hash?:string;movable:boolean;source_effects:false}>;
export type FrozenEntrySelection=Readonly<{entryPath:string|null;proof:EntrySelection;current:()=>boolean}>;
export type EntryReadingLease={workspace:string;namespace:string;current:()=>boolean};
function object(value:unknown):Record<string,unknown>{if(!value||typeof value!=="object"||Array.isArray(value))throw new Error("Workspace entry response is malformed.");return value as Record<string,unknown>;}
function hex(value:unknown,size:number):value is string{return typeof value==="string"&&value.length===size&&/^[0-9a-f]+$/.test(value);}
export function parseEntrySelection(value:unknown):EntrySelection{
 const body=object(value),entry=object(body.entry),path=entry.path;
 if(Object.keys(body).some(key=>!["version","entry_protocol","workspace","namespace","entry","guard","source_guard","source_hash","movable","source_effects"].includes(key))||Object.keys(entry).some(key=>!["version","identity","generation","path"].includes(key)))throw new Error("Workspace entry response has unsupported fields.");
 if(body.version!==1||body.entry_protocol!==1||!hex(body.workspace,32)||typeof body.namespace!=="string"||!body.namespace||!hex(body.guard,64)||body.source_effects!==false||typeof body.movable!=="boolean"||entry.version!==1||!hex(entry.identity,32)||typeof entry.generation!=="number"||!Number.isInteger(entry.generation)||entry.generation<0||entry.generation>0xffffffff)throw new Error("Workspace entry response has no supported exact authority.");
 if(path!==null&&(typeof path!=="string"||!path||path.includes('\\')||path.includes(':')||path.split('/').some(part=>!part||part==='.'||part==='..')||/[\u0000-\u001f\u007f]/.test(path)))throw new Error("Workspace entry path is outside the supported source namespace.");
 if(path===null){if(body.source_guard!==undefined||body.source_hash!==undefined||body.movable)throw new Error("An absent entry cannot carry source authority.");}
 else if(typeof body.source_guard!=="string"||!body.source_guard||new TextEncoder().encode(body.source_guard).length>16384||typeof body.source_hash!=="string"||!/^sha256:[0-9a-f]{64}$/.test(body.source_hash))throw new Error("Workspace entry lacks an original physical source proof.");
 const stamp=Object.freeze({version:1 as const,identity:entry.identity,generation:entry.generation,path:path as string|null});
 return Object.freeze({version:1 as const,entry_protocol:1 as const,workspace:body.workspace,namespace:body.namespace,entry:stamp,guard:body.guard,...(path===null?{}:{source_guard:body.source_guard as string,source_hash:body.source_hash as string}),movable:body.movable,source_effects:false as const});
}
export async function readFrozenEntrySelection(load:()=>Promise<unknown>,lease:EntryReadingLease):Promise<FrozenEntrySelection>{
 if(!lease.current())throw new Error("Original source context changed before entry acquisition.");
 const proof=parseEntrySelection(await load());
 if(!lease.current()||proof.workspace!==lease.workspace||proof.namespace!==lease.namespace)throw new Error("Workspace entry response belongs to an obsolete source context.");
 return Object.freeze({entryPath:proof.entry.path,proof,current:()=>lease.current()});
}
