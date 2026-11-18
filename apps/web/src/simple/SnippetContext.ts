import {supportedSnippetTransformPrograms} from './SnippetTransform';
import {sha256} from '../lib/reviews';
export interface SnippetClipboard {readonly available:boolean;readonly text:string|null}
export interface SnippetVariableSnapshot {readonly clipboard:Readonly<SnippetClipboard>|null;readonly clock:Readonly<{unix_millis:number;utc_offset_minutes:number}>|null;readonly random_hex:string|null}
export function captureSnippetVariables():SnippetVariableSnapshot {
 const now=new Date();let random_hex:string|null=null;try{const bytes=globalThis.crypto?.getRandomValues(new Uint8Array(16));if(bytes)random_hex=Array.from(bytes,b=>b.toString(16).padStart(2,'0')).join('');}catch{}
 const clock=Number.isFinite(now.getTime())?Object.freeze({unix_millis:now.getTime(),utc_offset_minutes:-now.getTimezoneOffset()}):null;return Object.freeze({clock,random_hex,clipboard:null});
}
function boundedClipboardText(value:string):boolean{if(value.length>1048576)return false;let bytes=0;for(let at=0;at<value.length;at++){const unit=value.charCodeAt(at);if(unit>=0xD800&&unit<=0xDBFF){const low=value.charCodeAt(++at);if(!(low>=0xDC00&&low<=0xDFFF))return false;bytes+=4;}else{if(unit>=0xDC00&&unit<=0xDFFF)return false;bytes+=unit<=0x7f?1:unit<=0x7ff?2:3;}if(bytes>1048576)return false;}return true;}
export async function captureSnippetClipboard():Promise<Readonly<SnippetClipboard>>{try{const value=await globalThis.navigator?.clipboard?.readText();if(typeof value==='string'&&boundedClipboardText(value))return Object.freeze({available:true,text:value});}catch{}return Object.freeze({available:false,text:null});}
export function validSnippetContextSelection(source:string,from:number,to:number):boolean {
 const boundary=(at:number)=>Number.isSafeInteger(at)&&at>=0&&at<=source.length&&!(at>0&&at<source.length&&(source.charCodeAt(at-1)>=0xD800&&source.charCodeAt(at-1)<=0xDBFF&&source.charCodeAt(at)>=0xDC00&&source.charCodeAt(at)<=0xDFFF||source[at-1]==='\r'&&source[at]==='\n'));
 return from<=to&&boundary(from)&&boundary(to)&&!/[\uD800-\uDFFF]/u.test(source);
}
export async function bindSnippetVariables(source:string,from:number,to:number,proof:unknown,snapshot:SnippetVariableSnapshot=captureSnippetVariables()):Promise<Record<string,unknown>> {
 if(!validSnippetContextSelection(source,from,to)||!proof||typeof proof!=='object'||Array.isArray(proof))throw Error('Snippet source context is unavailable. Request completions again.');const expected=proof as Record<string,unknown>;
 if(typeof expected.source_hash!=='string'||!/^sha256:[a-f0-9]{64}$/.test(expected.source_hash)||typeof expected.source_revision!=='string'||! /^[a-f0-9]{64}$/.test(expected.source_revision)||expected.source_hash!=='sha256:'+await sha256(source))throw Error('Snippet source context changed. Request completions again.');
 return Object.freeze({version:1,expected:Object.freeze({source_hash:expected.source_hash,source_revision:expected.source_revision}),selection:Object.freeze({from,to}),clock:snapshot.clock,random_hex:snapshot.random_hex,clipboard:snapshot.clipboard,transform_program_versions:supportedSnippetTransformPrograms});
}

function snippetRecord(value:unknown):value is Record<string,unknown>{return !!value&&typeof value==='object'&&!Array.isArray(value);}
export async function resolveSnippetCompletion<T extends {label:string;data?:unknown}>(original:T,adapter:{source:string;from:number;to:number;current:()=>boolean;request:(item:T)=>Promise<T>;clipboard?:()=>Promise<Readonly<SnippetClipboard>>}):Promise<T>{
 const current=()=>{if(!adapter.current())throw Error('Source, cursor or provider changed. Request completions again.');};current();
 if(!snippetRecord(original.data)||!Object.prototype.hasOwnProperty.call(original.data,'hickory_context')){const result=await adapter.request(original);current();return result;}
 let snapshot=captureSnippetVariables(),item=original;
 for(let attempt=0;attempt<2;attempt++){
  current();if(!snippetRecord(item.data))throw Error('Completion context proof is unavailable. Request again.');const bound=await bindSnippetVariables(adapter.source,adapter.from,adapter.to,item.data.hickory_context,snapshot);current();
  const result=await adapter.request({...item,data:{...item.data,hickory_context:bound}} as T);current();const data=snippetRecord(result.data)?result.data:{};const expected=bound.expected as Record<string,unknown>,proof=data.hickory_context;if(!snippetRecord(proof)||proof.source_hash!==expected.source_hash||proof.source_revision!==expected.source_revision)throw Error('Completion source context changed. Request again.');
  if(!Object.prototype.hasOwnProperty.call(data,'hickory_context_request'))return result;
  const request=data.hickory_context_request;if(attempt!==0||!snippetRecord(request)||Object.keys(request).length!==2||request.version!==1||!Array.isArray(request.variables)||request.variables.length!==1||request.variables[0]!=='CLIPBOARD')throw Error('Unsupported or repeated completion context request. No edits were applied.');
  const clipboard=await(adapter.clipboard??captureSnippetClipboard)();current();snapshot=Object.freeze({clock:snapshot.clock,random_hex:snapshot.random_hex,clipboard:Object.freeze({available:clipboard.available,text:clipboard.text})});item=result;
 }
 throw Error('Completion context is unavailable. No edits were applied.');
}
export function snippetContextNotice(item:unknown):string{const data=snippetRecord(item)&&snippetRecord(item.data)?item.data:{};if(!Object.prototype.hasOwnProperty.call(data,'hickory_context_unavailable'))return '';const values=data.hickory_context_unavailable;if(!Array.isArray(values)||values.length>40||!values.every(value=>typeof value==='string'&&value.length<=64))throw Error('Malformed completion context availability');return values.length?values.every(value=>value==='CLIPBOARD')?'Clipboard unavailable; its fallback or empty text was used.':'Some requested context values were unavailable; their fallback or empty text was used.':'';}
