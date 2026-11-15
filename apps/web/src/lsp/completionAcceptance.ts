import type {EditorView} from '@codemirror/view';
export interface CompletionAcceptance {job:string;phase:'resolving'|'applying'|'applied'|'unavailable';message:string;applied:boolean;source_changed:boolean;saved:false;verified:false}
interface PendingAcceptance {value:CompletionAcceptance;cancelled:boolean;promise:Promise<CompletionAcceptance>;update:(change:Partial<CompletionAcceptance>)=>void;cancel:(message:string)=>boolean}
const latest=new WeakMap<EditorView,PendingAcceptance>(),acceptingSignals=new WeakMap<EditorView,AbortSignal>();
export function withCompletionAcceptanceSignal<T>(view:EditorView,signal:AbortSignal,accept:()=>T):T {acceptingSignals.set(view,signal);try{return accept();}finally{acceptingSignals.delete(view);}}
export function readCompletionAcceptance(view:EditorView):CompletionAcceptance|null {const value=latest.get(view)?.value;return value?{...value}:null;}
export function waitCompletionAcceptance(view:EditorView,job:string):Promise<CompletionAcceptance> {const current=latest.get(view);return current?.value.job===job?current.promise:Promise.reject(new Error('Completion acceptance was superseded.'));}
export function cancelCompletionAcceptance(view:EditorView):boolean {return latest.get(view)?.cancel('Completion cancelled.')??false;}
export function beginCompletionAcceptance(view:EditorView,notice:(message:string)=>void=()=>{}) {
 const previous=latest.get(view);if(previous&&previous.value.phase==='resolving'){previous.cancelled=true;previous.update({phase:'unavailable',message:'Superseded by a newer completion.'});}
 const signal=acceptingSignals.get(view),value:CompletionAcceptance={job:crypto.randomUUID(),phase:'resolving',message:'Resolving completion…',applied:false,source_changed:false,saved:false,verified:false};
 let settle!:(value:CompletionAcceptance)=>void;
 const promise=new Promise<CompletionAcceptance>(resolve=>{settle=resolve;});
 const onAbort=()=>{entry.cancel('Completion cancelled.');};
 const entry:PendingAcceptance={value,cancelled:false,promise,update:change=>{Object.assign(value,change);if(value.phase==='applied'||value.phase==='unavailable'){signal?.removeEventListener('abort',onAbort);settle({...value});}},cancel:message=>{if(value.phase!=='resolving')return false;entry.cancelled=true;entry.update({phase:'unavailable',message});if(latest.get(view)===entry)notice(message);return true;}};
 latest.set(view,entry);signal?.addEventListener('abort',onAbort,{once:true});if(signal?.aborted)onAbort();
 return {job:value.job,cancelled:()=>entry.cancelled,current:()=>latest.get(view)===entry,update:(change:Partial<CompletionAcceptance>)=>{if(latest.get(view)===entry||change.phase==='applied'||change.phase==='unavailable')entry.update(change);}};
}
