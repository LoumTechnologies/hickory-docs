import {api,request} from '../api/client';
import {capturedSourceData} from '../lib/capturedContexts';
import type {DebugBreakpoint,BreakpointStatus} from './client';
export interface DebugSourceReading {uri:string;workspace:string;namespace:string;identity:unknown;text:string;saved:boolean;current:()=>boolean;provenance?:string}
export interface DebugSourceProof extends DebugSourceReading {hash:string;lines:number}
export class DebugSourceSignal {revision=0;private listeners=new Set<()=>void>();subscribe=(listener:()=>void)=>{this.listeners.add(listener);return()=>{this.listeners.delete(listener);};};changed(){this.revision++;for(const listener of this.listeners)listener();}}
export interface DebugSourceCapability {read:(uri:string)=>DebugSourceReading|Promise<DebugSourceReading>;subscribe?:(listener:()=>void)=>()=>void}
export function debugSourcePath(uri:string):string {
 if(!uri.startsWith('hick:///')||uri.includes('?')||uri.includes('#'))throw Error('Debug source URI is not an exact encoded workspace path.');
 let path:string;try{path=decodeURIComponent(uri.slice('hick:///'.length));}catch{throw Error('Debug source URI is malformed.');}
 if(!path||path.startsWith('/')||path.includes('\\')||path.includes('\0')||path.split('/').some(part=>!part||part==='.'||part==='..'))throw Error('Debug source path is outside its workspace namespace.');return path;
}
export function debugUTF8(text:string):ReturnType<TextEncoder["encode"]> {
 for(const character of text)if(character.length===1&&character.charCodeAt(0)>=0xD800&&character.charCodeAt(0)<=0xDFFF)throw Error('Source contains an invalid Unicode scalar; save or reload its exact UTF-8 bytes before Debug.');
 return new TextEncoder().encode(text);
}
export async function stampDebugSource(reading:DebugSourceReading):Promise<DebugSourceProof> {
 if(!reading.workspace||!reading.namespace||!reading.saved||!reading.current())throw Error('Save and observe the current owned source before applying debugger effects.');
 const bytes=debugUTF8(reading.text),digest=await crypto.subtle.digest('SHA-256',bytes),hash=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('');
 if(!reading.current())throw Error('Source ownership or text changed while preparing Debug. Observe it again.');return {...reading,hash,lines:reading.text.split('\n').length};
}
export async function readSavedDebugSource(uri:string):Promise<DebugSourceReading> {
 const workspace=(await api.files()).root_path;if(!workspace)throw Error("The engine did not report an exact workspace namespace. Reload or update it before Debug.");
 if(uri.startsWith('hick-project:///')){
  const captured=capturedSourceData(uri);if(captured)return {...captured,uri,text:new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(captured.bytes),saved:true};
  if(uri.includes('?')||uri.includes('#'))throw Error('Captured source URI must preserve its exact encoded identity.');
  const source=await request<{bytes:number[]}>('GET',`/api/project-contexts/file?uri=${encodeURIComponent(uri)}`),bytes=new Uint8Array(source.bytes),text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);
  return {uri,workspace,namespace:`${workspace}\0${uri}`,identity:uri,text,saved:true,current:()=>true};
 }
 const path=debugSourcePath(uri),opened=await api.openFile(path),text=opened.doc_id?(await api.doc(opened.doc_id)).source:(await api.file(path)).content;
 return {uri,workspace,namespace:workspace,identity:`${workspace}\0${uri}`,text,saved:true,current:()=>true};
}
export const savedDebugSources:DebugSourceCapability={read:readSavedDebugSource};
export function sameDebugSource(left:DebugSourceReading,right:DebugSourceReading):boolean{return left.uri===right.uri&&left.workspace===right.workspace&&left.namespace===right.namespace&&left.identity===right.identity&&left.text===right.text;}
export interface DebugBreakpointRow extends DebugBreakpoint {state:'pending'|'bound'|'refused';message?:string;requested_line?:number}
export interface CapturedDebugIntent {reading:DebugSourceReading;marks:DebugBreakpoint[]}
interface Intent {reading:DebugSourceReading|null;marks:DebugBreakpoint[];revision:number;proof?:DebugSourceProof}
interface ReceiptTicket {source:string;revision:number;request:string;session:string|null;proof:DebugSourceProof;marks:DebugBreakpoint[]}
export interface ArchivedDebugIntent {source:string;namespace:string|null;hash:string|null;marks:DebugBreakpoint[]}
const cleanMarks=(marks:readonly DebugBreakpoint[]):DebugBreakpoint[]=>{
 if(marks.length>4096)throw Error('Too many breakpoint intentions for one source.');const lines=new Set<number>();
 return marks.map(mark=>{if(!Number.isInteger(mark.line)||mark.line<0||mark.line>4294967295||lines.has(mark.line))throw Error('Breakpoint intentions must have unique exact source lines.');lines.add(mark.line);
  const result:DebugBreakpoint={line:mark.line};for(const field of ['condition','hit_condition','log_message'] as const){const value=mark[field]?.trim();if(value){if(value.length>65536)throw Error('Breakpoint expression is too large.');result[field]=value;}}return result;});
};
export class DebugBreakpointModel {
 private intents=new Map<string,Intent>();private pending=new Map<string,ReceiptTicket>();private receipts=new Map<string,{ticket:ReceiptTicket;rows:DebugBreakpointRow[]}>();private revision=0;private refusals=new Map<string,{ticket:ReceiptTicket;rows:DebugBreakpointRow[]}>();
 private history:ArchivedDebugIntent[]=[];
 get archived():readonly ArchivedDebugIntent[]{return this.history.map(row=>({...row,marks:row.marks.map(mark=>({...mark}))}));}
 sources():string[]{return [...this.intents.keys()];}
 captured(source:string):CapturedDebugIntent|null {const intent=this.intents.get(source);return intent?.reading?{reading:intent.reading,marks:intent.marks.map(mark=>({...mark}))}:null;}
 adopt(source:string,reading:DebugSourceReading,captured:CapturedDebugIntent):boolean {
  const immutable=!!reading.provenance&&reading.provenance===captured.reading.provenance&&reading.workspace===captured.reading.workspace&&reading.namespace===captured.reading.namespace&&reading.identity===captured.reading.identity&&reading.text===captured.reading.text;
  if(!reading.current()||reading.uri!==source||!immutable&&(!captured.reading.current()||!sameDebugSource(reading,captured.reading)))return false;
  this.intents.set(source,{reading,marks:cleanMarks(captured.marks),revision:++this.revision});this.pending.delete(source);this.receipts.delete(source);this.refusals.delete(source);return true;
 }
 desired(source:string):DebugBreakpoint[]{return this.intents.get(source)?.marks.map(mark=>({...mark}))??[];}
 private archive(source:string,intent:Intent){if(intent.marks.length)this.history=[...this.history,{source,namespace:intent.reading?.namespace??null,hash:intent.proof?.hash??null,marks:intent.marks.map(mark=>({...mark}))}].slice(-32);}
 unknown(source:string,marks:readonly DebugBreakpoint[]):boolean {
  const previous=this.intents.get(source);if(previous?.reading)return false;this.intents.set(source,{reading:null,marks:cleanMarks(marks),revision:++this.revision});this.pending.delete(source);this.receipts.delete(source);this.refusals.delete(source);return true;
 }
 preferences(source:string,reading:DebugSourceReading,marks:readonly DebugBreakpoint[],provenance?:string):boolean {
  if(!provenance||reading.provenance!==provenance||reading.uri!==source||!reading.saved||!reading.current())return this.unknown(source,marks);
  if(this.intents.get(source)?.reading)return false;this.intents.set(source,{reading,marks:cleanMarks(marks),revision:++this.revision});this.pending.delete(source);this.receipts.delete(source);this.refusals.delete(source);return true;
 }
 change(source:string,reading:DebugSourceReading,transform:(marks:DebugBreakpoint[])=>DebugBreakpoint[]):void {
  if(reading.uri!==source||!reading.current())throw Error('Source changed before breakpoint placement. Observe it again.');const previous=this.intents.get(source),same=!!previous?.reading&&sameDebugSource(previous.reading,reading);
  if(previous&&!same)this.archive(source,previous);const marks=cleanMarks(transform(same?previous!.marks.map(mark=>({...mark})):[]));if(marks.some(mark=>mark.line>=reading.text.split('\n').length))throw Error('Breakpoint line is outside its captured source.');
  this.intents.set(source,{reading,marks,revision:++this.revision});this.pending.delete(source);this.receipts.delete(source);this.refusals.delete(source);
 }
 request(source:string,proof:DebugSourceProof,session:string|null,request:string=crypto.randomUUID()):ReceiptTicket|null {
  const intent=this.intents.get(source);if(!intent?.reading||!sameDebugSource(intent.reading,proof)||!proof.saved||!proof.current())return null;
  intent.proof=proof;const ticket={source,revision:intent.revision,request,session,proof,marks:intent.marks.map(mark=>({...mark}))};this.refusals.delete(source);this.pending.set(source,ticket);this.receipts.delete(source);return ticket;
 }
 bindLaunch(session:string,request:string){for(const [source,ticket] of this.pending)if(ticket.request===request&&ticket.session===null)this.pending.set(source,{...ticket,session});}
 admit(source:string,session:string,request:unknown,hash:unknown,statuses:unknown):DebugBreakpointRow[]|null {
  const ticket=this.pending.get(source)??this.receipts.get(source)?.ticket,intent=this.intents.get(source);if(!ticket||!intent||ticket.session!==session||ticket.request!==request||ticket.proof.hash!==hash||ticket.revision!==intent.revision||!ticket.proof.current()||!intent.reading||!sameDebugSource(intent.reading,ticket.proof)||!Array.isArray(statuses)||statuses.length!==ticket.marks.length)return null;
  const requested=new Map(ticket.marks.map(mark=>[mark.line,mark])),seen=new Set<number>(),rows:DebugBreakpointRow[]=[];
  for(const raw of statuses){if(!raw||typeof raw!=='object')return null;const status=raw as BreakpointStatus,mark=requested.get(status.line);
   if(!mark||seen.has(status.line)||!['pending','bound','refused'].includes(status.state)||status.moved_to!==undefined&&status.moved_to!==null&&(!Number.isInteger(status.moved_to)||status.moved_to<0||status.moved_to>=ticket.proof.lines)||status.message!==undefined&&status.message!==null&&typeof status.message!=='string')return null;
   seen.add(status.line);rows.push({...mark,line:status.moved_to??status.line,requested_line:status.line,state:status.state,...(status.message?{message:status.message}:{})});
  }
  const previous=this.receipts.get(source);if(previous&&JSON.stringify(previous.rows)===JSON.stringify(rows))return null;this.pending.delete(source);this.receipts.set(source,{ticket,rows});return rows;
 }
 acceptedProof(source:string):DebugSourceProof|null{return this.receipts.get(source)?.ticket.proof??null;}
 failed(source:string,session:string,request:unknown,message="Provider refused the breakpoint request."):boolean {const ticket=this.pending.get(source);if(!ticket||ticket.session!==session||ticket.request!==request)return false;this.pending.delete(source);this.receipts.clear();this.refusals.set(source,{ticket,rows:ticket.marks.map(mark=>({...mark,requested_line:mark.line,state:"refused",message}))});return true;}
 rows(source:string,reading:DebugSourceReading|null,session:string|null,runHash:string|null):DebugBreakpointRow[] {
  const intent=this.intents.get(source);if(!intent?.reading||!reading||!reading.saved||!reading.current()||!sameDebugSource(intent.reading,reading)||session&&(!intent.proof||intent.proof.hash!==runHash))return [];
  const receipt=this.receipts.get(source)??this.refusals.get(source);return receipt&&receipt.ticket.session===session&&receipt.ticket.revision===intent.revision&&receipt.ticket.proof.current()?receipt.rows.map(row=>({...row})):intent.marks.map(mark=>({...mark,state:'pending'}));
 }
 notice(source:string,reading:DebugSourceReading|null,session:string|null,hash:string|null):string|null {
  const intent=this.intents.get(source);if(!intent?.marks.length)return null;if(!intent.reading)return `${intent.marks.length} legacy breakpoint wishes have no captured namespace/source; place a current-source breakpoint to bind an intention.`;
  if(!reading||!reading.saved||!reading.current()||!sameDebugSource(intent.reading,reading))return 'Breakpoint intentions retained from older or unsaved source. Positional marks and provider effects are unavailable.';
  if(session&&intent.proof?.hash!==hash)return 'Breakpoint intentions belong to a different captured launch source. Stop and launch from saved source to apply them.';return null;
 }
}
