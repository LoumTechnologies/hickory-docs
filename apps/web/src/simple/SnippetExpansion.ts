import {sourceChangeOffset} from './ChangePlan';
import {completionInsertion,type CompletionExpansion} from './CompletionPlan';
import {utf16ToPosition} from '../lsp/positions';
export interface SnippetField {index:number;from:number;to:number;choices:string[];id?:number;parent?:number|null}
export function completionExpansion(item:unknown):CompletionExpansion|undefined {
 const value=item as {insertTextFormat?:unknown;data?:{hickory_snippet?:unknown}};
 if(value?.insertTextFormat!==2)return undefined;
 const raw=value.data?.hickory_snippet as {version?:unknown;syntax?:unknown;text?:unknown;fields?:unknown}|undefined;
 if(!raw||raw.version!==1||raw.syntax!==completionInsertion(item)||typeof raw.text!=='string'||new TextEncoder().encode(raw.text).length>1048576||!Array.isArray(raw.fields)||!raw.fields.length||raw.fields.length>4096)throw new Error('Snippet expansion unavailable or out of date. Request completions again.');
 const text=raw.text;let metadataBytes=0;
 const number=(value:unknown)=>{if(typeof value!=='number'||!Number.isSafeInteger(value)||value<0)throw new Error('Malformed snippet field');return value;};
 const fields:SnippetField[]=raw.fields.map(value=>{
  if(!value||typeof value!=='object')throw new Error('Malformed snippet field');
  const field=value as Record<string,unknown>,index=number(field.index),from=number(field.from),to=number(field.to);
  if(index>1004096||from>to||to>text.length||!Array.isArray(field.choices))throw new Error('Malformed snippet field');
  if(field.choices.length>4096||metadataBytes+64+field.choices.length*32>1048576)throw new Error('Snippet metadata expansion limit exceeded');
  metadataBytes+=64;for(const choice of field.choices){if(typeof choice!=='string')throw new Error('Malformed snippet field');metadataBytes+=32+new TextEncoder().encode(choice).length;if(metadataBytes>1048576)throw new Error('Snippet metadata expansion limit exceeded');}
  sourceChangeOffset(text,utf16ToPosition(text,from));sourceChangeOffset(text,utf16ToPosition(text,to));
  const ancestry:Pick<SnippetField,'id'|'parent'>={};
  if(field.id!==undefined){ancestry.id=number(field.id);ancestry.parent=field.parent===null?null:number(field.parent);if(ancestry.id>=4096||ancestry.parent!==null&&ancestry.parent>=4096)throw new Error('Malformed snippet ancestry');}
  else if(field.parent!==undefined)throw new Error('Malformed snippet ancestry');
  return {index,from,to,choices:field.choices as string[],...ancestry};
 });
 if(fields.some(field=>field.id!==undefined)){
  const ids=new Map(fields.map(field=>[field.id,field]));
  if(ids.has(undefined)||ids.size!==fields.length)throw new Error('Malformed snippet ancestry');
  const visited=new Set<number>();
  for(const field of fields){const chain=new Set<number>();let next:SnippetField|undefined=field;while(next&&!visited.has(next.id!)){if(chain.has(next.id!))throw new Error('Cyclic snippet ancestry');chain.add(next.id!);if(next.parent===null)break;const parent=ids.get(next.parent!);if(!parent||parent.from>next.from||parent.to<next.to)throw new Error('Malformed snippet ancestry');next=parent;}for(const id of chain)visited.add(id);}
 }
 if(fields.filter(field=>field.index===0).length!==1)throw new Error('Malformed snippet exit');
 const first=new Map<number,SnippetField>();
 for(const field of fields){const previous=first.get(field.index);if(previous&&(text.slice(previous.from,previous.to)!==text.slice(field.from,field.to)||JSON.stringify(previous.choices)!==JSON.stringify(field.choices)))throw new Error('Inconsistent snippet mirrors');first.set(field.index,field);}
 const active=[...first.keys()].filter(index=>index>0).sort((a,b)=>a-b)[0]??0,selected=fields.find(field=>field.index===active)!;
 return {text,fields,selection:{from:selected.from,to:selected.to}};
}
